//! The sports fair's ball toss (`ac_tamaire_npc0.c`, `ac_tamaire_npc1.c`, `ac_tama.c`, `ac_tamaire_npc*_schedule.c_inc`).
//!
//! While `mEv_EVENT_SPORTS_FAIR_BALL_TOSS` runs (13:00-14:59 on the equinox days) the shrine has two baskets, red at unit
//! (3, 8) and white at (12, 8), each with a team of two tossers (`SP_NPC_EV_TAMAIRE_1..4`), and the leader
//! (`SP_NPC_EV_TAMAIRE_0`). A tosser's round (`aTMN1_*`): looks about (`KYORO`, 124 ticks), runs to a spot
//! `70 + RANDOM(20)` units from the basket, picks up three balls (`HIROU`), turns to the basket, throws them one by one
//! (`NAGERU`, the ball leaves the hand halfway through) and stands two seconds (`NAGERU_END`), then looks about again.
//! Only one tosser of a team picks up at a time (`aTMN1_nageru_bitcheck`; the other looks about once more, and a free one
//! goes with probability 0.8). The leader walks between four stations round the well (`aTMN0_set_move_pos`) and cheers
//! for 700 ticks at each. Tortimer stands at (9, 4) and does not talk.
//!
//! Talking: a tosser `base(looks) + offset + RANDOM(3)` with the offset `3` while looking about or running, `6` picking
//! up or throwing, `9` after a throw; the leader `base0(looks) + RANDOM(3)`. The bases are `7781 7793 7769 7805 7817
//! 7829` by looks (the leader's the same table).
//!
//! Not built: the animations, the basket and ball models (plain shapes stand in), the ball effect's own scatter and the
//! sounds. The shrine's structures are drawn as placeholders.

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

/// ALL_NPC_NUM + (SP_NPC_EV_TAMAIRE_0 - SP_NPC_START) is unknown to the port's models: the residents' own are used.
const TAMAIRE_NPC: u32 = 238 + 83;
const SOUND: u8 = 2;
const UNIT: f32 = 40.0;
const BASE_MSG: [u16; 6] = [7781, 7793, 7769, 7805, 7817, 7829];
/// Units of the leader and the four tossers, and of the two baskets (`l_fdd_tama_map`).
const LEADER_UNIT: (usize, usize) = (7, 12);
const TOSSER_UNITS: [(usize, usize); 4] = [(5, 6), (2, 9), (13, 6), (10, 9)];
const BASKET_UNITS: [(usize, usize); 2] = [(3, 8), (12, 8)];
pub(crate) const TORTIMER_UNIT: (usize, usize) = (9, 4);
/// The leader's stations relative to the well, in units.
const STATIONS: [(f32, f32); 4] = [(80.0, 160.0), (200.0, 40.0), (-40.0, 160.0), (-160.0, 40.0)];
/// The baskets' offsets from the well, in units (`yasiro_pos` of a team).
const BASKET_OFFSET: [(f32, f32); 2] = [(-160.0, 40.0), (200.0, 40.0)];

#[derive(Component)]
struct Toss {
    day: i64,
}

#[derive(Component)]
struct Basket;

#[derive(Component)]
struct Ball {
    from: Vec3,
    to: Vec3,
    age: f32,
}

