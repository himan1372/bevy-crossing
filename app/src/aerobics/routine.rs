//! Native radio-clock-driven calisthenics (`ac_taisou_npc0_*`).
//! The audio sequence owns time; unavailable music leaves the line standing.
use bevy::prelude::*;
use rustimal_logic::taisou;

use super::{Line, Role};
use crate::character::{CharacterAnimationSet, NpcClip};
use crate::game_rand::GameRand;
use crate::states::GameplaySet;

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

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(super) struct Routine {
    action: i32,
    direction: i32,
    state: u8,
    initialized: bool,
    delay: i32,
    current_ratio: f32,
    copper: bool,
    active: bool,
    frame: Option<u32>,
}
impl Routine {
    pub(super) fn new(_random: &mut GameRand, copper: bool) -> Self {
        Self {
            action: 11,
            direction: 0,
            state: 1,
            initialized: false,
            delay: 0,
            current_ratio: 0.0,
            copper,
            active: false,
            frame: None,
        }
    }
    fn reset(&mut self, clip: &mut NpcClip) {
        self.action = 11;
        self.direction = 0;
        self.state = 1;
        self.initialized = false;
        self.delay = 0;
        self.current_ratio = 0.0;
        self.active = false;
        self.frame = None;
        clip.start("cKF_ba_r_npc_1_wait1", false, 0.0);
    }
    fn animation(&self, clip: &mut NpcClip) {
        let id =
            taisou::pc_taisou_anim_id(self.action) + taisou::pc_taisou_anim_variant(self.direction);
        if (81..=95).contains(&id) {
            clip.start(CLIPS[(id - 81) as usize], true, 0.0);
        } else if id == 105 {
            clip.start("cKF_ba_r_npc_1_tired1", true, 0.0);
        } else {
            clip.start("cKF_ba_r_npc_1_wait1", false, 0.0);
        }
    }
    fn morph(&mut self, action: i32, clip: &mut NpcClip) {
        self.action = action;
        self.direction = 0;
        self.initialized = true;
        self.current_ratio = 0.0;
        self.animation(clip);
    }
}

#[derive(Resource, Default)]
struct Failing(Option<Entity>);

