//! The station train's controller (`m_train_control.c`, `mTRC_trainControl`): how fast the train runs,
//! where it is on the track, and which of its nine actions it is in. Positions are game units along the
//! track (`train_position.x`), speeds game units per frame at 60 Hz (`0.5 * speed * dt` per frame).
//! The constants are the pinned Rustimal `train.rs`'s (the USA ones); the schedule (`mTRC_schedule`,
//! the timetable) is not here.

/// `mTRC_ACTION_*`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    SpawnMoving,
    BeginSlowdown,
    BeginStop,
    SignalStopped,
    WaitStopped,
    SignalStarting,
    BeginPullOut,
    SpeedUp,
}

impl Action {
    /// The number the actors compare (`train_action`): `ac_train1` picks its animation by it.
    pub fn number(self) -> u8 {
        self as u8
    }
}

/// `mTRC_SLOW_SPEED` / `mTRC_FAST_SPEED`.
pub const SLOW_SPEED: f32 = 2.6;
pub const FAST_SPEED: f32 = 7.2;
/// The rates `chase_f` moves the speed by per frame.
pub const SLOW_RATE: f32 = 0.210_000_01;
pub const STOP_RATE: f32 = 0.0084;
pub const START_RATE: f32 = 0.0047;
pub const SPEEDUP_RATE: f32 = 0.0047;
/// Past this the train starts to stop (`BEGIN_SLOWDOWN`), and when it is past this it is gone.
pub const STOP_AFTER_X: f32 = 2165.0;
pub const GONE_X: f32 = 4400.0;
/// `mTRC_demo_init` (the new game: the train comes in slowing) and `mTRC_mati_init` (waiting).
pub const DEMO_X: f32 = 2037.0;
pub const WAITING_X: f32 = 2367.0;
/// The track's z, in game units.
pub const TRACK_Z: f32 = 740.0;
/// Ticks the signal rings after the train has stopped / before it starts.
pub const STOPPED_TICKS: u8 = 48;
pub const STARTING_TICKS: u8 = 84;
pub const PULL_OUT_TICKS: u8 = 180;

/// `chase_f`: `value` moves towards `target` by at most `step`.
fn chase(value: &mut f32, target: f32, step: f32) {
    if (target - *value).abs() <= step {
        *value = target;
    } else if *value < target {
        *value += step;
    } else {
        *value -= step;
    }
}

/// The controller's state (`train_action`, `train_speed`, `train_position`, `train_timer`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Train {
    pub action: Action,
    pub speed: f32,
    pub x: f32,
    timer: f32,
    /// The player has left (`train_control_state` changed): a waiting train may start.
    pub depart: bool,
}

/// What happened in a step that the sound and the actors care about (`mTRC_KishaStatusTrg`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// The train has come to a stop (state 2).
    Stopped,
    /// The train begins to pull out (state 3).
    PullingOut,
    /// The train has left the field (state 4).
    Gone,
}

/// One native 60-Hz tick of ac_train0 and Na_KishaStatusLevel presentation.
#[derive(Default)]
pub struct Presentation {
    pub wheel: f32,
    saved_wheel: u8,
    smoke_wait: u8,
    shu: u16,
    tonton: u16,
}

#[derive(Default)]
pub struct Cues {
    pub smoke: bool,
    pub steam: Option<usize>,
    pub chuff: bool,
    pub clack: bool,
}

impl Presentation {
    pub fn tick(&mut self, speed: f32) -> Cues {
        if self.wheel == 0.0 {
            self.wheel = 1.0;
        }
        let frame = self.wheel as u8;
        let changed = frame != self.saved_wheel;
        self.saved_wheel = frame;
        let mut cues = Cues::default();
        if speed > 0.0 {
            if self.smoke_wait == 0 {
                cues.smoke = true;
                self.smoke_wait = if speed >= 1.0 {
                    (12.0 / speed) as u8
                } else {
                    12
                };
            } else {
                self.smoke_wait -= 1;
            }
            // aTR0_animation reports changes to the integer wheel frame only.
            if changed {
                cues.steam = match frame {
                    2..=5 => Some(0),
                    13..=16 => Some(1),
                    _ => None,
                };
            }
        }
        cues.chuff = self.shu == 4 && speed > 0.4;
        cues.clack = self.tonton == 10 && speed > 0.4;
        if self.shu > (20.0 - 2.5 * speed) as u16 {
            self.shu = 0;
        }
        if self.tonton > (90.0 - 11.5 * speed) as u16 {
            self.tonton = 0;
        }
        self.shu += 1;
        self.tonton += 1;
        self.wheel += (speed / 4.0).min(0.5);
        if self.wheel >= 25.0 {
            self.wheel -= 24.0;
        }
        cues
    }
}

