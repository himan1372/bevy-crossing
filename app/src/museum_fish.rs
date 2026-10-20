//! The museum's fish room (ac_museum_fish): four tanks (obj_suisou1) at
//! suisou_pos 0-3 and the big tank (obj_museum5, scale 0.0625) at the back,
//! each donated fish swimming in its tank (mfish_group_tbl) as its museum
//! model (cKF_bs_r_act_mus_*) at its renderScale.
//!
//! Swimming is mfish_base_mv in outline: cruise at an eased speed, now and
//! then turn by 45-120 degrees, turn back inward at the tank's walls, and
//! keep between 60 and 110 units up (plus the fish's own ofs_y). The water
//! plants (kusa), the frog's lily pad and the jellyfish are not drawn yet.
use bevy::prelude::*;

use crate::disc_gfx::{run_display_lists, Segments};
use crate::disc_rig::DiscRig;
use crate::game_disc::GameDisc;
use crate::museum::{room_origin, Museum, MuseumCollection};
use crate::n64_material::N64Material;

const UNIT: f32 = 40.0;
/// The fish room's index in the museum's rooms.
const FISH_ROOM: usize = 4;
/// suisou_pos: tanks 0-3 (obj_suisou1, scale 0.01); 4 is the big tank.
const TANKS: [Vec2; 5] = [
    Vec2::new(220.0, 220.0),
    Vec2::new(420.0, 220.0),
    Vec2::new(220.0, 460.0),
    Vec2::new(420.0, 460.0),
    Vec2::new(320.0, 20.0),
];

/// mfish_model_tbl / anime table by fish (item) index; None: the jellyfish.
const MODELS: [Option<(&str, &str)>; 40] = [
    Some(("funa_a1", "funa_a1")),
    Some(("hera_a1", "hera_a1")),
    Some(("koi_a1", "koi_a1")),
    Some(("nisiki_a1", "nisiki_a1")),
    Some(("namazu_a1", "namazu_a1")),
    Some(("bass_a1", "bass_a1")),
    Some(("bassm_a1", "bassm_a1")),
    Some(("bassl_a1", "bassl_a1")),
    Some(("gill_a1", "gill_a1")),
    Some(("onamazu_a1", "onamazu_a1")),
    Some(("raigyo_a1", "raigyo_a1")),
    Some(("nigoi_a1", "nigoi_a1")),
    Some(("ugui_a1", "ugui_a1")),
    Some(("oikawa_a1", "oikawa_a1")),
    Some(("tanago_a1", "tanago_a1")),
    Some(("dojyo_a1", "dojyo_a1")),
    Some(("wakasa_a1", "wakasa_a1")),
    Some(("ayu_a1", "ayu_a1")),
    Some(("yamame_a1", "yamame_a1")),
    Some(("iwana_a1", "iwana_a1")),
    Some(("niji_a1", "niji_a1")),
    Some(("ito_a1", "ito_a1")),
    Some(("sake_a1", "sake_a1")),
    Some(("kingyo_a1", "kingyo_a1")),
    Some(("pirania_a1", "pirania_a1")),
    Some(("aroana_a1", "aroana_a1")),
    Some(("unagi_a1", "unagi_a2")),
    Some(("donko_a1", "donko_a1")),
    Some(("angel_a1", "angel_a1")),
    Some(("gupi_a1", "gupi_a1")),
    Some(("demekin_a1", "demekin_a1")),
    Some(("kaseki_a1", "kaseki_a1")),
    Some(("zari", "zari")),
    Some(("kaeru_a1", "kaeru_a1")),
    Some(("medaka_a1", "medaka_a1")),
    None,
    Some(("suzuki_a1", "suzuki_a1")),
    Some(("tai_a1", "tai_a1")),
    Some(("isidai_a1", "isidai_a1")),
    Some(("pira_a1", "pira_a1")),
];

/// mfish_init_data: renderScale and ofs_y.
const SIZE: [(f32, f32); 40] = [
    (0.0100, 3.8),
    (0.0100, 4.6),
    (0.0100, 6.0),
    (0.0100, 6.0),
    (0.0110, 6.0),
    (0.0110, 3.0),
    (0.0110, 4.7),
    (0.0110, 5.5),
    (0.0100, 4.3),
    (0.0120, 7.6),
    (0.0120, 9.6),
    (0.0100, 4.8),
    (0.0100, 3.0),
    (0.0100, 2.5),
    (0.0100, 3.5),
    (0.0100, 4.0),
    (0.0100, 2.0),
    (0.0100, 3.0),
    (0.0100, 3.5),
    (0.0100, 2.5),
    (0.0100, 2.5),
    (0.0120, 6.1),
    (0.0140, 6.2),
    (0.0080, 2.2),
    (0.0120, 4.2),
    (0.0130, 5.0),
    (0.0095, 3.4),
    (0.0120, 3.3),
    (0.0100, 5.1),
    (0.0080, 2.7),
    (0.0080, 2.5),
    (0.0160, 7.1),
    (0.0165, 5.8),
    (0.0100, 1.0),
    (0.0080, 1.5),
    (0.0100, 4.0),
    (0.0100, 4.0),
    (0.0100, 6.0),
    (0.0100, 5.0),
    (0.0120, 9.3),
];

