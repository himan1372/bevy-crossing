//! Cockroaches (`ac_ins_goki.c`): on a tree trunk, on a spoiled turnip,
//! or else on a flower (by the item on their unit, as
//! `aSOI_ins_set_insect_data` picks). On trunks and turnips they wiggle
//! their heading; on a flower they crawl inside 14 of home. Scared, they
//! fly off a little either side of south.

use super::{Action, Field, Insect, World};
use crate::gyo_ai::{chase_angle, deg};
use crate::ins_set::is_tree;

pub(super) const COCKROACH: u8 = 28;
const SPOILED_TURNIP: u16 = 0x2F03;
/// `aIGK_PLACE_*`.
const ON_TREE: u8 = 4;
const ON_FLOWER: u8 = 5;
const ON_ITEM: u8 = 6;

impl Insect {
    /// `aIGK_actor_init`.
    pub(super) fn cockroach(pos: [f32; 2], field: &impl Field) -> Option<Self> {
        let u = field.unit_of(pos[0], pos[1])?;
        let item = field.item(u)?;
        let g = field.ground_y(pos[0], pos[1]);
        let mut ins = if is_tree(item) {
            let mut t = Insect::on_trunk(COCKROACH, pos, field)?;
            t.stop = ON_TREE;
            t
        } else if item == SPOILED_TURNIP {
            let mut t = Insect::blank(COCKROACH, [pos[0], g + 7.0, pos[1]], u);
            t.stop = ON_ITEM;
            t.facing = i16::MIN;
            t.pitch = deg(45.0);
            t
        } else {
            let mut t = Insect::blank(COCKROACH, [pos[0], g + 25.0, pos[1]], u);
            t.stop = ON_FLOWER;
            t
        };
        ins.home = ins.pos;
        let first = if ins.stop == ON_FLOWER {
            Action::Wait
        } else {
            Action::Rest
        };
        ins.roach_setup(first, None, &mut |_| 0.5);
        Some(ins)
    }

    /// `aIGK_actor_move`. Flower: `Wait`/`Fly`; tree and item: `Rest`
    /// (wait) and `Hover` (wiggle). `aim` is the target heading, `rest_cool`
    /// the move timer, `dips` the flips left.
    pub(super) fn roach_step(
        &mut self,
        world: &World,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) {
        if matches!(self.action, Action::Avoid | Action::LetEscape) {
            self.anim = (self.anim + 0.5) % 2.0;
            self.gravity = (self.gravity * 1.1).min(12.0);
            return;
        }
        if self.roach_scared(world, field) {
            self.roach_setup(Action::Avoid, Some(world), rand);
            return;
        }
        let turn = deg(8.4375);
        match self.action {
            Action::Wait => {
                self.timer -= 1.0;
                if self.timer <= 0.0 {
                    self.roach_setup(Action::Fly, Some(world), rand);
                } else {
                    chase_angle(&mut self.angle, self.aim, turn);
                    self.facing = self.angle;
                }
            }
            Action::Fly => {
                self.rest_cool -= 1.0;
                if self.rest_cool <= 0.0 {
                    self.timer = (2.0 * (90.0 + rand(90.0))).floor();
                    self.roach_setup(Action::Wait, Some(world), rand);
                    return;
                }
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
                if edge != 0 {
                    self.aim = deg(BACK[edge]);
                    self.timer = 10.0;
                    self.roach_setup(Action::Wait, Some(world), rand);
                } else {
                    chase_angle(&mut self.angle, self.aim, turn);
                    if self.angle == self.aim {
                        self.aim = self
                            .angle
                            .wrapping_add(((rand(1.0) - 0.5) * 2.0 * deg(90.0) as f32) as i16);
                    }
                    self.facing = self.angle;
                }
            }
            Action::Rest => {
                self.timer -= 1.0;
                if self.timer <= 0.0 {
                    self.roach_setup(Action::Hover, Some(world), rand);
                }
            }
            Action::Hover => {
                chase_angle(&mut self.facing, self.aim, 128);
                self.rest_cool -= 1.0;
                if self.rest_cool <= 0.0 {
                    if self.dips <= 1 {
                        self.roach_setup(Action::Rest, Some(world), rand);
                    } else {
                        self.dips -= 1;
                        self.aim = self.aim.wrapping_neg();
                        self.rest_cool = 30.0;
                    }
                }
            }
            _ => {}
        }
    }

    /// `aIGK_check_patience`.
    fn roach_scared(&mut self, world: &World, field: &impl Field) -> bool {
        let near = |p: Option<[f32; 2]>, r: f32| {
            p.is_some_and(|p| (p[0] - self.pos[0]).powi(2) + (p[1] - self.pos[2]).powi(2) < r * r)
        };
        let tree = self.stop == ON_TREE
            && (near(world.tree_shaken, 40.0)
                || (near(world.axe_hit, 40.0)
                    && near(Some([world.player[0], world.player[2]]), 150.0)));
        let gone = self.stop == ON_ITEM
            && field
                .unit_of(self.home[0], self.home[2])
                .and_then(|u| field.item(u))
                != Some(SPOILED_TURNIP);
        if tree || gone || near(world.net_stop, 70.0) || near(world.scoop, 30.0) {
            self.patience = 100.0;
        }
        self.patience >= 90.0
    }

    fn roach_setup(
        &mut self,
        action: Action,
        world: Option<&World>,
        rand: &mut impl FnMut(f32) -> f32,
    ) {
        self.action = action;
        match action {
            Action::Avoid | Action::LetEscape => {
                self.start_fading();
                self.speed = 4.0;
                self.max_vy = 12.0;
                self.gravity = 0.06;
                self.pitch = 0;
                self.vy = 0.0;
                let r = rand(1.0);
                self.angle = match (action, world) {
                    (Action::LetEscape, Some(w)) => w
                        .player_facing
                        .wrapping_add(((rand(1.0) - 0.5) * deg(120.0) as f32) as i16),
                    _ => {
                        let side = if r >= 0.5 { deg(22.5) } else { -deg(22.5) };
                        ((r * deg(90.0) as f32) as i16)
                            .wrapping_sub(deg(45.0))
                            .wrapping_add(side)
                    }
                };
                self.facing = self.angle;
            }
            Action::Wait => {
                self.speed = 0.0;
                self.speed_step = 0.0;
                self.target_speed = 0.0;
            }
            Action::Fly => {
                if self.rest_cool <= 0.0 {
                    self.rest_cool = (2.0 * (90.0 + rand(210.0))).floor();
                }
                self.target_speed = 0.4;
                self.speed_step = 0.1;
            }
            Action::Rest => {
                self.timer = (2.0 * (10.0 + rand(10.0))).floor();
            }
            Action::Hover => {
                self.aim = deg(175.781_25);
                self.dips = 3 + (rand(2.0) as u8).min(1);
                self.rest_cool = 30.0;
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
    fn on_a_flower_it_crawls_and_flies_off_at_the_net() {
        let mut r = |n: f32| n * 0.5;
        let mut c = Insect::new(COCKROACH, [300.0, 300.0], &Flat, &mut r).unwrap();
        let home = c.pos;
        let mut crawled = false;
        for _ in 0..1500 {
            c.step(&world(&[]), &Flat, &mut r);
            crawled |= c.speed > 0.0;
            assert!((c.pos[0] - home[0]).abs() < 15.0 && (c.pos[2] - home[2]).abs() < 15.0);
        }
        assert!(crawled);
        let mut w = world(&[]);
        w.net_stop = Some([c.pos[0], c.pos[2]]);
        c.step(&w, &Flat, &mut r);
        assert_eq!(c.action, Action::Avoid);
    }
}
