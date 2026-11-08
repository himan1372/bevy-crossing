//! The sports fair's foot race (`ac_tokyoso_control.c`, `ac_tokyoso_npc0.c`, `ac_tokyoso_npc1.c`, `ac_tokyoso_clip.c_inc`).
//!
//! While `mEv_EVENT_SPORTS_FAIR_FOOT_RACE` runs (11:00-12:59 on the equinox days) and the player is in the shrine acre,
//! two runners, one of each team, stretch, get ready, and run four laps (`aTKC_SUBLAP_NUM` = 8 half laps) round the
//! wishing well. A runner keeps to a lane of `160 + RANDOM(20)` units minus 35 for team 1 and runs at
//! `4 + lane / 90 - 0.5 * team` units a tick, one unit slower when ahead (`aTKC_clip_next_run`). The first over the line
//! wins, the other loses (`aTKC_go`), both then stand a while (`aTKC_goal`, 600 ticks) before the next stretch
//! (`aTKC_ACT_STRECH`, 600 ticks, then ready 120). The two spectators (`aTKN1_kansen`) turn to follow their team's runner;
//! the starter (`SP_NPC_EV_TOKYOSO_4`) and Tortimer stand by.
//!
//! Talking (`aTKC_clip_set_talk_request`, `aTKN0_set_norm_talk_info`): `base_msg(looks) + offset + RANDOM(2)` where the
//! offset follows what the actor is doing: warming up `2`, getting ready `4`, running `6`, at the line `10`, a spectator
//! `12`, idle `14`; the starter's is `base_msg + RANDOM(2)`. The bases are `6621 6637 6605 6653 6669 6685` by looks.
//!
//! Not built: the animations (stretch, crouch, banzai, tired), the stumbles (`aTKN1_kokeru`), the relay of the teams'
//! pairs, the sounds and the camera.

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::Facing;
use crate::messages::MsgContext;
use crate::player::Player;
use crate::roster::TownRoster;
use crate::shop::ShopLocation;
use crate::states::GameplaySet;
use crate::town::Town;

/// ALL_NPC_NUM + (SP_NPC_EV_TOKYOSO_0 - SP_NPC_START).
const TOKYOSO_NPC: u32 = 238 + 73;
const SOUND: u8 = 2;
/// Game units per cell.
const UNIT: f32 = 40.0;
/// `base_msg_table` by looks.
const BASE_MSG: [u16; 6] = [6621, 6637, 6605, 6653, 6669, 6685];
/// `aTKC_SUBLAP_NUM`.
const SUBLAPS: u32 = 8;
/// The stretch, ready and goal pauses in seconds (600, 120 and 600 ticks).
const STRETCH: f32 = 10.0;
const READY: f32 = 2.0;
const GOAL: f32 = 10.0;
/// Units (x, z) of the five actors (`l_fdd_tokyoso_map`): runners 0 and 1, spectators 0 and 1, the starter.
const UNITS: [(usize, usize); 5] = [(13, 3), (11, 7), (12, 7), (14, 7), (14, 8)];
/// Where the stretch spots lie, relative to the well in units (`aTKC_clip_next_warmup`).
const WARMUP_X: [f32; 2] = [200.0, 160.0];
pub(crate) const TORTIMER_UNIT: (usize, usize) = (5, 8);

