//! Draws the town ground from the player's disc: each acre becomes one of
//! the game's own acre templates (`grd_s_*`), replacing the flat
//! placeholder planes.
//!
//! The layout comes from the retail generator (`logic::random_field`, a
//! port of `m_random_field_ovl.c`) fed with the disc's own combination
//! table, so cliffs, rivers, slopes, bridges, beach and ocean join up like
//! a real town. Each acre's combination names a `bg_id`, found in
//! `data_bgd` with its opaque and translucent display lists. The town plan
//! (`town_gen`) still drives gameplay until that moves over too.
//!
//! Ground textures live in runtime buffers (.bss) that the game fills with
//! the season's textures, and palettes are picked per month; both are
//! rebuilt here as a [`Segments`] map.

use bevy::prelude::*;
use chrono::Datelike;
use std::collections::HashMap;

use disc::{rel_bss, AssetCatalog, RomImage, RomSource, Target};
use rustimal_logic::random_field::{
    bt, make_random_field, ComboInfo, RandomField, BLOCK_X_NUM, BLOCK_Z_NUM,
};
use rustimal_logic::town_gen::{CellKind, Feature, TownAcre, TownPlan, ACRE_DEPTH, ACRE_WIDTH};

use crate::disc_gfx::{decode_texture, run_display_lists, Segments};
use crate::game_disc::GameDisc;
use crate::ground_map::{GroundMap, Unit};
use crate::n64_material::{part_mesh, texture_image, CombinerUniform, N64Material};
use crate::player::Player;
use crate::states::AppState;
use crate::town::{PlaceholderGround, PlaceholderTree, Town, CELL_SIZE, PLACEHOLDER_TRUNK_Y};

/// Acre models are stored at 16x; one game cell is 40 units.
const GROUND_SCALE: f32 = CELL_SIZE / 16.0 / 40.0;
/// The lowest ground tier sits at y = 640 in model units.
const GROUND_BASE_Y: f32 = 640.0;
const PALETTE_BYTES: u32 = 32;

/// (bss buffer, summer source) for ground textures, in the order of
/// `l_bg_tex_segment_rom_start_s_0`.
const GROUND_TEXTURES: &[(&str, &str)] = &[
    ("earth_tex_dummy", "mFM_grd_s_earth_tex"),
    ("cliff_tex_dummy", "mFM_grd_s_cliff_tex"),
    ("bush_a_tex_dummy", "mFM_grd_s_bushA_tex"),
    ("bush_b_tex_dummy", "mFM_grd_s_bushB_tex"),
    ("grass_tex_dummy", "mFM_grd_s_grass_tex"),
    ("rail_tex_dummy", "mFM_grd_s_rail_tex"),
    ("station_tex_dummy", "mFM_grd_s_station_tex"),
    ("stone_tex_dummy", "mFM_grd_s_stone_tex"),
    ("river_tex_dummy", "mFM_grd_s_river_tex"),
    ("water_1_tex_dummy", "mFM_grd_water1_tex"),
    ("water_2_tex_dummy", "mFM_grd_water2_tex"),
    ("bridge_1_tex_dummy", "mFM_grd_s_bridge1_tex"),
    ("bridge_2_tex_dummy", "mFM_grd_s_bridge2_tex"),
    ("tekkyo_tex_dummy", "mFM_grd_s_tekkyo_tex"),
    ("tunnel_tex_dummy", "mFM_grd_s_tunnel_tex"),
    ("beach_tex_dummy", "mFM_grd_s_beach_tex"),
    ("beach1_tex_dummy2", "mFM_grd_beachA_tex"),
    ("beach2_tex_dummy2", "mFM_grd_beachB_tex"),
    ("sand_tex_dummy", "mFM_grd_s_sand_tex"),
    ("wave1_tex_dummy", "mFM_grd_wave1_tex"),
    ("wave2_tex_dummy", "mFM_grd_wave2_tex"),
    ("wave3_tex_dummy", "mFM_grd_wave3_tex"),
    ("sprashA_tex_dummy", "mFM_grd_sprashA_tex"),
    ("sprashC_tex_dummy", "mFM_grd_sprashC_tex"),
];

