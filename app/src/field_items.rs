//! Town field items: every acre's 16x16 grid of `mActor_name_t` items,
//! as the save keeps them (`mFM_fg_c`). Starts from the acre layouts
//! (`fgdata.bin`), with the native fruit trees of a new town
//! (`mAGrw_ChangeTree2FruitTree`) and the shells the waves wash up
//! (`mFI_SetShell`, from `rustimal_logic::field_items`).
//!
//! Drawn the way the `bg_item` actor draws them, from its own tables on
//! the disc (`bg_item_data.c_inc`): an item's type data picks a
//! `draw_part_table_a` entry and a sub index, the part's draw lists name
//! the display lists, and items stand at the unit's `pos_table` spot
//! (plus `rnd_data` when the part asks), scaled 0.01. Trees and rocks are
//! drawn by `disc_town`; here a fruit tree only adds its fruit.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use disc::{RomSource, Target};
use rustimal_logic::field_items::{self as logic, BeachBlock, EMPTY_NO, SET_SHELL_NUM};

use crate::disc_town::{build_model, cell_for_item, seeded_rng};
use crate::game_disc::GameDisc;
use crate::n64_material::N64Material;
use crate::player::Player;
use crate::states::AppState;
use crate::town::{Town, CELL_SIZE};
use rustimal_logic::town_gen::{CellKind, ACRE_DEPTH, ACRE_WIDTH};

/// `bIT_TYPE_BASE` (`bIT_DRAW_TYPE_GOLD_TREE000_DEAD`): item tables
/// (`mNT_get_itemTableNo`) count from here.
const TYPE_BASE: usize = 68;
/// `bIT_DRAW_TYPE_LEAF`: furniture on the ground is a leaf.
const TYPE_LEAF: usize = 78;
/// `draw_part_table_a` entry: part pointer, u16 flags, pad.
const PART_ENTRY: u32 = 8;
/// Game units per unit.
const UNIT: f32 = 40.0;

/// m_name_table.c::obj_hight_table_item0_nogrow raises both units of
/// bulletin (0007/000B), map (000C/000D) and music (000E/000F) boards.
/// The second unit deliberately has no display list, but is still solid.
pub(crate) fn board_cells(acres: &[[u16; 256]]) -> HashSet<(usize, usize, usize, usize)> {
    acres
        .iter()
        .enumerate()
        .flat_map(|(a, acre)| {
            acre.iter().enumerate().filter_map(move |(c, &item)| {
                matches!(item, 0x0007 | 0x000B..=0x000F).then_some((
                    a % ACRE_WIDTH,
                    a / ACRE_WIDTH,
                    c % 16,
                    c / 16,
                ))
            })
        })
        .collect()
}

/// Include save/load edits to the field grid in collision before movement.
fn sync_board_collision(items: Res<FieldItems>, mut town: ResMut<Town>) {
    if items.is_changed() {
        if let Some(retail) = &mut town.retail {
            retail.board_cells = board_cells(&items.acres);
        }
    }
}

/// `rnd_data_x` / `rnd_data_z` (`bg_item_common.c_inc`): the scatter of
/// parts with flag 2 (shells), by `[ut_x & 3][ut_z & 3]`.
const RND_X: [[f32; 4]; 4] = [
    [7.5, 0.0, -5.0, 5.0],
    [-5.0, -7.5, 2.5, -2.5],
    [7.5, 10.0, -2.5, 5.0],
    [-2.5, 5.0, 7.5, -2.5],
];
const RND_Z: [[f32; 4]; 4] = [
    [-5.0, 0.0, 5.0, 0.0],
    [-5.0, 2.5, 0.0, 5.0],
    [-2.5, 0.0, 7.5, 5.0],
    [7.5, -7.5, -2.5, -5.0],
];

/// The town's field items, acre by acre (playable acres, row by row).
#[derive(Resource)]
pub struct FieldItems {
    pub acres: Vec<[u16; 256]>,
    /// The town's native fruit (Save_Get(fruit)).
    pub fruit: u16,
    /// `Save_Get(dust_flag)`: five or more pieces of trash were seen, and the
    /// town stays unrated (`crate::field_rank`) until none is left.
    pub dust_flag: bool,
    /// The new town's grid: what the save's changes are measured from.
    base: Vec<[u16; 256]>,
    /// Shells waiting to wash up (`l_reserve_set_shell`).
    reserved_shells: i32,
    /// The ten-minute mark already counted (`set_flag`).
    shell_mark: bool,
    rng: Box<dyn FnMut(i32) -> i32 + Send + Sync>,
}

