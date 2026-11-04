//! Gulliver (`SP_NPC_EV_DOZAEMON`), the seagull castaway who washes up on the beach.
//!
//! He is out while the original scheduler says `mEv_EVENT_DOZAEMON` is active (06:00-22:00 on the week's saved
//! Gulliver day, see `event_schedule`), until his errand is done (`gulliver_completed`).
//!
//! Placement (`make_actor_in_seaside_block` / `search_seaside_unit` in `ac_event_manager.c`): the bottom playable
//! row of blocks, block x = 1 + (seed + day + sec) % (xmax - 2), and a random wave-attribute unit of that block
//! with x and z in 1..=14 (`mFI_GetWaveUtinBlock`). The original rolls `RANDOM` and uses the clock's seconds, and
//! saves the chosen place; the port derives the pick from the game day instead, so a reload puts him in the same
//! spot. Waves are not walkable, so he stands on the nearest clear ground to that unit.
//!
//! Not built yet: the dock block exclusion (`dock_block` is not modelled), talking to him, the wake-up and the
//! furniture gift, and his lying / standing animations.

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::town_gen::{ACRE_DEPTH, ACRE_WIDTH};

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::event_today::TodayEvents;
use crate::shop::ShopLocation;
use crate::states::GameplaySet;
use crate::town::Town;

/// ALL_NPC_NUM + (SP_NPC_EV_DOZAEMON - SP_NPC_START).
const GULLIVER_NPC: u32 = 238 + 100;

#[derive(Component)]
struct Gulliver {
    /// The game day he came out for.
    day: i64,
}

/// Cheap integer mixer for the per-day picks.
fn mix(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ce_b9fe_1a85_1a85);
    x ^ (x >> 33)
}

/// The wave units of playable column `ax` in the bottom row, with x and z in 1..=14.
fn wave_units(town: &Town, ax: usize) -> Vec<(usize, usize)> {
    let Some(ground) = town.ground.as_ref() else {
        return Vec::new();
    };
    let az = ACRE_DEPTH - 1;
    let mut out = Vec::new();
    for cz in 1..15 {
        for cx in 1..15 {
            let at = Town::cell_to_world(ax, az, cx, cz);
            if ground.is_wave(at.x, at.z) {
                out.push((cx, cz));
            }
        }
    }
    out
}

/// Where he washes up on `day`: the nearest clear ground to a wave unit of the day's block.
fn beach_spot(town: &Town, day: i64) -> Option<Vec3> {
    let h = mix(day as u64);
    // The columns, starting from the day's pick, until one has waves (`search_seaside_unit` retries blocks).
    for step in 0..ACRE_WIDTH {
        let ax = (h as usize % ACRE_WIDTH + step) % ACRE_WIDTH;
        let waves = wave_units(town, ax);
        if waves.is_empty() {
            continue;
        }
        let (cx, cz) = waves[(mix(h) as usize) % waves.len()];
        let wave = Town::cell_to_world(ax, ACRE_DEPTH - 1, cx, cz);
        let mut best: Option<(f32, Vec3)> = None;
        for dz in -10i32..=4 {
            for dx in -10i32..=10 {
                let at = wave + Vec3::new(dx as f32, 0.0, dz as f32);
                if !town.is_clear_ground(at) || town.in_retail_obstacle(at) {
                    continue;
                }
                let d = (at - wave).length();
                if best.is_none_or(|(b, _)| d < b) {
                    best = Some((d, at));
                }
            }
        }
        if let Some((_, at)) = best {
            return Some(at.with_y(town.ground_y(at)));
        }
    }
    None
}

/// Brings him out for the day and sends him off when the event ends.
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Gulliver)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let out = today.is_active(ev::DOZAEMON, now.hour)
        && !today.gulliver_completed()
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    for (entity, g) in &existing {
        if !out || g.day != now.day_number {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    let Some(at) = beach_spot(&town, now.day_number) else {
        return;
    };
    commands.spawn((
        Name::new("Gulliver"),
        Gulliver {
            day: now.day_number,
        },
        NpcClip::default(),
        NpcGait::default(),
        NpcModel(GULLIVER_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        // Facing the sea (south).
        Transform::from_translation(at + Vec3::Y),
    ));
    info!("Gulliver washed up at {at:?}");
}

pub struct GulliverPlugin;

impl Plugin for GulliverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            visit.after(crate::event_today::refresh).in_set(GameplaySet),
        );
    }
}
