//! Nook's goods lists and the random-goods roll (`m_shop.c`).
//!
//! Each kind of goods (furniture, stationery, clothing, carpets,
//! wallpaper) has 23 lists on the disc (`mSP_ftr_list` ...): A, B and C,
//! then event, lottery, Halloween and other special lists. Which of A, B
//! and C is common, uncommon or rare is the shop's per-kind priority
//! (`mSP_DecideUniqueCommonList`), and a roll (`mSP_GetItemList`) picks
//! the rarity. Clothing lists hold all-year shirts first, then each
//! season's (`cloth_season_cnt`), and only this season's are sold.

use std::sync::OnceLock;

use disc::{RomSource, Target};

use crate::game_disc::GameDisc;
use crate::items::ItemId;

/// `mSP_KIND_*`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Furniture = 0,
    Paper = 1,
    Cloth = 2,
    Carpet = 3,
    Wallpaper = 4,
    Diary = 5,
}

/// `mSP_LIST_NUM`: lists per kind.
const LIST_COUNT: u32 = 23;
/// The pointer tables, by kind (`mSP_goods_seg_inf`).
const LIST_TABLES: [&str; 5] = [
    "mSP_ftr_list",
    "mSP_binsen_list",
    "mSP_cloth_list",
    "mSP_carpet_list",
    "mSP_wall_list",
];
/// Lists A-C of diaries (`diary_listA`..`C`, `ITM_DIARY_START` 0x2B00).
const DIARIES: [&[ItemId]; 3] = [
    &[0x2B01, 0x2B04, 0x2B07, 0x2B0A, 0x2B0D],
    &[0x2B02, 0x2B05, 0x2B08, 0x2B0B, 0x2B0E],
    &[0x2B03, 0x2B06, 0x2B09, 0x2B0C, 0x2B0F, 0x2B00],
];
/// `mSP_SetDummyItem`'s stand-ins when a kind has no lists.
const DUMMIES: [ItemId; 6] = [0x1004, 0x2000, 0x2400, 0x2600, 0x2700, 0x2B00];
/// `priority_candidate` in `mSP_DecideUniqueCommonList`: the rarity of
/// lists A, B and C (0 common, 1 uncommon, 2 rare).
const PRIORITY_CANDIDATES: [[u8; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [2, 0, 1],
    [2, 1, 0],
    [1, 2, 0],
];

struct GoodsLists {
    /// Per kind (furniture .. wallpaper), its 23 lists (empty: none).
    lists: Vec<Vec<Vec<ItemId>>>,
    /// All-year, spring, summer, autumn, winter shirt counts.
    cloth_season: [u8; 5],
}

static LISTS: OnceLock<GoodsLists> = OnceLock::new();

/// The disc's raw list `index` of `kind` (mSP_LIST_*: 0-2 A-C, 3 event,
/// 4 train, 5 lottery ...).
pub fn list(kind: Kind, index: usize) -> Vec<ItemId> {
    LISTS
        .get()
        .and_then(|g| g.lists.get(kind as usize)?.get(index).cloned())
        .unwrap_or_default()
}

/// Reads the goods lists once the disc is loaded.
pub fn install(disc: &GameDisc) {
    let rom = &disc.rom;
    let rel = |o: u32| match rom.pointer_at(RomSource::Rel, o) {
        Some(Target::File(RomSource::Rel, p)) => Some(p),
        _ => None,
    };
    // A list ends at EMPTY_NO.
    let read_list = |p: u32| -> Vec<ItemId> {
        (0..)
            .map_while(|i| rom.slice(RomSource::Rel, p + i * 2, 2))
            .map(|b| u16::from_be_bytes([b[0], b[1]]))
            .take_while(|&n| n != 0)
            .collect()
    };
    let lists = LIST_TABLES
        .iter()
        .map(|name| {
            let Some(table) = disc.catalog.get(name) else {
                return Vec::new();
            };
            (0..LIST_COUNT)
                .map(|i| rel(table.offset + i * 4).map(read_list).unwrap_or_default())
                .collect()
        })
        .collect();
    let mut cloth_season = [0; 5];
    if let Some(b) = disc
        .catalog
        .get("cloth_season_cnt")
        .and_then(|a| rom.slice(RomSource::Rel, a.offset, 5))
    {
        cloth_season.copy_from_slice(b);
    }
    let _ = LISTS.set(GoodsLists {
        lists,
        cloth_season,
    });
}

/// The shop's priorities (`Shop_c.priority_lists`), for rolls made
/// away from the shop (gifts, rewards).
static PRIORITIES: std::sync::Mutex<[[u8; 3]; 5]> = std::sync::Mutex::new([[0, 1, 2]; 5]);

/// Records the shop's priorities.
pub fn set_priorities(p: [[u8; 3]; 5]) {
    if let Ok(mut g) = PRIORITIES.lock() {
        *g = p;
    }
}

/// A roll today with the shop's priorities and no luck either way.
pub fn roll_now() -> Roll {
    let now = crate::clock::now();
    Roll {
        priorities: PRIORITIES.lock().map(|g| *g).unwrap_or([[0, 1, 2]; 5]),
        goods_power: 0,
        month: now.month,
        day: now.day,
    }
}

/// One random good of `kind` from the A/B/C lists (`mSP_SelectRandomItem_New`
/// with `mSP_LISTTYPE_ABC`), as villagers and errands hand out.
pub fn random(kind: Kind, below: &mut dyn FnMut(u32) -> u32) -> ItemId {
    select(&roll_now(), kind, 1, &[], below)
        .first()
        .copied()
        .unwrap_or(DUMMIES[kind as usize])
}

/// A new shop's rarity for lists A, B and C of each kind.
pub fn decide_priorities(below: &mut dyn FnMut(u32) -> u32) -> [[u8; 3]; 5] {
    std::array::from_fn(|_| PRIORITY_CANDIDATES[below(6) as usize])
}

/// `mSP_get_cloth_season`: 1 spring .. 4 winter, by month (1-12) and day.
fn cloth_season(month: u8, day: u8) -> usize {
    match (month, day) {
        (1, _) | (12, _) => 4,
        (2, d) => {
            if d > 24 {
                1
            } else {
                4
            }
        }
        (3 | 4, _) => 1,
        (5, d) => {
            if d > 26 {
                2
            } else {
                1
            }
        }
        (6 | 7, _) => 2,
        (8, d) => {
            if d > 26 {
                3
            } else {
                2
            }
        }
        (9 | 10, _) => 3,
        (11, d) => {
            if d > 26 {
                4
            } else {
                3
            }
        }
        _ => 1,
    }
}

/// The roll's settings: shop priorities, the player's goods luck
/// (`mPr_GetGoodsPower`, -30..50) and the date.
#[derive(Clone, Copy, Debug)]
pub struct Roll {
    pub priorities: [[u8; 3]; 5],
    pub goods_power: i32,
    pub month: u8,
    pub day: u8,
}

/// `mSP_GetItemList` / `mSP_SelectListFromPriority` for the A/B/C roll:
/// the list to draw from (index into the kind's lists).
fn pick_list(roll: &Roll, kind: Kind, below: &mut dyn FnMut(u32) -> u32) -> usize {
    let r = below(100) as i32;
    let p = roll.goods_power;
    let (rare, uncommon) = if p < 0 { (5, p + 40) } else { (p + 5, p + 40) };
    let rarity = if r < rare {
        2
    } else if r < uncommon {
        1
    } else {
        0
    };
    // Diaries use the furniture priorities (`mSP_GetGoodsPriority`).
    let kind = if kind == Kind::Diary {
        Kind::Furniture
    } else {
        kind
    };
    roll.priorities[kind as usize]
        .iter()
        .position(|&p| p == rarity)
        .unwrap_or(0)
}

/// `mSP_SelectRandomItem_New` (every item eligible): `count` different
/// goods of `kind`, none already in `exclude` (unless the list is too
/// short to avoid it).
pub fn select(
    roll: &Roll,
    kind: Kind,
    count: usize,
    exclude: &[ItemId],
    below: &mut dyn FnMut(u32) -> u32,
) -> Vec<ItemId> {
    let Some(goods) = LISTS.get() else {
        return vec![DUMMIES[kind as usize]; count];
    };
    let mut out: Vec<ItemId> = Vec::with_capacity(count);
    let mut guard = 0;
    while out.len() < count {
        guard += 1;
        if guard > 10_000 {
            break;
        }
        let i = pick_list(roll, kind, below);
        let list: &[ItemId] = match kind {
            Kind::Diary => DIARIES.get(i).copied().unwrap_or(DIARIES[0]),
            _ => goods.lists[kind as usize].get(i).map_or(&[], Vec::as_slice),
        };
        if list.is_empty() {
            return vec![DUMMIES[kind as usize]; count];
        }
        let (n, pick) = if kind == Kind::Cloth {
            // `mSP_CountElementInClothList`: all-year shirts, then this
            // season's (after the earlier seasons').
            let c = goods.cloth_season;
            let season = cloth_season(roll.month, roll.day);
            let n = u32::from(c[0]) + u32::from(c[season]);
            let mut pick = below(n.max(1));
            if pick >= u32::from(c[0]) {
                pick += c[1..season].iter().map(|&s| u32::from(s)).sum::<u32>();
            }
            (n, pick as usize)
        } else {
            let n = list.len() as u32;
            (n, below(n) as usize)
        };
        let Some(&item) = list.get(pick) else {
            continue;
        };
        // Duplicates only when the list is too short to avoid them.
        if (!out.contains(&item) && !exclude.contains(&item))
            || (n as usize) < count + exclude.len()
        {
            out.push(item);
        }
    }
    out
}

/// Goods per shop tier (the spotlight furniture of the two bigger shops,
/// `rare_item`, isn't stocked yet) (`l_zakka_goods` .. `l_dsuper_goods`):
/// stationery, clothing, furniture, rare furniture, carpets, wallpaper,
/// -, -, saplings, tools, flower bags.
const GOODS_COUNT: [[u8; 11]; 4] = [
    [1, 1, 1, 0, 1, 1, 1, 0, 1, 2, 2],
    [2, 2, 2, 0, 1, 1, 1, 0, 1, 3, 3],
    [2, 3, 3, 1, 2, 2, 1, 0, 2, 2, 4],
    [4, 5, 5, 1, 3, 3, 1, 0, 3, 3, 5],
];
/// Shovel, net, rod, axe (`mSP_SelectTool`).
const TOOLS: [ItemId; 4] = [0x2202, 0x2200, 0x2203, 0x2201];
/// Sales that unlock the net, rod and axe at Nook's Cranny.
const TOOL_SALES: [u32; 3] = [3000, 8000, 12000];
/// `ITM_UMBRELLA00`, `UMBRELLA_NUM`.
const UMBRELLA00: ItemId = 0x2204;
const UMBRELLA_NUM: u32 = 32;
/// `ITM_SAPLING`, `ITM_CEDAR_SAPLING`, `ITM_WHITE_PANSY_BAG`, `FLOWER_NUM`.
const SAPLING: ItemId = 0x2900;
const CEDAR_SAPLING: ItemId = 0x2901;
const FLOWER_BAG0: ItemId = 0x2902;
const FLOWER_NUM: u32 = 9;
/// `ITM_FOOD_CANDY`, stocked in the plant reserves during Halloween.
pub(crate) const CANDY: ItemId = 0x2806;
/// `ITM_RED_PAINT`, `ITM_SIGNBOARD`.
const RED_PAINT: ItemId = 0x222D;
const SIGNBOARD: ItemId = 0x251E;
const PAINT_NUM: u8 = 12;

/// A day's stock for a shop of `tier` (0 Nook's Cranny .. 3
/// Nookington's), `mSP_MakeGoodsList` on a normal day: furniture
/// (December trees and candles in season), stationery (and a diary from
/// Nookway), clothing, carpets, wallpaper, then tools (fewer at the
/// Cranny until its sales unlock them), paint and a sign from Nookway,
/// an umbrella, saplings and flower bags. October 16..30 replaces plants
/// with candy and saplings with flowers. `paint` is the next paint
/// colour, advanced when sold.
pub fn make_goods_list(
    roll: &Roll,
    tier: usize,
    sales_sum: u32,
    paint: &mut u8,
    below: &mut dyn FnMut(u32) -> u32,
) -> Vec<ItemId> {
    let counts = GOODS_COUNT[tier.min(3)];
    let n = |i: usize| usize::from(counts[i]);
    let mut out = select(roll, Kind::Furniture, n(2), &[], below);
    season_furniture(roll, &mut out, below);
    out.extend(select(roll, Kind::Paper, n(0), &[], below));
    if tier >= 2 {
        out.extend(select(roll, Kind::Diary, 1, &[], below));
    }
    out.extend(select(roll, Kind::Cloth, n(1), &[], below));
    out.extend(select(roll, Kind::Carpet, n(4), &[], below));
    out.extend(select(roll, Kind::Wallpaper, n(5), &[], below));

    // Tools (`mSP_SelectTool`).
    let tool_max = if tier > 0 {
        4
    } else {
        1 + TOOL_SALES.iter().filter(|&&s| sales_sum >= s).count()
    };
    let mut added = 0;
    while added < n(9).min(tool_max) {
        let tool = TOOLS[below(tool_max as u32) as usize];
        if !out.contains(&tool) {
            out.push(tool);
            added += 1;
        }
    }
    if tier >= 2 {
        if *paint >= PAINT_NUM {
            *paint = 0;
        }
        out.push(RED_PAINT + ItemId::from(*paint));
        *paint += 1;
        out.push(SIGNBOARD);
    }
    out.push(UMBRELLA00 + below(UMBRELLA_NUM) as ItemId);

    // Plants (`mSP_SelectPlant`).
    let halloween = roll.month == 10 && (16..=30).contains(&roll.day);
    let mut saplings = if halloween { 0 } else { n(8) };
    if tier >= 2 && saplings > 0 {
        out.push(CEDAR_SAPLING);
        saplings -= 1;
    }
    out.extend(std::iter::repeat_n(SAPLING, saplings));
    let mut used = [false; FLOWER_NUM as usize];
    let mut flowers = if halloween { n(8) } else { n(10) };
    while flowers > 0 {
        let i = below(FLOWER_NUM) as usize;
        if !used[i] {
            used[i] = true;
            out.push(FLOWER_BAG0 + i as ItemId);
            flowers -= 1;
        }
    }
    // Native stores these in flowers_candy_grab_bag_count. The existing
    // stock vector represents each unit explicitly, retaining its sold flag
    // and display identity across save/load without another save field.
    if halloween {
        out.extend(std::iter::repeat_n(CANDY, n(10)));
    }
    out
}

/// `mSP_SetSeasonFTR`: Christmas trees until the 24th of December, then
/// candles and flags, in the first furniture slots.
fn season_furniture(roll: &Roll, goods: &mut [ItemId], below: &mut dyn FnMut(u32) -> u32) {
    /// `FTR_NOG_XTREE`, `FTR_KON_XTREE02`, `FTR_YAZ_CANDLE`, `FTR_YOS_KFLAG`.
    const XMAS: (ItemId, ItemId) = (0x1EBC, 0x1E88);
    const NEW_YEAR: (ItemId, ItemId) = (0x3298, 0x327C);
    if roll.month != 12 || goods.is_empty() || roll.day == 25 {
        return;
    }
    let (a, b) = if roll.day <= 24 { XMAS } else { NEW_YEAR };
    if goods.len() > 1 {
        goods[0] = a;
        goods[1] = b;
    } else {
        goods[0] = if below(2) == 0 { a } else { b };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seasons_turn_late_in_the_month() {
        assert_eq!(cloth_season(2, 24), 4);
        assert_eq!(cloth_season(2, 25), 1);
        assert_eq!(cloth_season(8, 27), 3);
        assert_eq!(cloth_season(11, 27), 4);
    }

    #[test]
    fn luck_shifts_the_rarity() {
        let roll = Roll {
            priorities: [[0, 1, 2]; 5],
            goods_power: 0,
            month: 1,
            day: 1,
        };
        let list = |r: u32| pick_list(&roll, Kind::Paper, &mut |_| r);
        assert_eq!((list(4), list(5), list(39), list(40)), (2, 1, 1, 0));
        let lucky = Roll {
            goods_power: 50,
            ..roll
        };
        assert_eq!(pick_list(&lucky, Kind::Paper, &mut |_| 54), 2);
        let swapped = Roll {
            priorities: [[2, 0, 1]; 5],
            ..roll
        };
        // Common is list B when the priorities put A at rare.
        assert_eq!(pick_list(&swapped, Kind::Paper, &mut |_| 99), 1);
    }
}
