//! The two inventory collection pages (`m_inventory_ovl.c`).
//!
//! Discovery belongs to the player's furniture-collected bits and is recorded
//! by a catch, independently of carrying an item or donating it to the museum.
use crate::collection::{furniture_idx, ItemCollection};
use crate::furniture::{fish_to_furniture, insect_to_furniture};

pub const PAGE_INVENTORY: usize = 0;
pub const PAGE_FISH: usize = 1;
pub const PAGE_INSECTS: usize = 2;
pub const COUNT: usize = 40;

/// `mIV_fish_collect_list`, species IDs rather than display-slot IDs.
pub const FISH_ORDER: [u8; COUNT] = [
    0, 1, 2, 3, 11, 12, 4, 9, 13, 14, 15, 8, 5, 6, 7, 10, 26, 27, 16, 17, 18, 20, 19, 21, 22, 23,
    30, 29, 28, 24, 25, 31, 32, 33, 34, 35, 36, 37, 38, 39,
];
/// `mIV_insect_collect_list`.
pub const INSECT_ORDER: [u8; COUNT] = [
    0, 1, 2, 3, 7, 4, 5, 6, 10, 9, 11, 12, 15, 16, 18, 17, 24, 25, 26, 13, 14, 28, 8, 27, 19, 23,
    22, 20, 21, 29, 30, 31, 34, 38, 36, 39, 33, 37, 32, 35,
];

/// A display slot's pocket item; invalid pages and indices have no item.
pub fn item(page: usize, index: usize) -> Option<u16> {
    let (base, order) = match page {
        PAGE_FISH => (0x2300, &FISH_ORDER),
        PAGE_INSECTS => (0x2D00, &INSECT_ORDER),
        _ => return None,
    };
    order.get(index).map(|&species| base + u16::from(species))
}

fn bit(item: u16) -> Option<usize> {
    let furniture = fish_to_furniture(item).or_else(|| insect_to_furniture(item))?;
    furniture_idx(furniture)
}

/// `mIV_set_collect_itemNo`: undiscovered species are empty slots.
pub fn has(collection: &ItemCollection, item: u16) -> bool {
    bit(item).is_some_and(|bit| {
        collection
            .furniture
            .get(bit >> 5)
            .is_some_and(|word| word & (1 << (bit & 31)) != 0)
    })
}

/// Record an actual catch. Returns true only for a newly discovered species.
/// Trash, the whale, spirits and unrelated items do not occupy collection slots.
pub fn record_catch(collection: &mut ItemCollection, item: u16) -> bool {
    let Some(bit) = bit(item) else { return false };
    let Some(word) = collection.furniture.get_mut(bit >> 5) else {
        return false;
    };
    let mask = 1 << (bit & 31);
    let discovered = *word & mask == 0;
    *word |= mask;
    discovered
}
