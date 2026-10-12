//! Retail random town generator (`m_random_field_ovl.c`, USA decomp),
//! ported function by function. Produces the 7x10 acre map of
//! `mFM_BLOCK_TYPE_*` values ([`bt`]) and the per-acre base heights, the
//! way `mRF_MakeRandomField_ovl` builds them before choosing templates.
//!
//! Randomness comes from the caller (`RANDOM(max)` in the decomp), so the
//! game's own `qrand` or any seeded generator can drive it.

use crate::random_field_tables::*;

pub use crate::random_field_tables::bt;

pub const BLOCK_X_NUM: usize = 7;
pub const BLOCK_Z_NUM: usize = 10;
pub const BLOCK_TOTAL_NUM: usize = BLOCK_X_NUM * BLOCK_Z_NUM;
const FG_BLOCK_Z_NUM: i32 = 6;

/// `RANDOM(max)`: a value in `0..max`.
pub trait FieldRng {
    fn random(&mut self, max: i32) -> i32;
}

impl<F: FnMut(i32) -> i32> FieldRng for F {
    fn random(&mut self, max: i32) -> i32 {
        self(max)
    }
}

pub type Blocks = [u8; BLOCK_TOTAL_NUM];

// mRF_BLOCK_GROUP_*
const GROUP_CLIFF: usize = 0;
const GROUP_RIVER: usize = 1;
const GROUP_BRIDGE: usize = 2;
const GROUP_SLOPE: usize = 3;
const GROUP_RIVER_CLIFF_ANY: usize = 4;
const GROUP_RIVER_CLIFF_1: usize = 5;
const GROUP_RIVER_CLIFF_2: usize = 6;
const GROUP_RIVER_CLIFF_3: usize = 7;
const GROUP_CLIFF_ANY: usize = 8;

// mRF_STEPMODE_*
pub const STEPMODE_TWO: i32 = 0;
pub const STEPMODE_THREE: i32 = 1;

const CLIFF_START_TABLE: [&[u8]; 4] = [
    &L_CLIFF_START_A_FACTOR,
    &L_CLIFF_START_A_FACTOR,
    &L_CLIFF_START_B_FACTOR,
    &L_CLIFF_START_C_FACTOR,
];

const CLIFF_NEXT_DATA: [&[u8]; 7] = [
    &L_CLIFF1_NEXT,
    &L_CLIFF2_NEXT,
    &L_CLIFF3_NEXT,
    &L_CLIFF4_NEXT,
    &L_CLIFF5_NEXT,
    &L_CLIFF6_NEXT,
    &L_CLIFF7_NEXT,
];

const RIVER_NEXT_DATA: [&[u8]; 7] = [
    &L_RIVER1_NEXT,
    &L_RIVER2_NEXT,
    &L_RIVER3_NEXT,
    &L_RIVER4_NEXT,
    &L_RIVER5_NEXT,
    &L_RIVER6_NEXT,
    &L_RIVER7_NEXT,
];

const STEP3_BLOCKSS: [&Blocks; 10] = [
    &STEP3_BLOCKS_3,
    &STEP3_BLOCKS_7,
    &STEP3_BLOCKS_7R,
    &STEP3_BLOCKS_8,
    &STEP3_BLOCKS_B,
    &STEP3_BLOCKS_BR,
    &STEP3_BLOCKS_E,
    &STEP3_BLOCKS_ER,
    &STEP3_BLOCKS_F,
    &STEP3_BLOCKS_FR,
];

fn system_block_info(t: u8) -> i32 {
    SYSTEM_BLOCK_INFO.get(t as usize).copied().unwrap_or(0)
}

fn block_type_to_cliff_index(t: u8) -> i32 {
    let info = system_block_info(t);
    CLIFF_INFO
        .iter()
        .position(|&c| info & c != 0)
        .map_or(-1, |i| i as i32)
}

fn d2_to_d1(bx: i32, bz: i32) -> usize {
    (bz * BLOCK_X_NUM as i32 + bx) as usize
}

fn direct_to_block_no(base_bx: i32, base_bz: i32, direct: u8) -> (i32, i32) {
    let idx = (direct & 3) as usize;
    (base_bx + X_OFFSET[idx], base_bz + Z_OFFSET[idx])
}

fn check_correct_block_no(
    bx: i32,
    bz: i32,
    bx_min: i32,
    bx_max: i32,
    bz_min: i32,
    bz_max: i32,
) -> bool {
    (bx_min..=bx_max).contains(&bx) && (bz_min..=bz_max).contains(&bz)
}

const BLOCK_GROUP: [(u8, u8); 9] = [
    (bt::CLIFF_HORIZONTAL, bt::CLIFF_BOTTOM_LEFT_CORNER),
    (bt::RIVER_SOUTH, bt::RIVER_WEST_SOUTH),
    (bt::RIVER_SOUTH_BRIDGE, bt::RIVER_WEST_SOUTH_BRIDGE),
    (bt::SLOPE_HORIZONTAL, bt::SLOPE_BOTTOM_LEFT_CORNER),
    (
        bt::WATERFALL_STRAIGHT_CLIFF_HORIZONTAL,
        bt::WATERFALL_WEST_CLIFF_BOTTOM_LEFT_CORNER,
    ),
    (
        bt::WATERFALL_STRAIGHT_CLIFF_HORIZONTAL,
        bt::RIVER_STRAIGHT_CLIFF_BOTTOM_LEFT_CORNER,
    ),
    (
        bt::RIVER_STRAIGHT_CLIFF_HORIZONTAL,
        bt::RIVER_EAST_CLIFF_TOP_LEFT_CORNER,
    ),
    (
        bt::RIVER_WEST_CLIFF_HORIZONTAL,
        bt::WATERFALL_WEST_CLIFF_BOTTOM_LEFT_CORNER,
    ),
    (0, 0),
];

