//! The Wisp's spirits (`ac_ins_hitodama.c`): 40 over the ground (or the
//! water), drifting at 0.3 in long curves that flip way every 4-10 s,
//! turning back once 160 out from the acre's middle; bobbing 10. Without
//! the ghost event they fly off.

use super::{Field, Insect, World, UNIT};
use crate::gyo_ai::{angle_to, sin_s};
use crate::ins_set::is_water;

pub(super) const SPIRIT: u8 = 40;
/// `aIHD_RANGE`: the acre's middle, and half of it as the leash.
const RANGE: f32 = 8.0 * UNIT;

impl Insect {
    /// `aIHD_actor_init`.
    pub(super) fn spirit(
        pos: [f32; 2],
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> Option<Self> {
        let u = field.unit_of(pos[0], pos[1])?;
        let wet = field.attribute(pos[0], pos[1]).is_some_and(is_water);
        let base = if wet {
            field
                .water_y(pos[0], pos[1])
                .unwrap_or_else(|| field.ground_y(pos[0], pos[1]))
        } else {
            field.ground_y(pos[0], pos[1])
        };
        let mut ins = Insect::blank(SPIRIT, [pos[0], base + UNIT, pos[1]], u);
        ins.action = super::Action::Fly;
        ins.angle = (rand(1.0) * 65536.0) as i32 as i16;
        ins.facing = ins.angle;
        ins.timer = ins.spirit_move_info(rand);
        ins.phase = (rand(1.0) * 65536.0) as i32 as i16;
        let c = field.unit_center([u[0], u[1], 8, 8]);
        ins.centre_bk = [c[0] - UNIT / 2.0, c[1] - UNIT / 2.0];
        Some(ins)
    }

    /// `aIHD_set_move_info`: a new turn rate (40-104 a tick, the other
    /// way from before) and how long to keep it (4-10 s). The retail
    /// game picks both from the frame counter. `aim` holds the rate.
    fn spirit_move_info(&mut self, rand: &mut impl FnMut(f32) -> f32) -> f32 {
        const ADD: [i16; 4] = [40, 56, 80, 104];
        const TIME: [f32; 4] = [240.0, 360.0, 480.0, 600.0];
        let mut add = ADD[(rand(4.0) as usize).min(3)];
        if self.aim > 0 {
            add = -add;
        }
        self.aim = add;
        TIME[(rand(4.0) as usize).min(3)]
    }

    /// `aIHD_actor_move`.
    pub(super) fn spirit_step(&mut self, world: &World, rand: &mut impl FnMut(f32) -> f32) {
        self.anim = (self.anim + 0.5) % 2.0;
        if !self.fading && !world.wisp_night {
            // aIHD_avoid_init_sub: off past the player's heading.
            self.start_fading();
            self.speed = 1.5;
            self.angle = world
                .player_facing
                .wrapping_add((deg60() as f32 - rand(deg120() as f32)) as i16);
            self.home[1] = self.pos[1];
            self.phase = 0;
        }
        // aIHD_fuwafuwa: bob by 10 (13 when leaving, faster).
        let (step, size) = if self.fading {
            (0x400, 13.0)
        } else {
            (0x100 + rand(0x300 as f32) as i16, 10.0)
        };
        let before = sin_s(self.phase) * 10.0;
        self.phase = self.phase.wrapping_add(step);
        self.vy = self.gravity + (size * sin_s(self.phase) - before);
        if self.fading {
            self.gravity += 0.1;
            return;
        }
        self.target_speed = 0.3;
        self.speed_step = 0.1;
        // aIHD_calc_move_drt
        let here = [self.pos[0], self.pos[2]];
        let (dx, dz) = (self.centre_bk[0] - here[0], self.centre_bk[1] - here[1]);
        if (dx * dx + dz * dz).sqrt() > RANGE / 2.0 {
            let away = self.angle.wrapping_sub(angle_to(self.centre_bk, here));
            if away.unsigned_abs() < 0x1000 {
                self.timer = self.spirit_move_info(rand);
            }
        } else {
            self.timer -= 1.0;
            if self.timer < 0.0 {
                self.timer = self.spirit_move_info(rand);
            }
        }
        self.angle = self.angle.wrapping_add(self.aim);
        self.facing = self.angle;
    }
}

fn deg60() -> i16 {
    crate::gyo_ai::deg(60.0)
}

fn deg120() -> i16 {
    crate::gyo_ai::deg(120.0)
}

#[cfg(test)]
mod tests {
    use super::super::tests::{world, Flat};
    use super::*;

    #[test]
    fn a_spirit_drifts_near_the_middle_on_a_wisp_night_and_leaves_otherwise() {
        let mut r = |n: f32| n * 0.5;
        let mid = Flat.unit_center([0, 0, 8, 8]);
        let mut s = Insect::new(SPIRIT, mid, &Flat, &mut r).unwrap();
        let mut w = world(&[]);
        w.wisp_night = true;
        for _ in 0..3000 {
            s.step(&w, &Flat, &mut r);
            let (dx, dz) = (s.pos[0] - mid[0], s.pos[2] - mid[1]);
            assert!((dx * dx + dz * dz).sqrt() < RANGE, "stays about the middle");
        }
        assert!(!s.fading);
        s.step(&world(&[]), &Flat, &mut r);
        assert!(s.fading);
    }
}
