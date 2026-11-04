//! Event scheduler: which events are active today (`m_event.c`).
//!
//! Ported from ac-decomp `src/game/m_event.c` (`decode_date`, `m_weekday2day`, `after_n_day`,
//! `check_date_range`, `add_event_today`, `update_schedule_today`, `init_weekly_event`) with the
//! full 134-row table of `m_event_schedule.c_inc` and the `enum event_table` ids, both generated
//! from the C files. Pure and disc-free: the caller supplies the clock and the few save values the
//! rows read (`Inputs`).
//!
//! Supported: every plain dated row, weekly rows (Nth / last / every weekday, day-after), month-end,
//! town-day, save-slot dates (birthday, weekly), the harvest-moon date, the two equinox / sports-fair
//! adjustments, the summer camper, and the weekly picker (Joan on Sunday, K.K. Slider on Saturday,
//! Gulliver on a weekday, Blanca / bridge flags as inputs).
//!
//! NOT supported yet (their rows stay inactive, see `row_supported`): the shop / designer / broker /
//! gypsy / artist special-event rows are supported when `Inputs::special_type` and `dates[SPECIAL0..3]` are
//! filled in (`update_special_event`); the two handbill rows (leaflets) stay off. Before, the
//! rows were off (`update_special_event` needs the special-event save slots),
//! lighthouse vacation, soncho fishing-tourney gating, the harvest-moon / fishing-tourney gossip rows
//! (`update_event_rumor`), and the ghost and bridge-make rows (wisp / bridge save state). The
//! `update_soncho_event2` side effects (clearing the weekly flag) are not applied.

use crate::game_time::{self, autumnal_equinox_day, harvest_moon_day, vernal_equinox_day};

/// Schedule field flags (`m_event.h:17-50`).
pub mod sched {
    pub const WEEKLY: u8 = 0x80;
    pub const TOWN_DAY: u8 = 0x40;
    pub const DAY_AFTER: u8 = 0x40;
    pub const LAST_DAY_OF_MONTH: u8 = 0x20;
    pub const HARVEST_MOON_DATE: u8 = 0x40;
    pub const NOW_MONTH: u8 = 0x20;
    pub const USE_SAVE_MONTH: u8 = 0x10;
    pub const TODAY: u8 = 0x80;
    pub const MULTIDAY: u8 = 0x40;
    pub const USE_SAVE_SLOT_VALUE: u8 = 0x20;
    /// `mEv_SCHEDULE_LAST_WEEKDAY_OF_MONTH`.
    pub const LAST_WEEKDAY_OF_MONTH: u8 = 6;
    /// `mEv_SCHEDULE_EVERY_WEEK`.
    pub const EVERY_WEEK: u8 = 7;
}

/// `mEv_SAVE_DATE_*` indices into `event_save_common.dates`.
pub mod save_date {
    pub const TODAY: usize = 0;
    pub const LAST_PLAY_DATE: usize = 1;
    pub const BIRTHDAY: usize = 2;
    pub const SPECIAL0: usize = 3;
    pub const SPECIAL1: usize = 4;
    pub const SPECIAL2: usize = 5;
    pub const WEEKLY: usize = 6;
    pub const SPECIAL3: usize = 7;
    pub const NUM: usize = 8;
}

/// `mEv_SCHEDULE_MAKE_WEEKLY_DATA(week, weekday)`.
pub const fn weekly_data(week: u8, weekday: u8) -> u8 {
    sched::WEEKLY | (((week & 0b111) << 3) | (weekday & 0b111))
}

/// `mEv_SCHEDULE_SAVE_MONTH(m)`.
pub const fn save_month(m: u8) -> u8 {
    sched::USE_SAVE_MONTH | (m & 0xF)
}

/// `mEv_SCHEDULE_HOUR_SLOT(h)`.
pub const fn hour_slot(h: u8) -> u8 {
    sched::USE_SAVE_SLOT_VALUE | (h & 0xF)
}

/// One date field of a schedule row (`mEv_schedule_date_c`; the unused byte is dropped).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Raw {
    pub month: u8,
    pub day: u8,
    pub hour: u8,
}

impl Raw {
    pub const fn new(month: u8, day: u8, hour: u8) -> Self {
        Self { month, day, hour }
    }

    /// `md`: month in the high byte, day in the low byte (the PC build's ordering).
    pub const fn md(self) -> u16 {
        ((self.month as u16) << 8) | self.day as u16
    }
}

/// `mEv_schedule_c` without the always-zero `_pad`.
#[derive(Clone, Copy, Debug)]
pub struct Row {
    pub begin: Raw,
    pub end: Raw,
    pub event: u16,
}

// GENERATED from ac-decomp `include/m_event.h` (enum event_table) and `src/game/m_event_schedule.c_inc`
// by a throwaway script (every row, in order). Do not hand-edit this block.
/// `enum event_table` (`mEv_EVENT_*`), numbered in declaration order.
pub mod ev {
    pub const RUMOR_NEW_YEARS_DAY: u16 = 0;
    pub const NEW_YEARS_DAY: u16 = 1;
    pub const RUMOR_KAMAKURA: u16 = 2;
    pub const KAMAKURA: u16 = 3;
    pub const SONCHO_VACATION_JANUARY: u16 = 4;
    pub const SONCHO_VACATION_FEBRUARY: u16 = 5;
    pub const RUMOR_GROUNDHOG_DAY: u16 = 6;
    pub const GROUNDHOG_DAY: u16 = 7;
    pub const RUMOR_VALENTINES_DAY: u16 = 8;
    pub const VALENTINES_DAY: u16 = 9;
    pub const RUMOR_SPRING_SPORTS_FAIR: u16 = 10;
    pub const SPRING_EQUINOX: u16 = 11;
    pub const SPORTS_FAIR_BALL_TOSS: u16 = 12;
    pub const SPORTS_FAIR_AEROBICS: u16 = 13;
    pub const SPORTS_FAIR_TUG_OF_WAR: u16 = 14;
    pub const SPORTS_FAIR_FOOT_RACE: u16 = 15;
    pub const SPORTS_FAIR: u16 = 16;
    pub const APRILFOOLS_DAY: u16 = 17;
    pub const RUMOR_APRILFOOLS_DAY: u16 = 18;
    pub const RUMOR_CHERRY_BLOSSOM_FESTIVAL: u16 = 19;
    pub const CHERRY_BLOSSOM_FESTIVAL: u16 = 20;
    pub const NATURE_DAY: u16 = 21;
    pub const SPRING_CLEANING: u16 = 22;
    pub const MOTHERS_DAY: u16 = 23;
    pub const SUMMER_CAMPER: u16 = 24;
    pub const GRADUATION_DAY: u16 = 25;
    pub const FATHERS_DAY: u16 = 26;
    pub const RUMOR_FISHING_TOURNEY_1: u16 = 27;
    pub const TALK_FISHING_TOURNEY_1: u16 = 28;
    pub const FISHING_TOURNEY_1: u16 = 29;
    pub const TOWN_DAY: u16 = 30;
    pub const RUMOR_FIREWORKS_SHOW: u16 = 31;
    pub const FIREWORKS_SHOW: u16 = 32;
    pub const RUMOR_MORNING_AEROBICS: u16 = 33;
    pub const TALK_MORNING_AEROBICS: u16 = 34;
    pub const MORNING_AEROBICS: u16 = 35;
    pub const RUMOR_METEOR_SHOWER: u16 = 36;
    pub const METEOR_SHOWER: u16 = 37;
    pub const FOUNDERS_DAY: u16 = 38;
    pub const LABOR_DAY: u16 = 39;
    pub const RUMOR_FALL_SPORTS_FAIR: u16 = 40;
    pub const AUTUMN_EQUINOX: u16 = 41;
    pub const RUMOR_HARVEST_MOON_DAY: u16 = 42;
    pub const HARVEST_MOON_FESTIVAL: u16 = 43;
    pub const EXPLORERS_DAY: u16 = 44;
    pub const RUMOR_MUSHROOM_SEASON: u16 = 45;
    pub const TALK_MUSHROOM_SEASON: u16 = 46;
    pub const MUSHROOM_SEASON: u16 = 47;
    pub const RUMOR_HALLOWEEN: u16 = 48;
    pub const HALLOWEEN: u16 = 49;
    pub const MAYORS_DAY: u16 = 50;
    pub const OFFICERS_DAY: u16 = 51;
    pub const RUMOR_FISHING_TOURNEY_2: u16 = 52;
    pub const TALK_FISHING_TOURNEY_2: u16 = 53;
    pub const FISHING_TOURNEY_2: u16 = 54;
    pub const RUMOR_HARVEST_FESTIVAL: u16 = 55;
    pub const HARVEST_FESTIVAL: u16 = 56;
    pub const SALE_DAY: u16 = 57;
    pub const SNOW_DAY: u16 = 58;
    pub const TOY_DAY_SONCHO: u16 = 59;
    pub const RUMOR_TOY_DAY: u16 = 60;
    pub const TOY_DAY_JINGLE: u16 = 61;
    pub const SNOWMAN_SEASON: u16 = 62;
    pub const RUMOR_NEW_YEARS_EVE_COUNTDOWN: u16 = 63;
    pub const NEW_YEARS_EVE_COUNTDOWN: u16 = 64;
    pub const CHERRY_BLOSSOM_PETALS: u16 = 65;
    pub const TALK_TOY_DAY: u16 = 66;
    pub const PLAYER_BIRTHDAY: u16 = 67;
    pub const DOZAEMON: u16 = 68;
    pub const KABU_PEDDLER: u16 = 69;
    pub const LOTTERY: u16 = 70;
    pub const KK_SLIDER: u16 = 71;
    pub const HANDBILL_BROKER: u16 = 72;
    pub const HANDBILL_SHOP_SALE: u16 = 73;
    pub const ARTIST: u16 = 74;
    pub const BROKER_SALE: u16 = 75;
    pub const DESIGNER: u16 = 76;
    pub const GYPSY: u16 = 77;
    pub const SHOP_SALE: u16 = 78;
    pub const CARPET_PEDDLER: u16 = 79;
    pub const SONCHO_NEW_YEARS_DAY: u16 = 80;
    pub const SONCHO_GROUNDHOG_DAY: u16 = 81;
    pub const SONCHO_SPRING_SPORTS_FAIR: u16 = 82;
    pub const SONCHO_APRILFOOLS_DAY: u16 = 83;
    pub const SONCHO_CHERRY_BLOSSOM_FESTIVAL: u16 = 84;
    pub const SONCHO_NATURE_DAY: u16 = 85;
    pub const SONCHO_SPRING_CLEANING: u16 = 86;
    pub const SONCHO_MOTHERS_DAY: u16 = 87;
    pub const SONCHO_GRADUATION_DAY: u16 = 88;
    pub const SONCHO_FATHERS_DAY: u16 = 89;
    pub const SONCHO_FISHING_TOURNEY_1: u16 = 90;
    pub const SONCHO_TOWN_DAY: u16 = 91;
    pub const SONCHO_FIREWORKS_SHOW: u16 = 92;
    pub const SONCHO_METEOR_SHOWER: u16 = 93;
    pub const SONCHO_FOUNDERS_DAY: u16 = 94;
    pub const SONCHO_LABOR_DAY: u16 = 95;
    pub const SONCHO_FALL_SPORTS_FAIR: u16 = 96;
    pub const SONCHO_HARVEST_MOON_FESTIVAL: u16 = 97;
    pub const SONCHO_EXPLORERS_DAY: u16 = 98;
    pub const SONCHO_HALLOWEEN: u16 = 99;
    pub const SONCHO_MAYORS_DAY: u16 = 100;
    pub const SONCHO_OFFICERS_DAY: u16 = 101;
    pub const SONCHO_FISHING_TOURNEY_2: u16 = 102;
    pub const SONCHO_HARVEST_FESTIVAL: u16 = 103;
    pub const SONCHO_SALE_DAY: u16 = 104;
    pub const SONCHO_SNOW_DAY: u16 = 105;
    pub const SONCHO_TOY_DAY: u16 = 106;
    pub const TALK_NEW_YEARS_COUNTDOWN: u16 = 107;
    pub const HARVEST_FESTIVAL_FRANKLIN: u16 = 108;
    pub const WEATHER_CLEAR: u16 = 109;
    pub const WEATHER_SNOW: u16 = 110;
    pub const WEATHER_SPORTS_FAIR: u16 = 111;
    pub const BRIDGE_MAKE: u16 = 112;
    pub const SONCHO_BRIDGE_MAKE: u16 = 113;
    pub const GHOST: u16 = 114;
    pub const MASK_NPC: u16 = 115;
    pub const UNUSED_74: u16 = 116;
    pub const KOINOBORI: u16 = 117;
    pub const UNUSED_76: u16 = 118;
    /// `mEv_EVENT_NUM`.
    pub const NUM: u16 = 119;
}

