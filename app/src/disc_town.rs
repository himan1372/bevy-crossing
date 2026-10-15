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
use std::collections::{HashMap, HashSet};

use disc::{rel_bss, AssetCatalog, RomImage, RomSource, Target};
use rustimal_logic::random_field::{
    bt, make_random_field, ComboInfo, RandomField, BLOCK_X_NUM, BLOCK_Z_NUM,
};
use rustimal_logic::town_gen::{CellKind, Feature, TownAcre, TownPlan, ACRE_DEPTH, ACRE_WIDTH};

use crate::disc_gfx::{decode_texture, run_display_lists, run_ground_lists, Segments, LIST_TAG};
use crate::environment::BuildingLights;
use crate::game_disc::GameDisc;
use crate::ground_map::{GroundMap, Unit};
use crate::n64_material::{part_mesh, texture_image, CombinerUniform, N64Material};
use crate::player::Player;
use crate::states::AppState;
use crate::town::{
    PlaceholderGround, PlaceholderObject, PlaceholderTree, Town, CELL_SIZE, PLACEHOLDER_TRUNK_Y,
};

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

/// Ground palette per calendar term (`mFM_SetFieldPal`
/// `field_pal_idx_table`).
const FIELD_PAL_IDX: [u32; 18] = [9, 10, 11, 0, 1, 1, 1, 2, 2, 3, 4, 5, 6, 7, 8, 9, 9, 9];
/// Tree palette per calendar term (`mFM_SetFGPal` `tree_pal_idx_table`).
const TREE_PAL_IDX: [u32; 18] = [10, 12, 13, 0, 1, 2, 2, 3, 3, 4, 5, 6, 7, 8, 9, 10, 11, 10];

/// Today's calendar term on the game clock (`mTM_get_termIdx`).
pub(crate) fn current_term() -> usize {
    let now = crate::clock::now();
    rustimal_logic::game_time::term_index(now.month, now.day, false).unwrap_or(0)
}

/// Whether a term is winter: snow textures and models.
fn is_winter(term: usize) -> bool {
    use rustimal_logic::game_time::{Season, CALENDAR_TERMS};
    CALENDAR_TERMS[term % 18].season == Season::Winter
}

