//! The player's house record (`mHm_hs_c` size_info and roof palettes):
//! size, ordered upgrade, basement, statue, and the roof colour.
//!
//! mHm_CheckRehouseOrder runs when the game starts or a save is loaded: a
//! pending order from an earlier date is built (size, basement or statue),
//! the ordered roof colour applied, and `renew` set so Nook bills the next
//! loan when the player next visits ([`rustimal_logic::house`]).
use std::sync::atomic::{AtomicU16, Ordering};

use bevy::prelude::*;
use rustimal_logic::house::{HomeSizeInfo, House, HouseSize};
use serde::{Deserialize, Serialize};

use crate::interaction::PlayerInventory;

/// The saved house record.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseSave {
    pub size: u8,
    pub next_size: u8,
    pub statue_rank: u8,
    pub renew: bool,
    pub statue_ordered: bool,
    pub basement_ordered: bool,
    /// `pad_1`: the basement's loan has been billed (gates the upper floor).
    pub basement_billed: bool,
    pub has_basement: bool,
    /// `upgrade_order_date` (year, month, day).
    pub order: (u16, u8, u8),
    /// `outlook_pal` / `next_outlook_pal` / `ordered_outlook_pal` (0-11).
    pub outlook_pal: u8,
    pub next_outlook_pal: u8,
    pub ordered_outlook_pal: u8,
    /// Save `num_statues`: statues already in town (their rank order).
    #[serde(default)]
    pub town_statues: u8,
}

impl HouseSave {
    pub fn info(&self) -> HomeSizeInfo {
        HomeSizeInfo {
            size: HouseSize::from_u8(self.size).unwrap_or_default(),
            next_size: HouseSize::from_u8(self.next_size).unwrap_or_default(),
            statue_rank: self.statue_rank,
            renew: self.renew,
            statue_ordered: self.statue_ordered,
            basement_ordered: self.basement_ordered,
            basement_completion_marker: self.basement_billed,
            has_basement: self.has_basement,
            upgrade_order_ymd: self.order,
            ordered_outlook_pal: self.ordered_outlook_pal,
        }
    }

    pub fn set_info(&mut self, info: &HomeSizeInfo) {
        self.size = info.size as u8;
        self.next_size = info.next_size as u8;
        self.statue_rank = info.statue_rank;
        self.renew = info.renew;
        self.statue_ordered = info.statue_ordered;
        self.basement_ordered = info.basement_ordered;
        self.basement_billed = info.basement_completion_marker;
        self.has_basement = info.has_basement;
        self.order = info.upgrade_order_ymd;
        self.ordered_outlook_pal = info.ordered_outlook_pal;
    }

    /// Run `f` on the logic house (loan included) and write it back.
    pub fn with_house<R>(&mut self, loan: &mut u32, f: impl FnOnce(&mut House) -> R) -> R {
        let mut house = House {
            loan: *loan,
            size_info: self.info(),
        };
        let r = f(&mut house);
        *loan = house.loan;
        self.set_info(&house.size_info);
        r
    }

    /// mHm_CheckRehouseOrder for this house on `today`.
    pub fn check_rehouse_order(&mut self, loan: &mut u32, today: (u16, u8, u8)) -> bool {
        self.outlook_pal = self.next_outlook_pal;
        let main = self.size != self.next_size && self.next_size < HouseSize::Statue as u8;
        let built = self.with_house(loan, |h| h.check_rehouse_order(today));
        if built && main {
            self.outlook_pal = self.ordered_outlook_pal;
            self.next_outlook_pal = self.ordered_outlook_pal;
        }
        built
    }
}

/// `aNSC_MSG_*` numbers (before aNSC_get_msg_no) used for the house.
pub mod nook_msg {
    pub const REHOUSE_MEDIUM: usize = 0x00;
    pub const MEDIUM_BUILT: usize = 0x03;
    pub const REHOUSE_2_OFFER: usize = 0x04;
    pub const REHOUSE_2_BASEMENT: usize = 0x06;
    pub const LARGE_BUILT: usize = 0x07;
    pub const REHOUSE_STATUE: usize = 0x08;
    pub const STATUE_BUILT: usize = 0x09;
    pub const PALLET_SELECTED: usize = 0x0A;
    pub const BASEMENT_BUILT: usize = 0xCC;
    pub const BASEMENT_PAID: usize = 0xCD;
    pub const REHOUSE_BASEMENT: usize = 0xD0;
    pub const REHOUSE_LARGE: usize = 0xD1;
    pub const REHOUSE_UPPER: usize = 0xD4;
    pub const UPPER_BUILT: usize = 0xD5;
}

