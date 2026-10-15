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
use bevy::text::FontSize;
use rustimal_logic::species::{self, FishArea, FishType, InsectArea, InsectType};
use rustimal_logic::town_gen::ACRE_WIDTH;
use rustimal_logic::{gyo_ai, ins_ai};

use crate::house::HouseState;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::items;
use crate::player::Player;
use crate::states::GameplaySet;
use crate::town::Town;

/// How far above the river bed a shadow swims: the water surface is 20
/// game units up (`mCoBG_GetWaterHeight`) and fish 8 below it
/// (`aGTT_Get_water_surface_position_y`), 12 of a unit's 40.
const FISH_LIFT: f32 = 12.0 / 40.0;
/// Rod cast: how far ahead we look for water (facing cone samples).
const CAST_SAMPLES: [f32; 4] = [1.5, 2.5, 3.5, 4.5];
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
pub(crate) struct Fish {
    pub(crate) species: FishType,
    pub(crate) dir: Vec3,
    home: Vec3,
    /// `ac_gyo_test` behaviour, made on its first tick in the water.
    ai: Option<gyo_ai::Gyo>,
    /// 60 Hz ticks not yet run.
    ticks: f32,
}

/// A bug hopping/crawling/flying around the town.
#[derive(Component)]
pub(crate) struct Bug {
    pub(crate) insect: InsectType,
    pub(crate) target: Vec3,
    home: Vec3,
    pause: f32,
    speed: f32,
    /// Ground bugs stay on walkable tiles; flying bugs roam freely.
    grounded: bool,
    /// Height above the ground.
    lift: f32,
    /// `ac_ins_*` behaviour (butterflies so far), made on its first tick.
    pub(crate) ai: Option<ins_ai::Insect>,
    /// 60 Hz ticks not yet run.
    ticks: f32,
}

/// Where the last net swing came down (`mPlib_Check_StopNet`), for the
/// bugs to notice once.
#[derive(Resource, Default)]
struct NetSwing(Option<Vec3>);

/// The town as an insect sees it, in game units (scene x 40).
struct BugField<'a> {
    town: &'a Town,
    items: Option<&'a crate::field_items::FieldItems>,
}

impl ins_ai::Field for BugField<'_> {
    fn ground_y(&self, x: f32, z: f32) -> f32 {
        self.town.ground_y(Vec3::new(x, 0.0, z) / GAME_UNITS) * GAME_UNITS
    }
    fn water_y(&self, x: f32, z: f32) -> Option<f32> {
        let g = self.town.ground.as_ref()?;
        let (sx, sz) = (x / GAME_UNITS, z / GAME_UNITS);
        // fish_y is 8 under the surface.
        g.is_fish_water(sx, sz)
            .then(|| g.fish_y(sx, sz))
            .flatten()
            .map(|y| y * GAME_UNITS + 8.0)
    }
    fn unit_of(&self, x: f32, z: f32) -> Option<[i32; 4]> {
        Town::cell_indices_of(Vec3::new(x, 0.0, z) / GAME_UNITS)
            .map(|(a, b, c, d)| [a as i32, b as i32, c as i32, d as i32])
    }
    fn unit_center(&self, u: [i32; 4]) -> [f32; 2] {
        let [a, b, c, d] = u.map(|v| v.max(0) as usize);
        let p = Town::cell_to_world(a, b, c.min(15), d.min(15)) * GAME_UNITS;
        [p.x, p.z]
    }
    fn item(&self, u: [i32; 4]) -> Option<u16> {
        let c = self.unit_center(u);
        match self.items {
            Some(items) => items
                .at(Vec3::new(c[0], 0.0, c[1]) / GAME_UNITS)
                .map(|(_, _, item)| item)
                .filter(|&item| item != 0xFFFF),
            None => Some(0),
        }
    }
    fn attribute(&self, x: f32, z: f32) -> Option<u8> {
        self.town
            .ground
            .as_ref()?
            .attribute_at(x / GAME_UNITS, z / GAME_UNITS)
    }
    fn blocked(&self, from: [f32; 3], to: [f32; 3]) -> bool {
        self.ground_y(to[0], to[2]) > from[1]
    }
}