/// (bss buffer, palette set); sets with several palettes are per month.
const GROUND_PALETTES: &[(&str, &str)] = &[
    ("bush_pal_dummy", "mFM_bush_pal"),
    ("cliff_pal_dummy", "mFM_cliff_pal"),
    ("earth_pal_dummy", "mFM_earth_pal"),
    ("station_pal_dummy", "mFM_grd_s_station1_pal"),
    ("bridge_1_pal_dummy", "mFM_grd_s_bridge1_pal"),
    ("bridge_2_pal_dummy", "mFM_grd_s_bridge2_pal"),
    ("rail_pal_dummy", "mFM_rail_pal"),
    ("beach_pal_dummy2", "mFM_beach_pal"),
];

/// Ground palette index per month (Jan..Dec). The 12 palettes run green
/// (0-2), through autumn (3-8), to snow (9-11), judged from their colours;
/// the game's exact calendar for them isn't translated yet. December's
/// early weeks stay snow-free here; snow runs January-February.
const MONTH_PALETTE: [u32; 12] = [10, 11, 0, 0, 1, 1, 2, 2, 2, 3, 5, 7];
const FIRST_SNOW_PALETTE: u32 = 9;

/// What the game would have copied into the ground buffers this month.
fn ground_segments(catalog: &AssetCatalog, month0: u32) -> Segments {
    let mut segs = Segments::default();
    let palette = MONTH_PALETTE[month0 as usize % 12];
    let winter = palette >= FIRST_SNOW_PALETTE;
    for (buffer, source) in GROUND_TEXTURES {
        // Winter has its own texture set (`mFM_grd_w_*`) where one exists.
        let winter_source = source.replace("_grd_s_", "_grd_w_");
        let src = winter
            .then(|| catalog.get(&winter_source))
            .flatten()
            .or_else(|| catalog.get(source));
        if let (Some((bss, len)), Some(src)) = (rel_bss(buffer), src) {
            segs.set_bss(bss, len.min(src.size), src.offset);
        }
    }
    for (buffer, source) in GROUND_PALETTES {
        if let (Some((bss, len)), Some(src)) = (rel_bss(buffer), catalog.get(source)) {
            let count = (src.size / PALETTE_BYTES).max(1);
            let index = if count >= 12 { palette } else { 0 };
            segs.set_bss(bss, len, src.offset + index * PALETTE_BYTES);
        }
    }
    segs
}

/// `data_combi_table` entry size: bg_id u16, fg_id u16, block type u8, pad.
const COMBI_SIZE: usize = 6;
/// `data_bgd` entry (`mFM_bg_data_c`): bg_id at +0, opaque and translucent
/// display lists at +4 / +8.
const BGD_SIZE: u32 = 0x434;

/// The retail combination table (template choices per block type).
fn read_combos(rom: &RomImage, catalog: &AssetCatalog) -> Vec<ComboInfo> {
    let Some(table) = catalog.get("data_combi_table") else {
        return Vec::new();
    };
    let Some(raw) = rom.slice(RomSource::Rel, table.offset, table.size) else {
        return Vec::new();
    };
    raw.as_chunks::<COMBI_SIZE>()
        .0
        .iter()
        .map(|e| ComboInfo {
            bg_id: u16::from_be_bytes([e[0], e[1]]),
            fg_id: u16::from_be_bytes([e[2], e[3]]),
            block_type: e[4],
        })
        .collect()
}

/// A `data_bgd` entry: display lists and where its collision grid starts.
#[derive(Clone, Copy)]
struct BgEntry {
    opaque: Option<u32>,
    translucent: Option<u32>,
    /// REL offset of `collision[16][16]` (u32 words).
    collision: u32,
}

