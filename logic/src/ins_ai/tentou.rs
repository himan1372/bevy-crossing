//! Ladybugs, the mantis and the snail (`ac_ins_tentou.c`): on a flower,
//! 25 over the ground, waiting 3-6 s then crawling 3-5 s inside a box 14
//! either side of home, turning off its edges. Ladybugs and the mantis fly
//! off when scared; a snail off a flower drops and fades.

use super::{Action, Field, Insect, World};
use crate::gyo_ai::{chase_angle, deg};

const LADYBUG: u8 = 24;
const MANTIS: u8 = 26;
const SNAIL: u8 = 32;
/// `FLOWER_LEAVES_PANSIES0..FLOWER_TULIP2` (`IS_ITEM_FLOWER`).
const FLOWERS: (u16, u16) = (0x083C, 0x084D);
/// `aITT_*_SCARE_DIST`.
const NET_SCARE: f32 = 70.0;
const SCOOP_SCARE: f32 = 60.0;

pub(super) fn is_tentou(kind: u8) -> bool {
    (LADYBUG..=MANTIS).contains(&kind) || kind == SNAIL
}

impl Insect {
    /// `aITT_actor_init`: 6 over the flower's top (19 over the unit).
    pub(super) fn on_flower(
        kind: u8,
        pos: [f32; 2],
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> Option<Self> {
        let u = field.unit_of(pos[0], pos[1])?;
        let y = field.ground_y(pos[0], pos[1]) + 19.0 + 6.0;
        let mut ins = Insect::blank(kind, [pos[0], y, pos[1]], u);
        ins.tentou_setup(Action::Wait, None, rand);
        Some(ins)
    }

    /// `aITT_actor_move`. `phase` is the target heading, `dips` the turn
    /// delay.
    pub(super) fn tentou_step(
        &mut self,
        world: &World,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) {
        match self.action {
            Action::Avoid => {
                self.anim = (self.anim + 0.5) % 2.0;
                self.gravity = (self.gravity * 1.1).min(12.0);
            }
            // The snail's fall (aITT_avoid_maimai): walls turn it.
            Action::LetEscape => {
                chase_angle(&mut self.facing, self.angle, 0x800);
            }
            Action::Wait => {
                if !self.tentou_scared(world, field, rand) {
                    if self.timer <= 0.0 {
                        self.tentou_setup(Action::Fly, Some(world), rand);
                    } else {
                        self.timer -= 1.0;
                    }
                }
            }
            Action::Fly => {
                if self.tentou_scared(world, field, rand) {
                    return;
                }
                if self.timer <= 0.0 {
                    self.tentou_setup(Action::Wait, Some(world), rand);
                    return;
                }
                self.timer -= 1.0;
                // Kept inside 14 of home; the edge it hit picks the way back.
                const BACK: [f32; 16] = [
                    0.0, 90.0, -90.0, 0.0, 0.0, 45.0, -45.0, 0.0, 180.0, 135.0, -135.0, 180.0,
                    90.0, 90.0, -90.0, 0.0,
                ];
                let mut edge = 0;
                for (axis, lo, hi) in [(0, 1, 2), (2, 4, 8)] {
                    let d = self.pos[axis] - self.home[axis];
                    if d.abs() >= 15.0 {
                        self.pos[axis] = self.home[axis] + 14.0f32.copysign(d);
                        edge |= if d < 0.0 { lo } else { hi };
                    }
                }
                self.pos[1] = field.ground_y(self.pos[0], self.pos[2]) + 25.0;
                if edge != 0 {
                    self.phase = deg(BACK[edge]);
                    self.dips = 10;
                    self.speed_step = 0.0;
                    self.speed = 0.0;
                } else if self.angle == self.phase {
                    self.phase = self
                        .angle
                        .wrapping_add(((rand(2.0) - 1.0) * deg(90.0) as f32) as i16);
                }
                let step = if self.kind == SNAIL { 0x180 } else { 0x600 };
                chase_angle(&mut self.angle, self.phase, step);
                self.facing = self.angle;
                if self.dips == 0 {
                    self.speed_step = 0.1;
                } else {
                    self.dips -= 1;
                }
            }
            _ => {}
        }
    }

    /// `aITT_check_patience`: the snail needs its flower; the others mind
    /// the net, the shovel and a stressful approach.
    fn tentou_scared(
        &mut self,
        world: &World,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> bool {
        if self.kind == SNAIL {
            let on_flower = field
                .unit_of(self.pos[0], self.pos[2])
                .and_then(|u| field.item(u))
                .is_none_or(|i| (FLOWERS.0..=FLOWERS.1).contains(&i));
            if !on_flower {
                self.tentou_setup(Action::LetEscape, Some(world), rand);
                self.target_speed = 0.1;
                self.speed_step = 0.025;
                return true;
            }
            return false;
        }
        let near = |p: Option<[f32; 2]>, r: f32| {
            p.is_some_and(|p| (p[0] - self.pos[0]).powi(2) + (p[1] - self.pos[2]).powi(2) < r * r)
        };
        if near(world.net_stop, NET_SCARE) || near(world.scoop, SCOOP_SCARE) {
            self.patience = 100.0;
        }
        if self.patience > 90.0 {
            self.tentou_setup(Action::Avoid, Some(world), rand);
            return true;
        }
        false
    }

    /// Init procs. `Fly` is the crawl (aITT_move), `LetEscape` the
    /// snail's drop (avoid_maimai).
    fn tentou_setup(
        &mut self,
        action: Action,
        world: Option<&World>,
        rand: &mut impl FnMut(f32) -> f32,
    ) {
        self.action = action;
        let slow = if self.kind == SNAIL { 0.25 } else { 1.0 };
        match action {
            Action::Avoid | Action::LetEscape => {
                self.start_fading();
                self.speed = 4.0;
                self.max_vy = 12.0;
                self.gravity = 0.06;
                self.pitch = 0;
                if let Some(w) = world {
                    self.angle = w
                        .player_facing
                        .wrapping_add(((rand(1.0) - 0.5) * deg(120.0) as f32) as i16);
                    self.facing = self.angle;
                }
                if action == Action::LetEscape {
                    self.gravity = 2.0;
                    self.max_vy = -20.0;
                    self.speed = 0.2;
                    self.target_speed = 0.2;
                    self.speed_step = 0.05;
                }
            }
            Action::Fly => {
                self.timer = (90.0 + rand(60.0)) * 2.0;
                self.dips = 0;
                self.target_speed = 0.4 * slow;
                self.speed_step = 0.1 * slow;
            }
            Action::Wait => {
                self.timer = (90.0 + rand(90.0)) * 2.0;
                self.speed = 0.0;
                self.speed_step = 0.0;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{world, Flat};
    use super::*;

    #[test]
    fn a_ladybug_crawls_about_its_flower_and_stays_within_14() {
        let mut r = |n: f32| n * 0.7;
        let mut l = Insect::new(LADYBUG, [300.0, 300.0], &Flat, &mut r).unwrap();
        let home = l.pos;
        let mut crawled = false;
        for _ in 0..3000 {
            l.step(&world(&[]), &Flat, &mut r);
            crawled |= l.action == Action::Fly && l.speed > 0.0;
            assert!((l.pos[0] - home[0]).abs() < 15.0 && (l.pos[2] - home[2]).abs() < 15.0);
            assert_eq!(l.pos[1], 25.0);
        }
        assert!(crawled);
    }
}
