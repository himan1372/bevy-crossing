//! Phase 3: ecology — fish shadows in the river, bugs near trees and on the
//! ground, rod/net tools, proximity-based catching.
//!
//! Species selection uses the decomp-verified seasonal tables in
//! `rustimal_logic::species` (`fish_seasonal_table` / `insect_seasonal_table`,
//! ported from `src/actor/ac_set_ovl_gyoei.c` and `ac_set_ovl_insect.c`):
//! the current real-world month/day/hour picks the table, entries are
//! filtered to the habitats this port renders (river / tree / ground /
//! flying), and one species is chosen by weight. The retail bite-timing
//! minigame (`ecology::BITE_TIMING`) and net-sweep geometry
//! (`ecology::net_capture_test`) are intentionally simplified for now:
//! catching is proximity + E. A timing-bar minigame can attach later.
//!
//! Controls: **1** = equip rod, **2** = equip net, **3** = unequip.
//! With the rod, **E** near water casts a bobber; if a fish shadow is close
//! when it bites, you catch it. With the net, **E** swings at the nearest
//! bug in reach. Catches increment the HUD fish/bug counters.

use bevy::prelude::*;
use rustimal_logic::species::{self, FishArea, FishType, InsectArea, InsectType};
use rustimal_logic::town_gen::{ACRE_WIDTH, CellKind};

use crate::dialogue::DialogueState;
use crate::house::HouseState;
use crate::interaction::{Facing, PlayerInventory};
use crate::player::Player;
use crate::town::Town;

/// How many fish shadows the river holds.
const FISH_TARGET: usize = 4;
/// How many bugs roam the town.
const BUG_TARGET: usize = 6;
/// Seconds between critter top-up checks.
const RESPAWN_SECS: f32 = 30.0;
/// Rod cast: how far ahead we look for water (facing cone samples).
const CAST_SAMPLES: [f32; 4] = [1.5, 2.5, 3.5, 4.5];
/// Max distance from the bobber a fish can be and still bite.
const BITE_RADIUS: f32 = 2.5;
/// Net reach.
const NET_RANGE: f32 = 3.0;
/// Facing-cone cosine threshold, matching `interaction.rs`.
const FACING_DOT: f32 = 0.2;

/// Which tool the player is holding. When not `None`, the E key is claimed
/// by this plugin and `interaction::interact` stands down.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum EquippedTool {
    #[default]
    None,
    Rod,
    Net,
}

/// A fish shadow drifting on the river.
#[derive(Component)]
struct Fish {
    species: FishType,
    dir: Vec3,
    home: Vec3,
    speed: f32,
}

/// A bug hopping/crawling/flying around the town.
#[derive(Component)]
struct Bug {
    insect: InsectType,
    target: Vec3,
    home: Vec3,
    pause: f32,
    speed: f32,
    /// Ground bugs stay on walkable tiles; flying bugs roam freely.
    grounded: bool,
}

/// A cast bobber waiting for a bite.
#[derive(Component)]
struct Bobber {
    timer: f32,
    bite_at: f32,
}

/// Tiny xorshift RNG — same pattern as `npc.rs`, no `rand` dependency.
#[derive(Resource)]
struct EcoRng(u64);

impl EcoRng {
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
    fn range_usize(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u32() as usize) % n
    }
}

/// Repeating top-up timer for fish/bug respawns.
#[derive(Resource)]
struct RespawnTimer(Timer);

/// Countdown for the catch notification text.
#[derive(Resource, Default)]
struct CatchNotifyTimer(f32);

/// Marker for the tool HUD text (top-left, under the inventory HUD).
#[derive(Component)]
struct ToolText;

/// Marker for the catch notification text (top-center, under save's).
#[derive(Component)]
struct CatchNotifyText;

// ---- date/time ----

/// Current (month, day, hour) from the system clock, via Howard Hinnant's
/// civil-from-days algorithm. Feeds the seasonal spawn tables.
fn today_mdh() -> (u8, u8, u8) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let hour = ((secs % 86_400) / 3600) as u8;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u8; // [1, 12]
    (m, d, hour)
}