#[derive(Component)]
struct Racer {
    day: i64,
    role: Role,
    looks: u8,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Role {
    /// A runner of team 0 or 1.
    Runner(usize),
    /// A spectator of team 0 or 1.
    Spectator(usize),
    Starter,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Phase {
    /// The runners stretch near the start.
    #[default]
    Stretch,
    Ready,
    Run,
    /// The race is over: the winner and loser stand at the line.
    Goal,
}

#[derive(Resource, Default)]
struct Race {
    day: i64,
    phase: Phase,
    timer: f32,
    /// Per team: the angle on the circuit (radians, counter-clockwise from east), the half laps done, the lane
    /// in cells, the speed in cells a second, and the result (0 none, 1 won, 2 lost).
    angle: [f32; 2],
    sublaps: [u32; 2],
    lane: [f32; 2],
    speed: [f32; 2],
    result: [u8; 2],
    /// A runner's tumble (`aTKN1_kokeru` then `aTKN1_getup`): 0 none, 1 falling, 2 getting up.
    fall: [u8; 2],
}

fn well(town: &Town) -> Option<Vec3> {
    town.retail
        .as_ref()?
        .buildings
        .iter()
        .find(|b| b.item == 0x5825)
        .map(|b| b.origin)
}

/// Brings the five to the shrine for the race.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    roster: Option<Res<TownRoster>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Racer)>,
    mut race: ResMut<Race>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let on = today.is_active(ev::SPORTS_FAIR_FOOT_RACE, now.hour);
    let out =
        on && *state.get() == ShopLocation::Town && *house.get() == crate::house::HouseState::Town;
    for (entity, r) in &existing {
        if !out || r.day != now.day_number {
            commands.entity(entity).despawn();
        }
    }
    if race.day != now.day_number {
        *race = Race {
            day: now.day_number,
            ..default()
        };
    }
    if !out || !existing.is_empty() {
        return;
    }
    let Some(roster) = roster else { return };
    let roles = [
        Role::Runner(0),
        Role::Runner(1),
        Role::Spectator(0),
        Role::Spectator(1),
        Role::Starter,
    ];
    for (i, (role, unit)) in roles.into_iter().zip(UNITS).enumerate() {
        let Some(at) = crate::tortimer::shrine_cell(&town, unit.0, unit.1) else {
            return;
        };
        let looks = roster.residents.get(i).map_or(0, |e| e.looks);
        let name = roster
            .residents
            .get(i)
            .map_or("Runner".to_string(), |e| e.name.clone());
        commands.spawn((
            Name::new(format!("{name} (foot race)")),
            Racer {
                day: now.day_number,
                role,
                looks,
            },
            NpcClip::default(),
            NpcGait::default(),
            crate::event_body::EventBody::default(),
            // The villager's own model (the event slots' models carry no villager texture).
            NpcModel(
                roster
                    .residents
                    .get(i)
                    .map_or(TOKYOSO_NPC + i as u32, |e| e.index),
            ),
            Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
            Transform::from_translation(at + Vec3::Y),
        ));
    }
    info!("The foot race stands ready at the shrine");
}

/// `aTKC_clip_next_run`: a runner's lane and speed.
fn lanes(race: &mut Race, rand: &mut crate::game_rand::GameRand) {
    for team in 0..2 {
        let f = 160.0 + rand.0.fqrand() * 20.0 - team as f32 * 35.0;
        race.lane[team] = f / UNIT;
        // Units a tick to cells a second.
        race.speed[team] = (4.0 + f / 90.0 - team as f32 * 0.5) * 60.0 / UNIT;
    }
}

/// The race itself (`aTKC_*`): the phases, the laps and the result.
#[allow(clippy::too_many_arguments)]
fn run(
    time: Res<Time>,
    town: Res<Town>,
    today: Res<TodayEvents>,
    mut race: ResMut<Race>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    player: Query<&Transform, (With<Player>, Without<Racer>)>,
    mut racers: Query<(&Racer, &mut Transform, &mut NpcClip)>,
) {
    let now = crate::clock::now();
    if !today.is_active(ev::SPORTS_FAIR_FOOT_RACE, now.hour) {
        return;
    }
    let (Some(centre), Ok(player)) = (well(&town), player.single()) else {
        return;
    };
    // The show goes on while the player is in the shrine acre.
    let in_acre = Town::cell_indices_of(player.translation)
        .zip(Town::cell_indices_of(centre))
        .is_some_and(|(a, b)| (a.0, a.1) == (b.0, b.1));
    if !in_acre {
        race.phase = Phase::Stretch;
        race.timer = STRETCH;
        return;
    }
    let dt = time.delta_secs();
    race.timer -= dt;
    match race.phase {
        Phase::Stretch => {
            if race.timer <= 0.0 {
                race.phase = Phase::Ready;
                race.timer = READY;
                info!("Foot race: ready");
            }
        }
        Phase::Ready => {
            if race.timer <= 0.0 {
                race.phase = Phase::Run;
                race.angle = [0.0; 2];
                race.sublaps = [0; 2];
                race.result = [0; 2];
                race.fall = [0; 2];
                lanes(&mut race, &mut rand);
                info!("Foot race: go!");
            }
        }
        Phase::Run => {
            for team in 0..2 {
                if race.result[team] != 0 || race.fall[team] != 0 {
                    continue;
                }
                // The leader slows by one unit a tick (`aTKC_clip_top_check`).
                let ahead = race.sublaps[team] > race.sublaps[1 - team]
                    || (race.sublaps[team] == race.sublaps[1 - team]
                        && race.angle[team] % std::f32::consts::PI
                            > race.angle[1 - team] % std::f32::consts::PI);
                let speed = race.speed[team] - if ahead { 60.0 / UNIT } else { 0.0 };
                race.angle[team] += speed / race.lane[team] * dt;
                let before = race.sublaps[team];
                race.sublaps[team] = (race.angle[team] / std::f32::consts::PI) as u32;
                // At a turn the leader trips one time in ten (`aTKN1_run`: `top_check` and `RANDOM_F(1.0) < 0.1`).
                if race.sublaps[team] > before
                    && race.sublaps[team] < SUBLAPS
                    && ahead
                    && rand.0.fqrand() < 0.1
                {
                    race.fall[team] = 1;
                    info!("Foot race: team {team} falls");
                }
                if race.sublaps[team] >= SUBLAPS {
                    race.result[team] = if race.result[1 - team] == 0 { 1 } else { 2 };
                    info!(
                        "Foot race: team {team} is over the line ({})",
                        race.result[team]
                    );
                }
            }
            if race.result.iter().all(|&r| r != 0) {
                race.phase = Phase::Goal;
                race.timer = GOAL;
            }
        }
        Phase::Goal => {
            if race.timer <= 0.0 {
                race.phase = Phase::Stretch;
                race.timer = STRETCH;
                info!("Foot race: the next stretch");
            }
        }
    }
    // Where everyone stands.
    for (r, mut at, mut clip) in &mut racers {
        // The clip the actor plays: (name, held after one run); none leaves the stride to the gait.
        let want: Option<(&'static str, bool)> = match r.role {
            Role::Runner(team) => match race.phase {
                Phase::Stretch => Some(("cKF_ba_r_npc_1_warmup1", false)),
                Phase::Ready => Some(("cKF_ba_r_npc_1_ready1", true)),
                Phase::Run => match race.fall[team] {
                    1 => Some(("cKF_ba_r_npc_1_kokeru1", true)),
                    2 => Some(("cKF_ba_r_npc_1_kokeru_getup1", true)),
                    _ => None,
                },
                Phase::Goal => Some(if race.result[team] == 1 {
                    ("cKF_ba_r_npc_1_banzai1", false)
                } else {
                    ("cKF_ba_r_npc_1_tired1", false)
                }),
            },
            Role::Spectator(team) => (race.phase == Phase::Goal && race.result[team] == 1)
                .then_some(("cKF_ba_r_npc_1_clap1", false)),
            Role::Starter => match race.phase {
                Phase::Stretch => Some(("cKF_ba_r_npc_1_tamakome1", true)),
                Phase::Ready => Some(("cKF_ba_r_npc_1_youi1", true)),
                Phase::Run => Some(("cKF_ba_r_npc_1_don1", true)),
                Phase::Goal => None,
            },
        };
        // A tumble runs its two clips in turn.
        if let Role::Runner(team) = r.role {
            if clip.done && race.fall[team] != 0 && race.phase == Phase::Run {
                race.fall[team] = if race.fall[team] == 1 { 2 } else { 0 };
            }
        }
        if clip.name != want.map(|w| w.0) {
            match want {
                Some((name, once)) => clip.start(name, once, 8.0),
                None => clip.stop(8.0),
            }
        }
        let ground = |p: Vec3| p.with_y(town.ground_y(p) + 1.0);
        match r.role {
            Role::Runner(team) => {
                let (angle, lane) = match race.phase {
                    Phase::Run | Phase::Goal => (race.angle[team], race.lane[team]),
                    _ => (0.0, WARMUP_X[team] / UNIT),
                };
                let angle = if race.phase == Phase::Goal {
                    SUBLAPS as f32 * std::f32::consts::PI
                } else {
                    angle
                };
                let p = centre + Vec3::new(angle.cos() * lane, 0.0, -angle.sin() * lane);
                at.translation = ground(p);
                // Along the circuit while running, toward the well otherwise.
                let tangent = Vec3::new(-angle.sin(), 0.0, -angle.cos());
                let face = if race.phase == Phase::Run {
                    tangent
                } else {
                    -Vec3::new(angle.cos(), 0.0, -angle.sin())
                };
                at.rotation = Quat::from_rotation_y(face.x.atan2(face.z));
            }
            Role::Spectator(team) => {
                // They follow their team's runner (`aTKN1_kansen`).
                let angle = if race.phase == Phase::Run {
                    race.angle[team]
                } else {
                    0.0
                };
                let lane = if race.phase == Phase::Run {
                    race.lane[team]
                } else {
                    WARMUP_X[team] / UNIT
                };
                let runner = centre + Vec3::new(angle.cos() * lane, 0.0, -angle.sin() * lane);
                let to = (runner - at.translation).with_y(0.0);
                if to.length() > 0.01 {
                    at.rotation = Quat::from_rotation_y(to.x.atan2(to.z));
                }
            }
            Role::Starter => {}
        }
    }
}

fn context(identity: &crate::save::PlayerIdentity, name: &str) -> MsgContext {
    let now = crate::clock::now();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: name.into(),
        free: vec![String::new(); 20],
        items: vec![String::new(); 5],
        item_articles: vec![0; 5],
        town_name: identity.town.clone(),
        year: now.year,
        month: now.month,
        day: now.day,
        hour: now.hour,
        minute: now.minute,
        weekday: now.weekday,
        ..default()
    }
}

