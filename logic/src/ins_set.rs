//! Which insect hatches in an acre, and on which units
//! (`ac_set_ovl_insect.c`: `aSOI_ins_decide_insect`, `aSOI_make_live_ut`,
//! `aSOI_ins_get_idx`, `aSOI_ins_make`).

use crate::bee_ant::{limit_insect_data, SpawnCandidate, ANT_OVERLAY};
use crate::species::{insect_birth_sum, InsectArea, InsectType};

/// `aSOI_SPAWN_AREA_*` (the `InsectArea` order).
const ON_TREE: u8 = 0;
const ON_FLOWER: u8 = 1;
const RAINING_ON_FLOWER: u8 = 2;
const FLYING: u8 = 3;
const ON_GROUND: u8 = 4;
const IN_BUSH: u8 = 5;
const FLYING_NEAR_WATER: u8 = 6;
const ON_WATER: u8 = 7;
const ON_CANDY: u8 = 8;
const ON_TRASH: u8 = 9;
const UNDER_ROCK: u8 = 10;
const UNDERGROUND: u8 = 11;
const NEAR_FLOWERS_OR_AROUND: u8 = 12;
const NOTHING: u8 = 13;

/// `mCoBG_ATTRIBUTE_*`.
const SOIL1: u8 = 5;
const BUSH: u8 = 9;
const HOLE: u8 = 10;
const WATER: u8 = 12;
const RIVER_NE: u8 = 21;
const SEA: u8 = 24;

/// `FLOWER_PANSIES0..FLOWER_COSMOS2`: the flowers insects sit on.
const FLOWERS: (u16, u16) = (0x0845, 0x084A);
const CANDY: u16 = 0x2806;
const SPOILED_TURNIP: u16 = 0x2F03;
/// `ROCK_A..ROCK_E`.
const ROCKS: (u16, u16) = (0x0063, 0x0067);

/// `aSOI_tree_check`.
pub fn is_tree(item: u16) -> bool {
    matches!(
        item,
        0x0804
            | 0x005F
            | 0x0060
            | 0x0069
            | 0x080C
            | 0x0814
            | 0x081C
            | 0x0824
            | 0x082C
            | 0x0831
            | 0x0836
            | 0x083B
            | 0x0853
            | 0x0861
            | 0x0078
            | 0x0079
            | 0x0082
            | 0x0868
            | 0x007F
            | 0x0080
            | 0x0867
    )
}

/// `mCoBG_CheckWaterAttribute`.
pub fn is_water(attr: u8) -> bool {
    attr == SEA || (WATER..=RIVER_NE).contains(&attr)
}

/// `mCoBG_CheckHole_OrgAttr`.
pub(crate) fn is_hole(attr: u8) -> bool {
    matches!(attr, HOLE | 0..=2 | 4..=6 | 22 | 25 | 26 | 36 | 43..=46 | 59..=62)
}

/// One acre as the spawner sees it.
pub struct Acre<'a> {
    /// Field items, unit by unit (row by row).
    pub items: &'a [u16; 256],
    /// Collision attributes, unit by unit.
    pub attrs: &'a [u8; 256],
    /// Block kind: pool, waterfall, river, marine (sea), offing.
    pub pool: bool,
    pub waterfall: bool,
    pub river: bool,
    pub marine: bool,
    pub offing: bool,
    pub rain: bool,
    pub snow: bool,
    /// `mFAs_GetFieldRank` (0..6).
    pub field_rank: u8,
}

/// What hatches: the insect, its area, and the units it is on (one per
/// insect; the red dragonfly and firefly come in swarms).
#[derive(Debug, Clone, PartialEq)]
pub struct Hatch {
    pub insect: InsectType,
    pub area: InsectArea,
    pub units: Vec<usize>,
}

/// Units 2..14 on both axes (the spawner never uses the outer two rings).
fn inner() -> impl Iterator<Item = usize> {
    (0..256).filter(|ut| (2..14).contains(&(ut % 16)) && (2..14).contains(&(ut / 16)))
}