/// Turn a `Debug`-formatted enum name ("CrucianCarp") into "Crucian Carp".
fn pretty_name(debug: &str) -> String {
    let mut out = String::new();
    for (i, c) in debug.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

/// Weighted pick from (value, weight) pairs. Returns None on empty/zero total.
fn weighted_pick<T: Copy>(entries: &[(T, u8)], rng: &mut EcoRng) -> Option<T> {
    let total: u32 = entries.iter().map(|(_, w)| *w as u32).sum();
    if total == 0 {
        return None;
    }
    let mut roll = rng.next_u32() % total;
    for (v, w) in entries {
        let w = *w as u32;
        if roll < w {
            return Some(*v);
        }
        roll -= w;
    }
    None
}

/// Pick a river fish species from the seasonal table (env 0 = river).
fn pick_fish(rng: &mut EcoRng) -> FishType {
    let (m, d, h) = today_mdh();
    let entries: Vec<(FishType, u8)> = species::fish_seasonal_table(m, d, h, 0)
        .map(|t| {
            t.iter()
                .filter(|e| e.area == FishArea::River)
                .map(|e| (e.fish, e.weight))
                .collect()
        })
        .unwrap_or_default();
    weighted_pick(&entries, rng).unwrap_or(FishType::CrucianCarp)
}

/// Pick a bug species + renderable habitat from the seasonal table.
fn pick_bug(rng: &mut EcoRng) -> (InsectType, InsectArea) {
    let (m, _, h) = today_mdh();
    let entries: Vec<(InsectType, u8)> = species::insect_seasonal_table(m, h)
        .iter()
        .filter(|e| {
            matches!(
                e.area,
                InsectArea::OnTree | InsectArea::OnGround | InsectArea::Flying
            )
        })
        .map(|e| (e.insect, e.weight))
        .collect();
    let insect = weighted_pick(&entries, rng).unwrap_or(InsectType::CommonButterfly);
    // Recover the habitat for the chosen insect (first matching entry).
    let area = species::insect_seasonal_table(m, h)
        .iter()
        .find(|e| e.insect == insect)
        .map(|e| e.area)
        .unwrap_or(InsectArea::OnGround);
    let area = match area {
        InsectArea::OnTree | InsectArea::OnGround | InsectArea::Flying => area,
        _ => InsectArea::OnGround,
    };
    (insect, area)
}

// ---- spawning ----

/// World positions of every tree in the town (excluding trees on water).
fn tree_spots(town: &Town) -> Vec<Vec3> {
    let mut spots = Vec::new();
    for (acre_idx, acre) in town.plan.acres.iter().enumerate() {
        let ax = acre_idx % ACRE_WIDTH;
        let az = acre_idx / ACRE_WIDTH;
        for cz in 0..16 {
            for cx in 0..16 {
                if acre.cells[cz * 16 + cx] != CellKind::Tree as u8 {
                    continue;
                }
                if town.river_tiles.contains(&(ax, az, cx, cz)) {
                    continue;
                }
                spots.push(Town::cell_to_world(ax, az, cx, cz));
            }
        }
    }
    spots
}

fn spawn_fish(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    town: &Town,
    rng: &mut EcoRng,
    count: usize,
) {
    let tiles: Vec<(usize, usize, usize, usize)> = town.river_tiles.iter().copied().collect();
    if tiles.is_empty() {
        return;
    }
    let mesh = meshes.add(Sphere::new(0.45));
    let mat = materials.add(Color::srgb(0.08, 0.12, 0.2));
    for _ in 0..count {
        let (ax, az, cx, cz) = tiles[rng.range_usize(tiles.len())];
        let pos = Town::cell_to_world(ax, az, cx, cz);
        let angle = rng.range_f32(0.0, std::f32::consts::TAU);
        let species = pick_fish(rng);
        commands.spawn((
            Fish {
                species,
                dir: Vec3::new(angle.cos(), 0.0, angle.sin()),
                home: pos,
                speed: rng.range_f32(0.3, 0.7),
            },
            PbrBundle {
                mesh: mesh.clone(),
                material: mat.clone(),
                transform: Transform {
                    translation: Vec3::new(pos.x, 0.12, pos.z),
                    scale: Vec3::new(1.2, 0.22, 0.7),
                    ..default()
                },
                ..default()
            },
        ));
    }
}

fn spawn_bugs(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    town: &Town,
    rng: &mut EcoRng,
    count: usize,
) {
    let trees = tree_spots(town);
    if trees.is_empty() {
        return;
    }
    let mesh = meshes.add(Sphere::new(0.16));
    for _ in 0..count {
        let (insect, area) = pick_bug(rng);
        let tree = trees[rng.range_usize(trees.len())];
        let shade = rng.range_f32(-0.03, 0.05);
        let mat = materials.add(Color::srgb(0.2 + shade, 0.14 + shade, 0.08));
        let (pos, grounded) = match area {
            InsectArea::OnTree => (Vec3::new(tree.x, 1.6, tree.z), false),
            InsectArea::Flying => {
                let p = find_walkable_near(town, rng, tree, 6.0).unwrap_or(tree);
                (Vec3::new(p.x, 1.3, p.z), false)
            }
            _ => {
                let p = find_walkable_near(town, rng, tree, 4.0).unwrap_or(tree);
                (Vec3::new(p.x, 0.2, p.z), true)
            }
        };
        commands.spawn((
            Bug {
                insect,
                target: pos,
                home: pos,
                pause: rng.range_f32(0.5, 2.5),
                speed: rng.range_f32(0.8, 1.6),
                grounded,
            },
            PbrBundle {
                mesh: mesh.clone(),
                material: mat,
                transform: Transform::from_translation(pos),
                ..default()
            },
        ));
    }
}

/// Try a few random offsets around `center`; return the first walkable one.
fn find_walkable_near(town: &Town, rng: &mut EcoRng, center: Vec3, radius: f32) -> Option<Vec3> {
    for _ in 0..8 {
        let angle = rng.range_f32(0.0, std::f32::consts::TAU);
        let r = rng.range_f32(1.0, radius);
        let p = Vec3::new(
            center.x + angle.cos() * r,
            0.0,
            center.z + angle.sin() * r,
        );
        if town.is_walkable(p) {
            return Some(p);
        }
    }
    None
}

/// Startup: RNG resource + initial critters (runs after town generation).
fn spawn_initial_critters(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    town: Res<Town>,
) {
    let mut rng = EcoRng(0xC0FFEE1234567890);
    spawn_fish(&mut commands, &mut meshes, &mut materials, &town, &mut rng, FISH_TARGET);
    spawn_bugs(&mut commands, &mut meshes, &mut materials, &town, &mut rng, BUG_TARGET);
    commands.insert_resource(rng);
    println!("Ecology spawned: {FISH_TARGET} fish, {BUG_TARGET} bugs");
}

// ---- UI ----

/// Tool indicator + catch notification UI.
fn setup_ecology_ui(mut commands: Commands) {
    // Tool indicator — top-left, under the inventory HUD.
    commands.spawn((
        ToolText,
        TextBundle {
            text: Text::from_section(
                "Tool: none (1=rod 2=net)",
                TextStyle {
                    font_size: 18.0,
                    color: Color::srgba(1.0, 1.0, 1.0, 0.85),
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(40.0),
                left: Val::Px(12.0),
                ..default()
            },
            ..default()
        },
    ));

    // Catch notification — top-center, below the save notification.
    let mut text = Text::from_section(
        "",
        TextStyle {
            font_size: 26.0,
            color: Color::WHITE,
            ..default()
        },
    );
    text.justify = bevy::text::JustifyText::Center;
    commands.spawn((
        CatchNotifyText,
        TextBundle {
            text,
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(110.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                ..default()
            },
            ..default()
        },
    ));
}

fn show_catch_notify(
    msg: &str,
    timer: &mut ResMut<CatchNotifyTimer>,
    query: &mut Query<&mut Text, With<CatchNotifyText>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].value = msg.to_string();
    }
    timer.0 = 2.5;
}