impl FieldItems {
    /// A new town's items for the layout: one plain tree in two acres of
    /// every row bears the native fruit.
    pub fn new(layout: Vec<[u16; 256]>, seed: u32) -> Self {
        let mut rng = seeded_rng(seed ^ 0xF1E1_D17E);
        let mut acres = layout;
        let fruit = logic::decide_fruit(&mut rng);
        logic::change_tree_to_fruit_tree(&mut acres, ACRE_WIDTH, fruit, &mut rng);
        Self {
            fruit,
            base: acres.clone(),
            acres,
            dust_flag: false,
            reserved_shells: SET_SHELL_NUM,
            shell_mark: false,
            rng: Box::new(rng),
        }
    }

    /// Cells that differ from the new town: [acre, cell, item].
    pub fn changes(&self) -> Vec<[u16; 3]> {
        let mut out = Vec::new();
        for (a, (now, base)) in self.acres.iter().zip(&self.base).enumerate() {
            for c in 0..256 {
                if now[c] != base[c] {
                    out.push([a as u16, c as u16, now[c]]);
                }
            }
        }
        out
    }

    /// The new town's grid with the save's changes on it.
    pub fn restore(&mut self, changes: &[[u16; 3]]) {
        self.acres = self.base.clone();
        for &[a, c, item] in changes {
            if let Some(acre) = self.acres.get_mut(a as usize) {
                if let Some(cell) = acre.get_mut(c as usize) {
                    *cell = item;
                }
            }
        }
    }

    /// The item on the unit under a scene position: (acre, cell, item).
    pub fn at(&self, pos: Vec3) -> Option<(usize, usize, u16)> {
        let (ax, az, cx, cz) = Town::cell_indices_of(pos)?;
        let (a, c) = (az * ACRE_WIDTH + ax, cz * 16 + cx);
        Some((a, c, self.acres.get(a)?[c]))
    }
}

/// Builds the grid once the disc town exists, then applies the save.
fn setup_field_items(
    mut commands: Commands,
    town: Res<Town>,
    loaded: Option<Res<crate::save::LoadedSave>>,
) {
    let Some(retail) = &town.retail else { return };
    let mut items = FieldItems::new(retail.items.clone(), town.plan.seed);
    if let Some(changes) = loaded
        .as_ref()
        .and_then(|l| l.0.as_ref())
        .and_then(|s| s.field_items.as_ref())
    {
        items.restore(changes);
    }
    items.dust_flag = loaded
        .as_ref()
        .and_then(|l| l.0.as_ref())
        .is_some_and(|s| s.dust_flag);
    commands.insert_resource(items);
}

