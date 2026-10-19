//! Indoor window light from `m_kankyo.c` (`mEnv_MakeWindowLightAlpha`).
//!
//! Subset of the pinned Rustimal 122867f `kankyo_env.rs` (source-verified
//! against ac-decomp `src/game/m_kankyo.c`): only the window-light target
//! and its constants. Light periods, weather cross-fade and the rainbow stay
//! in the pinned module until a caller needs them. The per-frame easing is
//! added here from the same source function.
//!
//! - Target indoors: 0 near midnight and noon, otherwise
//!   `0.22 + 0.78 * (1 - lamp%)` in rooms with a light switch, otherwise 1.0
//!   between 05:00 and 18:00 (villager lights off), else 0.
//! - Quirk: the "near midnight/noon" test casts seconds to **s16**, so it
//!   also matches 05:45:45-05:49:43 and 18:10:17-18:14:15: window light
//!   briefly drops at those times too.
//! - `Global_kankyo_ct` clears the alpha and eases once with 0.5 steps;
//!   every frame after (`Global_kankyo_set`) eases with
//!   `add_calc(alpha, target, 1 - sqrt(0.8), 0.005, 0.005)`.

use crate::room_light::add_calc;

pub const NPC_LIGHTS_OFF: i32 = 5 * 3600;
pub const NPC_LIGHTS_ON: i32 = 18 * 3600;
pub const SECONDS_IN_DAY: i32 = 86400;
pub const SECONDS_IN_HALFDAY: i32 = 43200;

/// `mEnv_MakeWindowLightAlpha` target. `lamp` = Some(point-light %) in a
/// room with a light switch.
pub fn window_light_target(now_sec: i32, lamp: Option<f32>) -> f32 {
    let near = |v: i32| (v as i16 as i32).abs() < 120;
    if near(now_sec) || near(now_sec - SECONDS_IN_DAY) || near(now_sec - SECONDS_IN_HALFDAY) {
        0.0
    } else if let Some(p) = lamp {
        (1.0 - p) * 0.78 + 0.22
    } else if (NPC_LIGHTS_OFF..NPC_LIGHTS_ON).contains(&now_sec) {
        1.0
    } else {
        0.0
    }
}

/// `mEnv_MakeWindowLightAlpha` for one 60 Hz frame indoors. `enabled` is
/// false only for the scene-construct call, which restarts from 0.
pub fn window_light_step(alpha: &mut f32, target: f32, enabled: bool) {
    let mut step = 1.0 / 100.0;
    if !enabled {
        *alpha = 0.0;
        step = 1.0;
    }
    add_calc(alpha, target, 1.0 - 0.8f32.sqrt(), 0.5 * step, 0.5 * step);
}
