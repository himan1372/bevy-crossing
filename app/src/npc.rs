//! Phase 2: villagers — spawn, wander, collide.
//!
//! Uses `rustimal_logic::npc` for personality data (the six decomp-verified
//! `mNpc_LOOKS_*` personalities) and the daily schedule check. The actual
//! wander movement is a simple Bevy-side random walk: `npc_ai`'s
//! interrupt-driven think system expects C-actor state that doesn't map
//! cleanly to Bevy components yet, so full think-system integration is
//! deferred to a later phase (see the migration sketch's "Needs Adaptation"
//! table).

use bevy::prelude::*;
use rustimal_logic::npc::{self, Personality};

use crate::player::Player;
use crate::town::Town;

/// Marker + data for a villager.
#[derive(Component)]
pub struct Villager {
    /// Display name.
    pub name: &'static str,
    /// Decomp personality (`mNpc_LOOKS_*` order) from the logic crate.
    pub personality: Personality,
    /// `looks` index, kept alongside for logic-crate calls.
    pub looks: u8,
    /// Where this villager considers "home" (wander stays near here).
    home: Vec3,
    /// Current wander destination.
    target: Vec3,
    /// Seconds until a new destination is picked.
    retarget_timer: f32,
    /// Movement speed.
    speed: f32,
}

/// Tiny xorshift RNG — avoids adding a `rand` dependency for wander picks.
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
}

/// (name, looks/personality index, capsule color).
const VILLAGERS: [(&str, u8, (f32, f32, f32)); 4] = [
    ("Maple", 0, (0.95, 0.55, 0.35)),  // normal — orange
    ("Pippy", 1, (0.95, 0.35, 0.55)),  // peppy — pink
    ("Bob", 2, (0.45, 0.75, 0.45)),    // lazy — green
    ("Chief", 4, (0.55, 0.45, 0.75)),  // cranky — purple
];

const SPAWN_POINTS: [Vec3; 4] = [
    Vec3::new(7.0, 1.0, 5.0),
    Vec3::new(-6.0, 1.0, 7.0),
    Vec3::new(5.0, 1.0, -7.0),
    Vec3::new(-7.0, 1.0, -6.0),
];

/// Spawn villagers after the town exists.
fn spawn_villagers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    town: Res<Town>,
) {
    commands.insert_resource(WanderRng(0x9E3779B97F4A7C15));

    // Snap a desired spawn point to the nearest walkable cell.
    let snap_walkable = |p: Vec3, town: &Town| -> Vec3 {
        if town.is_walkable(p) {
            return p;
        }
        for r in 1..=10 {
            for dx in -r..=r {
                for dz in [-r, r] {
                    let c = Vec3::new(p.x + dx as f32, p.y, p.z + dz as f32);
                    if town.is_walkable(c) {
                        return c;
                    }
                }
                for dz in -r..=r {
                    for dx in [-r, r] {
                        let c = Vec3::new(p.x + dx as f32, p.y, p.z + dz as f32);
                        if town.is_walkable(c) {
                            return c;
                        }
                    }
                }
            }
        }
        p
    };

    for (i, (name, looks, (r, g, b))) in VILLAGERS.iter().enumerate() {
        let personality = Personality::from_looks(*looks).unwrap_or(Personality::Girl);
        let home = snap_walkable(SPAWN_POINTS[i % SPAWN_POINTS.len()], &town);
        let mesh = meshes.add(Capsule3d::new(0.35, 0.9));
        let material = materials.add(Color::srgb(*r, *g, *b));

        commands.spawn((
            Villager {
                name,
                personality,
                looks: *looks,
                home,
                target: home,
                retarget_timer: 0.0,
                speed: 1.6,
            },
            PbrBundle {
                mesh,
                material,
                transform: Transform::from_translation(home),
                ..default()
            },
        ));

        // Sanity-check the schedule logic from the logic crate at spawn:
        // midday (12:00) should have most personalities out in the field.
        let state = npc::schedule_state_at(personality, 12 * 3600);
        println!(
            "Spawned villager {name} ({}) — midday schedule: {:?}",
            personality.english_name(),
            state
        );
    }
    println!("Villagers spawned");
}

/// Simple wander: pick a nearby target every few seconds, walk toward it,
/// respecting town collision like the player does.
fn villager_wander(
    mut rng: ResMut<WanderRng>,
    town: Res<Town>,
    shop_building: Option<Res<crate::shop::ShopBuilding>>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut villagers: Query<(&mut Villager, &mut Transform), Without<Player>>,
) {
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    for (mut villager, mut transform) in villagers.iter_mut() {
        villager.retarget_timer -= time.delta_seconds();
        let to_target = villager.target - transform.translation;
        let dist = to_target.length();

        if villager.retarget_timer <= 0.0 || dist < 0.4 {
            // Pick a new target within ~7 units of home.
            let dx = rng.range_f32(-7.0, 7.0);
            let dz = rng.range_f32(-7.0, 7.0);
            villager.target = Vec3::new(
                villager.home.x + dx,
                villager.home.y,
                villager.home.z + dz,
            );
            villager.retarget_timer = rng.range_f32(2.0, 5.0);
        } else {
            let dir = to_target / dist;
            let step = dir * villager.speed * time.delta_seconds();
            let candidate = transform.translation + step;

            // Don't walk into unwalkable cells, the shop building, or
            // through the player.
            let shop_blocked = shop_building.as_ref().map_or(false, |s| {
                crate::shop::point_blocked(s, candidate.x, candidate.z)
            });
            if town.is_walkable(Vec3::new(candidate.x, 0.0, candidate.z))
                && !shop_blocked
                && candidate.distance(player_pos) > 1.2
            {
                transform.translation = candidate;
                // Face the direction of travel.
                if dir.length_squared() > 0.0001 {
                    let yaw = dir.x.atan2(dir.z);
                    transform.rotation = Quat::from_rotation_y(yaw);
                }
            } else {
                // Blocked — pick a new target next frame.
                villager.retarget_timer = 0.0;
            }
        }
    }
}

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_villagers.after(crate::town::generate_town))
            .add_systems(Update, villager_wander);
    }
}
