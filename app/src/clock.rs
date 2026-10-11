//! Game clock: the player's local date and time (as on the GameCube, the
//! game follows the real clock), plus a debug offset.
//!
//! F6 moves the clock forward one hour (debug) so schedules can be seen
//! without waiting. The full calendar engine from `logic::game_time` comes
//! with the roadmap's `game-clock` task; this module is the time source
//! everything reads in the meantime.

use std::sync::atomic::{AtomicI32, Ordering};

use bevy::prelude::*;
use bevy::text::FontSize;
use chrono::{Datelike, Local, TimeDelta, Timelike};

use crate::input::{Action, Actions};
use crate::states::GameplaySet;

/// Debug hours added to the real local time (wraps every 24 h).
static DEBUG_HOUR_OFFSET: AtomicI32 = AtomicI32::new(0);

/// A moment on the game clock.
#[derive(Clone, Copy, Debug)]
pub struct Now {
    /// 1-12.
    pub month: u8,
    /// 1-31.
    pub day: u8,
    /// 0-23.
    pub hour: u8,
    pub minute: u8,
    /// Seconds since midnight.
    pub day_secs: u32,
}

/// The current game time: local time plus the debug offset.
pub fn now() -> Now {
    let offset = DEBUG_HOUR_OFFSET.load(Ordering::Relaxed);
    let t = Local::now() + TimeDelta::hours(offset as i64);
    Now {
        month: t.month() as u8,
        day: t.day() as u8,
        hour: t.hour() as u8,
        minute: t.minute() as u8,
        day_secs: t.num_seconds_from_midnight(),
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

fn update_clock_text(mut text: Query<&mut Text, With<ClockText>>) {
    let n = now();
    let (h12, ampm) = match n.hour {
        0 => (12, "AM"),
        1..=11 => (n.hour, "AM"),
        12 => (12, "PM"),
        _ => (n.hour - 12, "PM"),
    };
    let offset = DEBUG_HOUR_OFFSET.load(Ordering::Relaxed);
    let label = if offset == 0 {
        format!("{h12}:{:02} {ampm}", n.minute)
    } else {
        format!("{h12}:{:02} {ampm} (debug +{offset}h)", n.minute)
    };
    if let Ok(mut t) = text.single_mut() {
        if t.0 != label {
            t.0 = label;
        }
    }
}

pub struct ClockPlugin;

impl Plugin for ClockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_clock_ui).add_systems(
            Update,
            (debug_advance_hour, update_clock_text)
                .chain()
                .in_set(GameplaySet),
        );
    }
}