fn check_block_group(t: u8, group: usize) -> bool {
    let within = |g: usize| (BLOCK_GROUP[g].0..=BLOCK_GROUP[g].1).contains(&t);
    if group == GROUP_CLIFF_ANY {
        // Any acre with a cliff in it, even with rivers or slopes.
        return within(GROUP_CLIFF) || within(GROUP_SLOPE) || within(GROUP_RIVER_CLIFF_ANY);
    }
    within(group)
}

fn trace_cliff_block(blocks: &mut Blocks, rng: &mut impl FieldRng, bx: i32, bz: i32) -> bool {
    let t = blocks[d2_to_d1(bx, bz)].wrapping_sub(bt::CLIFF_HORIZONTAL) as usize;
    let next_data = CLIFF_NEXT_DATA[t];
    let next = next_data[rng.random(next_data.len() as i32) as usize];
    let (n_bx0, n_bz0) = direct_to_block_no(bx, bz, L_CLIFF_NEXT_DIRECT[t]);
    let (n_bx1, n_bz1) = direct_to_block_no(
        n_bx0,
        n_bz0,
        L_CLIFF_NEXT_DIRECT[next.wrapping_sub(bt::CLIFF_HORIZONTAL) as usize],
    );
    if !(check_correct_block_no(n_bx0, n_bz0, 1, 5, 2, 5)
        && check_correct_block_no(n_bx1, n_bz1, 1, 6, 2, 5))
    {
        return false;
    }
    if blocks[d2_to_d1(n_bx1, n_bz1)] == bt::PLAYER_HOUSE
        || blocks[d2_to_d1(n_bx0, n_bz0)] != bt::FLAT
    {
        return false;
    }
    blocks[d2_to_d1(n_bx0, n_bz0)] = next;
    if n_bx0 == 5 {
        return true; // final cliff reached
    }
    trace_cliff_block(blocks, rng, n_bx0, n_bz0)
}

fn set_end_cliff_block(blocks: &mut Blocks) {
    let mut end_z = 0;
    let mut cliff_type = 0usize;
    for i in 0..FG_BLOCK_Z_NUM {
        let b = blocks[d2_to_d1(5, i)];
        if (bt::CLIFF_HORIZONTAL..=bt::CLIFF_BOTTOM_LEFT_CORNER).contains(&b) {
            end_z = i;
            cliff_type = (b - bt::CLIFF_HORIZONTAL) as usize;
        }
    }
    let direct = L_CLIFF_NEXT_DIRECT[cliff_type];
    if direct != DIRECT_EAST {
        // The cliff doesn't end facing the east wall: add a piece that does.
        for &t in CLIFF_NEXT_DATA[cliff_type] {
            if L_CLIFF_NEXT_DIRECT[(t - bt::CLIFF_HORIZONTAL) as usize] == DIRECT_EAST {
                let (bx, bz) = direct_to_block_no(5, end_z, direct);
                blocks[d2_to_d1(bx, bz)] = t;
                blocks[d2_to_d1(bx + 1, bz)] = bt::BORDER_CLIFF_RIGHT_TRANSITION;
                break;
            }
        }
    } else {
        blocks[d2_to_d1(6, end_z)] = bt::BORDER_CLIFF_RIGHT_TRANSITION;
    }
}

fn last_check_cliff(blocks: &Blocks, bx: i32, bz: i32) -> bool {
    let (mut last_x, mut last_z) = (bx, bz);
    let mut t = blocks[d2_to_d1(bx, bz)];
    while check_block_group(t, GROUP_CLIFF) {
        let direct = L_CLIFF_NEXT_DIRECT[(t - bt::CLIFF_HORIZONTAL) as usize];
        (last_x, last_z) = direct_to_block_no(last_x, last_z, direct);
        t = blocks[d2_to_d1(last_x, last_z)];
    }
    // The cliff can't start and end in the same row.
    last_x > 5 && bz != last_z
}

fn decide_base_cliff(blocks: &mut Blocks, rng: &mut impl FieldRng) -> bool {
    let start_z = rng.random(4) as usize;
    let bz = start_z as i32 + 2;
    let starts = CLIFF_START_TABLE[start_z];
    let start_type = starts[rng.random(starts.len() as i32) as usize];
    blocks[d2_to_d1(1, bz)] = start_type;
    blocks[d2_to_d1(0, bz)] = bt::BORDER_CLIFF_LEFT_TRANSITION;
    trace_cliff_block(blocks, rng, 1, bz);
    set_end_cliff_block(blocks);
    last_check_cliff(blocks, 1, bz)
}

const BLOCK_TYPE_NONE: u8 = 255;