/// `mFI_SetShell`: one more shell reserved every tenth minute; the
/// reserve washes up while the player is out in town, never in the acre
/// they stand in.
fn wash_up_shells(
    mut items: ResMut<FieldItems>,
    town: Res<Town>,
    house: Res<State<crate::house::HouseState>>,
    shop: Res<State<crate::shop::ShopLocation>>,
    player: Query<&Transform, With<Player>>,
) {
    if crate::clock::now().minute.is_multiple_of(10) {
        if !items.shell_mark {
            items.shell_mark = true;
            items.reserved_shells += 1;
        }
    } else {
        items.shell_mark = false;
    }
    let outside = *house.get() == crate::house::HouseState::Town
        && *shop.get() == crate::shop::ShopLocation::Town;
    let (Some(ground), Ok(player)) = (&town.ground, player.single()) else {
        return;
    };
    if !outside || items.reserved_shells <= 0 {
        return;
    }
    let here = Town::cell_indices_of(player.translation).map(|(ax, az, _, _)| (ax, az));
    // `l_sandy_beach_bx/bz`: the five beach acres of the bottom row.
    let az = ACRE_DEPTH - 1;
    let attrs: Vec<[u8; 256]> = (0..ACRE_WIDTH)
        .map(|ax| std::array::from_fn(|u| ground.unit_attribute(ax + 1, az + 1, u)))
        .collect();
    let set_num = items.reserved_shells;
    let FieldItems { acres, rng, .. } = &mut *items;
    let row = &mut acres[az * ACRE_WIDTH..(az + 1) * ACRE_WIDTH];
    let mut blocks: Vec<BeachBlock> = row
        .iter_mut()
        .zip(&attrs)
        .enumerate()
        .map(|(ax, (items, attrs))| BeachBlock {
            items,
            attrs,
            player_here: here == Some((ax, az)),
            island: false,
        })
        .collect();
    logic::set_shell_wave(&mut blocks, set_num, rng);
    items.reserved_shells = 0;
    let shells: Vec<Vec3> = (0..ACRE_WIDTH)
        .flat_map(|ax| (0..256).map(move |c| (ax, c)))
        .filter(|&(ax, c)| {
            let item = items.acres[az * ACRE_WIDTH + ax][c];
            (logic::ITM_SHELL0..=logic::ITM_SHELL7).contains(&item)
        })
        .map(|(ax, c)| Town::cell_to_world(ax, az, c % 16, c / 16))
        .collect();
    debug!(
        "{} shells on the beach, first at {:?}",
        shells.len(),
        shells.first()
    );
}

/// How `bg_item` draws an item: its `draw_part_table_a` type, sub index
/// and the unit spot tables (`fg_no2fg_type`).
struct DrawType {
    ty: usize,
    sub: i8,
    pos_x: [f32; 16],
    pos_z: [f32; 16],
}

/// A display list pointer in the REL.
fn rel_ptr(disc: &GameDisc, at: u32) -> Option<u32> {
    match disc.rom.pointer_at(RomSource::Rel, at) {
        Some(Target::File(RomSource::Rel, p)) => Some(p),
        _ => None,
    }
}

fn rel_bytes<const N: usize>(disc: &GameDisc, at: u32) -> Option<[u8; N]> {
    disc.rom
        .slice(RomSource::Rel, at, N as u32)?
        .try_into()
        .ok()
}

/// The `bg_item` copy of a table: several actors (cherry, winter, xmas)
/// carry statics of the same name; bg_item's follow its
/// `typeData_table_bgItem`.
fn bg_item_table<'d>(disc: &'d GameDisc, name: &str) -> Option<disc::Asset<'d>> {
    let anchor = disc.catalog.get("typeData_table_bgItem")?.offset;
    disc.catalog
        .all(name)
        .filter(|a| a.source == RomSource::Rel && a.offset > anchor)
        .min_by_key(|a| a.offset)
}

/// `bg_item_type_data_c` at `at`: s8 sub, s16 type, x and z tables.
fn read_type_data(disc: &GameDisc, at: u32, ty: Option<usize>) -> Option<DrawType> {
    let head = rel_bytes::<4>(disc, at)?;
    let table = |p: u32| -> Option<[f32; 16]> {
        let mut out = [0.0; 16];
        for (i, v) in out.iter_mut().enumerate() {
            *v = f32::from_be_bytes(rel_bytes::<4>(disc, p + i as u32 * 4)?);
        }
        Some(out)
    };
    Some(DrawType {
        ty: ty.unwrap_or(i16::from_be_bytes([head[2], head[3]]) as usize),
        sub: head[0] as i8,
        pos_x: table(rel_ptr(disc, at + 4)?)?,
        pos_z: table(rel_ptr(disc, at + 8)?)?,
    })
}

