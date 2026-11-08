//! The fireworks show's own effects (`ef_hanabi_switch.c`, `ef_hanabi_set.c`, `ef_hanabi_botan1/botan2/hoshi/yanagi.c`).
//!
//! While `mEv_EVENT_FIREWORKS_SHOW` runs the switch effect starts a set every 300 ticks (at its tick 40), midway between the lake
//! and the player; a set (200 ticks) fires five to eight bursts at fixed ticks, each within 125 units of the set's spot. The first
//! four hours use the sets 1-4, the last hour the grand sets 5-8 with brighter bursts. A burst is one translucent model
//! (`ef_hanabi_b_00` peony, `_h_00` star, `_y_00` willow) that grows to its size, spins and swells, and whose prim / env colours
//! follow the effect's morph table over 80 or 110 ticks; peonies of the second kind and willows draw an inner copy at 0.6.
//!
//! Not built: the bursts' lights on the ground (`regist_effect_light`) and their sounds (`NA_SE_HANABI0..3`).

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;

use crate::event_body::Props;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::n64_material::N64Material;
use crate::player::Player;
use crate::states::GameplaySet;
use crate::town::Town;

/// Ticks a second.
const TICKS: f32 = 60.0;
/// The switch's cycle and the tick it starts a set.
const CYCLE: f32 = 300.0;
const SET_AT: f32 = 40.0;
/// `eHanabiSet`'s lifetime.
const SET_LIFE: f32 = 200.0;
/// Game units to cells.
const UNIT: f32 = 40.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Botan1,
    Botan2,
    Hoshi,
    Yanagi,
}

/// A morph entry: start frame, end frame, moves, from, to (`eEC_morph_data_c`).
type Morph = (u8, u8, bool, f32, f32);

const BOTAN1: [[Morph; 9]; 2] = [
    [
        (0, 0, false, 255.0, 255.0),
        (0, 0, false, 255.0, 255.0),
        (34, 44, true, 0.0, 100.0),
        (44, 54, true, 150.0, 0.0),
        (10, 34, true, 0.0, 255.0),
        (0, 0, false, 255.0, 255.0),
        (0, 0, false, 0.0, 0.0),
        (34, 44, true, 100.0, 255.0),
        (0, 0, false, 255.0, 255.0),
    ],
    [
        (0, 0, false, 255.0, 255.0),
        (0, 0, false, 255.0, 255.0),
        (34, 44, true, 0.0, 50.0),
        (44, 54, true, 150.0, 0.0),
        (10, 34, true, 0.0, 255.0),
        (0, 0, false, 0.0, 0.0),
        (0, 0, false, 255.0, 255.0),
        (34, 44, false, 50.0, 255.0),
        (0, 0, false, 255.0, 255.0),
    ],
];
const BOTAN2_OUT: [Morph; 9] = [
    (34, 44, true, 255.0, 0.0),
    (0, 0, false, 255.0, 255.0),
    (0, 0, false, 255.0, 255.0),
    (44, 54, true, 150.0, 0.0),
    (9, 34, true, 0.0, 255.0),
    (9, 34, true, 255.0, 0.0),
    (0, 0, false, 0.0, 0.0),
    (0, 0, false, 255.0, 255.0),
    (0, 0, false, 255.0, 255.0),
];
const BOTAN2_IN: [Morph; 9] = [
    (0, 0, false, 255.0, 255.0),
    (0, 0, false, 255.0, 255.0),
    (0, 0, false, 0.0, 0.0),
    (44, 54, true, 150.0, 0.0),
    (14, 34, true, 0.0, 255.0),
    (0, 0, false, 255.0, 255.0),
    (0, 0, false, 0.0, 0.0),
    (14, 34, true, 0.0, 255.0),
    (0, 0, false, 255.0, 255.0),
];
const HOSHI: [[Morph; 9]; 2] = [
    [
        (0, 0, false, 255.0, 255.0),
        (0, 0, false, 200.0, 200.0),
        (0, 0, false, 200.0, 200.0),
        (29, 39, true, 150.0, 0.0),
        (10, 24, true, 0.0, 255.0),
        (0, 0, false, 255.0, 255.0),
        (0, 0, false, 0.0, 0.0),
        (0, 0, false, 100.0, 100.0),
        (0, 0, false, 255.0, 255.0),
    ],
    [
        (0, 0, false, 255.0, 255.0),
        (0, 0, false, 255.0, 255.0),
        (0, 0, false, 255.0, 255.0),
        (29, 39, true, 150.0, 0.0),
        (10, 24, true, 0.0, 255.0),
        (0, 0, false, 0.0, 0.0),
        (0, 0, false, 255.0, 255.0),
        (0, 0, false, 0.0, 0.0),
        (0, 0, false, 255.0, 255.0),
    ],
];
const YANAGI_OUT: [Morph; 9] = [
    (0, 0, false, 255.0, 255.0),
    (34, 54, true, 255.0, 200.0),
    (34, 54, true, 50.0, 100.0),
    (34, 54, true, 150.0, 0.0),
    (5, 34, true, 0.0, 255.0),
    (0, 0, false, 255.0, 255.0),
    (0, 0, false, 0.0, 0.0),
    (0, 0, false, 200.0, 200.0),
    (0, 0, false, 255.0, 255.0),
];
const YANAGI_IN: [Morph; 9] = [
    (0, 0, false, 255.0, 255.0),
    (34, 49, true, 255.0, 200.0),
    (34, 49, true, 50.0, 100.0),
    (34, 49, true, 180.0, 0.0),
    (5, 34, true, 0.0, 255.0),
    (0, 0, false, 255.0, 255.0),
    (0, 0, false, 0.0, 0.0),
    (0, 0, false, 200.0, 200.0),
    (0, 0, false, 255.0, 255.0),
];