fn tick_catch_notify(
    time: Res<Time>,
    mut timer: ResMut<CatchNotifyTimer>,
    mut query: Query<&mut Text, With<CatchNotifyText>>,
) {
    if timer.0 > 0.0 {
        timer.0 -= time.delta_seconds();
        if timer.0 <= 0.0 {
            if let Ok(mut text) = query.get_single_mut() {
                text.sections[0].value.clear();
            }
        }
    }
}

fn update_tool_hud(tool: Res<EquippedTool>, mut query: Query<&mut Text, With<ToolText>>) {
    if !tool.is_changed() {
        return;
    }
    let label = match *tool {
        EquippedTool::None => "Tool: none (1=rod 2=net)",
        EquippedTool::Rod => "Tool: Fishing Rod (E to cast near water)",
        EquippedTool::Net => "Tool: Bug Net (E to swing at bugs)",
    };
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].value = label.to_string();
    }
}

// ---- behavior ----

/// Fish drift along the river; turn around at banks or past their home range.
fn update_fish(
    time: Res<Time>,
    town: Res<Town>,
    mut query: Query<(&mut Fish, &mut Transform)>,
    mut rng: ResMut<EcoRng>,
) {
    for (mut fish, mut transform) in query.iter_mut() {
        let step = fish.dir * fish.speed * time.delta_seconds();
        let next = transform.translation + step;
        let in_range = (next - fish.home).length() < 9.0;
        if town.is_river(next) && in_range {
            transform.translation = next;
        } else {
            let angle = rng.range_f32(0.0, std::f32::consts::TAU);
            fish.dir = Vec3::new(angle.cos(), 0.0, angle.sin());
        }
    }
}

