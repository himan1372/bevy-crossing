//! ac_house_clock: the wall clock inside the shop (SCENE_SHOP0), retail's
//! time display indoors.
//!
//! obj_clock_shop1 stands at (200, 40, 40) in the shop scene, scale 0.01. Its
//! cKF clip repeats at speed 0.5 and restarts on the shared 120-frame beat,
//! which also ticks NA_SE_12D at frames 15 and 45. Joints 5 and 6 hide their
//! own shapes; the short and long hand lists are drawn at those joints'
//! origins turned by `90deg - rad_hour/rad_min` (no yaw, no offset). Higher
//! tiers select their native rig, hand joints and authored position;
//! on the monthly raffle day (FUKUBIKI) shop.rs does not place it.
use super::*;
use crate::shop::ShopLocation;
use rustimal_logic::furniture_clock as native;

/// NA_SE_12D.
const TICK_SOUND: u16 = 0x12D;

struct Loaded {
    skeleton: Skeleton,
    animation: Animation,
    short: u32,
    long: u32,
    hand_joints: [usize; 2],
}

fn load(disc: &GameDisc, tier: u8) -> Option<Loaded> {
    let number = tier.min(3) + 1;
    let hand_joints = if number == 2 || number == 3 {
        [3, 4]
    } else {
        [5, 6]
    };
    let skeleton = Skeleton::read(
        &disc.rom,
        disc.catalog
            .get(&format!("cKF_bs_r_obj_clock_shop{number}"))?
            .offset,
    )?;
    let animation = Animation::read(
        &disc.rom,
        disc.catalog
            .get(&format!("cKF_ba_r_obj_clock_shop{number}"))?
            .offset,
        skeleton.joints.len(),
    )?;
    (skeleton.joints.len() > hand_joints[1]).then_some(())?;
    Some(Loaded {
        short: disc
            .catalog
            .get(&format!("obj_clock_shop{number}_short_model"))?
            .offset,
        long: disc
            .catalog
            .get(&format!("obj_clock_shop{number}_long_model"))?
            .offset,
        hand_joints,
        skeleton,
        animation,
    })
}

#[derive(Component)]
pub(crate) struct HouseClock {
    skeleton: Skeleton,
    animation: Animation,
    joints: Vec<Entity>,
    hands: [Entity; 2],
    frame: f32,
    phase: Option<f32>,
    hand_joints: [usize; 2],
}

/// Spawn the shop's wall clock in the interior at `origin`.
pub(crate) fn spawn_shop_clock(
    commands: &mut Commands,
    disc: &GameDisc,
    origin: Vec3,
    layout: crate::shop::layout::Layout,
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
        hand_joints,
    }) = load(disc, layout.tier)
    else {
        warn!("incomplete shop wall-clock assets");
        return;
    };
    let native_position = match layout.tier {
        0 => Vec3::new(200.0, 40.0, 40.0),
        1 => Vec3::new(240.0, 0.0, 40.0),
        2 => Vec3::new(160.0, 0.0, 40.0),
        _ => Vec3::new(240.0, 0.0, 120.0),
    };
    let root = commands
        .spawn((
            Name::new("shop wall clock"),
            crate::shop::ShopInterior,
            Transform::from_translation(origin - layout.centre() + native_position / UNIT)
                .with_scale(Vec3::splat(0.01 / UNIT)),
            Visibility::default(),
        ))
        .id();
    let lists = skeleton
        .joints
        .iter()
        .enumerate()
        .filter(|(i, _)| !hand_joints.contains(i))
        .filter_map(|(i, j)| Some((i, j.model?, j.flags & 1 != 0)))
        .collect();
    let segments = Segments::default();
    let joints = super::rigs::spawn_skinned(
        commands,
        disc,
        root,
        "shop clock joint",
        &skeleton,
        &animation,
        lists,
        None,
        &segments,
        (&mut *meshes, &mut *materials, &mut *images),
    );
    let hands = [
        ("shop clock hour hand", short),
        ("shop clock minute hand", long),
    ]
    .map(|(name, list)| {
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
            commands, disc, anchor, &sink, None, &segments, false, meshes, materials, images,
        );
        anchor
    });
    commands.entity(root).insert(HouseClock {
        skeleton,
        animation,
        joints,
        hands,
        frame: 1.0,
        phase: None,
        hand_joints,
    });
}

fn update(
    time: Res<Time>,
    shop: Option<Res<State<ShopLocation>>>,
    mut clocks: Query<(&mut HouseClock, &GlobalTransform)>,
    mut transforms: Query<&mut Transform>,
) {
    let inside = shop.is_some_and(|s| *s.get() == ShopLocation::Inside);
    let now = crate::clock::now();
    let (hour_angle, minute_angle) =
        native::hand_angles(now.hour, now.minute, (now.day_secs % 60) as u8);
    let radians = std::f32::consts::PI / 32768.0;
    for (mut clock, at) in &mut clocks {
        if !inside {
            // The actor only exists in the shop; ct reseeds the beat.
            clock.phase = None;
            continue;
        }
        let pulse = match clock.phase {
            None => {
                // House_Clock_Actor_ct: phase = game_frame % 120.
                clock.phase = Some((time.elapsed_secs() * 60.0).rem_euclid(120.0));
                native::ClockPulse::default()
            }
            Some(mut phase) => {
                let pulse = native::advance_beat(&mut phase, time.delta_secs());
                clock.phase = Some(phase);
                pulse
            }
        };
        clock.frame = if pulse.reset {
            1.0
        } else {
            native::advance_frame(clock.frame, clock.animation.frames, time.delta_secs())
        };
        if pulse.tick {
            crate::sound::se_at(TICK_SOUND, at.translation());
        }
        let pose = clock
            .animation
            .pose(clock.frame)
            .local_transforms(&clock.skeleton);
        for (&joint, local) in clock.joints.iter().zip(&pose) {
            if let Ok(mut transform) = transforms.get_mut(joint) {
                *transform = *local;
            }
        }
        let origins = super::cuckoo::joint_origins(&pose, &clock.skeleton);
        for (hand, joint, angle) in [
            (clock.hands[0], clock.hand_joints[0], hour_angle),
            (clock.hands[1], clock.hand_joints[1], minute_angle),
        ] {
            if let Ok(mut transform) = transforms.get_mut(hand) {
                let turn = 0x4000i16.wrapping_sub(angle);
                *transform = Transform::from_translation(origins[joint])
                    .with_rotation(Quat::from_rotation_z(f32::from(turn) * radians));
            }
        }
    }
}

pub(crate) struct HouseClockPlugin;

impl Plugin for HouseClockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}
