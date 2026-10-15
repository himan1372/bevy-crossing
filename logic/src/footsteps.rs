//! Player footsteps: sAdo_Get_WalkLabel, Sou_WalkSe, Na_PlyWalkSe/Room,
//! and m_player_common's walk1/run1 foot contacts (USA Rev 0).
//! Sound content and the SE_FLOOR_DATA table remain on the player's disc.

pub const WOOD: u16 = 0x4204;
pub const FLOOR_START: u16 = 0x42E6;

/// Expects the effective mCoBG_Wpos2Attribute result, not a raw edge code.
pub fn outdoor_label(attribute: u8, winter: bool) -> u16 {
    match attribute {
        0..=2 if winter => 0x4206,
        0..=2 => 0x4201,
        7 => 0x4203,
        23 => WOOD,
        9 => 0x4205,
        22 => 0x4208,
        11 => 0x4209,
        _ => 0x4202,
    }
}

/// Na_PlyWalkSe and Na_PlyWalkSeRoom's optional volume multiplier.
pub fn volume(running: bool, indoors: bool) -> f32 {
    match (running, indoors) {
        (false, false) => 0.6,
        (true, false) => 0.8,
        (false, true) => 0.54,
        (true, true) => 0.719_999_97,
    }
}

/// Na_NpcWalkSe/Room: NPC optional gain multiplied by the original
/// distance2vol curve. Distance is in original game units (40 per world unit).
pub fn npc_volume(distance: f32, indoors: bool) -> f32 {
    crate::audio::distance2vol(distance) * if indoors { 0.612 } else { 0.68 }
}

/// Sou_WalkSe: alternating feet, with occasional alternate samples.
/// Drafted with local Qwen, corrected against the C to use owned state.
#[derive(Default)]
pub struct FootstepVariants {
    left: bool,
    last_left: bool,
    last_right: bool,
}

impl FootstepVariants {
    pub fn select(&mut self, base: u16, dash: bool, random: u16) -> u16 {
        let mut id = base + if dash { 40 } else { 0 };
        let alternate = random >> 14 == 3;
        if self.left {
            if alternate && !self.last_left && !self.last_right {
                id += 30;
                self.last_left = true;
            } else {
                id += 10;
                self.last_left = false;
            }
        } else if alternate && !self.last_left && !self.last_right {
            id += 20;
            self.last_right = true;
        } else {
            self.last_right = false;
        }
        self.left = !self.left;
        id
    }
}

/// Count walk1/run1 contacts at frames 1 and 9 over (frame, frame+advance],
/// including loop wrap. Unlike equality checks, this works above 60 fps.
pub fn contacts(frame: f32, advance: f32, end: f32) -> u32 {
    if !frame.is_finite()
        || !advance.is_finite()
        || !end.is_finite()
        || advance <= 0.0
        || end <= 1.0
    {
        return 0;
    }
    let period = end - 1.0;
    [1.0, 9.0]
        .into_iter()
        .filter(|&target| target < end)
        .map(|target| {
            let before = ((frame - target) / period).floor();
            let after = ((frame + advance - target) / period).floor();
            (after - before).max(0.0) as u32
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contacts_do_not_depend_on_render_rate() {
        for fps in [30, 60, 144, 300] {
            // Exactly ten walk cycles at 0.5 animation frames / 60 Hz tick.
            let mut frame = 1.0;
            let mut count = 0;
            for _ in 0..fps * 5 {
                let advance = 30.0 / fps as f32;
                count += contacts(frame, advance, 16.0);
                frame = 1.0 + (frame + advance - 1.0).rem_euclid(15.0);
            }
            // f32 accumulation can stop just before the final wrap.
            assert!((19..=20).contains(&count), "fps {fps}: {count}");
        }
        assert_eq!(contacts(8.9, 0.2, 16.0), 1);
        assert_eq!(contacts(9.1, 0.1, 16.0), 0);
        assert_eq!(contacts(15.9, 0.2, 16.0), 1);
        assert_eq!(contacts(9.0, 0.0, 16.0), 0);
    }

    #[test]
    fn alternate_samples_do_not_repeat_on_adjacent_feet() {
        let mut variants = FootstepVariants::default();
        let sounds: Vec<_> = (0..6)
            .map(|_| variants.select(0x4201, false, 0xFFFF))
            .collect();
        assert_eq!(sounds, [0x4215, 0x420B, 0x4201, 0x421F, 0x4201, 0x420B]);
        let mut regular = FootstepVariants::default();
        assert_eq!(regular.select(0x4201, false, 0), 0x4201);
        assert_eq!(regular.select(0x4201, false, 0), 0x420B);
    }
}