/// A cast bobber waiting for a bite.
#[derive(Component)]
struct Bobber {
    timer: f32,
    /// The fish that has hold of it, and whether it's biting now.
    hooked: Option<Entity>,
    biting: bool,
}

/// Game units per scene unit (one unit is 40).
const GAME_UNITS: f32 = 40.0;
/// The bobber settles this long after landing (`cast_timer`); a fish
/// within reach of the splash before then flees (`hit_water_flag`).
const BOBBER_SETTLE_SECS: f32 = 0.5;

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

/// Current (month, day, hour) on the game clock (local time). Feeds the
/// seasonal spawn tables.
pub(crate) fn today_mdh() -> (u8, u8, u8) {
    let n = crate::clock::now();
    (n.month, n.day, n.hour)
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

// ---- spawning ----

/// One fish shadow at `pos` (the critters module dresses it).
fn spawn_one_fish(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    species: FishType,
    pos: Vec3,
) {
    commands.spawn((
        Fish {
            species,
            dir: Vec3::Z,
            home: pos,
            ai: None,
            ticks: 0.0,
        },
        Mesh3d(meshes.add(Sphere::new(0.45))),
        MeshMaterial3d(materials.add(Color::srgb(0.08, 0.12, 0.2))),
        Transform {
            translation: pos,
            scale: Vec3::new(1.2, 0.22, 0.7),
            ..default()
        },
    ));
}

/// Which kinds of water an acre's units hold (`aSOG_gyoei_place_check`,
/// USA rules): river (and river mouth), still water (pool, pond), falls,
/// sea.
fn acre_water(ground: &crate::ground_map::GroundMap, ax: usize, az: usize) -> [bool; 4] {
    let mut kinds = [false; 4];
    for ut in 0..256 {
        match ground.unit_attribute(ax + 1, az + 1, ut) {
            14..=21 => kinds[0] = true,
            12 => kinds[1] = true,
            13 => kinds[2] = true,
            24 => kinds[3] = true,
            _ => {}
        }
    }
    kinds
}

/// A fish's height over the water at `p`.
fn fish_height(town: &Town, p: Vec3) -> f32 {
    town.ground
        .as_ref()
        .and_then(|g| g.fish_y(p.x, p.z))
        .unwrap_or_else(|| town.ground_y(p) + FISH_LIFT)
}

/// Water a fish or bobber can be in.
fn fish_water(town: &Town, p: Vec3) -> bool {
    town.ground
        .as_ref()
        .map_or_else(|| town.is_river(p), |g| g.is_fish_water(p.x, p.z))
}

/// `aSOG_gyoei_set`: when the player comes into an acre with water and no
/// fish, one fish of the season (whose area the acre has) appears on a
/// random water unit at least 2 units in from the edge
/// (`aSOG_gyoei_set_gyoei_data`; the Large Char only in falls). Fish more
/// than an acre from the player are let go.
#[allow(clippy::too_many_arguments)]
fn place_fish(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    town: Res<Town>,
    player: Query<&Transform, With<Player>>,
    fish: Query<(Entity, &Transform), With<Fish>>,
    mut rng: ResMut<EcoRng>,
    mut last: Local<Option<(usize, usize)>>,
) {
    let (Some(ground), Ok(p)) = (&town.ground, player.single()) else {
        return;
    };
    let Some((ax, az, _, _)) = Town::cell_indices_of(p.translation) else {
        return;
    };
    if *last == Some((ax, az)) {
        return;
    }
    *last = Some((ax, az));
    let acre_of = |t: &Transform| Town::cell_indices_of(t.translation).map(|(x, z, ..)| (x, z));
    for (e, t) in &fish {
        if acre_of(t).is_none_or(|(x, z)| x.abs_diff(ax) > 1 || z.abs_diff(az) > 1) {
            commands.entity(e).despawn();
        }
    }
    if fish.iter().any(|(_, t)| acre_of(t) == Some((ax, az))) {
        return;
    }
    let [river, still, falls, sea] = acre_water(ground, ax, az);
    let (m, d, h) = today_mdh();
    // The river table, plus the sea's for coastal acres and the pond's
    // for still water (aSOG_gyoei_make_*_range_data).
    let tables = [(0, true), (1, sea), (2, still)];
    let entries: Vec<(FishType, u8)> = tables
        .iter()
        .filter(|(_, on)| *on)
        .filter_map(|&(env, _)| species::fish_seasonal_table(m, d, h, env))
        .flat_map(|t| {
            t.iter()
                .filter(|e| match e.area {
                    FishArea::River | FishArea::RiverMouth => river,
                    FishArea::Pool | FishArea::Pond => still,
                    FishArea::Waterfall => falls,
                    FishArea::Sea => sea,
                    FishArea::Offing => false,
                })
                .map(|e| (e.fish, e.weight))
        })
        .collect();
    let Some(species) = weighted_pick(&entries, &mut rng) else {
        return;
    };
    // Per species as aSOG_gyoei_set_gyoei_data: Large Char in falls, sea
    // fish in deep sea, salmon in either, the whale well out, the rest in
    // fresh water.
    let deep = |ut: usize| {
        let c = Town::cell_to_world(ax, az, ut % 16, ut / 16);
        ground.deep_sea(c.x, c.z)
    };
    let sea_fish = matches!(
        species,
        FishType::Coelacanth
            | FishType::Jellyfish
            | FishType::SeaBass
            | FishType::RedSnapper
            | FishType::BarredKnifejaw
    );
    let spots: Vec<usize> = (0..256)
        .filter(|&ut| (2..14).contains(&(ut % 16)) && (2..14).contains(&(ut / 16)))
        .filter(|&ut| {
            let fresh = (12..=21).contains(&ground.unit_attribute(ax + 1, az + 1, ut));
            match species {
                FishType::LargeChar => ground.unit_attribute(ax + 1, az + 1, ut) == 13,
                FishType::Salmon | FishType::Salmon2 => fresh || deep(ut),
                FishType::Whale => {
                    (5..11).contains(&(ut % 16))
                        && (5..11).contains(&(ut / 16))
                        && (fresh || deep(ut))
                }
                _ if sea_fish => deep(ut),
                _ => fresh,
            }
        })
        .collect();
    if spots.is_empty() {
        return;
    }
    let ut = spots[rng.range_usize(spots.len())];
    let at = Town::cell_to_world(ax, az, ut % 16, ut / 16);
    let pos = at.with_y(fish_height(&town, at));
    spawn_one_fish(&mut commands, &mut meshes, &mut materials, species, pos);
    println!("A {species:?} in acre ({ax}, {az})");
}

fn spawn_one_bug(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    town: &Town,
    rng: &mut EcoRng,
    (insect, area): (InsectType, InsectArea),
    at: Vec3,
) {
    let (grounded, lift) = match area {
        InsectArea::OnTree => (false, 1.6),
        InsectArea::Flying
        | InsectArea::FlyingNearWater
        | InsectArea::FlyingNearFlowersOrAround => (false, 1.3),
        InsectArea::OnFlower | InsectArea::RainingOnFlower => (false, 0.5),
        InsectArea::OnWater => (false, 0.05),
        _ => (true, 0.05),
    };
    let pos = at.with_y(town.ground_y(at) + lift);
    let shade = rng.range_f32(-0.03, 0.05);
    commands.spawn((
        Bug {
            insect,
            target: pos,
            home: pos,
            pause: rng.range_f32(0.5, 2.5),
            speed: rng.range_f32(0.8, 1.6),
            grounded,
            lift,
            ai: None,
            ticks: 0.0,
        },
        Mesh3d(meshes.add(Sphere::new(0.16))),
        MeshMaterial3d(materials.add(Color::srgb(0.2 + shade, 0.14 + shade, 0.08))),
        Transform::from_translation(pos),
    ));
}

/// `aSOI_insect_set`: when the player comes into an acre with no insect,
/// the season's table decides what hatches there and on which units
/// (`rustimal_logic::ins_set`). Insects more than an acre away go.
#[allow(clippy::too_many_arguments)]
fn place_bugs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    town: Res<Town>,
    items: Option<Res<crate::field_items::FieldItems>>,
    player: Query<&Transform, With<Player>>,
    bugs: Query<(Entity, &Transform), With<Bug>>,
    mut rng: ResMut<EcoRng>,
    mut last: Local<Option<(usize, usize)>>,
) {
    let (Some(ground), Some(items), Ok(p)) = (&town.ground, items, player.single()) else {
        return;
    };
    let Some((ax, az, _, _)) = Town::cell_indices_of(p.translation) else {
        return;
    };
    if *last == Some((ax, az)) {
        return;
    }
    *last = Some((ax, az));
    let acre_of = |t: &Transform| Town::cell_indices_of(t.translation).map(|(x, z, ..)| (x, z));
    for (e, t) in &bugs {
        if acre_of(t).is_none_or(|(x, z)| x.abs_diff(ax) > 1 || z.abs_diff(az) > 1) {
            commands.entity(e).despawn();
        }
    }
    if bugs.iter().any(|(_, t)| acre_of(t) == Some((ax, az))) {
        return;
    }
    let Some(acre_items) = items.acres.get(az * ACRE_WIDTH + ax) else {
        return;
    };
    let attrs: [u8; 256] = std::array::from_fn(|ut| ground.unit_attribute(ax + 1, az + 1, ut));
    let has = |f: &dyn Fn(u8) -> bool| attrs.iter().any(|&a| f(a));
    let acre = rustimal_logic::ins_set::Acre {
        items: acre_items,
        attrs: &attrs,
        pool: has(&|a| a == 12),
        waterfall: has(&|a| a == 13),
        river: has(&|a| (14..=21).contains(&a)),
        marine: has(&|a| a == 24),
        offing: false,
        rain: false,
        snow: false,
        field_rank: 6,
    };
    let (m, _, h) = today_mdh();
    let table: Vec<_> = species::insect_seasonal_table(m, h)
        .iter()
        .map(|e| (e.insect, e.area, e.weight as f32))
        .collect();
    let Some(hatch) = acre.decide(&table, &mut |n| rng.range_f32(0.0, n)) else {
        return;
    };
    for &ut in &hatch.units {
        let at = Town::cell_to_world(ax, az, ut % 16, ut / 16);
        spawn_one_bug(
            &mut commands,
            &mut meshes,
            &mut materials,
            &town,
            &mut rng,
            (hatch.insect, hatch.area),
            at,
        );
    }
    println!(
        "{} {:?} ({:?}) in acre ({ax}, {az})",
        hatch.units.len(),
        hatch.insect,
        hatch.area
    );
}

