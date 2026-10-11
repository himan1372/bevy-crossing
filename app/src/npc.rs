//! Villagers: daily routines and wandering, driven by the translated NPC AI.
//!
//! From the decomp (`rustimal_logic::npc_ai` / `movement`):
//! - the per-personality daily schedule (`schedule_manager_sub`): FIELD
//!   (out in town), IN_HOUSE, SLEEP;
//! - the wander decision (`wander_decide_next`): wait / walk / run with the
//!   per-personality odds (e.g. peppy waits 70% of the time);
//! - wander targets are random points in a circle around home
//!   (`center + (sin a, cos a) * r`), steered toward directly — no
//!   pathfinding — with the wall-avoidance probes (`AVOID_PROBE_ANGLES`:
//!   ±22.5°, ±45°, ±90° two units ahead) and a turn-back when all fail;
//! - two villagers who meet greet each other and both become happy
//!   (`ac_npc_act_greeting`).
//!
//! Ours (not traced): walk/run speeds, wander radius, wait times, the
//! greeting distance and cooldown, and villager-to-house assignment (the
//! translated `villager_home` lot assignment isn't wired yet; villagers get
//! the town's non-player houses in order).

use bevy::prelude::*;
use rustimal_logic::movement::{AVOID_PROBE_ANGLES, AVOID_PROBE_DISTANCE_UNITS};
use rustimal_logic::npc::Personality;
use rustimal_logic::npc_ai::{self, sched, WanderAct};
use rustimal_logic::town_gen::{CellKind, ACRE_DEPTH, ACRE_WIDTH};

use crate::dialogue::{DialogueState, VillagerTalk};
use crate::player::Player;
use crate::town::{Town, HOUSE_SIZE};

/// Wander targets lie within this distance of home.
const WANDER_RADIUS: f32 = 6.0;
const WALK_SPEED: f32 = 1.6;
const RUN_SPEED: f32 = 3.4;
/// Seconds a "wait" lasts (min, max).
const WAIT_SECS: (f32, f32) = (1.5, 4.0);
/// Give up on a walk/run after this long.
const MOVE_TIMEOUT_SECS: f32 = 8.0;
/// Villagers closer than this greet each other.
const GREET_DIST: f32 = 1.6;
/// Seconds before the same villager greets again.
const GREET_COOLDOWN_SECS: f32 = 45.0;
/// How long a greeting pause lasts.
const GREET_SECS: f32 = 2.0;

/// Marker + data for a villager.
#[derive(Component)]
pub struct Villager {
    /// Display name.
    pub name: &'static str,
    /// Decomp personality (`mNpc_LOOKS_*` order) from the logic crate.
    pub personality: Personality,
    /// `looks` index, kept alongside for logic-crate calls.
    pub looks: u8,
    /// Wander centre: in front of their house.
    home: Vec3,
    /// Where they step out of / go into their house.
    door: Vec3,
    act: Act,
    /// Seconds left in the current act.
    timer: f32,
    greet_cooldown: f32,
}

/// What a villager is doing right now.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Act {
    Wait,
    Walk(Vec3),
    Run(Vec3),
    Greet,
    GoHome,
}

/// Present on villagers who are inside their house (hidden, can't be
/// talked to).
#[derive(Component)]
pub struct Indoors;

/// Tiny xorshift RNG — avoids adding a `rand` dependency.
#[derive(Resource)]
struct WanderRng(u64);

impl WanderRng {
    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }
    fn range_f32(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (self.next_u32() as f32 / u32::MAX as f32) * (hi - lo)
    }
    /// 0..n
    fn below(&mut self, n: u32) -> u32 {
        self.next_u32() % n.max(1)
    }
}

/// (name, looks/personality index, capsule color).
const VILLAGERS: [(&str, u8, (f32, f32, f32)); 4] = [
    ("Maple", 0, (0.95, 0.55, 0.35)), // normal — orange
    ("Pippy", 1, (0.95, 0.35, 0.55)), // peppy — pink
    ("Bob", 2, (0.45, 0.75, 0.45)),   // lazy — green
    ("Chief", 4, (0.55, 0.45, 0.75)), // cranky — purple
];

/// Fallback homes if the town has fewer houses than villagers.
const FALLBACK_HOMES: [Vec3; 4] = [
    Vec3::new(7.0, 1.0, 5.0),
    Vec3::new(-6.0, 1.0, 7.0),
    Vec3::new(5.0, 1.0, -7.0),
    Vec3::new(-7.0, 1.0, -6.0),
];

/// Centres of every house in the town except the player's.
fn villager_house_centres(town: &Town, player_door: Option<Vec3>) -> Vec<Vec3> {
    let mut out = Vec::new();
    for az in 0..ACRE_DEPTH {
        for ax in 0..ACRE_WIDTH {
            let acre = &town.plan.acres[az * ACRE_WIDTH + ax];
            for cz in 0..16 {
                for cx in 0..16 {
                    if acre.cells[cz * 16 + cx] != CellKind::House as u8 {
                        continue;
                    }
                    let c = Town::cell_to_world(ax, az, cx, cz);
                    let is_player = player_door.is_some_and(|d| {
                        (d.x - c.x).abs() < 0.5 && (d.z - c.z) > 0.0 && (d.z - c.z) < 3.0
                    });
                    if !is_player {
                        out.push(c);
                    }
                }
            }
        }
    }
    out
}