/// Bugs hop toward a target, pause, then pick a new one.
fn update_bugs(
    time: Res<Time>,
    town: Res<Town>,
    mut query: Query<(&mut Bug, &mut Transform)>,
    mut rng: ResMut<EcoRng>,
) {
    for (mut bug, mut transform) in query.iter_mut() {
        if bug.pause > 0.0 {
            bug.pause -= time.delta_seconds();
            continue;
        }
        let to_target = bug.target - transform.translation;
        if to_target.length() < 0.15 {
            bug.pause = rng.range_f32(1.0, 3.0);
            let angle = rng.range_f32(0.0, std::f32::consts::TAU);
            let r = rng.range_f32(1.0, 3.0);
            let mut next = transform.translation + Vec3::new(angle.cos() * r, 0.0, angle.sin() * r);
            // Drifted too far from home: head back.
            if (next - bug.home).length() > 10.0 {
                next = bug.home;
            }
            next.y = transform.translation.y;
            if !bug.grounded || town.is_walkable(next) {
                bug.target = next;
            } else {
                bug.target = transform.translation;
            }
            continue;
        }
        transform.translation += to_target.normalize() * bug.speed * time.delta_seconds();
    }
}

/// 1/2/3 to equip rod / net / nothing.
fn tool_select(
    keyboard: Res<ButtonInput<KeyCode>>,
    shop_loc: Res<State<crate::shop::ShopLocation>>,
    shop_ui: Res<crate::shop::ShopUi>,
    mut tool: ResMut<EquippedTool>,
) {
    // No tools inside the shop or while its menu is open — the shop
    // force-unequips on entry, and this keeps it that way.
    if *shop_loc != crate::shop::ShopLocation::Town || shop_ui.open {
        return;
    }
    if keyboard.just_pressed(KeyCode::Digit1) {
        *tool = EquippedTool::Rod;
        println!("Equipped fishing rod");
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        *tool = EquippedTool::Net;
        println!("Equipped bug net");
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        *tool = EquippedTool::None;
        println!("Unequipped tool");
    }
}

/// E with a tool equipped: cast the rod near water, or swing the net at bugs.
#[allow(clippy::too_many_arguments)]
fn tool_interact(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<HouseState>>,
    tool: Res<EquippedTool>,
    dialogue: Res<DialogueState>,
    mail_ui: Res<crate::mail::MailUi>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<(&Transform, &Facing), With<Player>>,
    town: Res<Town>,
    bug_query: Query<(Entity, &Bug, &Transform)>,
    bobber_query: Query<Entity, With<Bobber>>,
    mut inventory: ResMut<PlayerInventory>,
    mut rng: ResMut<EcoRng>,
    mut notify_timer: ResMut<CatchNotifyTimer>,
    mut notify_query: Query<&mut Text, With<CatchNotifyText>>,
) {
    // Inside the house, E belongs to the house doors. (The shop
    // force-unequips tools on entry, so no shop check is needed here.)
    if *state != HouseState::Town {
        return;
    }
    if *tool == EquippedTool::None || dialogue.active {
        return;
    }
    // The mail UI claims the E key while open.
    if mail_ui.open {
        return;
    }
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok((player_transform, facing)) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation;

    match *tool {
        EquippedTool::Rod => {
            // Reel in if a bobber is already out.
            if let Ok(bobber) = bobber_query.get_single() {
                commands.entity(bobber).despawn();
                show_catch_notify("Reeled in.", &mut notify_timer, &mut notify_query);
                return;
            }
            // Find the first river tile along the facing direction.
            let mut cast_point = None;
            for d in CAST_SAMPLES {
                let p = player_pos + facing.dir * d;
                if town.is_river(p) {
                    cast_point = Some(p);
                    break;
                }
            }
            let Some(cp) = cast_point else {
                show_catch_notify("No water in front of you.", &mut notify_timer, &mut notify_query);
                return;
            };
            let bite_at = rng.range_f32(1.5, 3.5);
            commands.spawn((
                Bobber { timer: 0.0, bite_at },
                PbrBundle {
                    mesh: meshes.add(Sphere::new(0.18)),
                    material: materials.add(Color::srgb(0.9, 0.15, 0.1)),
                    transform: Transform::from_xyz(cp.x, 0.35, cp.z),
                    ..default()
                },
            ));
            println!("Cast! (bite in {bite_at:.1}s)");
        }
        EquippedTool::Net => {
            let mut best: Option<(Entity, f32)> = None;
            for (entity, _bug, bug_transform) in bug_query.iter() {
                let to_bug = bug_transform.translation - player_pos;
                let dist = to_bug.length();
                if dist > NET_RANGE || dist < 0.001 {
                    continue;
                }
                if facing.dir.dot(to_bug / dist) < FACING_DOT {
                    continue;
                }
                if best.map_or(true, |(_, d)| dist < d) {
                    best = Some((entity, dist));
                }
            }
            match best {
                Some((entity, _)) => {
                    let insect = bug_query.get(entity).map(|(_, b, _)| b.insect).ok();
                    commands.entity(entity).despawn();
                    inventory.bugs += 1;
                    let name = insect
                        .map(|i| pretty_name(&format!("{i:?}")))
                        .unwrap_or_else(|| "Bug".to_string());
                    show_catch_notify(
                        &format!("Caught a {name}!"),
                        &mut notify_timer,
                        &mut notify_query,
                    );
                    println!("Caught a {name} (total bugs: {})", inventory.bugs);
                }
                None => {
                    show_catch_notify("No bugs in reach.", &mut notify_timer, &mut notify_query);
                }
            }
        }
        EquippedTool::None => {}
    }
}

