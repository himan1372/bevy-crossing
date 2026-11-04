//! Today's events, from the original scheduler (`m_event.c`).
//!
//! [`TodayEvents`] holds what `rustimal_logic::event_schedule` says is active on the game clock's
//! date: it is refreshed when the date changes (the weekly visitor is picked first, as
//! `init_weekly_event` does) and logged whenever the set of events or an event's active hours
//! change. Systems ask [`TodayEvents::is_active`] for the event they care about.
//!
//! The weekly state (which visitor, Gulliver's day) is saved as `SaveData::weekly_event` and restored at
//! start-up and on F9, so Gulliver keeps his weekday. The bridge and Blanca inputs are fixed to "no" until
//! their state exists, and the rows listed in `event_schedule::row_supported` stay off.

use bevy::prelude::*;
use rustimal_logic::event_schedule::{
    self as sch, ActiveEvent, Inputs, Now, WeeklyContext, WeeklyState,
};
use serde::{Deserialize, Serialize};

use crate::clock;
use crate::save::LoadedSave;
use crate::states::GameplaySet;

/// The weekly-visitor state as saved (`dates[mEv_SAVE_DATE_WEEKLY]`, `weekly_event`, `dozaemon_completed`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeeklySave {
    #[serde(default)]
    pub date: u16,
    #[serde(default)]
    pub kind: Option<u16>,
    #[serde(default)]
    pub flags: Option<u16>,
    #[serde(default)]
    pub gulliver_completed: bool,
    /// `mEv_dozaemon_c.flags` (`ac_ev_dozaemon.h`): woke up, gave the gift, gave it on the first talk.
    #[serde(default)]
    pub dozaemon_flags: u16,
}

impl From<WeeklySave> for WeeklyState {
    fn from(w: WeeklySave) -> Self {
        Self {
            date: w.date,
            kind: w.kind,
            flags: w.flags,
            gulliver_completed: w.gulliver_completed,
        }
    }
}

#[derive(Resource, Default)]
pub struct TodayEvents {
    /// The date `events` was computed for (year, month, day); `None` before the first update.
    date: Option<(u16, u8, u8)>,
    weekly: WeeklyState,
    /// `mEv_dozaemon_c.flags`; cleared whenever a new Gulliver day is rolled.
    dozaemon_flags: u16,
    events: Vec<ActiveEvent>,
    /// Last hour whose active set was logged.
    logged_hour: Option<u8>,
    /// The special event (`special_event.type`, `dates[SPECIAL0..3]`) `events` was computed with.
    special: Option<(u16, [u16; 4])>,
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

    /// Whether Gulliver has finished his errand this week (`dozaemon_completed`).
    pub fn gulliver_completed(&self) -> bool {
        self.weekly.gulliver_completed
    }

    /// Gulliver's event flags (`aEDZ_SAVE_FLAG_*`).
    pub fn dozaemon_flags(&self) -> u16 {
        self.dozaemon_flags
    }

    pub fn set_dozaemon_flags(&mut self, flags: u16) {
        self.dozaemon_flags = flags;
    }

    /// `Save_Get(event_save_common).dozaemon_completed = TRUE`.
    pub fn complete_gulliver(&mut self) {
        self.weekly.gulliver_completed = true;
    }

    /// The weekly state to write into the save.
    pub fn weekly_save(&self) -> WeeklySave {
        WeeklySave {
            date: self.weekly.date,
            kind: self.weekly.kind,
            flags: self.weekly.flags,
            gulliver_completed: self.weekly.gulliver_completed,
            dozaemon_flags: self.dozaemon_flags,
        }
    }

    /// Takes the saved weekly state (`None`: a save from before it existed, treated as never set) and
    /// forces today's events to be computed again from it.
    pub fn restore_weekly(&mut self, saved: Option<WeeklySave>) {
        let saved = saved.unwrap_or_default();
        self.dozaemon_flags = saved.dozaemon_flags;
        self.weekly = saved.into();
        self.date = None;
        self.events.clear();
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

pub fn refresh(mut today: ResMut<TodayEvents>, shop: Option<Res<crate::shop::ShopData>>) {
    let n = clock::now();
    let now = to_schedule_now(&n);
    let date = (n.year, n.month, n.day);
    let new_day = today.date != Some(date);
    // The shop's calendar holds the rolled special event (`init_special_event`).
    let special = shop.and_then(|s| s.events.special_event());
    if new_day {
        // `update_schedule_today` runs after the weekly picker on a new day.
        let mut weekly = today.weekly;
        let rolled_from = weekly.date;
        sch::init_weekly_event(&now, &mut weekly, &WeeklyContext::default());
        today.weekly = weekly;
        if weekly.date != rolled_from {
            // A new Gulliver day: his event save area starts empty.
            today.dozaemon_flags = 0;
        }
    }
    if new_day || today.special != special {
        let mut dates = [0; sch::save_date::NUM];
        if let Some((_, slots)) = special {
            dates[sch::save_date::SPECIAL0] = slots[0];
            dates[sch::save_date::SPECIAL1] = slots[1];
            dates[sch::save_date::SPECIAL2] = slots[2];
            dates[sch::save_date::SPECIAL3] = slots[3];
        }
        let inputs = Inputs {
            now,
            town_day: 0,
            dates,
            weekly: today.weekly,
            special_type: special.map(|(event, _)| event),
        };
        // The port keeps no birthday yet.
        today.events = sch::events_today(&inputs, None);
        today.date = Some(date);
        today.special = special;
        today.logged_hour = None;
        println!(
            "Weekly visitor {} on {:02}-{:02}{}",
            today.weekly.kind.map_or("none", sch::event_name),
            today.weekly.date >> 8,
            today.weekly.date & 0xFF,
            if today.weekly.gulliver_completed {
                " (Gulliver done)"
            } else {
                ""
            }
        );
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

/// Startup: restore the weekly visitor from the loaded save.
fn apply_loaded_weekly(loaded: Res<LoadedSave>, mut today: ResMut<TodayEvents>) {
    if let Some(save) = loaded.0.as_ref() {
        today.restore_weekly(save.weekly_event);
    }
}

pub struct EventTodayPlugin;

impl Plugin for EventTodayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TodayEvents>()
            .add_systems(Startup, apply_loaded_weekly)
            .add_systems(Update, refresh.in_set(GameplaySet));
    }
}