impl HouseSave {
    /// aNSC_start_wait with aNSC_set_talk_info_start_wait / _wait1: Nook's
    /// opening line about the house as the player walks in, applying its
    /// effects (the next loan after building, the statue order, the large
    /// house's automatic basement order). `None`: his ordinary greeting.
    pub fn nook_greeting(&mut self, loan: &mut u32, today: (u16, u8, u8)) -> Option<usize> {
        use nook_msg::*;
        let done_rehouse = *loan == 0
            && !self.renew
            && self.size < HouseSize::Upper as u8
            && self.size == self.next_size
            && !self.basement_ordered;
        if done_rehouse {
            return Some(match HouseSize::from_u8(self.size).unwrap_or_default() {
                HouseSize::Small => REHOUSE_MEDIUM,
                HouseSize::Medium if !self.has_basement => REHOUSE_2_OFFER,
                HouseSize::Medium => BASEMENT_PAID,
                _ if self.basement_billed => REHOUSE_UPPER,
                _ if !self.has_basement => {
                    self.with_house(loan, |h| h.order_basement(today));
                    REHOUSE_BASEMENT
                }
                _ => REHOUSE_LARGE,
            });
        }
        if self.renew {
            let basement = self.basement_ordered;
            let size = self.size;
            self.with_house(loan, |h| h.nook_process_renewal())?;
            return Some(if basement {
                BASEMENT_BUILT
            } else {
                [MEDIUM_BUILT, LARGE_BUILT, UPPER_BUILT, STATUE_BUILT]
                    [(size as usize).clamp(1, 4) - 1]
            });
        }
        if self.with_house(loan, |h| h.nook_process_statue_built()) {
            return Some(STATUE_BUILT);
        }
        let statues = self.town_statues;
        let ordered = self.with_house(loan, |h| h.order_statue(statues, today));
        if ordered {
            // Save num_statues: one more, at most three.
            self.town_statues = (statues + 1).min(3);
            return Some(REHOUSE_STATUE);
        }
        None
    }

    /// aNSC_check_roof_col_order(2): expand with roof palette `pal`.
    pub fn order_expansion(&mut self, loan: &mut u32, pal: u8, today: (u16, u8, u8)) -> bool {
        self.with_house(loan, |h| h.order_expansion(pal, today))
    }

    /// aNSC_set_make_basement_info.
    pub fn order_basement(&mut self, loan: &mut u32, today: (u16, u8, u8)) -> bool {
        self.with_house(loan, |h| h.order_basement(today))
    }
}

/// The player's house this session.
#[derive(Resource, Default, Debug)]
pub struct PlayerHouse(pub HouseSave);

/// Size (0-3) and roof palette of house 0, for the town model.
static LOOK: AtomicU16 = AtomicU16::new(0);

pub fn look() -> (u8, u8) {
    let v = LOOK.load(Ordering::Relaxed);
    ((v >> 8) as u8, v as u8)
}

fn set_look(house: &HouseSave) {
    let size = house.size.min(HouseSize::Upper as u8);
    LOOK.store(
        u16::from(size) << 8 | u16::from(house.outlook_pal.min(11)),
        Ordering::Relaxed,
    );
}

pub fn today() -> (u16, u8, u8) {
    let now = crate::clock::now();
    (now.year, now.month, now.day)
}

/// The loaded save's house, built up to date (PreStartup, before the town).
fn init_house(mut commands: Commands, loaded: Res<crate::save::LoadedSave>) {
    let mut house = loaded.0.as_ref().and_then(|s| s.house).unwrap_or_default();
    // The loan is restored with the inventory; construction only changes
    // size flags here, and the next loan is billed by Nook.
    let mut loan = loaded.0.as_ref().map_or(0, |s| s.loan);
    if house.check_rehouse_order(&mut loan, today()) {
        info!("house construction finished: {house:?}");
    }
    set_look(&house);
    commands.insert_resource(PlayerHouse(house));
}

/// F9: the loaded save's house (the town keeps the model it started with).
pub(crate) fn restore(
    house: &mut PlayerHouse,
    saved: Option<HouseSave>,
    inventory: &mut PlayerInventory,
) {
    let mut h = saved.unwrap_or_default();
    h.check_rehouse_order(&mut inventory.loan, today());
    set_look(&h);
    house.0 = h;
}

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, init_house.after(crate::save::init_save));
    }
}