/// Bobbers wait for their bite time, then resolve the catch.
#[allow(clippy::too_many_arguments)]
fn update_bobber(
    time: Res<Time>,
    mut commands: Commands,
    mut bobber_query: Query<(Entity, &mut Bobber, &Transform)>,
    fish_query: Query<(Entity, &Fish, &Transform)>,
    mut inventory: ResMut<PlayerInventory>,
    mut notify_timer: ResMut<CatchNotifyTimer>,
    mut notify_query: Query<&mut Text, With<CatchNotifyText>>,
) {
    for (entity, mut bobber, bobber_transform) in bobber_query.iter_mut() {
        bobber.timer += time.delta_seconds();
        if bobber.timer < bobber.bite_at {
            continue;
        }
        let mut best: Option<(Entity, f32)> = None;
        for (fish_entity, _fish, fish_transform) in fish_query.iter() {
            let dist = (fish_transform.translation - bobber_transform.translation).length();
            if dist < BITE_RADIUS && best.map_or(true, |(_, d)| dist < d) {
                best = Some((fish_entity, dist));
            }
        }
        match best {
            Some((fish_entity, _)) => {
                let species = fish_query
                    .get(fish_entity)
                    .map(|(_, f, _)| f.species)
                    .ok();
                commands.entity(fish_entity).despawn();
                inventory.fish += 1;
                let name = species
                    .map(|s| pretty_name(&format!("{s:?}")))
                    .unwrap_or_else(|| "Fish".to_string());
                show_catch_notify(
                    &format!("Caught a {name}!"),
                    &mut notify_timer,
                    &mut notify_query,
                );
                println!("Caught a {name} (total fish: {})", inventory.fish);
            }
            None => {
                show_catch_notify("Nothing bit...", &mut notify_timer, &mut notify_query);
            }
        }
        commands.entity(entity).despawn();
    }
}

/// Every 30s, top fish/bugs back up to their target counts.
#[allow(clippy::too_many_arguments)]
fn respawn_critters(
    time: Res<Time>,
    mut timer: ResMut<RespawnTimer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    town: Res<Town>,
    mut rng: ResMut<EcoRng>,
    fish_query: Query<&Fish>,
    bug_query: Query<&Bug>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    let fish_count = fish_query.iter().len();
    let bug_count = bug_query.iter().len();
    if fish_count < FISH_TARGET {
        spawn_fish(
            &mut commands,
            &mut meshes,
            &mut materials,
            &town,
            &mut rng,
            FISH_TARGET - fish_count,
        );
    }
    if bug_count < BUG_TARGET {
        spawn_bugs(
            &mut commands,
            &mut meshes,
            &mut materials,
            &town,
            &mut rng,
            BUG_TARGET - bug_count,
        );
    }
    if fish_count < FISH_TARGET || bug_count < BUG_TARGET {
        println!("Respawned critters (fish: {fish_count}, bugs: {bug_count})");
    }
}

pub struct EcologyPlugin;

impl Plugin for EcologyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EquippedTool::default())
            .insert_resource(CatchNotifyTimer::default())
            .insert_resource(RespawnTimer(Timer::from_seconds(
                RESPAWN_SECS,
                TimerMode::Repeating,
            )))
            .add_systems(Startup, setup_ecology_ui)
            .add_systems(
                Startup,
                spawn_initial_critters.after(crate::town::generate_town),
            )
            .add_systems(
                Update,
                (
                    tool_select,
                    tool_interact,
                    update_fish,
                    update_bugs,
                    update_bobber,
                    respawn_critters,
                    update_tool_hud,
                    tick_catch_notify,
                )
                    .chain(),
            );
    }
}
