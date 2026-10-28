//! `mSM_move_Move` (`m_submenu_ovl.c`): how a submenu overlay slides in from the top of the
//! screen and leaves upward, in 60 Hz ticks. `pos` is the height above its resting place in
//! pixels (300 off screen, 0 at rest).

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slide {
    pub pos: f32,
    pub speed: f32,
    pub alternate: bool,
    pub ticks: f32,
    pub closing: bool,
}

impl Default for Slide {
    fn default() -> Self {
        Self::new()
    }
}

impl Slide {
    /// `mSM_MOVE_IN_TOP`.
    pub fn new() -> Self {
        Self {
            pos: 300.0,
            speed: 75.0,
            alternate: false,
            ticks: 0.0,
            closing: false,
        }
    }

    /// `mSM_MOVE_OUT_TOP`: starts the closing slide from where the menu rests.
    pub fn close(&mut self) {
        self.closing = true;
        self.speed = 1.0;
        self.alternate = false;
        self.ticks = 0.0;
    }

    /// 60 Hz ticks of `mSM_move_Move`; true when the closing slide has finished.
    pub fn advance(&mut self, ticks: f32) -> bool {
        self.ticks += ticks;
        while self.ticks >= 1.0 {
            self.ticks -= 1.0;
            if !self.closing && self.pos <= 0.0 {
                self.ticks = 0.0;
                break;
            }
            let (threshold, target, direction, multiplier) = if self.closing {
                (0.0, 300.0, 1.0, 2.0)
            } else {
                (120.0, 0.0, -1.0, 0.5)
            };
            if !self.alternate && direction * (self.pos - threshold) >= 0.0 {
                self.speed = (self.speed * multiplier).clamp(1.0, 75.0);
            }
            self.alternate = !self.alternate;
            self.pos += self.speed * direction * 0.5;
            if direction * (self.pos - target) > 0.0 {
                self.pos = target;
                self.alternate = false;
                self.ticks = 0.0;
                return self.closing;
            }
        }
        self.pos = self.pos.trunc();
        false
    }

    pub fn settled(&self) -> bool {
        !self.closing && self.pos <= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_comes_in_to_rest_and_leaves_again() {
        let mut slide = Slide::new();
        for _ in 0..200 {
            assert!(!slide.advance(1.0));
        }
        assert!(slide.settled());
        slide.close();
        let mut done = false;
        for _ in 0..200 {
            done |= slide.advance(1.0);
        }
        assert!(done);
        assert_eq!(slide.pos, 300.0);
    }
}
