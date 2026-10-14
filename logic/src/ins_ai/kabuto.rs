//! Tree beetles (`ac_ins_kabuto.c`): on the trunk like a cicada, turning
//! their head a few degrees either way in bursts; off past the player
//! when scared.

use super::{Action, Insect, World};
use crate::gyo_ai::{chase_angle, deg};

const DRONE_BEETLE: u8 = 19;
const LONGHORN_BEETLE: u8 = 23;
const SAW_STAG_BEETLE: u8 = 29;
const GIANT_BEETLE: u8 = 31;

pub(super) fn is_beetle(kind: u8) -> bool {
    (DRONE_BEETLE..=LONGHORN_BEETLE).contains(&kind)
        || (SAW_STAG_BEETLE..=GIANT_BEETLE).contains(&kind)
}

impl Insect {
    /// `aIKB_actor_move`.
    pub(super) fn beetle_step(&mut self, world: &World, rand: &mut impl FnMut(f32) -> f32) {
        if self.action != Action::Wait {
            // aIKB_avoid
            self.anim = (self.anim + 0.5) % 2.0;
            self.gravity = (self.gravity * 1.1).min(12.0);
            return;
        }
        if self.tree_scared(world) {
            self.beetle_fly_off(world, rand);
            return;
        }
        // aIKB_wait: `timer` pauses, `dips` swings left, `hits` ticks a swing.
        const SIDES: [f32; 2] = [175.781_25, -175.781_25];
        if self.timer > 0.0 {
            self.timer -= 1.0;
            return;
        }
        chase_angle(&mut self.facing, deg(SIDES[(self.dips & 1) as usize]), 128);
        if self.hits == 0 {
            if self.dips == 0 {
                self.timer = ((10.0 + rand(10.0)) * 2.0).floor();
                self.dips = 3 + (rand(2.0) as u8).min(1);
            } else {
                self.dips -= 1;
            }
            self.hits = 30;
        } else {
            self.hits -= 1;
        }
    }

    /// `aIKB_avoid_init`: off past the player's heading, fading.
    fn beetle_fly_off(&mut self, world: &World, rand: &mut impl FnMut(f32) -> f32) {
        self.action = Action::Avoid;
        self.start_fading();
        self.gravity = 0.06;
        self.max_vy = 12.0;
        self.speed = 4.0;
        self.pitch = 0;
        self.angle = world
            .player_facing
            .wrapping_add(((rand(1.0) - 0.5) * deg(120.0) as f32) as i16);
        self.facing = self.angle;
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{world, Flat};
    use super::*;

    #[test]
    fn a_beetle_turns_its_head_a_little_and_flees_a_shaken_tree() {
        let mut r = |n: f32| n * 0.5;
        let mut b = Insect::new(DRONE_BEETLE, [300.0, 300.0], &Flat, &mut r).unwrap();
        let mut facings = std::collections::BTreeSet::new();
        for _ in 0..400 {
            b.step(&world(&[]), &Flat, &mut r);
            facings.insert(b.facing);
        }
        assert!(facings.len() > 2, "it turns");
        assert!(
            facings
                .iter()
                .all(|f| (*f as i32).abs() >= deg(175.0) as i32),
            "only a few degrees off 180"
        );
        let mut w = world(&[]);
        w.tree_shaken = Some([b.pos[0], b.pos[2]]);
        b.step(&w, &Flat, &mut r);
        assert_eq!(b.action, Action::Avoid);
    }
}
