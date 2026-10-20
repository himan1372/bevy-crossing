//! ac_sum_log_hatoclk: the cuckoo clock's hour/half-hour calls and its hands.
//!
//! The body is the skinned disc rig without joints 7 and 8, whose shapes
//! `aSumLogHatoclk_DwBefore` suppresses. `DwAfter` draws the short and long
//! hand lists at those joints' positions, turned only by the furniture's yaw
//! and `90deg - rad_hour/rad_min`, one game unit toward the front. The call
//! sequence runs while the room is shown, as the actor only exists there;
//! entering again reconstructs it (so a visit during minute 0 or 30 calls).
use super::*;
use crate::house::{CurrentRoom, HouseState};
use crate::player_room::{PlayerRoom, RoomPiece};
use rustimal_logic::furniture_clock::{self as native, Cuckoo};

const HOUR_JOINT: usize = 7;
const MINUTE_JOINT: usize = 8;
/// `Matrix_translate(0, 0, 1)` after the hand rotation, in model units
/// (furniture scale 0.01).
const HAND_FRONT: f32 = 100.0;
/// `sAdo_OngenTrgStart(0x133)`.
const CUCKOO_SOUND: u16 = 0x133;

pub(crate) fn is_cuckoo(index: u16) -> bool {
    index == 0xC1
}

struct Loaded {
    skeleton: Skeleton,
    animation: Animation,
    short: u32,
    long: u32,
}

fn load(disc: &GameDisc) -> Option<Loaded> {
    let skeleton = Skeleton::read(
        &disc.rom,
        disc.catalog.get("cKF_bs_r_int_sum_log_hatoclk")?.offset,
    )?;
    let animation = Animation::read(
        &disc.rom,
        disc.catalog.get("cKF_ba_r_int_sum_log_hatoclk")?.offset,
        skeleton.joints.len(),
    )?;
    (skeleton.joints.len() > MINUTE_JOINT).then_some(())?;
    Some(Loaded {
        short: disc.catalog.get("int_sum_log_hatoclk_short_model")?.offset,
        long: disc.catalog.get("int_sum_log_hatoclk_long_model")?.offset,
        skeleton,
        animation,
    })
}

pub(super) fn available(disc: &GameDisc, _profile: &Profile) -> bool {
    load(disc).is_some()
}

#[derive(Component)]
pub(crate) struct CuckooRig {
    skeleton: Skeleton,
    animation: Animation,
    joints: Vec<Entity>,
    hands: [Entity; 2],
    state: Option<Cuckoo>,
    ticks: f32,
}

pub(super) fn spawn(
    commands: &mut Commands,
    disc: &GameDisc,
    root: Entity,
    profile: &Profile,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) {
    let Some(Loaded {
        skeleton,
        animation,
        short,
        long,
    }) = load(disc)
    else {
        warn!("incomplete cuckoo-clock assets");
        return;
    };
    let lists = skeleton
        .joints
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != HOUR_JOINT && *i != MINUTE_JOINT)
        .filter_map(|(i, j)| Some((i, j.model?, j.flags & 1 != 0)))
        .collect();
    let segments = Segments::default();
    let joints = super::rigs::spawn_skinned(
        commands,
        disc,
        root,
        "cuckoo joint",
        &skeleton,
        &animation,
        lists,
        profile.palette,
        &segments,
        (&mut *meshes, &mut *materials, &mut *images),
    );
    let hands = [("cuckoo hour hand", short), ("cuckoo minute hand", long)].map(|(name, list)| {
        let anchor = commands
            .spawn((
                Name::new(name),
                Transform::default(),
                Visibility::default(),
                ChildOf(root),
            ))
            .id();
        let sink = run_display_lists(&disc.rom, &[list]);
        crate::interior::spawn_sink(
            commands,
            disc,
            anchor,
            &sink,
            profile.palette,
            &segments,
            false,
            meshes,
            materials,
            images,
        );
        anchor
    });
    commands.entity(root).insert(CuckooRig {
        skeleton,
        animation,
        joints,
        hands,
        state: None,
        ticks: 0.0,
    });
}

/// Joint origins in the furniture's model space (Matrix_Position of each).
pub(super) fn joint_origins(pose: &[Transform], skeleton: &Skeleton) -> Vec<Vec3> {
    let mut world: Vec<Transform> = Vec::with_capacity(pose.len());
    for (i, local) in pose.iter().enumerate() {
        let composed = match skeleton.parents[i] {
            Some(p) => world[p].mul_transform(*local),
            None => *local,
        };
        world.push(composed);
    }
    world.iter().map(|t| t.translation).collect()
}

pub(crate) fn update(
    time: Res<Time>,
    house: Res<State<HouseState>>,
    current: Res<CurrentRoom>,
    player_room: Res<PlayerRoom>,
    mut rigs: Query<(&mut CuckooRig, &GlobalTransform, Option<&RoomPiece>)>,
    mut transforms: Query<&mut Transform>,
) {
    let room = (*house.get() == HouseState::Inside).then_some(current.origin());
    let now = crate::clock::now();
    let (hour_angle, minute_angle) =
        native::hand_angles(now.hour, now.minute, (now.day_secs % 60) as u8);
    let radians = std::f32::consts::PI / 32768.0;
    for (mut rig, at, piece) in &mut rigs {
        let end = f32::from(rig.animation.frames);
        let shown = room.is_some_and(|room| at.translation().distance(room) < 8.0);
        if !shown {
            rig.state = None;
            rig.ticks = 0.0;
        } else {
            let mut state = rig.state.unwrap_or_else(|| Cuckoo::new(end));
            rig.ticks += time.delta_secs() * 60.0;
            let ticks = (rig.ticks as u32).min(4);
            rig.ticks = (rig.ticks - ticks as f32).min(1.0);
            for _ in 0..ticks {
                if state.tick(now.hour, now.minute, end) {
                    // aFTR_CAN_PLAY_SE: not while being born or removed.
                    if piece.is_none_or(|piece| player_room.clock_sound_allowed(piece)) {
                        crate::sound::se_at(CUCKOO_SOUND, at.translation());
                    }
                }
            }
            rig.state = Some(state);
        }
        let frame = rig.state.map_or(1.0, |s| s.frame);
        let pose = rig.animation.pose(frame).local_transforms(&rig.skeleton);
        for (&joint, local) in rig.joints.iter().zip(&pose) {
            if let Ok(mut transform) = transforms.get_mut(joint) {
                *transform = *local;
            }
        }
        let origins = joint_origins(&pose, &rig.skeleton);
        for (hand, joint, angle) in [
            (rig.hands[0], HOUR_JOINT, hour_angle),
            (rig.hands[1], MINUTE_JOINT, minute_angle),
        ] {
            let Ok(mut transform) = transforms.get_mut(hand) else {
                continue;
            };
            // DEG2SHORT_ANGLE(90) - rad, as a signed binary angle.
            let turn = 0x4000i16.wrapping_sub(angle);
            let rotation = Quat::from_rotation_z(f32::from(turn) * radians);
            *transform = Transform::from_translation(origins[joint] + Vec3::Z * HAND_FRONT)
                .with_rotation(rotation);
        }
    }
}