/// What the game would have copied into the ground buffers this term.
fn ground_segments(catalog: &AssetCatalog, term: usize) -> Segments {
    let mut segs = Segments::default();
    let palette = FIELD_PAL_IDX[term % 18];
    let winter = is_winter(term);
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
pub(crate) fn seeded_rng(seed: u32) -> impl FnMut(i32) -> i32 {
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
        for part in &run_ground_lists(rom, &[list]).parts {
            let texture = part.key.tex.as_ref().and_then(|k| {
                decode_texture(rom, k, None, segs).map(|rgba| images.add(texture_image(rgba, k)))
            });
            let material = materials.add(N64Material {
                combiner: CombinerUniform::from_key(&part.key, false, blend),
                texture,
                env: default(),
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
    /// Rock variant (0..5 = A..E) per plan cell (ax, az, cx, cz).
    pub rocks: HashMap<(usize, usize, usize, usize), u8>,
    /// The four player houses (`HOUSE0..3` items) by house number.
    pub player_homes: [Option<(usize, usize, usize, usize)>; 4],
    /// Shop, post office, police box, museum and tailor, as placed.
    pub buildings: Vec<RetailBuilding>,
    /// Each playable acre's field items (`mActor_name_t`, row by row),
    /// from its layout; objects the plan moved aside are emptied.
    pub items: Vec<[u16; 256]>,
    /// The four mailboxes (`ACTOR_PROP_MAILBOX0..3`) by house number.
    pub mailboxes: [Option<(usize, usize, usize, usize)>; 4],
    /// The four houses' gyroids (`ACTOR_PROP_HANIWA0..3`).
    pub gyroids: [Option<(usize, usize, usize, usize)>; 4],
}

/// A public building from the layout, placed like its actor does.
#[derive(Clone, Debug)]
pub struct RetailBuilding {
    /// Layout item (`SHOP0`, `POST_OFFICE`, ...).
    pub item: u16,
    pub spec: usize,
    pub centre: Vec3,
    pub yaw: f32,
    /// Half extents (x, z) of the footprint, in a frame turned by
    /// `box_yaw` (0 or 45 deg: whichever fits the model tighter, since
    /// several buildings stand at 45 deg).
    pub half: Vec2,
    pub box_yaw: f32,
    /// Where the model's origin goes (the actor's position).
    pub origin: Vec3,
    /// The door's world position, when the model has one.
    pub door: Option<Vec3>,
}

/// One kind of public building: layout item, model prefix, body parts,
/// light part, `aSTR_PAL_*` palette index, and the actor's position shift
/// in game units (from each actor's `_actor_ct`).
struct BuildingSpec {
    item: u16,
    prefix: &'static str,
    parts: &'static [&'static str],
    light: &'static str,
    palette: u32,
    shift: (f32, f32),
}

const BUILDINGS: [BuildingSpec; 5] = [
    // ac_shop.c: x -20, z +20.
    BuildingSpec {
        item: 0x5804,
        prefix: "shop1",
        parts: &["front", "side", "roof", "door"],
        light: "light",
        palette: 12,
        shift: (-20.0, 20.0),
    },
    // ac_post_office.c: x -20, z +20.
    BuildingSpec {
        item: 0x5808,
        prefix: "yubinkyoku",
        parts: &["t1", "t2", "t3_1", "door", "neon"],
        light: "light",
        palette: 16,
        shift: (-20.0, 20.0),
    },
    // ac_police_box.c: no shift.
    BuildingSpec {
        item: 0x580C,
        prefix: "kouban",
        parts: &["t1", "t2", "t3", "neon"],
        light: "light",
        palette: 34,
        shift: (0.0, 0.0),
    },
    // ac_museum.c: no shift.
    BuildingSpec {
        item: 0x584A,
        prefix: "museum",
        parts: &["t1", "t2", "t3T"],
        light: "lightT",
        palette: 66,
        shift: (0.0, 0.0),
    },
    // ac_needlework_shop.c: x -20, z +20.
    BuildingSpec {
        item: 0x584D,
        prefix: "tailor",
        parts: &["1", "2", "3", "door"],
        light: "light",
        palette: 68,
        shift: (-20.0, 20.0),
    },
];

/// Game units per cell.
const GAME_UNITS_PER_CELL: f32 = 40.0;

/// `structure_pal_adrs_(no)winter[index]`: the palette the game puts in
/// segment 8 for a structure (`aSTR_get_pal_segment`).
fn structure_palette(disc: &GameDisc, index: u32, winter: bool) -> Option<u32> {
    let table = if winter {
        "structure_pal_adrs_winter"
    } else {
        "structure_pal_adrs_nowinter"
    };
    let t = disc.catalog.get(table)?;
    match disc.rom.pointer_at(RomSource::Rel, t.offset + index * 4) {
        Some(Target::File(RomSource::Rel, p)) => Some(p),
        _ => None,
    }
}

/// Places the layout's public buildings and measures their footprints.
fn place_buildings(
    disc: &GameDisc,
    structures: &[(u16, usize, usize, usize, usize)],
) -> Vec<RetailBuilding> {
    let mut out = Vec::new();
    for &(item, ax, az, cx, cz) in structures {
        let Some(spec_index) = BUILDINGS.iter().position(|b| b.item == item) else {
            continue;
        };
        let spec = &BUILDINGS[spec_index];
        let prefix = format!("obj_s_{}", spec.prefix);
        let lists = part_lists(&disc.catalog, &prefix, spec.parts);
        if lists.is_empty() {
            continue;
        }
        let mut sink = run_display_lists(&disc.rom, &lists);
        pose_parts(disc, &lists, &mut sink);
        let points: Vec<Vec2> = sink
            .parts
            .iter()
            .flat_map(|p| &p.vertices)
            .map(|v| Vec2::new(v.pos[0], v.pos[2]) * OBJECT_SCALE)
            .collect();
        let (box_yaw, lo, hi) = tightest_box(&points);
        let shift = Vec3::new(spec.shift.0, 0.0, spec.shift.1) / GAME_UNITS_PER_CELL * CELL_SIZE;
        let origin = Town::cell_to_world(ax, az, cx, cz) + shift;
        let mid = (lo + hi) / 2.0;
        // The actors don't turn these models (rotation 0); the posed model
        // already faces the way the game shows it.
        let door = posed_centroid(disc, &prefix, "door").map(|d| origin + d * OBJECT_SCALE);
        out.push(RetailBuilding {
            item,
            spec: spec_index,
            centre: origin + Quat::from_rotation_y(box_yaw) * Vec3::new(mid.x, 0.0, mid.y),
            yaw: 0.0,
            half: (hi - lo) / 2.0,
            box_yaw,
            origin,
            door,
        });
    }
    out
}

/// The axis box (0 deg) or 45-deg box around `points`, whichever is
/// smaller: (yaw, min, max) with min/max in the box's frame.
fn tightest_box(points: &[Vec2]) -> (f32, Vec2, Vec2) {
    let fit = |yaw: f32| {
        let back = Vec2::from_angle(yaw);
        let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
        for &p in points {
            // Bevy's rotation_y(yaw) turns (x, z) by -yaw in the xz plane.
            let q = back.rotate(p);
            (lo, hi) = (lo.min(q), hi.max(q));
        }
        (yaw, lo, hi)
    };
    let area = |b: &(f32, Vec2, Vec2)| (b.2 - b.1).x * (b.2 - b.1).y;
    let straight = fit(0.0);
    let diagonal = fit(std::f32::consts::FRAC_PI_4);
    if area(&diagonal) < area(&straight) {
        diagonal
    } else {
        straight
    }
}

/// Model-space centroid of a structure part in its idle pose.
fn posed_centroid(disc: &GameDisc, prefix: &str, part: &str) -> Option<Vec3> {
    let lists = part_lists(&disc.catalog, prefix, &[part]);
    let mut sink = run_display_lists(&disc.rom, &lists);
    pose_parts(disc, &lists, &mut sink);
    let verts: Vec<Vec3> = sink
        .parts
        .iter()
        .flat_map(|p| &p.vertices)
        .map(|v| Vec3::from(v.pos))
        .collect();
    (!verts.is_empty()).then(|| verts.iter().sum::<Vec3>() / verts.len() as f32)
}

/// Draws the public buildings from the layout.
#[allow(clippy::too_many_arguments)]
fn spawn_buildings(
    commands: &mut Commands,
    disc: &GameDisc,
    town: &Town,
    winter: bool,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
    lights: &mut BuildingLights,
) {
    let Some(retail) = &town.retail else { return };
    let season = if winter { "w" } else { "s" };
    for b in &retail.buildings {
        let spec = &BUILDINGS[b.spec];
        let prefix = format!("obj_{season}_{}", spec.prefix);
        let palette = structure_palette(disc, spec.palette, winter);
        let lists = part_lists(&disc.catalog, &prefix, spec.parts);
        let mut model = build_model(disc, &lists, palette, meshes, materials, images);
        model.extend(build_windows(
            disc, &prefix, spec.light, palette, meshes, materials, images, lights,
        ));
        let ground_y = town.ground_y(b.origin);
        let origin = b.origin;
        let root = commands
            .spawn((
                DiscObject,
                Name::new(prefix.clone()),
                Transform::from_xyz(origin.x, ground_y, origin.z)
                    .with_rotation(Quat::from_rotation_y(b.yaw)),
                Visibility::default(),
            ))
            .id();
        for (mesh, material) in &model {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                ChildOf(root),
            ));
        }
    }
}

/// Half-width of a player house, in cells: its front wall (and door) sits
/// 2000 model units = 20 game units from the centre at 1/100 scale.
pub const PLAYER_HOUSE_HALF: f32 = 0.5;

/// Where and how a player house stands (`aMHS_actor_ct`): from the item's
/// cell centre it moves half a cell toward the acre path (+20 game units
/// in x for even houses, -20 for odd) and half a cell south (+20 in z);
/// even houses are turned 90 deg, odd ones 0. The house model stands at
/// 45 deg in its idle pose with the door at the south-west corner, so even
/// houses' doors face south-east and odd ones' south-west: both toward the
/// acre path.
/// Returns (centre, model yaw, door yaw from south).
pub fn player_home_placement(
    house: usize,
    (ax, az, cx, cz): (usize, usize, usize, usize),
) -> (Vec3, f32, f32) {
    let even = house.is_multiple_of(2);
    let shift_x = if even { 0.5 } else { -0.5 };
    let centre = Town::cell_to_world(ax, az, cx, cz) + Vec3::new(shift_x, 0.0, 0.5) * CELL_SIZE;
    let half = std::f32::consts::FRAC_PI_2;
    let quarter = std::f32::consts::FRAC_PI_4;
    let (model_yaw, door_yaw) = if even {
        (half, quarter)
    } else {
        (0.0, -quarter)
    };
    (centre, model_yaw, door_yaw)
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
pub(crate) fn cell_for_item(item: u16) -> Option<CellKind> {
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
/// `residents`: how many villagers live here (their homes go on the
/// layout's reserved spots); `None` keeps the plan's own homes.
pub fn build_retail_town(
    disc: &GameDisc,
    plan: &mut TownPlan,
    residents: Option<usize>,
) -> (RetailLayout, GroundMap) {
    let combos = read_combos(&disc.rom, &disc.catalog);
    let bg_lists = read_bg_lists(&disc.rom, &disc.catalog);
    let fg = read_fg_layouts(&disc.fgdata);
    let field = make_random_field(&combos, &mut seeded_rng(plan.seed));
    let mut units = Vec::with_capacity(BLOCK_X_NUM * BLOCK_Z_NUM);
    let mut tiers = [0u8; BLOCK_X_NUM * BLOCK_Z_NUM];
    let mut sound_sources = Vec::new();
    for (i, combi) in field.combi.iter().enumerate() {
        tiers[i] = combi.height;
        let entry = combos
            .get(combi.combi as usize)
            .and_then(|c| bg_lists.get(&c.bg_id));
        units.push(entry.map_or([Unit::default(); 256], |e| collision_units(&disc.rom, e)));
        // mFM_bg_data_c: six {kind:i16, ut_x:u8, ut_z:u8} records
        // immediately follow collision[16][16], at entry + 0x41C.
        if let Some(raw) =
            entry.and_then(|e| disc.rom.slice(RomSource::Rel, e.collision + 0x400, 6 * 4))
        {
            for (j, source) in raw.as_chunks::<4>().0.iter().enumerate() {
                let id = match i16::from_be_bytes([source[0], source[1]]) {
                    1 => 0x0B, // river
                    2 => 0x1C, // surf
                    _ => continue,
                };
                let block = [i % BLOCK_X_NUM, i / BLOCK_X_NUM];
                let corner = GroundMap::block_corner(block[0], block[1]);
                sound_sources.push(crate::ground_map::TerrainSound {
                    key: (i * 6 + j) as u64,
                    id,
                    block,
                    position: Vec3::new(
                        corner.x + (f32::from(source[2]) + 0.5) * CELL_SIZE,
                        0.0,
                        corner.y + (f32::from(source[3]) + 0.5) * CELL_SIZE,
                    ),
                });
            }
        }
    }
    for (bz, row) in field.combi.chunks(BLOCK_X_NUM).enumerate() {
        let cells: Vec<String> = row
            .iter()
            .map(|c| {
                let bg = combos.get(c.combi as usize).map_or(0, |c| c.bg_id);
                let name = bg_lists
                    .get(&bg)
                    .and_then(|e| e.opaque)
                    .and_then(|o| disc.catalog.containing(disc::RomSource::Rel, o))
                    .map_or("-", |a| a.name);
                format!("{}:{bg:04x}:{}:{name}", c.height, c.combi)
            })
            .collect();
        debug!("field row {bz}: {}", cells.join(" "));
    }
    let mut ground = GroundMap::new(units, tiers);
    for source in &mut sound_sources {
        // ac_field_draw: height is sampled before the shore's +175 Z offset.
        source.position.y = ground
            .height(source.position.x, source.position.z)
            .unwrap_or(0.0)
            + 1.0;
        if source.id == 0x1C {
            source.position.z += 175.0 / 40.0;
        }
    }
    // Waterfalls are standalone actors in retail. Until those actors are
    // present, locate one emitter at the waterfall collision footprint's
    // centre per acre, rather than inventing a global waterfall layer.
    for bz in 0..BLOCK_Z_NUM {
        for bx in 0..BLOCK_X_NUM {
            let mut centre = Vec2::ZERO;
            let mut count = 0;
            for unit in 0..256 {
                if ground.unit_attribute(bx, bz, unit) == 13 {
                    centre += GroundMap::block_corner(bx, bz)
                        + Vec2::new((unit % 16) as f32 + 0.5, (unit / 16) as f32 + 0.5) * CELL_SIZE;
                    count += 1;
                }
            }
            if count > 0 {
                centre /= count as f32;
                sound_sources.push(crate::ground_map::TerrainSound {
                    key: 0x10000 + (bz * BLOCK_X_NUM + bx) as u64,
                    id: 0x0C,
                    block: [bx, bz],
                    position: Vec3::new(
                        centre.x,
                        ground.height(centre.x, centre.y).unwrap_or(0.0) + 1.0,
                        centre.y,
                    ),
                });
            }
        }
    }
    ground.sound_sources = sound_sources;

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
    let mut rocks = HashMap::new();
    for az in 0..ACRE_DEPTH {
        for ax in 0..ACRE_WIDTH {
            let block = (az + 1) * BLOCK_X_NUM + ax + 1;
            if let Some(items) = combos
                .get(field.combi[block].combi as usize)
                .and_then(|c| fg.get(&c.fg_id))
            {
                for (cell, &item) in items.iter().enumerate() {
                    if (ROCK_A..ROCK_A + 5).contains(&item) {
                        rocks.insert((ax, az, cell % 16, cell / 16), (item - ROCK_A) as u8);
                    }
                }
            }
        }
    }
    let structures = find_structures(&field, &combos, &fg);
    let mut player_homes = [None; 4];
    for &(item, ax, az, cx, cz) in &structures {
        if (HOUSE0..HOUSE0 + 4).contains(&item) {
            player_homes[(item - HOUSE0) as usize] = Some((ax, az, cx, cz));
        }
    }
    let buildings = place_buildings(disc, &structures);
    // `mNT_IS_RESERVE`: SIGN00-SIGN20 mark where villager homes can go.
    let reserved: Vec<(usize, usize, usize, usize)> = structures
        .iter()
        .filter(|s| (SIGN00..=SIGN20).contains(&s.0))
        .map(|&(_, ax, az, cx, cz)| (ax, az, cx, cz))
        .collect();
    resite_homes(plan, &player_homes, &ground);
    if let Some(count) = residents.filter(|_| !reserved.is_empty()) {
        place_homes_on_reserved(plan, &reserved, count);
    }
    let mut field_items = Vec::with_capacity(ACRE_WIDTH * ACRE_DEPTH);
    for az in 0..ACRE_DEPTH {
        for ax in 0..ACRE_WIDTH {
            let block = (az + 1) * BLOCK_X_NUM + ax + 1;
            let mut items = combos
                .get(field.combi[block].combi as usize)
                .and_then(|c| fg.get(&c.fg_id))
                .copied()
                .unwrap_or([0; 256]);
            let cells = &plan.acres[az * ACRE_WIDTH + ax].cells;
            for (item, &cell) in items.iter_mut().zip(cells.iter()) {
                if cell_for_item(*item).is_some_and(|k| k as u8 != cell) {
                    *item = 0;
                }
            }
            field_items.push(items);
        }
    }
    let (mut mailboxes, mut gyroids) = ([None; 4], [None; 4]);
    for (a, items) in field_items.iter().enumerate() {
        for (c, &item) in items.iter().enumerate() {
            let at = Some((a % ACRE_WIDTH, a / ACRE_WIDTH, c % 16, c / 16));
            if (MAILBOX0..MAILBOX0 + 4).contains(&item) {
                mailboxes[(item - MAILBOX0) as usize] = at;
            } else if (HANIWA0..HANIWA0 + 4).contains(&item) {
                gyroids[(item - HANIWA0) as usize] = at;
            }
        }
    }
    (
        RetailLayout {
            field,
            combos,
            rocks,
            player_homes,
            buildings,
            mailboxes,
            gyroids,
            items: field_items,
        },
        ground,
    )
}

/// `ACTOR_PROP_MAILBOX0` (`m_name_table.h`): the player houses' mailboxes.
const MAILBOX0: u16 = 0xA000;
/// `ACTOR_PROP_HANIWA0`: the houses' gyroids.
const HANIWA0: u16 = 0xA004;

/// `ROCK_A` (`m_name_table.h`); ROCK_A..=ROCK_E are 0x63..=0x67.
const ROCK_A: u16 = 0x0063;

/// `STRUCTURE_START` items (`m_name_table.h`).
const HOUSE0: u16 = 0x5800;
pub const SHOP0: u16 = 0x5804;

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
    reach: &HashSet<(usize, usize)>,
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
    (cz - 1..=cz + 1).all(|z| (cx - 1..=cx + 1).all(|x| ok(x, z)))
        && ok(cx, cz + 2)
        && reach.contains(&(ax * 16 + cx, az * 16 + cz + 2))
}

/// Nearest fitting home spot in an acre to (cx, cz).
fn nearest_home_spot(
    acre: &TownAcre,
    ground: &GroundMap,
    reach: &HashSet<(usize, usize)>,
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
        .find(|&(x, z)| home_fits(acre, ground, reach, ax, az, x, z))
}

/// Cells (global x, z over the playable acres) reachable on foot from
/// `start`: neighbours joined when the terrain allows the step and the
/// cell holds no tree, rock or home.
fn reachable_cells(
    plan: &TownPlan,
    ground: &GroundMap,
    start: (usize, usize),
) -> HashSet<(usize, usize)> {
    let (w, d) = (ACRE_WIDTH * 16, ACRE_DEPTH * 16);
    let center = |(x, z): (usize, usize)| Town::cell_to_world(x / 16, z / 16, x % 16, z % 16);
    let open = |(x, z): (usize, usize)| {
        let c = plan.acres[(z / 16) * ACRE_WIDTH + x / 16].cells[(z % 16) * 16 + x % 16];
        c == CellKind::Grass as u8 || c == CellKind::Flower as u8 || c == CellKind::Weed as u8
    };
    let mut seen = HashSet::new();
    if !open(start) {
        return seen;
    }
    let mut queue = vec![start];
    seen.insert(start);
    while let Some((x, z)) = queue.pop() {
        let here = center((x, z));
        let neighbours = [
            (x.wrapping_sub(1), z),
            (x + 1, z),
            (x, z.wrapping_sub(1)),
            (x, z + 1),
        ];
        for n in neighbours {
            if n.0 >= w || n.1 >= d || seen.contains(&n) || !open(n) {
                continue;
            }
            if ground.can_step(here, center(n)) {
                seen.insert(n);
                queue.push(n);
            }
        }
    }
    seen
}

/// Moves the plan homes onto the retail town: the player house to the
/// layout HOUSE0 spot, each resident home to the nearest spot in its acre
/// where it fits on the terrain.
fn resite_homes(
    plan: &mut TownPlan,
    player_homes: &[Option<(usize, usize, usize, usize)>; 4],
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

    // Player houses are drawn from the layout (see `spawn_player_homes`),
    // not stamped into the plan; only the acre is flagged.
    if let Some((ax, az, ..)) = player_homes[0] {
        for acre in plan.acres.iter_mut() {
            if acre.feature == Feature::PlayerHouse as u8 {
                acre.feature = Feature::None as u8;
            }
        }
        plan.acres[az * ACRE_WIDTH + ax].feature = Feature::PlayerHouse as u8;
    }

    // Resident homes: same acre, nearest spot that fits and whose
    // doorstep connects to the player house (no homes walled in by
    // trees or cliffs).
    // Flood fill from the cell in front of the player's door.
    let reach = match player_homes[0] {
        Some(cell) => {
            let (centre, _, door_yaw) = player_home_placement(0, cell);
            let step =
                Quat::from_rotation_y(door_yaw) * Vec3::Z * (PLAYER_HOUSE_HALF + 0.75) * CELL_SIZE;
            let front = centre + step;
            let w = Town::cell_indices_of(front);
            match w {
                Some((ax, az, cx, cz)) => {
                    reachable_cells(plan, ground, (ax * 16 + cx, az * 16 + cz))
                }
                None => HashSet::new(),
            }
        }
        None => HashSet::new(),
    };
    for i in 0..plan.villager_count as usize {
        let a = plan.house_acres[i] as usize;
        let unit = plan.house_units[i] as usize;
        let (ax, az) = (a % ACRE_WIDTH, a / ACRE_WIDTH);
        let spot = nearest_home_spot(&plan.acres[a], ground, &reach, ax, az, unit % 16, unit / 16)
            .or_else(|| {
                // Nothing reachable in its own acre: try the others.
                (0..ACRE_WIDTH * ACRE_DEPTH)
                    .find_map(|b| {
                        let (bx, bz) = (b % ACRE_WIDTH, b / ACRE_WIDTH);
                        nearest_home_spot(&plan.acres[b], ground, &reach, bx, bz, 8, 8)
                            .map(|s| (b, s))
                    })
                    .map(|(b, s)| {
                        plan.house_acres[i] = b as u8;
                        s
                    })
            });
        let a = plan.house_acres[i] as usize;
        match spot {
            Some((x, z)) => {
                stamp_home(&mut plan.acres[a], x, z);
                plan.house_units[i] = (z * 16 + x) as u16;
            }
            None => warn!("no room for resident home {i} in acre ({ax},{az})"),
        }
    }
}

/// `SIGN00` / `SIGN20`: reserved villager home spots in the layouts.
const SIGN00: u16 = 0x5810;
const SIGN20: u16 = 0x5824;

/// `mNpc_SetNpcHome`: resident `i` lives on reserved spot `order[i]`, the
/// house centred on the spot's cell with its signboard down and left.
fn place_homes_on_reserved(
    plan: &mut TownPlan,
    reserved: &[(usize, usize, usize, usize)],
    count: usize,
) {
    // Clear the plan's homes; the reserved spots replace them.
    for acre in plan.acres.iter_mut() {
        let homes: Vec<usize> = (0..256)
            .filter(|&c| acre.cells[c] == CellKind::House as u8)
            .collect();
        for c in homes {
            clear_home(acre, c % 16, c / 16);
        }
    }
    let order = crate::roster::home_spot_order(reserved.len(), plan.seed);
    let count = count.min(reserved.len()).min(plan.house_acres.len());
    for (i, &spot) in order.iter().take(count).enumerate() {
        let (ax, az, cx, cz) = reserved[spot];
        if !(1..15).contains(&cx) || !(1..15).contains(&cz) {
            continue;
        }
        let a = az * ACRE_WIDTH + ax;
        stamp_home(&mut plan.acres[a], cx, cz);
        plan.house_acres[i] = a as u8;
        plan.house_units[i] = (cz * 16 + cx) as u16;
    }
    plan.villager_count = count as u8;
}

pub(crate) fn build_disc_town(
    mut commands: Commands,
    disc: Res<GameDisc>,
    town: Res<Town>,
    placeholders: Query<Entity, With<PlaceholderGround>>,
    trees: Query<(Entity, &Transform, &PlaceholderTree)>,
    objects: Query<(Entity, &PlaceholderObject)>,
    mut player: Query<&mut Transform, (With<Player>, Without<PlaceholderTree>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
    roster: Option<Res<crate::roster::TownRoster>>,
) {
    let (Some(retail), Some(ground)) = (&town.retail, &town.ground) else {
        return;
    };
    let house_looks = resident_house_looks(&disc, &town, roster.as_deref());
    let segs = ground_segments(&disc.catalog, current_term());
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
        current_term(),
        &mut meshes,
        &mut materials,
        &mut images,
    );
    let winter = is_winter(current_term());
    let mut lights = BuildingLights::default();
    spawn_buildings(
        &mut commands,
        &disc,
        &town,
        winter,
        &mut meshes,
        &mut materials,
        &mut images,
        &mut lights,
    );
    spawn_player_homes(
        &mut commands,
        &disc,
        &town,
        winter,
        &mut meshes,
        &mut materials,
        &mut images,
        &mut lights,
    );
    crate::mailbox::spawn_mailboxes(
        &mut commands,
        &disc,
        &town,
        winter,
        (&mut meshes, &mut materials, &mut images),
    );
    crate::gyroid::spawn_gyroids(
        &mut commands,
        &disc,
        &town,
        (&mut meshes, &mut materials, &mut images),
    );
    spawn_disc_objects(
        &mut commands,
        &disc,
        &town,
        &objects,
        winter,
        &mut meshes,
        &mut materials,
        &mut images,
        &mut lights,
        &house_looks,
    );
    commands.insert_resource(lights);
    info!(
        "retail town from disc: {} step, {} combinations",
        if retail.field.three_step { "3" } else { "2" },
        retail.combos.len()
    );
}

/// Object models are drawn at 1/100 scale; one game cell is 40 units.
pub(crate) const OBJECT_SCALE: f32 = 0.01 / 40.0 * CELL_SIZE;

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
    term: usize,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) {
    let palette_index = TREE_PAL_IDX[term % 18];
    let lists = tree_lists(&disc.catalog, is_winter(term));
    if lists.len() != 4 {
        warn!("tree model not found on the disc; keeping placeholder trees");
        return;
    }
    // Tree colours come from the 14 `mFM_obj_tree_01_pal_dol` palettes.
    let palette = disc
        .catalog
        .get("mFM_obj_tree_01_pal_dol")
        .map(|p| p.offset + (palette_index % (p.size / PALETTE_BYTES).max(1)) * PALETTE_BYTES);
    let parts = build_model(disc, &lists, palette, meshes, materials, images);
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

/// Meshes and materials for a disc model made of `lists` (each a display
/// list REL offset, run in order with shared state).
pub(crate) fn build_model(
    disc: &GameDisc,
    lists: &[u32],
    palette: Option<u32>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) -> Vec<(Handle<Mesh>, Handle<N64Material>)> {
    build_model_with_prim(
        disc, lists, palette, [255; 4], 255, false, meshes, materials, images,
    )
}

/// As [`build_model`], with the primitive colour/LOD game code sets first.
#[allow(clippy::too_many_arguments)]
fn build_model_with_prim(
    disc: &GameDisc,
    lists: &[u32],
    palette: Option<u32>,
    prim: [u8; 4],
    prim_lod: u8,
    blend: bool,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) -> Vec<(Handle<Mesh>, Handle<N64Material>)> {
    let colors = DrawColors {
        prim,
        prim_lod,
        env: [0; 4],
    };
    build_model_full(
        disc,
        lists,
        palette,
        colors,
        blend,
        (&Segments::default(), &[]),
        (meshes, materials, images),
    )
}

/// Colours the draw code sets before a model's lists.
#[derive(Clone, Copy)]
pub(crate) struct DrawColors {
    pub prim: [u8; 4],
    pub prim_lod: u8,
    pub env: [u8; 4],
}

/// As [`build_model_with_prim`], with the environment colour, and the
/// segments (`gSPSegment`) and TLUT slots (slot, REL offset) the draw
/// code sets.
pub(crate) fn build_model_full(
    disc: &GameDisc,
    lists: &[u32],
    palette: Option<u32>,
    colors: DrawColors,
    blend: bool,
    (segs, tluts): (&Segments, &[(usize, u32)]),
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> Vec<(Handle<Mesh>, Handle<N64Material>)> {
    let mut sink = crate::disc_gfx::run_display_lists_with_colors(
        &disc.rom,
        lists,
        colors.prim,
        colors.prim_lod,
        colors.env,
        tluts,
    );
    pose_parts(disc, lists, &mut sink);
    sink_meshes(
        disc,
        &sink,
        palette,
        blend,
        segs,
        (meshes, materials, images),
    )
}

/// Meshes and materials for an already run (and placed) sink.
pub(crate) fn sink_meshes(
    disc: &GameDisc,
    sink: &crate::disc_gfx::TriangleSink,
    palette: Option<u32>,
    blend: bool,
    segs: &Segments,
    assets: (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> Vec<(Handle<Mesh>, Handle<N64Material>)> {
    sink_meshes_scaled(disc, sink, palette, blend, segs, OBJECT_SCALE, assets)
}

/// As [`sink_meshes`], with model units scaled by `scale`.
pub(crate) fn sink_meshes_scaled(
    disc: &GameDisc,
    sink: &crate::disc_gfx::TriangleSink,
    palette: Option<u32>,
    blend: bool,
    segs: &Segments,
    scale: f32,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> Vec<(Handle<Mesh>, Handle<N64Material>)> {
    sink.parts
        .iter()
        .map(|part| {
            let texture = part.key.tex.as_ref().and_then(|k| {
                decode_texture(&disc.rom, k, palette, segs)
                    .map(|rgba| images.add(texture_image(rgba, k)))
            });
            let material = N64Material {
                combiner: CombinerUniform::from_key(&part.key, false, blend),
                texture,
                env: default(),
                blend,
            };
            (meshes.add(part_mesh(part, scale)), materials.add(material))
        })
        .collect()
}

/// Structure skeletons whose parts are drawn in their idle pose: houses,
/// player houses and public buildings, summer and winter.
const POSED_STRUCTURES: [&str; 14] = [
    "house1",
    "house2",
    "house3",
    "house4",
    "house5",
    "myhome1",
    "myhome2",
    "myhome3",
    "myhome4",
    "shop1",
    "yubinkyoku",
    "kouban",
    "museum",
    "tailor",
];

/// Model-space transform of each structure part's display list (REL
/// offset): its joint's world matrix in the structure's idle pose, frame
/// 1 of its `cKF_ba_r_obj_*` animation (the actors start it in STOP mode,
/// e.g. `aHUS_setup_animation`). Built once per run.
fn structure_poses(disc: &GameDisc) -> &'static HashMap<u32, Mat4> {
    static POSES: std::sync::OnceLock<HashMap<u32, Mat4>> = std::sync::OnceLock::new();
    POSES.get_or_init(|| {
        let mut out = HashMap::new();
        for season in ["s", "w"] {
            for name in POSED_STRUCTURES {
                let Some(skel) = disc
                    .catalog
                    .get(&format!("cKF_bs_r_obj_{season}_{name}"))
                    .and_then(|a| crate::skeleton::Skeleton::read(&disc.rom, a.offset))
                else {
                    continue;
                };
                // The winter mailbox has no animations of its own:
                // ac_mailbox.c poses both skeletons with the summer ones.
                let anim = disc
                    .catalog
                    .get(&format!("cKF_ba_r_obj_{season}_{name}"))
                    .or_else(|| disc.catalog.get(&format!("cKF_ba_r_obj_s_{name}")))
                    .and_then(|a| {
                        crate::skeleton::Animation::read(&disc.rom, a.offset, skel.joints.len())
                    });
                let world = skel.world_matrices(anim.as_ref(), 1.0);
                for (j, joint) in skel.joints.iter().enumerate() {
                    if let Some(list) = joint.model {
                        out.insert(list, world[j]);
                    }
                }
            }
        }
        out
    })
}

/// Moves each vertex onto its structure joint (see [`structure_poses`]).
fn pose_parts(disc: &GameDisc, lists: &[u32], sink: &mut crate::disc_gfx::TriangleSink) {
    let poses = structure_poses(disc);
    for v in sink.parts.iter_mut().flat_map(|p| p.vertices.iter_mut()) {
        if v.mtx & 0xFF00_0000 != LIST_TAG {
            continue;
        }
        let Some(m) = lists
            .get((v.mtx & 0x00FF_FFFF) as usize)
            .and_then(|l| poses.get(l))
        else {
            continue;
        };
        v.pos = m.transform_point3(Vec3::from(v.pos)).to_array();
        v.normal = m
            .transform_vector3(Vec3::from(v.normal))
            .normalize_or_zero()
            .to_array();
    }
}

/// Display lists of a multi-part model `<prefix>_<part>_model`, in order,
/// skipping parts the disc doesn't have.
fn part_lists(catalog: &AssetCatalog, prefix: &str, parts: &[&str]) -> Vec<u32> {
    parts
        .iter()
        .filter_map(|p| catalog.get(&format!("{prefix}_{p}_model")))
        .map(|a| a.offset)
        .collect()
}

/// Yaw that turns a model so its `door` part faces south (+Z, towards the
/// camera): the door's average vertex position gives the facing, snapped
/// to a quarter turn.
fn door_facing_yaw(disc: &GameDisc, prefix: &str) -> f32 {
    let lists = part_lists(&disc.catalog, prefix, &["door"]);
    if lists.is_empty() {
        return 0.0;
    }
    let mut sink = run_display_lists(&disc.rom, &lists);
    pose_parts(disc, &lists, &mut sink);
    let verts: Vec<[f32; 3]> = sink
        .parts
        .iter()
        .flat_map(|p| p.vertices.iter().map(|v| v.pos))
        .collect();
    if verts.is_empty() {
        return 0.0;
    }
    let n = verts.len() as f32;
    let (x, z) = verts
        .iter()
        .fold((0.0, 0.0), |(x, z), v| (x + v[0] / n, z + v[2] / n));
    // House fronts are axis-aligned in the model; a door off the front's
    // centre skews the angle, so snap to the nearest quarter turn.
    let quarter = std::f32::consts::FRAC_PI_2;
    (-x.atan2(z) / quarter).round() * quarter
}

/// House model scale relative to other objects. Back at 1.0 after a 3x
/// try played badly; Philip's rule: change house size by at most 25% at a
/// time.
const HOUSE_SCALE: f32 = 1.0;

/// A structure's window parts: the `light` glass (tinted by the primitive
/// colour) and the `window*` glow overlay (alpha = texel x LOD fraction),
/// registered so `environment` switches them with the clock.
#[allow(clippy::too_many_arguments)]
fn build_windows(
    disc: &GameDisc,
    prefix: &str,
    glass_part: &str,
    palette: Option<u32>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
    lights: &mut BuildingLights,
) -> Vec<(Handle<Mesh>, Handle<N64Material>)> {
    let glass = part_lists(&disc.catalog, prefix, &[glass_part]);
    let mut model = build_model_with_prim(
        disc,
        &glass,
        palette,
        HOUSE_LIGHT_PRIM_DAY,
        0,
        false,
        meshes,
        materials,
        images,
    );
    lights.glass.extend(model.iter().map(|(_, m)| m.clone()));
    let glow = part_lists(
        &disc.catalog,
        prefix,
        &["window", "windowL", "windowR", "windowT"],
    );
    let glow = build_model_with_prim(
        disc,
        &glow,
        palette,
        WINDOW_GLOW_PRIM,
        0,
        true,
        meshes,
        materials,
        images,
    );
    lights.glow.extend(glow.iter().map(|(_, m)| m.clone()));
    model.extend(glow);
    model
}

/// Colour of the window glow overlay (`aHUS` / `aMHS` draw): its strength
/// is the LOD fraction.
const WINDOW_GLOW_PRIM: [u8; 4] = [255, 255, 150, 255];

/// A house's `light` part: the window glass, tinted by the primitive colour
/// the house actor sets (`aHUS_actor_draw_before` / `aMHS_actor_draw_*`):
/// (0, 0, 0) with the lights off rising to (255, 255, 150) on; this is the
/// starting (off) colour, `environment` changes it with the clock.
const HOUSE_LIGHT_PRIM_DAY: [u8; 4] = [0, 0, 0, 255];

/// Small deterministic hash for picking per-object variants.
fn cell_hash((ax, az, cx, cz): (usize, usize, usize, usize)) -> usize {
    let mut h = (ax * 16 + cx) as u32 ^ (((az * 16 + cz) as u32) << 8);
    h = h.wrapping_mul(0x9E37_79B9);
    (h >> 16) as usize
}

/// Which disc model (and palette asset) a placeholder becomes.
/// House shape (0-4) and roof palette (0-4) by home cell, per resident.
type HouseLooks = HashMap<(usize, usize, usize, usize), (u8, u8)>;

/// Each resident's house look from `npc_house_list` (type, palette),
/// keyed by their home cell (resident i lives at plan home i).
fn resident_house_looks(
    disc: &GameDisc,
    town: &Town,
    roster: Option<&crate::roster::TownRoster>,
) -> HouseLooks {
    let mut out = HouseLooks::new();
    let (Some(roster), Some(list)) = (roster, disc.catalog.get("npc_house_list")) else {
        return out;
    };
    let plan = &town.plan;
    for (i, resident) in roster
        .residents
        .iter()
        .enumerate()
        .take(plan.villager_count as usize)
    {
        let Some(b) = disc
            .rom
            .slice(RomSource::Rel, list.offset + resident.index * 8, 2)
        else {
            continue;
        };
        let a = plan.house_acres[i] as usize;
        let u = plan.house_units[i] as usize;
        out.insert(
            (a % ACRE_WIDTH, a / ACRE_WIDTH, u % 16, u / 16),
            (b[0], b[1]),
        );
    }
    out
}

fn object_model(
    town: &Town,
    kind: CellKind,
    cell: (usize, usize, usize, usize),
    winter: bool,
    house_looks: &HouseLooks,
) -> Option<(String, Vec<&'static str>, Option<String>)> {
    let season = if winter { "w" } else { "s" };
    match kind {
        CellKind::Rock => {
            let town_rocks = town.retail.as_ref().map(|r| &r.rocks);
            let v = town_rocks.and_then(|r| r.get(&cell)).copied().unwrap_or(0);
            let letter = (b'A' + v) as char;
            // stoneA carries the material list the other shapes share.
            Some((
                format!("obj_{season}_stone{letter}"),
                vec!["gfx"],
                Some(format!("obj_{season}_stoneA_pal")),
            ))
        }
        CellKind::House => {
            // The resident's own house (`npc_house_list` type / palette, as
            // `aHUS_actor_ct` picks them); a hash for houses nobody owns.
            let (shape, colour) = match house_looks.get(&cell) {
                Some(&(t, p)) => (t as usize % 5 + 1, (b'a' + p % 5) as char),
                None => {
                    let h = cell_hash(cell);
                    (h % 5 + 1, (b'a' + (h / 5 % 5) as u8) as char)
                }
            };
            let parts = vec!["t1", "t2", "t3", "t4", "door", "name"];
            Some((
                format!("obj_{season}_house{shape}"),
                parts,
                Some(format!("obj_{season}_house{shape}_{colour}_pal")),
            ))
        }
        _ => None,
    }
}

/// Replaces house and rock placeholders with their disc models; each
/// distinct model is built once and shared.
#[allow(clippy::too_many_arguments)]
fn spawn_disc_objects(
    commands: &mut Commands,
    disc: &GameDisc,
    town: &Town,
    placeholders: &Query<(Entity, &PlaceholderObject)>,
    winter: bool,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
    lights: &mut BuildingLights,
    house_looks: &HouseLooks,
) {
    /// Shared meshes/materials and the yaw that faces the door south.
    type Model = (Vec<(Handle<Mesh>, Handle<N64Material>)>, f32);
    let mut built: HashMap<(String, Option<String>), Model> = HashMap::new();
    let mut count = 0;
    for (entity, obj) in placeholders {
        let Some((prefix, parts, palette)) =
            object_model(town, obj.kind, obj.cell, winter, house_looks)
        else {
            continue;
        };
        let key = (prefix.clone(), palette.clone());
        if !built.contains_key(&key) {
            let mut lists = Vec::new();
            if obj.kind == CellKind::Rock {
                let season = if winter { "w" } else { "s" };
                lists.extend(part_lists(
                    &disc.catalog,
                    &format!("obj_{season}_stoneA"),
                    &["mat"],
                ));
            }
            lists.extend(part_lists(&disc.catalog, &prefix, &parts));
            let pal = palette
                .as_deref()
                .and_then(|p| disc.catalog.get(p))
                .map(|a| a.offset);
            debug!(
                "object model {prefix} palette {palette:?} -> {pal:?}, {} lists",
                lists.len()
            );
            let mut model = if lists.is_empty() {
                Vec::new()
            } else {
                build_model(disc, &lists, pal, meshes, materials, images)
            };
            if obj.kind == CellKind::House && !model.is_empty() {
                model.extend(build_windows(
                    disc, &prefix, "light", pal, meshes, materials, images, lights,
                ));
            }
            let yaw = if obj.kind == CellKind::House {
                door_facing_yaw(disc, &prefix)
            } else {
                0.0
            };
            built.insert(key.clone(), (model, yaw));
        }
        let (model, yaw) = &built[&key];
        if model.is_empty() {
            continue; // keep the placeholder when the disc lacks the model
        }
        commands.entity(entity).despawn();
        let (ax, az, cx, cz) = obj.cell;
        let p = Town::cell_to_world(ax, az, cx, cz);
        let root = commands
            .spawn((
                DiscObject,
                Name::new(prefix.clone()),
                Transform::from_xyz(p.x, obj.ground_y, p.z)
                    .with_rotation(Quat::from_rotation_y(*yaw))
                    .with_scale(Vec3::splat(if obj.kind == CellKind::House {
                        HOUSE_SCALE
                    } else {
                        1.0
                    })),
                Visibility::default(),
            ))
            .id();
        for (mesh, material) in model {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                ChildOf(root),
            ));
        }
        count += 1;
    }
    info!(
        "{count} houses and rocks drawn from the disc ({} models)",
        built.len()
    );
}

/// Draws the four player houses from the layout (`aMHS_actor_ct`
/// placement), sharing one model per house size.
fn spawn_player_homes(
    commands: &mut Commands,
    disc: &GameDisc,
    town: &Town,
    winter: bool,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
    lights: &mut BuildingLights,
) {
    let Some(retail) = &town.retail else { return };
    let season = if winter { "w" } else { "s" };
    // House size comes from the save (upgrades); everyone starts at size 1.
    let prefix = format!("obj_{season}_myhome1");
    // `aMHS_actor_draw_before` hides the fish weathervane (kazamiA/B,
    // joints 3/5) and the insect plaque (fuda, joint 1) until the player
    // has finished that encyclopedia, which nothing tracks yet.
    let parts = ["t1", "t2", "t3", "door", "mark"];
    let lists = part_lists(&disc.catalog, &prefix, &parts);
    if lists.is_empty() {
        warn!("player house model not found on the disc");
        return;
    }
    // Roof colour: the house's saved paint (`outlook_pal`); `a` for now.
    let palette = disc
        .catalog
        .get(&format!("obj_{season}_myhome_a_pal"))
        .map(|a| a.offset);
    // The door design (`mark`): segments 9/A hold the default
    // `obj_myhome_mark_tex_txt` / `_pal` until the player hangs their own.
    let mut segs = Segments::default();
    for (seg, name) in [
        (0x09, "obj_myhome_mark_tex_txt"),
        (0x0A, "obj_myhome_mark_pal"),
    ] {
        if let Some(bytes) = disc
            .catalog
            .get(name)
            .and_then(|a| disc.rom.slice(disc::RomSource::Rel, a.offset, a.size))
        {
            segs.set_segment(seg, bytes.into());
        }
    }
    let colors = DrawColors {
        prim: [255; 4],
        prim_lod: 255,
        env: [0; 4],
    };
    let mut model = build_model_full(
        disc,
        &lists,
        palette,
        colors,
        false,
        (&segs, &[]),
        (meshes, materials, images),
    );
    model.extend(build_windows(
        disc, &prefix, "light", palette, meshes, materials, images, lights,
    ));
    for (house, cell) in retail.player_homes.iter().enumerate() {
        let Some(cell) = *cell else { continue };
        let (centre, yaw, _) = player_home_placement(house, cell);
        let ground_y = town.ground_y(centre);
        let root = commands
            .spawn((
                DiscObject,
                Name::new(format!("player house {house}")),
                Transform::from_xyz(centre.x, ground_y, centre.z)
                    .with_rotation(Quat::from_rotation_y(yaw)),
                Visibility::default(),
            ))
            .id();
        for (mesh, material) in &model {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                ChildOf(root),
            ));
        }
    }
}

/// Where a mailbox stands: its unit's centre.
pub fn mailbox_position((ax, az, cx, cz): (usize, usize, usize, usize)) -> Vec3 {
    Town::cell_to_world(ax, az, cx, cz)
}

/// Marker for a disc-drawn house or rock.
#[derive(Component)]
pub struct DiscObject;

/// Marker for a disc-drawn tree.
#[derive(Component)]
pub struct DiscTree;

/// Player capsule centre above the ground.
pub(crate) const PLAYER_HALF_HEIGHT: f32 = 1.0;

/// Keeps the player and villagers on the disc terrain's surface.
fn follow_ground(
    town: Res<Town>,
    mut walkers: Query<&mut Transform, Or<(With<Player>, With<crate::npc::Villager>)>>,
) {
    let Some(ground) = &town.ground else { return };
    for mut t in &mut walkers {
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

    /// Probe (needs the disc): joints of a skeleton and their lists.
    #[test]
    #[ignore]
    fn probe_skeleton_joints() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let disc = crate::game_disc::load(path.into()).unwrap();
        let name = std::env::var("SKELETON").unwrap_or("cKF_bs_r_obj_s_myhome1".into());
        let a = disc.catalog.get(&name).unwrap();
        let skel = crate::skeleton::Skeleton::read(&disc.rom, a.offset).unwrap();
        for (j, joint) in skel.joints.iter().enumerate() {
            let list = joint
                .model
                .and_then(|m| disc.catalog.containing(disc::RomSource::Rel, m))
                .map_or("-", |a| a.name);
            println!("{j} {list}");
        }
    }

    /// Probe (needs the disc): house parts' textures and transparency.
    #[test]
    #[ignore]
    fn probe_house_parts() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut disc = disc::Disc::open_path(std::path::Path::new(&path)).unwrap();
        let rom = RomImage::from_disc(&mut disc).unwrap();
        let cat = AssetCatalog::builtin(&rom);
        for shape in 1..=5 {
            let pal = cat
                .get(&format!("obj_s_house{shape}_a_pal"))
                .map(|a| a.offset);
            for part in [
                "t1", "t2", "t3", "t4", "door", "window", "windowL", "windowR", "name",
            ] {
                let Some(a) = cat.get(&format!("obj_s_house{shape}_{part}_model")) else {
                    continue;
                };
                let sink = run_display_lists(&rom, &[a.offset]);
                for p in &sink.parts {
                    let info = p.key.tex.map(|k| {
                        let px = decode_texture(&rom, &k, pal, &Segments::default());
                        let clear = px.as_ref().map(|px| {
                            px.chunks(4).filter(|c| c[3] < 128).count() * 100
                                / (px.len() / 4).max(1)
                        });
                        format!(
                            "{:?} {}x{} tlut={:x} clear%={clear:?}",
                            k.format, k.width, k.height, k.tlut_addr
                        )
                    });
                    eprintln!(
                        "house{shape} {part}: tris={} combine={:06x}/{:08x} {info:?}",
                        p.vertices.len() / 3,
                        p.key.combine.0,
                        p.key.combine.1
                    );
                }
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
    fn player_homes_placed_like_amhs_actor_ct() {
        let cell = (1, 1, 3, 3);
        let base = Town::cell_to_world(1, 1, 3, 3);
        let (c0, m0, d0) = player_home_placement(0, cell);
        assert!((c0 - base - Vec3::new(0.5, 0.0, 0.5) * CELL_SIZE).length() < 1e-5);
        assert_eq!(
            (m0, d0),
            (std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_4),
            "even: turned 90 deg, door south-east"
        );
        let (c1, m1, d1) = player_home_placement(1, cell);
        assert!((c1 - base - Vec3::new(-0.5, 0.0, 0.5) * CELL_SIZE).length() < 1e-5);
        assert_eq!(
            (m1, d1),
            (0.0, -std::f32::consts::FRAC_PI_4),
            "odd: not turned, door south-west"
        );
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