impl Acre<'_> {
    /// Units an insect of `area` can be on (`aSOI_make_live_ut`).
    fn live_units(&self, area: u8) -> Vec<usize> {
        let in_range = |ut: usize, (lo, hi): (u16, u16)| (lo..=hi).contains(&self.items[ut]);
        inner()
            .filter(|&ut| {
                let attr = self.attrs[ut];
                match area {
                    ON_TREE => is_tree(self.items[ut]),
                    ON_FLOWER | RAINING_ON_FLOWER | NEAR_FLOWERS_OR_AROUND => in_range(ut, FLOWERS),
                    FLYING | NOTHING => self.items[ut] == 0,
                    ON_GROUND => attr <= SOIL1 || attr == BUSH,
                    IN_BUSH => attr == BUSH,
                    FLYING_NEAR_WATER => is_water(attr),
                    ON_WATER => (WATER..=RIVER_NE).contains(&attr),
                    ON_CANDY => self.items[ut] == CANDY,
                    ON_TRASH => self.items[ut] == SPOILED_TURNIP,
                    UNDER_ROCK => in_range(ut, ROCKS),
                    UNDERGROUND => is_hole(attr) && self.items[ut] == 0,
                    _ => false,
                }
            })
            .collect()
    }

    /// The `chk_live_area_proc` of each area: can it hatch here at all?
    fn area_ok(&self, area: u8) -> bool {
        let any = || !self.live_units(area).is_empty();
        match area {
            ON_FLOWER | NEAR_FLOWERS_OR_AROUND => !self.rain && any(),
            RAINING_ON_FLOWER => self.rain && any(),
            ON_CANDY | ON_TRASH => !self.rain && !self.snow && any(),
            ON_WATER => (self.pool || !(self.waterfall || self.river || self.marine)) && any(),
            NOTHING => true,
            _ => any(),
        }
    }

    /// `aSOI_insect_set` for an acre with no insect yet. `table` is the
    /// season's (`species::insect_seasonal_table`); `rand(n)` is in 0..n.
    pub fn decide(
        &self,
        table: &[(InsectType, InsectArea, f32)],
        rand: &mut impl FnMut(f32) -> f32,
    ) -> Option<Hatch> {
        if self.offing {
            return None;
        }
        let mut cands: Vec<SpawnCandidate> = table
            .iter()
            .map(|&(insect, area, weight)| SpawnCandidate {
                insect: insect as u8,
                area: area as u8,
                weight,
            })
            .chain(ANT_OVERLAY.iter().map(|c| SpawnCandidate {
                area: c.area + ON_CANDY,
                ..*c
            }))
            .collect();
        let (mut candy, mut trash) = (false, false);
        for area in ON_TREE..=NOTHING {
            let ok = self.area_ok(area);
            if area == NEAR_FLOWERS_OR_AROUND && !self.rain {
                // aSOI_ins_chk_live_area_type_flower_or_free: on a flower
                // if there is one, else just flying.
                let to = if ok { ON_FLOWER } else { FLYING };
                for c in cands.iter_mut().filter(|c| c.area == area) {
                    c.area = to;
                }
                continue;
            }
            match area {
                ON_CANDY => candy = ok,
                ON_TRASH => trash = ok,
                _ => {}
            }
            if !ok {
                for c in cands.iter_mut().filter(|c| c.area == area) {
                    c.weight = 0.0;
                }
            }
        }
        if candy || trash {
            // bee_ant's overlay areas are 0/1; ours are 8/9.
            for c in cands.iter_mut() {
                c.area = c.area.wrapping_sub(ON_CANDY);
            }
            limit_insect_data(&mut cands, candy, trash);
            for c in cands.iter_mut() {
                c.area = c.area.wrapping_add(ON_CANDY);
            }
        }
        let pick = self.get_idx(&cands, candy || trash, rand)?;
        let c = cands[pick];
        if c.area >= NEAR_FLOWERS_OR_AROUND {
            return None;
        }
        let insect = InsectType::from_u8(c.insect)?;
        let area = area_of(c.area)?;
        let mut live = self.live_units(c.area);
        let (min, extra) = insect_birth_sum(insect);
        let count = min as usize + (rand(1.0) * extra as f32) as usize;
        let mut units = Vec::new();
        for _ in 0..count {
            if live.is_empty() {
                break;
            }
            let i = ((rand(1.0) * live.len() as f32) as usize).min(live.len() - 1);
            units.push(live.remove(i));
        }
        (!units.is_empty()).then_some(Hatch {
            insect,
            area,
            units,
        })
    }

    /// `aSOI_ins_get_idx`: a weighted pick; under 100 total, the rest of
    /// the 100 is no insect. The town's rank thins it out.
    fn get_idx(
        &self,
        cands: &[SpawnCandidate],
        candy_or_trash: bool,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> Option<usize> {
        const ENV_RATE: [f32; 7] = [0.5, 0.75, 0.875, 1.0, 1.0, 1.0, 1.0];
        let total: f32 = cands.iter().map(|c| c.weight).sum();
        if total <= 0.0 {
            return None;
        }
        let (mut roll, rate) = if candy_or_trash {
            (rand(1.0) * total, 1.0)
        } else {
            (
                rand(1.0) * total.max(100.0),
                ENV_RATE[self.field_rank.min(6) as usize],
            )
        };
        for (i, c) in cands.iter().enumerate() {
            roll -= c.weight * rate;
            if roll < 0.0 {
                return Some(i);
            }
        }
        None
    }
}

