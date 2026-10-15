//! Four spatial level sources (SE subtracks 8..11), following the original
//! Na_OngenPos range, distance2vol and angle2pan. Stable source identities
//! keep a loop running as its parameters change. This uses bounded nearest
//! selection instead of the old translation's full-pool slot-zero overwrite.
pub const VOICES: usize = 4;
pub const RANGE: f32 = 540.0;

/// Shared fixed-camera microphone, in original game units.
pub fn microphone(player: [f32; 3]) -> [f32; 3] {
    [player[0], player[1] + 240.0, player[2] + 77.0]
}

/// Na_OngenPos range, distance2vol and angle2pan for one world source.
pub fn spatial(position: [f32; 3], microphone: [f32; 3]) -> Option<(f32, u8)> {
    let distance = position
        .iter()
        .zip(microphone)
        .map(|(p, m)| (p - m).powi(2))
        .sum::<f32>()
        .sqrt();
    if !distance.is_finite() || distance >= RANGE {
        return None;
    }
    let angle = crate::gyo_ai::angle_to([microphone[0], microphone[2]], [position[0], position[2]]);
    Some((
        crate::audio::distance2vol(distance),
        crate::audio::angle2pan(angle as u16, 1),
    ))
}

/// mRF_block_info entries carrying mRF_BLOCKKIND_MARINE (m_random_field.c).
pub fn marine_block_type(kind: u8) -> bool {
    matches!(kind, 63 | 64 | 80..=82 | 85 | 94..=104)
}

#[derive(Clone, Copy, Debug)]
pub struct Source {
    pub key: u64,
    pub id: u8,
    /// Original game units.
    pub position: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Level {
    pub key: u64,
    /// High bit is the original echo flag, not a bank number.
    pub id: u8,
    pub volume: f32,
    pub pan: u8,
}

#[derive(Default)]
pub struct Mixer {
    active: [Option<Level>; VOICES],
}

impl Mixer {
    pub fn clear(&mut self) -> [Option<Level>; VOICES] {
        self.active = [None; VOICES];
        self.active
    }

    pub fn update(
        &mut self,
        sources: impl IntoIterator<Item = Source>,
        microphone: [f32; 3],
        quiet: bool,
    ) -> [Option<Level>; VOICES] {
        // Keep only four nearest candidates, without allocating a per-frame
        // cache or allowing old entries to survive a missing refresh.
        let mut nearest: [Option<(f32, Source)>; VOICES] = [None; VOICES];
        for source in sources {
            if source.id & 0x7F == 0 || nearest.iter().flatten().any(|(_, s)| s.key == source.key) {
                continue;
            }
            let d = source
                .position
                .iter()
                .zip(microphone)
                .map(|(p, m)| (p - m).powi(2))
                .sum::<f32>();
            if !d.is_finite() || d >= RANGE * RANGE {
                continue;
            }
            if let Some(i) = nearest
                .iter()
                .position(|n| n.is_none_or(|(distance, _)| d < distance))
            {
                for j in (i + 1..VOICES).rev() {
                    nearest[j] = nearest[j - 1];
                }
                nearest[i] = Some((d, source));
            }
        }
        let mut levels = nearest.map(|entry| {
            entry.map(|(d, source)| {
                let angle = crate::gyo_ai::angle_to(
                    [microphone[0], microphone[2]],
                    [source.position[0], source.position[2]],
                );
                Level {
                    key: source.key,
                    id: source.id,
                    volume: crate::audio::distance2vol(d.sqrt()) * if quiet { 0.5 } else { 1.0 },
                    pan: crate::audio::angle2pan(angle as u16, 1),
                }
            })
        });
        // Preserve slots for retained emitters; moving past an acre boundary
        // must not restart every other loop as the nearest ordering changes.
        for active in &mut self.active {
            if let Some(i) = levels.iter().position(|n| {
                n.is_some_and(|n| active.is_some_and(|a| a.key == n.key && a.id == n.id))
            }) {
                *active = levels[i].take();
            } else {
                *active = None;
            }
        }
        for level in levels.into_iter().flatten() {
            if let Some(slot) = self.active.iter_mut().find(|a| a.is_none()) {
                *slot = Some(level);
            }
        }
        self.active
    }
}
