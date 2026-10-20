//! The museum's insect room (ac_museum_insect): each donated insect as its
//! museum model (act_m_mu_*, the frames of its minsect_mdl table) at its
//! kind's spot: butterflies, the bee and the mosquito drift between the
//! flower beds (flower_pos), the purple emperor keeps to its tree, cicadas,
//! beetles, the bagworm and the spider sit on the trees (kabuto_base_pos,
//! tree_pos), dragonflies hover over their rocks (tonbo_rock_pos), ladybugs
//! and the mantis sit on flowers (tentou_flower_pos), the firefly hovers at
//! base_genji_pos, the pond skater glides about amenbo_center_pos and the
//! rest keep to the ground.
//!
//! In outline only: the original's per-kind behaviour (startling at the
//! player, active and resting hours, the firefly's glow) is not modelled.
use bevy::prelude::*;

use crate::disc_gfx::{run_display_lists, Segments};
use crate::game_disc::GameDisc;
use crate::museum::{room_origin, Museum, MuseumCollection};
use crate::n64_material::N64Material;

const UNIT: f32 = 40.0;
/// The insect room's index in the museum's rooms.
const INSECT_ROOM: usize = 3;

/// minsect_mdl: each insect's frames (act_m_mu_*).
const FRAMES: [&[&str]; 40] = [
    &["monshiro1T", "monshiro2T"],
    &["monki1T", "monki2T"],
    &["kiageha1T", "kiageha2T"],
    &["ohmurasaki1T", "ohmurasaki2T"],
    &["minmin1_1T", "minmin1_2T"],
    &["tukutuku1_1T", "tukutuku1_2T"],
    &["higurashi1_1T", "higurashi1_2T"],
    &["abura1_1T", "abura1_2T"],
    &["hachi1_1T", "hachi1_2T", "hachi2_2T"],
    &["shiokara1_1T", "shiokara1_2T", "shiokara2_2T"],
    &["akiakane1_1T", "akiakane1_2T", "akiakane2_2T"],
    &["ginyanma1_1T", "ginyanma1_2T", "ginyanma2_2T"],
    &["oniyanma1_1T", "oniyanma1_2T", "oniyanma2_2T"],
    &["syouryou1T", "syouryou2_2T"],
    &["tonosama1T", "tonosama2_2T"],
    &["koorogi1T", "koorogi2_2T"],
    &["kirigirisu1T", "kirigirisu2_2T"],
    &["suzumushi1T", "suzumushi2_2T"],
    &["matumushi1T", "matumushi2_2T"],
    &["kanabun1T"],
    &["kabuto1T"],
    &["hirata1T"],
    &["tamamushi1T"],
    &["gomadara1T"],
    &["tentou1T", "tentou2T"],
    &["nanahoshi1T", "nanahoshi2T"],
    &["kamakiri1T", "kamakiri2_2T"],
    &["genji2_a", "genji2_b", "genji2_c", "genji2_d"],
    &["danna1T", "danna2T"],
    &["nokogiri1T"],
    &["miyama1T"],
    &["okuwa1T"],
    &["maimaiT"],
    &["keraT"],
    &["amenbo1T", "amenbo2T"],
    &["mino1_1T", "mino1_2T", "mino2T"],
    &["dango1T", "dango2T"],
    &["kumo1_1T", "kumo1_2T", "kumo2T"],
    &["ariT"],
    &["ka1T", "ka2_1T", "ka2_2T"],
];

/// flower_pos.
const FLOWERS: [Vec3; 8] = [
    Vec3::new(147.0, 66.0, 169.0),
    Vec3::new(238.0, 66.0, 201.0),
    Vec3::new(264.0, 66.0, 164.0),
    Vec3::new(165.0, 66.0, 219.0),
    Vec3::new(181.0, 66.0, 235.0),
    Vec3::new(199.0, 66.0, 188.0),
    Vec3::new(327.5, 66.0, 154.0),
    Vec3::new(410.0, 66.0, 190.0),
];
/// kabuto_base_pos: the beetles' places on the trees.
const BEETLE_SPOTS: [Vec3; 9] = [
    Vec3::new(165.5, 78.5, 482.5),
    Vec3::new(389.5, 66.5, -14.5),
    Vec3::new(241.5, 67.0, -15.5),
    Vec3::new(165.0, 81.5, 0.5),
    Vec3::new(89.0, 64.0, -15.5),
    Vec3::new(466.0, 84.0, 1.5),
    Vec3::new(171.0, 64.5, 5.5),
    Vec3::new(319.0, 81.0, 1.0),
    Vec3::new(313.0, 63.0, 5.5),
];
/// tonbo_rock_pos.
const ROCKS: [Vec3; 6] = [
    Vec3::new(215.0, 68.0, 403.0),
    Vec3::new(232.0, 60.0, 381.0),
    Vec3::new(372.0, 65.0, 482.0),
    Vec3::new(390.0, 68.0, 469.0),
    Vec3::new(328.0, 72.0, 501.0),
    Vec3::new(351.0, 72.0, 501.0),
];
/// tentou_flower_pos.
const LADYBUG_FLOWERS: [Vec3; 4] = [
    Vec3::new(410.0, 65.0, 240.0),
    Vec3::new(213.5, 65.0, 155.0),
    Vec3::new(233.0, 65.0, 233.5),
    Vec3::new(294.0, 65.0, 170.5),
];