impl Kind {
    fn life(self) -> f32 {
        if self == Kind::Hoshi {
            80.0
        } else {
            110.0
        }
    }

    /// The size the burst grows to (`add_calc2`'s target).
    fn target(self) -> f32 {
        match self {
            Kind::Botan1 => 0.07,
            Kind::Botan2 => 0.08,
            Kind::Hoshi => 0.025,
            Kind::Yanagi => 0.06,
        }
    }

    fn model(self) -> &'static str {
        match self {
            Kind::Botan1 | Kind::Botan2 => "ef_hanabi_b_00_modelT",
            Kind::Hoshi => "ef_hanabi_h_00_modelT",
            Kind::Yanagi => "ef_hanabi_y_00_modelT",
        }
    }

    /// The morph tables of each layer, the outer first.
    fn layers(self, variant: usize) -> Vec<&'static [Morph; 9]> {
        match self {
            Kind::Botan1 => vec![&BOTAN1[variant]],
            Kind::Botan2 => vec![&BOTAN2_OUT, &BOTAN2_IN],
            Kind::Hoshi => vec![&HOSHI[variant]],
            Kind::Yanagi => vec![&YANAGI_OUT, &YANAGI_IN],
        }
    }
}

/// `eHanabiSet_set1..8`: (tick, burst).
const SETS: [&[(u16, Kind)]; 8] = [
    &[
        (0, Kind::Hoshi),
        (30, Kind::Hoshi),
        (50, Kind::Hoshi),
        (120, Kind::Botan1),
        (190, Kind::Botan1),
    ],
    &[
        (0, Kind::Botan1),
        (50, Kind::Botan1),
        (80, Kind::Botan1),
        (140, Kind::Botan2),
        (190, Kind::Botan2),
    ],
    &[
        (0, Kind::Yanagi),
        (50, Kind::Yanagi),
        (80, Kind::Yanagi),
        (140, Kind::Botan2),
        (190, Kind::Botan2),
    ],
    &[
        (0, Kind::Hoshi),
        (50, Kind::Yanagi),
        (80, Kind::Yanagi),
        (140, Kind::Botan1),
        (190, Kind::Botan1),
    ],
    &[
        (0, Kind::Hoshi),
        (20, Kind::Hoshi),
        (50, Kind::Botan2),
        (90, Kind::Botan2),
        (140, Kind::Hoshi),
        (160, Kind::Botan1),
        (170, Kind::Botan1),
        (170, Kind::Botan1),
    ],
    &[
        (0, Kind::Botan1),
        (20, Kind::Botan2),
        (50, Kind::Botan2),
        (90, Kind::Yanagi),
        (140, Kind::Yanagi),
        (160, Kind::Hoshi),
        (170, Kind::Botan2),
        (170, Kind::Botan2),
    ],
    &[
        (0, Kind::Botan2),
        (20, Kind::Botan2),
        (50, Kind::Hoshi),
        (90, Kind::Hoshi),
        (140, Kind::Yanagi),
        (160, Kind::Yanagi),
        (170, Kind::Yanagi),
        (170, Kind::Yanagi),
    ],
    &[
        (0, Kind::Yanagi),
        (10, Kind::Yanagi),
        (20, Kind::Yanagi),
        (30, Kind::Yanagi),
        (140, Kind::Botan2),
        (160, Kind::Botan2),
        (170, Kind::Botan2),
        (190, Kind::Botan2),
    ],
];

