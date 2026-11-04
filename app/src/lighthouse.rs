//! The lighthouse quest's data (`LightHouse_c`, `m_soncho.c`: `mSC_LightHouse_*`).
//!
//! Tortimer's winter job: in the middle of January or February he asks the player to look after the island's lighthouse
//! for a week. Talking to him (`aES2_TALK_LIGHTHOUSE_QUEST_START_1`) starts it: the date is kept
//! (`mSC_LightHouse_Quest_Start`) with the player's started bit. The quest's time (`mSC_LightHouse_get_period`): the day
//! it started is period 0, the next week period 1 (the lighthouse must be switched on in the evening of each of its seven
//! days), the following ten days period 2 (the reward), then it is over and the data is cleared when the next vacation
//! event is due (`mSC_LightHouse_Event_Start`).
//!
//! The port keeps the quest's data and its periods. The island and the switch are not built, so a started quest
//! cannot be carried through to its reward yet.

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
    fn dates_a_week_on_cross_the_month() {
        assert_eq!(after_days(2027, 1, 28, 7), (2, 4));
        assert_eq!(after_days(2027, 2, 25, 8), (3, 5));
        assert_eq!(after_days(2027, 12, 30, 2), (1, 1));
    }
}
