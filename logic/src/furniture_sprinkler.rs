//! ac_nog_sprinkler (fNS_ct/mv/dw/dt), per original 60 Hz frame.
//!
//! The saved switch seeds a local on flag; water alpha starts at 100 or 0
//! and moves 4 per frame toward its target. A switch change is honoured only
//! on a frame when the alpha has settled, so a press during a fade does not
//! reach the local flag; the destructor writes the local flag back.

/// `sAdo_OngenTrgStart` on a local toggle: on 0x16, off 0x17.
pub const SE_ON: u16 = 0x16;
pub const SE_OFF: u16 = 0x17;
/// `sAdo_OngenPos(.., 0x5B, ..)` level sound while on.
pub const LEVEL_SOUND: u8 = 0x5B;
/// Water environment colour (alpha from the fade).
pub const WATER_ENV: [u8; 3] = [0, 180, 255];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sprinkler {
    /// `dynamic_work_s[0]`.
    pub on: bool,
    /// `dynamic_work_f[0]`, 0..100.
    pub alpha: f32,
    /// `dynamic_work_f[1]`: scroll counter, wraps at 1024.
    pub counter: f32,
}

impl Sprinkler {
    /// fNS_ct.
    pub fn new(switch_on: bool) -> Self {
        Self {
            on: switch_on,
            alpha: if switch_on { 100.0 } else { 0.0 },
            counter: 0.0,
        }
    }

    /// fNS_mv for one frame. `switch_changed` is the actor's
    /// `switch_changed_flag`; returns the toggle sound when the flag applied.
    pub fn tick(&mut self, switch_changed: bool) -> Option<u16> {
        self.counter += 1.0;
        while self.counter >= 1024.0 {
            self.counter -= 1024.0;
        }
        let target = if self.on { 100.0 } else { 0.0 };
        if self.alpha > target {
            self.alpha = (self.alpha - 4.0).max(target);
            None
        } else if self.alpha < target {
            self.alpha = (self.alpha + 4.0).min(target);
            None
        } else if switch_changed {
            self.on = !self.on;
            Some(if self.on { SE_ON } else { SE_OFF })
        } else {
            None
        }
    }

    /// fNS_dw: water alpha byte (`(u8)(int)alpha`; 100 outside play).
    pub fn water_alpha(&self) -> u8 {
        self.alpha as i32 as u8
    }

    /// fNS_dw: tile 1 scroll `-counter * 10` (tile 0 does not scroll).
    pub fn tile1_scroll(&self) -> i32 {
        -(self.counter as i32) * 10
    }
}