/// Startup: RNG resource + initial critters (runs after town generation).
fn spawn_initial_critters(mut commands: Commands) {
    // Fish and bugs come and go per acre (`place_fish`, `place_bugs`).
    commands.insert_resource(EcoRng(0xC0FFEE1234567890));
}

// ---- UI ----

/// Tool indicator + catch notification UI.
fn setup_ecology_ui(mut commands: Commands) {
    // Tool indicator — top-left, under the inventory HUD.
    commands.spawn((
        ToolText,
        Text::new("Tool: none (1=rod 2=net)"),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));

    // Catch notification — top-center, below the save notification.
    commands.spawn((
        CatchNotifyText,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(26.0),
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(110.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            ..default()
        },
    ));
}

fn show_catch_notify(
    msg: &str,
    timer: &mut ResMut<CatchNotifyTimer>,
    query: &mut Query<&mut Text, With<CatchNotifyText>>,
) {
    if let Ok(mut text) = query.single_mut() {
        text.0 = msg.to_string();
    }
    timer.0 = 2.5;
}

fn tick_catch_notify(
    time: Res<Time>,
    mut timer: ResMut<CatchNotifyTimer>,
    mut query: Query<&mut Text, With<CatchNotifyText>>,
) {
    if timer.0 > 0.0 {
        timer.0 -= time.delta().as_secs_f32();
        if timer.0 <= 0.0 {
            if let Ok(mut text) = query.single_mut() {
                text.0.clear();
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
    if let Ok(mut text) = query.single_mut() {
        text.0 = label.to_string();
    }
}

// ---- behavior ----

/// Fish behave as `ac_gyo_test` does (`rustimal_logic::gyo_ai`): wait,
/// swim, come to the bobber, nibble, bite, flee a running player.
#[allow(clippy::too_many_arguments)]
fn update_fish(
    time: Res<Time>,
    town: Res<Town>,
    actions: Res<Actions>,
    player: Query<&Transform, (With<Player>, Without<Fish>)>,
    mut fish_q: Query<(Entity, &mut Fish, &mut Transform), Without<Bobber>>,
    mut bobbers: Query<(&mut Bobber, &Transform), Without<Fish>>,
    mut rng: ResMut<EcoRng>,
    mut notify: (
        ResMut<CatchNotifyTimer>,
        Query<&mut Text, With<CatchNotifyText>>,
    ),
) {
    let to_game = |p: Vec3| [p.x * GAME_UNITS, p.z * GAME_UNITS];
    let player_pos = player
        .single()
        .map_or([1e6, 1e6], |t| to_game(t.translation));
    let dashing = actions.pressed(Action::Run) && actions.move_dir() != Vec3::ZERO;
    let mut rand = |n: f32| rng.range_f32(0.0, n);
    for (entity, mut fish, mut transform) in &mut fish_q {
        fish.ticks += time.delta_secs() * 60.0;
        let flow = |p: Vec3| {
            town.ground
                .as_ref()
                .and_then(|g| g.attribute_at(p.x, p.z))
                .map_or(0, gyo_ai::flow_angle_rv)
        };
        if fish.ai.is_none() {
            let start = to_game(transform.translation);
            fish.ai = Some(gyo_ai::Gyo::new(
                fish.species as u8,
                start,
                flow(transform.translation),
                &mut rand,
            ));
        }
        while fish.ticks >= 1.0 {
            fish.ticks -= 1.0;
            let bobber = bobbers.iter().next().map(|(b, t)| gyo_ai::Bobber {
                pos: to_game(t.translation),
                splashed: b.timer < 0.1,
                settled: b.timer >= BOBBER_SETTLE_SECS,
                free: b.hooked.is_none_or(|h| h == entity),
            });
            let Some(ai) = fish.ai.as_mut() else { break };
            let here = Vec3::new(ai.pos[0], 0.0, ai.pos[1]) / GAME_UNITS;
            let (s, c) = (ai.angle as f32 * std::f32::consts::PI / 32768.0).sin_cos();
            let ahead = here + Vec3::new(s, 0.0, c) * 0.6;
            let world = gyo_ai::World {
                bobber,
                player: player_pos,
                player_dashing: dashing,
                tool_hit: false,
                flow_rv: flow(here),
                wall: !fish_water(&town, ahead),
                golden_rod: false,
            };
            let event = ai.step(&world, &mut rand);
            if let Some((mut b, _)) = bobbers.iter_mut().next() {
                match event {
                    gyo_ai::Event::Took => {
                        b.hooked = Some(entity);
                        println!("{:?} took the bobber", fish.species);
                    }
                    gyo_ai::Event::Bit => {
                        b.biting = true;
                        println!("{:?} bit", fish.species);
                        show_catch_notify("A bite! Press E!", &mut notify.0, &mut notify.1);
                    }
                    gyo_ai::Event::GotAway if b.hooked == Some(entity) => {
                        b.hooked = None;
                        b.biting = false;
                    }
                    _ => {}
                }
            }
        }
        if let Some(ai) = &fish.ai {
            let p = Vec3::new(ai.pos[0], 0.0, ai.pos[1]) / GAME_UNITS;
            transform.translation = p.with_y(fish_height(&town, p));
            let (s, c) = (ai.angle as f32 * std::f32::consts::PI / 32768.0).sin_cos();
            fish.dir = Vec3::new(s, 0.0, c);
        }
    }
}

/// When the water layout changes (the disc terrain replaced the plan's
/// river), move fish whose home is no longer water into the new water.
fn rehome_fish(
    town: Res<Town>,
    mut fish: Query<(&mut Fish, &mut Transform)>,
    mut rng: ResMut<EcoRng>,
) {
    let tiles: Vec<(usize, usize, usize, usize)> = town.river_tiles.iter().copied().collect();
    if tiles.is_empty() {
        return;
    }
    for (mut f, mut t) in &mut fish {
        if fish_water(&town, f.home) {
            continue;
        }
        let (ax, az, cx, cz) = tiles[rng.range_usize(tiles.len())];
        let pos = Town::cell_to_world(ax, az, cx, cz);
        let y = town
            .ground
            .as_ref()
            .and_then(|g| g.height(pos.x, pos.z))
            .unwrap_or(0.0)
            + FISH_LIFT;
        f.home = Vec3::new(pos.x, y, pos.z);
        f.ai = None;
        t.translation = f.home;
    }
}

/// Butterflies, cicadas and dragonflies behave as their `ac_ins_*` files
/// (`rustimal_logic::ins_ai`). Other bugs still hop toward a target,
/// pause, then pick a new one.
#[allow(clippy::too_many_arguments)]
fn update_bugs(
    mut commands: Commands,
    time: Res<Time>,
    town: Res<Town>,
    items: Option<Res<crate::field_items::FieldItems>>,
    player: Query<(&Transform, &Facing), (With<Player>, Without<Bug>)>,
    villagers: Query<
        (Entity, &Transform),
        (With<crate::npc::Villager>, Without<Bug>, Without<Player>),
    >,
    mut last: Local<std::collections::HashMap<Entity, Vec3>>,
    mut swing: ResMut<NetSwing>,
    mut query: Query<
        (Entity, &mut Bug, &mut Transform, &mut Visibility),
        (Without<Player>, Without<crate::npc::Villager>),
    >,
    mut rng: ResMut<EcoRng>,
) {
    let to_game = |p: Vec3| [p.x * GAME_UNITS, p.y * GAME_UNITS, p.z * GAME_UNITS];
    // Movers and how far they went per 60 Hz tick (stress).
    let ticks_now = (time.delta_secs() * 60.0).max(1e-3);
    let mut movers = Vec::new();
    let mut facing = 0i16;
    let mut player_pos = [1e6; 3];
    let all = player
        .iter()
        .map(|(t, f)| {
            facing = (f.dir.x.atan2(f.dir.z) * 32768.0 / std::f32::consts::PI) as i32 as i16;
            player_pos = to_game(t.translation);
            (Entity::PLACEHOLDER, t.translation)
        })
        .chain(villagers.iter().map(|(e, t)| (e, t.translation)))
        .collect::<Vec<_>>();
    for (e, pos) in all {
        let before = last.insert(e, pos).unwrap_or(pos);
        // A jump of a unit or more in one frame is a warp, not a step.
        let step = (pos - before).xz().length();
        let step = if step >= 1.0 { 0.0 } else { step };
        movers.push(ins_ai::Mover {
            pos: to_game(pos),
            moved: step * GAME_UNITS / ticks_now,
        });
    }
    let net = swing.0.take().map(|p| [p.x * GAME_UNITS, p.z * GAME_UNITS]);
    let field = BugField {
        town: &town,
        items: items.as_deref(),
    };
    for (entity, mut bug, mut transform, mut vis) in &mut query {
        if ins_ai::Insect::supported(bug.insect as u8) {
            let mut rand = |n: f32| rng.range_f32(0.0, n);
            if bug.ai.is_none() {
                let at = to_game(transform.translation);
                match ins_ai::Insect::new(bug.insect as u8, [at[0], at[2]], &field, &mut rand) {
                    Some(ai) => bug.ai = Some(ai),
                    None => {
                        commands.entity(entity).despawn();
                        continue;
                    }
                }
            }
            bug.ticks += time.delta_secs() * 60.0;
            let world = ins_ai::World {
                player: player_pos,
                player_facing: facing,
                movers: &movers,
                net_stop: net,
                scoop: None,
                tree_shaken: None,
                axe_hit: None,
                raining: false,
                busy: false,
                rock_struck: None,
                wisp_night: false,
            };
            let mut gone = false;
            while bug.ticks >= 1.0 {
                bug.ticks -= 1.0;
                let Some(ai) = bug.ai.as_mut() else { break };
                match ai.step(&world, &field, &mut rand) {
                    ins_ai::Event::Gone => gone = true,
                    // No stung reaction for the player yet.
                    ins_ai::Event::Stung => println!("A mosquito stung you"),
                    ins_ai::Event::None => {}
                }
            }
            if gone {
                commands.entity(entity).despawn();
                continue;
            }
            if let Some(ai) = &bug.ai {
                let p = Vec3::from(ai.pos) / GAME_UNITS;
                let (s, c) = (ai.angle as f32 * std::f32::consts::PI / 32768.0).sin_cos();
                // A pill bug under its rock isn't drawn.
                let want = if ai.hidden() {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                };
                vis.set_if_neq(want);
                transform.translation = p;
                bug.target = p + Vec3::new(s, 0.0, c);
            }
            continue;
        }
        if bug.pause > 0.0 {
            bug.pause -= time.delta().as_secs_f32();
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
        transform.translation += to_target.normalize() * bug.speed * time.delta().as_secs_f32();
        transform.translation.y = town.ground_y(transform.translation) + bug.lift;
    }
}

/// 1/2/3 to equip rod / net / nothing.
fn tool_select(
    actions: Res<Actions>,
    focus: Res<InputFocus>,
    shop_loc: Res<State<crate::shop::ShopLocation>>,
    mut tool: ResMut<EquippedTool>,
) {
    // Only while the world has focus (any open UI owns the number keys),
    // and never inside the shop — you enter it empty-handed.
    if !focus.is_world() || *shop_loc != crate::shop::ShopLocation::Town {
        return;
    }
    let before = *tool;
    if actions.just_pressed(Action::Tool1) {
        *tool = EquippedTool::Rod;
        println!("Equipped fishing rod");
    } else if actions.just_pressed(Action::Tool2) {
        *tool = EquippedTool::Net;
        println!("Equipped bug net");
    } else if actions.just_pressed(Action::Tool3) {
        *tool = EquippedTool::None;
        println!("Unequipped tool");
    }
    if *tool != before {
        crate::sound::se(crate::sound::se_id::POCKET_RUSTLE);
    }
}

/// E with a tool equipped: cast the rod near water, or swing the net at bugs.
#[allow(clippy::too_many_arguments)]
fn tool_interact(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<HouseState>>,
    tool: Res<EquippedTool>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<(&Transform, &Facing), With<Player>>,
    town: Res<Town>,
    bug_query: Query<(Entity, &Bug, &Transform)>,
    bobber_query: Query<(Entity, &Bobber, &Transform)>,
    fish_query: Query<&Fish>,
    (mut swing, mut landed): (
        ResMut<NetSwing>,
        bevy::ecs::message::MessageWriter<crate::critters::FishCaught>,
    ),
    mut inventory: ResMut<PlayerInventory>,
    mut notify_timer: ResMut<CatchNotifyTimer>,
    mut notify_query: Query<&mut Text, With<CatchNotifyText>>,
) {
    // Inside the house, E belongs to the house doors. (The shop
    // force-unequips tools on entry, so no shop check is needed here.)
    if *state != HouseState::Town {
        return;
    }
    if *tool == EquippedTool::None || !focus.is_world() {
        return;
    }
    // A held tool always answers E (cast, reel, swing, or a "nothing here"
    // notice), so it takes the press outright. Doors and the mailbox need
    // empty hands, so they never compete for it.
    if !actions.consume(Action::Interact) {
        return;
    }
    let Ok((player_transform, facing)) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    match *tool {
        EquippedTool::Rod => {
            // Reel in if a bobber is already out: a biting fish is caught;
            // otherwise whatever was nibbling lets go.
            if let Ok((bobber, b, bobber_transform)) = bobber_query.single() {
                crate::sound::se(crate::sound::se_id::ROD_BACK);
                crate::sound::se_at(
                    crate::sound::se_id::BOBBER_OUT,
                    bobber_transform.translation,
                );
                commands.entity(bobber).despawn();
                let caught = b.hooked.filter(|_| b.biting);
                match caught.and_then(|f| fish_query.get(f).ok().map(|fish| (f, fish))) {
                    Some((f, fish)) => {
                        let item = items::fish(fish.species);
                        let name = items::name(item);
                        if inventory.add(item) {
                            commands.entity(f).despawn();
                            landed.write(crate::critters::FishCaught {
                                species: fish.species,
                                // Left hand: about hand height, in front.
                                at: player_pos + facing.dir * 0.35 + Vec3::Y * 0.9,
                            });
                            crate::sound::se(crate::sound::se_id::FISH_LIFT);
                            crate::sound::se(crate::sound::se_id::ITEM_GET);
                            show_catch_notify(
                                &format!("Caught a {name}!"),
                                &mut notify_timer,
                                &mut notify_query,
                            );
                            println!("Caught a {name} ({}/15 pockets)", inventory.used());
                        } else {
                            show_catch_notify(
                                "Your pockets are full!",
                                &mut notify_timer,
                                &mut notify_query,
                            );
                        }
                    }
                    None => show_catch_notify("Reeled in.", &mut notify_timer, &mut notify_query),
                }
                return;
            }
            // Find the first river tile along the facing direction.
            let mut cast_point = None;
            for d in CAST_SAMPLES {
                let p = player_pos + facing.dir * d;
                if fish_water(&town, p) {
                    cast_point = Some(p);
                    break;
                }
            }
            let Some(cp) = cast_point else {
                crate::sound::se(crate::sound::se_id::ROD_AIR);
                show_catch_notify(
                    "No water in front of you.",
                    &mut notify_timer,
                    &mut notify_query,
                );
                return;
            };
            crate::sound::se(crate::sound::se_id::ROD_CAST);
            // This simplified bobber spawns directly on the water.
            crate::sound::se_at(crate::sound::se_id::BOBBER_LAND, cp);
            commands.spawn((
                Bobber {
                    timer: 0.0,
                    hooked: None,
                    biting: false,
                },
                Mesh3d(meshes.add(Sphere::new(0.18))),
                MeshMaterial3d(materials.add(Color::srgb(0.9, 0.15, 0.1))),
                Transform::from_xyz(cp.x, town.ground_y(cp) + 0.5, cp.z),
            ));
            println!("Cast!");
        }
        EquippedTool::Net => {
            crate::sound::se_at(
                crate::sound::se_id::TOOL_FURI,
                player_pos + facing.dir * NET_RANGE * 0.7,
            );
            swing.0 = Some(player_pos + facing.dir * NET_RANGE * 0.7);
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
                if best.is_none_or(|(_, d)| dist < d) {
                    best = Some((entity, dist));
                }
            }
            match best {
                Some((entity, _)) => {
                    let Ok((_, bug, bug_transform)) = bug_query.get(entity) else {
                        return;
                    };
                    let item = items::bug(bug.insect);
                    let name = items::name(item);
                    if !inventory.add(item) {
                        // Retail: a full pocket means you can't keep the catch.
                        show_catch_notify(
                            "Your pockets are full!",
                            &mut notify_timer,
                            &mut notify_query,
                        );
                        return;
                    }
                    commands.entity(entity).despawn();
                    // AMI_HIT is an obstacle impact; insect capture is TOOL_GET.
                    crate::sound::se_at(crate::sound::se_id::NET_CATCH, bug_transform.translation);
                    show_catch_notify(
                        &format!("Caught a {name}!"),
                        &mut notify_timer,
                        &mut notify_query,
                    );
                    println!("Caught a {name} ({}/15 pockets)", inventory.used());
                }
                None => {
                    show_catch_notify("No bugs in reach.", &mut notify_timer, &mut notify_query);
                }
            }
        }
        EquippedTool::None => {}
    }
}

/// The bobber's clock (it settles after landing) and its dip while a
/// fish is biting. Catching is the rod's E press (`tool_interact`).
fn update_bobber(
    time: Res<Time>,
    town: Res<Town>,
    mut bobbers: Query<(&mut Bobber, &mut Transform)>,
) {
    for (mut b, mut t) in &mut bobbers {
        b.timer += time.delta_secs();
        let rest = town.ground_y(t.translation) + 0.5;
        t.translation.y = if b.biting { rest - 0.2 } else { rest };
    }
}

pub struct EcologyPlugin;

impl Plugin for EcologyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EquippedTool::default())
            .insert_resource(CatchNotifyTimer::default())
            .init_resource::<NetSwing>()
            .add_systems(Startup, setup_ecology_ui)
            .add_systems(
                Startup,
                spawn_initial_critters.after(crate::town::generate_town),
            )
            .add_systems(
                Update,
                (tool_select, tool_interact.in_set(InteractSet::Tools))
                    .chain()
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                (
                    rehome_fish.run_if(resource_changed::<Town>),
                    place_fish,
                    place_bugs,
                    update_fish,
                    update_bugs,
                    update_bobber,
                    update_tool_hud,
                    tick_catch_notify,
                )
                    .chain()
                    .after(tool_interact)
                    .in_set(GameplaySet),
            );
    }
}
