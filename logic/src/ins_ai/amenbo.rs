//! Pondskaters (`ac_ins_amenbo.c`): skate off at 2.3, glide to a stop
//! stretching and squashing, rest up to a second, skate on within 60
//! degrees of the last way; a bank sends them back the other way.

use super::{Action, Field, Insect, World, UNIT};
use crate::gyo_ai::{chase_f, cos_s, deg, sin_s};
use crate::ins_set::is_water;

pub(super) const POND_SKATER: u8 = 34;

impl Insect {
    /// `aIAB_actor_init`: 14 over the unit's floor.
    pub(super) fn pond_skater(
        pos: [f32; 2],
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> Option<Self> {
        let u = field.unit_of(pos[0], pos[1])?;
        let y = field.ground_y(pos[0], pos[1]) + 14.0;
        let mut ins = Insect::blank(POND_SKATER, [pos[0], y, pos[1]], u);
        ins.skate_setup(Action::Fly, None, rand);
        Some(ins)
    }

    /// `aIAB_actor_move`. `hits` is `aIAB_HIT_WALL`, `rest_y` the squash,
    /// `rest_cool` its rate.
    pub(super) fn skater_step(
        &mut self,
        world: &World,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) {
        // aIAB_BGcheck: a bank within 40 ahead.
        let (s, c) = (sin_s(self.angle), cos_s(self.angle));
        let ahead = [self.pos[0] + s * UNIT, self.pos[2] + c * UNIT];
        let bank = self.wall_front || !field.attribute(ahead[0], ahead[1]).is_some_and(is_water);
        if self.action != Action::LetEscape {
            self.pos[1] = field.ground_y(self.pos[0], self.pos[2]) + 14.0;
            self.vy = 0.0;
        }
        match self.action {
            Action::Fly => {
                chase_f(&mut self.rest_cool, -0.0021, 0.00035);
                self.rest_y = (self.rest_y + self.rest_cool).max(0.0);
                if bank {
                    self.hits = 1;
                }
                if self.speed == 0.0 || self.hits == 1 {
                    self.skate_setup(Action::Rest, Some(world), rand);
                }
            }
            Action::Rest => {
                if bank {
                    self.hits = 1;
                }
                self.timer -= 1.0;
                if self.timer <= 0.0 {
                    self.skate_setup(Action::Fly, Some(world), rand);
                }
            }
            _ => {
                self.anim = (self.anim + 0.5) % 2.0;
                self.gravity = (self.gravity + self.gravity * 0.1 * 0.5).min(12.0);
            }
        }
        self.facing = self.angle;
    }

    /// Its scale against the usual 0.01: a gliding pondskater stretches
    /// (`0.01 + s`, `0.01 - s`), a hanging bagworm squashes, a mole
    /// cricket's body pulses.
    pub fn scale_xyz(&self) -> [f32; 3] {
        if self.kind == POND_SKATER && self.action == Action::Fly {
            [1.0 + self.rest_y * 100.0, 1.0 - self.rest_y * 100.0, 1.0]
        } else {
            self.scale
        }
    }

    fn skate_setup(
        &mut self,
        action: Action,
        world: Option<&World>,
        rand: &mut impl FnMut(f32) -> f32,
    ) {
        let turn = |r: f32| ((r - 0.5) * deg(120.0) as f32) as i16;
        self.action = action;
        match action {
            Action::Fly => {
                let back = if self.hits == 1 { i16::MIN } else { 0 };
                self.angle = self.angle.wrapping_add(back).wrapping_add(turn(rand(1.0)));
                self.facing = self.angle;
                self.gravity = 0.3;
                self.max_vy = -2.0;
                self.target_speed = 0.0;
                self.speed_step = 0.05 - rand(0.03);
                self.speed = 2.3;
                self.rest_y = 0.0;
                self.rest_cool = 0.0021;
                self.hits = 0;
            }
            Action::Rest => {
                self.speed = 0.0;
                self.speed_step = 0.0;
                self.timer = if self.hits == 1 {
                    0.0
                } else {
                    2.0 * rand(30.0)
                };
                self.rest_y = 0.0;
            }
            Action::LetEscape => {
                self.start_fading();
                self.gravity = 0.06;
                self.max_vy = 12.0;
                self.speed = 4.0;
                if let Some(w) = world {
                    self.angle = w.player_facing.wrapping_add(turn(rand(1.0)));
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::world;
    use super::*;

    /// All water, flat.
    struct Pond;
    impl Field for Pond {
        fn ground_y(&self, _: f32, _: f32) -> f32 {
            0.0
        }
        fn water_y(&self, _: f32, _: f32) -> Option<f32> {
            Some(20.0)
        }
        fn unit_of(&self, x: f32, z: f32) -> Option<[i32; 4]> {
            super::super::tests::Flat.unit_of(x, z)
        }
        fn unit_center(&self, u: [i32; 4]) -> [f32; 2] {
            super::super::tests::Flat.unit_center(u)
        }
        fn item(&self, _: [i32; 4]) -> Option<u16> {
            Some(0)
        }
        fn blocked(&self, _: [f32; 3], _: [f32; 3]) -> bool {
            false
        }
        fn attribute(&self, _: f32, _: f32) -> Option<u8> {
            Some(12)
        }
    }

    #[test]
    fn a_pondskater_skates_glides_to_a_stop_and_skates_on() {
        let mut r = |n: f32| n * 0.5;
        let mut p = Insect::new(POND_SKATER, [300.0, 300.0], &Pond, &mut r).unwrap();
        let mut skates = 0;
        let start = p.pos;
        for _ in 0..1000 {
            let before = p.action;
            p.step(&world(&[]), &Pond, &mut r);
            skates += (before == Action::Rest && p.action == Action::Fly) as u32;
            assert_eq!(p.pos[1], 14.0);
        }
        assert!(skates >= 3, "{skates}");
        assert!((p.pos[0] - start[0]).abs() + (p.pos[2] - start[2]).abs() > 30.0);
    }
}