/// `fg_no2fg_type` (no sand holes: the grid has none yet).
fn draw_type(disc: &GameDisc, item: u16) -> Option<DrawType> {
    let (kind, cat, index) = (item >> 12, (item >> 8) & 0xF, item & 0xFF);
    match kind {
        // NAME_TYPE_ITEM0: cat 0 bg items, cat 8 trees and flowers.
        0 => {
            let (name, i) = match cat {
                0 => ("typeData_table_bgItem", item),
                8 => ("typeData_table_bgItem2", item - 0x0800),
                _ => return None,
            };
            let table = disc.catalog.get(name)?;
            let at = table.offset + i as u32 * 12;
            (at + 12 <= table.offset + table.size).then_some(())?;
            read_type_data(disc, at, None)
        }
        // NAME_TYPE_FTR0/FTR1: a leaf.
        1 | 3 => read_type_data(
            disc,
            bg_item_table(disc, "typeData_table_ftr")?.offset,
            Some(TYPE_LEAF),
        ),
        // NAME_TYPE_ITEM1: mNT_get_itemTableNo.
        2 => {
            let tables = disc.catalog.get("item1_tableNo$430")?;
            let cat_table = rel_ptr(disc, tables.offset + cat as u32 * 4)?;
            let no = rel_bytes::<1>(disc, cat_table + index as u32)?[0] as usize;
            read_type_data(
                disc,
                bg_item_table(disc, "typeData_table_item1")?.offset,
                Some(TYPE_BASE + no),
            )
        }
        _ => None,
    }
}

/// A part's draw lists, each as (material list, geometry list) with the
/// sub index applied (`bg_item_common_draw_loop_type1`), and the part's
/// flags.
fn part_lists(disc: &GameDisc, ty: usize, sub: i8) -> Option<(Vec<[u32; 2]>, u16)> {
    let table = bg_item_table(disc, "draw_part_table_a")?;
    let entry = table.offset + ty as u32 * PART_ENTRY;
    (entry + PART_ENTRY <= table.offset + table.size).then_some(())?;
    let flags = u16::from_be_bytes(rel_bytes::<2>(disc, entry + 4)?);
    let part = rel_ptr(disc, entry)?;
    let dl_table = rel_ptr(disc, part)?;
    let count = u32::from_be_bytes(rel_bytes::<4>(disc, part + 4)?);
    let list_table = rel_ptr(disc, part + 8)?;
    let mut lists = Vec::new();
    for i in 0..count {
        let list = rel_ptr(disc, list_table + i * 4)?;
        let [gfx, mat] = rel_bytes::<2>(disc, list + 4)?;
        let geometry = mat as i32 + sub as i32;
        lists.push([
            rel_ptr(disc, dl_table + gfx as u32 * 4)?,
            rel_ptr(disc, dl_table + geometry as u32 * 4)?,
        ]);
    }
    Some((lists, flags))
}

type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;

/// What's drawn for an item, by item number: model and scatter flag.
#[derive(Default)]
struct ModelCache(HashMap<u16, Option<(Model, bool, DrawType)>>);

/// Item models standing in the town: (acre, cell) -> (item, entity).
#[derive(Resource, Default)]
struct DrawnItems(HashMap<(usize, usize), (u16, Entity)>);

/// Marker on a drawn field item.
#[derive(Component)]
pub struct FieldItemModel;

