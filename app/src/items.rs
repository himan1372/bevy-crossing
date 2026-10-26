//! Item numbers for the pockets: the game's own `mActor_name_t` values
//! (`m_name_table.h`), with names and prices read from the disc.
//!
//! | Range | Items |
//! |---|---|
//! | `0x1000`-`0x1FFF`, `0x3000`-`0x33FF` | furniture (index << 2 \| rotation) |
//! | `0x2000` | stationery | `0x2200` | tools | `0x2300` | fish |
//! | `0x2400` | clothing | `0x2500` | errand goods, shells... |
//! | `0x2600` | carpets | `0x2700` | wallpaper | `0x2800` | fruit |
//! | `0x2900` | saplings, seeds | `0x2D00` | insects |
//!
//! Fish and insect numbers follow the decomp-verified `FishType` /
//! `InsectType` order, which is the item order.
//!
//! Names come from `itemName_*` / `ftrName_table` (`mIN_copy_name_str`,
//! 16 characters each) and prices from the `*_price_table`s
//! (`mSP_ItemNo2ItemPrice`); Nook buys at a quarter (`SELL_BUY_RATIO`).

use std::sync::OnceLock;

use bevy::log::warn;
use disc::RomSource;
use rustimal_logic::inventory::EMPTY_NO;
use rustimal_logic::species::{FishType, InsectType};

use crate::game_disc::GameDisc;

pub type ItemId = u16;

const FISH_START: ItemId = 0x2300;
const INSECT_START: ItemId = 0x2D00;
/// `ITM_FOOD_APPLE`.
pub const FRUIT: ItemId = 0x2800;
/// What an errand has you deliver: `ITM_QST_VIDEOTAPE`, one of the
/// errand goods with a name of its own on the disc.
pub const PARCEL: ItemId = 0x2503;
const PAPER00: ItemId = 0x2000;
const CLOTH000: ItemId = 0x2400;
const CARPET00: ItemId = 0x2600;
const WALL00: ItemId = 0x2700;
const SHELL0: ItemId = 0x2514;
const SIGNBOARD: ItemId = 0x251E;
/// `mSP_SIGNBOARD_PRICE`.
const SIGNBOARD_PRICE: u32 = 500;
/// Nook pays a quarter of an item's price (`SELL_BUY_RATIO`).
const SELL_BUY_RATIO: u32 = 4;
/// `mIN_ITEM_NAME_LEN`.
const NAME_LEN: usize = 16;
/// Stationery comes in 64 designs, stacked up to four sheets
/// (`PAPER_UNIQUE_NUM`).
const PAPER_UNIQUE: ItemId = 64;

/// Reward goods for the non-furniture `rustimal_logic::quest::reward`
/// categories: the first item of each kind.
pub fn reward_good(category: u8) -> Option<ItemId> {
    use rustimal_logic::quest::reward;
    match category {
        reward::STATIONERY => Some(PAPER00),
        reward::CLOTH => Some(CLOTH000),
        reward::CARPET => Some(CARPET00),
        reward::WALLPAPER => Some(WALL00),
        reward::WORN_CLOTH => Some(CLOTH000),
        _ => None,
    }
}

pub fn fish(species: FishType) -> ItemId {
    FISH_START + species as ItemId
}

pub fn bug(species: InsectType) -> ItemId {
    INSECT_START + species as ItemId
}

fn fish_species(id: ItemId) -> Option<FishType> {
    id.checked_sub(FISH_START)
        .filter(|n| *n < 0x100)
        .and_then(|n| FishType::from_u8(n as u8))
}

fn bug_species(id: ItemId) -> Option<InsectType> {
    id.checked_sub(INSECT_START)
        .filter(|n| *n < 0x100)
        .and_then(|n| InsectType::from_u8(n as u8))
}

pub fn is_fish(id: ItemId) -> bool {
    fish_species(id).is_some()
}

pub fn is_bug(id: ItemId) -> bool {
    bug_species(id).is_some()
}

/// `ITEM_IS_FTR`.
pub fn is_furniture(id: ItemId) -> bool {
    matches!(id >> 12, 1 | 3)
}

/// `ITEM_NAME_GET_CAT`: an item1's category (0 stationery .. 15 turnips).
fn category(id: ItemId) -> usize {
    usize::from((id >> 8) & 0xF)
}

/// Item names and prices from the disc.
struct ItemTable {
    /// Per item1 category, the names in item order.
    names: Vec<Vec<String>>,
    /// Furniture names by index (`ftrName_table`, then `ftrName2_table`
    /// from 0x400).
    furniture: Vec<String>,
    /// Per item1 category, its price table (`l_price_info`).
    prices: Vec<Vec<u16>>,
    furniture_prices: Vec<u16>,
}