/// A set firing its bursts.
#[derive(Component)]
struct Set {
    which: usize,
    age: f32,
    next: usize,
    at: Vec3,
}

/// One burst (`eHanabiBotan1_*` and its kin).
#[derive(Component)]
struct Burst {
    kind: Kind,
    variant: usize,
    age: f32,
    scale: f32,
    /// `effect_specific[0..2]`: the swell, the spin and the willow's drift, in 1/65536 turns.
    r0: f32,
    r1: f32,
    r2: f32,
    base: Vec3,
    /// The layers' materials, outer first.
    materials: Vec<Handle<N64Material>>,
}

/// The switch: seconds into its cycle.
#[derive(Resource, Default)]
struct Switch {
    t: f32,
    started: bool,
}

/// `eEL_CalcAdjust_F` / `eEL_CalcAdjust`: `now` between `from_t` and `to_t` maps linearly from `a` to `b`.
fn adjust(now: f32, from_t: f32, to_t: f32, a: f32, b: f32) -> f32 {
    if now <= from_t {
        a
    } else if now >= to_t || to_t <= from_t {
        b
    } else {
        a + (b - a) * (now - from_t) / (to_t - from_t)
    }
}

/// `eEC_MorphCombine`: the nine colour values at tick `now`.
fn morph(table: &[Morph; 9], now: f32) -> [f32; 9] {
    let mut out = [0.0; 9];
    for (o, &(s, e, moves, a, b)) in out.iter_mut().zip(table) {
        *o = if moves {
            adjust(now, f32::from(s) * 2.0, f32::from(e) * 2.0, a, b)
        } else {
            a
        };
    }
    out
}

fn turns(r: f32) -> f32 {
    r / 65_536.0 * std::f32::consts::TAU
}

/// The switch: while the show runs, a set every 300 ticks (`eHanabiSwitch_mv`).
#[allow(clippy::too_many_arguments)]
fn switch(
    mut commands: Commands,
    time: Res<Time>,
    town: Res<Town>,
    today: Res<TodayEvents>,
    mut switch: ResMut<Switch>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    player: Query<&Transform, With<Player>>,
) {
    let now = crate::clock::now();
    if !today.is_active(ev::FIREWORKS_SHOW, now.hour) {
        *switch = Switch::default();
        return;
    }
    let Some(acre) = crate::fireworks::water_acre(&town) else {
        return;
    };
    if !switch.started {
        switch.started = true;
        switch.t = 0.0;
    }
    let before = switch.t;
    switch.t += time.delta_secs() * TICKS;
    if before < SET_AT && switch.t >= SET_AT {
        // `eHanabiSet_SearchNicePos`: midway between the lake's middle and the player, a unit south.
        let lake = Town::cell_to_world(acre.0, acre.1, 8, 8);
        let player = player.single().map_or(lake, |p| p.translation);
        let at = (lake + player) * 0.5 + Vec3::Z * -1.0;
        // The last hour of the show is the finale (`hour > end - 1`).
        let grand = i32::from(now.hour) > SHOW_END - 1 && i32::from(now.hour) <= SHOW_END;
        let pick = rand.0.random(1000) as usize % 4;
        commands.spawn((
            Name::new("fireworks set"),
            Set {
                which: pick + if grand { 4 } else { 0 },
                age: 0.0,
                next: 0,
                at,
            },
        ));
    }
    if switch.t >= CYCLE {
        switch.t -= CYCLE;
    }
}

/// `mEv_get_end_time(mEv_EVENT_FIREWORKS_SHOW)`: the show runs 19:00 to 20:00 (the schedule row's end hour).
const SHOW_END: i32 = 20;

