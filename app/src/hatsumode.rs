//! New Year's first shrine visit (`ac_hatumode_control.c`, `ac_hatumode_npc0.c`, `ac_hatumode_clip_move.c_inc`,
//! `ac_hatumode_clip_think.c_inc`, `ac_hatumode_npc0_schedule.c_inc`).
//!
//! While `mEv_EVENT_NEW_YEARS_DAY` runs (January 1, 06:00-09:59) five villagers (`SP_NPC_EV_HATUMODE_0..4`) come to the
//! shrine at units (7,9) (6,10) (9,10) (6,12) (9,12) moved by `(20,-10) (40,40) (-40,40) (40,0) (-40,0)` units
//! (`aHTMD_clip_set_start`), facing north, and walk the original's procession.
//!
//! Each villager runs the 46-entry think table (`dt_tbl`): a proc (`aHN0_THINK_PROC_*`), an init (`aHN0_THINK_INIT_*`),
//! whether the player may talk to him and with what (`talk_req`, `talk_idx`) and the think that follows a talk. The
//! procs wait on the shared flags (`flags0`: `0x1000` / `0x2000` the greeting handshake, `0x4000` the one who prayed,
//! `0x8000` the player's turn at the shrine; `flags1`: pair bits by `npc_id & 1`), on turning, on running to a point, on
//! a timer, or on an animation. The points are `yashiro_pos` + the nine of `aHN0_posX/Z`, taken in turns by
//! `aHTMD_next_position` round two loops of roots that both pass the front of the shrine.
//!
//! The player: an invitation (think 3 and 6, `base_msg`, Yes / No) with Yes (`base_msg + 1`) sends the villager to think
//! 34: the player is walked (demo walk, 2 units a tick) to (20,120) behind the queue, the villager steps on to the next
//! point, the player is walked to the front (20,85) and `flags0 |= 0x8000` hands over to the shrine: `4396` (50 Bells;
//! `4398` without them), yes `0x112D` and the coin (`aHTC_pay`, `aHTC_saisen`), no `0x112E`; the prayer ends it and the
//! villager goes on (`0x4000`, think 10). Chats at think 21 and 28 are `base_msg + 3 / 6 / 9 / 12 + RANDOM(3)` by the
//! villager's place after the one at the front (`aHN0_make_msg`). The bases are `7679 7697 7661 7715 7733 7751` by looks.
//!
//! Not built: the real animations (throwing, praying, greeting, bowing are timed pauses), the shrine's bell, the red
//! message window, collision rerouting (`aHN0_THINK_PROC_MOVE_BF`), the New Year table, the player's own pray and
//! throw-money animations.

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::messages::MsgContext;
use crate::player::Player;
use crate::roster::TownRoster;
use crate::shop::ShopLocation;
use crate::states::GameplaySet;
use crate::town::Town;