/// `l_river_next_direct` (m_random_field.c).
const RIVER_NEXT_DIRECT: [u8; 7] = [
    DIRECT_SOUTH,
    DIRECT_EAST,
    DIRECT_WEST,
    DIRECT_EAST,
    DIRECT_SOUTH,
    DIRECT_WEST,
    DIRECT_SOUTH,
];

/// `mRF_RiverIdx2NextDirect`; out of range gives `mRF_DIRECT_ERROR`.
pub fn river_idx_to_next_direct(river_idx: u8) -> u8 {
    RIVER_NEXT_DIRECT
        .get(river_idx as usize)
        .copied()
        .unwrap_or(DIRECT_ERROR as u8)
}

fn river_albumin_cliff(cliff_type: u8, river_type: u8) -> u8 {
    const ALBUM: [&[u8; 7]; 7] = [
        &RIVER1_ALBUM_DATA,   // south river
        &RIVER2_ALBUM_DATA,   // east river
        &RIVER3_ALBUM_DATA,   // west river
        &RIVER_NO_ALBUM_DATA, // south -> east
        &RIVER_NO_ALBUM_DATA, // east -> south
        &RIVER_NO_ALBUM_DATA, // south -> west
        &RIVER_NO_ALBUM_DATA, // west -> south
    ];
    if check_block_group(cliff_type, GROUP_CLIFF) && check_block_group(river_type, GROUP_RIVER) {
        let river = river_type.wrapping_sub(bt::RIVER_SOUTH) as usize;
        let cliff = cliff_type.wrapping_sub(bt::CLIFF_HORIZONTAL) as usize;
        if river < 7 && cliff < 7 {
            return ALBUM[river][cliff];
        }
    }
    BLOCK_TYPE_NONE
}

fn decide_river_albumin_cliff(cliff: &mut Blocks, river: &Blocks) {
    for bz in 0..(BLOCK_Z_NUM as i32 - 2) {
        for bx in 0..BLOCK_X_NUM as i32 {
            let n = d2_to_d1(bx, bz);
            let river_type = river[n];
            let album = river_albumin_cliff(cliff[n], river_type);
            if album != BLOCK_TYPE_NONE {
                cliff[n] = album;
            } else if check_block_group(river_type, GROUP_RIVER)
                || river_type == bt::BORDER_CLIFF_RIVER
                || river_type == bt::TRACKS_RIVER
            {
                cliff[n] = river[n];
            }
        }
    }
}

fn center_cross_zone_river_count(blocks: &Blocks) -> usize {
    (2..=(BLOCK_X_NUM as i32 - 2))
        .filter(|&i| check_block_group(blocks[d2_to_d1(3, i)], GROUP_RIVER))
        .count()
}

fn trace_river_part2(
    river: &mut Blocks,
    cliff: &Blocks,
    rng: &mut impl FieldRng,
    bx: i32,
    bz: i32,
) -> bool {
    let river_type = river[d2_to_d1(bx, bz)].wrapping_sub(bt::RIVER_SOUTH);
    let next_direct = river_idx_to_next_direct(river_type);
    loop {
        let next_types = RIVER_NEXT_DATA[river_type as usize];
        let mut next_type = next_types[rng.random(next_types.len() as i32) as usize];
        let mut next_next_direct =
            river_idx_to_next_direct(next_type.wrapping_sub(bt::RIVER_SOUTH));
        let (n_bx0, n_bz0) = direct_to_block_no(bx, bz, next_direct);
        if n_bz0 == 6 {
            next_type = bt::RIVER_SOUTH;
            next_next_direct = river_idx_to_next_direct(0);
        }
        let (n_bx1, n_bz1) = direct_to_block_no(n_bx0, n_bz0, next_next_direct);
        if !check_correct_block_no(n_bx0, n_bz0, 1, 5, 1, 6) {
            return false;
        }
        if !check_correct_block_no(n_bx1, n_bz1, 1, 5, 1, 7) {
            continue; // retry with another random piece
        }
        let next_bnum = d2_to_d1(n_bx0, n_bz0);
        if cliff[d2_to_d1(n_bx1, n_bz1)] == bt::PLAYER_HOUSE {
            continue;
        }
        if check_block_group(cliff[next_bnum], GROUP_CLIFF) {
            if river_albumin_cliff(cliff[next_bnum], next_types[0]) == BLOCK_TYPE_NONE {
                return false;
            }
            river[next_bnum] = next_types[0];
        } else {
            river[next_bnum] = next_type;
        }
        if n_bz1 != 7 {
            return trace_river_part2(river, cliff, rng, n_bx0, n_bz0);
        }
        // Reached the ocean: the river must have crossed the middle column
        // and can't end in the F-1 or F-5 acres.
        return center_cross_zone_river_count(river) != 0 && n_bx0 != 1 && n_bx0 != 5;
    }
}

