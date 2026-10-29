//! The clock window of the new-game intro (`m_timeIn_ovl.c`, `mTI_OVL_OPEN_INITIAL`): the values the
//! player edits, the rules that keep them a real date, and the sweep of the clock's hands to the new time.

use crate::game_time::rtc_week;

/// `mTI_IDX_*`: the fields in the order A steps through them; `OK` follows the last.
pub const HOUR: usize = 0;
pub const MINUTE: usize = 1;
pub const MONTH: usize = 2;
pub const DAY: usize = 3;
pub const YEAR: usize = 4;
pub const OK: usize = 5;
/// `mTI_MIN_YEAR` / `mTI_MAX_YEAR` (the PC build's): the year is two digits after 2000.
pub const MIN_YEAR: u16 = 1;
pub const MAX_YEAR: u16 = 99;
/// `GAME_YEAR_MIN`.
const YEAR_BASE: u16 = 2000;

/// `timeIn_ovl->values`: hour, minute, month, day, year (two digits).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Values(pub [u16; 5]);

impl Values {
    /// `mTI_timeIn_ovl_init`: the window starts at the current time (the year clamped to the range).
    pub fn new(year: u16, month: u8, day: u8, hour: u8, minute: u8) -> Self {
        let year = year.saturating_sub(YEAR_BASE).clamp(MIN_YEAR, MAX_YEAR);
        Self([
            u16::from(hour),
            u16::from(minute),
            u16::from(month),
            u16::from(day),
            year,
        ])
    }

    pub fn year(&self) -> u16 {
        YEAR_BASE + self.0[YEAR]
    }

    pub fn month(&self) -> u8 {
        self.0[MONTH] as u8
    }

    pub fn day(&self) -> u8 {
        self.0[DAY] as u8
    }

    pub fn hour(&self) -> u8 {
        self.0[HOUR] as u8
    }

    pub fn minute(&self) -> u8 {
        self.0[MINUTE] as u8
    }

    /// `lbRTC_Week` of the date shown (0 = Sunday).
    pub fn weekday(&self) -> u8 {
        rtc_week(self.year(), self.month().max(1), self.day().max(1))
    }

    /// `lbRTC_IsValidTime` for these values.
    pub fn valid(&self) -> bool {
        let (month, day) = (self.month(), self.day());
        if !(1..=12).contains(&month) || day == 0 || self.hour() > 23 || self.minute() > 59 {
            return false;
        }
        u16::from(day) <= days_in_month(self.year(), month)
    }

    /// `mTI_get_now_min`: the minutes on a twelve hour dial.
    pub fn dial_minutes(&self) -> u16 {
        self.0[MINUTE] + (self.0[HOUR] % 12) * 60
    }

    /// The up / down buttons on field `sel` (`mTI_move_Play`, C-up / C-down), then the corrections that
    /// keep it a real date.
    pub fn change(&mut self, sel: usize, up: bool) {
        if sel >= OK {
            return;
        }
        let mut v = *self;
        if up {
            v.0[sel] += 1;
            if sel == YEAR && v.0[YEAR] > MAX_YEAR {
                v.0[YEAR] = MIN_YEAR;
            }
        } else if v.0[sel] != 0 {
            v.0[sel] -= 1;
            if sel == YEAR && v.0[YEAR] < MIN_YEAR {
                v.0[YEAR] = MAX_YEAR;
            }
        } else if sel == HOUR {
            v.0[HOUR] = 23;
        } else if sel == MINUTE {
            v.0[MINUTE] = 59;
        }
        if !v.valid() {
            if sel == YEAR {
                v.0[DAY] = 28;
            } else if (sel == MONTH && (1..13).contains(&v.0[MONTH])) || (!up && sel == DAY) {
                let mut day = 31;
                while day > 1 && day > days_in_month(v.year(), v.month().clamp(1, 12)) {
                    day -= 1;
                }
                v.0[DAY] = day;
            } else if up {
                v.0[sel] = if sel == MONTH || sel == DAY { 1 } else { 0 };
            } else if sel == MONTH {
                v.0[MONTH] = 12;
            }
        }
        *self = v;
    }
}

