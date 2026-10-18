//! Nook's goods on display (`ac_shop_design` / `ac_shop_goods`).
//!
//! The shop's layout (`FG_TYPE_ROM_SHOP1` in `fgdata.bin`) marks a slot
//! for each kind of goods (`RSV_SHOP_*`, 0xFE00-0xFE0C); each day's goods
//! fill the slots of their kind in order (`aSD_MakeGoodsFg`). Furniture,
//! shirts (on a mannequin, `ac_shop_manekin`) and umbrellas (in a stand,
//! `ac_shop_umbrella`) stand on the floor; the rest lie on the tables,
//! drawn with `l_shop_goods_dsp_table`'s models. A sold slot shows its
//! sold-out sign (`RSV_SHOP_SOLD_*`), as does the Halloween slot on
//! ordinary days (`aSD_MakePlantFg_normal`).

use bevy::prelude::*;
use disc::{RomSource, Target};

use crate::disc_gfx::{run_display_lists, Segments};
use crate::game_disc::GameDisc;
use crate::items::ItemId;
use crate::n64_material::N64Material;
use crate::shop::ShopData;

/// `FG_TYPE_ROM_SHOP1`: Nook's Cranny's layout.
const SHOP1_FG: u16 = 0x22;
/// The shop room's floor centre in model cells (see `interior`).
const SHOP_CENTRE: Vec3 = Vec3::new(4.0, 1.0, 4.0);
/// Model units per room cell (`aSI` draws the room at 0.0625; 40 game
/// units per cell).
const CELL: f32 = 40.0 / 0.0625;
/// Goods are drawn at 0.01 (`Shop_Goods_Actor_single_draw`).
const GOODS_SCALE: f32 = 0.01 / 40.0;
/// One `aSG_data_c`: item range, four pairs of lists, flags.
const DSP_ENTRY: u32 = 40;

const RSV_PAPER: u16 = 0xFE00;
const RSV_CLOTH: u16 = 0xFE01;
const RSV_FTR: u16 = 0xFE02;
const RSV_CARPET: u16 = 0xFE03;
const RSV_WALL: u16 = 0xFE04;
const RSV_HALLOWEEN: u16 = 0xFE05;
const RSV_TOOL: u16 = 0xFE06;
const RSV_PLANT: u16 = 0xFE08;
const RSV_UMBRELLA: u16 = 0xFE0A;
const RSV_PAINT: u16 = 0xFE0B;
const RSV_SIGNBOARD: u16 = 0xFE0C;

/// `aSD_ItemName2ReservePointName` (ordinary days).
fn reserve_point(item: ItemId) -> Option<u16> {
    Some(match item {
        0x251E => RSV_SIGNBOARD,
        i if crate::items::is_furniture(i) => RSV_FTR,
        0x2000..=0x20FF | 0x2B00..=0x2B0F => RSV_PAPER,
        0x2600..=0x26FF => RSV_CARPET,
        0x2700..=0x27FF => RSV_WALL,
        0x2400..=0x24FF => RSV_CLOTH,
        0x2204..=0x2223 => RSV_UMBRELLA,
        0x222D..=0x2238 => RSV_PAINT,
        0x2200..=0x22FF => RSV_TOOL,
        // aSD_ItemName2ReservePointName: ordinary-day saplings occupy
        // the Halloween reserve; flower bags use the two plant slots.
        0x2900..=0x2901 => RSV_HALLOWEEN,
        0x2902..=0x290A => RSV_PLANT,
        _ => return None,
    })
}

/// `RSV_SHOP_SOLD_*` for a slot kind.
fn sold_sign(reserve: u16) -> u16 {
    match reserve {
        RSV_PAPER => 0xFE0E,
        RSV_CLOTH => 0xFE0F,
        RSV_FTR => 0xFE10,
        RSV_CARPET => 0xFE11,
        RSV_WALL => 0xFE12,
        RSV_PLANT | RSV_HALLOWEEN => 0xFE13,
        RSV_TOOL => 0xFE14,
        RSV_UMBRELLA => 0xFE16,
        RSV_PAINT => 0xFE17,
        _ => 0xFE18,
    }
}

/// What's laid out, so it's rebuilt only when the stock changes.
#[derive(Resource, Default)]
struct GoodsOnDisplay {
    shown: Option<Vec<(ItemId, bool)>>,
    entities: Vec<Entity>,
}

/// Stock identity survives duplicate item IDs and sold-out signs.
#[derive(Component)]
pub(crate) struct ShopDisplay {
    pub stock_index: usize,
    pub position: Vec3,
}

/// The `l_shop_goods_dsp_table` lists (mode 0) for an item or sign:
/// (opaque, translucent).
fn goods_lists(disc: &GameDisc, item: u16) -> Option<(Vec<u32>, Vec<u32>)> {
    let table = disc.catalog.get("l_shop_goods_dsp_table")?;
    let rom = &disc.rom;
    let rel = |o: u32| match rom.pointer_at(RomSource::Rel, o) {
        Some(Target::File(RomSource::Rel, p)) => Some(p),
        _ => None,
    };
    (0..table.size / DSP_ENTRY).find_map(|i| {
        let e = table.offset + i * DSP_ENTRY;
        let b = rom.slice(RomSource::Rel, e, 4)?;
        let (start, end) = (
            u16::from_be_bytes([b[0], b[1]]),
            u16::from_be_bytes([b[2], b[3]]),
        );
        (start..=end).contains(&item).then(|| {
            (
                [4, 12].iter().filter_map(|&o| rel(e + o)).collect(),
                [20, 28].iter().filter_map(|&o| rel(e + o)).collect(),
            )
        })
    })
}

