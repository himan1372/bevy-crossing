//! ac_nog_gong and the small-room ef_flashC/ef_flash timing kernels.

#[derive(Clone, Copy, Debug)]
pub struct Gong {
    next: f32,
    pub pose: f32,
}

impl Default for Gong {
    fn default() -> Self {
        // init_standard_stop followed by the constructor's first play.
        Self {
            next: 1.5,
            pose: 1.0,
        }
    }
}

impl Gong {
    fn play(&mut self, end: i16) -> bool {
        self.pose = self.next.min(f32::from(end.max(1)));
        if self.next >= f32::from(end.max(1)) {
            self.next = self.pose;
            true
        } else {
            self.next += 0.5;
            false
        }
    }

    /// cKF samples before advancing. The callback calls play twice while
    /// running; a change during that branch is consumed without retriggering.
    pub fn step(&mut self, changed: bool, end: i16) -> bool {
        if !self.play(end) {
            self.play(end);
        } else if changed {
            self.next = 1.0;
            self.play(end);
            return true;
        }
        false
    }
}

#[derive(Clone, Copy)]
pub struct Flash {
    pub position: [f32; 3],
    pub age: u8,
    amplitude: f32,
}

impl Flash {
    pub fn scale(self) -> f32 {
        // eEL_CalcAdjust_F clamps the interpolation, including t=3..4.
        if self.age <= 4 {
            let t = (f32::from(self.age) / 2.0).min(1.0);
            self.amplitude * 0.005 * (1.0 - t) + 0.01 * t
        } else {
            0.0
        }
    }
}

struct Sequence {
    age: u16,
    count: u16,
}

#[derive(Default)]
pub struct Flashes {
    sequences: Vec<Sequence>,
    pub particles: Vec<Flash>,
    rng: u32,
    pub light: u8,
    light_start: u8,
    light_age: u8,
    light_peak: u8,
    light_end: u8,
}

impl Flashes {
    pub fn clear(&mut self) {
        self.sequences.clear();
        self.particles.clear();
        self.light = 0;
        self.light_peak = 0;
    }

    pub fn start(&mut self) {
        if self.sequences.len() + self.particles.len() < 100 {
            self.sequences.push(Sequence { age: 0, count: 0 });
        }
    }

    fn random(&mut self) -> f32 {
        let mut x = if self.rng == 0 { 0x2545_F491 } else { self.rng };
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x >> 8) as f32 / 16_777_216.0
    }

    pub fn step(&mut self) {
        for flash in &mut self.particles {
            flash.age += 1;
        }
        self.particles.retain(|f| f.age < 5);
        let mut bursts = Vec::new();
        for sequence in &mut self.sequences {
            sequence.age += 1;
            if sequence.age % 8 == 0 {
                bursts.push(sequence.count & 1 != 0);
                sequence.count += 1;
            }
        }
        for light in bursts {
            if self.sequences.len() + self.particles.len() >= 100 {
                break;
            }
            let position = [
                40.0 + self.random() * 160.0,
                65.0 + self.random() * 20.0,
                43.0,
            ];
            let length = (3.0 + self.random() * 7.0) as u8;
            if light {
                self.light_start = self.light;
                self.light_age = 0;
                // Preserve eFlash_ct's argument order to RegistEffectLight:
                // max_frame=length, n_frames=length>>1, even though the latter
                // makes the source reset before reaching its requested peak.
                self.light_peak = length;
                self.light_end = length >> 1;
            }
            let amplitude = 0.3 + self.random() * 0.7;
            self.particles.push(Flash {
                position,
                age: 0,
                amplitude,
            });
        }
        self.sequences.retain(|s| s.age < 240);
        if self.light_peak != 0 {
            if self.light_age > self.light_end {
                self.light = 0;
                self.light_peak = 0;
            } else {
                self.light = (f32::from(self.light_start)
                    + (27.0 - f32::from(self.light_start)) * f32::from(self.light_age)
                        / f32::from(self.light_peak)) as u8;
                self.light_age += 1;
            }
        }
    }
}
