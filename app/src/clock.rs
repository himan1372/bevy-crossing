//! Game clock on the translated GameCube time stack
//! (`rustimal_logic::game_time`).
//!
//! As on the GameCube, the game clock is the hardware clock plus a saved
//! offset: `GameClock { time_delta }`, `game_time(hard_ticks)` =
//! `hard_ticks + time_delta` turned into a calendar by the OS tick maths
//! (`ticks_to_calendar_time`). The hardware clock here is the PC's local
//! time (the GameCube RTC also stores local time, with no time zone),
//! counted in GameCube timer ticks since 2000-01-01.
//!
//! - The game day rolls over at 6:00 am, not midnight (`game_day_ymd`);
//!   day numbers (quest day limits) count those game days.
//! - Seasons come from the 18-term calendar (`term_index` /
//!   `CALENDAR_TERMS`).
//! - F6 (debug) moves the clock forward one hour by changing `time_delta`,
//!   the same way the original's "set clock" works.
//!
//! `chrono` only supplies the local wall-clock time (Rust's std can't read
//! the Windows time zone).

use std::sync::atomic::{AtomicI32, Ordering};

use bevy::prelude::*;
use bevy::text::FontSize;
use rustimal_logic::game_time::{
    self, GameClock, RtcYmd, Season, CALENDAR_TERMS, GC_EPOCH_UNIX_DIFF, TIMER_CLOCK,
};

use crate::input::{Action, Actions};
use crate::states::GameplaySet;

/// Debug hours added through `time_delta` (wraps every 24 h).
static DEBUG_HOUR_OFFSET: AtomicI32 = AtomicI32::new(0);
/// Debug days added on top (`BEVY_CROSSING_DAY_OFFSET`), to try seasons.
static DEBUG_DAY_OFFSET: AtomicI32 = AtomicI32::new(0);

/// Day numbers count from this game day.
const DAY_ZERO: RtcYmd = RtcYmd {
    year: 2000,
    month: 1,
    day: 1,
};

/// A moment on the game clock.
#[derive(Clone, Copy, Debug)]
pub struct Now {
    /// 1-12.
    pub month: u8,
    /// 1-31.
    pub day: u8,
    /// 0 = Sunday.
    pub weekday: u8,
    /// 0-23.
    pub hour: u8,
    pub minute: u8,
    /// Seconds since midnight.
    pub day_secs: u32,
    /// Game days (6 am to 6 am) since 2000-01-01.
    pub day_number: i64,
    pub season: Season,
}

/// The PC's local time as GameCube hardware ticks since 2000-01-01.
fn hard_ticks() -> i64 {
    let local_secs = chrono::Local::now().naive_local().and_utc().timestamp();
    (local_secs - GC_EPOCH_UNIX_DIFF) * TIMER_CLOCK
}

/// Build `Now` from game-clock ticks (pure; used by tests too).
fn now_from(hard: i64, debug_hours: i32) -> Now {
    let clock = GameClock {
        time_delta: debug_hours as i64 * 3600 * TIMER_CLOCK,
    };
    let t = clock.game_time(hard);
    let day_number = game_time::interval_days_ymd(&DAY_ZERO, &game_time::game_day_ymd(&t)) as i64;
    let season = game_time::term_index(t.month, t.day, false)
        .map(|i| CALENDAR_TERMS[i].season)
        .unwrap_or(Season::Winter);
    Now {
        month: t.month,
        day: t.day,
        weekday: t.weekday,
        hour: t.hour,
        minute: t.min,
        day_secs: clock.now_seconds(hard) as u32,
        day_number,
        season,
    }
}

/// The current game time.
pub fn now() -> Now {
    let days = DEBUG_DAY_OFFSET.load(Ordering::Relaxed);
    now_from(
        hard_ticks(),
        DEBUG_HOUR_OFFSET.load(Ordering::Relaxed) + days * 24,
    )
}

pub fn season_name(season: Season) -> &'static str {
    match season {
        Season::Spring => "spring",
        Season::Summer => "summer",
        Season::Autumn => "fall",
        Season::Winter => "winter",
    }
}

#[derive(Component)]
struct ClockText;

