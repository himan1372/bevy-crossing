//! Copper (SP_NPC_POLICE) on guard outside the police box.
//!
//! m_field_make: he stands two units east of the POLICE_STATION unit,
//! facing south (rotation 0), and keeps whatever item lay there in the lost
//! and found (ac_npc_police ct, mPB_keep_item). He never walks: he turns to
//! talk and back again. ac_npc_police_move think, about once a second while
//! standing: 06:00-07:00 (not in rain) with the player off to the side (more
//! than 67.5 degrees) a 1-in-20 chance of exercises (taisou1/2/7, looped
//! 2/4/1 times, stopped when the player comes in front); 02:00-04:00 with the
//! player 60 or more game units away, a 1-in-20 chance of dozing (wait_nemu1
//! five times, then a startled gyafun1; the player coming near ends it).
//! Talk: the greeting by time (0x772 05-10, 0x773 10-17, 0x774 17-, 0x775
//! before 05), or 0x771 the first time the player comes out of the box;
//! then his question: an event hint, how many items are in the lost and
//! found (0x782 / 0x783), 0x777, or 0x776.
use bevy::prelude::*;

use crate::character::NpcClip;
use crate::dialogue::DialogueState;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::Facing;
use crate::messages::MsgContext;
use crate::player::Player;
use crate::shop::ShopLocation;
use crate::states::GameplaySet;
use crate::town::Town;

const UNIT: f32 = 40.0;
/// `POLICE_STATION` layout item.
const POLICE_STATION: u16 = 0x580C;
/// ALL_NPC_NUM + (SP_NPC_POLICE - SP_NPC_START); l_sp_actor_name sound 2.
const COPPER_NPC: u32 = 238 + 13;
const COPPER_SOUND: u8 = 2;
/// aNPC think: how often a standing Copper decides what to do next.
const THINK: f32 = 1.0;
/// DEG2SHORT_ANGLE2(67.5).
const SIDE: f32 = 67.5;
/// Seconds after coming out of the box before he calls out (0x771).
const ARRIVAL: f32 = 0.6;

/// His unit, which the player cannot walk through.
#[derive(Resource)]
pub(crate) struct CopperUnit(Vec3);

