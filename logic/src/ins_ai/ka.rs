//! Mosquitoes (`ac_ins_ka.c`): circle and bob; in the player's acre and
//! within 60 of their height, home in, hover at them for 3 s and sting,
//! then fly off.

use super::{Action, Event, Field, Insect, World, UNIT};
use crate::gyo_ai::{angle_to, chase_angle, chase_f, deg, sin_s};

pub(super) const MOSQUITO: u8 = 39;
/// `aIKA_ATTACK_DIST`, `aIKA_ATTACK_TIME`.
const ATTACK_DIST: f32 = UNIT / 2.0;
const ATTACK_TIME: f32 = 180.0;

impl Insect {
    /// `aIKA_actor_init`: from the unit's ground (less 30; the floor check
    /// lifts it to 14 over).
    pub(super) fn mosquito(pos: [f32; 2], field: &impl Field) -> Option<Self> {
        let u = field.unit_of(pos[0], pos[1])?;
        let y = field.ground_y(pos[0], pos[1]) - 30.0;
        let mut ins = Insect::blank(MOSQUITO, [pos[0], y, pos[1]], u);
        ins.floor = 14.0;
        ins.action = Action::Fly;
        ins.speed = 1.2;
        Some(ins)
    }

    /// `aIKA_actor_move`.
    pub(super) fn mosquito_step(
        &mut self,
        world: &World,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> Event {
        // aIKA_fuwafuwa: bob by a sine whose size changes each cycle;
        // `phase` is the angle, `rest_y` the size, `rest_cool` the climb.
        self.anim = (self.anim + 0.2) % 2.0;
        let before = self.rest_y * sin_s(self.phase);
        let next = self.phase.wrapping_add(0x180);
        if next < 0 && self.phase >= 0 {
            self.rest_y = 10.0 + rand(10.0);
        }
        self.phase = next;
        chase_f(&mut self.rest_cool, self.max_vy, self.gravity * 0.5);
        self.vy = self.rest_cool + (self.rest_y * sin_s(self.phase) - before);

        let to_player = {
            let (dx, dz) = (world.player[0] - self.pos[0], world.player[2] - self.pos[2]);
            (dx * dx + dz * dz).sqrt()
        };
        // aIKA_check_condition: same acre, within 60 of the player's height.
        let near = !world.busy
            && field
                .unit_of(world.player[0], world.player[2])
                .map(|u| [u[0], u[1]])
                == Some(self.acre)
            && (world.player[1] - self.pos[1]).abs() < 60.0;
        let toward = angle_to(
            [self.pos[0], self.pos[2]],
            [world.player[0], world.player[2]],
        );
        match self.action {
            Action::Fly => {
                // aIKA_BGcheck: a wall or the floor turns it right round.
                if self.wall_front || self.on_ground {
                    self.angle = self.angle.wrapping_add(i16::MIN);
                }
                if world.busy {
                    self.mosquito_fly_off(world, rand);
                } else if near {
                    self.action = Action::Landing; // search
                } else {
                    self.angle = self.angle.wrapping_add(0x80);
                }
            }
            Action::Landing => {
                if world.busy {
                    self.mosquito_fly_off(world, rand);
                } else if !near {
                    self.action = Action::Fly;
                    self.speed = 1.2;
                } else if to_player <= ATTACK_DIST {
                    self.action = Action::Hover; // attack wait
                    self.timer = 0.0;
                } else {
                    let step = if to_player <= UNIT { 0x100 } else { 0x200 };
                    chase_angle(&mut self.angle, toward, step);
                }
            }
            Action::Hover => {
                if world.busy {
                    self.mosquito_fly_off(world, rand);
                } else if !near {
                    self.action = Action::Fly;
                    self.speed = 1.2;
                } else if to_player > ATTACK_DIST {
                    self.action = Action::Landing;
                } else {
                    chase_angle(&mut self.angle, toward, 0x600);
                    self.timer += 1.0;
                    if self.timer > ATTACK_TIME {
                        // The player is stung; once that's over it leaves.
                        self.mosquito_fly_off(world, rand);
                        self.facing = self.angle;
                        return Event::Stung;
                    }
                }
            }
            _ => {
                // aIKA_avoid
                self.gravity = (self.gravity * 1.1).min(12.0);
            }
        }
        self.facing = self.angle;
        Event::None
    }

    /// `aIKA_avoid_init`.
    fn mosquito_fly_off(&mut self, world: &World, rand: &mut impl FnMut(f32) -> f32) {
        self.action = Action::Avoid;
        self.start_fading();
        self.gravity = 0.06;
        self.max_vy = 12.0;
        self.speed = 4.0;
        self.pitch = 0;
        self.rest_cool = self.vy;
        self.angle = world
            .player_facing
            .wrapping_add(((rand(1.0) - 0.5) * deg(120.0) as f32) as i16);
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{world, Flat};
    use super::*;
    use crate::ins_ai::Mover;

    #[test]
    fn a_mosquito_homes_in_and_stings_after_three_seconds() {
        let mut r = |n: f32| n * 0.5;
        let mut m = Insect::new(MOSQUITO, [300.0, 300.0], &Flat, &mut r).unwrap();
        let player = Mover {
            pos: [340.0, 20.0, 300.0],
            moved: 0.0,
        };
        let stung =
            (0..1200).position(|_| m.step(&world(&[player]), &Flat, &mut r) == Event::Stung);
        assert!(matches!(stung, Some(180..=1200)), "{stung:?}");
        assert_eq!(m.action, Action::Avoid);
    }
}