/// Start of the river, or `None` when the attempt failed.
fn trace_river_part1(
    river: &mut Blocks,
    cliff: &Blocks,
    rng: &mut impl FieldRng,
) -> Option<(i32, i32)> {
    loop {
        let bx = START_X_TABLE[rng.random(4) as usize];
        let starts = RIVER_NEXT_DATA[0];
        let start_type = starts[rng.random(starts.len() as i32) as usize];
        let (n_bx0, n_bz0) = direct_to_block_no(bx, 1, DIRECT_SOUTH);
        let next_bnum = d2_to_d1(n_bx0, n_bz0);
        let (n_bx1, n_bz1) = direct_to_block_no(
            n_bx0,
            n_bz0,
            river_idx_to_next_direct(start_type.wrapping_sub(bt::RIVER_SOUTH)),
        );
        let next_next_bnum = d2_to_d1(n_bx1, n_bz1);
        let mut end = false;
        if check_correct_block_no(n_bx0, n_bz0, 1, 5, 1, 6) && cliff[next_bnum] != bt::PLAYER_HOUSE
        {
            if check_block_group(cliff[next_bnum], GROUP_CLIFF) {
                if river_albumin_cliff(cliff[next_bnum], bt::RIVER_SOUTH) == BLOCK_TYPE_NONE {
                    return None;
                }
                if cliff[d2_to_d1(n_bx0, n_bz0 + 1)] != bt::PLAYER_HOUSE {
                    river[next_bnum] = bt::RIVER_SOUTH;
                    end = true;
                }
            } else if cliff[next_next_bnum] != bt::PLAYER_HOUSE {
                river[next_bnum] = start_type;
                end = true;
            }
        }
        if end {
            river[d2_to_d1(bx, 0)] = bt::BORDER_CLIFF_RIVER;
            river[d2_to_d1(bx, 1)] = bt::TRACKS_RIVER;
            return Some((n_bx0, n_bz0));
        }
    }
}

fn last_check_river(river: &Blocks, bx: i32, bz: i32) -> bool {
    let (mut x, mut z) = (bx, bz);
    let mut t = river[d2_to_d1(bx, bz)];
    while check_block_group(t, GROUP_RIVER) {
        let direct = river_idx_to_next_direct(t - bt::RIVER_SOUTH);
        (x, z) = direct_to_block_no(x, z, direct);
        t = river[d2_to_d1(x, z)];
    }
    z > 6 // the river must end at the beach
}

fn decide_base_river(river: &mut Blocks, cliff: &Blocks, rng: &mut impl FieldRng) -> bool {
    *river = *cliff;
    let Some((start_bx, start_bz)) = trace_river_part1(river, cliff, rng) else {
        return false;
    };
    trace_river_part2(river, cliff, rng, start_bx, start_bz)
        && last_check_river(river, start_bx, start_bz)
}

fn set_random_block_data(river: &mut Blocks, cliff: &mut Blocks, rng: &mut impl FieldRng) -> bool {
    decide_base_cliff(cliff, rng) && decide_base_river(river, cliff, rng)
}

/// Row of the first acre where the river crosses a cliff.
fn river_cross_cliff_row(blocks: &Blocks) -> i32 {
    blocks
        .iter()
        .take((BLOCK_Z_NUM - 2) * BLOCK_X_NUM)
        .position(|b| CROSS_DATA.contains(b))
        .map_or(0, |i| (i / BLOCK_X_NUM) as i32)
}

// mRF_BIT_*
pub const BIT_SLOPE_LEFT: i32 = 1 << 0;
pub const BIT_SLOPE_RIGHT: i32 = 1 << 1;
pub const BIT_BRIDGE_UPPER: i32 = 1 << 2;
pub const BIT_BRIDGE_LOWER: i32 = 1 << 3;
pub const BIT_SHRINE: i32 = 1 << 4;
pub const BIT_POLICE: i32 = 1 << 5;
pub const BIT_MUSEUM: i32 = 1 << 6;
pub const BIT_POOL: i32 = 1 << 7;
pub const BIT_NEEDLEWORK: i32 = 1 << 8;

const BRIDGE_DELTA: u8 = bt::RIVER_SOUTH_BRIDGE - bt::RIVER_SOUTH;

/// Turns the `selected`-th river acre matching `in_zone` into a bridge.
fn bridge_nth(blocks: &mut Blocks, in_zone: impl Fn(i32) -> bool, selected: usize) {
    let rows = (BLOCK_Z_NUM - 2) * BLOCK_X_NUM;
    let mut n = 0;
    for i in 0..rows {
        if check_block_group(blocks[i], GROUP_RIVER) && in_zone((i / BLOCK_X_NUM) as i32) {
            if n == selected {
                blocks[i] += BRIDGE_DELTA;
            }
            n += 1;
        }
    }
}

fn set_bridge_block(blocks: &mut Blocks, stepmode: i32, rng: &mut impl FieldRng) -> i32 {
    let mut flags = 0;
    let coin = rng.random(10) & 1;
    let cross_bz = river_cross_cliff_row(blocks);
    let rows = (BLOCK_Z_NUM - 2) * BLOCK_X_NUM;
    let count = |zone: &dyn Fn(i32) -> bool| {
        (0..rows)
            .filter(|&i| {
                check_block_group(blocks[i], GROUP_RIVER) && zone((i / BLOCK_X_NUM) as i32)
            })
            .count()
    };
    let before = count(&|z| z < cross_bz);
    let after = count(&|z| z > cross_bz);
    if before != 0 {
        let selected = rng.random(before as i32) as usize;
        bridge_nth(blocks, |z| z < cross_bz, selected);
        flags |= BIT_BRIDGE_UPPER;
    }
    if after != 0 && stepmode == STEPMODE_TWO && coin != 0 {
        let selected = rng.random(after as i32) as usize;
        bridge_nth(blocks, |z| z > cross_bz, selected);
        flags |= BIT_BRIDGE_LOWER;
    }
    flags
}

