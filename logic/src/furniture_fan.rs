//! US ac_nog_fan01.c: motor easing, cKF repeat and blade texture delay.
use crate::{furniture_rig::repeat_frame, room_light::add_calc};

#[derive(Clone, Copy, Debug)]
pub struct Fan {
    pub speed: f32,
    pub frame: f32,
    pub texture_frame: u8,
    texture_counter: i16,
}

impl Fan {
    /// The constructor restores powered fans at full speed, not mid-ramp.
    pub fn new(on: bool) -> Self {
        Self {
            speed: if on { 0.5 } else { 0.0 },
            frame: 1.0,
            texture_frame: 0,
            texture_counter: 0,
        }
    }

    /// One original 60 Hz update. US increments the counter even when the
    /// stopped (-1) texture delay disables changes; retain s16 wrapping.
    pub fn step(&mut self, on: bool, end: i16) {
        add_calc(
            &mut self.speed,
            if on { 0.5 } else { 0.0 },
            1.0 - (1.0_f32 - 0.05).sqrt(),
            0.25,
            0.0005,
        );
        self.frame = repeat_frame(self.frame, end, self.speed);
        let index = (9.0 - 2.0 * (self.speed * 9.0)) as usize;
        let delay = [1, 1, 1, 2, 2, 2, 2, 2, 3, -1][index];
        self.texture_counter = self.texture_counter.wrapping_add(1);
        if delay != -1 && self.texture_counter >= delay {
            self.texture_counter = 0;
            self.texture_frame = (self.texture_frame + 1) % 6;
        }
    }

    pub fn texture(self) -> usize {
        usize::from(self.texture_frame >> 1)
    }
}