/// Schedule type now for a personality (`mNPS_schedule_manager_sub`).
fn schedule_now(looks: u8) -> u32 {
    let now = crate::clock::now();
    npc_ai::schedule_manager_sub(looks as usize, now.day_secs, 0, 0, 0).1
}

fn is_out(schedule: u32) -> bool {
    schedule == sched::FIELD
}

/// Spawn villagers after the town and the player's house exist.
fn spawn_villagers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    town: Res<Town>,
    player_house: Option<Res<crate::house::PlayerHouse>>,
) {
    commands.insert_resource(WanderRng(0x9E3779B97F4A7C15));
    let houses = villager_house_centres(&town, player_house.map(|h| h.door_pos));

    for (i, (name, looks, (r, g, b))) in VILLAGERS.iter().enumerate() {
        let personality = Personality::from_looks(*looks).unwrap_or(Personality::Girl);
        // Door just in front of the house's south face; home a step further out.
        let (door, home) = match houses.get(i) {
            Some(c) => (
                Vec3::new(c.x, 1.0, c.z + HOUSE_SIZE / 2.0 + 0.5),
                Vec3::new(c.x, 1.0, c.z + HOUSE_SIZE / 2.0 + 2.0),
            ),
            None => (FALLBACK_HOMES[i % 4], FALLBACK_HOMES[i % 4]),
        };
        let schedule = schedule_now(*looks);
        let out = is_out(schedule);
        let mut entity = commands.spawn((
            VillagerTalk::default(),
            Villager {
                name,
                personality,
                looks: *looks,
                home,
                door,
                act: Act::Wait,
                timer: 0.0,
                greet_cooldown: 0.0,
            },
            Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
            MeshMaterial3d(materials.add(Color::srgb(*r, *g, *b))),
            Transform::from_translation(if out { home } else { door }),
            if out {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
        ));
        if !out {
            entity.insert(Indoors);
        }
        println!(
            "Spawned villager {name} ({}) — {} now",
            personality.english_name(),
            schedule_name(schedule)
        );
    }
    println!("Villagers spawned");
}

fn schedule_name(schedule: u32) -> &'static str {
    match schedule {
        sched::FIELD => "out in town",
        sched::IN_HOUSE => "at home",
        sched::SLEEP => "asleep",
        _ => "busy",
    }
}

/// Follow the daily schedule: step out for FIELD, head home otherwise.
fn villager_schedule(
    mut commands: Commands,
    dialogue: Res<DialogueState>,
    mut villagers: Query<(
        Entity,
        &mut Villager,
        &mut Transform,
        &mut Visibility,
        Has<Indoors>,
    )>,
) {
    // `aNPC_check_chg_schedule`: no schedule change mid-conversation.
    if dialogue.active {
        return;
    }
    for (entity, mut v, mut transform, mut vis, indoors) in villagers.iter_mut() {
        let out = is_out(schedule_now(v.looks));
        if out && indoors {
            commands.entity(entity).remove::<Indoors>();
            *vis = Visibility::Inherited;
            transform.translation = v.door;
            v.act = Act::Wait;
            v.timer = 1.0;
            println!("{} stepped outside", v.name);
        } else if !out && !indoors && v.act != Act::GoHome {
            v.act = Act::GoHome;
            v.timer = 60.0;
            println!("{} is heading home", v.name);
        }
    }
}

