//! CPU approximation of the retail NA_HALL_DELAY wet bus.
//! The RSP's fixed-point rounding, filters, Haas and surround are not emulated.
//! Delay lengths follow the engine's 200-spec-sample / 140-output-sample mapping.
use crate::nas_seq::{SAMPLES_PER_UPDATE, SAMPLE_RATE};

#[derive(Clone, Copy, Default)]
pub(crate) struct Frame {
    /// Effects, music, voices, ambience; original sequence gains remain intact.
    pub dry: [[f32; 2]; 4],
    pub wet: [f32; 2],
}

/// Cumulative observations since engine creation. Peaks are before protection.
#[derive(Clone, Copy, Debug)]
pub struct MixStats {
    pub bus_peaks: [f32; 4],
    pub peak: f32,
    pub output_peak: f32,
    pub limited_frames: u64,
    pub nonfinite_samples: u64,
    pub minimum_gain: f32,
    pub active_voices: usize,
}

impl Default for MixStats {
    fn default() -> Self {
        Self {
            bus_peaks: [0.0; 4],
            peak: 0.0,
            output_peak: 0.0,
            limited_frames: 0,
            nonfinite_samples: 0,
            minimum_gain: 1.0,
            active_voices: 0,
        }
    }
}

pub(crate) struct Mixer {
    ring: Vec<[f32; 2]>,
    cursor: usize,
    sub_delay: usize,
    gain: f32,
    release: f32,
    pub stats: MixStats,
}

impl Default for Mixer {
    fn default() -> Self {
        let scale = SAMPLES_PER_UPDATE as f32 / 200.0;
        Self {
            // audioconst.c: {1, 0x20, 0x4500, 0x14, 0x1100, 0x7fff, ...}.
            ring: vec![[0.0; 2]; (32.0 * 64.0 * scale).round() as usize],
            cursor: 0,
            sub_delay: (20.0 * 64.0 * scale).round() as usize,
            gain: 1.0,
            release: 1.0 - (-1.0 / (0.050 * SAMPLE_RATE as f32)).exp(),
            stats: MixStats::default(),
        }
    }
}

