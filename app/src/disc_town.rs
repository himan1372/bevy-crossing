//! Draws the town ground from the player's disc: each acre becomes one of
//! the game's own acre templates (`grd_s_*`), replacing the flat
//! placeholder planes.
//!
//! Template choice: the retail names encode the acre shape. `c1`..`c7` are
//! the cliff block types (horizontal, bottom-right corner, vertical right,
//! top-right corner, top-left corner, vertical left, bottom-left corner)
//! and `r1`..`r7` the river block types (south, east, west, south-east,
//! east-south, south-west, west-south), in `logic::field_gen` order. An
//! acre's cliff/river edges pick the shape; the variant number is chosen
//! per acre from the town seed. This is a visual mapping: collision still
//! comes from the generated town plan.
//!
//! Ground textures live in runtime buffers (.bss) that the game fills with
//! the season's textures, and palettes are picked per month; both are
//! rebuilt here as a [`Segments`] map.

use bevy::prelude::*;
use chrono::Datelike;
use disc::{rel_bss, AssetCatalog, RomImage};
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
];

/// (bss buffer, palette set); sets with several palettes are per month.
const GROUND_PALETTES: &[(&str, &str)] = &[
    ("bush_pal_dummy", "mFM_bush_pal"),
    ("cliff_pal_dummy", "mFM_cliff_pal"),
    ("earth_pal_dummy", "mFM_earth_pal"),
    ("station_pal_dummy", "mFM_grd_s_station1_pal"),
    ("bridge_1_pal_dummy", "mFM_grd_s_bridge1_pal"),
    ("bridge_2_pal_dummy", "mFM_grd_s_bridge2_pal"),
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

/// N=1, E=2, S=4, W=8 edge bits → retail river shape 1..7.
fn river_shape(edges: u8) -> Option<u8> {
    match edges {
        0b0101 => Some(1),      // N-S
        0b1010 => Some(2),      // W-E
        0b0011 => Some(4),      // N-E
        0b1100 => Some(5),      // W-S
        0b1001 => Some(6),      // N-W
        0b0110 => Some(7),      // E-S
        e if e != 0 => Some(1), // anything else: straight
        _ => None,
    }
}

/// N=1, E=2, S=4, W=8 edge bits → retail cliff shape 1..7.
fn cliff_shape(edges: u8) -> Option<u8> {
    match edges {
        0b1010 => Some(1), // W-E: horizontal
        0b0110 => Some(2), // E-S: bottom-right corner
        0b0101 => Some(3), // N-S: vertical
        0b0011 => Some(4), // N-E: top-right corner
        0b1001 => Some(5), // N-W: top-left corner
        0b1100 => Some(7), // W-S: bottom-left corner
        e if e != 0 => Some(1),
        _ => None,
    }
}

/// Template base names to try, most specific first.
fn template_bases(cliff: Option<u8>, river: Option<u8>) -> Vec<String> {
    let mut bases = Vec::new();
    match (cliff, river) {
        (Some(c), Some(r)) => {
            bases.push(format!("grd_s_c{c}_r{r}"));
            bases.push(format!("grd_s_r{r}"));
            bases.push(format!("grd_s_c{c}"));
        }
        (Some(c), None) => bases.push(format!("grd_s_c{c}")),
        (None, Some(r)) => bases.push(format!("grd_s_r{r}")),
        (None, None) => {}
    }
    bases.push("grd_s_f".to_string());
    bases
}

/// `<base>_<n>` names that have a ground model on the disc.
fn variants(catalog: &AssetCatalog, base: &str) -> Vec<String> {
    (1..=9)
        .map(|n| format!("{base}_{n}"))
        .filter(|name| catalog.get(&format!("{name}_model")).is_some())
        .collect()
}

fn pick_template(catalog: &AssetCatalog, cliff: u8, river: u8, salt: u32) -> Option<String> {
    template_bases(cliff_shape(cliff), river_shape(river))
        .iter()
        .map(|b| variants(catalog, b))
        .find(|v| !v.is_empty())
        .map(|v| v[salt as usize % v.len()].clone())
}

/// REL offsets of an acre's display lists: the opaque ground, then its
/// translucent layer (water, etc.), drawn alpha-blended.
fn acre_lists(catalog: &AssetCatalog, name: &str) -> Vec<(u32, bool)> {
    [("_model", false), ("_modelT", true)]
        .iter()
        .filter_map(|(suffix, blend)| {
            catalog
                .get(&format!("{name}{suffix}"))
                .map(|a| (a.offset, *blend))
        })
        .collect()
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
    let mut used = Vec::new();
    for az in 0..ACRE_DEPTH {
        for ax in 0..ACRE_WIDTH {
            let i = az * ACRE_WIDTH + ax;
            let acre = &town.plan.acres[i];
            let salt = town.plan.seed.wrapping_mul(31).wrapping_add(i as u32 * 7);
            let Some(name) = pick_template(&disc.catalog, acre.cliff_edges, acre.river_edges, salt)
            else {
                warn!("acre ({ax},{az}): no ground template on the disc");
                continue;
            };
            // Acre models span 0..16 cells from their north-west corner.
            let corner = Vec3::new(
                ax as f32 * ACRE_SIZE - ACRE_WIDTH as f32 * ACRE_SIZE / 2.0,
                -GROUND_BASE_Y * GROUND_SCALE,
                az as f32 * ACRE_SIZE - ACRE_DEPTH as f32 * ACRE_SIZE / 2.0,
            );
            let lists = acre_lists(&disc.catalog, &name);
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
            used.push(name);
        }
    }
    for e in &placeholders {
        commands.entity(e).despawn();
    }
    info!("town ground from disc: {}", used.join(" "));
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
    fn shapes_from_edges() {
        assert_eq!(river_shape(0b0101), Some(1));
        assert_eq!(river_shape(0), None);
        assert_eq!(cliff_shape(0b1010), Some(1));
        assert_eq!(
            template_bases(Some(1), Some(1)),
            ["grd_s_c1_r1", "grd_s_r1", "grd_s_c1", "grd_s_f"]
        );
        assert_eq!(template_bases(None, None), ["grd_s_f"]);
    }
}