static TABLE: OnceLock<ItemTable> = OnceLock::new();

/// `itemName_table` in `mIN_copy_name_str`, by category.
const NAME_TABLES: [&str; 16] = [
    "itemName_paper",
    "itemName_money",
    "itemName_tool",
    "itemName_fish",
    "itemName_cloth",
    "itemName_etc",
    "itemName_carpet",
    "itemName_wall",
    "itemName_fruit",
    "itemName_plant",
    "itemName_minidisk",
    "itemName_dummy",
    "itemName_ticket",
    "itemName_insect",
    "itemName_hukubukuro",
    "itemName_kabu",
];

/// `l_price_info`, by category (`None`: never sold or bought).
const PRICE_TABLES: [Option<&str>; 16] = [
    Some("binsen_price_table"),
    None,
    Some("tool_price_table"),
    Some("fish_price_table"),
    Some("cloth_price_table"),
    None,
    Some("carpet_price_table"),
    Some("wall_price_table"),
    Some("food_price_table"),
    Some("plant_price_table"),
    Some("md_price_table"),
    Some("diary_price_table"),
    None,
    Some("insect_price_table"),
    None,
    None,
];

/// `mSP_ItemNo2ItemPrice`'s shell prices.
const SHELL_PRICES: [u32; 8] = [160, 80, 600, 120, 240, 1800, 1400, 1000];

/// Reads the item names and prices once the disc is loaded.
pub fn install(disc: &GameDisc) {
    let rom = &disc.rom;
    let names_in = |sym: &str| -> Vec<String> {
        let Some(a) = disc.catalog.get(sym) else {
            return Vec::new();
        };
        rom.slice(RomSource::Rel, a.offset, a.size)
            .unwrap_or_default()
            .chunks(NAME_LEN)
            .map(|c| crate::messages::plain(c).trim_end().to_string())
            .collect()
    };
    // Price tables end at 0xFFFF (`mSP_CountPriceTableElement`).
    let prices_in = |sym: &str| -> Vec<u16> {
        let Some(a) = disc.catalog.get(sym) else {
            return Vec::new();
        };
        rom.slice(RomSource::Rel, a.offset, a.size)
            .unwrap_or_default()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&b| u16::from_be_bytes(b))
            .take_while(|&p| p != 0xFFFF)
            .collect()
    };
    let mut furniture = names_in("ftrName_table");
    furniture.truncate(0x400);
    furniture.extend(names_in("ftrName2_table"));
    let table = ItemTable {
        names: NAME_TABLES.iter().map(|s| names_in(s)).collect(),
        furniture,
        prices: PRICE_TABLES
            .iter()
            .map(|s| s.map(prices_in).unwrap_or_default())
            .collect(),
        furniture_prices: prices_in("ftr_price_table"),
    };
    if TABLE.set(table).is_err() {
        warn!("item names were already read");
    }
}

