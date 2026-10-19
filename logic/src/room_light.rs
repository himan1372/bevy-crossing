//! The player-room light switch (`m_kankyo.c` electric light,
//! `m_room_type.c` switch bits), verified against ac-decomp `src/game/
//! m_kankyo.c` / `m_room_type.c` and ACGC-PC-Port (same logic; the port only
//! scales the switch-on flicker and `add_calc` by dt).
//!
//! - Switch bits live in RAM (`l_mRmTp_switch_data`), not the save. Game
//!   start (`mRmTp_SetDefaultLightSwitchData(1)`) turns an owned house's
//!   main-floor light on from 18:00 to 05:00, otherwise off.
//! - Z toggles it in rooms whose `z_bottun_data` entry is set (player houses
//!   and the island cottage). While the light is on, Z is ignored until the
//!   20-frame switch-on flicker finishes. Each frame updates the light level
//!   first, then reads the trigger. On plays `NA_SE_LIGHT_ON`, off plays
//!   `NA_SE_LIGHT_OFF`, both as system triggers.
//! - Entering a room (`mEnv_RoomTypePointLightSet`) sets the level directly
//!   to 1 (on) or `point_light_min` = 0.14 (off), with no flicker.
//! - On: flicker table, then `add_calc(1, 0.01, 0.01, 0.01)`. Off:
//!   `add_calc(0.14, 0.2, 0.2, 0.02)`.
//! - The level scales the room point light colour, the room primitive colour
//!   (`mEnv_GetRoomPrimColor`) and the sun/moon diffuse colours
//!   (`mEnv_ChangeDiffuseLight`: x `(0.4 - 1) * level + 1`).

use crate::lights::LightPoint;

/// `NA_SE_LIGHT_ON` / `NA_SE_LIGHT_OFF` (`audio_defs.h`: ZOOMDOWN_LONG + 1, + 2).
pub const SE_LIGHT_ON: u16 = 0x16;
pub const SE_LIGHT_OFF: u16 = 0x17;

/// `point_light_min` (every scene in `Global_kankyo_ct`).
pub const POINT_LIGHT_MIN: f32 = 0.14;
/// `diffuse_adjust` outside the insect museum.
pub const DIFFUSE_ADJUST: f32 = 0.4;
/// `mEnv_LIGHT_ANIME_FRAMES`.
pub const LIGHT_ANIME_FRAMES: usize = 20;
const SWITCH_ON_ANIME: [f32; LIGHT_ANIME_FRAMES] = [
    0.00, 0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.25, 0.20, 0.15, 0.10, 0.30, 0.35, 0.40, 0.45, 0.50,
    0.45, 0.40, 0.35, 0.30,
];

/// `mEnv_GetNowRoomPointLightInfo`, `SCENE_MY_ROOM_S` (the first house).
pub const MY_ROOM_S_POINT_LIGHT: LightPoint = LightPoint {
    x: 120,
    y: 180,
    z: 180,
    color: [220, 220, 200],
    radius: 1000,
};

/// Retail `add_calc` (m_lib.c) for one 60 Hz frame. Returns `target - value`.
pub fn add_calc(value: &mut f32, target: f32, fraction: f32, max_step: f32, min_step: f32) -> f32 {
    if *value != target {
        let mut step = fraction * (target - *value);
        if step <= -min_step || min_step <= step {
            step = step.clamp(-max_step, max_step);
            *value += step;
            if step > 0.0 {
                if *value > target {
                    *value = target;
                }
            } else if *value < target {
                *value = target;
            }
        } else if step > 0.0 {
            *value += min_step;
            if *value > target {
                *value = target;
            }
        } else {
            *value -= min_step;
            if *value < target {
                *value = target;
            }
        }
    }
    target - *value
}

/// `mRmTp_CheckMyRoomLightON(house, TRUE)`: the game-start switch bit.
pub fn default_on(now_sec: u32, owned: bool) -> bool {
    owned && !(5 * 3600..18 * 3600).contains(&now_sec)
}

/// One room's switch bit and `l_mEnv_electric_light` point-light level.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoomLight {
    pub on: bool,
    pub percent: f32,
    animating_on: bool,
    anime_frame: usize,
}

impl RoomLight {
    pub fn new(on: bool) -> Self {
        let mut light = Self {
            on,
            percent: 0.0,
            animating_on: false,
            anime_frame: 0,
        };
        light.enter();
        light
    }

    /// `mEnv_RoomTypePointLightSet` after `Global_kankyo_ct` clears the state.
    pub fn enter(&mut self) {
        self.animating_on = false;
        self.anime_frame = 0;
        self.percent = if self.on { 1.0 } else { POINT_LIGHT_MIN };
    }

    /// `mEnv_ManagePointLight` for one 60 Hz frame. `z` is this frame's
    /// trigger; returns the switch sound when it changed the switch.
    pub fn tick(&mut self, z: bool) -> Option<u16> {
        if self.on {
            if self.animating_on {
                // mEnv_LightAnimeToSwitchON
                if self.anime_frame >= LIGHT_ANIME_FRAMES {
                    self.animating_on = false;
                } else {
                    self.percent = POINT_LIGHT_MIN
                        + (1.0 - POINT_LIGHT_MIN) * SWITCH_ON_ANIME[self.anime_frame];
                    self.anime_frame += 1;
                }
                None
            } else {
                add_calc(&mut self.percent, 1.0, 0.01, 0.01, 0.01);
                z.then(|| {
                    // mEnv_RequestChangeLightOFF(PLAYER)
                    self.on = false;
                    SE_LIGHT_OFF
                })
            }
        } else {
            add_calc(&mut self.percent, POINT_LIGHT_MIN, 0.2, 0.2, 0.02);
            z.then(|| {
                // mEnv_RequestChangeLightON(PLAYER, TRUE)
                self.on = true;
                self.animating_on = true;
                self.anime_frame = 0;
                SE_LIGHT_ON
            })
        }
    }

    /// The point light registered for the room, colour scaled by the level
    /// each frame (`color[i] = point_light_color[i] * percent`).
    pub fn point_light(&self, base: LightPoint) -> LightPoint {
        LightPoint {
            color: base.color.map(|c| (c as f32 * self.percent) as u8),
            ..base
        }
    }

    /// `mEnv_ChangeDiffuseLight`'s sun/moon factor in light-switch rooms.
    pub fn celestial_adjust(&self) -> f32 {
        (DIFFUSE_ADJUST - 1.0) * self.percent + 1.0
    }

    /// `mEnv_GetRoomPrimColor` (not a flame light, full diffuse effect).
    pub fn room_prim(&self, room: [u8; 3], point_color: [u8; 3]) -> [u8; 3] {
        let f0 = 1.0 + self.percent * -0.3;
        let f1 = self.percent * 0.6;
        std::array::from_fn(|i| {
            let c = (room[i] as f32 * f0 + point_color[i] as f32 * f1) as i32;
            c.clamp(0, 255) as u8
        })
    }
}
