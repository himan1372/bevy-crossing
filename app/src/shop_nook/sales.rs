//! Slot-bound Nook offers. Stage every removal, paper remainder and Bell bag.
use crate::interaction::PlayerInventory;

pub(crate) fn eligible(item: u16, condition: u32) -> bool {
    condition <= 3
        && item != 0
        && item >> 8 != 0x21
        && (item >> 8 != 0x2F || rustimal_logic::turnips::quantity(item).is_some())
        && !(0x2523..=0x2530).contains(&item)
        && (rustimal_logic::turnips::quantity(item).is_some()
            || crate::items::sell_price(item).is_some()
            || (crate::items::price(item) == Some(0)
                && !crate::items::name(item).starts_with("item 0x")))
}

/// Native tag marking excludes the whole turnip category on Sundays.
pub(crate) fn markable(item: u16, condition: u32) -> bool {
    eligible(item, condition) && !(item >> 8 == 0x2F && crate::clock::now().weekday == 0)
}

fn paper(item: u16) -> bool {
    (0x2000..=0x20FF).contains(&item)
}
fn quantity(item: u16) -> u32 {
    if paper(item) {
        u32::from((item - 0x2000) / 64) + 1
    } else {
        1
    }
}

#[derive(Clone)]
struct Line {
    slot: usize,
    item: u16,
    condition: u32,
    replacement: u16,
}

#[derive(Clone)]
pub(super) struct Sale {
    lines: Vec<Line>,
    pub price: u32,
    turnip_price: Option<u16>,
}

pub(super) enum Payment {
    Wallet,
    Bags,
}

impl Sale {
    pub fn selected(inventory: &PlayerInventory, mask: u16, turnip_price: u16) -> Option<Self> {
        if mask == 0 || mask & !0x7FFF != 0 {
            return None;
        }
        let mut sale = Self {
            lines: Vec::new(),
            price: 0,
            turnip_price: None,
        };
        for slot in 0..15 {
            if mask & (1 << slot) == 0 {
                continue;
            }
            let item = inventory.pockets.pockets[slot];
            let condition = inventory.pockets.item_cond(slot);
            if !eligible(item, condition) {
                return None;
            }
            let price = if let Some(quantity) = rustimal_logic::turnips::quantity(item) {
                if quantity > 0 {
                    sale.turnip_price = Some(turnip_price);
                }
                quantity * u32::from(turnip_price)
            } else {
                crate::items::sell_price(item).unwrap_or(0)
            };
            sale.price = sale.price.checked_add(price)?;
            sale.lines.push(Line {
                slot,
                item,
                condition,
                replacement: 0,
            });
        }
        Some(sale)
    }

    pub fn item(&self) -> u16 {
        self.lines[0].item
    }
    pub fn slots(&self) -> usize {
        self.lines.len()
    }
    pub fn quantity(&self) -> u32 {
        self.lines.iter().map(|l| quantity(l.item)).sum()
    }

    pub fn has_quest(&self) -> bool {
        self.lines.iter().any(|l| l.condition & 2 != 0)
    }

    pub fn fresh_turnips(&self) -> bool {
        self.turnip_price.is_some()
    }

    pub fn at_price(&self, price: u16) -> bool {
        self.turnip_price.is_none_or(|quoted| quoted == price)
    }

    /// aNSC_check_buy_*: duplicates exclude quest and wrapped slots. Paper
    /// compares design across all stack sizes, then offers one or all sheets.
    pub fn duplicates(&self, inventory: &PlayerInventory) -> Option<Self> {
        if self.lines.len() != 1 || self.lines[0].condition != 0 || self.price == 0 {
            return None;
        }
        let item = self.item();
        let mut mask = 0;
        for (slot, &candidate) in inventory.pockets.pockets.iter().enumerate() {
            let same = if paper(item) {
                paper(candidate) && (candidate - 0x2000) % 64 == (item - 0x2000) % 64
            } else {
                item == candidate
            };
            if same && inventory.pockets.item_cond(slot) == 0 && eligible(candidate, 0) {
                mask |= 1 << slot;
            }
        }
        Self::selected(inventory, mask, self.turnip_price.unwrap_or(0)).filter(|s| s.quantity() > 1)
    }

    pub fn one(mut self) -> Option<Self> {
        if self.lines.len() != 1 {
            return None;
        }
        let line = &mut self.lines[0];
        if paper(line.item) {
            let base = 0x2000 + (line.item - 0x2000) % 64;
            self.price = crate::items::sell_price(base)?;
            if quantity(line.item) > 1 {
                line.replacement = line.item - 64;
            }
        }
        Some(self)
    }

    pub fn valid(&self, inventory: &PlayerInventory) -> bool {
        (!self.fresh_turnips() || crate::clock::now().weekday != 0)
            && self.lines.iter().all(|l| {
                inventory.pockets.pockets[l.slot] == l.item
                    && inventory.pockets.item_cond(l.slot) == l.condition
                    && eligible(l.item, l.condition)
            })
    }

    pub fn commit(&self, inventory: &mut PlayerInventory) -> Option<Payment> {
        if self.has_quest() || self.price == 0 || !self.valid(inventory) {
            return None;
        }
        let mut bells = inventory.bells.checked_add(self.price)?;
        let mut payment = Payment::Wallet;
        let mut pockets = inventory.pockets.clone();
        for line in &self.lines {
            pockets.pockets[line.slot] = line.replacement;
            pockets.set_item_cond(line.slot, 0);
        }
        while bells >= 99999 {
            let empty = self
                .lines
                .iter()
                .map(|l| l.slot)
                .find(|&s| pockets.pockets[s] == 0)
                .or_else(|| pockets.pockets.iter().position(|&i| i == 0));
            let empty = empty?;
            pockets.pockets[empty] = 0x2102;
            pockets.set_item_cond(empty, 0);
            bells -= 30000;
            payment = Payment::Bags;
        }
        inventory.pockets = pockets;
        inventory.bells = bells;
        Some(payment)
    }

    /// aNSC_receive_check: no payment, ledger credit or wallet bag conversion.
    pub fn discard(&self, inventory: &mut PlayerInventory) -> bool {
        if self.has_quest() || self.price != 0 || !self.valid(inventory) {
            return false;
        }
        let mut pockets = inventory.pockets.clone();
        for line in &self.lines {
            pockets.pockets[line.slot] = 0;
            pockets.set_item_cond(line.slot, 0);
        }
        inventory.pockets = pockets;
        true
    }
}
