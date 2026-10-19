//! Audited US television callbacks, including ac_iku_turkey_TV.
//! Turkey advances even while off, with six ticks per texture-table entry;
//! the other sets advance only while powered, with two ticks per entry.
#[derive(Clone, Copy)]
pub enum Television {
    Tv01,
    Tv02,
    Fruit,
    Snow,
    Turkey,
}

impl Television {
    pub fn level_sound(self) -> u8 {
        match self {
            Self::Tv01 => 5,
            Self::Tv02 => 4,
            Self::Fruit => 6,
            Self::Snow => 0x2B,
            Self::Turkey => 0x5E,
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
            Self::Turkey => &[2, 3, 4, 3],
        }
    }

    fn ticks_per_entry(self) -> usize {
        if matches!(self, Self::Turkey) {
            6
        } else {
            2
        }
    }

    pub fn advance(self, ticks: f32, seconds: f32, on: bool) -> f32 {
        if on || matches!(self, Self::Turkey) {
            (ticks + seconds * 60.0)
                .rem_euclid((self.animation().len() * self.ticks_per_entry()) as f32)
        } else {
            ticks
        }
    }

    /// One-based cached texture index; the first texture is the off screen.
    pub fn screen(self, on: bool, ticks: f32) -> u8 {
        if on {
            self.animation()[(ticks as usize / self.ticks_per_entry()) % self.animation().len()]
        } else {
            1
        }
    }
}
