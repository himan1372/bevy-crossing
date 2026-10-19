//! US ac_sum_tv01/ac_sum_tv02/ac_sum_fruittv01/ac_kon_snowtv callbacks.
//! Frames advance at 60 Hz only while powered. Each table entry lasts two ticks.
#[derive(Clone, Copy)]
pub enum Television {
    Tv01,
    Tv02,
    Fruit,
    Snow,
}

impl Television {
    pub fn level_sound(self) -> u8 {
        match self {
            Self::Tv01 => 5,
            Self::Tv02 => 4,
            Self::Fruit => 6,
            Self::Snow => 0x2B,
        }
    }

    fn animation(self) -> &'static [u8] {
        match self {
            Self::Tv01 => &[
                2, 2, 2, 3, 3, 3, 2, 2, 2, 3, 3, 3, 2, 2, 2, 3, 3, 3, 4, 4, 4, 4, 2, 2, 2, 2, 2,
            ],
            Self::Tv02 => &[2, 2, 3, 3, 3],
            Self::Fruit => &[2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4],
            Self::Snow => &[
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4,
                4, 4,
            ],
        }
    }

    pub fn advance(self, ticks: f32, seconds: f32) -> f32 {
        (ticks + seconds * 60.0).rem_euclid(self.animation().len() as f32 * 2.0)
    }

    /// Texture suffix, with front1 being the powered-off screen.
    pub fn screen(self, on: bool, ticks: f32) -> u8 {
        if on {
            self.animation()[(ticks as usize / 2) % self.animation().len()]
        } else {
            1
        }
    }
}
