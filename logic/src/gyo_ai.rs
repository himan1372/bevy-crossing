//! A river fish's behaviour (`ac_gyo_test.c`, `aGTT_*`): wait, swim,
//! near (to the bobber), touch (nibbles), bite, escape. Reeling in
//! (`aGTT_comeback`) is the caller's.
//!
//! One `step` is one 60 Hz tick (the PC port's timers: decomp frame counts
//! x2, `chase_f` steps x0.5). Positions are game units (x, z); angles are
//! `s16` turns (0x10000 = 360 deg, 0 = +z). `rand(n)` is `RANDOM_F(n)`:
//! uniform in 0..n.

use crate::uki::{
    BACK_SPEED, BITE_TIME, GYOEI_TYPES, SEARCH_ANGLE, SEARCH_DIST, TOUCH_COUNT, TOUCH_DISTANCE,
    TRASH_BY_SIZE,
};

/// `aGTT_speed`: the approach speed by size.
const NEAR_SPEED: [f32; 8] = [1.0, 1.25, 1.25, 1.5, 1.75, 2.0, 2.2, 2.2];
/// `aGYO_SIZE_XS`: sizes up to this flee a splash from closer.
const SIZE_XS: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Swim,
    Wait,
    Escape,
    Near,
    Touch,
    Bite,
}

/// What the fish sees of the bobber (`UKI_ACTOR`).
#[derive(Clone, Copy, Debug)]
pub struct Bobber {
    pub pos: [f32; 2],
    /// It has just hit the water (`hit_water_flag`).
    pub splashed: bool,
    /// The cast has settled (`cast_timer == 0`).
    pub settled: bool,
    /// No other fish has it (`gyo_status == 1`), or this one does (2).
    pub free: bool,
}

/// The fish's surroundings for one tick.
#[derive(Clone, Copy, Debug)]
pub struct World {
    pub bobber: Option<Bobber>,
    pub player: [f32; 2],
    /// Running (`mPlayer_INDEX_DASH`), or a tool hit nearby.
    pub player_dashing: bool,
    pub tool_hit: bool,
    /// Reverse of the water's flow here (`aGTT_Get_flow_angle_rv`).
    pub flow_rv: i16,
    /// The next step would leave the water (`aGYO_check_wall`).
    pub wall: bool,
    pub golden_rod: bool,
}

/// What happened this tick, for the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    None,
    /// It took the bobber (`uki->gyo_command = 1`).
    Took,
    /// A nibble: the bobber dips (`touched_flag`).
    Nibble,
    /// It bit (`gyo_command = 2`): reel now.
    Bit,
    /// It let go of the bobber and fled.
    GotAway,
}

#[derive(Clone, Debug)]
pub struct Gyo {
    pub gyo_type: u8,
    pub action: Action,
    pub pos: [f32; 2],
    /// `world.angle.y` and `shape_info.rotation.y`.
    pub angle: i16,
    pub rotation: i16,
    pub speed: f32,
    size: usize,
    swim_flag: u8,
    flags: u16,
    work0: i32,
    touch_counter: i16,
    fwork3: f32,
    swork2: i16,
    swork3: i16,
}

pub(crate) fn sin_s(a: i16) -> f32 {
    (a as f32 * std::f32::consts::PI / 32768.0).sin()
}

pub(crate) fn cos_s(a: i16) -> f32 {
    (a as f32 * std::f32::consts::PI / 32768.0).cos()
}

fn deg(d: f32) -> i16 {
    (d * 65536.0 / 360.0) as i32 as i16
}