/// `mFM_bg_data_c::collision` offset: after bg_id, 3 pointers, the
/// animation count and the ROM start/end words.
const BGD_COLLISION: u32 = 0x1C;

fn collision_units(rom: &RomImage, entry: &BgEntry) -> [Unit; 256] {
    let mut units = [Unit::default(); 256];
    if let Some(raw) = rom.slice(RomSource::Rel, entry.collision, 256 * 4) {
        for (u, w) in units.iter_mut().zip(raw.as_chunks::<4>().0) {
            *u = Unit::from_raw(u32::from_be_bytes(*w));
        }
    }
    units
}

/// `bg_id` → its `data_bgd` entry.
fn read_bg_lists(rom: &RomImage, catalog: &AssetCatalog) -> HashMap<u16, BgEntry> {
    let mut map = HashMap::new();
    let Some(bgd) = catalog.get("data_bgd") else {
        return map;
    };
    let list = |at: u32| match rom.pointer_at(RomSource::Rel, at) {
        Some(Target::File(RomSource::Rel, t)) => Some(t),
        _ => None,
    };
    for i in 0..bgd.size / BGD_SIZE {
        let at = bgd.offset + i * BGD_SIZE;
        let Some(id) = rom.slice(RomSource::Rel, at, 2) else {
            break;
        };
        map.insert(
            u16::from_be_bytes([id[0], id[1]]),
            BgEntry {
                opaque: list(at + 4),
                translucent: list(at + 8),
                collision: at + BGD_COLLISION,
            },
        );
    }
    map
}