/// Event names (without the mEv_EVENT_ prefix), indexed by id.
pub const NAMES: [&str; 119] = [
    "RUMOR_NEW_YEARS_DAY",
    "NEW_YEARS_DAY",
    "RUMOR_KAMAKURA",
    "KAMAKURA",
    "SONCHO_VACATION_JANUARY",
    "SONCHO_VACATION_FEBRUARY",
    "RUMOR_GROUNDHOG_DAY",
    "GROUNDHOG_DAY",
    "RUMOR_VALENTINES_DAY",
    "VALENTINES_DAY",
    "RUMOR_SPRING_SPORTS_FAIR",
    "SPRING_EQUINOX",
    "SPORTS_FAIR_BALL_TOSS",
    "SPORTS_FAIR_AEROBICS",
    "SPORTS_FAIR_TUG_OF_WAR",
    "SPORTS_FAIR_FOOT_RACE",
    "SPORTS_FAIR",
    "APRILFOOLS_DAY",
    "RUMOR_APRILFOOLS_DAY",
    "RUMOR_CHERRY_BLOSSOM_FESTIVAL",
    "CHERRY_BLOSSOM_FESTIVAL",
    "NATURE_DAY",
    "SPRING_CLEANING",
    "MOTHERS_DAY",
    "SUMMER_CAMPER",
    "GRADUATION_DAY",
    "FATHERS_DAY",
    "RUMOR_FISHING_TOURNEY_1",
    "TALK_FISHING_TOURNEY_1",
    "FISHING_TOURNEY_1",
    "TOWN_DAY",
    "RUMOR_FIREWORKS_SHOW",
    "FIREWORKS_SHOW",
    "RUMOR_MORNING_AEROBICS",
    "TALK_MORNING_AEROBICS",
    "MORNING_AEROBICS",
    "RUMOR_METEOR_SHOWER",
    "METEOR_SHOWER",
    "FOUNDERS_DAY",
    "LABOR_DAY",
    "RUMOR_FALL_SPORTS_FAIR",
    "AUTUMN_EQUINOX",
    "RUMOR_HARVEST_MOON_DAY",
    "HARVEST_MOON_FESTIVAL",
    "EXPLORERS_DAY",
    "RUMOR_MUSHROOM_SEASON",
    "TALK_MUSHROOM_SEASON",
    "MUSHROOM_SEASON",
    "RUMOR_HALLOWEEN",
    "HALLOWEEN",
    "MAYORS_DAY",
    "OFFICERS_DAY",
    "RUMOR_FISHING_TOURNEY_2",
    "TALK_FISHING_TOURNEY_2",
    "FISHING_TOURNEY_2",
    "RUMOR_HARVEST_FESTIVAL",
    "HARVEST_FESTIVAL",
    "SALE_DAY",
    "SNOW_DAY",
    "TOY_DAY_SONCHO",
    "RUMOR_TOY_DAY",
    "TOY_DAY_JINGLE",
    "SNOWMAN_SEASON",
    "RUMOR_NEW_YEARS_EVE_COUNTDOWN",
    "NEW_YEARS_EVE_COUNTDOWN",
    "CHERRY_BLOSSOM_PETALS",
    "TALK_TOY_DAY",
    "PLAYER_BIRTHDAY",
    "DOZAEMON",
    "KABU_PEDDLER",
    "LOTTERY",
    "KK_SLIDER",
    "HANDBILL_BROKER",
    "HANDBILL_SHOP_SALE",
    "ARTIST",
    "BROKER_SALE",
    "DESIGNER",
    "GYPSY",
    "SHOP_SALE",
    "CARPET_PEDDLER",
    "SONCHO_NEW_YEARS_DAY",
    "SONCHO_GROUNDHOG_DAY",
    "SONCHO_SPRING_SPORTS_FAIR",
    "SONCHO_APRILFOOLS_DAY",
    "SONCHO_CHERRY_BLOSSOM_FESTIVAL",
    "SONCHO_NATURE_DAY",
    "SONCHO_SPRING_CLEANING",
    "SONCHO_MOTHERS_DAY",
    "SONCHO_GRADUATION_DAY",
    "SONCHO_FATHERS_DAY",
    "SONCHO_FISHING_TOURNEY_1",
    "SONCHO_TOWN_DAY",
    "SONCHO_FIREWORKS_SHOW",
    "SONCHO_METEOR_SHOWER",
    "SONCHO_FOUNDERS_DAY",
    "SONCHO_LABOR_DAY",
    "SONCHO_FALL_SPORTS_FAIR",
    "SONCHO_HARVEST_MOON_FESTIVAL",
    "SONCHO_EXPLORERS_DAY",
    "SONCHO_HALLOWEEN",
    "SONCHO_MAYORS_DAY",
    "SONCHO_OFFICERS_DAY",
    "SONCHO_FISHING_TOURNEY_2",
    "SONCHO_HARVEST_FESTIVAL",
    "SONCHO_SALE_DAY",
    "SONCHO_SNOW_DAY",
    "SONCHO_TOY_DAY",
    "TALK_NEW_YEARS_COUNTDOWN",
    "HARVEST_FESTIVAL_FRANKLIN",
    "WEATHER_CLEAR",
    "WEATHER_SNOW",
    "WEATHER_SPORTS_FAIR",
    "BRIDGE_MAKE",
    "SONCHO_BRIDGE_MAKE",
    "GHOST",
    "MASK_NPC",
    "74",
    "KOINOBORI",
    "76",
];