#[derive(Component)]
struct Leader {
    looks: u8,
    station: usize,
    timer: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stage {
    /// `KYORO_MAE` / `KYORO`: looking about.
    Look,
    /// Running to the picking spot.
    Run,
    /// Waiting to pick up (`HIROU_MAE`).
    Ready,
    /// Picking up the three balls.
    Pick,
    /// Throwing; `n` balls are left.
    Throw(u8),
    /// After the throws.
    Rest,
}

#[derive(Component)]
struct Tosser {
    looks: u8,
    team: usize,
    stage: Stage,
    timer: f32,
    /// Seconds into the throw; the ball leaves at the middle.
    throw_age: f32,
    thrown: bool,
    spot: Vec3,
}

/// `aTMN1_flag`: a team's tosser is picking up or throwing.
#[derive(Resource, Default)]
struct Busy([bool; 2]);

fn well(town: &Town) -> Option<Vec3> {
    town.retail
        .as_ref()?
        .buildings
        .iter()
        .find(|b| b.item == 0x5825)
        .map(|b| b.origin)
}

#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    roster: Option<Res<TownRoster>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Toss)>,
    mut busy: ResMut<Busy>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let now = crate::clock::now();
    let on = today.is_active(ev::SPORTS_FAIR_BALL_TOSS, now.hour);
    let out =
        on && *state.get() == ShopLocation::Town && *house.get() == crate::house::HouseState::Town;
    for (entity, t) in &existing {
        if !out || t.day != now.day_number {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    let Some(roster) = roster else { return };
    *busy = Busy::default();
    let toss = || Toss {
        day: now.day_number,
    };
    // The baskets.
    for (i, unit) in BASKET_UNITS.iter().enumerate() {
        let Some(at) = crate::tortimer::shrine_cell(&town, unit.0, unit.1) else {
            return;
        };
        let colour = if i == 0 {
            Color::srgb(0.85, 0.15, 0.1)
        } else {
            Color::srgb(0.95, 0.95, 0.95)
        };
        commands.spawn((
            Name::new(format!("basket {i}")),
            toss(),
            Basket,
            Mesh3d(meshes.add(Cylinder::new(0.55, 0.9))),
            MeshMaterial3d(materials.add(colour)),
            Transform::from_translation(at + Vec3::Y * 0.45),
        ));
    }
    let model = |i: usize| {
        roster
            .residents
            .get(i)
            .map_or(TAMAIRE_NPC + i as u32, |e| e.index)
    };
    let looks = |i: usize| roster.residents.get(i).map_or(0, |e| e.looks);
    if let Some(at) = crate::tortimer::shrine_cell(&town, LEADER_UNIT.0, LEADER_UNIT.1) {
        commands.spawn((
            Name::new("leader (ball toss)"),
            toss(),
            Leader {
                looks: looks(0),
                station: 0,
                timer: 0.0,
            },
            NpcClip::default(),
            NpcGait::default(),
            NpcModel(model(0)),
            Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
            Transform::from_translation(at + Vec3::Y),
        ));
    }
    for (k, unit) in TOSSER_UNITS.iter().enumerate() {
        let Some(at) = crate::tortimer::shrine_cell(&town, unit.0, unit.1) else {
            return;
        };
        commands.spawn((
            Name::new(format!("tosser {} (ball toss)", k + 1)),
            toss(),
            Tosser {
                looks: looks(k + 1),
                team: k / 2,
                stage: Stage::Look,
                timer: 0.2 * (k as f32 + 1.0),
                throw_age: 0.0,
                thrown: false,
                spot: at,
            },
            NpcClip::default(),
            NpcGait::default(),
            NpcModel(model(k + 1)),
            Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
            Transform::from_translation(at + Vec3::Y),
        ));
    }
    info!("The ball toss is set at the shrine");
}

/// Where a tosser picks up: `70 + RANDOM(20)` units from the basket at a random side (`aTMN1_Next_move`).
fn pick_spot(centre: Vec3, team: usize, rand: &mut crate::game_rand::GameRand) -> Vec3 {
    let basket = centre + Vec3::new(BASKET_OFFSET[team].0, 0.0, BASKET_OFFSET[team].1) / UNIT;
    let angle = rand.0.fqrand() * std::f32::consts::TAU;
    let r = (70.0 + rand.0.fqrand() * 20.0) / UNIT;
    basket + Vec3::new(angle.sin() * r, 0.0, angle.cos() * r)
}

fn face(at: &mut Transform, toward: Vec3) {
    let to = (toward - at.translation).with_y(0.0);
    if to.length() > 0.01 {
        at.rotation = Quat::from_rotation_y(to.x.atan2(to.z));
    }
}

/// The round of every tosser, the leader's walk and the balls in the air.
#[allow(clippy::too_many_arguments)]
fn run(
    mut commands: Commands,
    time: Res<Time>,
    town: Res<Town>,
    today: Res<TodayEvents>,
    mut busy: ResMut<Busy>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player: Query<
        &Transform,
        (
            With<Player>,
            Without<Tosser>,
            Without<Leader>,
            Without<Ball>,
        ),
    >,
    mut tossers: Query<(&mut Tosser, &mut Transform), (Without<Leader>, Without<Ball>)>,
    mut leader: Query<(&mut Leader, &mut Transform), (Without<Tosser>, Without<Ball>)>,
    mut balls: Query<(Entity, &mut Ball, &mut Transform), (Without<Tosser>, Without<Leader>)>,
) {
    let now = crate::clock::now();
    if !today.is_active(ev::SPORTS_FAIR_BALL_TOSS, now.hour) {
        return;
    }
    let (Some(centre), Ok(player)) = (well(&town), player.single()) else {
        return;
    };
    let in_acre = Town::cell_indices_of(player.translation)
        .zip(Town::cell_indices_of(centre))
        .is_some_and(|(a, b)| (a.0, a.1) == (b.0, b.1));
    if !in_acre {
        return;
    }
    let dt = time.delta_secs();
    let ground = |p: Vec3| p.with_y(town.ground_y(p) + 1.0);
    for (mut t, mut at) in &mut tossers {
        let basket =
            centre + Vec3::new(BASKET_OFFSET[t.team].0, 0.0, BASKET_OFFSET[t.team].1) / UNIT;
        t.timer -= dt;
        match t.stage {
            Stage::Look => {
                if t.timer <= 0.0 {
                    // `aTMN1_THINK_WALK_TURN`, then the run.
                    t.spot = pick_spot(centre, t.team, &mut rand);
                    t.stage = Stage::Run;
                }
            }
            Stage::Run => {
                let to = t.spot - at.translation;
                let to = to.with_y(0.0);
                if to.length() < 0.15 {
                    t.stage = Stage::Ready;
                    t.timer = 10.0 / 60.0;
                } else {
                    let step = to.normalize() * 4.0 * dt;
                    at.translation += step;
                    at.translation = ground(at.translation);
                    face(&mut at, t.spot);
                }
            }
            Stage::Ready => {
                if t.timer <= 0.0 {
                    // `aTMN1_hirou_mae_init`.
                    if !busy.0[t.team] && rand.0.fqrand() < 0.8 {
                        busy.0[t.team] = true;
                        t.stage = Stage::Pick;
                        t.timer = 1.4;
                    } else {
                        t.stage = Stage::Look;
                        t.timer = 10.0 / 60.0 + 124.0 / 60.0;
                    }
                }
            }
            Stage::Pick => {
                face(&mut at, basket);
                if t.timer <= 0.0 {
                    t.stage = Stage::Throw(3);
                    t.throw_age = 0.0;
                    t.thrown = false;
                }
            }
            Stage::Throw(left) => {
                face(&mut at, basket);
                t.throw_age += dt;
                // The ball leaves the hand at frame 15 of a ~1 s throw.
                if !t.thrown && t.throw_age >= 0.5 {
                    t.thrown = true;
                    let scatter =
                        Vec3::new(rand.0.fqrand() - 0.5, 0.0, rand.0.fqrand() - 0.5) * 0.7;
                    commands.spawn((
                        Name::new("ball"),
                        Ball {
                            from: at.translation + Vec3::Y * 0.6,
                            to: basket + Vec3::Y * 0.9 + scatter,
                            age: 0.0,
                        },
                        Mesh3d(meshes.add(Sphere::new(0.14))),
                        MeshMaterial3d(materials.add(if t.team == 0 {
                            Color::srgb(0.9, 0.2, 0.15)
                        } else {
                            Color::srgb(0.95, 0.95, 0.9)
                        })),
                        Transform::from_translation(at.translation + Vec3::Y * 0.6),
                    ));
                }
                if t.throw_age >= 1.0 {
                    t.throw_age = 0.0;
                    t.thrown = false;
                    if left > 1 {
                        t.stage = Stage::Throw(left - 1);
                    } else {
                        t.stage = Stage::Rest;
                        t.timer = 2.0;
                    }
                }
            }
            Stage::Rest => {
                face(&mut at, basket);
                if t.timer <= 0.0 {
                    busy.0[t.team] = false;
                    t.stage = Stage::Look;
                    t.timer = 124.0 / 60.0;
                }
            }
        }
    }
    // The leader walks station to station and cheers.
    for (mut l, mut at) in &mut leader {
        let (sx, sz) = STATIONS[l.station];
        let target = ground(centre + Vec3::new(sx, 0.0, sz) / UNIT);
        let to = (target - at.translation).with_y(0.0);
        if to.length() > 0.2 {
            at.translation += to.normalize() * 2.5 * dt;
            at.translation = ground(at.translation);
            face(&mut at, target);
            l.timer = 700.0 / 60.0;
        } else {
            l.timer -= dt;
            face(&mut at, centre);
            if l.timer <= 0.0 {
                l.station = (l.station + 1) % STATIONS.len();
            }
        }
    }
    // The balls fly an arc into the basket and are gone.
    for (entity, mut ball, mut at) in &mut balls {
        ball.age += dt;
        let k = (ball.age / 0.9).min(1.0);
        let mut p = ball.from.lerp(ball.to, k);
        p.y += 1.6 * (1.0 - (2.0 * k - 1.0).powi(2));
        at.translation = p;
        if ball.age > 1.3 {
            commands.entity(entity).despawn();
        }
    }
}

fn context(identity: &crate::save::PlayerIdentity) -> MsgContext {
    let now = crate::clock::now();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Villager".into(),
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

#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), (With<Player>, Without<Toss>)>,
    tossers: Query<(Entity, &Transform, &Tosser)>,
    leaders: Query<(Entity, &Transform, &Leader)>,
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
    let mine = |e: Entity| tossers.get(e).is_ok() || leaders.get(e).is_ok();
    if dialogue.active
        && dialogue.speaker.is_some_and(mine)
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
    let near = |t: &Transform| {
        let delta = (t.translation - at.translation).with_y(0.0);
        delta.length() < 1.8 && facing.dir.dot(delta.normalize_or_zero()) >= 0.5
    };
    let pick = tossers
        .iter()
        .find(|(_, t, _)| near(t))
        .map(|(e, _, s)| {
            let offset = match s.stage {
                Stage::Look | Stage::Run => 3,
                Stage::Ready | Stage::Pick | Stage::Throw(_) => 6,
                Stage::Rest => 9,
            };
            (e, s.looks, offset)
        })
        .or_else(|| {
            leaders
                .iter()
                .find(|(_, t, _)| near(t))
                .map(|(e, _, l)| (e, l.looks, 0))
        });
    let Some((actor, looks, offset)) = pick else {
        return;
    };
    actions.consume(Action::Interact);
    let msg = BASE_MSG[usize::from(looks).min(5)] + offset + rand.0.random(3) as u16;
    let Some(text) = disc.text.as_ref() else {
        return;
    };
    let mut ctx = context(&identity);
    ctx.tail = crate::dialogue::catchphrase(&disc, models.get(actor).map_or(u32::MAX, |m| m.0));
    let (pages, asked) = text.talk_asking(usize::from(msg), &ctx, &mut |_| 0);
    *focus = InputFocus::Dialogue;
    dialogue.begin_special(
        actor,
        "Villager",
        (TAMAIRE_NPC, SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Ball toss talk {msg}");
}

pub struct BallTossPlugin;

impl Plugin for BallTossPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Busy>()
            .add_systems(
                Update,
                visit.after(crate::event_today::refresh).in_set(GameplaySet),
            )
            .add_systems(Update, run.after(visit).in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
