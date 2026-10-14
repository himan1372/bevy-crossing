//! Grasshoppers and crickets (`ac_ins_batta.c`): sit a while, turn, hop
//! a new way (never into water or up a step), stay within 6 units of the
//! acre's middle; scared, they hop away from the player every 8 ticks
//! until calm. One that lands in water drowns.

use super::{Action, Field, Insect, World, UNIT};
use crate::gyo_ai::{angle_to, cos_s, sin_s};
use crate::ins_set::is_water;

const LONG_LOCUST: u8 = 13;
const MIGRATORY_LOCUST: u8 = 14;
const BELL_CRICKET: u8 = 17;
const PINE_CRICKET: u8 = 18;
/// `aIBT_chk_active_range`: 240 from the acre's middle.
const RANGE: f32 = 240.0;
/// `HOLE_START..HOLE_END`.
const HOLES: (u16, u16) = (0x0011, 0x0029);

pub(super) fn is_hopper(kind: u8) -> bool {
    (LONG_LOCUST..=PINE_CRICKET).contains(&kind)
}

/// `add_calc_short_angle2(a, target, CALC_EASE(0.5), 0x2000, 0)`.
fn ease_half(a: &mut i16, target: i16) {
    let diff = target.wrapping_sub(*a) as f32;
    let step = (diff * (1.0 - 0.5f32.sqrt())).clamp(-8192.0, 8192.0);
    *a = a.wrapping_add(step as i16);
}