/// The 134 rows of `event_schedule_data`, verbatim (date fields are the raw encoded values).
pub const ROWS: [Row; 134] = [
    Row {
        begin: Raw::new(save_month(3), 0, sched::MULTIDAY | 4),
        end: Raw::new(save_month(4), 0, hour_slot(7)),
        event: ev::HANDBILL_SHOP_SALE,
    },
    Row {
        begin: Raw::new(save_month(4), 0, 0),
        end: Raw::new(save_month(4), 0, 23),
        event: ev::SHOP_SALE,
    },
    Row {
        begin: Raw::new(save_month(4), 0, sched::MULTIDAY | 6),
        end: Raw::new(save_month(5), 0, 5),
        event: ev::DESIGNER,
    },
    Row {
        begin: Raw::new(save_month(3), 0, sched::MULTIDAY | 0),
        end: Raw::new(save_month(4), 0, 17),
        event: ev::HANDBILL_BROKER,
    },
    Row {
        begin: Raw::new(save_month(4), 0, sched::MULTIDAY | 18),
        end: Raw::new(save_month(5), 0, 17),
        event: ev::BROKER_SALE,
    },
    Row {
        begin: Raw::new(save_month(4), 0, sched::MULTIDAY | 6),
        end: Raw::new(save_month(5), 0, 5),
        event: ev::ARTIST,
    },
    Row {
        begin: Raw::new(save_month(4), 0, sched::MULTIDAY | 6),
        end: Raw::new(save_month(5), 0, 5),
        event: ev::CARPET_PEDDLER,
    },
    Row {
        begin: Raw::new(save_month(4), 0, sched::MULTIDAY | 21),
        end: Raw::new(save_month(5), 0, 20),
        event: ev::GYPSY,
    },
    Row {
        begin: Raw::new(sched::NOW_MONTH, weekly_data(7, 0), 6),
        end: Raw::new(sched::NOW_MONTH, weekly_data(7, 0), 11),
        event: ev::KABU_PEDDLER,
    },
    Row {
        begin: Raw::new(sched::NOW_MONTH, weekly_data(7, 6), 20),
        end: Raw::new(sched::NOW_MONTH, weekly_data(7, 6), 23),
        event: ev::KK_SLIDER,
    },
    Row {
        begin: Raw::new(save_month(6), 0, 6),
        end: Raw::new(save_month(6), 0, 22),
        event: ev::DOZAEMON,
    },
    Row {
        begin: Raw::new(sched::NOW_MONTH, sched::LAST_DAY_OF_MONTH, 0),
        end: Raw::new(sched::NOW_MONTH, sched::LAST_DAY_OF_MONTH, 23),
        event: ev::LOTTERY,
    },
    Row {
        begin: Raw::new(save_month(2), 0, 0),
        end: Raw::new(save_month(2), 0, 23),
        event: ev::PLAYER_BIRTHDAY,
    },
    Row {
        begin: Raw::new(12, 25, 0),
        end: Raw::new(2, 17, 23),
        event: ev::SNOWMAN_SEASON,
    },
    Row {
        begin: Raw::new(4, 3, 0),
        end: Raw::new(4, 8, 23),
        event: ev::CHERRY_BLOSSOM_PETALS,
    },
    Row {
        begin: Raw::new(1, 1, 0),
        end: Raw::new(12, 31, 3),
        event: ev::GHOST,
    },
    Row {
        begin: Raw::new(sched::NOW_MONTH, weekly_data(7, 1), 6),
        end: Raw::new(sched::NOW_MONTH, weekly_data(7, 1), 23),
        event: ev::MASK_NPC,
    },
    Row {
        begin: Raw::new(sched::NOW_MONTH, weekly_data(7, 2), 6),
        end: Raw::new(sched::NOW_MONTH, weekly_data(7, 2), 23),
        event: ev::MASK_NPC,
    },
    Row {
        begin: Raw::new(sched::NOW_MONTH, weekly_data(7, 3), 6),
        end: Raw::new(sched::NOW_MONTH, weekly_data(7, 3), 23),
        event: ev::MASK_NPC,
    },
    Row {
        begin: Raw::new(sched::NOW_MONTH, weekly_data(7, 4), 6),
        end: Raw::new(sched::NOW_MONTH, weekly_data(7, 4), 23),
        event: ev::MASK_NPC,
    },
    Row {
        begin: Raw::new(sched::NOW_MONTH, weekly_data(7, 5), 6),
        end: Raw::new(sched::NOW_MONTH, weekly_data(7, 5), 23),
        event: ev::MASK_NPC,
    },
    Row {
        begin: Raw::new(sched::NOW_MONTH, weekly_data(7, 6), 6),
        end: Raw::new(sched::NOW_MONTH, weekly_data(7, 6), 18),
        event: ev::MASK_NPC,
    },
    Row {
        begin: Raw::new(sched::NOW_MONTH, weekly_data(7, 0), 13),
        end: Raw::new(sched::NOW_MONTH, weekly_data(7, 0), 23),
        event: ev::MASK_NPC,
    },
    Row {
        begin: Raw::new(1, 1, 0),
        end: Raw::new(12, 31, 23),
        event: ev::BRIDGE_MAKE,
    },
    Row {
        begin: Raw::new(1, 1, 6),
        end: Raw::new(1, 1, 10),
        event: ev::NEW_YEARS_DAY,
    },
    Row {
        begin: Raw::new(1, 1, 6),
        end: Raw::new(1, 1, 10),
        event: ev::SONCHO_NEW_YEARS_DAY,
    },
    Row {
        begin: Raw::new(12, 25, sched::MULTIDAY | 0),
        end: Raw::new(12, 31, 23),
        event: ev::RUMOR_NEW_YEARS_DAY,
    },
    Row {
        begin: Raw::new(1, 2, sched::TODAY | 10),
        end: Raw::new(2, 23, 21),
        event: ev::KAMAKURA,
    },
    Row {
        begin: Raw::new(1, 2, sched::MULTIDAY | 0),
        end: Raw::new(2, 23, 20),
        event: ev::RUMOR_KAMAKURA,
    },
    Row {
        begin: Raw::new(2, 2, 7),
        end: Raw::new(2, 2, 8),
        event: ev::GROUNDHOG_DAY,
    },
    Row {
        begin: Raw::new(2, 2, 9),
        end: Raw::new(2, 2, 16),
        event: ev::SONCHO_GROUNDHOG_DAY,
    },
    Row {
        begin: Raw::new(1, 25, sched::MULTIDAY | 0),
        end: Raw::new(2, 1, 23),
        event: ev::RUMOR_GROUNDHOG_DAY,
    },
    Row {
        begin: Raw::new(2, 14, 0),
        end: Raw::new(2, 14, 23),
        event: ev::VALENTINES_DAY,
    },
    Row {
        begin: Raw::new(2, 1, sched::MULTIDAY | 0),
        end: Raw::new(2, 13, 23),
        event: ev::RUMOR_VALENTINES_DAY,
    },
    Row {
        begin: Raw::new(3, 19, 9),
        end: Raw::new(3, 21, 16),
        event: ev::SPRING_EQUINOX,
    },
    Row {
        begin: Raw::new(3, 19, 9),
        end: Raw::new(3, 21, 16),
        event: ev::SONCHO_SPRING_SPORTS_FAIR,
    },
    Row {
        begin: Raw::new(3, 19, 9),
        end: Raw::new(3, 21, 10),
        event: ev::SPORTS_FAIR_AEROBICS,
    },
    Row {
        begin: Raw::new(3, 19, 11),
        end: Raw::new(3, 21, 12),
        event: ev::SPORTS_FAIR_FOOT_RACE,
    },
    Row {
        begin: Raw::new(3, 19, 13),
        end: Raw::new(3, 21, 14),
        event: ev::SPORTS_FAIR_BALL_TOSS,
    },
    Row {
        begin: Raw::new(3, 19, 15),
        end: Raw::new(3, 21, 16),
        event: ev::SPORTS_FAIR_TUG_OF_WAR,
    },
    Row {
        begin: Raw::new(3, 19, 11),
        end: Raw::new(3, 21, 16),
        event: ev::SPORTS_FAIR,
    },
    Row {
        begin: Raw::new(3, 19, 0),
        end: Raw::new(3, 21, 23),
        event: ev::WEATHER_SPORTS_FAIR,
    },
    Row {
        begin: Raw::new(3, 9, sched::MULTIDAY | 0),
        end: Raw::new(3, 20, 23),
        event: ev::RUMOR_SPRING_SPORTS_FAIR,
    },
    Row {
        begin: Raw::new(4, 1, 0),
        end: Raw::new(4, 1, 23),
        event: ev::APRILFOOLS_DAY,
    },
    Row {
        begin: Raw::new(4, 1, 10),
        end: Raw::new(4, 1, 17),
        event: ev::SONCHO_APRILFOOLS_DAY,
    },
    Row {
        begin: Raw::new(4, 5, 10),
        end: Raw::new(4, 7, 21),
        event: ev::CHERRY_BLOSSOM_FESTIVAL,
    },
    Row {
        begin: Raw::new(4, 5, 10),
        end: Raw::new(4, 7, 21),
        event: ev::SONCHO_CHERRY_BLOSSOM_FESTIVAL,
    },
    Row {
        begin: Raw::new(3, 15, sched::MULTIDAY | 0),
        end: Raw::new(3, 31, 23),
        event: ev::RUMOR_APRILFOOLS_DAY,
    },
    Row {
        begin: Raw::new(4, 1, sched::MULTIDAY | 0),
        end: Raw::new(4, 4, 23),
        event: ev::RUMOR_CHERRY_BLOSSOM_FESTIVAL,
    },
    Row {
        begin: Raw::new(4, 22, 10),
        end: Raw::new(4, 22, 17),
        event: ev::NATURE_DAY,
    },
    Row {
        begin: Raw::new(4, 22, 10),
        end: Raw::new(4, 22, 17),
        event: ev::SONCHO_NATURE_DAY,
    },
    Row {
        begin: Raw::new(5, 1, 0),
        end: Raw::new(5, 1, 23),
        event: ev::SPRING_CLEANING,
    },
    Row {
        begin: Raw::new(5, 1, 10),
        end: Raw::new(5, 1, 17),
        event: ev::SONCHO_SPRING_CLEANING,
    },
    Row {
        begin: Raw::new(5, weekly_data(2, 0), 0),
        end: Raw::new(5, weekly_data(2, 0), 23),
        event: ev::MOTHERS_DAY,
    },
    Row {
        begin: Raw::new(5, weekly_data(2, 0), 10),
        end: Raw::new(5, weekly_data(2, 0), 17),
        event: ev::SONCHO_MOTHERS_DAY,
    },
    Row {
        begin: Raw::new(6, weekly_data(7, 6), sched::MULTIDAY | 9),
        end: Raw::new(6, weekly_data(7, 0), 14),
        event: ev::SUMMER_CAMPER,
    },
    Row {
        begin: Raw::new(6, weekly_data(2, 5), 10),
        end: Raw::new(6, weekly_data(2, 5), 17),
        event: ev::GRADUATION_DAY,
    },
    Row {
        begin: Raw::new(6, weekly_data(2, 5), 10),
        end: Raw::new(6, weekly_data(2, 5), 17),
        event: ev::SONCHO_GRADUATION_DAY,
    },
    Row {
        begin: Raw::new(6, weekly_data(3, 0), 0),
        end: Raw::new(6, weekly_data(3, 0), 23),
        event: ev::FATHERS_DAY,
    },
    Row {
        begin: Raw::new(6, weekly_data(3, 0), 10),
        end: Raw::new(6, weekly_data(3, 0), 17),
        event: ev::SONCHO_FATHERS_DAY,
    },
    Row {
        begin: Raw::new(6, weekly_data(7, 0), 6),
        end: Raw::new(6, weekly_data(7, 0), 17),
        event: ev::FISHING_TOURNEY_1,
    },
    Row {
        begin: Raw::new(6, weekly_data(7, 0), 6),
        end: Raw::new(6, weekly_data(7, 0), 17),
        event: ev::SONCHO_FISHING_TOURNEY_1,
    },
    Row {
        begin: Raw::new(5, 15, sched::MULTIDAY | 0),
        end: Raw::new(5, 31, 23),
        event: ev::RUMOR_FISHING_TOURNEY_1,
    },
    Row {
        begin: Raw::new(6, 1, sched::MULTIDAY | 0),
        end: Raw::new(6, weekly_data(6, 0), 0),
        event: ev::TALK_FISHING_TOURNEY_1,
    },
    Row {
        begin: Raw::new(7, 4, 19),
        end: Raw::new(7, 4, 20),
        event: ev::FIREWORKS_SHOW,
    },
    Row {
        begin: Raw::new(7, 4, 19),
        end: Raw::new(7, 4, 20),
        event: ev::SONCHO_FIREWORKS_SHOW,
    },
    Row {
        begin: Raw::new(7, 4, 0),
        end: Raw::new(7, 4, 23),
        event: ev::WEATHER_CLEAR,
    },
    Row {
        begin: Raw::new(6, 25, sched::MULTIDAY | 0),
        end: Raw::new(7, 3, 23),
        event: ev::RUMOR_FIREWORKS_SHOW,
    },
    Row {
        begin: Raw::new(7, sched::TOWN_DAY, 10),
        end: Raw::new(7, sched::TOWN_DAY, 17),
        event: ev::TOWN_DAY,
    },
    Row {
        begin: Raw::new(7, sched::TOWN_DAY, 10),
        end: Raw::new(7, sched::TOWN_DAY, 17),
        event: ev::SONCHO_TOWN_DAY,
    },
    Row {
        begin: Raw::new(7, 25, sched::TODAY | 6),
        end: Raw::new(8, 31, sched::TODAY | 6),
        event: ev::MORNING_AEROBICS,
    },
    Row {
        begin: Raw::new(7, 14, sched::MULTIDAY | 0),
        end: Raw::new(7, 24, 23),
        event: ev::RUMOR_MORNING_AEROBICS,
    },
    Row {
        begin: Raw::new(7, 26, sched::MULTIDAY | 0),
        end: Raw::new(8, 30, 23),
        event: ev::TALK_MORNING_AEROBICS,
    },
    Row {
        begin: Raw::new(8, 12, 18),
        end: Raw::new(8, 12, 20),
        event: ev::METEOR_SHOWER,
    },
    Row {
        begin: Raw::new(8, 12, 18),
        end: Raw::new(8, 12, 20),
        event: ev::SONCHO_METEOR_SHOWER,
    },
    Row {
        begin: Raw::new(8, 5, sched::MULTIDAY | 0),
        end: Raw::new(8, 11, 23),
        event: ev::RUMOR_METEOR_SHOWER,
    },
    Row {
        begin: Raw::new(8, 12, 0),
        end: Raw::new(8, 12, 23),
        event: ev::WEATHER_CLEAR,
    },
    Row {
        begin: Raw::new(8, 21, 10),
        end: Raw::new(8, 21, 17),
        event: ev::FOUNDERS_DAY,
    },
    Row {
        begin: Raw::new(8, 21, 10),
        end: Raw::new(8, 21, 17),
        event: ev::SONCHO_FOUNDERS_DAY,
    },
    Row {
        begin: Raw::new(9, weekly_data(1, 1), 10),
        end: Raw::new(9, weekly_data(1, 1), 17),
        event: ev::LABOR_DAY,
    },
    Row {
        begin: Raw::new(9, weekly_data(1, 1), 10),
        end: Raw::new(9, weekly_data(1, 1), 17),
        event: ev::SONCHO_LABOR_DAY,
    },
    Row {
        begin: Raw::new(9, 21, 9),
        end: Raw::new(9, 23, 16),
        event: ev::AUTUMN_EQUINOX,
    },
    Row {
        begin: Raw::new(9, 21, 9),
        end: Raw::new(9, 23, 16),
        event: ev::SONCHO_FALL_SPORTS_FAIR,
    },
    Row {
        begin: Raw::new(9, 21, 9),
        end: Raw::new(9, 23, 10),
        event: ev::SPORTS_FAIR_AEROBICS,
    },
    Row {
        begin: Raw::new(9, 21, 11),
        end: Raw::new(9, 23, 12),
        event: ev::SPORTS_FAIR_FOOT_RACE,
    },
    Row {
        begin: Raw::new(9, 21, 13),
        end: Raw::new(9, 23, 14),
        event: ev::SPORTS_FAIR_BALL_TOSS,
    },
    Row {
        begin: Raw::new(9, 21, 15),
        end: Raw::new(9, 23, 16),
        event: ev::SPORTS_FAIR_TUG_OF_WAR,
    },
    Row {
        begin: Raw::new(9, 21, 11),
        end: Raw::new(9, 23, 16),
        event: ev::SPORTS_FAIR,
    },
    Row {
        begin: Raw::new(9, 21, 0),
        end: Raw::new(9, 23, 23),
        event: ev::WEATHER_SPORTS_FAIR,
    },
    Row {
        begin: Raw::new(9, 11, sched::MULTIDAY | 0),
        end: Raw::new(9, 22, 23),
        event: ev::RUMOR_FALL_SPORTS_FAIR,
    },
    Row {
        begin: Raw::new(sched::HARVEST_MOON_DATE | 8, 15, 18),
        end: Raw::new(sched::HARVEST_MOON_DATE | 8, 15, 20),
        event: ev::HARVEST_MOON_FESTIVAL,
    },
    Row {
        begin: Raw::new(sched::HARVEST_MOON_DATE | 8, 15, 18),
        end: Raw::new(sched::HARVEST_MOON_DATE | 8, 15, 20),
        event: ev::SONCHO_HARVEST_MOON_FESTIVAL,
    },
    Row {
        begin: Raw::new(sched::HARVEST_MOON_DATE | 8, 15, 0),
        end: Raw::new(sched::HARVEST_MOON_DATE | 8, 15, 23),
        event: ev::WEATHER_CLEAR,
    },
    Row {
        begin: Raw::new(sched::HARVEST_MOON_DATE | 8, 8, sched::MULTIDAY | 0),
        end: Raw::new(sched::HARVEST_MOON_DATE | 8, 14, 23),
        event: ev::RUMOR_HARVEST_MOON_DAY,
    },
    Row {
        begin: Raw::new(10, weekly_data(2, 1), 10),
        end: Raw::new(10, weekly_data(2, 1), 17),
        event: ev::EXPLORERS_DAY,
    },
    Row {
        begin: Raw::new(10, weekly_data(2, 1), 10),
        end: Raw::new(10, weekly_data(2, 1), 17),
        event: ev::SONCHO_EXPLORERS_DAY,
    },
    Row {
        begin: Raw::new(10, 15, 0),
        end: Raw::new(10, 25, 23),
        event: ev::MUSHROOM_SEASON,
    },
    Row {
        begin: Raw::new(10, 10, sched::MULTIDAY | 0),
        end: Raw::new(10, 14, 23),
        event: ev::RUMOR_MUSHROOM_SEASON,
    },
    Row {
        begin: Raw::new(10, 15, sched::MULTIDAY | 0),
        end: Raw::new(10, 25, 23),
        event: ev::TALK_MUSHROOM_SEASON,
    },
    Row {
        begin: Raw::new(10, 31, sched::MULTIDAY | 18),
        end: Raw::new(11, 1, 0),
        event: ev::HALLOWEEN,
    },
    Row {
        begin: Raw::new(10, 31, sched::MULTIDAY | 18),
        end: Raw::new(11, 1, 0),
        event: ev::SONCHO_HALLOWEEN,
    },
    Row {
        begin: Raw::new(10, 31, sched::MULTIDAY | 0),
        end: Raw::new(11, 1, 0),
        event: ev::WEATHER_CLEAR,
    },
    Row {
        begin: Raw::new(10, 1, sched::MULTIDAY | 0),
        end: Raw::new(10, 30, 23),
        event: ev::RUMOR_HALLOWEEN,
    },
    Row {
        begin: Raw::new(11, sched::DAY_AFTER | weekly_data(1, 1), 10),
        end: Raw::new(11, sched::DAY_AFTER | weekly_data(1, 1), 17),
        event: ev::MAYORS_DAY,
    },
    Row {
        begin: Raw::new(11, sched::DAY_AFTER | weekly_data(1, 1), 10),
        end: Raw::new(11, sched::DAY_AFTER | weekly_data(1, 1), 17),
        event: ev::SONCHO_MAYORS_DAY,
    },
    Row {
        begin: Raw::new(11, 11, 10),
        end: Raw::new(11, 11, 17),
        event: ev::OFFICERS_DAY,
    },
    Row {
        begin: Raw::new(11, 11, 10),
        end: Raw::new(11, 11, 17),
        event: ev::SONCHO_OFFICERS_DAY,
    },
    Row {
        begin: Raw::new(11, weekly_data(7, 0), 6),
        end: Raw::new(11, weekly_data(7, 0), 17),
        event: ev::FISHING_TOURNEY_2,
    },
    Row {
        begin: Raw::new(11, weekly_data(7, 0), 6),
        end: Raw::new(11, weekly_data(7, 0), 17),
        event: ev::SONCHO_FISHING_TOURNEY_2,
    },
    Row {
        begin: Raw::new(10, 15, sched::MULTIDAY | 0),
        end: Raw::new(10, 31, 23),
        event: ev::RUMOR_FISHING_TOURNEY_2,
    },
    Row {
        begin: Raw::new(11, 1, sched::MULTIDAY | 0),
        end: Raw::new(11, weekly_data(6, 0), 0),
        event: ev::TALK_FISHING_TOURNEY_2,
    },
    Row {
        begin: Raw::new(11, weekly_data(4, 4), 15),
        end: Raw::new(11, weekly_data(4, 4), 20),
        event: ev::HARVEST_FESTIVAL,
    },
    Row {
        begin: Raw::new(11, weekly_data(4, 4), 15),
        end: Raw::new(11, weekly_data(4, 4), 20),
        event: ev::SONCHO_HARVEST_FESTIVAL,
    },
    Row {
        begin: Raw::new(11, weekly_data(4, 4), 15),
        end: Raw::new(11, weekly_data(4, 4), 20),
        event: ev::HARVEST_FESTIVAL_FRANKLIN,
    },
    Row {
        begin: Raw::new(11, 11, sched::MULTIDAY | 0),
        end: Raw::new(11, sched::DAY_AFTER | weekly_data(1, 0), 23),
        event: ev::RUMOR_HARVEST_FESTIVAL,
    },
    Row {
        begin: Raw::new(11, weekly_data(4, 4), 0),
        end: Raw::new(11, weekly_data(4, 4), 23),
        event: ev::WEATHER_CLEAR,
    },
    Row {
        begin: Raw::new(11, sched::DAY_AFTER | weekly_data(4, 4), 0),
        end: Raw::new(11, sched::DAY_AFTER | weekly_data(4, 4), 23),
        event: ev::SALE_DAY,
    },
    Row {
        begin: Raw::new(11, sched::DAY_AFTER | weekly_data(4, 4), 10),
        end: Raw::new(11, sched::DAY_AFTER | weekly_data(4, 4), 17),
        event: ev::SONCHO_SALE_DAY,
    },
    Row {
        begin: Raw::new(12, 1, 10),
        end: Raw::new(12, 1, 17),
        event: ev::SNOW_DAY,
    },
    Row {
        begin: Raw::new(12, 1, 10),
        end: Raw::new(12, 1, 17),
        event: ev::SONCHO_SNOW_DAY,
    },
    Row {
        begin: Raw::new(12, 23, 10),
        end: Raw::new(12, 23, 17),
        event: ev::TOY_DAY_SONCHO,
    },
    Row {
        begin: Raw::new(12, 23, 10),
        end: Raw::new(12, 23, 17),
        event: ev::SONCHO_TOY_DAY,
    },
    Row {
        begin: Raw::new(12, 24, sched::MULTIDAY | 20),
        end: Raw::new(12, 25, 0),
        event: ev::TOY_DAY_JINGLE,
    },
    Row {
        begin: Raw::new(12, 24, 0),
        end: Raw::new(12, 24, 23),
        event: ev::TALK_TOY_DAY,
    },
    Row {
        begin: Raw::new(12, 24, 0),
        end: Raw::new(12, 24, 23),
        event: ev::WEATHER_SNOW,
    },
    Row {
        begin: Raw::new(12, 10, sched::MULTIDAY | 0),
        end: Raw::new(12, 23, 23),
        event: ev::RUMOR_TOY_DAY,
    },
    Row {
        begin: Raw::new(12, 31, sched::MULTIDAY | 23),
        end: Raw::new(1, 1, 0),
        event: ev::NEW_YEARS_EVE_COUNTDOWN,
    },
    Row {
        begin: Raw::new(12, 31, sched::MULTIDAY | 23),
        end: Raw::new(1, 1, 0),
        event: ev::TALK_NEW_YEARS_COUNTDOWN,
    },
    Row {
        begin: Raw::new(12, 25, sched::MULTIDAY | 0),
        end: Raw::new(12, 30, 23),
        event: ev::RUMOR_NEW_YEARS_EVE_COUNTDOWN,
    },
    Row {
        begin: Raw::new(12, 31, 0),
        end: Raw::new(12, 31, 23),
        event: ev::WEATHER_CLEAR,
    },
    Row {
        begin: Raw::new(1, 15, 10),
        end: Raw::new(1, 24, 17),
        event: ev::SONCHO_VACATION_JANUARY,
    },
    Row {
        begin: Raw::new(2, 12, 10),
        end: Raw::new(2, 21, 17),
        event: ev::SONCHO_VACATION_FEBRUARY,
    },
    Row {
        begin: Raw::new(1, 1, 7),
        end: Raw::new(12, 31, 23),
        event: ev::SONCHO_BRIDGE_MAKE,
    },
    Row {
        begin: Raw::new(5, 1, 0),
        end: Raw::new(5, 5, 23),
        event: ev::KOINOBORI,
    },
];

