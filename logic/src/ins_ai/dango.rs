//! Pill bugs (`ac_ins_dango.c`): hidden under their rock until an axe or
//! shovel strikes it; then they pop out, curl up while it's tense, and
//! crawl off past the player once calm. A net, shovel or axe within 70
//! curls them again; water ahead drowns them; 400 from the acre's middle
//! they go.

use super::{Action, Field, Insect, World, UNIT};
use crate::gyo_ai::{chase_angle, cos_s, sin_s};
use crate::ins_set::is_water;

pub(super) const PILL_BUG: u8 = 36;
/// `aIDG_chk_active_range`: 400 from the acre's middle.
const RANGE: f32 = 400.0;

impl Insect {
    /// `aIDG_actor_init`: hidden on its rock's unit.
    pub(super) fn pill_bug(pos: [f32; 2], field: &impl Field) -> Option<Self> {
        let u = field.unit_of(pos[0], pos[1])?;
        let y = field.ground_y(pos[0], pos[1]);
        let mut ins = Insect::blank(PILL_BUG, [pos[0], y, pos[1]], u);
        let c = field.unit_center([u[0], u[1], 8, 8]);
        ins.centre_bk = [c[0] - UNIT / 2.0, c[1] - UNIT / 2.0];
        ins.action = Action::Wait; // hide
        ins.hits = 1; // still on the rock's unit
        ins.gravity = 2.0;
        ins.max_vy = -20.0;
        Some(ins)
    }

    /// Under its rock (or up in its tree), not drawn.
    pub fn hidden(&self) -> bool {
        (self.kind == PILL_BUG && self.action == Action::Wait) || self.action == Action::Hide
    }

    /// `aIDG_actor_move`; true when it drowned.
    pub(super) fn pill_bug_step(
        &mut self,
        world: &World,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> bool {
        match self.action {
            Action::Wait => {
                // aIDG_check_strike_stone
                let struck = world.rock_struck.is_some_and(|p| {
                    field.unit_of(p[0], p[1]).map(|u| [u[2], u[3]]) == Some(self.unit)
                        && field.unit_of(p[0], p[1]).map(|u| [u[0], u[1]]) == Some(self.acre)
                });
                if struck {
                    self.crawl_off(world, rand);
                    self.anim = 0.0;
                    self.vy = 12.0;
                    self.action = Action::Landing; // appear
                }
            }
            Action::Landing => {
                if self.on_ground {
                    self.curl_up();
                }
            }
            Action::Rest => {
                if !self.water_ahead(field) && self.patience < 50.0 {
                    self.crawl_off(world, rand);
                }
            }
            Action::Fly => {
                if self.water_ahead(field) {
                    return false;
                }
                let (dx, dz) = (
                    self.centre_bk[0] - self.pos[0],
                    self.centre_bk[1] - self.pos[2],
                );
                if dx * dx + dz * dz >= RANGE * RANGE {
                    self.start_fading();
                    self.action = Action::LetEscape;
                    return false;
                }
                self.turn_off_walls();
                if self.hits == 1 {
                    // Off the rock's unit, it minds things again.
                    let here = field
                        .unit_of(self.pos[0], self.pos[2])
                        .map(|u| [u[2], u[3]]);
                    if here != Some(self.unit) {
                        self.hits = 0;
                    }
                } else if self.pill_bug_scared(world) {
                    self.curl_up();
                }
            }
            // Diving into water: under the surface, it's gone.
            Action::TouchWater => {
                if field
                    .water_y(self.pos[0], self.pos[2])
                    .is_some_and(|w| self.pos[1] <= w)
                {
                    return true;
                }
            }
            Action::LetEscape => {
                let wet = self.water_ahead(field);
                if !wet {
                    self.turn_off_walls();
                }
            }
            _ => {}
        }
        self.facing_follow();
        false
    }

    /// `aIDG_chk_water_attr`: on the ground with water just ahead, it
    /// dives in.
    fn water_ahead(&mut self, field: &impl Field) -> bool {
        if !self.on_ground {
            return false;
        }
        let reach = 2.0 + self.speed;
        let (x, z) = (
            self.pos[0] + reach * sin_s(self.angle),
            self.pos[2] + reach * cos_s(self.angle),
        );
        if field.attribute(x, z).is_some_and(is_water) {
            self.action = Action::TouchWater;
            self.speed = 1.5;
            self.speed_step = 0.3;
            self.target_speed = 1.5;
            self.vy = 8.0;
            self.anim = 0.0;
            return true;
        }
        false
    }

    /// `aIDG_calc_direction_angl`: a wall ahead turns it along the wall.
    fn turn_off_walls(&mut self) {
        if self.wall_front {
            self.angle = self.angle.wrapping_add(0x4000);
        }
    }

    fn facing_follow(&mut self) {
        chase_angle(&mut self.facing, self.angle, 0x800);
    }

    /// `aIDG_check_patience`: net, shovel or axe within 70, or stress.
    fn pill_bug_scared(&mut self, world: &World) -> bool {
        let near = |p: Option<[f32; 2]>| {
            p.is_some_and(|p| (p[0] - self.pos[0]).powi(2) + (p[1] - self.pos[2]).powi(2) < 4900.0)
        };
        if near(world.net_stop) || near(world.scoop) || near(world.axe_hit) {
            self.patience = 100.0;
        }
        self.patience > 90.0
    }

    /// `aIDG_avoid_init`: crawl off within 60 degrees of the player's way.
    fn crawl_off(&mut self, world: &World, rand: &mut impl FnMut(f32) -> f32) {
        self.action = Action::Fly;
        self.speed = 1.5;
        self.target_speed = 1.5;
        self.speed_step = 0.3;
        self.anim = 1.0;
        self.angle = world
            .player_facing
            .wrapping_add((21845.0 * (rand(1.0) - 0.5)) as i16);
        self.facing = self.angle;
    }

    /// `aIDG_stop_init`: curl up and roll to a stop.
    fn curl_up(&mut self) {
        self.action = Action::Rest;
        self.target_speed = 0.0;
        self.speed_step = 0.05;
        self.anim = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{world, Flat};
    use super::*;

    #[test]
    fn a_pill_bug_hides_until_its_rock_is_struck_then_crawls_off() {
        let mut r = |n: f32| n * 0.5;
        let at = Flat.unit_center([0, 0, 5, 5]);
        let mut p = Insect::new(PILL_BUG, at, &Flat, &mut r).unwrap();
        for _ in 0..60 {
            p.step(&world(&[]), &Flat, &mut r);
        }
        assert!(p.hidden());
        let mut w = world(&[]);
        w.rock_struck = Some(at);
        p.step(&w, &Flat, &mut r);
        assert!(!p.hidden());
        let mut seen = Vec::new();
        for _ in 0..200 {
            p.step(&world(&[]), &Flat, &mut r);
            if seen.last() != Some(&p.action) {
                seen.push(p.action);
            }
        }
        assert!(
            seen.contains(&Action::Rest) && seen.contains(&Action::Fly),
            "{seen:?}"
        );
    }
}
