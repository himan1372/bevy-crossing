//! mTG_TABLE_MONEY: the wallet target and its denomination choices.
use super::*;

pub(crate) const AT: Vec2 = Vec2::new(179.0, 102.0);

impl PocketsUi {
    pub(crate) fn displayed_bells(&self, wallet: u32) -> u32 {
        self.money_counter
            .filter(|counter| counter.active())
            .map_or(wallet, |counter| counter.displayed)
    }
    pub(crate) fn normal_mode(&self) -> bool {
        !self.selling
            && !self.donating
            && !self.sending
            && self.giving.is_none()
            && !self.mailing
            && self.storing.is_none()
            && !self.registering
            && self.room_request.is_none()
            && !self.at_mailbox
    }
}

pub(crate) fn options(bells: u32) -> Vec<&'static str> {
    [
        (100, "100 Bells"),
        (1000, "1,000 Bells"),
        (10000, "10,000 Bells"),
        (30000, "30,000 Bells"),
    ]
    .into_iter()
    .filter(|(amount, _)| *amount <= bells)
    .map(|(_, label)| label)
    .chain(std::iter::once("Quit"))
    .collect()
}

pub(super) fn input(
    ui: &mut PocketsUi,
    inventory: &mut PlayerInventory,
    actions: &mut Actions,
    warning: &mut crate::warning_panel::WarningPanel,
    dx: i32,
    dy: i32,
) -> bool {
    if !ui.normal_mode() || ui.mail_slot.is_some() {
        return false;
    }
    if !ui.on_money {
        let wearable = ui.grabbed.is_some_and(|slot| {
            wearable_shirt(
                inventory.pockets.pockets[slot],
                inventory.pockets.item_cond(slot),
            )
        });
        let can_deposit = ui.grabbed.is_none_or(|slot| {
            inventory.pockets.item_cond(slot) == 0
                && rustimal_logic::pocket_money::amount(inventory.pockets.pockets[slot]).is_some()
        });
        if dy < 0 && ui.selected < 5 && ui.menu.is_none() && !wearable && can_deposit {
            ui.on_money = true;
            crate::sound::se(crate::sound::se_id::CURSOR);
            return true;
        }
        return false;
    }
    if let Some(choice) = ui.money_menu {
        let options = options(inventory.bells);
        if dy != 0 {
            ui.money_menu = Some((choice as i32 + dy).rem_euclid(options.len() as i32) as usize);
            crate::sound::se(crate::sound::se_id::CURSOR);
        } else if actions.consume(Action::Confirm) || actions.consume(Action::Interact) {
            ui.money_menu = None;
            if choice + 1 < options.len() {
                let amount = [100, 1000, 10000, 30000][choice];
                let PlayerInventory { pockets, bells, .. } = inventory;
                let before = *bells;
                let ok = rustimal_logic::pocket_money::withdraw(pockets, bells, amount).is_ok();
                if ok {
                    ui.money_counter =
                        Some(rustimal_logic::pocket_money::Counter::new(before, *bells));
                }
                crate::sound::se(if ok {
                    0x52
                } else {
                    crate::sound::se_id::MENU_EXIT
                });
            }
        }
    } else if dy > 0 || dx != 0 {
        ui.on_money = false;
        crate::sound::se(crate::sound::se_id::CURSOR);
    } else if actions.consume(Action::Confirm) || actions.consume(Action::Interact) {
        if let Some(slot) = ui.grabbed {
            let PlayerInventory { pockets, bells, .. } = inventory;
            let before = *bells;
            if rustimal_logic::pocket_money::deposit(pockets, bells, slot).is_ok() {
                ui.money_counter = Some(rustimal_logic::pocket_money::Counter::new(before, *bells));
                ui.grabbed = None;
                crate::sound::se(0x52);
            } else {
                if pockets.item_cond(slot) == 0
                    && rustimal_logic::pocket_money::amount(pockets.pockets[slot]).is_some()
                {
                    warning.open(&rustimal_logic::warning::MONEY);
                }
                crate::sound::se(crate::sound::se_id::MENU_EXIT);
            }
        } else {
            if inventory.bells >= 100 {
                ui.money_menu = Some(0);
                crate::sound::se(crate::sound::se_id::CHOICE_OPEN);
            } else {
                crate::sound::se(crate::sound::se_id::MENU_EXIT);
            }
        }
    }
    true
}