/// Name of event `id` (`"?"` out of range).
pub fn event_name(id: u16) -> &'static str {
    NAMES.get(id as usize).copied().unwrap_or("?")
}

/// The clock the scheduler reads (`lbRTC_time_c`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Now {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    /// 0 = Sunday .. 6 = Saturday.
    pub weekday: u8,
}

impl Now {
    /// `today_date.md`.
    pub const fn md(&self) -> u16 {
        ((self.month as u16) << 8) | self.day as u16
    }
}

/// Weekly-event save fields (`event_save_common.weekly_event`, `dates[WEEKLY]`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WeeklyState {
    /// `dates[mEv_SAVE_DATE_WEEKLY]` as a month/day value (0 = never set).
    pub date: u16,
    /// `weekly_event.type`: the weekly visitor of this week.
    pub kind: Option<u16>,
    /// `weekly_event.flags`: today's special flag (`DOZAEMON`, `SONCHO_BRIDGE_MAKE`, `MASK_NPC`) or none.
    pub flags: Option<u16>,
    /// `dozaemon_completed`.
    pub gulliver_completed: bool,
}

/// What `init_weekly_event` reads besides the clock.
#[derive(Clone, Copy, Debug, Default)]
pub struct WeeklyContext {
    /// The player is not a foreigner, the town is full (`now_npc_max >= ANIMAL_NUM_MAX`), no bridge exists
    /// and the lighthouse trip is not under way: Tortimer's bridge event may be scheduled.
    pub bridge_make_ok: bool,
    /// `mGH_check_birth2() || mMC_check_birth()`: Blanca (mask NPC) may appear.
    pub mask_npc_birth: bool,
}