const SOUND: u8 = 2;
const UNIT: f32 = 40.0;
const BASE_MSG: [u16; 6] = [7679, 7697, 7661, 7715, 7733, 7751];
/// Units and start nudges of the five villagers.
const UNITS: [(usize, usize); 5] = [(7, 9), (6, 10), (9, 10), (6, 12), (9, 12)];
const NUDGE: [(f32, f32); 5] = [
    (20.0, -10.0),
    (40.0, 40.0),
    (-40.0, 40.0),
    (40.0, 0.0),
    (-40.0, 0.0),
];
/// `aHN0_posX/Z` of `aHTMD_next_position` and its `aHN0_root`.
const POINT_X: [f32; 9] = [20.0, 0.0, 40.0, 0.0, 40.0, -60.0, -60.0, 100.0, 100.0];
const POINT_Z: [f32; 9] = [75.0, 160.0, 160.0, 200.0, 200.0, 75.0, 200.0, 75.0, 200.0];
const ROOT: [usize; 10] = [0, 5, 6, 3, 1, 0, 7, 8, 4, 2];
/// `aHTMD_clip_player_move`: where the player is walked, from the shrine, in units.
const WALK: [(f32, f32); 4] = [(20.0, 120.0), (20.0, 85.0), (-80.0, 85.0), (120.0, 85.0)];
/// `aHTMD_clip_set_start`'s roots and `aHN0_ready2`'s roots and first thinks.
const START_ROOT: [u8; 5] = [0, 4, 9, 3, 8];
const READY_ROOT: [u8; 5] = [5, 4, 9, 3, 8];
const READY_THINK: [usize; 5] = [4, 27, 27, 20, 27];
const PRICE: u32 = 50;
/// Run speed of the villagers, cells a second.
const RUN: f32 = 3.5;
/// The player's demo walk, 2 units a tick.
const ESCORT: f32 = 2.0 * 60.0 / UNIT;
/// Animation lengths, seconds (a stand-in for the clips).
const THROW: f32 = 2.6;
const PRAY: f32 = 3.2;
const GREET: f32 = 1.6;
/// `aHN0_THINK_INIT_OMAIRI_AF`: 120 ticks.
const AFTER_PRAYER: f32 = 2.0;
/// The player's own prayer, seconds.
const PLAYER_PRAYER: f32 = 3.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Proc {
    Ready1,
    Ready2,
    TurnNext,
    Flag2Wait,
    AnimeWait,
    AnimeWaitSaisen,
    TimerWait,
    MoveWait,
    MoveBf,
    AisatuWait,
    Aisatu2Wait,
    ToOmairi,
    PlayerMove,
    PlayerWait,
    Kasasimai,
    SanpaiWait,
    Nothing,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Init {
    Wait,
    Move,
    TurnF,
    Saisen,
    OmairiAf,
    NextPos,
    TurnMove,
    TurnAisatu,
    Aisatu,
    Aisatu2,
    Haigo,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Req {
    None,
    Norm,
}

#[derive(Clone, Copy)]
struct Think {
    proc: Proc,
    init: Init,
    req: Req,
    talk: u8,
    next: u8,
}

const fn th(proc: Proc, init: Init, req: Req, talk: u8, next: u8) -> Think {
    Think {
        proc,
        init,
        req,
        talk,
        next,
    }
}

/// `dt_tbl` of `ac_hatumode_clip_think.c_inc`.
const THINK: [Think; 46] = {
    use Init as I;
    use Proc as P;
    use Req::{None as N, Norm};
    [
        th(P::Ready1, I::Wait, N, 0, 0),
        th(P::Ready2, I::Wait, N, 0, 1),
        th(P::TurnNext, I::TurnF, N, 0, 2),
        th(P::Flag2Wait, I::Wait, Norm, 0, 2),
        th(P::AnimeWaitSaisen, I::Saisen, N, 0, 4),
        th(P::AnimeWait, I::Saisen, N, 0, 5),
        th(P::TimerWait, I::OmairiAf, Norm, 1, 7),
        th(P::TimerWait, I::NextPos, N, 0, 7),
        th(P::TurnNext, I::TurnMove, N, 0, 8),
        th(P::MoveWait, I::Move, N, 0, 8),
        th(P::TimerWait, I::NextPos, N, 0, 10),
        th(P::TurnNext, I::TurnMove, N, 0, 11),
        th(P::MoveWait, I::Move, N, 0, 12),
        th(P::Flag2Wait, I::Wait, N, 0, 13),
        th(P::TimerWait, I::NextPos, N, 0, 14),
        th(P::TurnNext, I::TurnMove, N, 0, 15),
        th(P::MoveWait, I::Move, N, 0, 15),
        th(P::TurnNext, I::TurnAisatu, N, 0, 17),
        th(P::AisatuWait, I::Wait, N, 0, 18),
        th(P::AnimeWait, I::Aisatu, N, 0, 19),
        th(P::TurnNext, I::TurnF, N, 0, 20),
        th(P::Aisatu2Wait, I::Wait, Norm, 2, 20),
        th(P::TurnNext, I::TurnAisatu, N, 0, 22),
        th(P::AnimeWait, I::Aisatu2, N, 0, 23),
        th(P::TimerWait, I::NextPos, N, 0, 24),
        th(P::TurnNext, I::TurnMove, N, 0, 25),
        th(P::MoveWait, I::Move, N, 0, 25),
        th(P::TurnNext, I::TurnF, N, 0, 27),
        th(P::Flag2Wait, I::Wait, Norm, 3, 27),
        th(P::TimerWait, I::NextPos, N, 0, 29),
        th(P::TurnNext, I::TurnMove, N, 0, 30),
        th(P::MoveWait, I::Move, N, 0, 30),
        th(P::ToOmairi, I::Wait, N, 0, 32),
        th(P::MoveBf, I::Wait, N, 0, 33),
        th(P::PlayerMove, I::Wait, N, 0, 34),
        th(P::PlayerWait, I::Wait, N, 0, 35),
        th(P::TimerWait, I::NextPos, N, 0, 36),
        th(P::TurnNext, I::TurnMove, N, 0, 37),
        th(P::MoveWait, I::Move, N, 0, 38),
        th(P::PlayerMove, I::Wait, N, 0, 39),
        th(P::PlayerWait, I::Wait, N, 0, 40),
        th(P::Kasasimai, I::Wait, N, 0, 41),
        th(P::SanpaiWait, I::Wait, N, 0, 42),
        th(P::TurnNext, I::Haigo, N, 0, 43),
        th(P::MoveWait, I::Move, N, 0, 44),
        th(P::Nothing, I::Wait, N, 0, 29),
    ]
};

/// What a villager is doing (`action.idx`).
#[derive(Clone, Copy, Debug)]
enum Act {
    Wait,
    Turn(f32),
    Run(Vec3),
    Anim(f32),
}

#[derive(Component)]
struct Walker {
    day: i64,
    id: usize,
    looks: u8,
    index: u32,
    think: usize,
    proc: Proc,
    talk_idx: u8,
    next_think: usize,
    req: Req,
    root: u8,
    /// The point to go to, units from the shrine.
    pos: Vec2,
    timer: f32,
    act: Act,
    finished: bool,
    /// Seconds into the running animation, and whether its coin has gone.
    anim_t: f32,
    coin: bool,
    /// How far the player's walk has gone (`player`).
    player: usize,
    /// Seconds the run has been held up by a body or the ground.
    blocked: f32,
    /// The disc clip the action plays (`aNPC_ANIM_*`).
    want: Option<&'static str>,
}

/// `aEv_hatumode_save_c`.
#[derive(Resource, Default)]
struct Hatsu {
    state: u8,
    flags0: u16,
    flags1: u16,
    cur_npc_idx: usize,
}

/// The player's demo walk.
#[derive(Resource, Default)]
struct Escort {
    to: Option<Vec3>,
}

#[derive(Default)]
enum Phase {
    #[default]
    Idle,
    /// The invitation is up (the villager's entity).
    Asked(Option<Entity>),
    /// A closing line is up; the villager goes on at his next think.
    Line(Option<Entity>),
    /// The villager's `base + 1` is up: the player's walk follows.
    Joining(Option<Entity>),
    /// The shrine's question is up.
    Prompt,
    /// `0x112D` or the no-money line is up; the prayer follows (`true` when the coin goes).
    Said(bool),
    /// Praying: seconds left.
    Praying(f32),
}

#[derive(Resource, Default)]
struct Visit {
    phase: Phase,
}

fn well(town: &Town) -> Option<Vec3> {
    town.retail
        .as_ref()?
        .buildings
        .iter()
        .find(|b| b.item == 0x5825)
        .map(|b| b.origin)
}

/// A point `(x, z)` units from the shrine, in the world.
fn world(town: &Town, centre: Vec3, p: Vec2) -> Vec3 {
    let at = centre + Vec3::new(p.x, 0.0, p.y) / UNIT;
    at.with_y(town.ground_y(at) + 1.0)
}

fn yaw_to(from: Vec3, to: Vec3) -> f32 {
    let d = to - from;
    d.x.atan2(d.z)
}

fn yaw_of(t: &Transform) -> f32 {
    t.rotation.to_euler(EulerRot::YXZ).0
}

/// `aHTMD_next_position`.
fn next_position(root: u8) -> (u8, Vec2) {
    let mut root = root + 1;
    if root == 10 {
        root = 5;
    } else if root == 5 {
        root = 0;
    }
    let p = ROOT[usize::from(root)];
    (root, Vec2::new(POINT_X[p], POINT_Z[p]))
}

/// `aHN0_setup_think_proc` with the init of the think.
fn setup(w: &mut Walker, at: &Transform, hs: &mut Hatsu, town: &Town, centre: Vec3, think: usize) {
    let d = THINK[think.min(THINK.len() - 1)];
    w.think = think;
    w.proc = d.proc;
    w.talk_idx = d.talk;
    w.next_think = usize::from(d.next);
    w.req = d.req;
    if d.req == Req::Norm {
        info!("Villager {} can be talked to at think {think}", w.id);
    }
    w.finished = false;
    w.anim_t = 0.0;
    w.coin = false;
    let parity = (w.id & 1) as u32;
    w.act = Act::Wait;
    w.want = None;
    match d.init {
        Init::Wait => {}
        Init::Move => w.act = Act::Run(world(town, centre, w.pos)),
        Init::TurnF => w.act = Act::Turn(std::f32::consts::PI),
        Init::Saisen => {
            w.act = Act::Anim(if think == 4 { THROW } else { PRAY });
            w.want = Some(if think == 4 {
                "cKF_ba_r_npc_1_saisen1"
            } else {
                "cKF_ba_r_npc_1_omairi1"
            });
        }
        Init::OmairiAf => w.timer = AFTER_PRAYER,
        Init::NextPos => {
            let (root, pos) = next_position(w.root);
            w.root = root;
            w.pos = pos;
            w.timer = 0.0;
            match think {
                10 => hs.flags1 |= 0x10 << parity,
                29 => hs.flags1 |= 0x04 << parity,
                24 => hs.flags1 |= 0x01 << parity,
                _ => {}
            }
        }
        Init::TurnMove => {
            w.act = Act::Turn(yaw_to(at.translation, world(town, centre, w.pos)));
        }
        Init::TurnAisatu => {
            let side = if parity == 1 { 1.0 } else { -1.0 };
            w.act = Act::Turn(side * std::f32::consts::FRAC_PI_2);
            hs.flags0 |= 0x1000;
        }
        Init::Aisatu => {
            w.act = Act::Anim(GREET);
            w.want = Some(greeting(w.looks));
            hs.flags0 &= !(0x2000 | 0x1000);
        }
        Init::Aisatu2 => {
            w.act = Act::Anim(GREET);
            w.want = Some(greeting(w.looks));
            hs.flags0 |= 0x2000;
        }
        Init::Haigo => {
            w.pos = Vec2::new(20.0, 115.0);
            w.act = Act::Turn(yaw_to(at.translation, world(town, centre, w.pos)));
        }
    }
}

/// `aHN0_aisatu_local`: the greeting by looks.
fn greeting(looks: u8) -> &'static str {
    match looks {
        1 | 3 => "cKF_ba_r_npc_1_aisatu1",
        0 | 2 => "cKF_ba_r_npc_1_aisatu2",
        4 => "cKF_ba_r_npc_1_aisatu3",
        _ => "cKF_ba_r_npc_1_aisatu4",
    }
}

/// `aHN0_flag2_wait_check`.
fn flag_bit(w: &Walker) -> u16 {
    let parity = (w.id & 1) as u32;
    match w.think {
        3 => 0x01 << parity,
        13 => 0x04 << (parity ^ 1),
        28 => 0x10 << (parity ^ 1),
        _ => 0,
    }
}

/// Brings the five villagers to the shrine.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    roster: Option<Res<TownRoster>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Walker)>,
    mut hs: ResMut<Hatsu>,
    mut escort: ResMut<Escort>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let on = today.is_active(ev::NEW_YEARS_DAY, now.hour);
    let out =
        on && *state.get() == ShopLocation::Town && *house.get() == crate::house::HouseState::Town;
    for (entity, v) in &existing {
        if !out || v.day != now.day_number {
            commands.entity(entity).despawn();
        }
    }
    if !out {
        *hs = Hatsu::default();
        escort.to = None;
        return;
    }
    if !existing.is_empty() {
        return;
    }
    let Some(roster) = roster else { return };
    // `aEv_init_hatumode_save_area`.
    *hs = Hatsu::default();
    escort.to = None;
    for (i, entry) in roster.residents.iter().take(5).enumerate() {
        let Some(at) = crate::tortimer::shrine_cell(&town, UNITS[i].0, UNITS[i].1) else {
            continue;
        };
        let at = at + Vec3::new(NUDGE[i].0, 0.0, NUDGE[i].1) / UNIT;
        let first = if i == 0 { 0 } else { 1 };
        let d = THINK[first];
        commands.spawn((
            Name::new(format!("{} (hatsumode)", entry.name)),
            Walker {
                day: now.day_number,
                id: i,
                looks: entry.looks,
                index: entry.index,
                think: first,
                proc: d.proc,
                talk_idx: 0,
                next_think: usize::from(d.next),
                req: Req::None,
                root: START_ROOT[i],
                pos: Vec2::ZERO,
                timer: 0.0,
                act: Act::Wait,
                finished: false,
                anim_t: 0.0,
                coin: false,
                player: 0,
                blocked: 0.0,
                want: None,
            },
            NpcClip::default(),
            NpcGait::default(),
            crate::event_body::EventBody::default(),
            NpcModel(entry.index),
            Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
            // Facing north, toward the shrine (`DEG2SHORT_ANGLE2(180)`).
            Transform::from_translation(at + Vec3::Y)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        ));
    }
    info!("The New Year visitors come to the shrine");
}