impl CopperUnit {
    pub(crate) fn blocks(&self, x: f32, z: f32) -> bool {
        (x - self.0.x).abs() < 0.85 && (z - self.0.z).abs() < 0.85
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Act {
    Wait,
    /// Exercises: loops still to play.
    Taisou(u32),
    /// Dozing (false), then startled awake (true).
    Nemu(bool),
}

#[derive(Component)]
struct Copper {
    act: Act,
    think: f32,
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// His question after the greeting.
    Menu,
    Finish,
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
    context: MsgContext,
    /// exit_greeting: he has called out to a player leaving the box.
    exit_greeted: bool,
    /// Seconds since coming out of the box, until he calls out.
    outside_for: Option<f32>,
    /// Calling out without turning (talk_turn FALSE).
    no_turn: bool,
}

fn spawn(town: Res<Town>, mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let Some(b) = town
        .retail
        .as_ref()
        .and_then(|r| r.buildings.iter().find(|b| b.item == POLICE_STATION))
    else {
        return;
    };
    // The station's origin is its unit's centre; he is two units east.
    let p = b.origin + Vec3::X * 2.0;
    let home = p.with_y(town.ground_y(p));
    commands.insert_resource(CopperUnit(home));
    commands.spawn((
        Name::new("Copper"),
        Copper {
            act: Act::Wait,
            think: 0.0,
        },
        NpcClip::default(),
        crate::character::NpcModel(COPPER_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(home + Vec3::Y),
    ));
    info!("Copper on guard at {home:?}");
}

/// ac_npc_police ct: the item on his unit goes to the lost and found.
fn keep_item_on_post(
    unit: Option<Res<CopperUnit>>,
    items: Option<ResMut<crate::field_items::FieldItems>>,
    lost: Option<ResMut<crate::police_box::LostAndFound>>,
    mut done: Local<bool>,
) {
    let (Some(unit), Some(mut items), Some(mut lost)) = (unit, items, lost) else {
        return;
    };
    if *done {
        return;
    }
    *done = true;
    if let Some((a, c, item)) = items.at(unit.0) {
        if matches!(item >> 12, 1..=3) {
            lost.keep(item);
            items.acres[a][c] = 0;
        }
    }
}

/// Degrees between his facing (rotation 0: +z) and the way to the player.
fn player_angle(copper: Vec3, player: Vec3) -> f32 {
    let d = (player - copper).with_y(0.0);
    d.x.atan2(d.z).to_degrees().abs()
}

/// aPOL_think_main_proc and the exercise / doze actions.
#[allow(clippy::type_complexity)]
fn think(
    time: Res<Time>,
    flow: Res<Talk>,
    dialogue: Res<DialogueState>,
    player: Query<&Transform, (With<Player>, Without<Copper>)>,
    mut copper: Query<(Entity, &mut Transform, &mut Copper, &mut NpcClip)>,
) {
    let (Ok(at), Ok((entity, mut transform, mut copper, mut clip))) =
        (player.single(), copper.single_mut())
    else {
        return;
    };
    let dt = time.delta_secs();
    let talking =
        dialogue.active && dialogue.speaker == Some(entity) || matches!(flow.step, Step::Menu);
    // Turn to the player while talking, back to the south otherwise.
    let to = (at.translation - transform.translation).with_y(0.0);
    let want = if talking && !flow.no_turn && to.length() > 0.01 {
        Quat::from_rotation_y(to.x.atan2(to.z))
    } else {
        Quat::IDENTITY
    };
    transform.rotation = transform.rotation.slerp(want, (dt * 8.0).min(1.0));
    if talking {
        if copper.act != Act::Wait {
            copper.act = Act::Wait;
            clip.stop(8.0);
        }
        return;
    }
    let angle = player_angle(transform.translation, at.translation);
    let distance = (at.translation - transform.translation)
        .with_y(0.0)
        .length()
        * UNIT;
    match copper.act {
        Act::Taisou(loops) => {
            if clip.loops >= loops || angle <= SIDE {
                copper.act = Act::Wait;
                clip.stop(8.0);
            }
        }
        Act::Nemu(false) => {
            if clip.loops >= 5 {
                copper.act = Act::Nemu(true);
                clip.start("cKF_ba_r_npc_1_gyafun1", true, 2.0);
            } else if distance < 60.0 {
                copper.act = Act::Wait;
                clip.stop(8.0);
            }
        }
        Act::Nemu(true) => {
            if clip.done {
                copper.act = Act::Wait;
                clip.stop(8.0);
            }
        }
        Act::Wait => {
            copper.think += dt;
            if copper.think < THINK || transform.rotation.angle_between(Quat::IDENTITY) > 0.05 {
                return;
            }
            copper.think = 0.0;
            let now = crate::clock::now();
            let secs = now.day_secs % 86_400;
            let chance = rand_unit(secs ^ (time.elapsed_secs() * 1000.0) as u32) < 0.05;
            if (6 * 3600..7 * 3600).contains(&secs) && angle > SIDE && chance {
                const TAISOU: [(&str, u32); 3] = [
                    ("cKF_ba_r_npc_1_taisou1", 2),
                    ("cKF_ba_r_npc_1_taisou2", 4),
                    ("cKF_ba_r_npc_1_taisou7", 1),
                ];
                let (name, loops) = TAISOU[(rand_unit(secs.rotate_left(7)) * 3.0) as usize % 3];
                copper.act = Act::Taisou(loops);
                clip.start(name, false, 8.0);
                info!("Copper exercises ({name})");
            } else if (2 * 3600..4 * 3600).contains(&secs) && distance >= 60.0 && chance {
                copper.act = Act::Nemu(false);
                clip.start("cKF_ba_r_npc_1_wait_nemu1", false, 16.0);
                info!("Copper dozes off");
            }
        }
    }
}

/// A number in [0, 1) from a seed (RANDOM_F).
fn rand_unit(seed: u32) -> f32 {
    let mut x = seed.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    (x >> 8) as f32 / (1u32 << 24) as f32
}

fn context(identity: &crate::save::PlayerIdentity) -> MsgContext {
    let now = crate::clock::now();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Copper".into(),
        free: vec![String::new(); 20],
        items: vec![String::new(); 5],
        item_articles: vec![0; 5],
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
    msg: usize,
) -> bool {
    let Some(text) = disc.text.as_ref() else {
        return false;
    };
    let (pages, choice) = text.talk(msg, ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "Copper",
        (COPPER_NPC, COPPER_SOUND),
        pages,
        choice.as_ref().map(|c| c.labels.clone()),
    );
    info!("Copper message {msg:#06x}");
    choice.is_some()
}

/// aPOL_get_hint_msg_no. No special event is ever scheduled yet and the
/// part-time job is not played, so it is always "nothing planned".
fn hint() -> usize {
    0x2C37
}

#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    time: Res<Time>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    mut player: Query<(&Transform, &Facing, &mut crate::locomotion::Locomotion), With<Player>>,
    copper: Query<(Entity, &Transform), With<Copper>>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    lost: Res<crate::police_box::LostAndFound>,
    identity: Res<crate::save::PlayerIdentity>,
    mut was: Local<Option<ShopLocation>>,
) {
    let came_out = was.replace(*state.get()) == Some(ShopLocation::PoliceBox)
        && *state.get() == ShopLocation::Town;
    if came_out && !flow.exit_greeted {
        flow.outside_for = Some(0.0);
    }
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, actor_at)) = copper.single() else {
        return;
    };

