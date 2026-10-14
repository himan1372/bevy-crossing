//! Mole crickets (`ac_ins_kera.c`): underground until a shovel digs their
//! unit; then out in an arc and off past the player, body pulsing, the
//! pace re-rolled every 10 ticks. From 400 out they burrow back down on
//! soft ground (or fade off on hard); water ahead drowns them.

use super::{Action, Field, Insect, World, UNIT};
use crate::gyo_ai::{angle_to, chase_angle, cos_s, deg, sin_s};
use crate::ins_set::{is_hole, is_water};

pub(super) const MOLE_CRICKET: u8 = 33;
const RANGE: f32 = 400.0;

impl Insect {
    /// `aIKR_actor_init`: under its unit, unseen.
    pub(super) fn mole_cricket(pos: [f32; 2], field: &impl Field) -> Option<Self> {
        let u = field.unit_of(pos[0], pos[1])?;
        let y = field.ground_y(pos[0], pos[1]);
        let mut ins = Insect::blank(MOLE_CRICKET, [pos[0], y, pos[1]], u);
        let c = field.unit_center([u[0], u[1], 8, 8]);
        ins.centre_bk = [c[0] - UNIT / 2.0, c[1] - UNIT / 2.0];
        ins.action = Action::Hide;
        Some(ins)
    }

    /// `aIKR_actor_move`; true when it drowned.
    pub(super) fn mole_step(
        &mut self,
        world: &World,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> bool {
        match self.action {
            Action::Hide => {
                // aIKR_check_dig_hole_scoop: dug on its very unit.
                let dug = world.scoop.is_some_and(|p| {
                    field.unit_of(p[0], p[1]) == field.unit_of(self.pos[0], self.pos[2])
                });
                if dug {
                    self.angle = world
                        .player_facing
                        .wrapping_add(((rand(1.0) - 0.5) * deg(120.0) as f32) as i16);
                    self.facing = self.angle;
                    self.action = Action::Appear;
                    self.target_speed = 1.5;
                    self.speed_step = 0.3;
                    self.speed = 1.5;
                    self.gravity = 1.0;
                    self.max_vy = -20.0;
                    self.vy = 5.0;
                }
            }
            Action::Appear => {
                self.pitch = angle_to([0.0, 0.0], [-self.vy, self.speed]);
                if self.on_ground {
                    self.scurry();
                }
            }
            Action::Fly => {
                chase_angle(&mut self.pitch, 0, 0x1000);
                self.timer -= 1.0;
                if self.timer <= 0.0 {
                    self.target_speed = (1.1 - rand(0.2)) * 1.5;
                    self.timer = 10.0;
                }
                self.pulse();
                if self.mole_water_ahead(field) {
                    return false;
                }
                let (dx, dz) = (
                    self.centre_bk[0] - self.pos[0],
                    self.centre_bk[1] - self.pos[2],
                );
                if dx * dx + dz * dz >= RANGE * RANGE {
                    let soft = field
                        .attribute(self.pos[0], self.pos[2])
                        .is_some_and(is_hole);
                    self.start_fading();
                    self.pitch = 0;
                    if soft {
                        // aIKR_dug_init: sink slowly, nose down.
                        self.action = Action::Climb;
                        self.gravity = 0.01;
                        self.max_vy = -0.2;
                        self.vy = 0.0;
                        self.speed = 0.0;
                        self.target_speed = 0.0;
                        self.speed_step = 0.0;
                    } else {
                        self.action = Action::LetEscape;
                        self.gravity = 1.0;
                        self.max_vy = -20.0;
                    }
                    return false;
                }
                self.mole_turn();
            }
            Action::LetEscape => {
                self.pulse();
                if !self.mole_water_ahead(field) {
                    self.mole_turn();
                }
            }
            // Burrowing (aIKR_dug).
            Action::Climb => {
                self.pulse();
                chase_angle(&mut self.pitch, deg(157.5), 0x300);
            }
            Action::TouchWater => {
                self.pitch = angle_to([0.0, 0.0], [-self.vy, self.speed]);
                if field
                    .water_y(self.pos[0], self.pos[2])
                    .is_some_and(|w| self.pos[1] <= w)
                {
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    /// `aIKR_avoid_init`.
    fn scurry(&mut self) {
        self.action = Action::Fly;
        self.target_speed = 1.5;
        self.speed_step = 0.3;
        self.pitch = 0;
        self.timer = 0.0;
    }

    /// `aIKR_calc_move_scale`: y and z swell 0.8-1.0 and back.
    fn pulse(&mut self) {
        let (target, dir) = if self.dips == 0 {
            (0.8, -1.0)
        } else {
            (1.0, 1.0)
        };
        let mut s = self.scale[2] + dir * 0.2 * 0.5;
        if dir * (target - s) <= 0.0 {
            self.dips ^= 1;
            s = target;
        }
        self.scale[1] = s;
        self.scale[2] = s;
    }

    /// `aIKR_calc_direction_angl`.
    fn mole_turn(&mut self) {
        if self.wall_front {
            self.angle = self.angle.wrapping_add(deg(90.0));
        }
        chase_angle(&mut self.facing, self.angle, 0x800);
    }

    /// `aIKR_chk_water_attr`, then dive (`aIKR_dive_init`).
    fn mole_water_ahead(&mut self, field: &impl Field) -> bool {
        if !self.on_ground {
            return false;
        }
        let reach = 5.0 + self.speed;
        let (x, z) = (
            self.pos[0] + reach * sin_s(self.angle),
            self.pos[2] + reach * cos_s(self.angle),
        );
        if field.attribute(x, z).is_some_and(is_water) {
            self.action = Action::TouchWater;
            self.target_speed = 1.5;
            self.speed_step = 0.3;
            self.speed = 1.5;
            self.vy = 8.0;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{world, Flat};
    use super::*;

    #[test]
    fn a_mole_cricket_waits_for_the_shovel_then_scurries_off() {
        let mut r = |n: f32| n * 0.5;
        let at = Flat.unit_center([0, 0, 8, 8]);
        let mut m = Insect::new(MOLE_CRICKET, at, &Flat, &mut r).unwrap();
        for _ in 0..30 {
            m.step(&world(&[]), &Flat, &mut r);
        }
        assert!(m.hidden());
        let mut w = world(&[]);
        w.scoop = Some(at);
        m.step(&w, &Flat, &mut r);
        assert_eq!(m.action, Action::Appear);
        let mut seen = vec![m.action];
        for _ in 0..1200 {
            m.step(&world(&[]), &Flat, &mut r);
            if seen.last() != Some(&m.action) {
                seen.push(m.action);
            }
        }
        assert_eq!(seen[..2], [Action::Appear, Action::Fly], "{seen:?}");
        let (dx, dz) = (m.pos[0] - at[0], m.pos[2] - at[1]);
        assert!(dx * dx + dz * dz > 100.0 * 100.0, "it ran off");
    }
}
