//! Item hand-overs between two talkers (`ac_handOverItem`): the item grows out
//! of the giver's left hand, is passed across, and the taker puts it away. Nook's
//! present and Joan's money bag and turnip bundles use it. The animation curves
//! are `rustimal_logic::hand_over_item`; the item models (`present_DL_*`,
//! `bag_DL_*`, `kabu_DL_*`) come from the disc, and the arm clips are the NPC
//! and player `transfer1`/`trans_wait1`/`get*` clips. The pocket transaction has
//! already been committed, so this is presentation only.
use std::collections::HashMap;

use bevy::prelude::*;
use rustimal_logic::hand_over_item as hoi;

use crate::character::{CharacterRig, NpcClip};
use crate::game_disc::GameDisc;
use crate::n64_material::N64Material;
use crate::player::Player;
use crate::shop::ShopLocation;
use crate::shop_nook::Shopkeeper;

/// One item-model unit in world units: the draw scales by 0.01 of an
/// original unit, and 40 original units make one world unit.
const MODEL_UNIT: f32 = 0.01 / 40.0;
/// Original translation units to world units.
const ORIGINAL_UNIT: f32 = 1.0 / 40.0;
/// Seconds the item waits in the giver's hand before the taker reaches for
/// it (the message script's order in the original).
const HOLD: f32 = 0.3;
/// Blend frames into each of the arm clips (`animation_init_proc`).
const MORPH: f32 = 8.0;

/// A talker's arm clips (`aNPC_ANIM_*` / `mPlayer_ANIM_*`).
struct Clips {
    transfer: &'static str,
    wait: &'static str,
    get: &'static str,
    pull: &'static str,
    away: &'static str,
}

const NPC_CLIPS: Clips = Clips {
    transfer: "cKF_ba_r_npc_1_transfer1",
    wait: "cKF_ba_r_npc_1_trans_wait1",
    get: "cKF_ba_r_npc_1_get1",
    pull: "cKF_ba_r_npc_1_get_pull1",
    away: "cKF_ba_r_npc_1_get_putaway1",
};

const PLAYER_CLIPS: Clips = Clips {
    transfer: "cKF_ba_r_ply_1_transfer1",
    wait: "cKF_ba_r_ply_1_trans_wait1",
    get: "cKF_ba_r_ply_1_get1",
    pull: "cKF_ba_r_ply_1_get_pull1",
    away: "cKF_ba_r_ply_1_get_putaway1",
};

/// One of the two talkers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Party {
    Player,
    /// The shopkeeper at the counter.
    Nook,
    Npc(Entity),
}

/// What changes hands (`mNT_ITEM_TYPE_*` draw types).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Item {
    /// The wrapped present (`mNT_ITEM_TYPE_PRESENT`).
    Present,
    /// A bag of money (`mNT_ITEM_TYPE_BAG`).
    Money,
    /// A turnip bundle (`mNT_ITEM_TYPE_KABU`).
    Turnips,
}