/// The save values the rows read.
#[derive(Clone, Copy, Debug)]
pub struct Inputs {
    pub now: Now,
    /// `Save_Get(town_day)` (low bits of a `TOWN_DAY` row's day).
    pub town_day: u8,
    /// `event_save_common.dates` (`TODAY`, `BIRTHDAY` and `WEEKLY` are refreshed by `events_today`).
    pub dates: [u16; save_date::NUM],
    pub weekly: WeeklyState,
    /// `event_save_common.special_event.type`: which of the six special events (`SHOP_SALE`, `DESIGNER`,
    /// `BROKER_SALE`, `ARTIST`, `CARPET_PEDDLER`, `GYPSY`) is the current one, if any.
    pub special_type: Option<u16>,
}

/// `lbRTC_IS_LEAPYEAR`.
fn is_leap(year: u16) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

/// `last_day_of_month` (leap year taken from the current clock).
fn last_day_of_month(year: u16, month: u8) -> u8 {
    const DAYS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if !(1..=12).contains(&month) {
        return 0;
    }
    if month == 2 && is_leap(year) {
        29
    } else {
        DAYS[month as usize - 1]
    }
}

/// `after_n_day`: month/day value `n_day` days later (or earlier).
pub fn after_n_day(year: u16, month_day: u16, n_day: i32) -> u16 {
    let mut month = (month_day >> 8) as u8;
    let mut day = (month_day & 0xFF) as u8;
    if n_day >= 0 {
        let last = last_day_of_month(year, month);
        day = day.wrapping_add(n_day as u8);
        if day > last {
            month = if month == 12 { 1 } else { month + 1 };
            day = day.wrapping_sub(last);
        }
    } else {
        day = day.wrapping_add(n_day as u8);
        if day & 0x80 != 0 || day == 0 {
            month = if month == 1 {
                12
            } else {
                month.wrapping_sub(1)
            };
            let last = last_day_of_month(year, month);
            day = day.wrapping_add(last);
        }
    }
    ((month as u16) << 8) | day as u16
}

