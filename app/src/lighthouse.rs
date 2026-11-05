//! The lighthouse quest's data (`LightHouse_c`, `m_soncho.c`: `mSC_LightHouse_*`).
//!
//! Tortimer's winter job: in the middle of January or February he asks the player to look after the island's lighthouse
//! for a week. Talking to him (`aES2_TALK_LIGHTHOUSE_QUEST_START_1`) starts it: the date is kept
//! (`mSC_LightHouse_Quest_Start`) with the player's started bit. The quest's time (`mSC_LightHouse_get_period`): the day
//! it started is period 0, the next week period 1 (the lighthouse must be switched on in the evening of each of its seven
//! days), the following ten days period 2 (the reward), then it is over and the data is cleared when the next vacation
//! event is due (`mSC_LightHouse_Event_Start`).
//!
//! The port keeps the quest's data and its periods, the switch days (`mSC_LightHouse_Switch_On`, `_Switch_Check`,
//! `_In_Check`) and the reward (`_Event_Check`, `_Event_Clear`, `_Event_Present_Item`); `toudai.rs` is the lighthouse that
//! throws the switch and the present demo that gives the reward.

use serde::{Deserialize, Serialize};

/// The days of the quest's weeks (`lbRTC_WEEK` and `mSC_LIGHTHOUSE_DAYS`).
const WEEK: i64 = 7;
const LIGHTHOUSE_DAYS: i64 = 17;

/// `mSC_LIGHTHOUSE_PERIOD_*`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Period {
    Start,
    Week,
    Reward,
    None,
}

/// `mSC_LIGHTHOUSE_EVENT_*`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    None,
    JanStarted,
    JanContributed,
    FebStarted,
    FebContributed,
}

/// The quest's save (`LightHouse_c`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LightHouse {
    /// `renew_time`: the day the quest started (year 0: never).
    pub renew: (u16, u8, u8),
    /// `players_quest_started`: bit 0..3 the players who started it, bit 4..7 who switched the lighthouse on.
    pub started: u8,
    /// `days_switched_on`: bit n is the quest's day n.
    pub switched: u8,
    /// `players_completed`.
    pub completed: u8,
}