/// A set fires its bursts at their ticks.
#[allow(clippy::too_many_arguments)]
fn sets(
    mut commands: Commands,
    time: Res<Time>,
    disc: Option<Res<GameDisc>>,
    mut props: ResMut<Props>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut sets: Query<(Entity, &mut Set)>,
) {
    let Some(disc) = disc else { return };
    for (entity, mut set) in &mut sets {
        set.age += time.delta_secs() * TICKS;
        let table = SETS[set.which];
        while set.next < table.len() && f32::from(table[set.next].0) <= set.age {
            let (_, kind) = table[set.next];
            set.next += 1;
            let spot = set.at
                + Vec3::new(
                    (rand.0.fqrand() * 250.0 - 125.0) / UNIT,
                    0.0,
                    (rand.0.fqrand() * 250.0 - 125.0) / UNIT,
                );
            let parts = props.get(
                &disc,
                &[kind.model()],
                None,
                None,
                (&mut meshes, &mut materials, &mut images),
            );
            let variant = usize::from(rand.0.random(10) & 1 == 1);
            let layers = kind.layers(variant);
            let root = commands
                .spawn((
                    Name::new("fireworks burst"),
                    Transform::from_translation(spot),
                    Visibility::default(),
                ))
                .id();
            let mut handles = Vec::new();
            for layer in 0..layers.len() {
                let holder = commands
                    .spawn((
                        Transform::from_scale(Vec3::splat(if layer == 1 { 0.6 } else { 1.0 })),
                        Visibility::default(),
                        ChildOf(root),
                    ))
                    .id();
                // A copy of the model's material for this layer, whose colours the morph drives.
                let mut own = None;
                for (mesh, material) in &parts {
                    let Some(base) = materials.get(material).cloned() else {
                        continue;
                    };
                    let handle = materials.add(base);
                    own.get_or_insert(handle.clone());
                    commands.spawn((
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(handle),
                        ChildOf(holder),
                    ));
                }
                if let Some(handle) = own {
                    handles.push(handle);
                }
            }
            commands.entity(root).insert(Burst {
                kind,
                variant,
                age: 0.0,
                scale: 0.01,
                r0: 0.0,
                r1: 0.0,
                r2: 0.0,
                base: spot,
                materials: handles,
            });
        }
        if set.age >= SET_LIFE {
            commands.entity(entity).despawn();
        }
    }
}

/// A burst grows, spins and changes colour until its life is over.
fn bursts(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<N64Material>>,
    mut bursts: Query<(Entity, &mut Burst, &mut Transform)>,
) {
    let ticks = time.delta_secs() * TICKS;
    for (entity, mut burst, mut at) in &mut bursts {
        let b = &mut *burst;
        b.age += ticks;
        if b.age >= b.kind.life() {
            commands.entity(entity).despawn();
            continue;
        }
        b.r0 += 768.0 * ticks;
        b.r1 += 256.0 * ticks;
        b.r2 += 128.0 * ticks;
        for v in [&mut b.r0, &mut b.r1, &mut b.r2] {
            *v %= 65_536.0;
        }
        // `add_calc2(&scale, target, 1 - sqrt(0.8), 5)` each tick.
        let target = b.kind.target();
        let step = ((target - b.scale) * (1.0 - 0.8_f32.sqrt())).clamp(-5.0, 5.0);
        b.scale += step * ticks;
        let offset = match b.kind {
            Kind::Yanagi => Vec2::new(turns(b.r2).sin(), (-turns(b.r2)).cos()),
            _ => Vec2::new(turns(b.r1).sin(), (-turns(b.r1)).sin()),
        } * 2.0
            / UNIT;
        let swell = (turns(b.r0).sin() + 1.0) * 0.5 * 0.14 + 0.93;
        let life = b.kind.life();
        let size = b.scale + adjust(b.age, 0.0, life - 1.0, 0.0, 0.01);
        // `Matrix_RotateX(270)`, the spin with a swell along one axis, the size.
        let m = Mat4::from_rotation_x(3.0 * std::f32::consts::FRAC_PI_2)
            * Mat4::from_rotation_z(-turns(b.r1))
            * Mat4::from_scale(Vec3::new(swell, 1.0, 1.0))
            * Mat4::from_rotation_z(turns(b.r1))
            * Mat4::from_scale(Vec3::splat(size / 0.01));
        let mut transform = Transform::from_matrix(Mat4::from_translation(b.base) * m);
        transform.translation += Vec3::new(offset.x, 0.0, offset.y);
        *at = transform;
        let tables = b.kind.layers(b.variant);
        for (handle, table) in b.materials.iter().zip(tables) {
            let c = morph(table, b.age);
            if let Some(mut material) = materials.get_mut(handle) {
                material.combiner.prim = Vec4::new(c[0], c[1], c[2], c[3]) / 255.0;
                material.combiner.extra.x = c[4] / 255.0;
                material.combiner.env = Vec4::new(c[5], c[6], c[7], c[8]) / 255.0;
            }
        }
    }
}

pub struct HanabiPlugin;

impl Plugin for HanabiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Switch>()
            .add_systems(Update, (switch, sets, bursts).chain().in_set(GameplaySet));
    }
}