/// Wander, greet, and walk home. Everyone stands still while the player is
/// in a conversation.
#[allow(clippy::too_many_arguments)]
fn villager_move(
    mut commands: Commands,
    mut rng: ResMut<WanderRng>,
    town: Res<Town>,
    shop_building: Option<Res<crate::shop::ShopBuilding>>,
    time: Res<Time>,
    dialogue: Res<DialogueState>,
    player_query: Query<&Transform, With<Player>>,
    mut villagers: Query<
        (
            Entity,
            &mut Villager,
            &mut Transform,
            &mut Visibility,
            &mut VillagerTalk,
        ),
        (Without<Player>, Without<Indoors>),
    >,
) {
    if dialogue.active {
        return;
    }
    let dt = time.delta().as_secs_f32();
    let player_pos = player_query
        .single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::splat(f32::MAX));

    // Greetings first: pairs of out-and-about villagers who meet.
    let positions: Vec<(Entity, Vec3, bool)> = villagers
        .iter()
        .map(|(e, v, t, ..)| {
            let free = v.greet_cooldown <= 0.0 && !matches!(v.act, Act::GoHome | Act::Greet);
            (e, t.translation, free)
        })
        .collect();
    let mut greeters: Vec<(Entity, Vec3)> = Vec::new();
    for (i, a) in positions.iter().enumerate() {
        for b in positions.iter().skip(i + 1) {
            if a.2 && b.2 && a.1.distance(b.1) < GREET_DIST {
                greeters.push((a.0, b.1));
                greeters.push((b.0, a.1));
            }
        }
    }
    for (entity, other_pos) in &greeters {
        if let Ok((_, mut v, mut t, _, mut talk)) = villagers.get_mut(*entity) {
            v.act = Act::Greet;
            v.timer = GREET_SECS;
            v.greet_cooldown = GREET_COOLDOWN_SECS;
            talk.happy = true; // ac_npc_act_greeting: both feel happy
            let d = *other_pos - t.translation;
            if d.length_squared() > 1e-4 {
                t.rotation = Quat::from_rotation_y(d.x.atan2(d.z));
            }
        }
    }
    if !greeters.is_empty() {
        let names: Vec<&str> = greeters
            .iter()
            .filter_map(|(e, _)| villagers.get(*e).ok().map(|(_, v, ..)| v.name))
            .collect();
        println!("{} greeted each other", names.join(" and "));
    }

    let blocked = |p: Vec3, from: Vec3| {
        !town.can_step(from, Vec3::new(p.x, 0.0, p.z))
            || shop_building
                .as_ref()
                .is_some_and(|s| crate::shop::point_blocked(s, p.x, p.z))
            || p.distance(player_pos) < 1.2
    };

    for (entity, mut v, mut transform, mut vis, _) in villagers.iter_mut() {
        v.greet_cooldown -= dt;
        v.timer -= dt;
        let (target, speed) = match v.act {
            Act::Walk(t) => (Some(t), WALK_SPEED),
            Act::Run(t) => (Some(t), RUN_SPEED),
            Act::GoHome => (Some(v.door), WALK_SPEED),
            Act::Wait | Act::Greet => (None, 0.0),
        };

        // Arrived home: go inside.
        if v.act == Act::GoHome && (transform.translation.distance(v.door) < 0.5 || v.timer <= 0.0)
        {
            transform.translation = v.door;
            *vis = Visibility::Hidden;
            commands.entity(entity).insert(Indoors);
            println!("{} went inside", v.name);
            continue;
        }

        let arrived = target.is_some_and(|t| transform.translation.distance(t) < 0.4);
        if v.act != Act::GoHome && (v.timer <= 0.0 || arrived) {
            // aNPC_think_wander_decide_next with the personality borders.
            let schedule = schedule_now(v.looks);
            let act = npc_ai::wander_decide_next(
                v.looks as usize,
                rng.below(10) as i32,
                false,
                false,
                false,
                false,
                schedule,
                schedule,
            );
            let angle = rng.range_f32(0.0, std::f32::consts::TAU);
            let radius = rng.range_f32(1.0, WANDER_RADIUS);
            let spot = v.home + Vec3::new(angle.sin(), 0.0, angle.cos()) * radius;
            (v.act, v.timer) = match act {
                WanderAct::Wait => (Act::Wait, rng.range_f32(WAIT_SECS.0, WAIT_SECS.1)),
                WanderAct::Walk => (Act::Walk(spot), MOVE_TIMEOUT_SECS),
                WanderAct::Run => (Act::Run(spot), MOVE_TIMEOUT_SECS),
            };
            continue;
        }

        let Some(target) = target else {
            continue;
        };
        let to = Vec3::new(
            target.x - transform.translation.x,
            0.0,
            target.z - transform.translation.z,
        );
        let dist = to.length();
        if dist < 1e-4 {
            continue;
        }
        let dir = to / dist;
        let from = transform.translation;
        let step = dir * speed * dt;

        // Steer straight at the goal; when blocked, try the decomp's probe
        // angles two units ahead, then give up and re-think.
        let mut chosen = None;
        if !blocked(from + step, from) {
            chosen = Some(dir);
        } else {
            'probe: for a in AVOID_PROBE_ANGLES {
                for sign in [1.0, -1.0] {
                    let rot = Quat::from_rotation_y((a * sign).to_radians());
                    let d = rot * dir;
                    let ahead = from + d * AVOID_PROBE_DISTANCE_UNITS;
                    if !blocked(from + d * speed * dt, from) && !blocked(ahead, from) {
                        chosen = Some(d);
                        break 'probe;
                    }
                }
            }
        }
        match chosen {
            Some(d) => {
                transform.translation += d * speed * dt;
                transform.rotation = Quat::from_rotation_y(d.x.atan2(d.z));
            }
            None if v.act == Act::GoHome => {
                // Can't reach the door this way; the timer will put them inside.
            }
            None => v.timer = 0.0, // stuck: decide again next frame
        }
    }
}

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            spawn_villagers
                .after(crate::town::generate_town)
                .after(crate::house::find_player_house),
        )
        .add_systems(
            Update,
            (villager_schedule, villager_move)
                .chain()
                .in_set(crate::states::GameplaySet),
        );
    }
}