impl Insect {
    /// `aIBT_actor_init`; `None` on a hole, a building, furniture or an
    /// item (`aIBT_check_live_condition`).
    pub(super) fn hopper(
        kind: u8,
        pos: [f32; 2],
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> Option<Self> {
        let u = field.unit_of(pos[0], pos[1])?;
        let item = field.item(u)?;
        if (HOLES.0..=HOLES.1).contains(&item) || (1..=5).contains(&(item >> 12)) {
            return None;
        }
        let y = field.ground_y(pos[0], pos[1]);
        let mut ins = Insect::blank(kind, [pos[0], y, pos[1]], u);
        ins.gravity = 0.2;
        ins.max_vy = -20.0;
        let c = field.unit_center([u[0], u[1], 8, 8]);
        ins.centre_bk = [c[0] - UNIT / 2.0, c[1] - UNIT / 2.0];
        ins.hopper_setup(Action::Landing, None, field, rand);
        Some(ins)
    }

    fn in_range(&self) -> bool {
        let (dx, dz) = (
            self.centre_bk[0] - self.pos[0],
            self.centre_bk[1] - self.pos[2],
        );
        dx * dx + dz * dz < RANGE * RANGE
    }

    fn in_water(&self, field: &impl Field) -> bool {
        field
            .water_y(self.pos[0], self.pos[2])
            .is_some_and(|w| self.pos[1] < w)
    }

    /// `aIBT_anime_proc`: the bell cricket's wings go fast.
    fn hopper_flap(&mut self) {
        let step = if self.kind == BELL_CRICKET { 1.0 } else { 0.3 };
        self.anim = (self.anim + step) % 2.0;
    }

    /// `aIBT_actor_move`; true when it drowned.
    pub(super) fn hopper_step(
        &mut self,
        world: &World,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> bool {
        // `swerve` counts aINS_dt_phase(_254, 200) here.
        self.swerve = (self.swerve + 1.0) % 200.0;
        match self.action {
            // Change direction (aIBT_chg_direction); `corner` counts tries.
            Action::Landing => {
                if !self.in_range() {
                    self.angle = angle_to([self.pos[0], self.pos[2]], self.centre_bk);
                    self.hopper_setup(Action::Wait, Some(world), field, rand);
                    return false;
                }
                const SPREAD: [f32; 16] = [
                    4096.0, 4096.0, 8192.0, 8192.0, 8192.0, 12288.0, 12288.0, 12288.0, 12288.0,
                    12288.0, 16384.0, 16384.0, 16384.0, 16384.0, 16384.0, 16384.0,
                ];
                let a = self
                    .facing
                    .wrapping_add(i16::MIN)
                    .wrapping_add((SPREAD[self.corner.min(15)] * 2.0 * (rand(1.0) - 0.5)) as i16);
                let reach = if self.kind == MIGRATORY_LOCUST {
                    218.0
                } else {
                    53.0
                };
                let (x, z) = (
                    self.pos[0] + reach * sin_s(a),
                    self.pos[2] + reach * cos_s(a),
                );
                let rise = (self.pos[1] - field.ground_y(x, z)).abs();
                let wet = field.attribute(x, z).is_some_and(is_water);
                if rise < 40.0 && !wet {
                    self.angle = a;
                    self.hopper_setup(Action::Wait, Some(world), field, rand);
                } else {
                    self.corner += 1;
                    if self.corner > 15 {
                        self.hopper_setup(Action::Wait, Some(world), field, rand);
                    }
                }
            }
            Action::Wait => {
                if self.in_water(field) {
                    return true;
                }
                if self.hopper_scared(world) {
                    self.hopper_setup(Action::Avoid, Some(world), field, rand);
                    return false;
                }
                ease_half(&mut self.facing, self.angle);
                // Crickets sing on the ground while calm; the bell
                // cricket's wings move as it does.
                if self.kind == BELL_CRICKET && self.on_ground && self.patience < 20.0 {
                    self.hopper_flap();
                }
                if self.timer > 0.0 {
                    self.timer -= 1.0;
                } else {
                    let phase = self.swerve as i32;
                    let jump = !self.in_range()
                        || if matches!(self.kind, LONG_LOCUST | MIGRATORY_LOCUST) {
                            phase > 20
                        } else {
                            phase < 20
                        };
                    let next = if jump { Action::Hover } else { Action::Landing };
                    self.hopper_setup(next, Some(world), field, rand);
                }
            }
            // Jump (aIBT_jump): land, then pick a new direction.
            Action::Hover => {
                if self.on_ground {
                    self.hopper_setup(Action::Landing, Some(world), field, rand);
                }
            }
            Action::Avoid | Action::LetEscape => {
                self.hopper_flap();
                if self.in_water(field) {
                    return true;
                }
                if !self.on_ground {
                    return false;
                }
                if self.timer > 0.0 {
                    self.timer -= 1.0;
                    self.speed = 0.0;
                    return false;
                }
                self.timer = 8.0;
                let mut a = self.angle;
                if self.action == Action::Avoid {
                    if self.patience < 85.0 {
                        self.timer = 2.0 * (5.0 + rand(10.0));
                        self.hopper_setup(Action::Landing, Some(world), field, rand);
                        return false;
                    }
                    a = if self.in_range() {
                        angle_to(
                            [self.pos[0], self.pos[2]],
                            [world.player[0], world.player[2]],
                        )
                        .wrapping_add(i16::MIN)
                    } else {
                        angle_to([self.pos[0], self.pos[2]], self.centre_bk)
                    };
                }
                // aIBT_chk_avoid_jump_angle: a wall ahead turns it a quarter.
                if self.wall_front {
                    a = self
                        .angle
                        .wrapping_add(if rand(2.0) < 1.0 { -0x4000 } else { 0x4000 });
                }
                self.angle = a;
                self.facing = a;
                // aIBT_set_avoid_jump_spd
                if self.kind == MIGRATORY_LOCUST {
                    (self.speed, self.vy, self.gravity) = (7.5, 9.0, 0.6);
                } else {
                    (self.speed, self.vy, self.gravity) = (5.0, 3.0, 0.3);
                }
            }
            _ => {}
        }
        false
    }

    /// `aIBT_check_patience`: a net or shovel within 70 maxes it.
    fn hopper_scared(&mut self, world: &World) -> bool {
        let near = |p: Option<[f32; 2]>| {
            p.is_some_and(|p| (p[0] - self.pos[0]).powi(2) + (p[1] - self.pos[2]).powi(2) < 4900.0)
        };
        if near(world.net_stop) || near(world.scoop) {
            self.patience = 100.0;
        }
        self.patience >= 90.0
    }

    /// Init procs. `Landing` stands for change-direction, `Hover` for jump.
    fn hopper_setup(
        &mut self,
        action: Action,
        world: Option<&World>,
        _field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) {
        self.action = action;
        match action {
            Action::Avoid => {
                self.gravity = 0.2;
                self.timer = 0.0;
                if let Some(w) = world {
                    let a = angle_to([self.pos[0], self.pos[2]], [w.player[0], w.player[2]])
                        .wrapping_sub(i16::MIN)
                        .wrapping_add((8192.0 * 2.0 * (rand(1.0) - 0.5)) as i16);
                    self.angle = a;
                    self.facing = a;
                }
            }
            Action::LetEscape => {
                self.start_fading();
                self.gravity = 0.2;
                self.timer = 0.0;
                self.speed = 5.0;
                self.vy = 3.0;
                if let Some(w) = world {
                    let a = w
                        .player_facing
                        .wrapping_add((21845.0 * (rand(1.0) - 0.5)) as i16);
                    self.angle = a;
                    self.facing = a;
                }
            }
            Action::Landing => {
                self.corner = 0;
                self.anim = 0.0;
                self.speed = 0.0;
            }
            Action::Wait => {
                self.timer = if self.in_range() {
                    2.0 * (120.0 + rand(240.0))
                } else {
                    60.0
                };
            }
            Action::Hover => {
                self.vy = 4.0;
                self.speed = 5.5;
                self.gravity = 0.7;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{world, Flat};
    use super::*;
    use crate::ins_ai::Mover;

    #[test]
    fn a_grasshopper_sits_turns_and_hops_within_its_acre() {
        let mut r = |n: f32| n * 0.3;
        let start = [320.0, 320.0];
        let mut g = Insect::new(16, start, &Flat, &mut r).unwrap();
        let mut hops = 0;
        for _ in 0..6000 {
            let before = g.action;
            g.step(&world(&[]), &Flat, &mut r);
            hops += (before != Action::Hover && g.action == Action::Hover) as u32;
            assert!(g.pos[1] >= 0.0);
        }
        assert!(hops >= 2, "{hops} hops");
        assert!(g.in_range() || g.action == Action::Wait);
    }

    #[test]
    fn scared_it_hops_away_from_the_player() {
        let mut r = |n: f32| n * 0.5;
        let mut g = Insect::new(16, [320.0, 320.0], &Flat, &mut r).unwrap();
        for _ in 0..3 {
            g.step(&world(&[]), &Flat, &mut r);
        }
        let player = Mover {
            pos: [g.pos[0] - 30.0, 0.0, g.pos[2]],
            moved: 3.0,
        };
        for _ in 0..30 {
            g.step(&world(&[player]), &Flat, &mut r);
        }
        assert_eq!(g.action, Action::Avoid);
        assert!(
            g.pos[0] > 320.0,
            "away from the player on the -x side ({})",
            g.pos[0]
        );
    }
}
