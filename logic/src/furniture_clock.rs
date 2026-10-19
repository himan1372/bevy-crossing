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