/// The talks.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    race: Res<Race>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), (With<Player>, Without<Racer>)>,
    racers: Query<(Entity, &Transform, &Racer)>,
    models: Query<&NpcModel>,
    mut dialogue: ResMut<DialogueState>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    // A line of theirs ends when it is read.
    if dialogue.active
        && racers.iter().any(|(e, ..)| dialogue.speaker == Some(e))
        && dialogue.external_result.take().is_some()
    {
        *dialogue = DialogueState::default();
        *focus = InputFocus::World;
        return;
    }
    if !focus.is_world() || !actions.just_pressed(Action::Interact) {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    let Some((actor, _, r)) = racers.iter().find(|(_, t, _)| {
        let delta = (t.translation - at.translation).with_y(0.0);
        delta.length() < 1.8 && facing.dir.dot(delta.normalize_or_zero()) >= 0.5
    }) else {
        return;
    };
    actions.consume(Action::Interact);
    let base = BASE_MSG[usize::from(r.looks).min(5)];
    // The offset by what they are doing.
    let offset = match r.role {
        Role::Starter => 0,
        Role::Spectator(_) => 12,
        Role::Runner(_) => match race.phase {
            Phase::Stretch => {
                if rand.0.fqrand() < 0.5 {
                    2
                } else {
                    14
                }
            }
            Phase::Ready => 4,
            Phase::Run => 6,
            Phase::Goal => 10,
        },
    };
    let msg = base + offset + rand.0.random(2) as u16;
    let mut ctx = context(&identity, "Villager");
    ctx.tail = crate::dialogue::catchphrase(&disc, models.get(actor).map_or(u32::MAX, |m| m.0));
    let Some(text) = disc.text.as_ref() else {
        return;
    };
    let (pages, asked) = text.talk_asking(usize::from(msg), &ctx, &mut |_| 0);
    *focus = InputFocus::Dialogue;
    dialogue.begin_special(
        actor,
        "Villager",
        (TOKYOSO_NPC, SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Foot race talk {msg}");
}

pub struct FootRacePlugin;

impl Plugin for FootRacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Race>()
            .add_systems(
                Update,
                visit.after(crate::event_today::refresh).in_set(GameplaySet),
            )
            .add_systems(Update, run.after(visit).in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
