//! Item IDs for the pockets (`rustimal_logic::inventory` stores `u16` IDs).
//!
//! The translated code has no full item table yet (only scattered IDs such
//! as fossils and money bags), so the slice uses its own ID ranges here.
//! Species IDs come straight from the decomp-verified `FishType` /
//! `InsectType` discriminants, so a caught Sea Bass and Carp stay distinct.
//!
//! | Range | Items |
//! |---|---|
//! | `0x1000 + n` | furniture, `n` = index into [`FURNITURE_CATALOG`] |
//! | `0x2300 + n` | fish, `n` = `FishType as u8` |
//! | `0x2400 + n` | bugs, `n` = `InsectType as u8` |
//! | `0x2800` | fruit |
//! | `0x2900` | quest parcel |
//! | `0x2A00 + c` | quest reward goods, `c` = `quest::reward` category |

use rustimal_logic::inventory::EMPTY_NO;
use rustimal_logic::species::{FishType, InsectType};

pub type ItemId = u16;

const FURNITURE_BASE: ItemId = 0x1000;
const FISH_BASE: ItemId = 0x2300;
const BUG_BASE: ItemId = 0x2400;
pub const FRUIT: ItemId = 0x2800;
pub const PARCEL: ItemId = 0x2900;
const REWARD_BASE: ItemId = 0x2A00;

/// Reward goods for the non-furniture `rustimal_logic::quest::reward`
/// categories (placeholder originals; the slice has no clothing/carpet
/// systems yet, so these are pocket items).
pub fn reward_good(category: u8) -> Option<ItemId> {
    use rustimal_logic::quest::reward;
    match category {
        reward::STATIONERY
        | reward::CLOTH
        | reward::CARPET
        | reward::WALLPAPER
        | reward::WORN_CLOTH => Some(REWARD_BASE + category as ItemId),
        _ => None,
    }
}

fn reward_good_name(id: ItemId) -> Option<&'static str> {
    use rustimal_logic::quest::reward;
    match id.checked_sub(REWARD_BASE)? as u8 {
        reward::STATIONERY => Some("Letter Paper"),
        reward::CLOTH => Some("Striped Shirt"),
        reward::CARPET => Some("Plaid Rug"),
        reward::WALLPAPER => Some("Floral Wallpaper"),
        reward::WORN_CLOTH => Some("Worn Shirt"),
        _ => None,
    }
}

/// Sell prices (bells per item) — sensible originals; the ported modules
/// carry no price table.
const FRUIT_SELL: u32 = 100;
const FISH_SELL: u32 = 250;
const BUG_SELL: u32 = 150;

/// Furniture the shop can sell, in ID order. Names are placeholders (the
/// decomp only carries numeric furniture IDs).
pub const FURNITURE_CATALOG: [&str; 6] = [
    "Wooden Chair",
    "Potted Plant",
    "Wall Clock",
    "Modern Lamp",
    "Bookshelf",
    "Cozy Bed",
];

pub fn fish(species: FishType) -> ItemId {
    FISH_BASE + species as ItemId
}

pub fn bug(species: InsectType) -> ItemId {
    BUG_BASE + species as ItemId
}

/// Furniture ID for a catalog name, if it's in the catalog.
pub fn furniture(name: &str) -> Option<ItemId> {
    FURNITURE_CATALOG
        .iter()
        .position(|n| *n == name)
        .map(|i| FURNITURE_BASE + i as ItemId)
}

fn fish_species(id: ItemId) -> Option<FishType> {
    id.checked_sub(FISH_BASE)
        .filter(|n| *n < 0x100)
        .and_then(|n| FishType::from_u8(n as u8))
}

fn bug_species(id: ItemId) -> Option<InsectType> {
    id.checked_sub(BUG_BASE)
        .filter(|n| *n < 0x100)
        .and_then(|n| InsectType::from_u8(n as u8))
}

fn furniture_name(id: ItemId) -> Option<&'static str> {
    id.checked_sub(FURNITURE_BASE)
        .and_then(|n| FURNITURE_CATALOG.get(n as usize).copied())
}

pub fn is_fish(id: ItemId) -> bool {
    fish_species(id).is_some()
}

pub fn is_bug(id: ItemId) -> bool {
    bug_species(id).is_some()
}

pub fn is_furniture(id: ItemId) -> bool {
    furniture_name(id).is_some()
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

/// Display name for an item (empty slot: "").
pub fn name(id: ItemId) -> String {
    if id == EMPTY_NO {
        return String::new();
    }
    if id == FRUIT {
        return "Fruit".to_string();
    }
    if id == PARCEL {
        return "Parcel".to_string();
    }
    if let Some(s) = fish_species(id) {
        return pretty_name(&format!("{s:?}"));
    }
    if let Some(s) = bug_species(id) {
        return pretty_name(&format!("{s:?}"));
    }
    if let Some(n) = furniture_name(id) {
        return n.to_string();
    }
    if let Some(n) = reward_good_name(id) {
        return n.to_string();
    }
    format!("Item {id:#06x}")
}

/// What the shop pays for one, or `None` if it won't buy it.
pub fn sell_price(id: ItemId) -> Option<u32> {
    if id == FRUIT {
        Some(FRUIT_SELL)
    } else if is_fish(id) {
        Some(FISH_SELL)
    } else if is_bug(id) {
        Some(BUG_SELL)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_to_names() {
        assert_eq!(name(fish(FishType::Carp)), "Carp");
        assert_eq!(name(bug(InsectType::CommonButterfly)), "Common Butterfly");
        assert_eq!(name(furniture("Bookshelf").unwrap()), "Bookshelf");
        assert_eq!(name(FRUIT), "Fruit");
        assert_eq!(name(EMPTY_NO), "");
        let rug = reward_good(rustimal_logic::quest::reward::CARPET).unwrap();
        assert_eq!(name(rug), "Plaid Rug");
        assert_eq!(reward_good(rustimal_logic::quest::reward::MONEY), None);
    }

    #[test]
    fn categories_do_not_overlap() {
        let f = fish(FishType::Carp);
        let b = bug(InsectType::CommonButterfly);
        assert!(is_fish(f) && !is_bug(f));
        assert!(is_bug(b) && !is_fish(b));
        assert!(!is_fish(FRUIT) && !is_bug(FRUIT));
        assert_eq!(furniture("Not A Thing"), None);
    }

    #[test]
    fn shop_buys_catches_and_fruit_only() {
        assert_eq!(sell_price(FRUIT), Some(100));
        assert_eq!(sell_price(fish(FishType::Carp)), Some(250));
        assert_eq!(sell_price(PARCEL), None);
        assert_eq!(sell_price(furniture("Bookshelf").unwrap()), None);
    }
}