// mRF_FIELD_STEP*
pub const FIELD_STEP1: u8 = 0;
pub const FIELD_STEP2: u8 = 1;
pub const FIELD_STEP3: u8 = 2;

/// `mRF_GetBlockBase`: height tier per acre, counted up from the south;
/// each cliff edge crossed going north adds a tier.
fn get_block_base(base: &mut Blocks, blocks: &Blocks) {
    for bx in 0..BLOCK_X_NUM as i32 {
        let mut height = FIELD_STEP1;
        for bz in (0..BLOCK_Z_NUM as i32).rev() {
            let t = blocks[d2_to_d1(bx, bz)];
            base[d2_to_d1(bx, bz)] = height;
            let info = system_block_info(t);
            if info & (CLIFF_HORIZONTAL | CLIFF_TOP_RIGHT | CLIFF_TOP_LEFT) != 0
                || t == bt::BORDER_CLIFF_LEFT_TRANSITION
                || t == bt::BORDER_CLIFF_RIGHT_TRANSITION
            {
                height += 1;
            }
        }
    }
}

/// One `data_combi_table` entry (`mFM_combo_info_c`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComboInfo {
    /// `BG_TYPE_*`: which ground template (matches `data_bgd` entries).
    pub bg_id: u16,
    /// Object layout id.
    pub fg_id: u16,
    /// `mFM_BLOCK_TYPE_*` this combination can stand in for (255 = none).
    pub block_type: u8,
}

/// One acre of the result (`mFM_combination_c`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Combination {
    /// Index into the combination table.
    pub combi: u16,
    /// Height tier (`FIELD_STEP*`).
    pub height: u8,
}

/// `BLOCK_COMBI_GRD_S_F_7`: fallback when no combination fits.
pub const COMBI_GRD_S_F_7: u16 = 161;
const BG_TYPE_NUM: u16 = 293;

/// `exceptional_table` in `mRF_GetExceptionalSeaBgDownBgName`: the sea
/// acre to use below a given beach acre (`BG_TYPE_*` pairs).
const EXCEPTIONAL_SEA: [(u16, u16); 26] = [
    (80, 81),
    (85, 86),
    (120, 144),
    (121, 145),
    (122, 146),
    (123, 147),
    (124, 148),
    (125, 149),
    (126, 150),
    (127, 151),
    (128, 152),
    (129, 153),
    (130, 156),
    (131, 157),
    (132, 158),
    (133, 159),
    (134, 160),
    (135, 161),
    (136, 162),
    (137, 163),
    (138, 164),
    (139, 165),
    (140, 166),
    (141, 167),
    (142, 168),
    (143, 169),
];

fn exceptional_sea_bg_down_bg_name(sea_bg: u16) -> u16 {
    EXCEPTIONAL_SEA
        .iter()
        .take_while(|(from, _)| *from != BG_TYPE_NUM)
        .find(|(from, _)| *from == sea_bg)
        .map_or(sea_bg, |(_, to)| *to)
}

/// `mRF_BgName2RandomConbiNo`. Retail calls `RANDOM(0)` here (a known bug),
/// so the first matching combination always wins; kept as-is.
fn bg_name_to_combi_no(bg: u16, combos: &[ComboInfo], rng: &mut impl FieldRng) -> u16 {
    if combos
        .iter()
        .any(|c| c.bg_id == bg && c.block_type != BLOCK_TYPE_NONE)
    {
        rng.random(0); // retail's RANDOM(0): consumes a number, always 0
    }
    combos
        .iter()
        .position(|c| c.bg_id == bg && c.block_type != BLOCK_TYPE_NONE)
        .unwrap_or(combos.len()) as u16
}

/// Combinations of `block_type`, skipping already-used ones unless `reuse`.
fn type_combos<'a>(
    combos: &'a [ComboInfo],
    block_type: u8,
    used: &'a [i16; BLOCK_TOTAL_NUM],
    reuse: bool,
) -> impl Iterator<Item = usize> + 'a {
    combos
        .iter()
        .enumerate()
        .filter(move |(i, c)| c.block_type == block_type && (reuse || !used.contains(&(*i as i16))))
        .map(|(i, _)| i)
}

fn pick_combo(
    combos: &[ComboInfo],
    block_type: u8,
    used: &[i16; BLOCK_TOTAL_NUM],
    reuse: bool,
    rng: &mut impl FieldRng,
) -> Option<usize> {
    let count = type_combos(combos, block_type, used, reuse).count();
    let selected = rng.random(count as i32) as usize;
    type_combos(combos, block_type, used, reuse).nth(selected)
}

/// `mRF_SelectBlock`: choose a combination (template) for every acre.
fn select_block(
    table: &mut [Combination; BLOCK_TOTAL_NUM],
    combos: &[ComboInfo],
    blocks: &Blocks,
    rng: &mut impl FieldRng,
) {
    let mut used = [-1i16; BLOCK_TOTAL_NUM];
    for bz in 0..BLOCK_Z_NUM as i32 {
        for bx in 0..BLOCK_X_NUM as i32 {
            let n = d2_to_d1(bx, bz);
            let t = blocks[n];
            if t == bt::SEA_EXCEPTIONAL {
                let above = table[d2_to_d1(bx, bz - 1)].combi as usize;
                let above_bg = combos.get(above).map_or(0, |c| c.bg_id);
                table[n].combi =
                    bg_name_to_combi_no(exceptional_sea_bg_down_bg_name(above_bg), combos, rng);
                continue;
            }
            let fresh = if type_combos(combos, t, &used, false).count() != 0 {
                pick_combo(combos, t, &used, false, rng)
            } else {
                None
            };
            match fresh.or_else(|| pick_combo(combos, t, &used, true, rng)) {
                Some(i) => {
                    table[n].combi = i as u16;
                    used[n] = i as i16;
                }
                None => table[n].combi = COMBI_GRD_S_F_7,
            }
        }
    }
}