fn setup_clock_ui(mut commands: Commands) {
    commands.spawn((
        ClockText,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(20.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            right: Val::Px(16.0),
            ..default()
        },
    ));
}

/// F6: one hour forward (debug).
fn debug_advance_hour(actions: Res<Actions>) {
    if actions.just_pressed(Action::DebugHour) {
        let h = (DEBUG_HOUR_OFFSET.load(Ordering::Relaxed) + 1) % 24;
        DEBUG_HOUR_OFFSET.store(h, Ordering::Relaxed);
        let n = now();
        println!("Debug clock: +{h}h -> {:02}:{:02}", n.hour, n.minute);
    }
}

/// "Sat Oct 10  9:05 PM".
fn clock_label(n: &Now, debug_hours: i32) -> String {
    const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let (h12, ampm) = match n.hour {
        0 => (12, "AM"),
        1..=11 => (n.hour, "AM"),
        12 => (12, "PM"),
        _ => (n.hour - 12, "PM"),
    };
    let mut label = format!(
        "{} {} {}  {h12}:{:02} {ampm}",
        DAYS[n.weekday as usize % 7],
        MONTHS[(n.month as usize).saturating_sub(1) % 12],
        n.day,
        n.minute
    );
    if debug_hours != 0 {
        label.push_str(&format!(" (debug +{debug_hours}h)"));
    }
    label
}

fn update_clock_text(mut text: Query<&mut Text, With<ClockText>>) {
    let label = clock_label(&now(), DEBUG_HOUR_OFFSET.load(Ordering::Relaxed));
    if let Ok(mut t) = text.single_mut() {
        if t.0 != label {
            t.0 = label;
        }
    }
}

pub struct ClockPlugin;

impl Plugin for ClockPlugin {
    fn build(&self, app: &mut App) {
        // BEVY_CROSSING_HOUR_OFFSET=N starts with N debug hours (as if F6
        // had been pressed N times), for side-by-side checks.
        if let Some(hours) = std::env::var("BEVY_CROSSING_HOUR_OFFSET")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
        {
            DEBUG_HOUR_OFFSET.store(hours.rem_euclid(24), Ordering::Relaxed);
        }
        // BEVY_CROSSING_DAY_OFFSET=N: N days ahead (another season's
        // critters and colours).
        if let Some(days) = std::env::var("BEVY_CROSSING_DAY_OFFSET")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
        {
            DEBUG_DAY_OFFSET.store(days, Ordering::Relaxed);
        }
        app.add_systems(Startup, setup_clock_ui).add_systems(
            Update,
            (debug_advance_hour, update_clock_text)
                .chain()
                .in_set(GameplaySet),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ticks for a local date/time.
    fn ticks(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> i64 {
        let secs = chrono::NaiveDate::from_ymd_opt(y, mo, d)
            .unwrap()
            .and_hms_opt(h, mi, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        (secs - GC_EPOCH_UNIX_DIFF) * TIMER_CLOCK
    }

    #[test]
    fn calendar_comes_out_right() {
        let n = now_from(ticks(2026, 10, 10, 21, 5), 0);
        assert_eq!((n.month, n.day, n.hour, n.minute), (10, 10, 21, 5));
        assert_eq!(n.weekday, 6, "10 Oct 2026 is a Saturday");
        assert_eq!(n.season, Season::Autumn);
        assert_eq!(clock_label(&n, 0), "Sat Oct 10  9:05 PM");
    }

    #[test]
    fn debug_offset_is_a_time_delta() {
        let n = now_from(ticks(2026, 10, 10, 23, 30), 2);
        assert_eq!((n.month, n.day, n.hour, n.minute), (10, 11, 1, 30));
    }

    #[test]
    fn game_day_rolls_over_at_6am() {
        let late = now_from(ticks(2026, 10, 10, 23, 0), 0).day_number;
        let early = now_from(ticks(2026, 10, 11, 5, 59), 0).day_number;
        let morning = now_from(ticks(2026, 10, 11, 6, 0), 0).day_number;
        assert_eq!(late, early, "before 6 am is still the previous game day");
        assert_eq!(morning, late + 1);
    }
}
