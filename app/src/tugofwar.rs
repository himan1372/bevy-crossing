//! The sports fair's tug of war (`ac_tunahiki_control.c`, `ac_tunahiki_npc0.c`, `ac_tunahiki_npc1.c`).
//!
//! While `mEv_EVENT_SPORTS_FAIR_TUG_OF_WAR` runs (15:00-16:59 on the equinox days) two teams of two pullers stand on
//! the rope at row 11 of the shrine block (units (3,11) and (4,11) facing east, (11,11) and (12,11) facing west, moved in
//! by 85 or 95 units: `aTNN1_think_init_proc`), the leader waves a flag at (7,10) moved by (+20, -30) units, and
//! Tortimer stands at (10,8).
//!
//! The rope (`aTNC_wait`, `aTNN1_aiko`): a pull lasts 63 ticks (1.05 s). At 0.2 s the shake begins and the rope's speed
//! becomes the next speed; at 0.8 s it ends and a new next speed is drawn (`fqrand < 0.45`: `RANDOM_F(20) + 3`; `< 0.9`:
//! the same negated; else 0). While shaking, the rope base moves `speed * 0.025` a tick, clamped to +-10 units (the
//! speed is cut at the ends), and stands still while one of the pullers is being talked to. The pullers stand at their
//! home plus the rope base. At the end of a pull each picks its pose from where the rope is: swaying (`yuri`) when
//! `-(rope * dir) > 5`; straining (`furi`) when `-(rope * dir) < -5` and `next_speed * dir >= 0`; else even (`aiko`).
//!
//! Talking: `base(looks) + offset + RANDOM(3)`, the offset `3` even, `6` swaying, `9` straining; the leader's `0`. The
//! bases are `6532 6544 6520 6556 6568 6580` by looks.
//!
//! Not built: the animations, the rope model (a brown cylinder stands in), the sweat effect, the sounds, any winner.

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

const TUNAHIKI_NPC: u32 = 238 + 88;
const SOUND: u8 = 2;
const UNIT: f32 = 40.0;
const BASE_MSG: [u16; 6] = [6532, 6544, 6520, 6556, 6568, 6580];
/// Units of the four pullers and the leader (`l_fdd_tuna_map`), and how far a puller stands in from its unit.
const PULLER_UNITS: [(usize, usize); 4] = [(3, 11), (4, 11), (11, 11), (12, 11)];
const PULLER_IN: [f32; 4] = [85.0, 95.0, 95.0, 85.0];
const LEADER_UNIT: (usize, usize) = (7, 10);
pub(crate) const TORTIMER_UNIT: (usize, usize) = (10, 8);
/// A pull lasts 63 ticks; the shake runs from frame 6 to frame 24 of the 30-frames-a-second animation.
const CYCLE: f32 = 63.0 / 60.0;
const SHAKE_ON: f32 = 6.0 / 30.0;
const SHAKE_OFF: f32 = 24.0 / 30.0;

#[derive(Component)]
struct Tug {
    day: i64,
}

/// The rope: where its middle is at rest.
#[derive(Component)]
struct Rope(f32);

#[derive(Component)]
struct Puller {
    looks: u8,
    dir: f32,
    home: Vec3,
    pose: Pose,
}

#[derive(Component)]
struct Leader {
    looks: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Pose {
    Even,
    Swaying,
    Straining,
}

#[derive(Resource, Default)]
struct Pull {
    /// Seconds into the pull.
    t: f32,
    shaking: bool,
    /// `rope_base` in units, `speed` and `next_speed`.
    rope: f32,
    speed: f32,
    next_speed: f32,
}

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
    existing: Query<(Entity, &Tug)>,
    mut pull: ResMut<Pull>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let now = crate::clock::now();
    let on = today.is_active(ev::SPORTS_FAIR_TUG_OF_WAR, now.hour);
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
    *pull = Pull::default();
    let tug = || Tug {
        day: now.day_number,
    };
    let model = |i: usize| {
        roster
            .residents
            .get(i)
            .map_or(TUNAHIKI_NPC + i as u32, |e| e.index)
    };
    let looks = |i: usize| roster.residents.get(i).map_or(0, |e| e.looks);
    let mut rope_ends = Vec::new();
    for (k, unit) in PULLER_UNITS.iter().enumerate() {
        let Some(at) = crate::tortimer::shrine_cell(&town, unit.0, unit.1) else {
            return;
        };
        let dir = if k < 2 { 1.0 } else { -1.0 };
        let home = at + Vec3::new(dir * PULLER_IN[k] / UNIT, 0.0, dir * 5.0 / UNIT);
        rope_ends.push(home);
        commands.spawn((
            Name::new(format!("puller {} (tug of war)", k + 1)),
            tug(),
            Puller {
                looks: looks(k + 1),
                dir,
                home,
                pose: Pose::Even,
            },
            NpcClip::default(),
            NpcGait::default(),
            crate::event_body::EventBody::default(),
            crate::event_body::Idle("cKF_ba_r_npc_1_tunahiki_aiko1"),
            NpcModel(model(k + 1)),
            Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
            Transform::from_translation(home + Vec3::Y)
                .with_rotation(Quat::from_rotation_y(dir * std::f32::consts::FRAC_PI_2)),
        ));
    }
    if let Some(at) = crate::tortimer::shrine_cell(&town, LEADER_UNIT.0, LEADER_UNIT.1) {
        let at = at + Vec3::new(20.0 / UNIT, 0.0, -30.0 / UNIT);
        commands.spawn((
            Name::new("leader (tug of war)"),
            tug(),
            Leader { looks: looks(0) },
            NpcClip::default(),
            NpcGait::default(),
            crate::event_body::EventBody::default(),
            crate::event_body::Idle("cKF_ba_r_npc_1_hatafuri1"),
            NpcModel(model(0)),
            Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
            Transform::from_translation(at + Vec3::Y),
        ));
    }
    // The rope between the teams.
    let (from, to) = (rope_ends[0], rope_ends[3]);
    let length = (to - from).length();
    commands.spawn((
        Name::new("rope"),
        tug(),
        Rope((from.x + to.x) * 0.5),
        Mesh3d(meshes.add(Cylinder::new(0.06, length))),
        MeshMaterial3d(materials.add(Color::srgb(0.55, 0.38, 0.2))),
        Transform::from_translation((from + to) * 0.5 + Vec3::Y * 0.9)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
    ));
    info!("The tug of war is set at the shrine");
}

