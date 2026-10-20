//! ac_sum_slot: stopped-only cKF playback and the original two-reel table.

const REELS: [usize; 121] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1,
];

#[derive(Clone, Copy, Debug)]
pub struct Slot {
    next: f32,
    speed: f32,
    pub pose: f32,
    running: bool,
    texture_frame: u16,
    pub sound_on: bool,
}

impl Default for Slot {
    fn default() -> Self {
        // Constructor samples frame1 at half speed, then sets speed to zero.
        Self {
            next: 1.5,
            speed: 0.0,
            pose: 1.0,
            running: false,
            texture_frame: 0,
            sound_on: false,
        }
    }
}

impl Slot {
    fn play(&mut self, end: i16) -> bool {
        let end = f32::from(end.max(1));
        self.pose = self.next.min(end);
        // stop_proc detects the crossing before the controller advances;
        // the already sampled pose remains at end-0.5 on the stopping tick.
        if self.next >= end || self.next + self.speed >= end {
            self.next = end;
            true
        } else {
            self.next = (self.next + self.speed).min(end);
            false
        }
    }

    /// One original 60Hz callback. Busy changes are consumed, never queued.
    pub fn step(&mut self, changed: bool, end: i16) {
        self.sound_on = false;
        if self.running {
            if self.play(end) {
                self.running = false;
            } else {
                self.speed = 0.5;
                self.texture_frame = (self.texture_frame + 1) % 242;
                self.sound_on = true;
            }
        } else if changed {
            self.running = true;
            self.next = 1.0;
            self.play(end);
            self.speed = 0.5;
        }
    }

    pub fn texture(&self) -> usize {
        REELS[usize::from(self.texture_frame >> 1)]
    }
}
