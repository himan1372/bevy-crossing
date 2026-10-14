//! Fireflies (`ac_ins_hotaru.c`): hatch near fresh water, 70 over the
//! ground, drift about a target point while floating 10 up and down and
//! blinking; a stressful approach steers them away from the player (or
//! back toward the acre's middle), and once calm they settle on a unit.

use super::{Field, Insect, World, UNIT};
use crate::gyo_ai::{angle_to, deg, sin_s};

pub(super) const FIREFLY: u8 = 27;
/// `aIHT_anim_data`: the blink's brightness steps.
const BLINK: [f32; 6] = [0.0, 1.0, 2.0, 3.0, 2.0, 1.0];

/// `add_calc_short_angle2(a, target, 1 - sqrt(0.9), 250, 0)`.
fn ease_slow(a: &mut i16, target: i16) {
    let diff = target.wrapping_sub(*a) as f32;
    let step = (diff * (1.0 - 0.9f32.sqrt())).clamp(-250.0, 250.0);
    *a = a.wrapping_add(step as i16);
}

impl Insect {
    /// `aIHT_actor_init`: needs fresh water on its unit or one or two
    /// units off on an axis; it sets off from the last of those spots
    /// (two units north), 70 up, within half a unit.
    pub(super) fn firefly(
        pos: [f32; 2],
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> Option<Self> {
        const CHECK: [[f32; 2]; 9] = [
            [0.0, 0.0],
            [UNIT, 0.0],
            [-UNIT, 0.0],
            [0.0, UNIT],
            [0.0, -UNIT],
            [2.0 * UNIT, 0.0],
            [-2.0 * UNIT, 0.0],
            [0.0, 2.0 * UNIT],
            [0.0, -2.0 * UNIT],
        ];
        let mut water = false;
        let mut at = pos;
        for d in CHECK {
            let p = [pos[0] + d[0], pos[1] + d[1]];
            let fresh = field
                .attribute(p[0], p[1])
                .is_some_and(|a| (12..=21).contains(&a));
            if !water && fresh {
                water = true;
            } else {
                at = p;
            }
        }
        if !water {
            return None;
        }
        let u = field.unit_of(at[0], at[1])?;
        let y = field.ground_y(at[0], at[1]) + 70.0;
        let x = at[0] + rand(UNIT) - UNIT / 2.0;
        let z = at[1] + rand(UNIT) - UNIT / 2.0;
        let mut ins = Insect::blank(FIREFLY, [x, y, z], u);
        ins.action = super::Action::Fly;
        ins.phase = (rand(1.0) * 65536.0) as i32 as i16;
        ins.centre = [x, z];
        ins.corner = (rand(5.0) as usize).min(5);
        let c = field.unit_center([u[0], u[1], 8, 8]);
        ins.centre_bk = [c[0] - UNIT / 2.0, c[1] - UNIT / 2.0];
        Some(ins)
    }

    /// How bright it is now, 0..1 (the blink drives the point light's
    /// radius, 25-40, in `aIHT_light_proc`).
    pub fn glow(&self) -> f32 {
        if self.kind == FIREFLY {
            self.anim / 3.0
        } else {
            0.0
        }
    }

    /// `aIHT_actor_move`. `centre` is the target point, `aim` the heading
    /// it eases to, `phase` the float, `corner` the blink step, `dips`
    /// the scared flag.
    pub(super) fn firefly_step(
        &mut self,
        world: &World,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) {
        // aIHT_anime_proc: one blink step every 10 ticks.
        if self.timer > 0.0 {
            self.timer -= 1.0;
        } else {
            self.timer = 10.0;
            self.corner = (self.corner + 1) % BLINK.len();
            self.anim = BLINK[self.corner];
        }
        if self.fading {
            // aIHT_avoid: float fast and climb ever faster.
            self.phase = self.phase.wrapping_add(0x400);
            self.speed = 1.5;
            self.vy = self.home[1] + 10.0 * sin_s(self.phase) - self.pos[1];
            self.gravity += 0.75;
            self.vy += self.gravity;
            return;
        }
        // aIHT_BGcheck: a wall moves the target a unit back from it.
        if self.wall_front && self.hits == 0 {
            let (dx, dz) = (
                self.centre_bk[0] - self.pos[0],
                self.centre_bk[1] - self.pos[2],
            );
            let a = self.angle as u16;
            if a > deg(45.0) as u16 && a <= deg(135.0) as u16 {
                if dx >= -200.0 {
                    self.centre[0] -= UNIT;
                }
            } else if a > deg(135.0) as u16 && a <= deg(225.0) as u16 {
                if dz >= -200.0 {
                    self.centre[1] -= UNIT;
                }
            } else if a > deg(225.0) as u16 && a <= deg(315.0) as u16 {
                if dx <= 200.0 {
                    self.centre[0] += UNIT;
                }
            } else if dz <= 200.0 {
                self.centre[1] += UNIT;
            }
            self.hits = 1;
        } else if !self.wall_front {
            self.hits = 0;
        }
        self.target_speed = 1.5;
        self.speed_step = 0.1;
        self.phase = self
            .phase
            .wrapping_add(((rand(0x600 as f32) + 0x200 as f32) * 0.5) as i16);
        self.vy = self.home[1] + 10.0 * sin_s(self.phase) - self.pos[1];
        let here = [self.pos[0], self.pos[2]];
        if (self.centre[0] - here[0]).abs() > 30.0 || (self.centre[1] - here[1]).abs() > 30.0 {
            let wobble = deg(67.5) as i32 - rand(deg(135.0) as f32) as i32;
            self.aim = angle_to(here, self.centre).wrapping_add(wobble as i16);
            if self.patience > 90.0 {
                let (dx, dz) = (self.centre_bk[0] - here[0], self.centre_bk[1] - here[1]);
                self.aim = if dx.abs() > 240.0 || dz.abs() > 240.0 {
                    angle_to(here, self.centre_bk)
                } else {
                    angle_to(here, [world.player[0], world.player[2]]).wrapping_add(i16::MIN)
                };
                self.dips = 1;
            } else if self.dips == 1 && self.patience < 10.0 {
                if let Some(u) = field.unit_of(here[0], here[1]) {
                    self.centre = field.unit_center(u);
                }
                self.dips = 0;
            }
        }
        ease_slow(&mut self.angle, self.aim);
        self.facing = self.angle;
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::world;
    use super::*;

    /// Grass with a stream along x = 0..40.
    struct Stream;
    impl Field for Stream {
        fn ground_y(&self, _: f32, _: f32) -> f32 {
            0.0
        }
        fn water_y(&self, _: f32, _: f32) -> Option<f32> {
            None
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
        fn attribute(&self, x: f32, _: f32) -> Option<u8> {
            Some(if (0.0..40.0).contains(&x) { 14 } else { 0 })
        }
    }

    #[test]
    fn fireflies_need_water_and_hover_70_up_blinking() {
        let mut r = |n: f32| n * 0.5;
        assert!(Insect::new(FIREFLY, [300.0, 300.0], &Stream, &mut r).is_none());
        let mut f = Insect::new(FIREFLY, [20.0, 300.0], &Stream, &mut r).unwrap();
        let mut glows = std::collections::BTreeSet::new();
        for _ in 0..600 {
            f.step(&world(&[]), &Stream, &mut r);
            glows.insert((f.glow() * 3.0) as u8);
            assert!((55.0..=85.0).contains(&f.pos[1]), "{}", f.pos[1]);
        }
        assert_eq!(glows.len(), 4, "{glows:?}");
    }
}