fn area_of(a: u8) -> Option<InsectArea> {
    use InsectArea::*;
    [
        OnTree,
        OnFlower,
        RainingOnFlower,
        Flying,
        OnGround,
        InBush,
        FlyingNearWater,
        OnWater,
        OnCandy,
        OnTrash,
        UnderRock,
        Underground,
        FlyingNearFlowersOrAround,
        Nothing,
    ]
    .get(a as usize)
    .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn acre<'a>(items: &'a [u16; 256], attrs: &'a [u8; 256]) -> Acre<'a> {
        Acre {
            items,
            attrs,
            pool: false,
            waterfall: false,
            river: false,
            marine: false,
            offing: false,
            rain: false,
            snow: false,
            field_rank: 6,
        }
    }

    #[test]
    fn a_flower_lover_lands_on_the_flower_otherwise_flies() {
        let attrs = [0u8; 256];
        let mut items = [0u16; 256];
        let table = [(
            InsectType::CommonButterfly,
            InsectArea::FlyingNearFlowersOrAround,
            100.0,
        )];
        let mut r = |n: f32| n * 0.3;
        let h = acre(&items, &attrs).decide(&table, &mut r).unwrap();
        assert_eq!(h.area, InsectArea::Flying);
        items[5 * 16 + 5] = 0x0846;
        let h = acre(&items, &attrs).decide(&table, &mut r).unwrap();
        assert_eq!(
            (h.area, h.units.clone()),
            (InsectArea::OnFlower, vec![5 * 16 + 5])
        );
    }

    #[test]
    fn no_tree_no_tree_bug_and_light_tables_often_give_nothing() {
        let (items, attrs) = ([0u16; 256], [0u8; 256]);
        let table = [(InsectType::RobustCicada, InsectArea::OnTree, 100.0)];
        assert_eq!(acre(&items, &attrs).decide(&table, &mut |n| n * 0.1), None);
        let table = [(InsectType::LongLocust, InsectArea::OnGround, 30.0)];
        assert_eq!(
            acre(&items, &attrs).decide(&table, &mut |n| n * 0.5),
            None,
            "roll 50 of 100 misses 30"
        );
        assert!(acre(&items, &attrs)
            .decide(&table, &mut |n| n * 0.2)
            .is_some());
    }

    #[test]
    fn fireflies_come_in_swarms() {
        let mut attrs = [0u8; 256];
        attrs[..].fill(WATER);
        let items = [0u16; 256];
        let table = [(InsectType::Firefly, InsectArea::FlyingNearWater, 100.0)];
        let h = acre(&items, &attrs)
            .decide(&table, &mut |n| n * 0.5)
            .unwrap();
        assert!((6..=9).contains(&h.units.len()), "{h:?}");
    }
}