/// "SeaBass" -> "Sea Bass".
pub fn pretty_name(debug: &str) -> String {
    let mut out = String::new();
    for (i, c) in debug.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

/// Display name for an item (empty slot: ""), from the disc when loaded.
pub fn name(id: ItemId) -> String {
    if id == EMPTY_NO {
        return String::new();
    }
    if let Some(n) = TABLE.get().and_then(|t| disc_name(t, id)) {
        return n;
    }
    if let Some(s) = fish_species(id) {
        return pretty_name(&format!("{s:?}"));
    }
    if let Some(s) = bug_species(id) {
        return pretty_name(&format!("{s:?}"));
    }
    match id {
        FRUIT => "apple".to_string(),
        PARCEL => "videotape".to_string(),
        _ => format!("item {id:#06x}"),
    }
}

fn disc_name(t: &ItemTable, id: ItemId) -> Option<String> {
    let n = if is_furniture(id) {
        let index = crate::furniture::furniture_index(id)? as usize;
        t.furniture.get(index)
    } else {
        t.names.get(category(id))?.get(usize::from(id & 0xFF))
    }?;
    (!n.is_empty()).then(|| n.clone())
}

/// What an item costs at Nook's (`mSP_ItemNo2ItemPrice`); 0 if he
/// doesn't trade it, `None` without the disc.
pub fn price(id: ItemId) -> Option<u32> {
    let t = TABLE.get()?;
    if id == crate::goods::GRAB_BAG {
        return Some(u32::from(crate::clock::now().year));
    }
    if (SHELL0..SHELL0 + 8).contains(&id) {
        return Some(SHELL_PRICES[usize::from(id - SHELL0)]);
    }
    if id == SIGNBOARD {
        return Some(SIGNBOARD_PRICE);
    }
    let price = if is_furniture(id) {
        let index = crate::furniture::furniture_index(id)? as usize;
        t.furniture_prices.get(index).copied()
    } else if id >> 12 == 2 {
        let table = t.prices.get(category(id))?;
        if category(id) == 0 {
            // Stationery: a stack of n sheets costs n times one.
            let sheet = (id - PAPER00) % PAPER_UNIQUE;
            let stack = u32::from((id - PAPER00) / PAPER_UNIQUE) + 1;
            return Some(
                table
                    .get(usize::from(sheet))
                    .map_or(0, |&p| u32::from(p) * stack),
            );
        }
        table.get(usize::from(id & 0xFF)).copied()
    } else {
        None
    };
    Some(price.map_or(0, u32::from))
}

/// What Nook pays for one, or `None` if he won't buy it.
pub fn sell_price(id: ItemId) -> Option<u32> {
    let pays = match price(id) {
        Some(p) => p / SELL_BUY_RATIO,
        // No disc: catches and fruit at stand-in prices.
        None if id == FRUIT => 100,
        None if is_fish(id) => 250,
        None if is_bug(id) => 150,
        None => 0,
    };
    (pays > 0).then_some(pays)
}

/// Maps a pocket item from saves before real item numbers (version 1:
/// bugs at 0x2400, the parcel at 0x2900, reward goods at 0x2A00, and six
/// stand-in furniture pieces) to its item number now.
pub fn migrate_v1(id: ItemId) -> ItemId {
    /// Wooden Chair, Potted Plant, Wall Clock, Modern Lamp, Bookshelf,
    /// Cozy Bed -> the nearest real pieces.
    /// classic chair, snake plant, cabin clock, modern lamp, blue
    /// bookcase, modern bed.
    const FURNITURE: [ItemId; 6] = [0x10F8, 0x13AC, 0x1304, 0x1F50, 0x1368, 0x1340];
    match id {
        0x1000..=0x1005 => FURNITURE[usize::from(id - 0x1000)],
        0x2400..=0x24FF => INSECT_START + (id & 0xFF),
        0x2900 => PARCEL,
        0x2A00..=0x2AFF => reward_good((id & 0xFF) as u8).unwrap_or(EMPTY_NO),
        _ => id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Probe (needs the disc): names and prices.
    #[test]
    #[ignore]
    fn probe_item_names() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        install(&crate::game_disc::load(path.into()).unwrap());
        for id in [
            0x2300, 0x2D00, 0x2800, 0x2503, 0x2000, 0x2400, 0x2600, 0x2700, 0x2202, 0x2900, 0x2902,
        ] {
            println!(
                "{id:#06x} {} {:?}/{:?}",
                name(id),
                price(id),
                sell_price(id)
            );
        }
        let t = TABLE.get().unwrap();
        for word in ["chair", "plant", "clock", "lamp", "book", "bed"] {
            let hits: Vec<String> = t
                .furniture
                .iter()
                .enumerate()
                .filter(|(_, n)| n.contains(word))
                .take(8)
                .map(|(i, n)| {
                    format!(
                        "{:#06x} {n} {}",
                        0x1000 + i * 4,
                        t.furniture_prices.get(i).copied().unwrap_or(0)
                    )
                })
                .collect();
            println!("{word}: {hits:?}");
        }
    }

    #[test]
    fn real_item_numbers() {
        assert_eq!(fish(FishType::CrucianCarp), 0x2300);
        assert_eq!(bug(InsectType::CommonButterfly), 0x2D00);
        assert!(is_furniture(0x1004) && is_furniture(0x3000));
        assert!(!is_furniture(FRUIT));
        assert_eq!(category(0x2D05), 13);
    }

    #[test]
    fn names_without_the_disc() {
        assert_eq!(name(fish(FishType::Carp)), "Carp");
        assert_eq!(name(bug(InsectType::CommonButterfly)), "Common Butterfly");
        assert_eq!(name(EMPTY_NO), "");
    }

    #[test]
    fn old_saves_move_to_real_numbers() {
        assert_eq!(migrate_v1(0x2401), bug(InsectType::YellowButterfly));
        assert_eq!(migrate_v1(0x2900), PARCEL);
        assert_eq!(migrate_v1(0x2300), 0x2300);
        assert!(is_furniture(migrate_v1(0x1004)));
    }
}