fn make_base_landform_step2(cliff: &mut Blocks, river: &mut Blocks, rng: &mut impl FieldRng) {
    *cliff = L_BASE_BLOCKS;
    *river = L_BASE_BLOCKS;
    while !set_random_block_data(river, cliff, rng) {
        *cliff = L_BASE_BLOCKS;
        *river = L_BASE_BLOCKS;
    }
    decide_river_albumin_cliff(cliff, river);
}

// mRF_RIVER_SIDE_* / mRF_CLIFF_HEIGHT_*
const SIDE_LEFT: i32 = 0;
const SIDE_RIGHT: i32 = 1;
const SIDE_BOTH: i32 = 2;
const HEIGHT_ABOVE: i32 = 0;
const HEIGHT_BELOW: i32 = 1;
const HEIGHT_BOTH: i32 = 2;

/// Walks the cliff from its west end; calls `on_piece(acre, cliff index)`
/// for each piece on `half` of the river until it returns true.
fn walk_cliff_half(
    blocks: &Blocks,
    bz: i32,
    half: i32,
    mut on_piece: impl FnMut(usize, i32) -> bool,
) -> bool {
    let (mut x, mut z) = (1, bz);
    let mut side = SIDE_LEFT;
    let mut t = blocks[d2_to_d1(x, z)];
    while t != bt::BORDER_CLIFF_RIGHT_TRANSITION {
        let idx = block_type_to_cliff_index(t);
        if check_block_group(t, GROUP_RIVER_CLIFF_ANY) {
            side = SIDE_RIGHT; // past the river: the town's right half
        } else if side == half && on_piece(d2_to_d1(x, z), idx) {
            return true;
        }
        if idx < 0 {
            return false; // malformed cliff; retail would read out of bounds
        }
        (x, z) = direct_to_block_no(x, z, L_CLIFF_NEXT_DIRECT[idx as usize]);
        t = blocks[d2_to_d1(x, z)];
    }
    false
}

fn set_slope_block(blocks: &mut Blocks, rng: &mut impl FieldRng) -> i32 {
    let mut flags = 0;
    for bz in 0..(BLOCK_Z_NUM as i32 - 2) {
        if blocks[d2_to_d1(0, bz)] != bt::BORDER_CLIFF_LEFT_TRANSITION {
            continue;
        }
        for (half, bit) in [(SIDE_LEFT, BIT_SLOPE_LEFT), (SIDE_RIGHT, BIT_SLOPE_RIGHT)] {
            let mut count = 0;
            walk_cliff_half(blocks, bz, half, |_, _| {
                count += 1;
                false
            });
            if count == 0 {
                continue;
            }
            let selected = rng.random(count);
            let mut n = 0;
            let mut target = None;
            walk_cliff_half(blocks, bz, half, |acre, idx| {
                if n == selected {
                    target = Some((acre, idx));
                    return true;
                }
                n += 1;
                false
            });
            if let Some((acre, idx)) = target {
                blocks[acre] = bt::SLOPE_HORIZONTAL + idx as u8;
                flags |= bit;
            }
        }
    }
    flags
}

/// `mRF_river_left_right_info` / `mRF_cliff_up_down_info`.
struct FlatInfo {
    river_side: [i32; BLOCK_TOTAL_NUM],
    cliff_height: [i32; BLOCK_TOTAL_NUM],
}

fn make_flat_place_information(blocks: &Blocks) -> FlatInfo {
    let mut info = FlatInfo {
        river_side: [SIDE_BOTH; BLOCK_TOTAL_NUM],
        cliff_height: [HEIGHT_BOTH; BLOCK_TOTAL_NUM],
    };
    for bx in 1..(BLOCK_X_NUM as i32 - 1) {
        let mut side = HEIGHT_ABOVE;
        for bz in 1..(BLOCK_Z_NUM as i32 - 1) {
            let n = d2_to_d1(bx, bz);
            if side == HEIGHT_ABOVE && check_block_group(blocks[n], GROUP_CLIFF_ANY) {
                side = HEIGHT_BELOW;
            }
            info.cliff_height[n] = side;
        }
    }
    for bz in 1..(BLOCK_Z_NUM as i32 - 1) {
        let mut side = SIDE_LEFT;
        for bx in 1..(BLOCK_X_NUM as i32 - 1) {
            let n = d2_to_d1(bx, bz);
            let t = blocks[n];
            if side == SIDE_LEFT
                && (check_block_group(t, GROUP_RIVER)
                    || check_block_group(t, GROUP_RIVER_CLIFF_ANY))
            {
                side = SIDE_RIGHT;
            }
            info.river_side[n] = side;
        }
    }
    info
}