/// How an insect keeps its place.
#[derive(Clone, Copy, PartialEq)]
enum Way {
    /// Drifts from flower to flower at this height (base_high_tbl).
    Flower(f32),
    /// Hovers close about its spot.
    Hover,
    /// Sits still, flicking through its frames now and then.
    Sit,
    /// Glides about the pond (amenbo: +-25 x, +-18 z).
    Pond,
}

/// Each insect's spot (room-local game units; y NaN: on the ground).
fn spot(i: usize) -> (Vec3, Way) {
    let ground = |x: f32, z: f32| Vec3::new(x, f32::NAN, z);
    match i {
        0..=2 => (FLOWERS[i * 2], Way::Flower([85.0, 95.0, 90.0][i])),
        // ohmurasaki_tree_pos.
        3 => (Vec3::new(360.0, 85.0, 265.0), Way::Hover),
        // High on the trunks of tree_pos 0, 2, 4 and 5.
        4..=7 => {
            let (x, z) = [(92.0, -15.0), (244.0, -15.0), (392.0, -15.0), (468.0, 5.0)][i - 4];
            (Vec3::new(x, 100.0, z + 12.0), Way::Sit)
        }
        8 => (FLOWERS[6], Way::Flower(90.0)),
        9..=12 => (ROCKS[i - 9] + Vec3::Y * 22.0, Way::Hover),
        13..=18 => (
            ground(FLOWERS[i - 13].x + 18.0, FLOWERS[i - 13].z + 22.0),
            Way::Sit,
        ),
        19..=23 => (BEETLE_SPOTS[i - 19], Way::Sit),
        24 | 25 => (LADYBUG_FLOWERS[i - 24], Way::Sit),
        26 => (LADYBUG_FLOWERS[2], Way::Sit),
        // base_genji_pos.
        27 => (Vec3::new(258.0, 62.0, 258.5), Way::Hover),
        28 => (ground(240.0, 360.0), Way::Sit),
        29..=31 => (BEETLE_SPOTS[[5, 7, 8][i - 29]], Way::Sit),
        32 => (ROCKS[2] + Vec3::new(-12.0, -20.0, 10.0), Way::Sit),
        33 => (ground(120.0, 420.0), Way::Sit),
        // amenbo_center_pos.
        34 => (Vec3::new(289.0, 25.0, 218.0), Way::Pond),
        35 => (Vec3::new(392.0, 70.0, 0.0), Way::Sit),
        36 => (ground(395.0, 500.0), Way::Sit),
        37 => (Vec3::new(92.0, 75.0, 0.0), Way::Sit),
        38 => (ground(300.0, 420.0), Way::Sit),
        _ => (FLOWERS[3], Way::Flower(95.0)),
    }
}

/// minsect_scale_tbl.
fn scale(i: usize) -> f32 {
    match i {
        4..=7 => 0.008,
        22..=25 => 0.009,
        34 => 0.007,
        _ => 0.01,
    }
}

#[derive(Component)]
struct Insect {
    way: Way,
    home: Vec3,
    target: Vec3,
    frames: Vec<Entity>,
    frame: f32,
    phase: f32,
}

#[derive(Component)]
struct InsectShown;

/// xorshift (RANDOM_F).
fn rand(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    (*seed >> 8) as f32 / (1u32 << 24) as f32
}

