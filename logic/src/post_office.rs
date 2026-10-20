//! The post office counter: ac_npc_post_girl's talk numbers and the loan
//! repay submenu (m_repay_ovl), per original frame and button press.
//!
//! The clerk is Pelly by day and Phyllis from 19:00 to 07:00
//! (bg_post_item); every message number adds the clerk index. Repaying is
//! only offered while a loan is owed (aPG_STATUS_DONE_FIRST_JOB, the first
//! job being over); with no loan and a bank account the slot is savings.

use crate::inventory::Inventory;

/// `post_girl_npc_type`: Phyllis from 19:00 to 07:00.
pub fn night_clerk(day_secs: u32) -> bool {
    let secs = day_secs % 86400;
    !(7 * 3600..19 * 3600).contains(&secs)
}

/// mBGM: post office music, 62 at night (Phyllis) else 41.
pub fn music(night: bool) -> u8 {
    if night {
        62
    } else {
        41
    }
}

/// `aPG_STATUS_*` bits.
pub const STATUS_DESK_FULL: u8 = 1;
pub const STATUS_DONE_FIRST_JOB: u8 = 2;
pub const STATUS_HAS_BANK: u8 = 4;

/// aPG_set_post_status. `desk_full` is five letters held at the counter;
/// `first_job_done` is !mEv_CheckFirstJob.
pub fn status(desk_full: bool, loan: u32, first_job_done: bool, has_bank: bool) -> u8 {
    let mut status = 0;
    if desk_full {
        status |= STATUS_DESK_FULL;
    }
    if loan != 0 {
        if first_job_done {
            status |= STATUS_DONE_FIRST_JOB;
        }
    } else if has_bank {
        status |= STATUS_HAS_BANK;
    }
    status
}

/// aPG_set_talk_info: the greeting (which asks the business question).
pub fn greeting(status: u8, clerk: u8) -> usize {
    const MSG: [usize; 8] = [0x31d3, 0x31d1, 0x08b1, 0x8ad, 0x8d1, 0x8cf, 0x8b1, 0x8ad];
    MSG[status as usize & 7] + clerk as usize
}

/// aPG_repay_after_init: the line after the repay window closes.
pub fn after_repay(status: u8) -> usize {
    const MSG: [usize; 8] = [0x31d7, 0x31d5, 0x8db, 0x8d9, 0x8df, 0x8dd, 0x8db, 0x8d9];
    MSG[status as usize & 7]
}

/// aPG_repay_before: the message that opens the repay window.
pub fn repay_prompt(clerk: u8) -> usize {
    0x8d3 + clerk as usize
}

/// aPG_repay_menu_close_wait: nothing paid, paid off, or paid in part.
pub fn repay_result(loan_before: u32, loan_after: u32, status_after: u8) -> usize {
    if loan_before == loan_after {
        0x8e5
    } else if status_after & STATUS_DONE_FIRST_JOB != 0 {
        0x8d5
    } else {
        0x8d7
    }
}

/// What a menu answer leads to (aPG_ask_for_business).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Business {
    SendLetter,
    DeskFull,
    Repay,
    Deposit,
    Card,
    Quit,
}

/// aPG_ask_for_business: without a loan or bank account the repay/deposit
/// answer is absent, so later answers shift up by one.
pub fn business(status: u8, choice: usize) -> Business {
    let choice = if matches!(status, 0 | STATUS_DESK_FULL) && choice > 0 {
        choice + 1
    } else {
        choice
    };
    match choice {
        0 if status & STATUS_DESK_FULL != 0 => Business::DeskFull,
        0 => Business::SendLetter,
        1 if status & STATUS_HAS_BANK != 0 => Business::Deposit,
        1 => Business::Repay,
        2 => Business::Card,
        _ => Business::Quit,
    }
}

/// aPG_set_loan_balance: FREE_STR1 is the millions with a comma (empty
/// below a million), FREE_STR2 the rest with thousands separators.
pub fn loan_strings(loan: u32) -> (String, String) {
    let millions = loan / 1_000_000;
    let rest = loan % 1_000_000;
    let rest = if millions != 0 {
        format!("{:03},{:03}", rest / 1000, rest % 1000)
    } else {
        group(rest)
    };
    let millions = if millions != 0 {
        format!("{millions},")
    } else {
        String::new()
    };
    (millions, rest)
}