/// mfish_group_tbl: each fish's tank.
const TANK_OF: [u8; 40] = [
    0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 3, 0, 2, 2, 0, 1, 2, 2, 2, 2, 2, 2, 4, 0, 3, 3, 0, 1, 3, 3, 0, 4,
    2, 1, 0, 4, 4, 4, 4, 3,
];

/// Each tank's water, room-local game units (x/z box).
#[derive(Resource, Default)]
struct Tanks(Vec<Rect>);

#[derive(Component)]
struct Fish {
    tank: usize,
    heading: f32,
    speed: f32,
    target_speed: f32,
    height: f32,
    timer: f32,
}

#[derive(Component)]
struct FishShown;

/// xorshift for the swimmers (RANDOM_F).
fn rand(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    (*seed >> 8) as f32 / (1u32 << 24) as f32
}

/// Draws one static display list at a room-local point and returns the
/// x/z box of its parts above the floor, room-local game units.
#[allow(clippy::too_many_arguments)]
fn place(
    commands: &mut Commands,
    disc: &GameDisc,
    (opaque, translucent): (&str, &str),
    at: Vec2,
    scale: f32,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> Option<Rect> {
    let origin = room_origin(FISH_ROOM);
    let root = commands
        .spawn((
            Name::new(opaque.to_string()),
            Transform::from_translation(origin + Vec3::new(at.x, 0.0, at.y) / UNIT)
                .with_scale(Vec3::splat(scale / UNIT)),
            Visibility::default(),
        ))
        .id();
    let mut bounds: Option<Rect> = None;
    for (name, blend) in [(opaque, false), (translucent, true)] {
        let Some(list) = disc.catalog.get(name).map(|a| a.offset) else {
            continue;
        };
        let sink = run_display_lists(&disc.rom, &[list]);
        for v in sink.parts.iter().flat_map(|p| &p.vertices) {
            if v.pos[1] * scale > 20.0 {
                let p = at + Vec2::new(v.pos[0], v.pos[2]) * scale;
                bounds = Some(bounds.map_or(Rect::from_corners(p, p), |b| b.union_point(p)));
            }
        }
        crate::interior::spawn_sink(
            commands,
            disc,
            root,
            &sink,
            None,
            &Segments::default(),
            blend,
            meshes,
            materials,
            images,
        );
    }
    bounds
}

/// The tanks, once the rooms exist; their units are solid.
fn build_tanks(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    mut museum: ResMut<Museum>,
    mut tanks: ResMut<Tanks>,
    (mut meshes, mut materials, mut images): (
        ResMut<Assets<Mesh>>,
        ResMut<Assets<N64Material>>,
        ResMut<Assets<Image>>,
    ),
) {
    let Some(disc) = disc else { return };
    if !tanks.0.is_empty() || museum.rooms.len() <= FISH_ROOM {
        return;
    }
    let assets = (&mut *meshes, &mut *materials, &mut *images);
    let mut boxes = Vec::new();
    for (i, &at) in TANKS.iter().enumerate() {
        let (models, at, scale) = if i < 4 {
            (("obj_suisou1_model", "obj_suisou1_modelT"), at, 0.01)
        } else {
            // Drawn from the room's origin (Matrix_scale with MTX_LOAD).
            (
                ("obj_museum5_model", "obj_museum5_modelT"),
                Vec2::ZERO,
                0.0625,
            )
        };
        let b = place(
            &mut commands,
            &disc,
            models,
            at,
            scale,
            (&mut *assets.0, &mut *assets.1, &mut *assets.2),
        )
        .unwrap_or(Rect::from_center_size(TANKS[i], Vec2::splat(120.0)));
        boxes.push(b);
    }
    let mut solid = Vec::new();
    for b in &boxes {
        let (lo, hi) = ((b.min / UNIT).floor(), (b.max / UNIT).ceil());
        for x in lo.x as i32..hi.x as i32 {
            for z in lo.y as i32..hi.y as i32 {
                solid.push((x, z));
            }
        }
    }
    if let Some(Some(room)) = museum.rooms.get_mut(FISH_ROOM) {
        room.set_furniture(solid);
    }
    info!("fish tanks: {boxes:?}");
    tanks.0 = boxes;
}

/// The donated fish, rebuilt when the collection changes.
#[allow(clippy::too_many_arguments)]
fn spawn_fish(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    collection: Res<MuseumCollection>,
    tanks: Res<Tanks>,
    shown: Query<Entity, With<FishShown>>,
    (mut meshes, mut materials, mut images): (
        ResMut<Assets<Mesh>>,
        ResMut<Assets<N64Material>>,
        ResMut<Assets<Image>>,
    ),
    mut seed: Local<u32>,
) {
    let Some(disc) = disc else { return };
    if tanks.0.is_empty() || !(collection.is_changed() || tanks.is_changed()) {
        return;
    }
    for e in &shown {
        commands.entity(e).despawn();
    }
    if *seed == 0 {
        *seed = 0x2545_F491;
    }
    let origin = room_origin(FISH_ROOM);
    for (i, &donor) in collection.0.fish.iter().enumerate() {
        let Some((skeleton, animation)) = MODELS[i].filter(|_| donor != 0) else {
            continue;
        };
        let tank = TANK_OF[i] as usize;
        let water = tanks.0[tank].inflate(-15.0);
        let p = water.min + water.size() * Vec2::new(rand(&mut seed), rand(&mut seed));
        let (scale, ofs_y) = SIZE[i];
        let height = 60.0 + 50.0 * rand(&mut seed) + ofs_y;
        let heading = rand(&mut seed) * std::f32::consts::TAU;
        let root = commands
            .spawn((
                Name::new(format!("museum fish {i}")),
                FishShown,
                Transform::from_translation(origin + Vec3::new(p.x, height, p.y) / UNIT)
                    .with_rotation(Quat::from_rotation_y(heading))
                    .with_scale(Vec3::splat(scale / UNIT)),
                Visibility::default(),
            ))
            .id();
        let assets = (&mut *meshes, &mut *materials, &mut *images);
        let Some(rig) = crate::disc_rig::spawn(
            &mut commands,
            &disc,
            root,
            (
                &format!("cKF_bs_r_act_mus_{skeleton}"),
                &format!("cKF_ba_r_act_mus_{animation}"),
            ),
            assets,
        ) else {
            warn!("museum fish {i}: model not on the disc");
            continue;
        };
        commands.entity(root).insert((
            rig,
            Fish {
                tank,
                heading,
                speed: 0.0,
                target_speed: 0.5 + rand(&mut seed),
                height,
                timer: 2.0 + 3.0 * rand(&mut seed),
            },
        ));
    }
}

/// mfish_base_mv, in outline.
fn swim(
    time: Res<Time>,
    state: Res<State<crate::shop::ShopLocation>>,
    tanks: Res<Tanks>,
    mut fish: Query<(&mut Transform, &mut Fish, &mut DiscRig)>,
    mut transforms: Query<&mut Transform, Without<Fish>>,
    mut seed: Local<u32>,
) {
    if *state.get() != crate::shop::ShopLocation::Museum || tanks.0.is_empty() {
        return;
    }
    if *seed == 0 {
        *seed = 0x9E37_79B9;
    }
    let ticks = time.delta_secs() * 60.0;
    let origin = room_origin(FISH_ROOM);
    for (mut transform, mut f, mut rig) in &mut fish {
        let water = tanks.0[f.tank].inflate(-15.0);
        let local = Vec2::new(
            transform.translation.x - origin.x,
            transform.translation.z - origin.z,
        ) * UNIT;
        f.timer -= time.delta_secs();
        if f.timer <= 0.0 {
            // mfish_turn_process_init: a new heading and pace.
            let turn = (45.0 + 75.0 * rand(&mut seed)).to_radians();
            f.heading += if rand(&mut seed) < 0.5 { turn } else { -turn };
            f.target_speed = 0.2 + 1.0 * rand(&mut seed);
            f.timer = 2.0 + 4.0 * rand(&mut seed);
        }
        // mfish_PosWallCheck: head back in from the glass.
        let dir = Vec2::new(f.heading.sin(), f.heading.cos());
        let ahead = local + dir * 20.0;
        if !water.contains(ahead) {
            let to_centre = water.center() - local;
            let want = to_centre.x.atan2(to_centre.y);
            let d = (want - f.heading + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            f.heading += d.clamp(-0.08 * ticks, 0.08 * ticks);
            f.target_speed = f.target_speed.min(0.4);
        }
        f.speed += (f.target_speed - f.speed) * (0.05 * ticks).min(1.0);
        let dir = Vec2::new(f.heading.sin(), f.heading.cos());
        let next = (local + dir * f.speed * ticks).clamp(water.min, water.max);
        transform.translation = origin + Vec3::new(next.x, f.height, next.y) / UNIT;
        let want = Quat::from_rotation_y(f.heading);
        transform.rotation = transform.rotation.slerp(want, (0.15 * ticks).min(1.0));
        rig.step((0.5 + f.speed * 0.5) * ticks, &mut transforms);
    }
}

pub struct MuseumFishPlugin;

impl Plugin for MuseumFishPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tanks>()
            .add_systems(Update, (build_tanks, spawn_fish).chain())
            .add_systems(Update, swim.in_set(crate::states::GameplaySet));
    }
}