/// Each villager's think and action (`aHN0_think_main_proc`).
#[allow(clippy::too_many_arguments)]
fn walkers(
    mut commands: Commands,
    time: Res<Time>,
    town: Res<Town>,
    dialogue: Res<DialogueState>,
    mut hs: ResMut<Hatsu>,
    mut escort: ResMut<Escort>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut walkers: Query<(Entity, &mut Walker, &mut Transform, &mut NpcClip)>,
    player: Query<&Transform, (With<Player>, Without<Walker>)>,
) {
    let Some(centre) = well(&town) else { return };
    let dt = time.delta_secs();
    let player_at = player.single().ok().map(|t| t.translation);
    // Where the others stand, for the bodies.
    let bodies: Vec<(Entity, Vec3)> = walkers
        .iter()
        .map(|(e, _, t, _)| (e, t.translation))
        .collect();
    for (entity, mut w, mut at, mut clip) in &mut walkers {
        // The action's clip.
        match (w.want, clip.name) {
            (Some(name), cur) if cur != Some(name) => clip.start(name, true, 8.0),
            (None, Some(_)) => clip.stop(8.0),
            _ => {}
        }
        // A talk holds him (`mDemo_Check(TALK)`).
        if dialogue.active && dialogue.speaker == Some(entity) {
            continue;
        }
        // The action.
        match w.act {
            Act::Wait => {}
            Act::Turn(to) => {
                let cur = yaw_of(&at);
                let diff = (to - cur + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                    - std::f32::consts::PI;
                let step = 9.0 * dt;
                if diff.abs() <= step {
                    at.rotation = Quat::from_rotation_y(to);
                    w.finished = true;
                } else {
                    at.rotation = Quat::from_rotation_y(cur + step * diff.signum());
                }
            }
            Act::Run(to) => {
                let d = (to - at.translation).with_y(0.0);
                let step = RUN * dt;
                if d.length() <= step {
                    at.translation.x = to.x;
                    at.translation.z = to.z;
                    w.finished = true;
                } else {
                    let dir = d.normalize();
                    let next = at.translation + dir * step;
                    // The ground and the bodies (`aNPC_BG_CHECK`, the collision pipes); held up too long he goes on.
                    let free = (town.can_step(at.translation, next)
                        && crate::event_body::step_clear(
                            at.translation,
                            next,
                            bodies.iter().filter(|(e, _)| *e != entity).map(|(_, p)| *p),
                            player_at,
                        ))
                        || w.blocked > 1.5;
                    at.rotation = Quat::from_rotation_y(dir.x.atan2(dir.z));
                    if free {
                        w.blocked = if w.blocked > 1.5 { 0.0 } else { w.blocked };
                        at.translation = next;
                        at.translation.y = town.ground_y(at.translation) + 1.0;
                    } else {
                        w.blocked += dt;
                    }
                }
            }
            Act::Anim(left) => {
                w.anim_t += dt;
                // The coin leaves the hand at frame 33 of the throw.
                if w.think == 4 && !w.coin && w.anim_t >= 1.1 {
                    w.coin = true;
                    let forward = at.rotation * Vec3::Z;
                    let mesh = meshes.add(Sphere::new(0.12));
                    let material = materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 0.85, 0.2),
                        emissive: LinearRgba::new(2.0, 1.6, 0.3, 1.0),
                        unlit: true,
                        ..default()
                    });
                    commands.spawn((
                        crate::fireworks::Spark {
                            vel: forward * 2.0 + Vec3::Y * 3.5,
                            age: 0.0,
                        },
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        Transform::from_translation(at.translation + Vec3::Y * 0.6),
                    ));
                }
                if w.anim_t >= left {
                    w.finished = true;
                }
            }
        }
        // The think runs when the action is done or he waits.
        if !(matches!(w.act, Act::Wait) || w.finished) {
            continue;
        }
        let think = w.think;
        match w.proc {
            Proc::Nothing | Proc::MoveBf => {}
            Proc::Ready1 => {
                // `aHN0_ready1`.
                hs.state = 1;
                hs.cur_npc_idx = 0;
                w.root = 5;
                setup(&mut w, &at, &mut hs, &town, centre, 4);
            }
            Proc::Ready2 => {
                if hs.state == 1 {
                    let id = w.id;
                    w.root = READY_ROOT[id];
                    setup(&mut w, &at, &mut hs, &town, centre, READY_THINK[id]);
                }
            }
            Proc::TurnNext => {
                if matches!(w.act, Act::Turn(_)) && w.finished {
                    setup(&mut w, &at, &mut hs, &town, centre, think + 1);
                }
            }
            Proc::Flag2Wait => {
                let bit = flag_bit(&w);
                if hs.flags1 & bit != 0 {
                    hs.flags1 &= !bit;
                    if think == 28 && hs.flags0 & 0x4000 != 0 {
                        hs.flags0 &= !0x4000;
                        setup(&mut w, &at, &mut hs, &town, centre, 43);
                    } else {
                        setup(&mut w, &at, &mut hs, &town, centre, think + 1);
                    }
                }
            }
            Proc::AnimeWait | Proc::AnimeWaitSaisen => {
                if matches!(w.act, Act::Anim(_)) && w.finished {
                    setup(&mut w, &at, &mut hs, &town, centre, think + 1);
                }
            }
            Proc::TimerWait => {
                if w.timer > 0.0 {
                    w.timer -= dt;
                } else {
                    setup(&mut w, &at, &mut hs, &town, centre, think + 1);
                }
            }
            Proc::MoveWait => {
                if matches!(w.act, Act::Run(_)) && w.finished {
                    setup(&mut w, &at, &mut hs, &town, centre, think + 1);
                }
            }
            Proc::AisatuWait => {
                if hs.flags0 & 0x2000 != 0 {
                    setup(&mut w, &at, &mut hs, &town, centre, think + 1);
                }
            }
            Proc::Aisatu2Wait => {
                if hs.flags0 & 0x1000 != 0 {
                    setup(&mut w, &at, &mut hs, &town, centre, think + 1);
                }
            }
            Proc::ToOmairi => {
                hs.cur_npc_idx = w.id;
                setup(&mut w, &at, &mut hs, &town, centre, 2);
            }
            Proc::PlayerMove => {
                // `aHTMD_clip_player_move`: the walk starts and the villager goes on.
                let (x, z) = WALK[w.player.min(3)];
                escort.to = Some(world(&town, centre, Vec2::new(x, z)));
                w.player += 1;
                setup(&mut w, &at, &mut hs, &town, centre, think + 1);
            }
            Proc::PlayerWait => {
                // `aHTMD_clip_look_player`, until the walk is over.
                if escort.to.is_none() {
                    setup(&mut w, &at, &mut hs, &town, centre, think + 1);
                }
            }
            Proc::Kasasimai => {
                if escort.to.is_none() {
                    hs.flags0 |= 0x8000;
                    setup(&mut w, &at, &mut hs, &town, centre, think + 1);
                }
            }
            Proc::SanpaiWait => {
                if hs.flags0 & 0x8000 == 0 {
                    hs.flags0 |= 0x4000;
                    hs.flags1 &= !(1 << (w.id & 1));
                    setup(&mut w, &at, &mut hs, &town, centre, 10);
                }
            }
        }
    }
}

