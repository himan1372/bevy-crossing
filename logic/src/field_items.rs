//! Town field items that change from day to day: the native fruit trees
//! and the shells the waves leave on the beach.
//!
//! Verified against `m_all_grow.c` (`mAGrw_ChangeTree2FruitTree`,
//! `mAGrw_ChangeItemBlock`), `m_start_data_init.c` (`decide_fruit`) and
//! `m_field_info.c` (`mFI_SetShellWave` and helpers), USA Rev. 0 decomp.
//!
//! `rand(n)` stands in for the game's `RANDOM(n)`: a value in `0..n`.

/// `EMPTY_NO`.
pub const EMPTY_NO: u16 = 0x0000;
/// `ENV_START + 4`: a fully grown plain tree.
pub const TREE: u16 = 0x0804;
/// `ITM_FOOD_START`: apple, cherry, pear, peach, orange, ..., coconut.
pub const ITM_FOOD_START: u16 = 0x2800;
/// `ITM_FOOD_COCONUT`.
pub const ITM_FOOD_COCONUT: u16 = ITM_FOOD_START + 7;
/// `ITM_SHELL0` (`ITM_ETC_START + 20`) .. `ITM_SHELL7`.
pub const ITM_SHELL0: u16 = 0x2514;
pub const ITM_SHELL7: u16 = ITM_SHELL0 + 7;
/// `mFI_SET_SHELL_NUM`: shells reserved when a game starts.
pub const SET_SHELL_NUM: i32 = 20;
/// `mFI_MAX_SHELLS_PER_BLOCK`.
pub const MAX_SHELLS_PER_BLOCK: u8 = 4;

/// `mAGrw_FRUIT_NUM`.
const FRUIT_NUM: i32 = 5;

/// `l_tree_max_table`: the fruiting tree for each fruit, in `ITM_FOOD`
/// order (apple, cherry, pear, peach, orange).
const TREE_MAX_TABLE: [u16; FRUIT_NUM as usize] = [
    0x080C, // TREE_APPLE_FRUIT
    0x082C, // TREE_CHERRY_FRUIT
    0x0824, // TREE_PEAR_FRUIT
    0x081C, // TREE_PEACH_FRUIT
    0x0814, // TREE_ORANGE_FRUIT
];

/// `decide_fruit`: the town's native fruit (an `ITM_FOOD` item).
pub fn decide_fruit(rand: &mut impl FnMut(i32) -> i32) -> u16 {
    rand(FRUIT_NUM) as u16 | ITM_FOOD_START
}

/// The fruiting tree for a native fruit (`l_tree_max_table`, apple when
/// the fruit is out of range).
pub fn fruit_tree(fruit: u16) -> u16 {
    let i = fruit.wrapping_sub(ITM_FOOD_START) as usize;
    TREE_MAX_TABLE.get(i).copied().unwrap_or(TREE_MAX_TABLE[0])
}

/// `mAGrw_ChangeItemBlock`: replace the `RANDOM(possible_num)`-th
/// `target` in the block with `new_item`.
pub fn change_item_block(
    block: &mut [u16; 256],
    new_item: u16,
    target: u16,
    possible_num: u8,
    rand: &mut impl FnMut(i32) -> i32,
) {
    let mut selected = rand(possible_num as i32);
    for item in block.iter_mut() {
        if *item == target {
            if selected <= 0 {
                *item = new_item;
                return;
            }
            selected -= 1;
        }
    }
}

/// `mAGrw_ChangeTree2FruitTree`: in every row of acres, one plain tree
/// in each of two different acres becomes a fruit tree. `blocks` are the
/// town's acres row by row, `cols` per row.
pub fn change_tree_to_fruit_tree(
    blocks: &mut [[u16; 256]],
    cols: usize,
    fruit: u16,
    rand: &mut impl FnMut(i32) -> i32,
) {
    let tree = fruit_tree(fruit);
    for row in blocks.chunks_mut(cols) {
        let block0 = rand(cols as i32) as usize;
        let mut block1 = rand(cols as i32 - 1) as usize;
        if block1 == block0 {
            block1 += 1;
        }
        for b in [block0, block1] {
            // mAGrw_ChangeTree2OtherBlock(.., change_num = 1)
            let count = row[b].iter().filter(|&&i| i == TREE).count() as u8;
            if count != 0 {
                change_item_block(&mut row[b], tree, TREE, count, rand);
            }
        }
    }
}

/// `mCoBG_CheckWaveAttr`: wave, wave_s, wave_se2, wave_sw2.
pub fn is_wave_attr(attr: u8) -> bool {
    matches!(attr, 11 | 25 | 26 | 36)
}

fn is_shell(item: u16) -> bool {
    (ITM_SHELL0..=ITM_SHELL7).contains(&item) || item == ITM_FOOD_COCONUT
}

/// One sandy-beach acre for [`set_shell_wave`] (`l_sandy_beach_bx/bz`).
pub struct BeachBlock<'a> {
    pub items: &'a mut [u16; 256],
    /// Unit attributes (`mCoBG_Collision_u.unit_attribute`).
    pub attrs: &'a [u8; 256],
    /// The player stands in this acre: it is left alone.
    pub player_here: bool,
    pub island: bool,
}