/// The donated insects, rebuilt when the collection changes.
#[allow(clippy::too_many_arguments)]
fn spawn_insects(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    museum: Res<Museum>,
    collection: Res<MuseumCollection>,
    shown: Query<Entity, With<InsectShown>>,
    (mut meshes, mut materials, mut images): (
        ResMut<Assets<Mesh>>,
        ResMut<Assets<N64Material>>,
        ResMut<Assets<Image>>,
    ),
    mut built: Local<bool>,
    mut seed: Local<u32>,
) {
    let Some(disc) = disc else { return };
    let Some(Some(room)) = museum.rooms.get(INSECT_ROOM) else {
        return;
    };
    if *built && !collection.is_changed() {
        return;
    }
    *built = true;
    if *seed == 0 {
        *seed = 0x1234_5679;
    }
    for e in &shown {
        commands.entity(e).despawn();
    }
    let origin = room_origin(INSECT_ROOM);
    for (i, &donor) in collection.0.insect.iter().enumerate() {
        if donor == 0 {
            continue;
        }
        let (mut home, way) = spot(i);
        if home.y.is_nan() {
            let up = room
                .height_at(Vec2::new(home.x, home.z) / UNIT)
                .unwrap_or(0.0);
            home.y = up * UNIT + 1.0;
        }
        let root = commands
            .spawn((
                Name::new(format!("museum insect {i}")),
                InsectShown,
                Transform::from_translation(origin + home / UNIT)
                    .with_rotation(Quat::from_rotation_y(
                        rand(&mut seed) * std::f32::consts::TAU,
                    ))
                    .with_scale(Vec3::splat(scale(i) / UNIT)),
                Visibility::default(),
            ))
            .id();
        let mut frames = Vec::new();
        for (n, name) in FRAMES[i].iter().enumerate() {
            let full = format!("act_m_mu_{name}_model");
            let Some(list) = disc.catalog.get(&full).map(|a| a.offset) else {
                warn!("museum insect {i}: {full} not on the disc");
                continue;
            };
            let frame = commands
                .spawn((
                    Transform::default(),
                    if n == 0 {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                    ChildOf(root),
                ))
                .id();
            let sink = run_display_lists(&disc.rom, &[list]);
            crate::interior::spawn_sink(
                &mut commands,
                &disc,
                frame,
                &sink,
                None,
                &Segments::default(),
                false,
                &mut meshes,
                &mut materials,
                &mut images,
            );
            frames.push(frame);
        }
        commands.entity(root).insert(Insect {
            way,
            home,
            target: home,
            frames,
            frame: 0.0,
            phase: rand(&mut seed) * 10.0,
        });
    }
}

/// Moves and flaps the insects while the museum is shown.
fn fly(
    time: Res<Time>,
    state: Res<State<crate::shop::ShopLocation>>,
    mut insects: Query<(&mut Transform, &mut Insect)>,
    mut visibility: Query<&mut Visibility>,
    mut seed: Local<u32>,
) {
    if *state.get() != crate::shop::ShopLocation::Museum {
        return;
    }
    if *seed == 0 {
        *seed = 0x0BAD_5EED;
    }
    let dt = time.delta_secs();
    let ticks = dt * 60.0;
    let origin = room_origin(INSECT_ROOM);
    for (mut transform, mut bug) in &mut insects {
        bug.phase += dt;
        let local = (transform.translation - origin) * UNIT;
        let (next, rate) = match bug.way {
            Way::Flower(high) => {
                if (bug.target - local).length() < 6.0 {
                    let f = FLOWERS[(rand(&mut seed) * 8.0) as usize % 8];
                    bug.target = Vec3::new(f.x, high, f.z);
                }
                let to = bug.target - local;
                let step = to.normalize_or_zero() * (1.2 * ticks).min(to.length());
                let bob = Vec3::Y * (bug.phase * 6.0).sin() * 0.6;
                (local + step + bob, 0.5)
            }
            Way::Hover => {
                let p = bug.phase;
                let wobble = Vec3::new(
                    (p * 0.9).sin() * 6.0,
                    (p * 2.3).sin() * 3.0,
                    (p * 0.7).cos() * 6.0,
                );
                (bug.home + wobble, 0.5)
            }
            Way::Pond => {
                let p = bug.phase * 0.4;
                let off = Vec3::new(p.sin() * 25.0, 0.0, (p * 1.3).cos() * 18.0);
                (bug.home + off, 0.1)
            }
            Way::Sit => (local, 0.03),
        };
        let moved = next - local;
        if moved.with_y(0.0).length() > 0.01 {
            transform.rotation = Quat::from_rotation_y(moved.x.atan2(moved.z));
        }
        transform.translation = origin + next / UNIT;
        if bug.frames.len() > 1 {
            let count = bug.frames.len();
            let old = bug.frame as usize % count;
            bug.frame += rate * ticks;
            let new = bug.frame as usize % count;
            if old != new {
                if let Ok(mut v) = visibility.get_mut(bug.frames[old]) {
                    *v = Visibility::Hidden;
                }
                if let Ok(mut v) = visibility.get_mut(bug.frames[new]) {
                    *v = Visibility::Inherited;
                }
            }
        }
    }
}

pub struct MuseumInsectPlugin;

impl Plugin for MuseumInsectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_insects)
            .add_systems(Update, fly.in_set(crate::states::GameplaySet));
    }
}
