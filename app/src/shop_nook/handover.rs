//! Nook's present hand-over (`ac_handOverItem`): the wrapped present grows out
//! of his left hand, is passed across, and the player puts it away. The
//! animation curves are `rustimal_logic::hand_over_item`; the present model
//! (`present_DL_mode`/`present_DL_vtx`) comes from the disc. The pocket
//! transaction has already been committed, so this is presentation only.
use bevy::prelude::*;
use rustimal_logic::hand_over_item as hoi;

use crate::character::CharacterRig;
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
/// Seconds the present waits in Nook's hand before the player takes it
/// (the message script's order in the original).
const HOLD: f32 = 0.3;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Nook is the master: `aHOI_REQUEST_TRANSFER`.
    Out,
    /// `aHOI_REQUEST_TRANS_WAIT`, until the player reaches for it.
    Wait,
    /// The player is the master: `aHOI_REQUEST_GET`.
    Get,
    /// `aHOI_REQUEST_PUTAWAY` (the present's `after_mode` 7), into the player.
    Away,
}

struct Active {
    entity: Option<Entity>,
    phase: Phase,
    count: f32,
    held: f32,
    played: bool,
}

#[derive(Resource, Default)]
pub(crate) struct HandOver {
    active: Option<Active>,
    pending: bool,
    model: Option<Vec<(Handle<Mesh>, Handle<N64Material>)>>,
}

impl HandOver {
    /// `birth_proc`: a present starts from Nook's hand.
    pub(super) fn start(&mut self) {
        self.pending = true;
    }

    /// The transfer is still running; the player stays put.
    pub(crate) fn busy(&self) -> bool {
        self.pending || self.active.is_some()
    }
}

fn mode_of(phase: Phase) -> usize {
    match phase {
        Phase::Out => hoi::REQ_TRANSFER,
        Phase::Wait => hoi::REQ_TRANS_WAIT,
        Phase::Get => hoi::REQ_GET,
        Phase::Away => hoi::REQ_PUTAWAY,
    }
}

/// Runs the transfer and keeps the present at the current master's hand.
#[allow(clippy::too_many_arguments)]
pub(crate) fn animate(
    mut commands: Commands,
    time: Res<Time>,
    disc: Option<Res<GameDisc>>,
    location: Res<State<ShopLocation>>,
    mut handover: ResMut<HandOver>,
    nook: Query<(Entity, &Transform), With<Shopkeeper>>,
    player: Query<(Entity, &Transform), With<Player>>,
    rigs: Query<(Entity, &CharacterRig, &ChildOf)>,
    transforms: Query<&Transform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let h = &mut *handover;
    if *location != ShopLocation::Inside {
        if let Some(entity) = h.active.take().and_then(|a| a.entity) {
            commands.entity(entity).despawn();
        }
        h.pending = false;
        return;
    }
    let (Some(disc), Ok((nook_entity, nook_at)), Ok((player_entity, player_at))) =
        (disc, nook.single(), player.single())
    else {
        return;
    };
    if h.pending {
        h.pending = false;
        h.active = Some(Active {
            entity: None,
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
        if h.model.is_none() {
            let lists: Vec<u32> = ["present_DL_mode", "present_DL_vtx"]
                .iter()
                .filter_map(|name| disc.catalog.get(name).map(|s| s.offset))
                .collect();
            let sink = crate::disc_gfx::run_display_lists(&disc.rom, &lists);
            h.model = Some(crate::disc_town::sink_meshes_scaled(
                &disc,
                &sink,
                None,
                false,
                &crate::disc_gfx::Segments::default(),
                1.0,
                (&mut meshes, &mut materials, &mut images),
            ));
        }
        let root = commands
            .spawn((
                Name::new("hand-over present"),
                Transform::from_scale(Vec3::ZERO),
                Visibility::default(),
            ))
            .id();
        for (mesh, material) in h.model.iter().flatten() {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                ChildOf(root),
            ));
        }
        active.entity = Some(root);
    }
    let Some(entity) = active.entity else { return };
    let master = if matches!(active.phase, Phase::Out | Phase::Wait) {
        nook_entity
    } else {
        player_entity
    };
    let (master_at, other_at) = if master == nook_entity {
        (nook_at, player_at)
    } else {
        (player_at, nook_at)
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
                    if master == nook_entity {
                        Vec3::ZERO
                    } else {
                        Vec3::X * 1100.0
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

    let done = match active.phase {
        Phase::Out if active.count >= max => Some(Phase::Wait),
        Phase::Wait if active.held >= HOLD => Some(Phase::Get),
        Phase::Get if active.count >= max => Some(Phase::Away),
        Phase::Away if scale.abs() < hoi::F32_ZERO_EPS && active.count >= max => None,
        _ => return,
    };
    match done {
        Some(next) => {
            active.phase = next;
            active.count = 0.0;
            active.held = 0.0;
        }
        None => {
            commands.entity(entity).despawn();
            h.active = None;
        }
    }
}