/// `mFI_GetShell`: a shell for the beach, rare one time in four.
pub fn get_shell(island: bool, rand: &mut impl FnMut(i32) -> i32) -> u16 {
    const NORMAL: [u16; 6] = [
        ITM_SHELL0,
        ITM_SHELL0 + 1,
        ITM_SHELL0 + 2,
        ITM_SHELL0 + 3,
        ITM_SHELL0 + 4,
        ITM_SHELL7,
    ];
    const RARE: [u16; 2] = [ITM_SHELL0 + 6, ITM_SHELL0 + 5];
    const ISLAND_RARE: [u16; 3] = [ITM_SHELL0 + 6, ITM_SHELL0 + 5, ITM_FOOD_COCONUT];
    // shell_max: the island table's 2.0 is the retail bug (no coconuts).
    let tables: [(&[u16], i32); 3] = [(&NORMAL, 6), (&RARE, 2), (&ISLAND_RARE, 2)];
    let mut table = 0;
    if rand(4) == 3 {
        table = if island { 2 } else { 1 };
    }
    let (shells, max) = tables[table];
    shells[rand(max) as usize]
}

/// `mFI_SetShellWave`: wash up to `set_num` shells onto empty wave units
/// of the beach acres, at most [`MAX_SHELLS_PER_BLOCK`] per acre, spread
/// evenly first and then at random. No acre has buried items here, so
/// every shell on the beach counts (`mFI_GetBlockDeposit` is false).
pub fn set_shell_wave(blocks: &mut [BeachBlock], set_num: i32, rand: &mut impl FnMut(i32) -> i32) {
    let n = blocks.len();
    let mut can_set = vec![0u8; n];
    let mut in_block = vec![0u8; n];
    let mut on = vec![0u8; n];
    let (mut total_set, mut total_blocks) = (0i32, 0i32);

    // mFI_ResearchShell
    for (i, b) in blocks.iter().enumerate() {
        if b.player_here {
            continue;
        }
        // mFI_GetCanSetShellNum
        let mut valid = 0u8;
        for (item, &attr) in b.items.iter().zip(b.attrs.iter()) {
            if *item == EMPTY_NO && is_wave_attr(attr) {
                valid += 1;
            } else if is_shell(*item) {
                on[i] += 1;
            }
        }
        can_set[i] = valid;
        if valid != 0 {
            if on[i] < MAX_SHELLS_PER_BLOCK {
                let add = (MAX_SHELLS_PER_BLOCK - on[i]) as i32;
                if add > valid as i32 {
                    on[i] += (add - valid as i32) as u8;
                    total_set += valid as i32;
                } else {
                    total_set += add;
                }
                total_blocks += 1;
            } else {
                can_set[i] = 0;
            }
        }
    }

    let mut set_num = set_num.min(total_set);
    if total_blocks <= 0 || set_num <= 0 {
        return;
    }

    // mFI_DivideShellSameSum
    while set_num > 0 && total_blocks > 0 && set_num > total_blocks {
        for i in 0..n {
            if can_set[i] != 0 && on[i] < MAX_SHELLS_PER_BLOCK {
                in_block[i] += 1;
                on[i] += 1;
                set_num -= 1;
                if on[i] >= MAX_SHELLS_PER_BLOCK {
                    total_blocks -= 1;
                }
            }
        }
    }
    // mFI_DivideShellRandom
    while total_blocks > 0 && set_num > 0 {
        let mut selected = rand(total_blocks);
        for i in 0..n {
            if can_set[i] > 0 && on[i] < MAX_SHELLS_PER_BLOCK {
                if selected <= 0 {
                    in_block[i] += 1;
                    on[i] += 1;
                    if on[i] >= MAX_SHELLS_PER_BLOCK {
                        total_blocks -= 1;
                    }
                    break;
                }
                selected -= 1;
            }
        }
        set_num -= 1;
    }

    // mFI_SetShellSandyBeachBlock / mFI_SetShellBlock
    for (i, b) in blocks.iter_mut().enumerate() {
        let mut shells = in_block[i];
        while shells != 0 && can_set[i] != 0 {
            let mut selected = rand(can_set[i] as i32);
            let slot = (0..256).find(|&u| {
                let free = b.items[u] == EMPTY_NO && is_wave_attr(b.attrs[u]);
                if free {
                    if selected <= 0 {
                        return true;
                    }
                    selected -= 1;
                }
                false
            });
            let Some(u) = slot else { break };
            b.items[u] = get_shell(b.island, rand);
            shells -= 1;
            can_set[i] -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn two_fruit_trees_per_row() {
        let mut block = [EMPTY_NO; 256];
        block[..12].fill(TREE);
        let mut blocks = vec![block; 30];
        let mut rand = lcg(7);
        let fruit = decide_fruit(&mut rand);
        change_tree_to_fruit_tree(&mut blocks, 5, fruit, &mut rand);
        for row in blocks.chunks(5) {
            let changed: Vec<usize> = row
                .iter()
                .map(|b| b.iter().filter(|&&i| i == fruit_tree(fruit)).count())
                .collect();
            assert_eq!(changed.iter().sum::<usize>(), 2);
            assert!(changed.iter().all(|&c| c <= 1));
        }
    }

    #[test]
    fn shells_fill_wave_units_up_to_four_per_acre() {
        let mut items = vec![[EMPTY_NO; 256]; 5];
        let mut attrs = [0u8; 256];
        attrs[200..240].fill(11);
        let mut rand = lcg(3);
        let mut blocks: Vec<BeachBlock> = items
            .iter_mut()
            .enumerate()
            .map(|(i, items)| BeachBlock {
                items,
                attrs: &attrs,
                player_here: i == 2,
                island: false,
            })
            .collect();
        set_shell_wave(&mut blocks, SET_SHELL_NUM, &mut rand);
        set_shell_wave(&mut blocks, 1, &mut rand);
        for (i, b) in items.iter().enumerate() {
            let shells: Vec<usize> = (0..256).filter(|&u| is_shell(b[u])).collect();
            assert!(shells.iter().all(|&u| (200..240).contains(&u)));
            assert_eq!(shells.len(), if i == 2 { 0 } else { 4 });
        }
    }
}
