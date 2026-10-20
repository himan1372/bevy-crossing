//! ac_my_room's common furniture rig play (aMR_SystemAnimeCt_UniqueCt) and
//! the fNBC balloon speed ramp, per original 60Hz tick.

/// aMR furniture rigs play cKF standard repeat at speed 0.5.
pub const SYSTEM_RIG_SPEED: f32 = 0.5;

/// cKF standard repeat over frames 1..end at `speed` frames per tick.
pub fn repeat_frame(frame: f32, end: i16, speed: f32) -> f32 {
    if end <= 1 {
        1.0
    } else {
        1.0 + (frame - 1.0 + speed).rem_euclid(f32::from(end - 1))
    }
}

/// fNBC_ct/fNBC_mv dynamic_work_f[0] (speed) and [1] (target).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BalloonSpeed {
    pub speed: f32,
    pub target: f32,
}

impl Default for BalloonSpeed {
    fn default() -> Self {
        Self {
            speed: 0.0,
            target: 0.5,
        }
    }
}

impl BalloonSpeed {
    /// One fNBC_mv tick: a switch change aims for 1.25, reaching it settles
    /// back toward 0.5; speed approaches its target by 0.01 per tick.
    pub fn step(&mut self, switch_changed: bool) -> f32 {
        if switch_changed {
            self.target = 1.25;
        } else if self.speed >= 1.25 {
            self.target = 0.5;
        }
        if self.speed < self.target {
            self.speed = (self.speed + 0.01).min(self.target);
        } else if self.speed > self.target {
            self.speed = (self.speed - 0.01).max(self.target);
        }
        self.speed
    }
}

/// cKF standard stop: advance by `speed` and hold the last frame.
pub fn stop_frame(frame: f32, end: i16, speed: f32) -> f32 {
    (frame + speed).min(f32::from(end.max(1)))
}