/// Deterministic `RANDOM(max)` for a town seed.
fn seeded_rng(seed: u32) -> impl FnMut(i32) -> i32 {
    let mut s = seed ^ 0x5EED_ACAC;
    move |max| {
        s = s.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        if max <= 0 {
            0
        } else {
            ((s >> 16) % max as u32) as i32
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_acre(
    commands: &mut Commands,
    rom: &RomImage,
    lists: &[(u32, bool)],
    segs: &Segments,
    at: Vec3,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) {
    let root = commands
        .spawn((
            DiscGround,
            Transform::from_translation(at),
            Visibility::default(),
        ))
        .id();
    for &(list, blend) in lists {
        for part in &run_display_lists(rom, &[list]).parts {
            let texture = part.key.tex.as_ref().and_then(|k| {
                decode_texture(rom, k, None, segs).map(|rgba| images.add(texture_image(rgba, k)))
            });
            let material = materials.add(N64Material {
                combiner: CombinerUniform::from_key(&part.key, false, blend),
                texture,
                blend,
            });
            commands.spawn((
                Mesh3d(meshes.add(part_mesh(part, GROUND_SCALE))),
                MeshMaterial3d(material),
                ChildOf(root),
            ));
        }
    }
}

/// Marker for disc-drawn ground.
#[derive(Component)]
pub struct DiscGround;

/// The retail town built from the disc: layout and combination table.
#[derive(Clone)]
pub struct RetailLayout {
    pub field: RandomField,
    pub combos: Vec<ComboInfo>,
    /// Where the retail layout puts the shop (`SHOP0..3` item), if any.
    pub shop: Option<Vec3>,
}

/// `fgdata.bin` entry: `fg_id` u16, `items[16][16]` u16, 4 bytes of
/// `haniwa_step`.
const FG_ENTRY_SIZE: usize = 518;

/// `fg_id` to the acre's 256 item ids (`mActor_name_t`).
fn read_fg_layouts(fgdata: &[u8]) -> HashMap<u16, [u16; 256]> {
    fgdata
        .as_chunks::<FG_ENTRY_SIZE>()
        .0
        .iter()
        .map(|e| {
            let mut items = [0u16; 256];
            for (i, item) in items.iter_mut().enumerate() {
                *item = u16::from_be_bytes([e[2 + i * 2], e[3 + i * 2]]);
            }
            (u16::from_be_bytes([e[0], e[1]]), items)
        })
        .collect()
}

/// What a retail object id becomes in the town plan (`m_name_table.h`).
fn cell_for_item(item: u16) -> Option<CellKind> {
    const ENV_START: u16 = 0x0800;
    match item {
        // Pansy/cosmos/tulip leaves and flowers (ENV_START + 60..=77).
        i if (ENV_START + 60..=ENV_START + 77).contains(&i) => Some(CellKind::Flower),
        // Saplings and grown trees: fruit, cedar, palm, gold, bells.
        i if (ENV_START..ENV_START + 0x80).contains(&i) => Some(CellKind::Tree),
        0x0063..=0x0067 => Some(CellKind::Rock), // ROCK_A..
        _ => None,
    }
}

/// Builds the retail town from the disc for the plan seed: the layout,
/// the walkable ground, and the plan trees/rocks/flowers from each acre
/// object layout (resident homes from the plan are kept).
pub fn build_retail_town(disc: &GameDisc, plan: &mut TownPlan) -> (RetailLayout, GroundMap) {
    let combos = read_combos(&disc.rom, &disc.catalog);
    let bg_lists = read_bg_lists(&disc.rom, &disc.catalog);
    let fg = read_fg_layouts(&disc.fgdata);
    let field = make_random_field(&combos, &mut seeded_rng(plan.seed));
    let mut units = Vec::with_capacity(BLOCK_X_NUM * BLOCK_Z_NUM);
    let mut tiers = [0u8; BLOCK_X_NUM * BLOCK_Z_NUM];
    for (i, combi) in field.combi.iter().enumerate() {
        tiers[i] = combi.height;
        let entry = combos
            .get(combi.combi as usize)
            .and_then(|c| bg_lists.get(&c.bg_id));
        units.push(entry.map_or([Unit::default(); 256], |e| collision_units(&disc.rom, e)));
    }
    let ground = GroundMap::new(units, tiers);

    // Objects for the playable acres (blocks 1..=5, 1..=6).
    let homes = [
        CellKind::House,
        CellKind::HouseSign,
        CellKind::HouseReserved,
    ]
    .map(|k| k as u8);
    for az in 0..ACRE_DEPTH {
        for ax in 0..ACRE_WIDTH {
            let block = (az + 1) * BLOCK_X_NUM + ax + 1;
            let items = combos
                .get(field.combi[block].combi as usize)
                .and_then(|c| fg.get(&c.fg_id));
            let acre = &mut plan.acres[az * ACRE_WIDTH + ax];
            for (cell, slot) in acre.cells.iter_mut().enumerate() {
                if homes.contains(slot) {
                    continue;
                }
                *slot = items
                    .and_then(|it| cell_for_item(it[cell]))
                    .unwrap_or(CellKind::Grass) as u8;
            }
        }
    }
    let structures = find_structures(&field, &combos, &fg);
    let shop = structures
        .iter()
        .find(|s| (SHOP0..=SHOP0 + 3).contains(&s.0))
        .map(|&(_, ax, az, cx, cz)| Town::cell_to_world(ax, az, cx, cz));
    resite_homes(plan, &structures, &ground);
    (
        RetailLayout {
            field,
            combos,
            shop,
        },
        ground,
    )
}

/// `STRUCTURE_START` items (`m_name_table.h`).
const HOUSE0: u16 = 0x5800;
const SHOP0: u16 = 0x5804;

/// Structure items in the playable acres: (item, ax, az, cx, cz).
fn find_structures(
    field: &RandomField,
    combos: &[ComboInfo],
    fg: &HashMap<u16, [u16; 256]>,
) -> Vec<(u16, usize, usize, usize, usize)> {
    let mut out = Vec::new();
    for az in 0..ACRE_DEPTH {
        for ax in 0..ACRE_WIDTH {
            let block = (az + 1) * BLOCK_X_NUM + ax + 1;
            let Some(items) = combos
                .get(field.combi[block].combi as usize)
                .and_then(|c| fg.get(&c.fg_id))
            else {
                continue;
            };
            for (cell, &item) in items.iter().enumerate() {
                if (0x5800..0x5900).contains(&item) {
                    out.push((item, ax, az, cell % 16, cell / 16));
                }
            }
        }
    }
    out
}

const HOME_KINDS: [CellKind; 3] = [
    CellKind::House,
    CellKind::HouseSign,
    CellKind::HouseReserved,
];

/// Clears a home footprint centred on (cx, cz).
fn clear_home(acre: &mut TownAcre, cx: usize, cz: usize) {
    for z in cz.saturating_sub(1)..=(cz + 1).min(15) {
        for x in cx.saturating_sub(1)..=(cx + 1).min(15) {
            let c = &mut acre.cells[z * 16 + x];
            if HOME_KINDS.iter().any(|k| *c == *k as u8) {
                *c = CellKind::Grass as u8;
            }
        }
    }
}

/// Stamps a home footprint as `town_gen` does: centre, sign at the
/// south-west, the rest reserved.
fn stamp_home(acre: &mut TownAcre, cx: usize, cz: usize) {
    for z in cz - 1..=cz + 1 {
        for x in cx - 1..=cx + 1 {
            let kind = if x == cx && z == cz {
                CellKind::House
            } else if x + 1 == cx && z == cz + 1 {
                CellKind::HouseSign
            } else {
                CellKind::HouseReserved
            };
            acre.cells[z * 16 + x] = kind as u8;
        }
    }
}

/// A 3x3 home fits at (cx, cz): free grass, walkable, level ground, and a
/// walkable doorstep to the south.
fn home_fits(
    acre: &TownAcre,
    ground: &GroundMap,
    ax: usize,
    az: usize,
    cx: usize,
    cz: usize,
) -> bool {
    if !(1..=14).contains(&cx) || !(1..=13).contains(&cz) {
        return false;
    }
    let level = ground.height(
        Town::cell_to_world(ax, az, cx, cz).x,
        Town::cell_to_world(ax, az, cx, cz).z,
    );
    let ok = |x: usize, z: usize| {
        let p = Town::cell_to_world(ax, az, x, z);
        acre.cells[z * 16 + x] == CellKind::Grass as u8
            && ground.walkable_at(p.x, p.z)
            && ground
                .height(p.x, p.z)
                .zip(level)
                .is_some_and(|(h, l)| (h - l).abs() < 0.05)
    };
    (cz - 1..=cz + 1).all(|z| (cx - 1..=cx + 1).all(|x| ok(x, z))) && ok(cx, cz + 2)
}

/// Nearest fitting home spot in an acre to (cx, cz).
fn nearest_home_spot(
    acre: &TownAcre,
    ground: &GroundMap,
    ax: usize,
    az: usize,
    cx: usize,
    cz: usize,
) -> Option<(usize, usize)> {
    let mut spots: Vec<(usize, usize)> =
        (0..16).flat_map(|z| (0..16).map(move |x| (x, z))).collect();
    spots.sort_by_key(|&(x, z)| (x as i32 - cx as i32).pow(2) + (z as i32 - cz as i32).pow(2));
    spots
        .into_iter()
        .find(|&(x, z)| home_fits(acre, ground, ax, az, x, z))
}

/// Moves the plan homes onto the retail town: the player house to the
/// layout HOUSE0 spot, each resident home to the nearest spot in its acre
/// where it fits on the terrain.
fn resite_homes(
    plan: &mut TownPlan,
    structures: &[(u16, usize, usize, usize, usize)],
    ground: &GroundMap,
) {
    // Lift every existing footprint off the map first.
    let mut homes = Vec::new();
    for (i, acre) in plan.acres.iter_mut().enumerate() {
        for cell in 0..256 {
            if acre.cells[cell] == CellKind::House as u8 {
                homes.push((i, cell % 16, cell / 16));
            }
        }
    }
    for &(i, cx, cz) in &homes {
        clear_home(&mut plan.acres[i], cx, cz);
    }

    // Player house on the retail spot.
    if let Some(&(_, ax, az, cx, cz)) = structures
        .iter()
        .find(|s| (HOUSE0..HOUSE0 + 4).contains(&s.0))
    {
        for acre in plan.acres.iter_mut() {
            if acre.feature == Feature::PlayerHouse as u8 {
                acre.feature = Feature::None as u8;
            }
        }
        let acre = &mut plan.acres[az * ACRE_WIDTH + ax];
        acre.feature = Feature::PlayerHouse as u8;
        stamp_home(acre, cx.clamp(1, 14), cz.clamp(1, 14));
    }

    // Resident homes: same acre, nearest spot that fits.
    for i in 0..plan.villager_count as usize {
        let a = plan.house_acres[i] as usize;
        let unit = plan.house_units[i] as usize;
        let (ax, az) = (a % ACRE_WIDTH, a / ACRE_WIDTH);
        let spot = nearest_home_spot(&plan.acres[a], ground, ax, az, unit % 16, unit / 16);
        match spot {
            Some((x, z)) => {
                stamp_home(&mut plan.acres[a], x, z);
                plan.house_units[i] = (z * 16 + x) as u16;
            }
            None => warn!("no room for resident home {i} in acre ({ax},{az})"),
        }
    }
}

fn build_disc_town(
    mut commands: Commands,
    disc: Res<GameDisc>,
    town: Res<Town>,
    placeholders: Query<Entity, With<PlaceholderGround>>,
    trees: Query<(Entity, &Transform, &PlaceholderTree)>,
    mut player: Query<&mut Transform, (With<Player>, Without<PlaceholderTree>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (Some(retail), Some(ground)) = (&town.retail, &town.ground) else {
        return;
    };
    let segs = ground_segments(&disc.catalog, chrono::Local::now().month0());
    let bg_lists = read_bg_lists(&disc.rom, &disc.catalog);
    let mut missing = 0;
    for bz in 0..BLOCK_Z_NUM {
        for bx in 0..BLOCK_X_NUM {
            let combi = retail.field.combi[bz * BLOCK_X_NUM + bx];
            let Some(entry) = retail
                .combos
                .get(combi.combi as usize)
                .and_then(|c| bg_lists.get(&c.bg_id))
            else {
                missing += 1;
                continue;
            };
            let lists: Vec<(u32, bool)> = [(entry.opaque, false), (entry.translucent, true)]
                .into_iter()
                .filter_map(|(l, blend)| l.map(|l| (l, blend)))
                .collect();
            // The playable 5x6 acres are blocks (1..=5, 1..=6); the rest is
            // the border ring, rail line, beach and ocean.
            let c = GroundMap::block_corner(bx, bz);
            let corner = Vec3::new(
                c.x,
                GroundMap::tier_offset(combi.height) - GROUND_BASE_Y * GROUND_SCALE,
                c.y,
            );
            spawn_acre(
                &mut commands,
                &disc.rom,
                &lists,
                &segs,
                corner,
                &mut meshes,
                &mut materials,
                &mut images,
            );
        }
    }
    if missing > 0 {
        warn!("{missing} acres have no ground template on the disc");
    }
    // A saved position from the old layout may be in water now.
    if let Ok(mut t) = player.single_mut() {
        if !ground.walkable_at(t.translation.x, t.translation.z) {
            let house = retail
                .field
                .blocks
                .iter()
                .position(|&b| b == bt::PLAYER_HOUSE);
            if let Some(spot) =
                house.and_then(|i| ground.walkable_spot_in(i % BLOCK_X_NUM, i / BLOCK_X_NUM))
            {
                t.translation.x = spot.x;
                t.translation.z = spot.y;
                info!("player moved onto walkable ground by the house");
            }
        }
    }
    for e in &placeholders {
        commands.entity(e).despawn();
    }
    spawn_disc_trees(
        &mut commands,
        &disc,
        &trees,
        chrono::Local::now().month0(),
        &mut meshes,
        &mut materials,
        &mut images,
    );
    info!(
        "retail town from disc: {} step, {} combinations",
        if retail.field.three_step { "3" } else { "2" },
        retail.combos.len()
    );
}

/// Tree palette per month (Jan..Dec) among the 14 in
/// `mFM_obj_tree_01_pal_dol`, judged from their colours: 0 spring green,
/// 1 cherry blossom, 2-3 summer, 4-9 autumn green to brown, 10-12 snow,
/// 13 thaw. The game picks by date; that calendar isn't translated yet.
const TREE_MONTH_PALETTE: [u32; 12] = [11, 12, 13, 0, 0, 2, 3, 3, 4, 6, 8, 10];
const TREE_FIRST_SNOW_PALETTE: u32 = 10;

/// Object models are drawn at 1/100 scale; one game cell is 40 units.
const OBJECT_SCALE: f32 = 0.01 / 40.0 * CELL_SIZE;

/// The grown tree (summer or winter set): trunk then leaves, each a
/// material list followed by its geometry.
fn tree_lists(catalog: &AssetCatalog, winter: bool) -> Vec<u32> {
    let set = if winter { "obj_w_tree5" } else { "obj_s_tree5" };
    ["trunkT_mat", "trunkT_gfx", "leafT_mat", "leafT_gfx"]
        .iter()
        .filter_map(|part| catalog.get(&format!("{set}_{part}_model")))
        .map(|a| a.offset)
        .collect()
}

/// Replaces the placeholder trees with the disc's grown-tree model, built
/// once and shared by every tree.
#[allow(clippy::too_many_arguments)]
fn spawn_disc_trees(
    commands: &mut Commands,
    disc: &GameDisc,
    placeholders: &Query<(Entity, &Transform, &PlaceholderTree)>,
    month0: u32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) {
    let palette_index = TREE_MONTH_PALETTE[month0 as usize % 12];
    let lists = tree_lists(&disc.catalog, palette_index >= TREE_FIRST_SNOW_PALETTE);
    if lists.len() != 4 {
        warn!("tree model not found on the disc; keeping placeholder trees");
        return;
    }
    // Tree colours come from the 14 `mFM_obj_tree_01_pal_dol` palettes.
    let palette = disc
        .catalog
        .get("mFM_obj_tree_01_pal_dol")
        .map(|p| p.offset + (palette_index % (p.size / PALETTE_BYTES).max(1)) * PALETTE_BYTES);
    let sink = run_display_lists(&disc.rom, &lists);
    let segs = Segments::default();
    let parts: Vec<(Handle<Mesh>, Handle<N64Material>)> = sink
        .parts
        .iter()
        .map(|part| {
            let texture = part.key.tex.as_ref().and_then(|k| {
                decode_texture(&disc.rom, k, palette, &segs)
                    .map(|rgba| images.add(texture_image(rgba, k)))
            });
            let material = N64Material {
                combiner: CombinerUniform::from_key(&part.key, false, false),
                texture,
                blend: false,
            };
            (
                meshes.add(part_mesh(part, OBJECT_SCALE)),
                materials.add(material),
            )
        })
        .collect();
    let mut trees = 0;
    for (entity, t, tree) in placeholders {
        commands.entity(entity).despawn();
        if !tree.trunk {
            continue;
        }
        let at = t.translation - Vec3::Y * PLACEHOLDER_TRUNK_Y;
        let root = commands
            .spawn((
                DiscTree,
                Transform::from_translation(at),
                Visibility::default(),
            ))
            .id();
        for (mesh, material) in &parts {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                ChildOf(root),
            ));
        }
        trees += 1;
    }
    info!("{trees} trees drawn from the disc");
}

/// Marker for a disc-drawn tree.
#[derive(Component)]
pub struct DiscTree;

/// Player capsule centre above the ground.
const PLAYER_HALF_HEIGHT: f32 = 1.0;

/// Keeps the player on the disc terrain's surface.
fn follow_ground(town: Res<Town>, mut player: Query<&mut Transform, With<Player>>) {
    let Some(ground) = &town.ground else { return };
    for mut t in &mut player {
        if let Some(h) = ground.height(t.translation.x, t.translation.z) {
            t.translation.y = h + PLAYER_HALF_HEIGHT;
        }
    }
}

pub struct DiscTownPlugin;

impl Plugin for DiscTownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            build_disc_town.run_if(resource_exists::<GameDisc>.and_then(resource_exists::<Town>)),
        )
        .add_systems(Update, follow_ground.run_if(in_state(AppState::InGame)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Probe (needs the disc): attribute / corner samples of a river acre.
    #[test]
    #[ignore]
    fn probe_river_collision() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut disc = disc::Disc::open_path(std::path::Path::new(&path)).unwrap();
        let rom = RomImage::from_disc(&mut disc).unwrap();
        let cat = AssetCatalog::builtin(&rom);
        let combos = read_combos(&rom, &cat);
        let bgs = read_bg_lists(&rom, &cat);
        for t in [bt::RIVER_SOUTH, bt::CLIFF_HORIZONTAL] {
            let c = combos.iter().find(|c| c.block_type == t).unwrap();
            let units = collision_units(&rom, &bgs[&c.bg_id]);
            for z in 0..16 {
                let row: Vec<String> = (0..16)
                    .map(|x| {
                        let u = units[z * 16 + x];
                        format!(
                            "{:2}:{:2}{:2}{:2}{:2}",
                            u.attribute, u.top_left, u.top_right, u.bot_left, u.bot_right
                        )
                    })
                    .collect();
                eprintln!("t{t} z{z:2} {}", row.join(" "));
            }
        }
    }

    /// Probe (needs the disc): average colour of each tree palette.
    #[test]
    #[ignore]
    fn probe_tree_palettes() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut disc = disc::Disc::open_path(std::path::Path::new(&path)).unwrap();
        let rom = RomImage::from_disc(&mut disc).unwrap();
        let cat = AssetCatalog::builtin(&rom);
        let a = cat.get("mFM_obj_tree_01_pal_dol").unwrap();
        let pal = rom.slice(disc::RomSource::Rel, a.offset, a.size).unwrap();
        for (i, p) in pal.chunks(32).enumerate() {
            let cols: Vec<String> = p
                .chunks(2)
                .map(|c| {
                    let v = u16::from_be_bytes([c[0], c[1]]);
                    if v & 0x8000 != 0 {
                        format!(
                            "{:02x}{:02x}{:02x}",
                            (v >> 10 & 31) * 8,
                            (v >> 5 & 31) * 8,
                            (v & 31) * 8
                        )
                    } else {
                        format!(
                            "{:02x}{:02x}{:02x}",
                            (v >> 8 & 15) * 17,
                            (v >> 4 & 15) * 17,
                            (v & 15) * 17
                        )
                    }
                })
                .collect();
            eprintln!("treepal {i:2}: {}", cols.join(" "));
        }
    }

    #[test]
    fn seeded_rng_is_in_range_and_repeatable() {
        let (mut a, mut b) = (seeded_rng(42), seeded_rng(42));
        for max in [1, 2, 7, 100] {
            let v = a(max);
            assert!((0..max).contains(&v));
            assert_eq!(v, b(max));
        }
        assert_eq!(a(0), 0);
    }
}