/// The rope and the pullers.
#[allow(clippy::too_many_arguments)]
fn run(
    time: Res<Time>,
    town: Res<Town>,
    today: Res<TodayEvents>,
    dialogue: Res<DialogueState>,
    mut pull: ResMut<Pull>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    player: Query<&Transform, (With<Player>, Without<Puller>, Without<Rope>)>,
    mut pullers: Query<(Entity, &mut Puller, &mut Transform), Without<Rope>>,
    mut rope: Query<(&Rope, &mut Transform), Without<Puller>>,
) {
    let now = crate::clock::now();
    if !today.is_active(ev::SPORTS_FAIR_TUG_OF_WAR, now.hour) {
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
    // `aTNC_FLAG_NPC_TALK_ALL`: the rope stands still while one of them is talked to.
    let talking = dialogue.active && pullers.iter().any(|(e, ..)| dialogue.speaker == Some(e));
    let dt = time.delta_secs();
    let before = pull.t;
    pull.t += dt;
    if pull.t >= CYCLE {
        pull.t -= CYCLE;
        // The poses for the next pull (`aTNN1_aiko`'s end).
        let (rope, next) = (pull.rope, pull.next_speed);
        for (_, mut p, _) in &mut pullers {
            p.pose = if -(rope * p.dir) > 5.0 {
                Pose::Swaying
            } else if -(rope * p.dir) < -5.0 && next * p.dir >= 0.0 {
                Pose::Straining
            } else {
                Pose::Even
            };
        }
    }
    if before < SHAKE_ON && pull.t >= SHAKE_ON {
        pull.shaking = true;
        pull.speed = pull.next_speed;
    }
    if (before < SHAKE_OFF || before > pull.t) && pull.t >= SHAKE_OFF {
        pull.shaking = false;
        let rng = rand.0.fqrand();
        pull.next_speed = if rng < 0.45 {
            rand.0.fqrand() * 20.0 + 3.0
        } else if rng < 0.9 {
            -(rand.0.fqrand() * 20.0 + 3.0)
        } else {
            0.0
        };
    }
    if pull.shaking && !talking {
        // 0.025 a tick at 60 ticks a second.
        pull.rope += pull.speed * 0.025 * 60.0 * dt;
        if pull.rope.abs() >= 10.0 {
            pull.rope = pull.rope.clamp(-10.0, 10.0);
            pull.speed = 0.0;
        }
    }
    let shift = pull.rope / UNIT;
    for (_, p, mut at) in &mut pullers {
        let ground = p.home + Vec3::new(shift, 0.0, 0.0);
        at.translation = ground.with_y(town.ground_y(ground) + 1.0);
    }
    for (r, mut at) in &mut rope {
        // The jitter of a shaking rope (`aTNC_wait`: +-0.3 a frame).
        let wobble = if pull.shaking {
            0.3 / UNIT
                * if (pull.t * 60.0) as u32 & 1 == 0 {
                    1.0
                } else {
                    -1.0
                }
        } else {
            0.0
        };
        at.translation.x = r.0 + shift + wobble;
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
    player: Query<(&Transform, &Facing), (With<Player>, Without<Tug>)>,
    pullers: Query<(Entity, &Transform, &Puller)>,
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
    let mine = |e: Entity| pullers.get(e).is_ok() || leaders.get(e).is_ok();
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
    let pick = pullers
        .iter()
        .find(|(_, t, _)| near(t))
        .map(|(e, _, p)| {
            let offset = match p.pose {
                Pose::Even => 3,
                Pose::Swaying => 6,
                Pose::Straining => 9,
            };
            (e, p.looks, offset)
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
        (TUNAHIKI_NPC, SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Tug of war talk {msg}");
}

pub struct TugOfWarPlugin;

impl Plugin for TugOfWarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pull>()
            .add_systems(
                Update,
                visit.after(crate::event_today::refresh).in_set(GameplaySet),
            )
            .add_systems(Update, run.after(visit).in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
