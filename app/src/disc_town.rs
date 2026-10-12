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
use rustimal_logic::random_field::{make_random_field, ComboInfo, BLOCK_X_NUM, BLOCK_Z_NUM};
use rustimal_logic::town_gen::{ACRE_DEPTH, ACRE_WIDTH};

use crate::disc_gfx::{decode_texture, run_display_lists, Segments};
use crate::game_disc::GameDisc;
use crate::n64_material::{part_mesh, texture_image, CombinerUniform, N64Material};
use crate::states::AppState;
use crate::town::{PlaceholderGround, Town, ACRE_SIZE, CELL_SIZE};

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

/// `bg_id` → (opaque list, translucent list) REL offsets from `data_bgd`.
fn read_bg_lists(
    rom: &RomImage,
    catalog: &AssetCatalog,
) -> HashMap<u16, (Option<u32>, Option<u32>)> {
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
            (list(at + 4), list(at + 8)),
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

fn build_disc_town(
    mut commands: Commands,
    disc: Res<GameDisc>,
    town: Res<Town>,
    placeholders: Query<Entity, With<PlaceholderGround>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let segs = ground_segments(&disc.catalog, chrono::Local::now().month0());
    let combos = read_combos(&disc.rom, &disc.catalog);
    let bg_lists = read_bg_lists(&disc.rom, &disc.catalog);
    let field = make_random_field(&combos, &mut seeded_rng(town.plan.seed));
    let mut missing = 0;
    for bz in 0..BLOCK_Z_NUM {
        for bx in 0..BLOCK_X_NUM {
            let combi = field.combi[bz * BLOCK_X_NUM + bx];
            let Some(&(opaque, translucent)) = combos
                .get(combi.combi as usize)
                .and_then(|c| bg_lists.get(&c.bg_id))
            else {
                missing += 1;
                continue;
            };
            let lists: Vec<(u32, bool)> = [(opaque, false), (translucent, true)]
                .into_iter()
                .filter_map(|(l, blend)| l.map(|l| (l, blend)))
                .collect();
            // The playable 5x6 acres are blocks (1..=5, 1..=6); the rest is
            // the border ring, rail line, beach and ocean.
            let corner = Vec3::new(
                (bx as f32 - 1.0) * ACRE_SIZE - ACRE_WIDTH as f32 * ACRE_SIZE / 2.0,
                -GROUND_BASE_Y * GROUND_SCALE,
                (bz as f32 - 1.0) * ACRE_SIZE - ACRE_DEPTH as f32 * ACRE_SIZE / 2.0,
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
    info!(
        "retail town from disc: {} step, {} combinations",
        if field.three_step { "3" } else { "2" },
        combos.len()
    );
    for e in &placeholders {
        commands.entity(e).despawn();
    }
}

pub struct DiscTownPlugin;

impl Plugin for DiscTownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            build_disc_town.run_if(resource_exists::<GameDisc>.and_then(resource_exists::<Town>)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