impl Item {
    fn lists(self) -> [&'static str; 2] {
        match self {
            Item::Present => ["present_DL_mode", "present_DL_vtx"],
            Item::Money => ["bag_DL_mode", "bag_DL_vtx"],
            Item::Turnips => ["kabu_DL_mode", "kabu_DL_vtx"],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Request {
    pub item: Item,
    pub giver: Party,
    pub taker: Party,
}

impl Request {
    /// A present from Nook to the player.
    pub fn nook_present() -> Self {
        Self {
            item: Item::Present,
            giver: Party::Nook,
            taker: Party::Player,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// The giver is the master: `aHOI_REQUEST_TRANSFER`.
    Out,
    /// `aHOI_REQUEST_TRANS_WAIT`, until the taker reaches for it.
    Wait,
    /// The player is the master: `aHOI_REQUEST_GET`.
    Get,
    /// A villager is the master: `aHOI_REQUEST_GET_PULL`.
    Pull,
    /// `aHOI_REQUEST_PUTAWAY` (the item's `after_mode` 7), into the taker.
    Away,
}

struct Active {
    entity: Option<Entity>,
    request: Request,
    phase: Phase,
    count: f32,
    held: f32,
    played: bool,
}

#[derive(Resource, Default)]
pub(crate) struct HandOver {
    active: Option<Active>,
    pending: Option<Request>,
    models: HashMap<Item, Vec<(Handle<Mesh>, Handle<N64Material>)>>,
}

impl HandOver {
    /// `birth_proc`: the item starts from the giver's hand.
    pub(crate) fn start(&mut self, request: Request) {
        self.pending = Some(request);
    }

    /// The transfer is still running; the player stays put.
    pub(crate) fn busy(&self) -> bool {
        self.pending.is_some() || self.active.is_some()
    }
}

/// Starts one of a talker's arm clips (`once`, held at its end), or stands
/// them back up.
fn play(clips: &mut Query<&mut NpcClip>, entity: Entity, name: Option<&'static str>, once: bool) {
    if let Ok(mut clip) = clips.get_mut(entity) {
        match name {
            Some(name) => clip.start(name, once, MORPH),
            None => clip.stop(MORPH),
        }
    }
}

fn mode_of(phase: Phase) -> usize {
    match phase {
        Phase::Out => hoi::REQ_TRANSFER,
        Phase::Wait => hoi::REQ_TRANS_WAIT,
        Phase::Get => hoi::REQ_GET,
        Phase::Pull => hoi::REQ_GET_PULL,
        Phase::Away => hoi::REQ_PUTAWAY,
    }
}

/// Runs the transfer and keeps the item at the current master's hand.
#[allow(clippy::too_many_arguments)]
pub(crate) fn animate(
    mut commands: Commands,
    time: Res<Time>,
    disc: Option<Res<GameDisc>>,
    location: Res<State<ShopLocation>>,
    mut handover: ResMut<HandOver>,
    shopkeepers: Query<Entity, With<Shopkeeper>>,
    players: Query<Entity, With<Player>>,
    mut clips: Query<&mut NpcClip>,
    rigs: Query<(Entity, &CharacterRig, &ChildOf)>,
    transforms: Query<&Transform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let h = &mut *handover;
    let Some(request) = h.active.as_ref().map(|a| a.request).or(h.pending) else {
        return;
    };
    let resolve = |party: Party| match party {
        Party::Player => players.single().ok(),
        Party::Nook => shopkeepers.single().ok(),
        Party::Npc(entity) => Some(entity),
    };
    let nook_away =
        [request.giver, request.taker].contains(&Party::Nook) && *location != ShopLocation::Inside;
    let parties = resolve(request.giver)
        .zip(resolve(request.taker))
        .filter(|&(giver, taker)| {
            !nook_away && transforms.contains(giver) && transforms.contains(taker)
        });
    let Some((giver, taker)) = parties else {
        // A party left (the shop emptied, Joan went home): drop the transfer.
        if let Some(entity) = h.active.take().and_then(|a| a.entity) {
            commands.entity(entity).despawn();
        }
        h.pending = None;
        for party in [request.giver, request.taker] {
            if let Some(Ok(mut clip)) = resolve(party).map(|e| clips.get_mut(e)) {
                clip.stop(MORPH);
            }
        }
        return;
    };
    let Some(disc) = disc else {
        return;
    };
    let clips_of = |party: Party| {
        if party == Party::Player {
            &PLAYER_CLIPS
        } else {
            &NPC_CLIPS
        }
    };
    if let Some(request) = h.pending.take() {
        h.active = Some(Active {
            entity: None,
            request,
            phase: Phase::Out,
            count: 0.0,
            held: 0.0,
            played: false,
        });
    }
    let Some(active) = h.active.as_mut() else {
        return;
    };
    if active.entity.is_none() {
        let model = h.models.entry(request.item).or_insert_with(|| {
            let lists: Vec<u32> = request
                .item
                .lists()
                .iter()
                .filter_map(|name| disc.catalog.get(name).map(|s| s.offset))
                .collect();
            let sink = crate::disc_gfx::run_display_lists(&disc.rom, &lists);
            crate::disc_town::sink_meshes_scaled(
                &disc,
                &sink,
                None,
                false,
                &crate::disc_gfx::Segments::default(),
                1.0,
                (&mut meshes, &mut materials, &mut images),
            )
        });
        let root = commands
            .spawn((
                Name::new("hand-over item"),
                Transform::from_scale(Vec3::ZERO),
                Visibility::default(),
            ))
            .id();
        for (mesh, material) in model.iter() {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                ChildOf(root),
            ));
        }
        active.entity = Some(root);
        play(
            &mut clips,
            giver,
            Some(clips_of(request.giver).transfer),
            true,
        );
    }
    let Some(entity) = active.entity else { return };
    let giving = matches!(active.phase, Phase::Out | Phase::Wait);
    let (master, master_party, other) = if giving {
        (giver, request.giver, taker)
    } else {
        (taker, request.taker, giver)
    };
    let (Ok(master_at), Ok(other_at)) = (transforms.get(master), transforms.get(other)) else {
        return;
    };
    let hand = rigs.iter().find_map(|(root, rig, parent)| {
        (parent.parent() == master)
            .then(|| {
                rig.left_hand_point(
                    transforms.get(root).ok()?,
                    master_at,
                    &transforms,
                    // The NPC callback keeps the joint origin; the player's
                    // arm callback reaches along its hand (VecX 1100).
                    if master_party == Party::Player {
                        Vec3::X * 1100.0
                    } else {
                        Vec3::ZERO
                    },
                )
            })
            .flatten()
    });
    let Some(hand) = hand else { return };

    // Both talkers face each other: x runs toward the other, z across.
    let mut forward = (other_at.translation - master_at.translation).with_y(0.0);
    forward = if forward.length_squared() > 1e-6 {
        forward.normalize()
    } else {
        Vec3::NEG_Z
    };
    let across = Vec3::Y.cross(forward);

    // aHOI_anime_proc: half a count a 60 Hz frame, to the mode's maximum.
    let mode = mode_of(active.phase);
    let max = hoi::MAX_ANM[hoi::ITEM_TYPE_ITEM][mode];
    if active.phase == Phase::Wait {
        active.held += time.delta_secs();
    } else {
        active.count = (active.count + 0.5 * time.delta_secs() * 60.0).min(max);
    }
    if active.phase == Phase::Out && active.count > hoi::TRANSFER_SE_FRAME && !active.played {
        active.played = true;
        crate::sound::se(hoi::TRANSFER_SE);
    }
    let trans = hoi::calc_trans(hoi::ITEM_TYPE_ITEM, mode, active.count);
    let scale = hoi::calc_scale(hoi::ITEM_TYPE_ITEM, mode, active.count);
    let at = hand + (forward * trans[0] + Vec3::Y * trans[1] + across * trans[2]) * ORIGINAL_UNIT;
    commands.entity(entity).insert(
        Transform::from_translation(at).with_scale(Vec3::splat(scale.max(0.0) * MODEL_UNIT)),
    );

    // A villager takes with her own arm clips and the item waits for them
    // (`aNPC_act_get`); the player takes after a short hold.
    let taker_npc = request.taker != Party::Player;
    let taker_clip_done = clips.get(taker).is_ok_and(|clip| clip.done);
    let done = match active.phase {
        Phase::Out if active.count >= max => Some(Phase::Wait),
        Phase::Wait if taker_npc && active.held >= HOLD && taker_clip_done => Some(Phase::Pull),
        Phase::Wait if !taker_npc && active.held >= HOLD => Some(Phase::Get),
        Phase::Get if active.count >= max => Some(Phase::Away),
        Phase::Pull if taker_clip_done => Some(Phase::Away),
        Phase::Away if scale.abs() < hoi::F32_ZERO_EPS && active.count >= max => None,
        _ => return,
    };
    let taker_clips = clips_of(request.taker);
    match done {
        Some(next) => {
            info!("hand-over {:?} -> {}", request.item, mode_of(next));
            active.phase = next;
            active.count = 0.0;
            active.held = 0.0;
            match next {
                Phase::Wait => {
                    play(&mut clips, giver, Some(clips_of(request.giver).wait), false);
                    if taker_npc {
                        // aNPC_ACT_GET_STEP_GET: she reaches out while it waits.
                        play(&mut clips, taker, Some(taker_clips.get), true);
                    }
                }
                Phase::Get => {
                    // aNSC_pc_present_end_wait: back to standing.
                    play(&mut clips, giver, None, true);
                    play(&mut clips, taker, Some(taker_clips.get), true);
                }
                Phase::Pull => {
                    play(&mut clips, giver, None, true);
                    play(&mut clips, taker, Some(taker_clips.pull), true);
                }
                Phase::Away => play(&mut clips, taker, Some(taker_clips.away), true),
                Phase::Out => {}
            }
        }
        None => {
            commands.entity(entity).despawn();
            play(&mut clips, giver, None, true);
            play(&mut clips, taker, None, true);
            h.active = None;
        }
    }
}