/// `check_date_range`: inclusive, wrapping over the year end when `lower > upper`.
pub fn check_date_range(date: u16, lower: u16, upper: u16) -> bool {
    if lower > upper {
        lower <= date || date <= upper
    } else {
        lower <= date && date <= upper
    }
}

/// `m_weekday2day` (weekday of the 1st of each month taken from the clock's year).
fn weekday2day(now: &Now, month: u8, day_enc: u8) -> u8 {
    if !(1..=12).contains(&month) {
        return 0;
    }
    let mut week = (day_enc >> 3) & 0b111;
    let weekday = (day_enc & 0b111) as i32;
    let mut day: i32 = 0;
    let last_day = last_day_of_month(now.year, month) as i32;
    let first = game_time::rtc_week(now.year, month, 1) as i32;

    if week == sched::EVERY_WEEK {
        if month > now.month {
            week = 1;
        } else if month < now.month {
            week = sched::LAST_WEEKDAY_OF_MONTH;
        } else {
            day = (now.day as i32 - (now.weekday as i32 - weekday)) & 0xFF;
            week = if day > last_day {
                sched::LAST_WEEKDAY_OF_MONTH
            } else {
                0
            };
        }
    }

    if week == sched::LAST_WEEKDAY_OF_MONTH {
        let t_day = (-1 + last_day + first) % 7;
        day = if weekday <= t_day {
            last_day - (t_day - weekday)
        } else {
            (last_day - (t_day - weekday)) - 7
        };
    } else if week != 0 {
        let w = week as i32;
        day = if weekday >= first {
            1 + (w - 1) * 7 + (weekday - first)
        } else {
            1 + w * 7 + (weekday - first)
        };
    }
    (day & 0xFF) as u8
}

/// Day of month of the `week`-th (1..=5, or 6 = last) `weekday` of `month` in the clock's year.
pub fn weekly_day(now: &Now, month: u8, week: u8, weekday: u8) -> u8 {
    weekday2day(now, month, weekly_data(week, weekday) & !sched::WEEKLY)
}

/// `decode_date`: resolve the encoded month / day / hour of one date field.
pub fn decode_date(raw: Raw, inp: &Inputs) -> Raw {
    let now = &inp.now;
    let mut month = raw.month;
    let mut day = raw.day;
    let mut hour = raw.hour;

    if month & sched::USE_SAVE_MONTH != 0 {
        let md = inp.dates.get((month & 0xF) as usize).copied().unwrap_or(0);
        month = (md >> 8) as u8;
        day = (md & 0xFF) as u8;
    } else if month & sched::NOW_MONTH != 0 {
        month = now.month;
    } else if month & sched::HARVEST_MOON_DATE != 0 {
        (month, day) = harvest_moon_day(now.year as i32).unwrap_or((0, 0));
    }

    let day_enc = day;
    if day_enc & sched::WEEKLY != 0 {
        day = weekday2day(now, month, day_enc & !sched::DAY_AFTER);
        if day_enc & sched::DAY_AFTER != 0 {
            let md = after_n_day(now.year, ((month as u16) << 8) | day as u16, 1);
            month = (md >> 8) as u8;
            day = (md & 0xFF) as u8;
        }
    } else if day_enc & sched::LAST_DAY_OF_MONTH != 0 {
        day = last_day_of_month(now.year, month);
    } else if day_enc & sched::TOWN_DAY != 0 {
        day = (day_enc & !sched::TOWN_DAY) | inp.town_day;
    }

    if hour & sched::USE_SAVE_SLOT_VALUE != 0 {
        let slot = ((hour & 0x1F) % 8) as usize;
        hour = (inp.dates[slot] as u8) | (hour & 0xC0);
    }
    Raw::new(month, day, hour)
}

/// An event active today (`mEv_event_today_c`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActiveEvent {
    pub event: u16,
    /// Bit `h` set: the event runs during hour `h` (24 bits).
    pub active_hours: u32,
    pub begin: u16,
    pub end: u16,
}

impl ActiveEvent {
    pub fn active_at(&self, hour: u8) -> bool {
        hour < 24 && self.active_hours & (1 << hour) != 0
    }
}

/// Rows this port can evaluate (the module header lists the rest).
pub fn row_supported(event: u16) -> bool {
    !matches!(
        event,
        ev::HANDBILL_SHOP_SALE
            | ev::HANDBILL_BROKER
            | ev::SONCHO_VACATION_JANUARY
            | ev::SONCHO_VACATION_FEBRUARY
            | ev::SONCHO_FISHING_TOURNEY_1
            | ev::SONCHO_FISHING_TOURNEY_2
            | ev::TALK_FISHING_TOURNEY_1
            | ev::TALK_FISHING_TOURNEY_2
            | ev::RUMOR_HARVEST_MOON_DAY
            | ev::RUMOR_HARVEST_FESTIVAL
            | ev::GHOST
            | ev::BRIDGE_MAKE
            | ev::SONCHO_BRIDGE_MAKE
    )
}

/// `update_special_event`: the six special-event rows run only while they are the saved special event.
fn special_allows(event: u16, current: Option<u16>) -> bool {
    match event {
        ev::ARTIST
        | ev::BROKER_SALE
        | ev::DESIGNER
        | ev::GYPSY
        | ev::SHOP_SALE
        | ev::CARPET_PEDDLER => current == Some(event),
        _ => true,
    }
}

