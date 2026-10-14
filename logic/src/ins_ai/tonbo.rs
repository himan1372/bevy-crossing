//! Dragonflies (`ac_ins_tonbo.c`): dart, slow to a stop, turn, dart
//! again at 52-62 over the ground; dip into fresh water; rest on a
//! reserve post. The banded dragonfly patrols without stopping and leaves
//! once 12 units from its acre's centre.

use super::{Action, Field, Insect, World, UNIT};
use crate::gyo_ai::{angle_to, deg};

const COMMON_DRAGONFLY: u8 = 9;
const DARNER_DRAGONFLY: u8 = 11;
const BANDED_DRAGONFLY: u8 = 12;
/// `DUMMY_RESERVE`: the post a dragonfly rests on.
const DUMMY_RESERVE: u16 = 0xF102;

/// `aITB_STOP_TYPE_*`.
const STOP_NONE: u8 = 0;
const STOP_COLLISION: u8 = 1;
const STOP_REST: u8 = 2;
/// `aITB_ONIYAMA_MAX_RANGE`, `aITB_OTHER_MAX_RANGE`.
const BANDED_RANGE: f32 = 12.0 * UNIT;
const OTHER_RANGE: f32 = 6.0 * UNIT;
const REST_TIMER: f32 = 200.0;

pub(super) fn is_dragonfly(kind: u8) -> bool {
    (COMMON_DRAGONFLY..=BANDED_DRAGONFLY).contains(&kind)
}

/// `add_calc_short_angle2(a, target, 1 - sqrt(0.7), 2500, 0)`.
fn ease_angle(a: &mut i16, target: i16) {
    let diff = target.wrapping_sub(*a) as f32;
    let step = (diff * (1.0 - 0.7f32.sqrt())).clamp(-2500.0, 2500.0);
    *a = a.wrapping_add(step as i16);
}

/// `add_calc(v, target, 1 - sqrt(0.8), 0.5, 0)`.
fn ease(v: &mut f32, target: f32) {
    *v += ((target - *v) * (1.0 - 0.8f32.sqrt())).clamp(-0.5, 0.5);
}