fn context(identity: &crate::save::PlayerIdentity, disc: &GameDisc, index: u32) -> MsgContext {
    let now = crate::clock::now();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Villager".into(),
        tail: crate::dialogue::catchphrase(disc, index),
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

fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &MsgContext,
    name: &'static str,
    msg: u16,
) {
    let Some(text) = disc.text.as_ref() else {
        return;
    };
    let (pages, asked) = text.talk_asking(usize::from(msg), ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        name,
        (0, SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Hatsumode message {msg}");
}

/// The talks, the player's walk and the prayer.
#[allow(clippy::too_many_arguments)]
fn talk(
    time: Res<Time>,
    disc: Option<Res<GameDisc>>,
    town: Res<Town>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    mut player: Query<(&mut Transform, &mut Facing), (With<Player>, Without<Walker>)>,
    mut walkers: Query<(Entity, &mut Transform, &mut Walker)>,
    mut flow: ResMut<Visit>,
    mut escort: ResMut<Escort>,
    mut hs: ResMut<Hatsu>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Some(centre) = well(&town) else { return };
    let dt = time.delta_secs();
    // The player's walk.
    if let Some(to) = escort.to {
        if let Ok((mut at, mut facing)) = player.single_mut() {
            let d = (to - at.translation).with_y(0.0);
            let step = ESCORT * dt;
            if d.length() <= step {
                at.translation.x = to.x;
                at.translation.z = to.z;
                escort.to = None;
            } else {
                let dir = d.normalize();
                at.translation += dir * step;
                at.translation.y = town.ground_y(at.translation) + 1.0;
                facing.dir = dir;
                at.rotation = Quat::from_rotation_y(dir.x.atan2(dir.z));
            }
        } else {
            escort.to = None;
        }
        *focus = InputFocus::Transition;
        return;
    }
    // The prayer.
    if let Phase::Praying(left) = flow.phase {
        let left = left - dt;
        if left <= 0.0 {
            flow.phase = Phase::Idle;
            hs.flags0 &= !0x8000;
            *focus = InputFocus::World;
        } else {
            flow.phase = Phase::Praying(left);
            *focus = InputFocus::Transition;
        }
        return;
    }
    // The shrine's turn (`aHTC_wait` -> `aHTC_request`).
    if hs.flags0 & 0x8000 != 0 && matches!(flow.phase, Phase::Idle) && !dialogue.active {
        let Some((actor, _, w)) = walkers.iter().min_by_key(|(_, _, w)| w.id) else {
            return;
        };
        let index = w.index;
        if let Ok((mut at, mut facing)) = player.single_mut() {
            facing.dir = Vec3::NEG_Z;
            at.rotation = Quat::from_rotation_y(std::f32::consts::PI);
        }
        let ctx = context(&identity, &disc, index);
        *focus = InputFocus::Dialogue;
        if inventory.bells >= PRICE {
            say(&disc, &mut dialogue, actor, &ctx, "", 4396);
            flow.phase = Phase::Prompt;
        } else {
            say(&disc, &mut dialogue, actor, &ctx, "", 4398);
            flow.phase = Phase::Said(false);
        }
        return;
    }
    // A line was read, or an answer given.
    if let Some(actor) = dialogue.speaker.filter(|&e| {
        walkers.get(e).is_ok() && (dialogue.active || dialogue.external_result.is_some())
    }) {
        let Some(choice) = dialogue.external_result.take() else {
            return;
        };
        *focus = InputFocus::Dialogue;
        let (looks, index) = walkers
            .get(actor)
            .map(|(_, _, w)| (w.looks, w.index))
            .unwrap_or((0, 0));
        let ctx = context(&identity, &disc, index);
        let base = BASE_MSG[usize::from(looks).min(5)];
        match std::mem::take(&mut flow.phase) {
            Phase::Asked(who) => {
                if choice == 0 {
                    say(&disc, &mut dialogue, actor, &ctx, "Villager", base + 1);
                    flow.phase = Phase::Joining(who);
                } else {
                    say(&disc, &mut dialogue, actor, &ctx, "Villager", base + 2);
                    flow.phase = Phase::Line(who);
                }
            }
            Phase::Joining(who) => {
                // `aHN0_talk_stop_player`: he goes to think 34 and the player is walked.
                *dialogue = DialogueState::default();
                if let Some((_, at, mut w)) = who.and_then(|e| walkers.get_mut(e).ok()) {
                    w.player = 0;
                    setup(&mut w, &at, &mut hs, &town, centre, 34);
                }
                *focus = InputFocus::Transition;
            }
            Phase::Prompt => {
                if choice == 0 {
                    say(&disc, &mut dialogue, actor, &ctx, "", 0x112D);
                    flow.phase = Phase::Said(true);
                } else {
                    say(&disc, &mut dialogue, actor, &ctx, "", 0x112E);
                    flow.phase = Phase::Said(false);
                }
            }
            Phase::Said(coin) => {
                // `aHTC_pay`, `aHTC_saisen`, `aHTC_inori`.
                *dialogue = DialogueState::default();
                if coin {
                    inventory.bells = inventory.bells.saturating_sub(PRICE);
                    info!("The New Year offering: {PRICE} Bells");
                }
                flow.phase = Phase::Praying(PLAYER_PRAYER);
                *focus = InputFocus::Transition;
            }
            Phase::Line(who) => {
                // The villager goes on at the think that follows his talk.
                *dialogue = DialogueState::default();
                if let Some((_, at, mut w)) = who.and_then(|e| walkers.get_mut(e).ok()) {
                    let next = w.next_think;
                    setup(&mut w, &at, &mut hs, &town, centre, next);
                }
                *focus = InputFocus::World;
            }
            Phase::Idle | Phase::Praying(_) => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
            }
        }
        return;
    }
    if !matches!(flow.phase, Phase::Idle)
        || !focus.is_world()
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    let cur = hs.cur_npc_idx;
    let Some((actor, _, mut w)) = walkers.iter_mut().find(|(_, t, w)| {
        let delta = (t.translation - at.translation).with_y(0.0);
        w.req == Req::Norm
            && delta.length() < 1.8
            && facing.dir.dot(delta.normalize_or_zero()) >= 0.5
    }) else {
        return;
    };
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    let ctx = context(&identity, &disc, w.index);
    let base = BASE_MSG[usize::from(w.looks).min(5)];
    if w.talk_idx <= 1 {
        // `dt_tbl1[0 / 1]`: the invitation.
        say(&disc, &mut dialogue, actor, &ctx, "Villager", base);
        flow.phase = Phase::Asked(Some(actor));
    } else {
        // `aHN0_make_msg` by his place after the one at the front.
        let idx = (4 + w.id - cur) & 3;
        let offset = match idx {
            1 => 3,
            2 => 6,
            3 => 9,
            _ => 12,
        };
        let msg = base + offset + rand.0.random(3) as u16;
        say(&disc, &mut dialogue, actor, &ctx, "Villager", msg);
        flow.phase = Phase::Line(Some(actor));
    }
    // He waits for the talk to end.
    w.act = Act::Wait;
}

pub struct HatsumodePlugin;

impl Plugin for HatsumodePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Visit>()
            .init_resource::<Hatsu>()
            .init_resource::<Escort>()
            .add_systems(
                Update,
                (
                    visit.after(crate::event_today::refresh),
                    walkers.after(visit),
                )
                    .in_set(GameplaySet),
            )
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
