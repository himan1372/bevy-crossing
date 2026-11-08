//! The radio calisthenics' routine (`ac_taisou_npc0.c`, `ac_taisou_npc0_anime.c_inc`, `ac_taisou_npc0_talk.c_inc`).
//!
//! The line plays ten measures: the intro, the exercises `taisou1`, `2`, `4`, `5`, `6`, `7`, `3`, `1` (the clip ids follow each
//! other in the disc's table, so measure 2 alternates `taisou2` / `taisou3_a` and so on, as `animeSeqNo[action] + (anime_dir & 1)`
//! does) and the trailing wait. Each measure is split into quarters (halves for the second and fourth) and every part plays the
//! next clip of its pair. The original stretches the clips to the radio's measure counter; the port plays them at their own pace,
//! one after another, so the music (not built) is the only thing missing from the sync.
//!
//! Mistakes (`aTS0_ctrl_gymnastic_sub0`): at a measure's end, when nobody is failing, a villager at place `n` slips with the
//! chance `[0, 0.05, 0.1, 0.15, 0.3]` (Copper and Tortimer never): they do a different exercise, notice it half a measure later
//! (`kiduku1`, sweating), and rejoin at the next measure. Tortimer tires 20 % of the time instead (`tired1`, one measure).
//!
//! Not built: the radio's own measure counter, the shared failure flag across both lines, the player's own exercises.

use bevy::prelude::*;

use super::{Line, Role};
use crate::character::NpcClip;
use crate::game_rand::GameRand;
use crate::states::GameplaySet;

/// The clips in the disc table's order (`ac_npc_data.c_inc` 0x51..0x5F), `animeSeqNo[action] + (anime_dir & 1)`.
const CLIPS: [&str; 15] = [
    "cKF_ba_r_npc_1_intro1_a",
    "cKF_ba_r_npc_1_intro1_b",
    "cKF_ba_r_npc_1_taisou1",
    "cKF_ba_r_npc_1_taisou2",
    "cKF_ba_r_npc_1_taisou3_a",
    "cKF_ba_r_npc_1_taisou3_b",
    "cKF_ba_r_npc_1_taisou4_a",
    "cKF_ba_r_npc_1_taisou4_b",
    "cKF_ba_r_npc_1_taisou5_a",
    "cKF_ba_r_npc_1_taisou5_b",
    "cKF_ba_r_npc_1_taisou6_a",
    "cKF_ba_r_npc_1_taisou6_b",
    "cKF_ba_r_npc_1_taisou7",
    "cKF_ba_r_npc_1_kiduku1",
    "cKF_ba_r_npc_1_kiduku2",
];
/// `animeSeqNo` by measure: the first clip index of each (the wait and the trailing note stand).
const BASE: [usize; 10] = [0, 2, 3, 6, 8, 10, 12, 4, 2, usize::MAX];
/// How many parts of each measure (`startRatio` entries minus the end).
const PARTS: [u8; 10] = [2, 4, 2, 4, 4, 4, 4, 4, 4, 1];
/// `chk_failure_rate` by the actor's place.
const FAILURE: [f32; 5] = [0.0, 0.05, 0.1, 0.15, 0.3];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    /// In step.
    Follow,
    /// Doing another exercise (`gymnastic_state` 2), until the measure's half.
    Wrong,
    /// Noticed it (`kiduku1`), until the measure ends.
    Notice,
    /// Tired (Tortimer, `gymnastic_state` 4), one measure.
    Tired,
}

#[derive(Component)]
pub(super) struct Routine {
    measure: usize,
    part: u8,
    state: State,
    /// Seconds to wait before starting (`delay_cnt`: up to 48 ticks).
    delay: f32,
    /// The measure the actor does while wrong.
    shown: usize,
    started: bool,
}

impl Routine {
    pub(super) fn new(rand: &mut GameRand, copper: bool) -> Self {
        Self {
            measure: 0,
            part: 0,
            state: State::Follow,
            delay: if copper {
                0.0
            } else {
                rand.0.fqrand() * 48.0 / 60.0
            },
            shown: 0,
            started: false,
        }
    }
}

/// The name of the clip for a part of a measure, if it is an exercise.
fn clip_of(measure: usize, part: u8) -> Option<&'static str> {
    let base = *BASE.get(measure)?;
    (base != usize::MAX).then(|| CLIPS[base + usize::from(part & 1)])
}

/// Set when one actor is failing, so two never fail at once (`aTS0_failure_actor_p`).
#[derive(Resource, Default)]
struct Failing(bool);

fn routine(
    time: Res<Time>,
    mut rand: ResMut<GameRand>,
    mut failing: ResMut<Failing>,
    mut line: Query<(&Role, &mut Routine, &mut NpcClip), With<Line>>,
) {
    let dt = time.delta_secs();
    for (role, mut r, mut clip) in &mut line {
        if r.delay > 0.0 {
            r.delay -= dt;
            continue;
        }
        // A clip is running: wait for it to finish.
        if r.started && clip.name.is_some() && !clip.done {
            continue;
        }
        if !r.started {
            r.started = true;
        } else {
            match r.state {
                State::Follow => {
                    r.part += 1;
                    if r.part >= PARTS[r.measure] {
                        next_measure(&mut r, role, &mut failing, &mut rand, &mut clip);
                    }
                }
                State::Wrong => {
                    r.part += 1;
                    if r.part >= (PARTS[r.shown] / 2).max(1) {
                        r.state = State::Notice;
                        clip.start(CLIPS[13], true, 8.0);
                        continue;
                    }
                }
                State::Notice | State::Tired => {
                    next_measure(&mut r, role, &mut failing, &mut rand, &mut clip);
                }
            }
            if r.state == State::Tired && clip.name == Some("cKF_ba_r_npc_1_tired1") && !clip.done {
                continue;
            }
        }
        let measure = if r.state == State::Wrong {
            r.shown
        } else {
            r.measure
        };
        match clip_of(measure, r.part) {
            Some(name) => clip.start(name, true, 8.0),
            // The trailing note stands a moment (the wait).
            None => {
                clip.stop(8.0);
                r.delay = 2.0;
            }
        }
    }
}

/// The measure is over: on to the next, where a villager may slip and Tortimer may tire.
fn next_measure(
    r: &mut Routine,
    role: &Role,
    failing: &mut Failing,
    rand: &mut GameRand,
    clip: &mut NpcClip,
) {
    if matches!(r.state, State::Wrong | State::Notice) {
        failing.0 = false;
    }
    r.state = State::Follow;
    r.measure = (r.measure + 1) % 10;
    r.part = 0;
    // `next_action` neither the intro nor the trailing note.
    if r.measure == 0 || r.measure == 9 {
        return;
    }
    let place = match role {
        Role::Villager(place, _) => usize::from(*place).min(4),
        _ => 0,
    };
    if !failing.0 && rand.0.fqrand() < FAILURE[place] {
        failing.0 = true;
        let mut other = 1 + rand.0.random(8) as usize;
        while other == r.measure {
            other = 1 + rand.0.random(8) as usize;
        }
        r.state = State::Wrong;
        r.shown = other;
    } else if *role == Role::Leader && rand.0.fqrand() < 0.2 {
        r.state = State::Tired;
        clip.start("cKF_ba_r_npc_1_tired1", true, 8.0);
    }
}

pub(super) fn register(app: &mut App) {
    app.init_resource::<Failing>()
        .add_systems(Update, routine.in_set(GameplaySet));
}