impl Insect {
    /// `aITB_actor_init`: 20 over the ground, any heading, cruising at
    /// 52-62; `None` in the rain or on a building (`check_live_condition`).
    pub(super) fn dragonfly(
        kind: u8,
        pos: [f32; 2],
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> Option<Self> {
        let u = field.unit_of(pos[0], pos[1])?;
        match field.item(u) {
            None => return None,
            Some(i) if i >> 12 == 5 => return None,
            _ => {}
        }
        let y = field.ground_y(pos[0], pos[1]) + 20.0;
        let mut ins = Insect::blank(kind, [pos[0], y, pos[1]], u);
        ins.gravity = 0.1;
        ins.height = 52.0 + rand(10.0);
        ins.angle = i16::MIN.wrapping_sub((rand(1.0) * 65536.0) as i32 as i16);
        ins.facing = ins.angle;
        let c = field.unit_center([u[0], u[1], 8, 8]);
        ins.centre_bk = [c[0] - UNIT / 2.0, c[1] - UNIT / 2.0];
        let first = if kind == BANDED_DRAGONFLY {
            Action::Patrol
        } else {
            Action::Fly
        };
        ins.dragonfly_setup(first, None, field, rand);
        Some(ins)
    }

    /// `aITB_actor_move`.
    pub(super) fn dragonfly_step(
        &mut self,
        world: &World,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) {
        self.phase100 = (self.phase100 + 1) % 100;
        match self.action {
            Action::LetEscape | Action::Avoid => {
                self.gravity = (self.gravity * 1.1).min(12.0);
            }
            Action::Fly => {
                ease_angle(&mut self.facing, self.angle);
                self.height_ctrl(field);
                if self.check_stop(field) {
                    let next = if self.stop == STOP_REST {
                        Action::FlyToRest
                    } else {
                        Action::Wait
                    };
                    self.dragonfly_setup(next, Some(world), field, rand);
                } else {
                    self.fly_ctrl(world, field, rand);
                }
            }
            Action::Patrol => {
                ease_angle(&mut self.facing, self.angle);
                self.height_ctrl(field);
                self.bg_check(world, field, rand);
                let (dx, dz) = (
                    self.centre_bk[0] - self.pos[0],
                    self.centre_bk[1] - self.pos[2],
                );
                if self.action == Action::Patrol && dx * dx + dz * dz >= BANDED_RANGE * BANDED_RANGE
                {
                    self.dragonfly_setup(Action::LetEscape, Some(world), field, rand);
                }
            }
            Action::Wait => {
                ease_angle(&mut self.facing, self.angle);
                self.height_ctrl(field);
                if (self.facing.wrapping_sub(self.angle) as i32).abs() < deg(2.8125) as i32 {
                    self.stop = STOP_NONE;
                    self.dragonfly_setup(Action::Fly, Some(world), field, rand);
                }
            }
            Action::TouchWater => {
                ease_angle(&mut self.facing, self.angle);
                let in_water = field
                    .water_y(self.pos[0], self.pos[2])
                    .is_some_and(|w| self.pos[1] <= w);
                if in_water {
                    self.dragonfly_setup(Action::TouchWaterUp, Some(world), field, rand);
                }
            }
            Action::TouchWaterUp => {
                ease_angle(&mut self.facing, self.angle);
                self.timer = (self.timer - 1.0).max(0.0);
                if self.timer <= 0.0 {
                    self.dragonfly_setup(Action::HoverOnWater, Some(world), field, rand);
                }
            }
            Action::HoverOnWater => {
                self.timer = (self.timer - 1.0).max(0.0);
                if self.timer <= 0.0 {
                    let next = if self.dips < 2 {
                        self.dips += 1;
                        Action::TouchWater
                    } else {
                        Action::Wait
                    };
                    self.dragonfly_setup(next, Some(world), field, rand);
                }
            }
            Action::FlyToRest => {
                ease_angle(&mut self.facing, self.angle);
                if self.patience < 90.0 {
                    ease(&mut self.pos[1], self.rest_y);
                    let c = field
                        .unit_of(self.pos[0], self.pos[2])
                        .map_or([self.pos[0], self.pos[2]], |u| field.unit_center(u));
                    let (dx, dz) = (c[0] - self.pos[0], c[1] - self.pos[2]);
                    if dx * dx + dz * dz > 25.0 {
                        self.angle = angle_to([self.pos[0], self.pos[2]], c);
                    } else if (self.rest_y - self.pos[1]).abs() < 3.0 {
                        self.dragonfly_setup(Action::RestOnSpot, Some(world), field, rand);
                    }
                } else {
                    self.stop = STOP_NONE;
                    self.patience = 100.0;
                    self.dragonfly_setup(Action::Fly, Some(world), field, rand);
                }
            }
            Action::RestOnSpot => {
                if self.timer > 0.0 && self.patience < 90.0 {
                    ease(&mut self.pos[1], self.rest_y);
                    self.timer = (self.timer - 1.0).max(0.0);
                    // Wings settle over the first 20 ticks.
                    if REST_TIMER - self.timer < 20.0 {
                        self.anim = (self.anim + 0.5) % 2.0;
                    }
                } else {
                    self.stop = STOP_NONE;
                    self.patience = 100.0;
                    self.dragonfly_setup(Action::Fly, Some(world), field, rand);
                }
            }
            _ => {}
        }
        if self.stop != STOP_REST {
            self.anim = (self.anim + 0.5) % 2.0;
        }
    }

    /// `aITB_height_ctrl`: climb under the cruise height, sink above it.
    fn height_ctrl(&mut self, field: &impl Field) {
        let g = field.ground_y(self.pos[0], self.pos[2]);
        self.max_vy = if self.height + g > self.pos[1] {
            1.0
        } else {
            -1.0
        };
    }

    /// `aITB_BGcheck`: a wall ahead or the ground turns it up to 45
    /// degrees off south; four in a row and it gives up and leaves.
    /// True when it collided (and stayed).
    fn bg_check(
        &mut self,
        world: &World,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> bool {
        if !(self.wall_front || self.on_ground) {
            self.hits = 0;
            return false;
        }
        let swing = (rand(1.0) * deg(45.0) as f32) as i16;
        self.angle = if self.angle >= 0 { swing } else { -swing };
        self.hits += 1;
        if self.hits >= 4 {
            self.dragonfly_setup(Action::LetEscape, Some(world), field, rand);
            false
        } else {
            true
        }
    }

    /// `aITB_check_stop`: a reserve post to rest on, or something in the way.
    fn check_stop(&mut self, field: &impl Field) -> bool {
        self.stop = STOP_NONE;
        if self.kind == BANDED_DRAGONFLY {
            return false;
        }
        let u = field.unit_of(self.pos[0], self.pos[2]);
        if self.patience < 40.0 && u.and_then(|u| field.item(u)) == Some(DUMMY_RESERVE) {
            if let Some(top) = u.and_then(|u| field.column_top(u)) {
                self.stop = STOP_REST;
                self.rest_y = top;
                return true;
            }
        }
        if self.wall_front || self.on_ground {
            self.stop = STOP_COLLISION;
            return true;
        }
        false
    }

    /// `aITB_fly_ctrl`: at speed, count down; then brake; stopped, dip
    /// into fresh water now and then, else wait and turn.
    fn fly_ctrl(&mut self, world: &World, field: &impl Field, rand: &mut impl FnMut(f32) -> f32) {
        if self.speed != self.target_speed {
            return;
        }
        if self.timer > 0.0 {
            self.timer -= 1.0;
        } else if self.speed == 0.0 {
            let fresh = field
                .attribute(self.pos[0], self.pos[2])
                .is_some_and(|a| (12..=21).contains(&a));
            if self.kind != BANDED_DRAGONFLY && self.phase100 < 20 && fresh {
                self.dips = 1;
                self.dragonfly_setup(Action::TouchWater, Some(world), field, rand);
            } else {
                self.dragonfly_setup(Action::Wait, Some(world), field, rand);
            }
        } else {
            self.target_speed = 0.0;
        }
    }

    fn dragonfly_setup(
        &mut self,
        action: Action,
        world: Option<&World>,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) {
        self.action = action;
        match action {
            Action::LetEscape | Action::Avoid => {
                self.start_fading();
                self.height = 0.0;
                self.speed = 4.0;
                self.max_vy = 12.0;
                self.gravity = 0.06;
                self.pitch = 0;
                self.vy = 0.0;
                if let (Action::Avoid, Some(w)) = (action, world) {
                    self.angle = angle_to([w.player[0], w.player[2]], [self.pos[0], self.pos[2]]);
                }
            }
            Action::Fly | Action::Patrol => {
                self.facing = self.angle;
                // aITB_move_spd_set
                let (base, timer) = match self.kind {
                    BANDED_DRAGONFLY => (4.0, self.timer),
                    DARNER_DRAGONFLY => (2.4, 24.0),
                    _ => (2.0, 20.0),
                };
                self.target_speed = base + rand(2.0);
                self.timer = timer;
                self.speed_step = 0.2;
            }
            Action::Wait | Action::TouchWater => {
                self.wait_init(world, field, rand);
                if action == Action::TouchWater {
                    self.max_vy = -1.0;
                }
            }
            Action::TouchWaterUp => {
                self.max_vy = 4.0;
                self.vy = 4.0;
                self.timer = 6.0;
            }
            Action::HoverOnWater => {
                self.max_vy = 0.0;
                self.vy = 0.0;
                self.timer = 20.0;
            }
            Action::FlyToRest => {
                self.patience = 0.0;
                self.hits = 0;
                self.dips = 0;
                self.max_vy = 0.0;
                self.speed = 1.0;
                self.target_speed = 2.0 + rand(2.0);
            }
            Action::RestOnSpot => {
                self.speed = 0.0;
                self.speed_step = 0.0;
                self.target_speed = 0.0;
                self.timer = REST_TIMER;
            }
            _ => {}
        }
    }

    /// `aITB_wait_init`: a new heading (a swerve near the acre's middle,
    /// back toward it when far), stopped; turn off walls ahead.
    fn wait_init(
        &mut self,
        world: Option<&World>,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) {
        const SWERVE: [f32; 4] = [-67.5, -33.75, 33.75, 67.5];
        if self.stop == STOP_NONE {
            let (dx, dz) = (
                self.centre_bk[0] - self.pos[0],
                self.centre_bk[1] - self.pos[2],
            );
            if dx * dx + dz * dz < OTHER_RANGE * OTHER_RANGE {
                let i = (rand(4.0) as usize).min(3);
                self.angle = self.angle.wrapping_add(deg(SWERVE[i]));
            } else {
                self.angle = angle_to([self.pos[0], self.pos[2]], self.centre_bk);
            }
        }
        self.patience = 0.0;
        self.hits = 0;
        self.speed = 0.0;
        self.speed_step = 0.0;
        self.timer = 0.0;
        // Probe 12 ahead; a wall turns it, up to four times.
        for _ in 0..4 {
            let (s, c) = (
                crate::gyo_ai::sin_s(self.angle),
                crate::gyo_ai::cos_s(self.angle),
            );
            let ahead = [self.pos[0] + s * 12.0, self.pos[1], self.pos[2] + c * 12.0];
            self.wall_front = field.blocked(self.pos, ahead);
            self.on_ground = false;
            let Some(w) = world else { break };
            if !self.bg_check(w, field, rand) {
                break;
            }
        }
        self.wall_front = false;
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{world, Flat};
    use super::*;

    #[test]
    fn a_dragonfly_darts_stops_turns_and_darts_again() {
        let mut r = |n: f32| n * 0.5;
        let mut d = Insect::new(COMMON_DRAGONFLY, [300.0, 300.0], &Flat, &mut r).unwrap();
        let mut seen = Vec::new();
        let start = d.pos;
        for _ in 0..600 {
            d.step(&world(&[]), &Flat, &mut r);
            if seen.last() != Some(&d.action) {
                seen.push(d.action);
            }
        }
        assert!(
            seen.windows(3)
                .any(|w| w == [Action::Fly, Action::Wait, Action::Fly]),
            "{seen:?}"
        );
        let moved = ((d.pos[0] - start[0]).powi(2) + (d.pos[2] - start[2]).powi(2)).sqrt();
        assert!(moved > 20.0, "it travels ({moved})");
        assert!(
            (50.0..=64.0).contains(&d.pos[1]),
            "cruises at its height ({})",
            d.pos[1]
        );
    }

    #[test]
    fn the_banded_dragonfly_patrols_then_leaves_far_from_the_middle() {
        let mut r = |n: f32| n * 0.5;
        let mut d = Insect::new(BANDED_DRAGONFLY, [100.0, 100.0], &Flat, &mut r).unwrap();
        d.centre_bk = [100.0, 100.0];
        let left = (0..2000).any(|_| {
            d.step(&world(&[]), &Flat, &mut r);
            d.action == Action::LetEscape
        });
        assert!(left);
    }
}
