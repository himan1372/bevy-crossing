//! Transient storage demo: open -> wait for storage UI -> close.
//! Source: aMR_OpenCloseCommonMove and player kagu_{open,wait,close}_{d,k,h}1.
use super::*;
use crate::furniture::storage::StorageRig;
use crate::skeleton::{Animation, Skeleton};

struct Clips {
    open: Animation,
    wait: Animation,
    close: Animation,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Closet,
    Wardrobe,
    Drawers,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Opening,
    Waiting,
    ClosingWait,
    Closing,
}

struct Demo {
    owner: usize,
    kind: Kind,
    phase: Phase,
    ticks: f32,
    end: f32,
    at: Vec3,
    facing: Vec3,
    shut_sound: bool,
}

#[derive(Default)]
pub(super) struct Storage {
    clips: [Option<Clips>; 3], // closet d1, wardrobe k1, drawers h1
    active: Option<Demo>,
}

impl Storage {
    pub(super) fn load(&mut self, disc: Option<&GameDisc>) {
        self.clips = ["d1", "k1", "h1"].map(|suffix| {
            let disc = disc?;
            let n = disc
                .catalog
                .get("cKF_bs_r_boy_1")
                .and_then(|s| Skeleton::read(&disc.rom, s.offset))?
                .joints
                .len();
            let read = |name| {
                let symbol = disc
                    .catalog
                    .get(&format!("cKF_ba_r_ply_1_kagu_{name}_{suffix}"))?;
                Animation::read(&disc.rom, symbol.offset, n)
            };
            Some(Clips {
                open: read("open")?,
                wait: read("wait")?,
                close: read("close")?,
            })
        });
        if disc.is_some() && self.clips.iter().any(Option::is_none) {
            warn!(
                "original storage player clips unavailable; keeping immediate storage interaction"
            );
        }
    }

    pub(super) fn reset(&mut self) {
        self.active = None;
    }

    pub(super) fn busy(&self) -> bool {
        self.active.is_some()
    }

    pub(super) fn ready(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|a| a.phase == Phase::Waiting)
    }

    pub(super) fn begin(
        &mut self,
        owner: usize,
        kind: Kind,
        end: f32,
        at: Vec3,
        facing: Vec3,
    ) -> bool {
        if self.clips[kind as usize].is_none() {
            return false;
        }
        self.active = Some(Demo {
            owner,
            kind,
            phase: Phase::Opening,
            ticks: 0.0,
            end,
            at,
            facing,
            shut_sound: false,
        });
        // NA_SE_DRAWER_OPEN / NA_SE_FTR_DOOR_OPEN.
        crate::sound::se_at(if kind == Kind::Drawers { 0x42E } else { 0x42C }, at);
        true
    }

    pub(super) fn animation(&self) -> Option<(&Animation, f32, u8)> {
        let demo = self.active.as_ref()?;
        let clips = self.clips[demo.kind as usize].as_ref()?;
        let (clip, looping, phase) = match demo.phase {
            Phase::Opening => (&clips.open, false, 6),
            Phase::Waiting | Phase::ClosingWait => (&clips.wait, true, 7),
            Phase::Closing => (&clips.close, false, 8),
        };
        let elapsed = demo.ticks * 0.5;
        let span = (clip.frames as f32 - 1.0).max(1.0);
        let frame = 1.0
            + if looping {
                elapsed.rem_euclid(span)
            } else {
                elapsed.min(span)
            };
        Some((clip, frame, phase))
    }
}

#[allow(clippy::too_many_arguments)] // One demo owns player pose and modal focus.
pub(super) fn advance(
    mut room: ResMut<PlayerRoom>,
    current: Res<CurrentRoom>,
    state: Res<State<HouseState>>,
    time: Res<Time>,
    mut focus: ResMut<InputFocus>,
    ui: Res<PocketsUi>,
    pockets: Option<Res<crate::pockets_disc::PocketAssets>>,
    mut player: Query<(&mut Facing, &mut crate::locomotion::Locomotion), With<Player>>,
) {
    if !crate::house::is_player_room(&current, &state) {
        room.storage.reset();
        return;
    }
    let idle = matches!(room.flow, StorageFlow::Idle);
    let storage = &mut room.storage;
    let Some(demo) = storage.active.as_mut() else {
        return;
    };
    if let Ok((mut facing, mut loco)) = player.single_mut() {
        facing.dir = demo.facing;
        *loco = crate::locomotion::Locomotion::default();
    }
    demo.ticks += time.delta_secs() * 60.0;
    if demo.phase == Phase::Closing && !demo.shut_sound && demo.ticks >= demo.end + 1.0 {
        // NA_SE_DRAWER_SHUT / NA_SE_FTR_DOOR_CLOSE at the furniture stop.
        crate::sound::se_at(
            if demo.kind == Kind::Drawers {
                0x3A
            } else {
                0x42D
            },
            demo.at,
        );
        demo.shut_sound = true;
    }
    let Some(clips) = storage.clips[demo.kind as usize].as_ref() else {
        storage.reset();
        return;
    };
    match demo.phase {
        Phase::Opening
            if demo.ticks >= ((clips.open.frames as f32 - 1.0) * 2.0 + 2.0).max(demo.end + 2.0) =>
        {
            demo.phase = Phase::Waiting;
            demo.ticks = 0.0;
        }
        Phase::Waiting if idle => {
            demo.phase = Phase::ClosingWait;
            demo.ticks = 0.0;
        }
        Phase::ClosingWait if !ui.open && pockets.as_ref().is_none_or(|p| p.world_ready()) => {
            demo.phase = Phase::Closing;
            demo.ticks = 0.0;
            if demo.kind == Kind::Drawers {
                crate::sound::se_at(0x42F, demo.at); // NA_SE_DRAWER_CLOSE at closing start
            }
        }
        Phase::Closing
            if demo.ticks
                >= ((clips.close.frames as f32 - 1.0) * 2.0 + 2.0).max(demo.end + 2.0) =>
        {
            storage.reset();
            *focus = InputFocus::World;
            return;
        }
        _ => {}
    }
    if !matches!(demo.phase, Phase::Waiting) || idle {
        *focus = InputFocus::Furniture;
    }
}

pub(super) fn render(
    room: Res<PlayerRoom>,
    mut rigs: Query<(&RoomPiece, &mut StorageRig)>,
    mut transforms: Query<&mut Transform>,
) {
    for (piece, mut rig) in &mut rigs {
        let frame = room
            .storage
            .active
            .as_ref()
            .filter(|a| a.owner == piece.slot)
            .map_or(1.0, |a| match a.phase {
                // CommonMove case3/case7 call play twice at speed0.5 per tick;
                // the first moving tick is a single play in case2/case5.
                Phase::Opening => (1.0 + (a.ticks - 0.5).max(0.0)).min(a.end),
                Phase::Waiting | Phase::ClosingWait => a.end,
                Phase::Closing => (a.end - (a.ticks - 0.5).max(0.0)).max(1.0),
            });
        rig.pose(frame, &mut transforms);
    }
}
