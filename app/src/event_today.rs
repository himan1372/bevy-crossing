//! Today's events, from the original scheduler (`m_event.c`).
//!
//! [`TodayEvents`] holds what `rustimal_logic::event_schedule` says is active on the game clock's
//! date: it is refreshed when the date changes (the weekly visitor is picked first, as
//! `init_weekly_event` does) and logged whenever the set of events or an event's active hours
//! change. Systems ask [`TodayEvents::is_active`] for the event they care about.
//!
//! Not saved yet: the weekly state (which visitor, Gulliver's day) is rebuilt at start-up, so
//! Gulliver's weekday can differ after a restart. The bridge and Blanca inputs are fixed to "no" until
//! their state exists, and the rows listed in `event_schedule::row_supported` stay off.

use bevy::prelude::*;
use rustimal_logic::event_schedule::{
    self as sch, ActiveEvent, Inputs, Now, WeeklyContext, WeeklyState,
};

use crate::clock;
use crate::states::GameplaySet;

#[derive(Resource, Default)]
pub struct TodayEvents {
    /// The date `events` was computed for (year, month, day); `None` before the first update.
    date: Option<(u16, u8, u8)>,
    weekly: WeeklyState,
    events: Vec<ActiveEvent>,
    /// Last hour whose active set was logged.
    logged_hour: Option<u8>,
}

// `is_active` / `all` are for the event NPCs (K.K. Slider is the first); nothing reads them yet.
#[allow(dead_code)]
impl TodayEvents {
    /// Whether `event` (an `event_schedule::ev` id) is running during `hour` today.
    pub fn is_active(&self, event: u16, hour: u8) -> bool {
        self.events
            .iter()
            .any(|a| a.event == event && a.active_at(hour))
    }

    /// Events scheduled today at any hour.
    pub fn all(&self) -> &[ActiveEvent] {
        &self.events
    }
}

fn to_schedule_now(n: &clock::Now) -> Now {
    Now {
        year: n.year,
        month: n.month,
        day: n.day,
        hour: n.hour,
        weekday: n.weekday,
    }
}

fn describe(events: &[ActiveEvent], hour: u8) -> String {
    let names: Vec<String> = events
        .iter()
        .filter(|a| a.active_at(hour))
        .map(|a| sch::event_name(a.event).to_string())
        .collect();
    if names.is_empty() {
        "none".to_string()
    } else {
        names.join(", ")
    }
}

fn refresh(mut today: ResMut<TodayEvents>) {
    let n = clock::now();
    let now = to_schedule_now(&n);
    let date = (n.year, n.month, n.day);
    let new_day = today.date != Some(date);
    if new_day {
        // `update_schedule_today` runs after the weekly picker on a new day.
        let mut weekly = today.weekly;
        sch::init_weekly_event(&now, &mut weekly, &WeeklyContext::default());
        today.weekly = weekly;
        let inputs = Inputs {
            now,
            town_day: 0,
            dates: [0; sch::save_date::NUM],
            weekly,
        };
        // The port keeps no birthday yet.
        today.events = sch::events_today(&inputs, None);
        today.date = Some(date);
        today.logged_hour = None;
        println!(
            "Events {:04}-{:02}-{:02}: {}",
            n.year,
            n.month,
            n.day,
            today
                .events
                .iter()
                .map(|a| sch::event_name(a.event))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if today.logged_hour != Some(n.hour) {
        let previous = today.logged_hour;
        today.logged_hour = Some(n.hour);
        let now_active = describe(&today.events, n.hour);
        let before = previous.map(|h| describe(&today.events, h));
        if before.as_deref() != Some(now_active.as_str()) {
            println!("Events active at {:02}:00: {now_active}", n.hour);
        }
    }
}

pub struct EventTodayPlugin;

impl Plugin for EventTodayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TodayEvents>()
            .add_systems(Update, refresh.in_set(GameplaySet));
    }
}
