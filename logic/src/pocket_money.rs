//! Inventory wallet transactions (`mTG_make_money_sack`, `mTG_drop_item`).
use crate::inventory::{Inventory, EMPTY_NO};

pub const WALLET_MAX: u32 = 99_999;

/// mTG_move_func: count by signed amount/42 for 41 native ticks, then
/// snap the remainder to the committed wallet on tick 42.
#[derive(Clone, Copy, Debug)]
pub struct Counter {
    pub displayed: u32,
    target: u32,
    step: i64,
    remaining: u32,
    fraction: f32,
}

impl Counter {
    pub fn new(before: u32, after: u32) -> Self {
        Self {
            displayed: before,
            target: after,
            step: (i64::from(after) - i64::from(before)) / 42,
            remaining: 42,
            fraction: 0.0,
        }
    }

    pub fn active(&self) -> bool {
        self.remaining != 0
    }

    pub fn advance(&mut self, native_ticks: f32) {
        if !self.active() {
            return;
        }
        self.fraction += native_ticks.max(0.0);
        let ticks = (self.fraction as u32).min(self.remaining);
        self.fraction -= ticks as f32;
        self.remaining -= ticks;
        if self.remaining == 0 {
            self.displayed = self.target;
        } else {
            self.displayed = (i64::from(self.displayed) + self.step * i64::from(ticks)) as u32;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    Invalid,
    Capacity,
    Funds,
}

pub fn amount(item: u16) -> Option<u32> {
    match item {
        0x2100 => Some(1000),
        0x2101 => Some(10000),
        0x2102 => Some(30000),
        0x2103 => Some(100),
        _ => None,
    }
}

pub fn deposit(pockets: &mut Inventory, wallet: &mut u32, slot: usize) -> Result<(), Refusal> {
    let item = *pockets.pockets.get(slot).ok_or(Refusal::Invalid)?;
    let value = amount(item).ok_or(Refusal::Invalid)?;
    if pockets.item_cond(slot) != 0 {
        return Err(Refusal::Invalid);
    }
    let next = wallet
        .checked_add(value)
        .filter(|&n| n <= WALLET_MAX)
        .ok_or(Refusal::Capacity)?;
    *wallet = next;
    pockets.pockets[slot] = EMPTY_NO;
    pockets.set_item_cond(slot, 0);
    Ok(())
}

pub fn withdraw(pockets: &mut Inventory, wallet: &mut u32, value: u32) -> Result<usize, Refusal> {
    let item = match value {
        100 => 0x2103,
        1000 => 0x2100,
        10000 => 0x2101,
        30000 => 0x2102,
        _ => return Err(Refusal::Invalid),
    };
    if *wallet < value {
        return Err(Refusal::Funds);
    }
    let slot = pockets
        .pockets
        .iter()
        .position(|&item| item == EMPTY_NO)
        .ok_or(Refusal::Capacity)?;
    pockets.pockets[slot] = item;
    pockets.set_item_cond(slot, 0);
    *wallet -= value;
    Ok(slot)
}