/// Days since 1970-01-01 of a civil date (the proleptic Gregorian calendar).
fn civil_days(y: i32, m: u32, d: u32) -> i64 {
    let y = i64::from(if m <= 2 { y - 1 } else { y });
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = i64::from((m + 9) % 12);
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The civil date of `days` since 1970-01-01.
fn civil_date(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((y + i64::from(m <= 2)) as i32, m, d)
}

/// The month and day `n` days after a date.
pub(crate) fn after_days(y: u16, m: u8, d: u8, n: i64) -> (u8, u8) {
    let (_, month, day) = civil_date(civil_days(i32::from(y), u32::from(m), u32::from(d)) + n);
    (month as u8, day as u8)
}

impl LightHouse {
    fn start_days(&self) -> Option<i64> {
        let (y, m, d) = self.renew;
        (d != 0).then(|| civil_days(i32::from(y), u32::from(m), u32::from(d)))
    }

    /// `mSC_LightHouse_get_period`.
    pub fn period(&self, now: (u16, u8, u8)) -> Period {
        let Some(start) = self.start_days() else {
            return Period::None;
        };
        let today = civil_days(i32::from(now.0), u32::from(now.1), u32::from(now.2));
        let since = today - start;
        if since < 0 {
            Period::None
        } else if since == 0 {
            Period::Start
        } else if since <= WEEK {
            Period::Week
        } else if since <= LIGHTHOUSE_DAYS {
            Period::Reward
        } else {
            Period::None
        }
    }

    /// `mSC_LightHouse_Event_Start`: whether the vacation event may run (the data is cleared when an old quest is over).
    pub fn event_start(&mut self, now: (u16, u8, u8)) -> bool {
        if self.renew.2 == 0 {
            return true;
        }
        if self.period(now) != Period::None {
            return false;
        }
        *self = Self::default();
        true
    }

    /// `mSC_LightHouse_Talk_After_Check`: a quest is running and somebody has started it.
    pub fn talk_after(&self, now: (u16, u8, u8)) -> bool {
        self.period(now) != Period::None && self.started & 0x0F != 0
    }

    /// `mSC_LightHouse_day`: the quest's day (0 on the first, up to 6) of a date.
    pub fn day(&self, now: (u16, u8, u8)) -> u8 {
        let Some(start) = self.start_days() else {
            return 0;
        };
        let today = civil_days(i32::from(now.0), u32::from(now.1), u32::from(now.2));
        if today < start {
            return 0;
        }
        let days = today - start - 1;
        days.clamp(0, WEEK - 1) as u8
    }

    /// `mSC_LightHouse_Switch_Check` (the date six hours back, as the lamp's day starts at 06:00): the lamp may be on at
    /// night, either because no quest is running (it lights by itself) or because today's switch was thrown.
    pub fn switch_check(&self, now: (u16, u8, u8), hour: u8) -> bool {
        if self.renew.2 == 0 {
            return true;
        }
        let back = if hour < 6 { -1 } else { 0 };
        let (m, d) = after_days(now.0, now.1, now.2, back);
        let year = if back < 0 && now.1 == 1 && now.2 == 1 {
            now.0 - 1
        } else {
            now.0
        };
        let adjusted = (year, m, d);
        if self.period(adjusted) != Period::Week {
            return true;
        }
        self.switched & (1 << self.day(adjusted)) != 0
    }

    /// `mSC_LightHouse_In_Check`: in the quest's week, 18:00-21:59, today's switch not thrown yet.
    pub fn in_check(&self, now: (u16, u8, u8), hour: u8) -> bool {
        if self.renew.2 == 0 || self.period(now) != Period::Week || !(18..22).contains(&hour) {
            return false;
        }
        self.switched & (1 << self.day(now)) == 0
    }

    /// `mSC_LightHouse_Switch_On` for the one player: today's bit and the player's own (bit 4).
    pub fn switch_on(&mut self, now: (u16, u8, u8)) {
        self.switched |= 1 << self.day(now);
        self.started |= 1 << 4;
    }

    /// `mSC_LightHouse_Event_Check` for the one player.
    pub fn event_check(&self, now: (u16, u8, u8)) -> Event {
        if self.period(now) != Period::Reward || self.completed & 1 != 0 {
            return Event::None;
        }
        let january = self.renew.1 == 1;
        if self.switched & 0x7F == 0x7F {
            if self.started & (1 << 4) != 0 {
                return if january {
                    Event::JanContributed
                } else {
                    Event::FebContributed
                };
            }
        } else if self.started & 1 != 0 {
            return if january {
                Event::JanStarted
            } else {
                Event::FebStarted
            };
        }
        Event::None
    }

    /// `mSC_LightHouse_Event_Clear`.
    pub fn event_clear(&mut self, now: (u16, u8, u8)) {
        self.completed |= 1;
        if matches!(
            self.event_check(now),
            Event::JanContributed | Event::JanStarted
        ) {
            self.completed |= 0x0F;
        }
    }

    /// `mSC_LightHouse_Event_Present_Item`: `FTR_TAK_TOUDAI` in January, `FTR_IKE_JNY_MAKADA01` in February.
    pub fn present_item(&self) -> u16 {
        if self.renew.1 == 1 {
            0x3070
        } else {
            0x1FA4
        }
    }

    /// `mSC_LightHouse_Quest_Start` for the one player.
    pub fn quest_start(&mut self, now: (u16, u8, u8)) {
        *self = Self {
            renew: now,
            started: 1,
            ..Self::default()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_quest_periods_follow_the_start_day() {
        let mut lh = LightHouse::default();
        assert!(lh.event_start((2027, 1, 16)));
        lh.quest_start((2027, 1, 16));
        assert_eq!(lh.period((2027, 1, 16)), Period::Start);
        assert_eq!(lh.period((2027, 1, 20)), Period::Week);
        assert_eq!(lh.period((2027, 1, 23)), Period::Week);
        assert_eq!(lh.period((2027, 1, 24)), Period::Reward);
        assert_eq!(lh.period((2027, 2, 2)), Period::Reward);
        assert_eq!(lh.period((2027, 2, 3)), Period::None);
        assert!(lh.talk_after((2027, 1, 17)));
        // A running quest keeps the vacation event off; an old one is cleared.
        assert!(!lh.clone().event_start((2027, 1, 20)));
        assert!(lh.event_start((2027, 2, 12)));
        assert_eq!(lh, LightHouse::default());
    }

    #[test]
    fn the_switch_days_and_the_reward() {
        let mut lh = LightHouse::default();
        lh.quest_start((2027, 1, 16));
        assert_eq!(lh.day((2027, 1, 16)), 0);
        assert_eq!(lh.day((2027, 1, 17)), 0);
        assert_eq!(lh.day((2027, 1, 18)), 1);
        assert_eq!(lh.day((2027, 1, 23)), 6);
        assert!(lh.in_check((2027, 1, 18), 19));
        assert!(!lh.in_check((2027, 1, 18), 10));
        // At night before the switch the lamp is dark; afterwards it shines.
        assert!(!lh.switch_check((2027, 1, 18), 20));
        lh.switch_on((2027, 1, 18));
        assert!(lh.switch_check((2027, 1, 18), 20));
        assert!(!lh.in_check((2027, 1, 18), 19));
        // After 1 a.m. the lamp's day is still the evening's.
        assert!(lh.switch_check((2027, 1, 19), 2));
        // Seven switches earn the present; one only the "started" line.
        assert_eq!(lh.event_check((2027, 1, 25)), Event::JanStarted);
        for d in 17..24 {
            lh.switch_on((2027, 1, d));
        }
        assert_eq!(lh.event_check((2027, 1, 25)), Event::JanContributed);
        assert_eq!(lh.present_item(), 0x3070);
        lh.event_clear((2027, 1, 25));
        assert_eq!(lh.event_check((2027, 1, 25)), Event::None);
    }

    #[test]
    fn dates_a_week_on_cross_the_month() {
        assert_eq!(after_days(2027, 1, 28, 7), (2, 4));
        assert_eq!(after_days(2027, 2, 25, 8), (3, 5));
        assert_eq!(after_days(2027, 12, 30, 2), (1, 1));
    }
}
