//! m_banti: the outdoor clock/date window that fades in while the player
//! stands still, per original 60 Hz frame.
//!
//! banti_move commits a change of the player's "addressable" display type
//! after a delay (30 frames into TRUE, 50 out of TRUE or into READY_NET,
//! talking at once), banti_calc_disp_alpha_rate eases the alpha, and
//! banti_chk_disp_left moves the window left when the player stands in the
//! lower right of the screen. Each date/time digit flips through its cKF
//! clip (frames 1..15 at 0.5, stop) when the shown value differs from the
//! clock; the displayed time catches up when every flip has finished.

use crate::room_light::add_calc;

/// `mPlayer_ADDRESSABLE_*`, from the player's main state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Addressable {
    /// Standing (WAIT) and some tool actions: the window may show.
    #[default]
    True,
    FalseMovement,
    FalseTalking,
    FalseReadyNet,
}

/// The clock fields the window shows (`lbRTC_time_c`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BantiTime {
    pub month: u8,
    pub day: u8,
    /// 0 = Sunday.
    pub weekday: u8,
    pub hour: u8,
    pub min: u8,
}

/// Digit clips: month, day, hour tens, hour ones, minute tens, minute ones.
pub const MONTH: usize = 0;
pub const DAY: usize = 1;
pub const HOUR_UPPER: usize = 2;
pub const HOUR_LOWER: usize = 3;
pub const MIN_UPPER: usize = 4;
pub const MIN_LOWER: usize = 5;

/// banti_animation_init_hiduke/jikan: start 1, end 15, speed 0.5, stop.
const FLIP_END: f32 = 15.0;
const FLIP_SPEED: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flip {
    pub playing: bool,
    pub frame: f32,
    speed: f32,
}

impl Default for Flip {
    fn default() -> Self {
        Self {
            playing: false,
            frame: 1.0,
            speed: 0.0,
        }
    }
}

impl Flip {
    fn start(&mut self) {
        *self = Self {
            playing: true,
            frame: 1.0,
            speed: FLIP_SPEED,
        };
    }

    /// cKF_SkeletonInfo_R_play in stop mode; banti_anime_proc rewinds a
    /// clip that stopped.
    fn play(&mut self) {
        let stopped = if self.frame == FLIP_END {
            true
        } else if self.speed > 0.0 && self.frame < FLIP_END && self.frame + self.speed >= FLIP_END {
            self.frame = FLIP_END;
            true
        } else {
            self.frame += self.speed;
            false
        };
        if stopped {
            *self = Self::default();
        }
    }
}

/// 12-hour display value (0 and 12 show as 12).
pub fn hour12(hour: u8) -> u8 {
    match hour {
        0 | 12 => 12,
        h if h > 11 => h - 12,
        h => h,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Banti {
    addressable: Addressable,
    timer: f32,
    pub alpha: f32,
    pub disp_left: bool,
    move_left: bool,
    move_timer: f32,
    disabled: bool,
    pub flips: [Flip; 6],
    pub displayed: BantiTime,
    pub next: BantiTime,
}

/// One frame's inputs.
#[derive(Clone, Copy, Debug)]
pub struct BantiInput {
    pub addressable: Addressable,
    /// The player's projected position, 320 x 240, y down.
    pub player_screen: (f32, f32),
    /// `submenu.process_status == mSM_PROCESS_WAIT` (no menu open).
    pub submenu_idle: bool,
    /// mDemo_CheckDemo / title demo: fade out quickly.
    pub demo: bool,
    pub now: BantiTime,
}

impl Banti {
    /// banti_ct.
    pub fn new(now: BantiTime) -> Self {
        Self {
            addressable: Addressable::True,
            timer: 0.0,
            alpha: 0.0,
            disp_left: false,
            move_left: false,
            move_timer: 0.0,
            disabled: false,
            flips: [Flip::default(); 6],
            displayed: now,
            next: now,
        }
    }

    /// banti_move for one frame.
    pub fn tick(&mut self, input: &BantiInput) {
        self.chk_disp_left(input.player_screen);
        let wanted = input.addressable;
        if self.addressable != wanted {
            self.timer += 1.0;
            let update = if wanted == Addressable::FalseReadyNet {
                self.timer > 50.0
            } else if self.addressable != Addressable::True {
                self.timer > 30.0 || wanted == Addressable::FalseTalking
            } else {
                self.timer > 50.0 || wanted == Addressable::FalseTalking
            };
            if update {
                self.timer = 0.0;
                self.addressable = wanted;
            }
        } else {
            self.timer = 0.0;
        }
        self.calc_alpha(input);
        self.time_check(input.now);
    }

    fn chk_disp_left(&mut self, (x, y): (f32, f32)) {
        let lower_right = x >= 180.0 && y >= 150.0;
        if lower_right != self.move_left {
            self.move_left = lower_right;
            self.move_timer = 0.0;
        }
        self.move_timer += 1.0;
        if self.move_timer > 10.0 {
            self.move_timer = 10.0;
            if self.alpha < 0.01 {
                self.disp_left = lower_right;
                self.disabled = false;
            } else if self.disp_left != lower_right {
                // Fade out on the old side before moving.
                self.disabled = true;
            }
        }
    }

    fn calc_alpha(&mut self, input: &BantiInput) {
        let fraction = 1.0 - 0.8f32.sqrt();
        if input.demo || self.disabled {
            self.addressable = Addressable::FalseMovement;
            add_calc(&mut self.alpha, 0.0, fraction, 0.1, 0.005);
        } else if self.addressable == Addressable::True && input.submenu_idle {
            add_calc(&mut self.alpha, 1.0, fraction, 0.0425, 0.0005);
        } else {
            add_calc(&mut self.alpha, 0.0, fraction, 0.0425, 0.0005);
        }
    }

    fn any_playing(&self) -> bool {
        self.flips.iter().any(|f| f.playing)
    }

    /// banti_time_check.
    fn time_check(&mut self, now: BantiTime) {
        let pre = self.any_playing();
        if !pre {
            self.chk_anime_start(now);
        }
        for flip in &mut self.flips {
            if flip.playing {
                flip.play();
            }
        }
        let post = self.any_playing();
        // pre_state != post_state in the original compares before the
        // starts with after the plays.
        if pre != post {
            if !pre {
                self.next = now;
            } else {
                self.displayed = self.next;
            }
        }
    }

    /// banti_chk_anime_start.
    fn chk_anime_start(&mut self, now: BantiTime) {
        let shown = self.displayed;
        let (h0, h1) = (hour12(shown.hour), hour12(now.hour));
        let changes = [
            shown.month != now.month,
            shown.day != now.day,
            h0 / 10 != h1 / 10,
            h0 % 10 != h1 % 10,
            shown.min / 10 != now.min / 10,
            shown.min % 10 != now.min % 10,
        ];
        for (flip, changed) in self.flips.iter_mut().zip(changes) {
            if changed && !flip.playing {
                flip.start();
            }
        }
    }
}
