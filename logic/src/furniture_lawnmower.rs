//! ac_nog_lawnmower (fNLM_mv/dw) and its aMR_SetMoveSE case.
//!
//! Cut grass shows only on the backyard-lawn and daisy-meadow carpets
//! (FLOOR_NPC026/FLOOR_NPC048) while the mower is pushed, or waiting to be
//! pushed, with the player's contact at its back. Its alpha eases with
//! `add_calc(.., 0.04, 0.1, 0.001)`; a push from the back on those carpets
//! plays NA_SE_LAWNMOWER instead of the floor's drag sound.

use crate::room_light::add_calc;

/// Furniture index of `iam_nog_lawnmower`.
pub const LAWNMOWER: u16 = 0x4E8;
/// FLOOR_NPC026, FLOOR_NPC048.
pub const LAWN_FLOORS: [u8; 2] = [26, 48];
/// NA_SE_LAWNMOWER.
pub const SE_LAWNMOWER: u16 = 0x177;
/// fNLM_dw primitive colour (alpha from the grass level).
pub const GRASS_PRIM: [u8; 3] = [120, 255, 180];

pub fn lawn_floor(floor: u8) -> bool {
    LAWN_FLOORS.contains(&floor)
}

/// fNLM_mv for one 60 Hz frame: `cutting` is the floor/state/back-contact test.
pub fn grass_step(alpha: &mut f32, cutting: bool) {
    let target = if cutting { 1.0 } else { 0.0 };
    if *alpha != target {
        add_calc(alpha, target, 0.04, 0.1, 0.001);
    }
}

/// fNLM_dw: `(u8)(alpha * 255)`.
pub fn grass_alpha(alpha: f32) -> u8 {
    (alpha * 255.0) as u8
}
