//! Bagworms and spiders (`ac_ins_mino.c`): hidden in a tree until it's
//! shaken, then down on a thread on the far side from the player, a
//! bounce, 20 s hanging (twisting, now and then squashing; another shake
//! swings it), and back up. If the tree is felled it drops and crawls off.

use super::{Action, Field, Insect, World};
use crate::gyo_ai::{chase_angle, chase_f, cos_s, deg, sin_s};
use crate::ins_set::{is_tree, is_water};

pub(super) const BAGWORM: u8 = 35;
pub(super) const SPIDER: u8 = 37;
/// The thread hangs at this pitch (`world.angle.x`).
const THREAD: f32 = 47.8125;
/// `scaleX_table` / `scaleY_table` (relative to 0.01).
const SQUASH: [[f32; 2]; 7] = [
    [1.0, 1.0],
    [0.875, 1.125],
    [1.125, 0.875],
    [0.875, 1.125],
    [1.125, 0.875],
    [0.875, 1.125],
    [1.125, 0.875],
];

pub(super) fn is_mino(kind: u8) -> bool {
    kind == BAGWORM || kind == SPIDER
}

impl Insect {
    /// `aIMN_actor_init`: up in the leaves, 65 over the ground.
    pub(super) fn hanger(kind: u8, pos: [f32; 2], field: &impl Field) -> Option<Self> {
        let u = field.unit_of(pos[0], pos[1])?;
        let y = field.ground_y(pos[0], pos[1]) + 65.0;
        let mut ins = Insect::blank(kind, [pos[0], y, pos[1]], u);
        ins.pitch = deg(THREAD);
        ins.facing = i16::MIN;
        ins.centre[1] = ins.pos[2] - 25.0;
        ins.mino_setup(Action::Hide, None, &mut |_| 0.5);
        Some(ins)
    }

    /// `aIMN_position_move`: along the thread by the counter `rest_y`.
    pub(super) fn thread_move(&mut self) {
        chase_f(&mut self.speed, self.target_speed, self.speed_step * 0.5);
        self.rest_y += self.speed * 0.5;
        let a = deg(THREAD);
        self.pos[1] = self.home[1] + cos_s(a) * self.rest_y;
        self.pos[2] = self.centre[1] - sin_s(a) * self.rest_y;
    }

    /// On the thread (custom move) rather than the ground.
    pub(super) fn on_thread(&self) -> bool {
        matches!(
            self.action,
            Action::Hide | Action::Appear | Action::AppearStop | Action::Hang | Action::Climb
        )
    }

