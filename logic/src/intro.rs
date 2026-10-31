//! What Rover's questions make of the player (`ac_npc_guide_move.c_inc`: `aNGD_sex_select_wait2`,
//! `aNGD_set_pl_face_type`, and `mPr_GetRandomCloth`).

/// `mPr_FACE_TYPE_NUM`.
pub const FACE_TYPES: usize = 8;
/// `face_type_table` for a boy, indexed by the rude-answer bits shifted down by one.
const BOY_FACES: [u8; FACE_TYPES] = [5, 6, 1, 4, 0, 2, 7, 3];
/// `face_type_table` for a girl.
const GIRL_FACES: [u8; FACE_TYPES] = [0, 5, 2, 6, 4, 7, 3, 1];
/// `mPr_boy_cloth_table` / `mPr_girl_cloth_table`: ITM_CLOTH000-007 and ITM_CLOTH008-015.
const CLOTHS: usize = 8;

/// The player's gender from the two answers Rover asks for: "Isn't it cool?" or "Isn't it cute?"
/// (`cool`), then whether his guess ("You know it!" / "I forgive you.") was right (`agreed`).
/// Cool and agreed, or cute and corrected, is a boy.
pub fn is_male(cool: bool, agreed: bool) -> bool {
    cool == agreed
}

/// `aNGD_set_pl_face_type`: with none of the four answers that shape the face given (bit 0 clear) the
/// face is random (`random` in 0..8); otherwise it comes from the table by the other three bits.
pub fn face_type(male: bool, answer_flags: u8, random: usize) -> usize {
    if answer_flags & 1 == 0 {
        random % FACE_TYPES
    } else {
        let table = if male { &BOY_FACES } else { &GIRL_FACES };
        usize::from(table[usize::from(answer_flags >> 1) % FACE_TYPES])
    }
}

/// `mPr_GetRandomCloth`: the starting shirt (`random` in 0..8), as a clothing bank index.
pub fn starting_cloth(male: bool, random: usize) -> usize {
    random % CLOTHS + if male { 0 } else { CLOTHS }
}

/// `cKF_HermitCalc(r, 1.0, 0.0, 1.0, 3.2, 0.0)`: how far the camera has moved from its resting centre
/// to Rover's head, `r` going from 0 to 1 over `CAMERA_MORPH` frames (`aNGD_set_camera`).
pub fn camera_blend(r: f32) -> f32 {
    let r = r.clamp(0.0, 1.0);
    let (r2, r3) = (r * r, r * r * r);
    let end_weight = -2.0 * r3 + 3.0 * r2;
    let start_tangent = r + (r3 - 2.0 * r2);
    end_weight + start_tangent * 3.2
}

/// Frames the camera takes to come round to Rover once he talks.
pub const CAMERA_MORPH: f32 = 40.0;

/// The train's sway on the camera (`aNGD_set_camera`: `camera_move`, `camera_move_range`): a slow
/// side-to-side wobble and an up-and-down one that dies away over a few swings and starts again.
#[derive(Clone, Copy, Debug)]
pub struct Sway {
    angle: u16,
    range: f32,
    count: i8,
    set: i8,
    last_y: f32,
}

impl Default for Sway {
    fn default() -> Self {
        Self {
            angle: 0,
            range: 0.3,
            count: 0,
            set: 1,
            last_y: 0.0,
        }
    }
}

impl Sway {
    /// One 60 Hz frame: the offset (x, y) in game units.
    pub fn tick(&mut self) -> (f32, f32) {
        let angle = self.angle.wrapping_add(0xE20);
        self.angle = angle;
        let radians = f32::from(angle as i16) / 32768.0 * std::f32::consts::PI;
        let x = radians.cos() * 0.1;
        let y = radians.sin() * self.range;
        if self.last_y <= 0.0 && y >= 0.0 {
            self.count -= 1;
            if self.count < 0 {
                self.set -= 1;
                if self.set < 0 {
                    self.set = 1;
                }
                self.count = [3, 0][self.set as usize];
                self.range = 0.3;
            } else {
                self.range *= 0.35;
            }
        }
        self.last_y = y;
        (x, y)
    }
}

/// `aTrainWindow_GetTreePalletIdx`: which of the fifteen tree palettes (spring greens to autumn reds to bare
/// winter) the window scenery uses on a date: the first whose (month, day) limit has not passed.
pub fn tree_palette(month: u8, day: u8) -> usize {
    const TILL: [(u8, u8); 15] = [
        (2, 3),
        (2, 17),
        (2, 24),
        (4, 3),
        (4, 8),
        (7, 22),
        (9, 15),
        (10, 5),
        (10, 23),
        (11, 7),
        (11, 14),
        (11, 28),
        (12, 10),
        (12, 17),
        (12, 31),
    ];
    TILL.iter()
        .position(|&(m, d)| month < m || (month == m && day <= d))
        .unwrap_or(0)
}

/// `aNRG_menu_close_wait_talk_proc`: only the starting quest Bell bag pays Nook.
pub fn take_starting_bells(pockets: &mut crate::inventory::Inventory, slot: usize) -> bool {
    if pockets.pockets.get(slot) != Some(&0x2100) || pockets.item_cond(slot) != 2 {
        return false;
    }
    pockets.pockets[slot] = crate::inventory::EMPTY_NO;
    pockets.set_item_cond(slot, 0);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_answers_decide_the_gender() {
        assert!(is_male(true, true));
        assert!(!is_male(true, false));
        assert!(!is_male(false, true));
        assert!(is_male(false, false));
    }

    #[test]
    fn the_face_follows_the_answers_or_is_random() {
        assert_eq!(face_type(true, 0b1000, 6), 6);
        assert_eq!(face_type(true, 0b0001, 6), BOY_FACES[0].into());
        assert_eq!(face_type(false, 0b1111, 6), usize::from(GIRL_FACES[7]));
    }

    #[test]
    fn a_girl_starts_in_one_of_the_girls_shirts() {
        assert_eq!(starting_cloth(true, 3), 3);
        assert_eq!(starting_cloth(false, 3), 11);
    }

    #[test]
    fn the_camera_comes_round_to_rover() {
        assert_eq!(camera_blend(0.0), 0.0);
        assert!((camera_blend(1.0) - 1.0).abs() < 1e-6);
        assert!(camera_blend(0.25) > 0.25, "it sets off fast");
    }

    #[test]
    fn the_sway_stays_small_and_restarts() {
        let mut sway = Sway::default();
        let mut biggest: f32 = 0.0;
        for _ in 0..600 {
            let (x, y) = sway.tick();
            biggest = biggest.max(x.abs()).max(y.abs());
        }
        assert!(biggest <= 0.31, "{biggest}");
    }

    #[test]
    fn the_tree_palette_follows_the_date() {
        assert_eq!(tree_palette(1, 1), 0);
        assert_eq!(tree_palette(2, 4), 1);
        assert_eq!(tree_palette(7, 1), 5);
        assert_eq!(tree_palette(12, 31), 14);
    }
}