pub(crate) fn angle_to(from: [f32; 2], to: [f32; 2]) -> i16 {
    let a = (to[0] - from[0]).atan2(to[1] - from[1]);
    (a * 32768.0 / std::f32::consts::PI) as i32 as i16
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// `chase_f`: step `v` toward `target`; true once there.
pub(crate) fn chase_f(v: &mut f32, target: f32, step: f32) -> bool {
    if (*v - target).abs() <= step {
        *v = target;
        true
    } else {
        *v += step.copysign(target - *v);
        false
    }
}

/// `chase_angle`.
pub(crate) fn chase_angle(a: &mut i16, target: i16, step: i16) {
    let diff = target.wrapping_sub(*a);
    *a = if diff.unsigned_abs() <= step as u16 {
        target
    } else if diff > 0 {
        a.wrapping_add(step)
    } else {
        a.wrapping_sub(step)
    };
}

/// `RANDOM2_F(n)`: uniform in -n/2..n/2.
fn rand2(rand: &mut impl FnMut(f32) -> f32, n: f32) -> f32 {
    rand(n) - n * 0.5
}

impl Gyo {
    /// `aGTT_actor_init`: waiting, facing upstream.
    pub fn new(
        gyo_type: u8,
        pos: [f32; 2],
        flow_rv: i16,
        rand: &mut impl FnMut(f32) -> f32,
    ) -> Self {
        let size = GYOEI_TYPES
            .get(gyo_type as usize)
            .map_or(1, |t| t.size as usize)
            .min(7);
        let mut g = Self {
            gyo_type,
            action: Action::Wait,
            pos,
            angle: flow_rv,
            rotation: flow_rv,
            speed: 0.0,
            size,
            swim_flag: 0,
            flags: 0,
            work0: 0,
            touch_counter: 0,
            fwork3: 19.0,
            swork2: 0,
            swork3: 0,
        };
        g.setup(Action::Wait, flow_rv, rand);
        g
    }

    pub fn size(&self) -> usize {
        self.size
    }

    fn set_angle(&mut self, a: i16) {
        self.angle = a;
        self.rotation = a;
    }

    fn setup(&mut self, action: Action, flow_rv: i16, rand: &mut impl FnMut(f32) -> f32) {
        self.action = action;
        match action {
            Action::Swim => {
                match self.swim_flag {
                    0 => {
                        self.fwork3 = 50.0;
                        self.swork2 = 0;
                        self.swork3 = 0;
                    }
                    1 => {
                        self.fwork3 = 0.0;
                        self.swork2 = 0;
                        self.swork3 = rand2(rand, 32768.0) as i16;
                        self.angle = self.angle.wrapping_add(self.swork3);
                    }
                    _ => {
                        self.flags |= 0x80;
                        self.fwork3 = 0.0;
                        self.swork2 = flow_rv;
                        self.swork3 = self.angle.wrapping_add(rand2(rand, 16384.0) as i16);
                        self.rotation = self.angle;
                    }
                }
                self.speed = 0.0;
            }
            Action::Wait => {
                self.flags &= !2;
                self.work0 = ((100.0 + rand(30.0)) * 2.0) as i32;
                self.speed = -0.15 + rand2(rand, 0.2);
            }
            Action::Escape => {
                self.work0 = 100;
                self.speed = 2.0;
            }
            Action::Near => {
                self.flags |= 2;
                self.speed = NEAR_SPEED[self.size];
            }
            Action::Touch => {
                self.work0 = 0;
                self.touch_counter = 5;
                self.speed = 0.0;
            }
            Action::Bite => {
                self.speed = 0.0;
            }
        }
    }

    /// `aGTT_player_near`: flee a running player or a tool hit nearby.
    fn player_near(&mut self, w: &World) -> bool {
        let d = dist(self.pos, w.player);
        if (d < 110.0 && w.player_dashing) || (d < 150.0 && w.tool_hit) {
            self.set_angle(angle_to(self.pos, w.player).wrapping_add(deg(180.0)));
            true
        } else {
            false
        }
    }

    /// `aGTT_search_Uki`: true when the bobber is in sight. A splash
    /// too close sends the fish off.
    fn search_bobber(&mut self, w: &World, rand: &mut impl FnMut(f32) -> f32) -> bool {
        let Some(b) = w.bobber else { return false };
        let area = GYOEI_TYPES
            .get(self.gyo_type as usize)
            .map_or(2, |t| t.search_area as usize);
        let d = dist(self.pos, b.pos);
        let to = angle_to(self.pos, b.pos);
        let escape_dist = if self.size <= SIZE_XS { 17.0 } else { 22.0 };
        if b.splashed && d < escape_dist {
            self.set_angle(to.wrapping_add(deg(180.0)));
            self.setup(Action::Escape, w.flow_rv, rand);
            return false;
        }
        let half = deg(SEARCH_ANGLE[w.golden_rod as usize][area.min(4)]);
        let goal = to.wrapping_sub(self.rotation);
        self.flags & 1 == 0
            && b.settled
            && d < SEARCH_DIST[area.min(4)]
            && goal > -half
            && goal < half
    }

    /// One 60 Hz tick: act, then move along `angle` at `speed`.
    pub fn step(&mut self, w: &World, rand: &mut impl FnMut(f32) -> f32) -> Event {
        let mut event = Event::None;
        match self.action {
            Action::Wait => {
                // aGTT_flow_direction: turn upstream.
                let off = w.flow_rv.wrapping_sub(self.angle).unsigned_abs() > 0x4000;
                chase_angle(&mut self.angle, w.flow_rv, if off { 0x400 } else { 0x100 });
                self.rotation = self.angle;
                self.work0 -= 1;
                if self.work0 <= 0 {
                    let r = rand(3.0);
                    self.swim_flag = if r < 1.0 {
                        0
                    } else if r < 2.0 {
                        1
                    } else {
                        2
                    };
                    self.setup(Action::Swim, w.flow_rv, rand);
                } else if self.player_near(w) {
                    self.setup(Action::Escape, w.flow_rv, rand);
                } else if self.search_bobber(w, rand) {
                    self.setup(Action::Near, w.flow_rv, rand);
                }
            }
            Action::Swim => {
                if w.wall {
                    self.turn_from_wall();
                    self.setup(Action::Escape, w.flow_rv, rand);
                } else {
                    let done = match self.swim_flag {
                        0 => self.swim_speed(360.0, 5.0, 0.5),
                        1 => {
                            let d = self.swim_speed(180.0, 5.0, 0.5);
                            if d {
                                self.angle = self.rotation;
                            }
                            d
                        }
                        _ if self.flags & 0x80 != 0 => {
                            chase_angle(&mut self.angle, self.swork3, 0x400);
                            self.rotation = self.angle;
                            if self.angle.wrapping_sub(self.swork3).unsigned_abs() < 0x400 {
                                self.flags &= !0x80;
                            }
                            false
                        }
                        _ => self.swim_speed(180.0, 5.0, 1.0),
                    };
                    if done {
                        self.setup(Action::Wait, w.flow_rv, rand);
                    } else if self.player_near(w) {
                        self.setup(Action::Escape, w.flow_rv, rand);
                    } else if self.search_bobber(w, rand) {
                        self.setup(Action::Near, w.flow_rv, rand);
                    }
                }
            }
            Action::Near => match w.bobber {
                Some(b) => {
                    let area = GYOEI_TYPES
                        .get(self.gyo_type as usize)
                        .map_or(2, |t| t.search_area as usize);
                    self.set_angle(angle_to(self.pos, b.pos));
                    let d = dist(self.pos, b.pos);
                    if d > SEARCH_DIST[area.min(4)] {
                        self.setup(Action::Wait, w.flow_rv, rand);
                    } else if d < TOUCH_DISTANCE[self.size] {
                        if b.free {
                            self.setup(Action::Touch, w.flow_rv, rand);
                            event = Event::Took;
                        } else {
                            self.setup(Action::Wait, w.flow_rv, rand);
                        }
                    }
                }
                None => self.setup(Action::Wait, w.flow_rv, rand),
            },
            Action::Touch => match w.bobber {
                Some(b) => {
                    self.work0 -= 1;
                    if self.work0 <= 0 {
                        self.speed = NEAR_SPEED[self.size];
                        self.set_angle(angle_to(self.pos, b.pos));
                        if dist(self.pos, b.pos) < TOUCH_DISTANCE[self.size] {
                            self.touch_counter -= 1;
                            if rand(4.0) < 1.0 || self.touch_counter <= 0 {
                                // 1 in 20: it's trash after all.
                                if rand(20.0) < 1.0 {
                                    self.gyo_type = TRASH_BY_SIZE[self.size];
                                }
                                let bite = BITE_TIME[w.golden_rod as usize][GYOEI_TYPES
                                    .get(self.gyo_type as usize)
                                    .map_or(2, |t| t.bite_time as usize)
                                    .min(4)];
                                self.work0 = (bite * 2.0) as i32;
                                self.setup(Action::Bite, w.flow_rv, rand);
                                event = Event::Bit;
                            } else {
                                self.work0 = ((TOUCH_COUNT[self.size] as f32 + rand2(rand, 30.0))
                                    * 2.0) as i32;
                                self.speed = BACK_SPEED[self.size] + rand2(rand, 0.2);
                                event = Event::Nibble;
                            }
                        }
                    }
                }
                None => {
                    self.setup(Action::Escape, w.flow_rv, rand);
                    event = Event::GotAway;
                }
            },
            Action::Bite => {
                // On the hook: held at the bobber until reeled or let go.
                if let Some(b) = w.bobber {
                    self.pos = b.pos;
                }
                self.work0 -= 1;
                if self.work0 <= 0 || w.bobber.is_none() {
                    self.setup(Action::Escape, w.flow_rv, rand);
                    event = Event::GotAway;
                }
            }
            Action::Escape => {
                if !self.player_near(w) {
                    if self.search_bobber(w, rand) {
                        self.setup(Action::Near, w.flow_rv, rand);
                    } else if self.flags & 0x40 != 0 || !w.wall {
                        self.work0 -= 1;
                        if self.work0 <= 0 {
                            self.flags &= !0x40;
                            self.setup(Action::Wait, w.flow_rv, rand);
                        } else {
                            chase_f(&mut self.speed, 0.0, 0.02);
                        }
                    } else {
                        self.turn_from_wall();
                    }
                }
            }
        }
        if self.action != Action::Bite {
            // Actor_position_moveF, at the PC port's half step per tick.
            if w.wall && self.speed > 0.0 {
                self.turn_from_wall();
            } else {
                self.pos[0] += sin_s(self.angle) * self.speed * 0.5;
                self.pos[1] += cos_s(self.angle) * self.speed * 0.5;
            }
        }
        event
    }

    /// `aGYO_check_wall`: a quarter turn off the bank.
    fn turn_from_wall(&mut self) {
        self.set_angle(self.angle.wrapping_add(deg(90.0)));
        self.flags |= 0x40;
    }

    /// `aGTT_swim_speed_check`: speed follows a sine as `fwork3` runs
    /// toward `target` degrees.
    fn swim_speed(&mut self, target: f32, step: f32, speed: f32) -> bool {
        let done = chase_f(&mut self.fwork3, target, step * 0.5);
        self.speed = speed * sin_s(deg(self.fwork3));
        done
    }
}

/// `mCoBG_GetWaterFlow` + `aGTT_Get_flow_angle_rv`: the reverse of the
/// water's flow for a unit attribute (water, waterfall, river N..NE, sea).
pub fn flow_angle_rv(attribute: u8) -> i16 {
    const FLOW: [[f32; 2]; 10] = [
        [0.0, 0.0],
        [0.0, 0.0],
        [0.0, -0.5],
        [-0.353_553_38, -0.353_553_38],
        [-0.5, 0.0],
        [-0.353_553_38, 0.353_553_38],
        [0.0, 0.5],
        [0.353_553_38, 0.353_553_38],
        [0.5, 0.0],
        [0.353_553_38, -0.353_553_38],
    ];
    let [x, z] = match attribute {
        24 => [0.0, -1.0],
        a if (12..=21).contains(&a) => FLOW[(a - 12) as usize],
        _ => [0.0, 0.0],
    };
    // atans_table(z, x): angle of (x, z) with 0 = +z.
    let a = (x.atan2(z) * 32768.0 / std::f32::consts::PI) as i32 as i16;
    a.wrapping_add(deg(180.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(bobber: Option<Bobber>) -> World {
        World {
            bobber,
            player: [1000.0, 1000.0],
            player_dashing: false,
            tool_hit: false,
            flow_rv: 0,
            wall: false,
            golden_rod: false,
        }
    }

    #[test]
    fn fish_in_front_of_a_settled_bobber_goes_for_it_and_bites() {
        let mut r = |n: f32| n * 0.1; // low rolls: keep waiting, bite early
        let mut g = Gyo::new(0, [0.0, 0.0], 0, &mut r);
        let b = Bobber {
            pos: [0.0, 30.0],
            splashed: false,
            settled: true,
            free: true,
        };
        let mut events = Vec::new();
        for _ in 0..2000 {
            let e = g.step(&world(Some(b)), &mut r);
            if e != Event::None {
                events.push(e);
            }
            if e == Event::Bit {
                break;
            }
        }
        assert_eq!(events.first(), Some(&Event::Took));
        assert_eq!(events.last(), Some(&Event::Bit));
    }

    #[test]
    fn a_running_player_scares_it_off() {
        let mut r = |n: f32| n * 0.5;
        let mut g = Gyo::new(0, [0.0, 0.0], 0, &mut r);
        let mut w = world(None);
        w.player = [0.0, 50.0];
        w.player_dashing = true;
        g.step(&w, &mut r);
        assert_eq!(g.action, Action::Escape);
        // Facing away from the player (who is at +z).
        assert!(g.angle.unsigned_abs() > 0x6000);
    }

    #[test]
    fn flow_points_upstream() {
        // RIVER_S (18) flows +z; upstream is -z (0x8000).
        assert_eq!(flow_angle_rv(18).unsigned_abs(), 0x8000);
    }
}