fn judge_flat_block(
    blocks: &Blocks,
    info: &FlatInfo,
    n: usize,
    river_side: i32,
    cliff_height: i32,
) -> bool {
    if n >= (BLOCK_Z_NUM - 2) * BLOCK_X_NUM || blocks[n] != bt::FLAT {
        return false;
    }
    // Every branch of the decomp's nested ifs reduces to this.
    info.cliff_height[n] == cliff_height
        && (river_side == SIDE_BOTH || info.river_side[n] == river_side)
}

fn flat_block_to_unique(
    blocks: &mut Blocks,
    info: &FlatInfo,
    unique: u8,
    river_side: i32,
    cliff_height: i32,
    rng: &mut impl FieldRng,
) -> bool {
    let rows = (BLOCK_Z_NUM - 2) * BLOCK_X_NUM;
    let flats: Vec<usize> = (0..rows)
        .filter(|&n| judge_flat_block(blocks, info, n, river_side, cliff_height))
        .collect();
    if flats.is_empty() {
        return false;
    }
    blocks[flats[rng.random(flats.len() as i32) as usize]] = unique;
    true
}

fn set_needlework_and_wharf_block(blocks: &mut Blocks, rng: &mut impl FieldRng) -> i32 {
    let needlework_bx = rng.random(3);
    let wharf = d2_to_d1(5, 6);
    if blocks[wharf] != bt::BEACH {
        return 0;
    }
    blocks[wharf] = bt::PORT;
    let mut flags = 0;
    let mut bx = 0;
    for i in 1..(BLOCK_X_NUM as i32 - 1) {
        let n = d2_to_d1(i, 6);
        if blocks[n] == bt::BEACH {
            if needlework_bx == bx {
                blocks[n] = bt::NEEDLEWORK;
                flags |= BIT_NEEDLEWORK;
            }
            bx += 1;
        }
    }
    flags
}

fn set_unique_flat_block(blocks: &mut Blocks, info: &FlatInfo, rng: &mut impl FieldRng) -> i32 {
    let mut flags = 0;
    let side0 = rng.random(100) & 1;
    let side1 = (side0 ^ 1) & 1;
    // Wishing well: one side of the river, below the cliff.
    if flat_block_to_unique(blocks, info, bt::SHRINE, side0, HEIGHT_BELOW, rng)
        || flat_block_to_unique(blocks, info, bt::SHRINE, side1, HEIGHT_BELOW, rng)
    {
        flags |= BIT_SHRINE;
    }
    // Police box prefers the other side from the well.
    if flat_block_to_unique(blocks, info, bt::POLICE_BOX, side1, HEIGHT_BELOW, rng)
        || flat_block_to_unique(blocks, info, bt::POLICE_BOX, side0, HEIGHT_BELOW, rng)
    {
        flags |= BIT_POLICE;
    }
    if flat_block_to_unique(blocks, info, bt::MUSEUM, SIDE_BOTH, HEIGHT_BELOW, rng) {
        flags |= BIT_MUSEUM;
    }
    flags
}

fn set_unique_rail_block(blocks: &mut Blocks, rng: &mut impl FieldRng) {
    let (t0, t1) = if rng.random(1000) & 1 != 0 {
        (bt::TRACKS_SHOP, bt::TRACKS_POST_OFFICE)
    } else {
        (bt::TRACKS_POST_OFFICE, bt::TRACKS_SHOP)
    };
    // The retail loops spin until a dump track acre turns up; the base
    // layouts always have one in each pair, but don't loop forever here.
    for (base, t) in [(1, t0), (4, t1)] {
        for _ in 0..1000 {
            let n = d2_to_d1(base + rng.random(2), 1);
            if blocks[n] == bt::TRACKS_DUMP {
                blocks[n] = t;
                break;
            }
        }
    }
}

fn is_pure_river(t: u8) -> bool {
    (bt::RIVER_SOUTH..=bt::RIVER_WEST_SOUTH).contains(&t)
}

fn set_pool_block(blocks: &mut Blocks, rng: &mut impl FieldRng) -> i32 {
    let rows = (BLOCK_Z_NUM - 2) * BLOCK_X_NUM;
    let rivers: Vec<usize> = (0..rows).filter(|&i| is_pure_river(blocks[i])).collect();
    if rivers.is_empty() {
        return 0;
    }
    let i = rivers[rng.random(rivers.len() as i32) as usize];
    blocks[i] += bt::POOL_SOUTH - bt::RIVER_SOUTH;
    BIT_POOL
}

fn set_marin_block(blocks: &mut Blocks) {
    for i in 1..(BLOCK_X_NUM as i32 - 1) {
        let n = d2_to_d1(i, 6);
        if blocks[n] == bt::FLAT {
            blocks[n] = bt::BEACH;
        } else if blocks[n] == bt::RIVER_SOUTH {
            blocks[n] = bt::BEACH_RIVER;
        }
    }
    blocks[d2_to_d1(0, 6)] = bt::BORDER_CLIFF_OCEAN_LEFT;
    blocks[d2_to_d1(6, 6)] = bt::BORDER_CLIFF_OCEAN_RIGHT;
}