/// `update_weekly_event`: whether the weekly state lets this row run.
fn weekly_allows(event: u16, w: &WeeklyState) -> bool {
    match event {
        ev::KK_SLIDER | ev::KABU_PEDDLER => w.kind == Some(event),
        ev::DOZAEMON => w.flags != Some(ev::SONCHO_BRIDGE_MAKE) && w.kind == Some(event),
        ev::MASK_NPC => w.flags == Some(event),
        _ => true,
    }
}

/// `add_event_today` for one decoded row: merge its hours into `out`.
fn add_event_today(out: &mut Vec<ActiveEvent>, month_day: u16, event: u16, begin: Raw, end: Raw) {
    let mut start_hour = begin.hour;
    let mut end_hour = end.hour;
    let (b, e);
    if start_hour & sched::TODAY != 0 {
        b = month_day;
        e = month_day;
        start_hour &= !sched::TODAY;
        end_hour &= !sched::TODAY;
    } else {
        b = begin.md();
        e = end.md();
    }
    if start_hour & sched::MULTIDAY != 0 {
        start_hour &= !sched::MULTIDAY;
        end_hour &= !sched::MULTIDAY;
        if month_day != begin.md() {
            start_hour = 0;
        }
        if month_day != end.md() {
            end_hour = 23;
        }
    }
    let mut hours = 0u32;
    for h in 0..24u8 {
        if start_hour <= h && h <= end_hour {
            hours |= 1 << h;
        }
    }
    if let Some(a) = out.iter_mut().find(|a| a.event == event) {
        a.active_hours |= hours;
        a.begin = b;
        a.end = e;
    } else {
        out.push(ActiveEvent {
            event,
            active_hours: hours,
            begin: b,
            end: e,
        });
    }
}

/// `update_schedule_today`: every event active on `inp.now`'s date (any hour), in table order.
/// `dates[TODAY]`, `dates[BIRTHDAY]` (`birthday` as month, day) and `dates[WEEKLY]` are refreshed first.
pub fn events_today(inp: &Inputs, birthday: Option<(u8, u8)>) -> Vec<ActiveEvent> {
    let mut inp = *inp;
    let now = inp.now;
    inp.dates[save_date::TODAY] = now.md();
    inp.dates[save_date::BIRTHDAY] = birthday.map_or(0, |(m, d)| ((m as u16) << 8) | d as u16);
    inp.dates[save_date::WEEKLY] = inp.weekly.date;

    let mut out = Vec::new();
    let mut equinox_day = 0u8;
    for row in ROWS.iter() {
        let mut begin = row.begin;
        let mut end = row.end;
        let mut live = row_supported(row.event)
            && weekly_allows(row.event, &inp.weekly)
            && special_allows(row.event, inp.special_type);

        // update_sports_fair
        match row.event {
            ev::SPRING_EQUINOX | ev::SONCHO_SPRING_SPORTS_FAIR => {
                if now.month != 3 || now.day as i32 != vernal_equinox_day(now.year as i32) {
                    live = false;
                } else {
                    begin.day = now.day;
                    end.day = now.day;
                    equinox_day = now.day;
                }
            }
            ev::AUTUMN_EQUINOX | ev::SONCHO_FALL_SPORTS_FAIR => {
                if now.month != 9 || now.day as i32 != autumnal_equinox_day(now.year as i32) {
                    live = false;
                } else {
                    begin.day = now.day;
                    end.day = now.day;
                    equinox_day = now.day;
                }
            }
            ev::RUMOR_SPRING_SPORTS_FAIR => {
                let d = vernal_equinox_day(now.year as i32);
                begin.day = (d - 10) as u8;
                end.day = (d - 1) as u8;
            }
            ev::RUMOR_FALL_SPORTS_FAIR => {
                let d = autumnal_equinox_day(now.year as i32);
                begin.day = (d - 10) as u8;
                end.day = (d - 1) as u8;
            }
            ev::SPORTS_FAIR_BALL_TOSS
            | ev::SPORTS_FAIR_AEROBICS
            | ev::SPORTS_FAIR_TUG_OF_WAR
            | ev::SPORTS_FAIR_FOOT_RACE
            | ev::SPORTS_FAIR
            | ev::WEATHER_SPORTS_FAIR => {
                if equinox_day == 0 {
                    live = false;
                } else {
                    begin.day = equinox_day;
                    end.day = equinox_day;
                }
            }
            _ => {}
        }
        if !live {
            continue;
        }

        let (b, e) = if row.event == ev::SUMMER_CAMPER {
            // The camper arrives on this month's Saturday (June to August), a week back on a Sunday,
            // and leaves the day after.
            if matches!(now.month, 6..=8) {
                begin.month = now.month;
            }
            let mut b = decode_date(begin, &inp);
            let mut md = b.md();
            if now.weekday == 0 {
                md = after_n_day(now.year, md, -7);
                b.month = (md >> 8) as u8;
                b.day = (md & 0xFF) as u8;
            }
            md = after_n_day(now.year, md, 1);
            let mut e = decode_date(end, &inp);
            e.month = (md >> 8) as u8;
            e.day = (md & 0xFF) as u8;
            (b, e)
        } else {
            (decode_date(begin, &inp), decode_date(end, &inp))
        };

        if check_date_range(now.md(), b.md(), e.md()) {
            add_event_today(&mut out, now.md(), row.event, b, e);
        }
    }
    out
}