/// Whether the runtime goods table supplies any display lists for this item.
pub(crate) fn has_goods_model(disc: &GameDisc, item: u16) -> bool {
    goods_lists(disc, item)
        .is_some_and(|(opaque, translucent)| !opaque.is_empty() || !translucent.is_empty())
}

/// Lays the stock out whenever it changes.
#[allow(clippy::too_many_arguments)]
fn show_goods(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    shop: Res<ShopData>,
    mut display: ResMut<GoodsOnDisplay>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(disc) = disc else { return };
    let key: Vec<(ItemId, bool)> = shop.stock.iter().map(|s| (s.item, s.sold)).collect();
    if display.shown.as_ref() == Some(&key) {
        return;
    }
    for e in display.entities.drain(..) {
        commands.entity(e).despawn();
    }
    display.shown = Some(key);
    let Some(layout) = rustimal_logic::parse_fg_records(&disc.fgdata)
        .ok()
        .and_then(|r| r.into_iter().find(|r| r.fg_id == SHOP1_FG))
    else {
        return;
    };
    let mut slots: Vec<(u16, (i32, i32), Option<(usize, ItemId, bool)>)> = layout
        .items
        .iter()
        .enumerate()
        .filter(|(_, &m)| (RSV_PAPER..=RSV_SIGNBOARD).contains(&m))
        .map(|(i, &m)| (m, ((i % 16) as i32, (i / 16) as i32), None))
        .collect();
    for (index, s) in shop.stock.iter().enumerate() {
        let Some(reserve) = reserve_point(s.item) else {
            continue;
        };
        if let Some(slot) = slots.iter_mut().find(|t| t.0 == reserve && t.2.is_none()) {
            slot.2 = Some((index, s.item, s.sold));
        }
    }

    // Table tops: the highest surface of the room over each slot.
    let room: Vec<u32> = ["rom_shop1f_model", "rom_shop1w_model"]
        .iter()
        .filter_map(|n| disc.catalog.get(n).map(|a| a.offset))
        .collect();
    let surface = run_display_lists(&disc.rom, &room);
    let corner = crate::shop::SHOP_ORIGIN - SHOP_CENTRE.with_y(0.0);
    let floor = SHOP_CENTRE.y;
    for (reserve, ut, goods) in slots {
        let centre = Vec2::new(ut.0 as f32 + 0.5, ut.1 as f32 + 0.5);
        let top = surface
            .height_at(centre.x * CELL, centre.y * CELL)
            .map_or(floor, |y| y / CELL);
        // A table is under half a cell high; anything taller is a wall.
        let lift = if top > floor && top < floor + 1.0 {
            top - floor
        } else {
            0.0
        };
        let shown = match goods {
            Some((_, item, false)) => item,
            Some((_, _, true)) | None if reserve == RSV_HALLOWEEN => sold_sign(reserve),
            Some((_, _, true)) => sold_sign(reserve),
            None => continue,
        };
        let assets = (&mut *meshes, &mut *materials, &mut *images);
        if let Some(e) = spawn_goods(&mut commands, &disc, shown, ut, corner, lift, assets) {
            if let Some((stock_index, ..)) = goods {
                commands.entity(e).insert(ShopDisplay {
                    stock_index,
                    position: corner + Vec3::new(centre.x, 0.0, centre.y),
                });
            }
            display.entities.push(e);
        }
    }
}

/// One good (or sold-out sign) standing on unit `ut`.
pub(crate) fn spawn_goods(
    commands: &mut Commands,
    disc: &GameDisc,
    item: ItemId,
    ut: (i32, i32),
    corner: Vec3,
    lift: f32,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> Option<Entity> {
    // Floor goods drawn as furniture: the piece itself, a mannequin in
    // the shirt (`FTR_CLOTH_START`), an umbrella in its stand.
    let as_furniture = match item {
        i if crate::items::is_furniture(i) => Some(i),
        0x2400..=0x24FF => Some(0x1000 + (0x1EB + (item - 0x2400)) * 4),
        0x2204..=0x2223 => Some(0x1000 + (0x342 + (item - 0x2204)) * 4),
        _ => None,
    };
    if let Some(ftr) = as_furniture {
        return crate::furniture::spawn_furniture(
            commands, disc, ftr, ut, corner, lift, meshes, materials, images,
        );
    }
    let (opaque, translucent) = goods_lists(disc, item)?;
    let parent = commands
        .spawn((
            Name::new(format!("goods {}", crate::items::name(item))),
            Transform::from_translation(
                corner + Vec3::new(ut.0 as f32 + 0.5, lift, ut.1 as f32 + 0.5),
            )
            .with_scale(Vec3::splat(GOODS_SCALE)),
            Visibility::default(),
        ))
        .id();
    for (lists, blend) in [(opaque, false), (translucent, true)] {
        if lists.is_empty() {
            continue;
        }
        let sink = run_display_lists(&disc.rom, &lists);
        crate::interior::spawn_sink(
            commands,
            disc,
            parent,
            &sink,
            None,
            &Segments::default(),
            blend,
            meshes,
            materials,
            images,
        );
    }
    Some(parent)
}

pub struct ShopDisplayPlugin;

impl Plugin for ShopDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GoodsOnDisplay>()
            .add_systems(Update, show_goods);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goods_find_their_slots() {
        assert_eq!(reserve_point(0x1004), Some(RSV_FTR));
        assert_eq!(reserve_point(0x2202), Some(RSV_TOOL));
        assert_eq!(reserve_point(0x2210), Some(RSV_UMBRELLA));
        assert_eq!(reserve_point(0x2905), Some(RSV_PLANT));
        assert_eq!(reserve_point(0x2B03), Some(RSV_PAPER));
        assert_eq!(reserve_point(0x2800), None);
        assert_eq!(sold_sign(RSV_TOOL), 0xFE14);
    }
}