/// `lbRTC_IS_LEAPYEAR`.
fn leap(year: u16) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn days_in_month(year: u16, month: u8) -> u16 {
    match month {
        2 if leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// `mTI_set_time`: after the hour or minute was changed and the player moves on to OK, the clock's
/// hands sweep round (twelve minutes of dial per tick) until they reach the new time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sweep {
    /// `saved_min`: where the hands are, in minutes on the twelve hour dial.
    pub minutes: u16,
    started: bool,
}

impl Sweep {
    pub fn new(dial_minutes: u16) -> Self {
        Self {
            minutes: dial_minutes % 720,
            started: true,
        }
    }

    /// Begins a sweep (`init_flag = FALSE`): the first tick only arms it.
    pub fn restart(&mut self) {
        self.started = false;
    }

    /// One 60 Hz tick towards `target` dial minutes; true when the hands have arrived.
    pub fn tick(&mut self, target: u16) -> bool {
        let old = self.minutes;
        let mut now = old + 12;
        if now >= 720 {
            now -= 720;
        }
        let reached =
            (old < target && target <= now) || (old > now && (old < target || target <= now));
        let mut arrived = false;
        if reached {
            if self.started {
                now = target;
                arrived = true;
            } else {
                now = old;
            }
        }
        self.started = true;
        self.minutes = now;
        arrived
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(year: u16, month: u8, day: u8, hour: u8, minute: u8) -> Values {
        Values::new(year, month, day, hour, minute)
    }

    #[test]
    fn it_starts_at_the_time_it_is_given() {
        let v = values(2026, 10, 29, 15, 5);
        assert_eq!(
            (v.year(), v.month(), v.day(), v.hour(), v.minute()),
            (2026, 10, 29, 15, 5)
        );
        assert_eq!(v.weekday(), 4, "a Thursday");
        assert_eq!(v.dial_minutes(), 3 * 60 + 5);
    }

    #[test]
    fn the_hours_and_minutes_wrap() {
        let mut v = values(2026, 10, 29, 23, 59);
        v.change(HOUR, true);
        v.change(MINUTE, true);
        assert_eq!((v.hour(), v.minute()), (0, 0));
        v.change(HOUR, false);
        v.change(MINUTE, false);
        assert_eq!((v.hour(), v.minute()), (23, 59));
    }

    #[test]
    fn the_day_follows_the_month() {
        let mut v = values(2026, 1, 31, 12, 0);
        v.change(MONTH, true);
        assert_eq!(
            (v.month(), v.day()),
            (2, 28),
            "February has 28 days in 2026"
        );
        let mut v = values(2028, 1, 31, 12, 0);
        v.change(MONTH, true);
        assert_eq!(v.day(), 29, "and 29 in a leap year");
        let mut v = values(2026, 4, 30, 12, 0);
        v.change(DAY, true);
        assert_eq!(v.day(), 1, "one past the end goes back to the 1st");
        v.change(DAY, false);
        assert_eq!(v.day(), 30, "and one before the 1st goes to the last day");
    }

    #[test]
    fn the_year_stays_in_range() {
        let mut v = values(2099, 6, 1, 12, 0);
        v.change(YEAR, true);
        assert_eq!(v.year(), 2001);
        v.change(YEAR, false);
        assert_eq!(v.year(), 2099);
        let mut v = values(2028, 2, 29, 12, 0);
        v.change(YEAR, true);
        assert_eq!((v.year(), v.day()), (2029, 28));
    }

    #[test]
    fn the_hands_sweep_to_the_new_time() {
        let mut sweep = Sweep::new(0);
        sweep.restart();
        let mut ticks = 0;
        while !sweep.tick(305) {
            ticks += 1;
            assert!(ticks < 100);
        }
        assert_eq!(sweep.minutes, 305);
        assert!(ticks >= 20, "twelve minutes a tick: {ticks}");
    }
}
