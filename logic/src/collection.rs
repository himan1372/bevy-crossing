//! What the player has ever held (`Private_c` collected bitfields), which
//! the catalog lists and the lottery lineup checks.
//!
//! mPr_SetItemCollectBit (`m_private.c`): an item is first turned into its
//! room furniture where it has one (shirts, umbrellas, balloons:
//! mRmTp_Item1ItemNo2FtrItemNo_AtPlayerRoom); fish and insects are skipped;
//! furniture sets its furniture index bit, and stationery, carpets,
//! wallpaper and music their own fields. The bit is set whenever an item
//! lands in the pockets in normal condition (mPr_SetPossessionItem), and
//! when a present is opened.

use crate::furniture::{
    balloon_to_furniture, clothing_to_furniture, fish_to_furniture, insect_to_furniture,
    umbrella_to_furniture, FISH_FIRST, INSECT_FIRST,
};

/// FTR_NUM.
pub const FTR_NUM: usize = 1266;
const PAPER_START: u16 = 0x2000;
const CARPET_START: u16 = 0x2600;
const WALL_START: u16 = 0x2700;
const MUSIC_START: u16 = 0x2A00;

/// The five collected bitfields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemCollection {
    pub furniture: [u32; 43],
    pub wall: [u32; 3],
    pub carpet: [u32; 3],
    pub paper: [u32; 2],
    pub music: [u32; 2],
}

impl Default for ItemCollection {
    fn default() -> Self {
        Self {
            furniture: [0; 43],
            wall: [0; 3],
            carpet: [0; 3],
            paper: [0; 2],
            music: [0; 2],
        }
    }
}

/// Which bitfield an item lands in, and its bit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectBit {
    Furniture(usize),
    Wall(usize),
    Carpet(usize),
    Paper(usize),
    Music(usize),
}

/// mRmTp_FtrItemNo2FtrIdx: furniture item to its index.
pub fn furniture_idx(item: u16) -> Option<usize> {
    match item >> 12 {
        1 => Some(usize::from(item - 0x1000) >> 2),
        3 => Some(0x400 + (usize::from(item - 0x3000) >> 2)),
        _ => None,
    }
}

/// The bit mPr_SetItemCollectBit would set for `item`.
pub fn collect_bit(item: u16) -> Option<CollectBit> {
    if insect_to_furniture(item).is_some() || fish_to_furniture(item).is_some() {
        return None;
    }
    let ftr = clothing_to_furniture(item)
        .or_else(|| umbrella_to_furniture(item))
        .or_else(|| balloon_to_furniture(item))
        .unwrap_or(item);
    if let Some(idx) = furniture_idx(ftr) {
        // FTR_INSECT_START .. FTR_FISH_END: the critters as furniture.
        let critters = usize::from(INSECT_FIRST)..usize::from(FISH_FIRST) + 40;
        return (idx < FTR_NUM && !critters.contains(&idx)).then_some(CollectBit::Furniture(idx));
    }
    if ftr >> 12 != 2 {
        return None;
    }
    match (ftr >> 8) & 0xF {
        0 => Some(CollectBit::Paper(usize::from(ftr - PAPER_START) % 64)),
        6 => Some(CollectBit::Carpet(usize::from(ftr - CARPET_START))),
        7 => Some(CollectBit::Wall(usize::from(ftr - WALL_START))),
        0xA => Some(CollectBit::Music(usize::from(ftr - MUSIC_START))),
        _ => None,
    }
}

impl ItemCollection {
    fn field(&mut self, bit: CollectBit) -> (&mut [u32], usize) {
        match bit {
            CollectBit::Furniture(i) => (&mut self.furniture, i),
            CollectBit::Wall(i) => (&mut self.wall, i),
            CollectBit::Carpet(i) => (&mut self.carpet, i),
            CollectBit::Paper(i) => (&mut self.paper, i),
            CollectBit::Music(i) => (&mut self.music, i),
        }
    }

    /// mPr_SetItemCollectBit. Returns true when the bit is new.
    pub fn set(&mut self, item: u16) -> bool {
        let Some(bit) = collect_bit(item) else {
            return false;
        };
        let (field, i) = self.field(bit);
        let Some(word) = field.get_mut(i >> 5) else {
            return false;
        };
        let mask = 1 << (i & 31);
        let new = *word & mask == 0;
        *word |= mask;
        new
    }

    /// Whether the item has been held.
    pub fn has(&self, item: u16) -> bool {
        let Some(bit) = collect_bit(item) else {
            return false;
        };
        let (field, i): (&[u32], usize) = match bit {
            CollectBit::Furniture(i) => (&self.furniture, i),
            CollectBit::Wall(i) => (&self.wall, i),
            CollectBit::Carpet(i) => (&self.carpet, i),
            CollectBit::Paper(i) => (&self.paper, i),
            CollectBit::Music(i) => (&self.music, i),
        };
        field.get(i >> 5).is_some_and(|w| w & (1 << (i & 31)) != 0)
    }
}