/// `init_weekly_event` (weekday picker): Sunday is Joan, Saturday is K.K. Slider, any other day keeps or
/// re-rolls Gulliver's day between Monday and Friday. Updates `w` in place. The wisp (ghost) setup that the
/// original does at the end is not covered.
pub fn init_weekly_event(now: &Now, w: &mut WeeklyState, ctx: &WeeklyContext) {
    let today = now.md();
    let mut sched_date = w.date;
    let mut flag: Option<u16> = None;

    match now.weekday {
        0 => {
            if today != sched_date {
                sched_date = today;
                w.date = sched_date;
                w.kind = Some(ev::KABU_PEDDLER);
                flag = Some(1);
            }
        }
        6 => {
            if today != sched_date {
                sched_date = today;
                w.date = sched_date;
                w.kind = Some(ev::KK_SLIDER);
                flag = Some(1);
            }
        }
        wd => {
            let monday = after_n_day(now.year, today, 1 - wd as i32);
            let friday = after_n_day(now.year, today, 5 - wd as i32);
            if sched_date == 0 || !check_date_range(sched_date, monday, friday) {
                let ofs = 1 + (today as u32 + now.hour as u32) % 5;
                sched_date = after_n_day(now.year, today, ofs as i32 - wd as i32);
                w.date = sched_date;
                w.kind = Some(ev::DOZAEMON);
                w.gulliver_completed = false;
            }
            if sched_date == today {
                flag = Some(ev::DOZAEMON);
            }
        }
    }

    if (now.weekday == 6 || (now.weekday != 0 && sched_date != today)) && ctx.bridge_make_ok {
        flag = Some(ev::SONCHO_BRIDGE_MAKE);
    }

    if flag != Some(ev::DOZAEMON) && flag != Some(ev::SONCHO_BRIDGE_MAKE) && ctx.mask_npc_birth {
        flag = Some(ev::MASK_NPC);
    }
    // `flag = 1` above only records "a new weekly visitor was set"; the saved flag is an event id or none.
    w.flags = match flag {
        Some(1) => None,
        other => other,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(year: u16, month: u8, day: u8, hour: u8) -> Now {
        Now {
            year,
            month,
            day,
            hour,
            weekday: game_time::rtc_week(year, month, day),
        }
    }

    fn inputs(now: Now) -> Inputs {
        Inputs {
            now,
            town_day: 0,
            dates: [0; save_date::NUM],
            weekly: WeeklyState::default(),
            special_type: None,
        }
    }

    fn active(inp: &Inputs, birthday: Option<(u8, u8)>, event: u16) -> Option<ActiveEvent> {
        events_today(inp, birthday)
            .into_iter()
            .find(|a| a.event == event)
    }

    #[test]
    fn special_visitors_run_only_as_the_saved_special_event() {
        // Katrina (GYPSY): saved start 11-10, end 11-11, start hour 21 (SPECIAL3); row 21:00 .. 05:59 next day.
        let md = |m: u8, d: u8| (u16::from(m) << 8) | u16::from(d);
        let mut inp = inputs(at(2026, 11, 10, 22));
        inp.dates[save_date::SPECIAL0] = md(11, 8);
        inp.dates[save_date::SPECIAL1] = md(11, 10);
        inp.dates[save_date::SPECIAL2] = md(11, 11);
        inp.dates[save_date::SPECIAL3] = 21;
        assert!(
            active(&inp, None, ev::GYPSY).is_none(),
            "no special event saved"
        );
        inp.special_type = Some(ev::ARTIST);
        assert!(
            active(&inp, None, ev::GYPSY).is_none(),
            "a different one is saved"
        );
        inp.special_type = Some(ev::GYPSY);
        let a = active(&inp, None, ev::GYPSY).expect("gypsy");
        assert!(a.active_at(21) && a.active_at(23) && !a.active_at(20));
        // The next morning she is still around until the end hour of the second day.
        let mut next = inp;
        next.now = at(2026, 11, 11, 3);
        let b = active(&next, None, ev::GYPSY).expect("gypsy next morning");
        assert!(b.active_at(0) && b.active_at(3));
        // Another day entirely.
        let mut other = inp;
        other.now = at(2026, 11, 14, 22);
        assert!(active(&other, None, ev::GYPSY).is_none());
    }

    #[test]
    fn table_has_every_row_and_event() {
        assert_eq!(ROWS.len(), 134);
        assert_eq!(ev::NEW_YEARS_DAY, 1);
        const { assert!(ev::NUM > ev::KK_SLIDER) };
    }

    #[test]
    fn new_years_day_runs_six_to_ten() {
        let inp = inputs(at(2026, 1, 1, 8));
        let a = active(&inp, None, ev::NEW_YEARS_DAY).expect("new year");
        assert!(a.active_at(6) && a.active_at(9) && a.active_at(10));
        assert!(!a.active_at(5) && !a.active_at(11));
        assert!(active(&inputs(at(2026, 1, 2, 8)), None, ev::NEW_YEARS_DAY).is_none());
    }

    #[test]
    fn lottery_is_the_last_day_of_the_month() {
        assert!(active(&inputs(at(2028, 2, 29, 12)), None, ev::LOTTERY).is_some());
        assert!(active(&inputs(at(2028, 2, 28, 12)), None, ev::LOTTERY).is_none());
        assert!(active(&inputs(at(2026, 11, 30, 12)), None, ev::LOTTERY).is_some());
    }

    #[test]
    fn year_wrapping_row_covers_both_sides_of_the_new_year() {
        for (m, d) in [(12, 26), (12, 31), (1, 1), (2, 17)] {
            assert!(
                active(&inputs(at(2026, m, d, 0)), None, ev::SNOWMAN_SEASON).is_some(),
                "{m}-{d}"
            );
        }
        assert!(active(&inputs(at(2026, 2, 18, 0)), None, ev::SNOWMAN_SEASON).is_none());
        assert!(active(&inputs(at(2026, 12, 24, 0)), None, ev::SNOWMAN_SEASON).is_none());
    }

    #[test]
    fn multiday_rows_start_and_end_on_their_hours() {
        // Rumor of New Year's Eve: Dec 25 (from midnight) to Dec 31 23:00.
        let a = active(&inputs(at(2026, 12, 28, 3)), None, ev::RUMOR_NEW_YEARS_DAY).expect("rumor");
        assert_eq!(a.active_hours, 0x00FF_FFFF);
        // Kamakura starts at 10:00 on January 2 only.
        let k = active(&inputs(at(2026, 1, 2, 0)), None, ev::KAMAKURA).expect("kamakura");
        assert!(!k.active_at(9) && k.active_at(10));
    }

    #[test]
    fn weekly_decoder_finds_the_fourth_thursday_and_the_last_sunday() {
        let now = at(2026, 11, 3, 12);
        // Thanksgiving 2026 is Thursday November 26.
        assert_eq!(weekly_day(&now, 11, 4, 4), 26);
        // Last Sunday of June 2026 is the 28th.
        assert_eq!(weekly_day(&now, 6, sched::LAST_WEEKDAY_OF_MONTH, 0), 28);
        // January of the clock's year (2026): the 1st is a Thursday, so the first Monday is the 5th.
        assert_eq!(weekly_day(&now, 1, 1, 1), 5);
    }

    #[test]
    fn every_weekday_means_this_week_in_the_current_month() {
        // Tuesday 2026-11-03: this week's Saturday is the 7th and this week's Sunday the 1st.
        let now = at(2026, 11, 3, 12);
        assert_eq!(now.weekday, 2);
        assert_eq!(weekly_day(&now, 11, sched::EVERY_WEEK, 6), 7);
        assert_eq!(weekly_day(&now, 11, sched::EVERY_WEEK, 0), 1);
    }

    #[test]
    fn kk_slider_plays_saturday_evening_only_when_he_is_this_weeks_visitor() {
        let now = at(2026, 11, 7, 21);
        assert_eq!(now.weekday, 6);
        let mut inp = inputs(now);
        assert!(
            active(&inp, None, ev::KK_SLIDER).is_none(),
            "no weekly visitor chosen yet"
        );
        init_weekly_event(&now, &mut inp.weekly, &WeeklyContext::default());
        assert_eq!(inp.weekly.kind, Some(ev::KK_SLIDER));
        let a = active(&inp, None, ev::KK_SLIDER).expect("kk");
        assert!(!a.active_at(19) && a.active_at(20) && a.active_at(23));
        // Friday: the same save state does not make him appear.
        let fri = at(2026, 11, 6, 21);
        let mut inp2 = inputs(fri);
        inp2.weekly = inp.weekly;
        assert!(active(&inp2, None, ev::KK_SLIDER).is_none());
    }

    #[test]
    fn joan_is_sunday_morning() {
        let now = at(2026, 11, 8, 8);
        assert_eq!(now.weekday, 0);
        let mut inp = inputs(now);
        init_weekly_event(&now, &mut inp.weekly, &WeeklyContext::default());
        assert_eq!(inp.weekly.kind, Some(ev::KABU_PEDDLER));
        let a = active(&inp, None, ev::KABU_PEDDLER).expect("joan");
        assert!(a.active_at(6) && a.active_at(11) && !a.active_at(12));
    }

    #[test]
    fn gulliver_gets_one_weekday_and_keeps_it_all_week() {
        let mut state = WeeklyState::default();
        let mut hit = Vec::new();
        for day in 2..=6u8 {
            // Monday 2026-11-02 .. Friday 2026-11-06
            let now = at(2026, 11, day, 10);
            init_weekly_event(&now, &mut state, &WeeklyContext::default());
            let mut inp = inputs(now);
            inp.weekly = state;
            if active(&inp, None, ev::DOZAEMON).is_some() {
                hit.push(day);
            }
        }
        assert_eq!(hit.len(), 1, "exactly one Gulliver day: {hit:?}");
        assert_eq!(state.kind, Some(ev::DOZAEMON));
    }

    #[test]
    fn blanca_needs_her_birth_flag() {
        let now = at(2026, 11, 3, 12);
        let mut inp = inputs(now);
        let mut w = WeeklyState::default();
        init_weekly_event(
            &now,
            &mut w,
            &WeeklyContext {
                mask_npc_birth: true,
                ..Default::default()
            },
        );
        inp.weekly = w;
        // A weekday that is not Gulliver's day: flag is the mask NPC.
        if w.date != now.md() {
            assert_eq!(w.flags, Some(ev::MASK_NPC));
            assert!(active(&inp, None, ev::MASK_NPC).is_some());
        }
    }

    #[test]
    fn equinox_sports_fair_follows_the_equinox_day() {
        let day = vernal_equinox_day(2026) as u8;
        let yes = active(
            &inputs(at(2026, 3, day, 10)),
            None,
            ev::SPORTS_FAIR_AEROBICS,
        );
        assert!(yes.is_some());
        assert!(active(
            &inputs(at(2026, 3, day + 1, 10)),
            None,
            ev::SPORTS_FAIR_AEROBICS
        )
        .is_none());
        assert!(active(&inputs(at(2026, 3, day, 10)), None, ev::SPRING_EQUINOX).is_some());
    }

    #[test]
    fn birthday_row_reads_the_save_slot() {
        let now = at(2026, 11, 3, 12);
        let inp = inputs(now);
        assert!(active(&inp, Some((11, 3)), ev::PLAYER_BIRTHDAY).is_some());
        assert!(active(&inp, Some((11, 4)), ev::PLAYER_BIRTHDAY).is_none());
        assert!(active(&inp, None, ev::PLAYER_BIRTHDAY).is_none());
    }

    #[test]
    fn unsupported_rows_stay_off() {
        for e in [
            ev::HANDBILL_SHOP_SALE,
            ev::GHOST,
            ev::BRIDGE_MAKE,
            ev::SONCHO_BRIDGE_MAKE,
        ] {
            assert!(!row_supported(e));
            assert!(active(&inputs(at(2026, 11, 3, 12)), None, e).is_none());
        }
    }

    #[test]
    fn after_n_day_crosses_month_and_year_edges() {
        let md = |m: u16, d: u16| (m << 8) | d;
        assert_eq!(after_n_day(2026, md(12, 31), 1), md(1, 1));
        assert_eq!(after_n_day(2026, md(1, 1), -1), md(12, 31));
        assert_eq!(after_n_day(2028, md(2, 28), 1), md(2, 29));
        assert_eq!(after_n_day(2026, md(2, 28), 1), md(3, 1));
        assert_eq!(after_n_day(2026, md(3, 1), -1), md(2, 28));
    }
}