impl Train {
    /// `mTRC_demo_init`: the new game's train, coming in at slow speed.
    pub fn demo() -> Self {
        Self {
            action: Action::BeginSlowdown,
            speed: SLOW_SPEED,
            x: DEMO_X,
            timer: 0.0,
            depart: false,
        }
    }

    /// `mTRC_mati_init`: a train standing at the station.
    pub fn waiting() -> Self {
        Self {
            action: Action::WaitStopped,
            speed: 0.0,
            x: WAITING_X,
            timer: 0.0,
            depart: false,
        }
    }

    /// One frame of `mTRC_trainControl` (`dt` is the number of 60 Hz frames it covers).
    pub fn step(&mut self, dt: f32) -> Option<Event> {
        let mut event = None;
        match self.action {
            Action::None => {}
            Action::SpawnMoving => {
                self.speed = FAST_SPEED;
                // The field's block 2 (`block_x >= 2`) starts at x = 1920 less the border: 640 per block.
                if self.x >= 1280.0 {
                    self.action = Action::BeginSlowdown;
                }
            }
            Action::BeginSlowdown => {
                chase(&mut self.speed, SLOW_SPEED, SLOW_RATE * dt);
                if self.x > STOP_AFTER_X {
                    self.action = Action::BeginStop;
                    self.speed = SLOW_SPEED;
                }
            }
            Action::BeginStop => {
                chase(&mut self.speed, 0.0, STOP_RATE * dt);
                if self.speed.abs() < 0.008 {
                    self.timer = f32::from(STOPPED_TICKS);
                    self.action = Action::SignalStopped;
                    self.speed = 0.0;
                    event = Some(Event::Stopped);
                }
            }
            Action::SignalStopped => {
                self.timer = (self.timer - dt).max(0.0);
                if self.timer <= 0.0 {
                    self.action = Action::WaitStopped;
                }
            }
            Action::WaitStopped => {
                if self.depart {
                    self.timer = f32::from(STARTING_TICKS);
                    self.action = Action::SignalStarting;
                }
            }
            Action::SignalStarting => {
                self.timer = (self.timer - dt).max(0.0);
                if self.timer <= 0.0 {
                    self.timer = f32::from(PULL_OUT_TICKS);
                    self.action = Action::BeginPullOut;
                    event = Some(Event::PullingOut);
                }
            }
            Action::BeginPullOut => {
                chase(&mut self.speed, SLOW_SPEED, START_RATE * dt);
                self.timer = (self.timer - dt).max(0.0);
                if self.timer <= 0.0 {
                    self.action = Action::SpeedUp;
                }
            }
            Action::SpeedUp => {
                chase(&mut self.speed, FAST_SPEED, SPEEDUP_RATE * dt);
                if self.x > GONE_X {
                    self.action = Action::None;
                    event = Some(Event::Gone);
                }
            }
        }
        if self.action != Action::None {
            self.x += 0.5 * self.speed * dt;
        }
        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_new_games_train_slows_and_stops_where_it_waits() {
        let mut train = Train::demo();
        let mut frames = 0;
        let mut stopped = false;
        while !stopped && frames < 2000 {
            stopped = train.step(1.0) == Some(Event::Stopped);
            frames += 1;
        }
        assert!(stopped, "it stops");
        assert_eq!(train.action, Action::SignalStopped);
        assert!(
            (train.x - WAITING_X).abs() < 40.0,
            "it stops near where a waiting train stands: {}",
            train.x
        );
    }

    #[test]
    fn a_waiting_train_pulls_out_when_it_is_let_go() {
        let mut train = Train::waiting();
        for _ in 0..10 {
            train.step(1.0);
        }
        assert_eq!(train.action, Action::WaitStopped);
        train.depart = true;
        let mut events = Vec::new();
        for _ in 0..4000 {
            if let Some(e) = train.step(1.0) {
                events.push(e);
            }
        }
        assert_eq!(events, [Event::PullingOut, Event::Gone]);
        assert_eq!(train.action, Action::None);
    }
}