    /// `aIMN_actor_move`; true when it drowned.
    pub(super) fn hanger_step(
        &mut self,
        world: &World,
        field: &impl Field,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> bool {
        let felled = self.on_thread()
            && !field
                .unit_of(self.home[0], self.home[2])
                .and_then(|u| field.item(u))
                .is_some_and(is_tree);
        if felled {
            self.mino_setup(Action::Fall, Some(world), rand);
            return false;
        }
        let shaken = world.tree_shaken.is_some_and(|p| {
            field.unit_of(p[0], p[1]) == field.unit_of(self.home[0], self.home[2])
        });
        match self.action {
            Action::Hide => {
                if shaken {
                    self.mino_setup(Action::Appear, Some(world), rand);
                }
            }
            Action::Appear => {
                let range = if self.dips == 1 { -47.0 } else { -41.0 };
                if self.rest_y < range {
                    if self.target_speed < 0.0 && self.speed < 0.0 {
                        if self.speed > -2.0 {
                            self.mino_setup(Action::AppearStop, Some(world), rand);
                        } else {
                            // Bounce on the thread.
                            self.target_speed = -self.speed * 0.2;
                            self.speed_step = 8.0;
                        }
                    }
                } else {
                    self.target_speed = -12.0;
                    self.speed_step = 2.0;
                }
                self.twist();
            }
            Action::AppearStop => {
                if self.speed == 0.0 {
                    self.mino_setup(Action::Hang, Some(world), rand);
                }
                self.twist();
            }
            Action::Hang => {
                if shaken {
                    self.phase = 0x80;
                    self.aim = -0x80;
                }
                self.timer -= 1.0;
                if self.timer <= 0.0 {
                    if self.aim == 0 {
                        self.scale = [1.0; 3];
                        self.mino_setup(Action::Climb, Some(world), rand);
                        return false;
                    }
                    self.timer = 0.0;
                } else {
                    self.idle_squash(rand);
                }
                self.swing();
                self.twist();
            }
            Action::Climb => {
                if self.pos[1] > self.home[1] {
                    self.mino_setup(Action::Hide, Some(world), rand);
                }
                self.twist();
            }
            Action::TouchWater => {
                if field
                    .water_y(self.pos[0], self.pos[2])
                    .is_some_and(|w| self.pos[1] <= w)
                {
                    return true;
                }
            }
            Action::Fall => {
                let g = field.ground_y(self.pos[0], self.pos[2]);
                if self.pos[1] <= g {
                    self.pos[1] = g;
                    self.mino_setup(Action::LetEscape, Some(world), rand);
                }
            }
            Action::LetEscape => {
                // aIMN_chk_water_attr, then turn off walls.
                let reach = 4.0 + self.speed;
                let (x, z) = (
                    self.pos[0] + reach * sin_s(self.angle),
                    self.pos[2] + reach * cos_s(self.angle),
                );
                if self.on_ground && field.attribute(x, z).is_some_and(is_water) {
                    self.mino_setup(Action::TouchWater, Some(world), rand);
                    return false;
                }
                if self.wall_front {
                    self.angle = self.angle.wrapping_add(deg(90.0));
                }
                let back = if self.kind == SPIDER { i16::MIN } else { 0 };
                chase_angle(&mut self.facing, self.angle.wrapping_add(back), 0x800);
            }
            _ => {}
        }
        false
    }

    /// `aIMN_calc_twist_angl`: roll 22.5 degrees each way.
    fn twist(&mut self) {
        self.twist = self.twist.wrapping_add(0x100);
        self.roll = (sin_s(self.twist) * deg(22.5) as f32) as i16;
    }

    /// `aIMN_calc_shake_angl`: a swing on the thread that halves each way
    /// until it stops; `phase` is the swing now, `aim` its target, `centre_bk`
    /// where it hangs.
    fn swing(&mut self) {
        let mut heading = self.facing;
        if (self.aim as i32) * (heading as i32) < 0 {
            self.aim = (-(self.aim as f32) * 0.5) as i16;
            if self.aim.unsigned_abs() < 16 {
                self.aim = 0;
                self.phase = 0;
                heading = i16::MIN;
            }
        }
        chase_angle(&mut self.phase, self.aim, 16);
        heading = heading.wrapping_add((self.phase as f32 * 0.5) as i16);
        self.facing = heading;
        self.pos[0] = self.centre_bk[0] - 50.0 * sin_s(heading);
        self.pos[1] = self.centre_bk[1] + 50.0 + 50.0 * cos_s(heading);
    }

    /// The hanging idle: long waits (`rest_cool`), then a six-step squash.
    fn idle_squash(&mut self, rand: &mut impl FnMut(f32) -> f32) {
        if self.rest_cool < 7.0 {
            let target = SQUASH[self.rest_cool as usize];
            let x = chase_f(&mut self.scale[0], target[0], 0.1125);
            let y = chase_f(&mut self.scale[1], target[1], 0.1125);
            if x && y {
                self.rest_cool -= 1.0;
                if self.rest_cool < 0.0 {
                    self.rest_cool = (0x133 as f32 + 60.0 * rand(5.0)).floor();
                }
            }
        } else {
            self.rest_cool -= 1.0;
        }
    }

    fn mino_setup(
        &mut self,
        action: Action,
        world: Option<&World>,
        rand: &mut impl FnMut(f32) -> f32,
    ) {
        self.action = action;
        match action {
            Action::Hide => {
                self.speed = 0.0;
                self.target_speed = 0.0;
                self.speed_step = 0.0;
                self.pos[1] = self.home[1];
                self.anim = 1.0;
            }
            Action::Appear => {
                // Down on the side away from the player.
                if let Some(w) = world {
                    let (x, z, far) = if w.player[0] < self.home[0] {
                        (self.home[0] + 30.0, self.home[2] - 18.0, false)
                    } else {
                        (self.home[0] - 30.0, self.home[2] - 25.0, true)
                    };
                    self.pos[0] = x;
                    self.pos[2] = z;
                    self.rest_y = if far { -6.0 } else { 0.0 };
                    if far {
                        self.pos[1] = self.home[1] - 47.0 + 41.0;
                    }
                    self.dips = far as u8;
                }
                self.centre[1] = self.pos[2];
                self.speed = 0.0;
                self.target_speed = -20.0;
                self.speed_step = 2.0;
            }
            Action::AppearStop => {
                self.speed_step = 8.0;
                self.target_speed = 0.0;
            }
            Action::Hang => {
                self.timer = 1200.0;
                self.speed = 0.0;
                self.target_speed = 0.0;
                self.speed_step = 0.0;
                self.rest_cool = (0x133 as f32 + 60.0 * rand(5.0)).floor();
                self.centre_bk = [self.pos[0], self.pos[1]];
            }
            Action::Climb => {
                self.target_speed = 0.5;
                self.speed_step = 0.025;
                self.roll = 0;
            }
            Action::Fall => {
                self.gravity = 2.0;
                self.max_vy = -20.0;
                self.roll = 0;
                self.speed = 0.0;
                self.anim = if self.kind == BAGWORM { 2.0 } else { 0.0 };
            }
            Action::TouchWater => {
                self.target_speed = 1.5;
                self.speed_step = 0.3;
                self.speed = 1.5;
                self.vy = 8.0;
            }
            Action::LetEscape => {
                self.start_fading();
                self.gravity = 2.0;
                self.max_vy = -20.0;
                self.pitch = 0;
                self.roll = 0;
                if let Some(w) = world {
                    self.angle = w
                        .player_facing
                        .wrapping_add(((rand(1.0) - 0.5) * deg(120.0) as f32) as i16);
                }
                let (speed, step, back) = if self.kind == BAGWORM {
                    (0.75, 0.15, 0)
                } else {
                    (1.5, 0.3, i16::MIN)
                };
                self.target_speed = speed;
                self.speed_step = step;
                self.facing = self.angle.wrapping_add(back);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::world;
    use super::*;

    /// A tree on every unit.
    struct Grove;
    impl Field for Grove {
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
            Some(0x0804)
        }
        fn blocked(&self, _: [f32; 3], _: [f32; 3]) -> bool {
            false
        }
        fn attribute(&self, _: f32, _: f32) -> Option<u8> {
            Some(0)
        }
    }

    #[test]
    fn a_bagworm_drops_when_the_tree_is_shaken_hangs_then_climbs_back() {
        let mut r = |n: f32| n * 0.5;
        let at = Grove.unit_center([0, 0, 5, 5]);
        let mut b = Insect::new(BAGWORM, at, &Grove, &mut r).unwrap();
        for _ in 0..30 {
            b.step(&world(&[]), &Grove, &mut r);
        }
        assert!(b.hidden());
        let mut w = world(&[]);
        w.tree_shaken = Some(at);
        b.step(&w, &Grove, &mut r);
        assert!(!b.hidden());
        let mut seen = vec![b.action];
        let mut lowest = b.pos[1];
        for _ in 0..1600 {
            b.step(&world(&[]), &Grove, &mut r);
            lowest = lowest.min(b.pos[1]);
            if seen.last() != Some(&b.action) {
                seen.push(b.action);
            }
        }
        assert_eq!(
            seen,
            [
                Action::Appear,
                Action::AppearStop,
                Action::Hang,
                Action::Climb,
                Action::Hide
            ]
        );
        assert!(lowest < 65.0 - 25.0, "it came down ({lowest})");
    }
}