// This reader runs even while gameplay is paused so a reload cannot retain failure ownership.
fn reset(
    mut loads: MessageReader<crate::sound::ResetAudio>,
    mut failing: ResMut<Failing>,
    mut line: Query<(&mut Routine, &mut NpcClip), With<Line>>,
) {
    if loads.read().count() != 0 {
        failing.0 = None;
        for (mut routine, mut clip) in &mut line {
            routine.reset(&mut clip);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn routine(
    time: Res<Time>,
    frame: Res<bevy::diagnostic::FrameCount>,
    clock: Res<crate::sound::AerobicsClock>,
    mut random: ResMut<GameRand>,
    mut failing: ResMut<Failing>,
    radio: Query<(), With<super::radio_effect::Emitter>>,
    mut line: Query<(Entity, &Role, &mut Routine, &mut NpcClip), With<Line>>,
    mut fraction: Local<f32>,
) {
    if failing.0.is_some_and(|entity| line.get(entity).is_err()) {
        failing.0 = None;
    }
    let counter = clock
        .counter
        .filter(|counter| (0..10).contains(&counter.measure) && !radio.is_empty());
    let Some(counter) = counter else {
        *fraction = 0.0;
        failing.0 = None;
        for (_, _, mut routine, mut clip) in &mut line {
            if routine.active || clip.name != Some("cKF_ba_r_npc_1_wait1") {
                routine.reset(&mut clip);
            }
        }
        return;
    };
    let measure = i32::from(counter.measure);
    let progress = counter.progress;
    if !progress.is_finite() {
        return;
    }
    for (entity, _, mut r, mut clip) in &mut line {
        let skipped = r.frame.is_some_and(|last| frame.0.wrapping_sub(last) > 1);
        r.frame = Some(frame.0);
        if skipped && matches!(r.state, 0 | 2) {
            if failing.0 == Some(entity) {
                failing.0 = None;
            }
            r.action = measure;
            r.direction = (0..5)
                .find(|&part| {
                    !taisou::pc_taisou_over_block(progress - r.current_ratio, measure, part)
                })
                .unwrap_or(0);
            r.initialized = false;
            r.state = 0;
            r.animation(&mut clip);
        }
    }
    *fraction += time.delta_secs() * 60.0;
    let ticks = fraction.floor() as u32;
    *fraction -= ticks as f32;
    for _ in 0..ticks {
        for (entity, role, mut r, mut clip) in &mut line {
            r.active = true;
            let end = taisou::END_RATIO[measure as usize];
            match r.state {
                0 => {
                    if measure == r.action {
                        if r.initialized {
                            r.initialized = false;
                        } else if progress >= end {
                            let next = taisou::pc_taisou_next_action(measure);
                            let place = match role {
                                Role::Villager(n, _) => i32::from(*n),
                                _ => 0,
                            };
                            let rate = taisou::pc_taisou_failure_rate(place);
                            if failing.0.is_none()
                                && next != 0
                                && next != 9
                                && random.0.fqrand() < rate
                            {
                                let mut wrong = 1 + random.0.random(8);
                                while taisou::pc_taisou_failure_reject(wrong, next) {
                                    wrong = 1 + random.0.random(8);
                                }
                                r.state = 2;
                                r.morph(wrong, &mut clip);
                                failing.0 = Some(entity);
                            } else if *role == Role::Leader && random.0.fqrand() < 0.2 {
                                r.state = 4;
                                r.morph(12, &mut clip);
                            } else {
                                r.morph(next, &mut clip);
                                if !r.copper {
                                    r.delay = random.0.random(48);
                                }
                            }
                        }
                    } else if progress >= end {
                        let next = taisou::pc_taisou_next_action(measure);
                        if next != r.action {
                            r.morph(next, &mut clip);
                        }
                    }
                }
                1 => {
                    if progress >= end {
                        r.morph(taisou::pc_taisou_next_action(measure), &mut clip);
                        r.state = 0;
                    }
                }
                2 => {
                    if progress < 0.5 {
                        r.initialized = false;
                        r.state = 3;
                    }
                }
                3 => {
                    if r.action == 10 {
                        if progress >= end {
                            r.morph(taisou::pc_taisou_next_action(measure), &mut clip);
                            failing.0 = None;
                            r.state = 0;
                        } else if clip.done {
                            r.direction = 1;
                            r.animation(&mut clip);
                        }
                    } else if progress >= 0.5 {
                        r.action = 10;
                        r.direction = 0;
                        r.animation(&mut clip);
                    }
                }
                4 => {
                    if progress < 0.5 {
                        r.state = 1;
                    }
                }
                _ => r.reset(&mut clip),
            }
            if r.action < 10 {
                if !r.initialized {
                    let mut adjusted = [0.0; 2];
                    r.delay = taisou::pc_taisou_delay_step(
                        progress,
                        r.delay,
                        r.current_ratio,
                        true,
                        adjusted.as_mut_ptr(),
                    );
                    r.current_ratio = adjusted[0];
                    if taisou::pc_taisou_over_block(adjusted[1], r.action, r.direction) {
                        r.direction += 1;
                        r.animation(&mut clip);
                    }
                    clip.sync_progress = Some(taisou::pc_taisou_sync_frame(
                        0.0,
                        1.0,
                        progress,
                        r.current_ratio,
                        r.action,
                        r.direction,
                    ));
                    clip.morph = 0.0;
                } else {
                    clip.sync_progress = Some(0.0);
                    let delta = progress - end;
                    clip.morph = if delta > 0.0 {
                        (taisou::MORPH_RATIO[measure as usize] / delta).max(1.0)
                    } else {
                        1.0
                    };
                }
            } else {
                clip.sync_progress = None;
            }
        }
    }
}

pub(super) fn register(app: &mut App) {
    app.init_resource::<Failing>()
        .register_type::<Routine>()
        .add_systems(Update, reset.before(routine))
        .add_systems(
            Update,
            routine.in_set(GameplaySet).before(CharacterAnimationSet),
        );
}
