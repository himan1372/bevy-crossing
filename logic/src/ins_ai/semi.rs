//! Cicadas (`ac_ins_semi.c`): stood up on a tree trunk, shivering as they
//! sing; off and away when the tree is shaken or a tool comes near.

use super::{Action, Field, Insect, World, UNIT};
use crate::gyo_ai::deg;

const ROBUST_CICADA: u8 = 4;
const BROWN_CICADA: u8 = 7;
/// `CEDAR_TREE`: a cicada sits differently on a cedar.
const CEDAR_TREE: u16 = 0x0861;
/// `aISM_*_SCARE_DIST`.
const NET_SCARE: f32 = 70.0;
const SCOOP_SCARE: f32 = 30.0;
const AXE_SCARE: f32 = 150.0;
/// `aISM_CRY_TIMER`.
const CRY_TIMER: f32 = 60.0;

pub(super) fn is_cicada(kind: u8) -> bool {
    (ROBUST_CICADA..=BROWN_CICADA).contains(&kind)
}

fn near(p: Option<[f32; 2]>, pos: [f32; 3], r: f32) -> bool {
    p.is_some_and(|p| (p[0] - pos[0]).powi(2) + (p[1] - pos[2]).powi(2) < r * r)
}

impl Insect {
    /// `aISM_actor_init` (and `aIKB_actor_init`) on a tree: 35 up the trunk (30 on a cedar),
    /// pitched upright, facing 180 degrees.
    pub(super) fn on_trunk(kind: u8, pos: [f32; 2], field: &impl Field) -> Option<Self> {
        let u = field.unit_of(pos[0], pos[1])?;
        let cedar = field.item(u) == Some(CEDAR_TREE);
        let (up, dz) = if cedar { (30.0, 8.0) } else { (35.0, -2.0) };
        let y = field.ground_y(pos[0], pos[1]) + up;
        let mut ins = Insect::blank(kind, [pos[0], y, pos[1] + dz], u);
        ins.pitch = deg(90.0);
        ins.facing = i16::MIN;
        ins.action = Action::Wait;
        Some(ins)
    }

    /// `aISM_actor_move`.
    pub(super) fn cicada_step(&mut self, world: &World, rand: &mut impl FnMut(f32) -> f32) {
        if self.action != Action::Wait {
            // aISM_avoid: flap, and climb ever faster.
            self.anim = (self.anim + 0.5) % 2.0;
            self.gravity = (self.gravity * 1.1).min(12.0);
            return;
        }
        if self.tree_scared(world) {
            self.cicada_fly_off(Action::Avoid, world, rand);
        } else if self.patience < 50.0 {
            // Singing: shiver about home every tick the cry timer is out.
            self.timer -= 1.0;
            if !world.raining && self.timer < 0.0 {
                self.timer = 0.0;
                self.pos[0] = self.home[0] + rand(0.4);
            }
        } else {
            self.timer = CRY_TIMER;
        }
    }

    /// `aISM_check_patience`.
    pub(super) fn tree_scared(&mut self, world: &World) -> bool {
        let shaken = near(world.tree_shaken, self.pos, UNIT);
        let axe = near(world.axe_hit, self.pos, UNIT)
            && near(
                Some([world.player[0], world.player[2]]),
                self.pos,
                AXE_SCARE,
            );
        if shaken
            || axe
            || near(world.net_stop, self.pos, NET_SCARE)
            || near(world.scoop, self.pos, SCOOP_SCARE)
        {
            self.patience = 100.0;
        }
        self.patience > 90.0
    }

    /// `aISM_avoid_init` (and let-escape): fly off fading, turned a
    /// little off south, or past the player for a released one.
    pub(super) fn cicada_fly_off(
        &mut self,
        action: Action,
        world: &World,
        rand: &mut impl FnMut(f32) -> f32,
    ) {
        let r = rand(1.0);
        self.action = action;
        self.start_fading();
        self.speed = 4.0;
        self.max_vy = 12.0;
        self.gravity = 0.06;
        self.pitch = 0;
        self.facing = if action == Action::Avoid {
            let side = if r >= 0.5 { deg(22.5) } else { -deg(22.5) };
            ((r * deg(90.0) as f32) as i16)
                .wrapping_sub(deg(45.0))
                .wrapping_add(side)
        } else {
            world
                .player_facing
                .wrapping_add(((rand(1.0) - 0.5) * deg(120.0) as f32) as i16)
        };
        self.angle = self.facing;
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{world, Flat};
    use super::*;
    use crate::ins_ai::Event;

    #[test]
    fn a_cicada_sings_until_the_net_comes_down_then_flies_off() {
        let mut r = |n: f32| n * 0.5;
        let mut c = Insect::new(ROBUST_CICADA, [300.0, 300.0], &Flat, &mut r).unwrap();
        let home = c.pos;
        for _ in 0..120 {
            c.step(&world(&[]), &Flat, &mut r);
        }
        assert_eq!(c.action, Action::Wait);
        assert!((c.pos[0] - home[0]).abs() <= 0.4 && c.pos[1] == home[1]);
        let mut w = world(&[]);
        w.net_stop = Some([c.pos[0] + 30.0, c.pos[2]]);
        c.step(&w, &Flat, &mut r);
        assert_eq!(c.action, Action::Avoid);
        let gone = (0..200).position(|_| c.step(&world(&[]), &Flat, &mut r) == Event::Gone);
        assert!(gone.is_some());
        assert!(c.pos[1] > home[1] + 100.0, "climbs away");
    }
}
