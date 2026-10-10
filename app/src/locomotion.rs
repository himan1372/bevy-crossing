//! Player walk / run, driven by the translated controller in
//! `rustimal_logic::player_move` (`Player_actor_Movement_Walk` family).
//!
//! What comes from the decomp:
//! - facing turns toward the stick angle by `calc_ease(turn_mod(move_pR))`
//!   per tick — a light push turns slowly, a full push turns fast;
//! - the player moves along that smoothed facing (so direction changes arc,
//!   instead of snapping);
//! - braking removes `BRAKE_AMOUNT` per tick after the stick is released;
//! - animation speed follows actual speed (`anim_speed`).
//!
//! What is ours (not yet traced in the decomp): the world-space speed scale
//! ([`FULL_TILT_SPEED`]) and the keyboard "walk" tilt ([`WALK_TILT`]). The
//! GameCube game ticks at 30 fps, so per-tick values are scaled by elapsed
//! ticks to feel the same at any frame rate. Speed is kept in stick units
//! (0..1), which is also the unit braking is applied in.
//!
//! Each location (town, house, shop) calls [`step`] and then applies its
//! own collision to the returned movement.

use bevy::prelude::*;
use rustimal_logic::player_move::{self, LocomotionState, PlayerMovement};

use crate::input::{Action, Actions};
use crate::interaction::Facing;

/// World units per second at full tilt (running).
pub const FULL_TILT_SPEED: f32 = 6.0;
/// Stick magnitude a keyboard walk stands in for (Run held = full tilt).
pub const WALK_TILT: f32 = 0.55;
/// The GameCube game's logic rate.
const RETAIL_FPS: f32 = 30.0;

/// The player's locomotion state (facing lives in [`Facing`]).
#[derive(Component, Default)]
pub struct Locomotion(pub PlayerMovement);

impl Locomotion {
    /// Walk / run / wait, for animation and sound.
    #[allow(dead_code)] // first caller: the M2 player model
    pub fn state(&self) -> LocomotionState {
        self.0.state
    }

    /// Playback speed for the walk/run animation (decomp formula; full
    /// tilt gives 0.6).
    #[allow(dead_code)] // first caller: the M2 player model
    pub fn anim_speed(&self) -> f32 {
        player_move::anim_speed(self.0.speed, player_move::DASH_SPEED_NUMERATOR)
    }
}

/// Facing angle in the decomp convention: 0° = +z, 90° = +x.
fn facing_deg(dir: Vec3) -> f32 {
    dir.x.atan2(dir.z).to_degrees()
}

/// Advance one frame. Updates `facing` and `loco`, and returns this frame's
/// movement in world space; the caller applies collision.
pub fn step(actions: &Actions, dt: f32, facing: &mut Facing, loco: &mut Locomotion) -> Vec3 {
    let ticks = dt * RETAIL_FPS;
    let m = &mut loco.0;
    // Doors and other systems may set the facing directly; start from it.
    m.facing_deg = facing_deg(facing.dir);

    let input = actions.move_dir();
    if input.length_squared() > 0.0 {
        let move_pr = if actions.pressed(Action::Run) {
            1.0
        } else {
            WALK_TILT
        };
        let target = facing_deg(input);
        // Per-tick fraction from the decomp, compounded over this frame's ticks.
        let per_tick = player_move::calc_ease(player_move::turn_mod(move_pr));
        let frac = 1.0 - (1.0 - per_tick).powf(ticks);
        let diff = player_move::wrap_deg(target - m.facing_deg);
        m.facing_deg = player_move::wrap_deg(m.facing_deg + diff * frac);
        m.speed = move_pr;
        m.state = if move_pr >= 1.0 {
            LocomotionState::Run
        } else {
            LocomotionState::Walk
        };
    } else if m.speed > 0.0 {
        m.speed = (m.speed - player_move::BRAKE_AMOUNT * ticks).max(0.0);
        if m.speed == 0.0 {
            m.state = LocomotionState::Wait;
        }
    }

    let rad = m.facing_deg.to_radians();
    facing.dir = Vec3::new(rad.sin(), 0.0, rad.cos());
    facing.dir * m.speed * FULL_TILT_SPEED * dt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facing_convention_matches_decomp() {
        assert!((facing_deg(Vec3::Z) - 0.0).abs() < 1e-5);
        assert!((facing_deg(Vec3::X) - 90.0).abs() < 1e-5);
        assert!((facing_deg(-Vec3::Z).abs() - 180.0).abs() < 1e-5);
    }

    #[test]
    fn turn_is_frame_rate_independent() {
        // Two half-ticks must land where one full tick does.
        let per_tick = player_move::calc_ease(player_move::turn_mod(1.0));
        let one = 1.0 - (1.0 - per_tick).powf(1.0);
        let half = 1.0 - (1.0 - per_tick).powf(0.5);
        let two_halves = half + (1.0 - half) * half;
        assert!((one - two_halves).abs() < 1e-6);
    }
}
