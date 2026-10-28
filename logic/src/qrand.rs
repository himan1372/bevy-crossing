//! `libc64/qrand.c`: the game's global linear congruential generator.
//!
//! `idum = idum * 0x19660D + 0x3C6EF35F` on every call. `fqrand` builds a float in
//! `[1, 2)` from the top 23 bits and subtracts 1; `RANDOM(n)` (`sys_math.h`) is
//! `(int)(fqrand() * (f32)n)`. The original keeps one global state seeded at boot
//! (`sqrand`); here the caller owns the state, so callers share one `Qrand`.

const MUL: u32 = 0x0019_660D;
const ADD: u32 = 0x3C6E_F35F;

#[derive(Clone, Copy, Debug)]
pub struct Qrand {
    idum: u32,
}

impl Default for Qrand {
    /// `__qrand_idum = 1` before `sqrand`.
    fn default() -> Self {
        Self { idum: 1 }
    }
}

impl Qrand {
    pub fn new(seed: u32) -> Self {
        Self { idum: seed }
    }

    /// `sqrand`.
    pub fn seed(&mut self, seed: u32) {
        self.idum = seed;
    }

    /// `qrand`.
    pub fn qrand(&mut self) -> u32 {
        self.idum = self.idum.wrapping_mul(MUL).wrapping_add(ADD);
        self.idum
    }

    /// `fqrand`: uniform in `[0, 1)` (23-bit steps).
    pub fn fqrand(&mut self) -> f32 {
        f32::from_bits((self.qrand() >> 9) | 0x3F80_0000) - 1.0
    }

    /// `fqrand2`: uniform in `[-0.5, 0.5)`.
    pub fn fqrand2(&mut self) -> f32 {
        f32::from_bits((self.qrand() >> 9) | 0x3F80_0000) - 1.5
    }

    /// `RANDOM(n)`: `(int)(fqrand() * (f32)n)`.
    pub fn random(&mut self, n: i32) -> i32 {
        (self.fqrand() * n as f32) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_the_source_recurrence() {
        let mut rng = Qrand::new(0);
        assert_eq!(rng.qrand(), 0x3C6E_F35F);
        assert_eq!(
            rng.qrand(),
            0x3C6E_F35Fu32.wrapping_mul(MUL).wrapping_add(ADD)
        );
        assert_eq!(Qrand::default().qrand(), MUL.wrapping_add(ADD));
    }

    #[test]
    fn fqrand_is_the_mantissa_trick() {
        let mut rng = Qrand::new(0);
        assert_eq!(
            rng.fqrand(),
            f32::from_bits(0x3F80_0000 | (0x3C6E_F35F >> 9)) - 1.0
        );
        let mut rng = Qrand::new(7);
        for _ in 0..10_000 {
            let f = rng.fqrand();
            assert!((0.0..1.0).contains(&f));
            let g = rng.fqrand2();
            assert!((-0.5..0.5).contains(&g));
        }
    }

    #[test]
    fn random_stays_below_n() {
        let mut rng = Qrand::new(12345);
        let mut seen = [false; 8];
        for _ in 0..1000 {
            let r = rng.random(8);
            assert!((0..8).contains(&r));
            seen[r as usize] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }
}