/// Builds the model for an item. Trees and rocks are `disc_town`'s; a
/// fruit tree's own draw lists beyond the plain tree's are its fruit.
fn item_model(
    disc: &GameDisc,
    item: u16,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> Option<(Model, bool, DrawType)> {
    if (0x0063..=0x0067).contains(&item) {
        return None;
    }
    let draw = draw_type(disc, item)?;
    let (mut lists, flags) = part_lists(disc, draw.ty, draw.sub)?;
    if cell_for_item(item) == Some(CellKind::Tree) {
        let plain = draw_type(disc, logic::TREE)
            .and_then(|t| part_lists(disc, t.ty, t.sub))
            .map(|(l, _)| l)
            .unwrap_or_default();
        lists.retain(|l| !plain.contains(l));
    }
    if lists.is_empty() {
        return None;
    }
    let flat: Vec<u32> = lists.into_iter().flatten().collect();
    let model = build_model(disc, &flat, None, meshes, materials, images);
    Some((model, flags & 2 != 0, draw))
}

/// An item's ground model (as `bg_item` / `bg_police_item` draw it).
pub(crate) fn item_meshes(
    disc: &GameDisc,
    item: u16,
    assets: (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> Option<Model> {
    item_model(disc, item, assets).map(|(model, ..)| model)
}

/// Keeps the drawn items in step with the grid.
#[allow(clippy::too_many_arguments)]
fn draw_field_items(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    town: Res<Town>,
    items: Res<FieldItems>,
    mut drawn: ResMut<DrawnItems>,
    mut cache: Local<ModelCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (Some(disc), Some(ground)) = (disc, &town.ground) else {
        return;
    };
    if !items.is_changed() {
        return;
    }
    for (a, acre) in items.acres.iter().enumerate() {
        let (ax, az) = (a % ACRE_WIDTH, a / ACRE_WIDTH);
        for (c, &item) in acre.iter().enumerate() {
            let key = (a, c);
            if drawn.0.get(&key).map(|d| d.0) == Some(item) {
                continue;
            }
            if let Some((_, e)) = drawn.0.remove(&key) {
                commands.entity(e).despawn();
            }
            if item == EMPTY_NO {
                continue;
            }
            let entry = cache.0.entry(item).or_insert_with(|| {
                item_model(&disc, item, (&mut meshes, &mut materials, &mut images))
            });
            let Some((model, scatter, draw)) = entry else {
                continue;
            };
            let (ux, uz) = (c % 16, c / 16);
            let corner = Town::cell_to_world(ax, az, 0, 0) - Vec3::new(0.5, 0.0, 0.5) * CELL_SIZE;
            let (mut gx, mut gz) = (draw.pos_x[ux], draw.pos_z[uz]);
            if *scatter {
                gx += RND_X[ux & 3][uz & 3];
                gz += RND_Z[ux & 3][uz & 3];
            }
            let mut at = corner + Vec3::new(gx, 0.0, gz) / UNIT * CELL_SIZE;
            let centre = Town::cell_to_world(ax, az, ux, uz);
            at.y = ground.height(centre.x, centre.z).unwrap_or(0.0) + CELL_SIZE / UNIT;
            let root = commands
                .spawn((
                    FieldItemModel,
                    Name::new(format!("field item {item:04x}")),
                    Transform::from_translation(at),
                    Visibility::default(),
                ))
                .id();
            if matches!(item, 0x0007 | 0x000B..=0x000F) {
                commands
                    .entity(root)
                    .insert(crate::intro_camera::Occluder::new(65.0));
            }
            for (mesh, material) in model.iter() {
                commands.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    ChildOf(root),
                ));
            }
            drawn.0.insert(key, (item, root));
        }
    }
}

/// Whether the player can pick an item up off the ground: furniture
/// (`NAME_TYPE_FTR0/1`) and items (`NAME_TYPE_ITEM1`).
pub fn can_pick_up(item: u16) -> bool {
    matches!(item >> 12, 1..=3)
}

pub struct FieldItemsPlugin;

impl Plugin for FieldItemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DrawnItems>()
            .add_systems(
                Update,
                sync_board_collision
                    .before(crate::player::player_movement)
                    .run_if(in_state(AppState::InGame).and_then(resource_exists::<FieldItems>)),
            )
            .add_systems(
                OnEnter(AppState::InGame),
                setup_field_items
                    .after(crate::disc_town::build_disc_town)
                    .run_if(resource_exists::<Town>),
            )
            .add_systems(
                Update,
                (wash_up_shells, draw_field_items)
                    .chain()
                    .run_if(in_state(AppState::InGame).and_then(resource_exists::<FieldItems>)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Probe (needs the disc): how bg_item draws a few items.
    #[test]
    #[ignore]
    fn probe_bg_item_tables() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let disc = crate::game_disc::load(path.into()).unwrap();
        for item in [
            0x2800u16, 0x2514, 0x251B, 0x080C, 0x0804, 0x0840, 0x0008, 0x1000,
        ] {
            let Some(d) = draw_type(&disc, item) else {
                println!("{item:04x}: no draw type");
                continue;
            };
            let lists = part_lists(&disc, d.ty, d.sub);
            let names: Vec<String> = lists
                .iter()
                .flat_map(|(l, _)| l.iter().flatten())
                .map(|&o| {
                    disc.catalog
                        .containing(RomSource::Rel, o)
                        .map_or(format!("{o:x}"), |a| a.name.to_string())
                })
                .collect();
            println!(
                "{item:04x}: type {} sub {} flags {:?} x0 {} x15 {} lists {names:?}",
                d.ty,
                d.sub,
                lists.as_ref().map(|l| l.1),
                d.pos_x[0],
                d.pos_x[15]
            );
        }
    }
}
