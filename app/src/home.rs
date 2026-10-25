//! The player's house record (`mHm_hs_c` size_info and roof palettes):
//! size, ordered upgrade, basement, statue, and the roof colour.
//!
//! mHm_CheckRehouseOrder runs when the game starts or a save is loaded: a
//! pending order from an earlier date is built (size, basement or statue),
//! the ordered roof colour applied, and `renew` set so Nook bills the next
//! loan when the player next visits ([`rustimal_logic::house`]).
use std::sync::atomic::{AtomicU16, AtomicU8, Ordering};

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
    /// aNSC_sell_check queues paint for mHm_CheckRehouseOrder at startup/load.
    /// Paint never enters pockets and does not replace an expansion's palette.
    pub(crate) fn order_paint(&mut self, item: u16) -> bool {
        let Some(palette) = item.checked_sub(0x222D).filter(|&p| p < 12) else {
            return false;
        };
        self.next_outlook_pal = palette as u8;
        true
    }

    /// The station statue is up (ac_douzou: next_size is STATUE).
    pub fn statue_built(&self) -> bool {
        self.next_size == HouseSize::Statue as u8
    }

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

/// The main floor's room by house size (aMI_size_dsp_table; floor widths
/// 240/320/400/400 game units): model prefix and width in units.
const ROOMS: [(&str, f32); 4] = [
    ("rom_myhome1", 6.0),
    ("rom_myhome2B", 8.0),
    ("rom_myhome3", 10.0),
    ("rom_myhome4_1", 10.0),
];

/// mFI_GetPlayerHouseFloorNo: 0 main floor, 1 upstairs (LL2), 2 basement.
pub const MAIN_FLOOR: u8 = 0;
pub const UPPER_FLOOR: u8 = 1;
pub const BASEMENT: u8 = 2;

/// The floor of the player's house being shown.
static FLOOR: AtomicU8 = AtomicU8::new(MAIN_FLOOR);

pub fn floor() -> u8 {
    FLOOR.load(Ordering::Relaxed)
}

pub fn set_floor(floor: u8) {
    FLOOR.store(floor.min(BASEMENT), Ordering::Relaxed);
}

/// The room on the current floor: model prefix and width in units
/// (LL2 rom_myhome4_2 is size M; the basement rom_myhome_ug is size L).
fn room() -> (&'static str, f32) {
    match floor() {
        UPPER_FLOOR => ("rom_myhome4_2", 8.0),
        BASEMENT => ("rom_myhome_ug", 10.0),
        _ => ROOMS[(look().0 as usize).min(3)],
    }
}

/// The current floor's floor and wall lists; the basement's single model
/// has no separate walls (aMI_size_dsp_table).
pub fn room_lists() -> (String, Option<String>) {
    let prefix = room().0;
    if floor() == BASEMENT {
        (format!("{prefix}_model"), None)
    } else {
        (
            format!("{prefix}_floor_model"),
            Some(format!("{prefix}_wall_model")),
        )
    }
}

/// A staircase on the current floor (aMI_MakeStepData / aMI_DrawMyStep).
#[derive(Clone, Copy, Debug)]
pub struct Stair {
    pub model: &'static str,
    /// aMI_scale_x_table: the upper floor and basement mirror theirs.
    pub mirror: bool,
    /// The floor it leads to.
    pub to: u8,
}

impl Stair {
    /// Its span across the south wall, in room-local cells (x from the room
    /// centre): the step models reach 40..115 game units to one side.
    pub fn span(&self) -> (f32, f32) {
        let pos = stair_pos();
        let side = if self.model.ends_with("down_model") != self.mirror {
            1.0
        } else {
            -1.0
        };
        let (a, b) = (pos.x + side * 40.0, pos.x + side * 115.0);
        let centre = room_centre().x;
        (a.min(b) / 40.0 - centre, a.max(b) / 40.0 - centre)
    }
}

/// aMI_DrawMyStep step_draw_pos by room size (S/M/L), in game units.
pub fn stair_pos() -> Vec3 {
    match room().1 as i32 {
        6 => Vec3::new(120.0, 40.0, 220.0),
        8 => Vec3::new(160.0, 40.0, 300.0),
        _ => Vec3::new(200.0, 40.0, 380.0),
    }
}

/// The current floor's stairs for this house.
pub fn stairs(house: &HouseSave) -> Vec<Stair> {
    const DOWN: &str = "obj_myhome_step_down_model";
    const UP: &str = "obj_myhome_step_up_model";
    let mut out = Vec::new();
    match floor() {
        MAIN_FLOOR => {
            if (1..=3).contains(&house.size) && house.has_basement {
                out.push(Stair {
                    model: DOWN,
                    mirror: false,
                    to: BASEMENT,
                });
            }
            if house.size == HouseSize::Upper as u8 {
                out.push(Stair {
                    model: UP,
                    mirror: false,
                    to: UPPER_FLOOR,
                });
            }
        }
        UPPER_FLOOR => out.push(Stair {
            model: DOWN,
            mirror: true,
            to: MAIN_FLOOR,
        }),
        _ => out.push(Stair {
            model: UP,
            mirror: true,
            to: MAIN_FLOOR,
        }),
    }
    out
}

/// The room's unit (0, 0) corner to its centre, at floor height 1.
pub fn room_centre() -> Vec3 {
    let half = room().1 / 2.0;
    Vec3::new(half, 1.0, half)
}

/// Half the walkable inside, in cells (the walls take one unit each side).
pub fn room_half() -> f32 {
    room().1 / 2.0 - 1.0
}

/// The last furniture unit inside the walls (units 1..=this).
pub fn room_last_unit() -> i32 {
    room().1 as i32 - 2
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

/// F9: restore house state; the town refreshes roof palettes at the same size.
/// A size-changing exterior reload still needs a restart.
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
