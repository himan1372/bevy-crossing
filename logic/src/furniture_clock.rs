//! m_time hand angles, ac_my_room's shared clock beat and common cKF repeat.

/// Original signed binary angles: one revolution is 65536 units. Include
/// seconds in both hands and wrap afternoon hours as the original s16 fields do.
pub fn hand_angles(hour: u8, minute: u8, second: u8) -> (i16, i16) {
    let within_hour = f32::from(second) + f32::from(minute) * 60.0;
    let minute = (within_hour / 3600.0 * 65536.0) as u32 as i16;
    let hour = ((within_hour + f32::from(hour) * 3600.0) / 43200.0 * 65536.0) as u32 as i16;
    (hour, minute)
}

/// cKF standard repeat: frames 1..end, speed 0.5 per original 60Hz tick.
pub fn advance_frame(frame: f32, end: i16, seconds: f32) -> f32 {
    if end <= 1 {
        1.0
    } else {
        1.0 + (frame - 1.0 + seconds * 30.0).rem_euclid(f32::from(end - 1))
    }
}

#[derive(Default, Clone, Copy)]
pub struct ClockPulse {
    pub reset: bool,
    pub tick: bool,
}

/// aMR_MakeClockInfo / aMR_ClockCrossedFrame; phase is shared by room clocks.
pub fn advance_beat(phase: &mut f32, seconds: f32) -> ClockPulse {
    let previous = *phase;
    *phase = (previous + seconds * 60.0).rem_euclid(120.0);
    let crossed = |target| {
        if previous <= *phase {
            previous < target && *phase >= target
        } else {
            previous < target || *phase >= target
        }
    };
    ClockPulse {
        reset: crossed(0.0),
        tick: crossed(15.0) || crossed(45.0),
    }
}

/// aSumLogHatoclk_mv `pappo_count_table`: calls on the hour (12 at 0 and 12).
pub const CUCKOO_CALLS: [i16; 24] = [
    12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
];
/// `GETREG(MYKREG, 33) + 2`: no code sets the register, so the call sounds
/// on frame 2.
pub const CUCKOO_SOUND_FRAME: f32 = 2.0;

/// The cuckoo clock (ac_sum_log_hatoclk): `dynamic_work_s[0]` state,
/// `[1]` remaining calls and its cKF standard-stop frame control.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cuckoo {
    pub state: u8,
    pub calls: i16,
    pub frame: f32,
    pub speed: f32,
}

impl Cuckoo {
    /// aSumLogHatoclk_ct: stop clip at speed 1, played once from frame 1.
    pub fn new(end: f32) -> Self {
        let mut cuckoo = Self {
            state: 0,
            calls: 0,
            frame: 1.0,
            speed: 1.0,
        };
        cuckoo.play(end);
        cuckoo
    }

    /// cKF_FrameControl_stop_proc + play: holds and reports the end frame.
    fn play(&mut self, end: f32) -> bool {
        if self.frame == end {
            return true;
        }
        if self.speed > 0.0 && self.frame < end && self.frame + self.speed >= end {
            self.frame = end;
            return true;
        }
        self.frame += self.speed;
        false
    }

    /// StopAnimeInitNormal(speed 1) followed by `current_frame = 1`.
    fn restart(&mut self) {
        self.speed = 1.0;
        self.frame = 1.0;
    }

    /// aSumLogHatoclk_mv for one 60 Hz frame; true when sound 0x133 starts.
    pub fn tick(&mut self, hour: u8, minute: u8, end: f32) -> bool {
        match self.state {
            0 => {
                if minute == 0 {
                    self.calls = CUCKOO_CALLS[usize::from(hour % 24)];
                    self.state = 1;
                    self.restart();
                } else if minute == 30 {
                    self.calls = 1;
                    self.state = 1;
                    self.restart();
                }
                false
            }
            1 => {
                let sound = self.frame == CUCKOO_SOUND_FRAME;
                if self.play(end) {
                    if self.speed == 0.0 {
                        self.restart();
                        self.calls -= 1;
                        if self.calls <= 0 {
                            self.state = 2;
                        }
                    } else {
                        self.speed = 0.0;
                    }
                }
                sound
            }
            _ => {
                if minute != 0 && minute != 30 {
                    self.state = 0;
                }
                false
            }
        }
    }
}
