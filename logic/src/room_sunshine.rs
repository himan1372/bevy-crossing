//! Window sunshine patches in player rooms (`ef_room_sunshine.c`), verified
//! against ac-decomp `src/effect/ef_room_sunshine.c` and ACGC-PC-Port (same
//! logic). The light alpha the patches multiply by is
//! [`crate::kankyo_env::window_light_step`].
//!
//! - Player-room scenes place two `ROOM_SUNSHINE` actors; `SCENE_MY_ROOM_S`
//!   has ct arg 2 at (40, 0, 120) and ct arg 3 at (200, 0, 120). Arg 0 and 2
//!   are the left patch (`L` vertices, extending +x), others the right one.
//! - Each frame the move proc sets `scale.x` from the clock (a quarter sine
//!   to 1.5); the draw is skipped while it is 0 or the window alpha is below
//!   1e-4. The draw translates by `(x, y + 0.1, z)`, scales by
//!   `(scale.x, 1, 1)` and sets the primitive colour to the period's window
//!   sun colour (04:00-20:00) or window moon colour, with alpha
//!   `(u8)calc_alpha * window_alpha`.
//! - `GETREG(TAKREG, 30)` is a debug register, 0 in retail.

/// Seconds at which the sun window colour starts and ends.
const SUN_START: i32 = 14400;
const SUN_END: i32 = 72000;
const HALFDAY: i32 = 43200;
const DAY: i32 = 86400;

/// `Room_Sunshine` actors of `SCENE_MY_ROOM_S` (`player_room_s.c`):
/// (ct arg, x, z); y comes from the floor.
pub const MY_ROOM_S_ACTORS: [(i16, f32, f32); 2] = [(2, 40.0, 120.0), (3, 200.0, 120.0)];

/// Which vertex set and move proc an actor uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// `SunshineL`: patch extends toward +x; lit afternoons and after midnight.
    Left,
    /// `SunshineR`: patch extends toward -x; lit mornings and evenings.
    Right,
}

/// `Ef_Room_Sunshine_actor_ct`: the side and the world position, given the
/// floor height `bg_y(x, z)` (`mCoBG_GetBgY_OnlyCenter_FromWpos`).
pub fn actor_ct(arg: i16, x: f32, z: f32, bg_y: impl Fn(f32, f32) -> f32) -> (Side, [f32; 3]) {
    let mut side = Side::Right;
    let mut x = x;
    if arg == 0 {
        side = Side::Left;
    } else {
        x -= 1.0;
    }
    let mut y = bg_y(x, z);
    match arg {
        2 => {
            side = Side::Left;
            x += 5.0;
            y = 1.0 + bg_y(x, z);
            x -= 6.0;
        }
        3 => {
            x -= 5.0;
            y = 1.0 + bg_y(x, z);
            x += 6.0;
        }
        _ => {}
    }
    (side, [x, y, z])
}

/// `sin_s` of a binary angle (65536 per turn).
fn sin_s(a: i16) -> f32 {
    (a as f32 * std::f32::consts::PI / 32768.0).sin()
}

/// `calc_scale_Ef_Room_Sunshine`: `flag` 0 = an 8-hour ramp, 1 = 4 hours.
fn calc_scale(flag: bool, sec: i32) -> f32 {
    let span = if flag { 14400.0 } else { 28800.0 };
    1.5 * sin_s(((sec << 14) as f32 / span) as i16)
}

/// The move procs' `scale.x` at `now_sec`.
pub fn scale_x(side: Side, now_sec: i32) -> f32 {
    match side {
        Side::Left if now_sec < SUN_START => calc_scale(true, now_sec),
        Side::Left if (HALFDAY..SUN_END).contains(&now_sec) => calc_scale(false, now_sec - HALFDAY),
        Side::Right if (SUN_START..HALFDAY).contains(&now_sec) => {
            calc_scale(false, HALFDAY - now_sec)
        }
        Side::Right if now_sec >= SUN_END => calc_scale(true, DAY - now_sec),
        _ => 0.0,
    }
}

/// `calc_alpha_Ef_Room_Sunshine`: 120 at midnight falling to 0 at 04:00,
/// 0 at 04:00 rising to 255 at noon and back to 0 at 20:00, then up to 120
/// at midnight; x0.6 in rain or snow.
pub fn calc_alpha(now_sec: i32, rain_or_snow: bool) -> u8 {
    let mut ret = if now_sec < SUN_START {
        120.0 * ((SUN_START - now_sec) as f32 / 14400.0)
    } else if now_sec < SUN_END {
        let sec = (now_sec - HALFDAY).abs();
        255.0 * ((28800 - sec) as f32 / 28800.0)
    } else {
        120.0 * ((14400 - (DAY - now_sec)) as f32 / 14400.0)
    };
    if rain_or_snow {
        ret *= 0.6;
    }
    ret as u8
}

/// Whether the patches use the window sun colour (else the moon colour).
pub fn sun_colour(now_sec: i32) -> bool {
    (SUN_START..SUN_END).contains(&now_sec)
}

/// The primitive alpha `setup_mode_Ef_Room_Sunshine` sets.
pub fn prim_alpha(now_sec: i32, rain_or_snow: bool, window_alpha: f32) -> u8 {
    (calc_alpha(now_sec, rain_or_snow) as f32 * window_alpha) as i32 as u8
}

/// Whether the draw proc draws at all.
pub fn drawn(window_alpha: f32, scale_x: f32) -> bool {
    // 9.99999974738e-05f in the source: 1e-4 as an f32.
    window_alpha >= 1e-4 && scale_x != 0.0
}