    if let Some(choice) = dialogue.external_result.take() {
        *focus = InputFocus::Dialogue;
        match std::mem::take(&mut flow.step) {
            // aPOL_set_select_after_msg.
            Step::Menu => {
                let msg = match choice {
                    0 => hint(),
                    1 if lost.count() != 0 => {
                        flow.context.free[0] = lost.count().to_string();
                        0x0782
                    }
                    1 => 0x0783,
                    2 => 0x0777,
                    _ => 0x0776,
                };
                say(&disc, &mut dialogue, actor, &flow.context, msg);
                flow.step = Step::Finish;
            }
            _ => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
            }
        }
        return;
    }

    // aPOL_talk_request: the first time out of the box he calls out.
    if let Some(since) = flow.outside_for.as_mut() {
        *since += time.delta_secs();
        if *since >= ARRIVAL && matches!(flow.step, Step::Idle | Step::Finish) && focus.is_world() {
            flow.outside_for = None;
            flow.exit_greeted = true;
            flow.context = context(&identity);
            if let Ok((_, _, mut loco)) = player.single_mut() {
                *loco = default();
            }
            *focus = InputFocus::Dialogue;
            flow.no_turn = true;
            let asks = say(&disc, &mut dialogue, actor, &flow.context, 0x0771);
            flow.step = if asks { Step::Menu } else { Step::Finish };
            return;
        }
    }

    if !matches!(flow.step, Step::Idle | Step::Finish)
        || !focus.is_world()
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let Ok((at, facing, _)) = player.single() else {
        return;
    };
    let delta = (actor_at.translation - at.translation).with_y(0.0);
    if delta.length() > 1.6 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    flow.no_turn = false;
    flow.context = context(&identity);
    let hour = crate::clock::now().hour;
    let msg = match hour {
        5..=9 => 0x0772,
        10..=16 => 0x0773,
        17.. => 0x0774,
        _ => 0x0775,
    };
    let asks = say(&disc, &mut dialogue, actor, &flow.context, msg);
    flow.step = if asks { Step::Menu } else { Step::Finish };
}

pub struct CopperPlugin;

impl Plugin for CopperPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .add_systems(Startup, spawn.after(crate::town::generate_town))
            .add_systems(Update, (keep_item_on_post, think))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