fn set_sea_block_with_bridge_river(blocks: &mut Blocks, flags: i32) -> i32 {
    if flags & BIT_BRIDGE_LOWER != 0 {
        return 0;
    }
    let rows = (BLOCK_Z_NUM - 2) * BLOCK_X_NUM;
    match blocks[..rows].iter().position(|&b| b == bt::BEACH_RIVER) {
        Some(i) => {
            blocks[i] = bt::BEACH_RIVER_BRIDGE;
            BIT_BRIDGE_LOWER
        }
        None => 0,
    }
}

const PERFECT_BIT: i32 = (1 << 9) - 1;

/// The generated town.
#[derive(Clone, Debug)]
pub struct RandomField {
    /// `mFM_BLOCK_TYPE_*` per acre, row-major 7x10.
    pub blocks: Blocks,
    /// Chosen combination (template) and height tier per acre.
    pub combi: [Combination; BLOCK_TOTAL_NUM],
    /// 3-tier town (`mRF_STEPMODE_THREE`) instead of 2.
    pub three_step: bool,
}

/// `mRF_MakeRandomField_ovl`: generates a town. Retries the whole layout
/// until every feature (slopes, bridges, buildings, pond) was placed.
pub fn make_random_field(combos: &[ComboInfo], rng: &mut impl FieldRng) -> RandomField {
    let stepmode = if rng.random(100) < 15 {
        STEPMODE_THREE
    } else {
        STEPMODE_TWO
    };
    let mut cliff = [0u8; BLOCK_TOTAL_NUM];
    let mut combi = [Combination::default(); BLOCK_TOTAL_NUM];
    let mut bit = 0;
    while bit & PERFECT_BIT != PERFECT_BIT {
        bit = 0;
        combi = [Combination {
            combi: 1,
            height: FIELD_STEP1,
        }; BLOCK_TOTAL_NUM];
        if stepmode == STEPMODE_THREE {
            cliff = *STEP3_BLOCKSS[rng.random(STEP3_BLOCKSS.len() as i32) as usize];
        } else {
            // The river layer only matters while the river is traced.
            let mut river = [0u8; BLOCK_TOTAL_NUM];
            make_base_landform_step2(&mut cliff, &mut river, rng);
        }
        let info = make_flat_place_information(&cliff);
        set_marin_block(&mut cliff);
        bit |= set_bridge_block(&mut cliff, stepmode, rng);
        bit |= set_slope_block(&mut cliff, rng);
        bit |= set_needlework_and_wharf_block(&mut cliff, rng);
        bit |= set_unique_flat_block(&mut cliff, &info, rng);
        set_unique_rail_block(&mut cliff, rng);
        bit |= set_pool_block(&mut cliff, rng);
        bit |= set_sea_block_with_bridge_river(&mut cliff, bit);
        let mut base = [FIELD_STEP1; BLOCK_TOTAL_NUM];
        get_block_base(&mut base, &cliff);
        select_block(&mut combi, combos, &cliff, rng);
        for (c, h) in combi.iter_mut().zip(base) {
            c.height = h;
        }
    }
    RandomField {
        blocks: cliff,
        combi,
        three_step: stepmode == STEPMODE_THREE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Small deterministic generator for tests.
    fn lcg(seed: u32) -> impl FnMut(i32) -> i32 {
        let mut s = seed;
        move |max| {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            if max <= 0 {
                0
            } else {
                ((s >> 16) % max as u32) as i32
            }
        }
    }

    /// One combination per block type, so selection always succeeds.
    fn combos() -> Vec<ComboInfo> {
        (0..bt::NUM as u8)
            .map(|t| ComboInfo {
                bg_id: t as u16,
                fg_id: 0,
                block_type: t,
            })
            .collect()
    }

    #[test]
    fn towns_have_all_features() {
        for seed in 0..40 {
            let f = make_random_field(&combos(), &mut lcg(seed));
            let has = |t: u8| f.blocks.contains(&t);
            assert!(has(bt::PLAYER_HOUSE), "seed {seed}");
            assert!(
                has(bt::SHRINE) && has(bt::POLICE_BOX) && has(bt::MUSEUM),
                "seed {seed}"
            );
            assert!(
                has(bt::TRACKS_SHOP) && has(bt::TRACKS_POST_OFFICE),
                "seed {seed}"
            );
            assert!(has(bt::NEEDLEWORK) && has(bt::PORT), "seed {seed}");
            assert!(
                (bt::POOL_SOUTH..=bt::POOL_WEST_SOUTH).any(has),
                "seed {seed}"
            );
            // Height tiers only grow going north.
            for bx in 0..BLOCK_X_NUM {
                for bz in 1..BLOCK_Z_NUM {
                    let up = f.combi[(bz - 1) * BLOCK_X_NUM + bx].height;
                    assert!(up >= f.combi[bz * BLOCK_X_NUM + bx].height, "seed {seed}");
                }
            }
        }
    }

    #[test]
    fn same_seed_same_town() {
        let a = make_random_field(&combos(), &mut lcg(7));
        let b = make_random_field(&combos(), &mut lcg(7));
        assert_eq!(a.blocks, b.blocks);
        assert_eq!(a.combi, b.combi);
    }

    #[test]
    fn selection_uses_matching_combos() {
        let f = make_random_field(&combos(), &mut lcg(3));
        for (i, c) in f.combi.iter().enumerate() {
            if f.blocks[i] != bt::SEA_EXCEPTIONAL {
                assert_eq!(c.combi as u8, f.blocks[i]);
            }
        }
    }
}