impl Mixer {
    pub fn render(&mut self, frames: &[Frame], out: &mut [f32]) {
        for (frame, output) in frames.iter().zip(out.as_chunks_mut::<2>().0) {
            let mut dry = [0.0; 2];
            for (bus, samples) in frame.dry.iter().enumerate() {
                for channel in 0..2 {
                    let sample = samples[channel];
                    if sample.is_finite() {
                        self.stats.bus_peaks[bus] = self.stats.bus_peaks[bus].max(sample.abs());
                    }
                    dry[channel] += sample;
                }
            }
            // driver.c Nas_Load/SaveAuxBuffer1[_B]: return main tap to dry,
            // decay it in place, and add its sub tap plus note sends at offset.
            let delayed = self.ring[self.cursor];
            let sub = (self.cursor + self.sub_delay) % self.ring.len();
            for channel in 0..2 {
                dry[channel] += delayed[channel] * (32767.0 / 32768.0);
                self.ring[self.cursor][channel] = delayed[channel] * (17664.0 / 32768.0);
                self.ring[sub][channel] +=
                    delayed[channel] * (4352.0 / 32768.0) + frame.wet[channel];
                // Avoid denormal work on tails below any useful PCM precision.
                if self.ring[sub][channel].abs() < 1.0e-15 {
                    self.ring[sub][channel] = 0.0;
                }
                if !self.ring[sub][channel].is_finite() {
                    self.ring[sub][channel] = 0.0;
                    self.stats.nonfinite_samples += 1;
                }
                if !dry[channel].is_finite() {
                    dry[channel] = 0.0;
                    self.stats.nonfinite_samples += 1;
                }
            }
            self.cursor = (self.cursor + 1) % self.ring.len();
            let peak = dry[0].abs().max(dry[1].abs());
            self.stats.peak = self.stats.peak.max(peak);
            // Port output protection, not retail DSP: linked stereo, immediate
            // attack and 50 ms recovery; unity below the 0.98 ceiling.
            let required = if peak > 0.98 { 0.98 / peak } else { 1.0 };
            self.gain = required.min(self.gain + (1.0 - self.gain) * self.release);
            if self.gain < 0.99999 {
                self.stats.limited_frames += 1;
            }
            self.stats.minimum_gain = self.stats.minimum_gain.min(self.gain);
            for channel in 0..2 {
                let sample = dry[channel] * self.gain;
                self.stats.output_peak = self.stats.output_peak.max(sample.abs());
                // Engine::update adds to its caller's output, as before.
                output[channel] += sample;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_mix_preserves_levels_and_adds_to_output() {
        let mut mixer = Mixer::default();
        let mut frame = Frame::default();
        frame.dry[0] = [0.25, -0.125];
        let mut out = [0.1, 0.1];
        mixer.render(&[frame], &mut out);
        assert!((out[0] - 0.35).abs() < 1.0e-6);
        assert!((out[1] + 0.025).abs() < 1.0e-6);
        assert_eq!(mixer.stats.minimum_gain, 1.0);
        assert_eq!(mixer.stats.limited_frames, 0);
    }

    #[test]
    fn dense_overlap_is_finite_limited_and_keeps_stereo_ratio() {
        let mut mixer = Mixer::default();
        let frame = Frame {
            dry: [[1.5, 0.75]; 4],
            wet: [0.4, 0.2],
        };
        let frames = vec![frame; SAMPLE_RATE as usize];
        let mut out = vec![0.0; frames.len() * 2];
        mixer.render(&frames, &mut out);
        assert!(out.iter().all(|s| s.is_finite() && s.abs() <= 0.98001));
        assert!(out
            .as_chunks::<2>()
            .0
            .iter()
            .all(|s| (s[0] - 2.0 * s[1]).abs() < 1.0e-5));
        assert!(mixer.stats.peak >= 6.0 && mixer.stats.minimum_gain < 0.2);
        assert!(mixer.stats.limited_frames > 0);
        let silence = vec![Frame::default(); SAMPLE_RATE as usize * 3];
        let mut tail = vec![0.0; silence.len() * 2];
        mixer.render(&silence, &mut tail);
        assert!(mixer.gain > 0.9999, "gain recovers after overlap");
        assert!(tail[tail.len() - 1000..].iter().all(|s| s.abs() < 1.0e-6));
    }

    #[test]
    fn wet_impulse_has_a_localized_decaying_tail() {
        let mut mixer = Mixer::default();
        let mut frames = vec![Frame::default(); SAMPLE_RATE as usize * 3];
        frames[0].wet = [0.5, 0.0];
        let mut out = vec![0.0; frames.len() * 2];
        // Sends are written ahead of the read cursor by sub_delay.
        let first_return = mixer.sub_delay;
        mixer.render(&frames, &mut out);
        assert!(out[..first_return * 2].iter().all(|s| *s == 0.0));
        assert!(out[first_return * 2] > 0.49);
        assert!(out.as_chunks::<2>().0.iter().all(|s| s[1] == 0.0));
        assert!(out[out.len() - 1000..].iter().all(|s| s.abs() < 1.0e-6));
    }

    #[test]
    fn invalid_input_does_not_poison_delay_or_meters() {
        let mut mixer = Mixer::default();
        let mut frame = Frame {
            dry: [[0.0; 2]; 4],
            wet: [f32::INFINITY, f32::NAN],
        };
        frame.dry[0] = [f32::NAN, f32::INFINITY];
        let mut out = [0.0; 2];
        mixer.render(&[frame], &mut out);
        assert!(out.iter().all(|s| s.is_finite()));
        assert!(mixer.ring.iter().flatten().all(|s| s.is_finite()));
        assert!(mixer.stats.bus_peaks.iter().all(|s| s.is_finite()));
        assert!(mixer.stats.nonfinite_samples >= 4);
    }
}