fn group(v: u32) -> String {
    if v >= 1000 {
        format!("{},{:03}", v / 1000, v % 1000)
    } else {
        v.to_string()
    }
}

/// `mRP_money_name` / `mRP_money_amount`: 100, 1,000, 10,000, 30,000 bags.
pub const MONEY_BAGS: [(u16, u32); 4] = [
    (0x2103, 100),
    (0x2100, 1000),
    (0x2101, 10000),
    (0x2102, 30000),
];

/// `mRP_CURSOR_*`: six digits, then OK.
pub const CURSOR_OK: usize = 6;
const STEP: [u32; 7] = [100000, 10000, 1000, 100, 10, 1, 0];

/// One controller press in the repay window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepayInput {
    Left,
    Right,
    Up,
    Down,
    A,
    B,
    Start,
}

/// What the press did (and which sound the original plays).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepayEvent {
    None,
    /// NA_SE_CURSOL.
    Cursor,
    /// 0x426.
    Changed,
    /// 0x1003: at a limit.
    Limit,
    /// NA_SE_MENU_EXIT, nothing written.
    Cancel,
    /// NA_SE_MENU_EXIT; [`RepayMenu::commit`] writes the payment.
    Confirm,
}

/// `mRP_Ovl_c`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RepayMenu {
    /// Money left after the chosen amount (wallet plus bags).
    pub money: u32,
    pub max_money: u32,
    /// Loan left after the chosen amount.
    pub loan: u32,
    pub loan_at_open: u32,
    pub amount: u32,
    pub max_repay: u32,
    pub cursor: usize,
}

impl RepayMenu {
    /// mRP_repay_ovl_init.
    pub fn open(wallet: u32, loan: u32, pockets: &Inventory) -> Self {
        let mut money = wallet;
        for slot in 0..pockets.pockets.len() {
            if pockets.item_cond(slot) == 0 {
                if let Some(&(_, value)) = MONEY_BAGS
                    .iter()
                    .find(|(item, _)| *item == pockets.pockets[slot])
                {
                    money += value;
                }
            }
        }
        Self {
            money,
            max_money: money,
            loan,
            loan_at_open: loan,
            amount: 0,
            max_repay: money.min(loan),
            cursor: 0,
        }
    }

    /// mRP_move_Play.
    pub fn press(&mut self, input: RepayInput) -> RepayEvent {
        use RepayInput::*;
        if input == B {
            return RepayEvent::Cancel;
        }
        if self.cursor >= CURSOR_OK || input == Start {
            return match input {
                A | Start => RepayEvent::Confirm,
                Up | Left => {
                    self.cursor -= 1;
                    RepayEvent::Cursor
                }
                _ => RepayEvent::None,
            };
        }
        match input {
            Left if self.cursor != 0 => {
                self.cursor -= 1;
                RepayEvent::Cursor
            }
            Right => {
                self.cursor += 1;
                RepayEvent::Cursor
            }
            Up | Down => {
                let step = STEP[self.cursor];
                let mut amount = self.amount;
                if input == Down {
                    if amount < step {
                        if amount == 0 {
                            return RepayEvent::Limit;
                        }
                        amount = 0;
                    } else {
                        amount -= step;
                    }
                } else {
                    if amount == self.max_repay {
                        return RepayEvent::Limit;
                    }
                    amount = (amount + step).min(self.max_repay);
                }
                self.money = self.max_money - amount;
                self.loan = self.loan_at_open - amount;
                self.amount = amount;
                RepayEvent::Changed
            }
            _ => RepayEvent::None,
        }
    }

    /// The Confirm branch: returns the new loan and wallet and removes the
    /// bags spent (mRP_money_repay). A zero amount writes nothing.
    pub fn commit(&self, wallet: u32, pockets: &mut Inventory) -> Option<(u32, u32)> {
        if self.amount == 0 {
            return None;
        }
        let wallet = if wallet >= self.amount {
            wallet - self.amount
        } else {
            let mut total = wallet;
            let mut left = 0;
            'bags: for &(item, value) in &MONEY_BAGS {
                for slot in 0..pockets.pockets.len() {
                    if pockets.item_cond(slot) == 0 && pockets.pockets[slot] == item {
                        total += value;
                        pockets.pockets[slot] = crate::inventory::EMPTY_NO;
                        if total >= self.amount {
                            left = total - self.amount;
                            break 'bags;
                        }
                    }
                }
            }
            left
        };
        Some((self.loan, wallet))
    }
}
