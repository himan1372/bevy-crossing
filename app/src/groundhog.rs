//! Groundhog Day at the shrine (`ac_groundhog_control.c`, `ac_ev_majin*.c`, `ac_ev_speech_soncho*.c`).
//!
//! `mEv_EVENT_GROUNDHOG_DAY` runs on February 2 from 07:00 to 08:59. Tortimer (`SP_NPC_EV_SPEECH_SONCHO`) waits at unit
//! (8, 9) of the shrine block; until 08:00 a talk with him is `0x3DB5 + 1 + RANDOM(5)` (`aESS_set_norm_talk_info`). At
//! 08:00 the control actor starts the show for a player in the shrine acre: the groundhog (`SP_NPC_EV_MAJIN`) comes
//! up out of the ground at unit (5, 8) and speaks by himself (`aEMJ_set_force_talk_info`: clear `0x3DAF + RANDOM(3)`,
//! snow `0x3DB2 + RANDOM(3)`), then goes back down; then Tortimer gives his speech `0x3DB5` (`aESS_force_talk_info`).
//! After the speech a talk with him is `0x3DB5 + 6 + RANDOM(5)` in clear weather, `0x3DB5 + 11 + RANDOM(5)` in snow.
//!
//! Port adaptations: the port has no weather beyond the scheduler's snow days, so the weather is snow on a snow day and
//! clear otherwise (rain does not exist); the groundhog's rise, the hole effect, the camera and the music are not
//! drawn; he appears and leaves after a pause.
//!
//! Not built: the four villagers who stand with the audience, the camera work, and the later Tortimer
//! (`SONCHO_GROUNDHOG_DAY`, 09:00-16:59) with the commemorative item.

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
use crate::shop::ShopLocation;
use crate::states::GameplaySet;
use crate::town::Town;

/// ALL_NPC_NUM + (SP_NPC_EV_MAJIN - SP_NPC_START).
const MAJIN_NPC: u32 = 238 + 129;
/// ALL_NPC_NUM + (SP_NPC_EV_SPEECH_SONCHO - SP_NPC_START).
const TORTIMER_NPC: u32 = 238 + 135;
/// `l_sp_actor_name` sound id of both.
const SOUND: u8 = 2;
/// `mString_SPEECH_SONCHO_START`: the base of the speech messages.
const SPEECH: u16 = 0x3DB5;
/// Seconds the groundhog is up before he speaks, and down before Tortimer starts.
const RISE: f32 = 2.5;
const SINK: f32 = 1.5;

#[derive(Component)]
struct Speaker {
    day: i64,
}

#[derive(Component)]
struct Groundhog;

#[derive(Component)]
struct SpeechTortimer;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    /// Before the show (or the player is elsewhere).
    #[default]
    Waiting,
    /// The groundhog is up and about to speak.
    Rising,
    /// The groundhog's line is up.
    GroundhogSpeaks,
    /// The groundhog is going back down.
    Sinking,
    /// Tortimer's speech is up.
    TortimerSpeaks,
    /// The show is over for the day.
    Done,
}

#[derive(Resource, Default)]
struct Show {
    day: i64,
    phase: Phase,
    timer: f32,
}

fn snowing(today: &TodayEvents, hour: u8) -> bool {
    today.is_active(ev::WEATHER_SNOW, hour)
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

fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    identity: &crate::save::PlayerIdentity,
    (name, model): (&'static str, u32),
    msg: u16,
) {
    let Some(text) = disc.text.as_ref() else {
        return;
    };
    let ctx = context(identity, name);
    let (pages, _) = text.talk_asking(usize::from(msg), &ctx, &mut |_| 0);
    dialogue.begin_special(actor, name, (model, SOUND), pages, None);
    info!("{name} message {msg:#06x}");
}

/// The shrine acre the player is in?
fn at_shrine(town: &Town, player: Vec3) -> bool {
    let (Some(shrine), Some(here)) = (
        crate::tortimer::shrine_cell(town, 8, 8).and_then(Town::cell_indices_of),
        Town::cell_indices_of(player),
    ) else {
        return false;
    };
    (shrine.0, shrine.1) == (here.0, here.1)
}

/// Tortimer waits at the shrine during the event's two hours.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Speaker)>,
    mut show: ResMut<Show>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let on = today.is_active(ev::GROUNDHOG_DAY, now.hour);
    let out =
        on && *state.get() == ShopLocation::Town && *house.get() == crate::house::HouseState::Town;
    for (entity, s) in &existing {
        if !on || s.day != now.day_number {
            commands.entity(entity).despawn();
        }
    }
    if show.day != now.day_number {
        *show = Show {
            day: now.day_number,
            ..default()
        };
    }
    if !out || !existing.is_empty() {
        return;
    }
    let Some(at) = crate::tortimer::shrine_cell(&town, 8, 9) else {
        return;
    };
    commands.spawn((
        Name::new("Tortimer (speech)"),
        Speaker {
            day: now.day_number,
        },
        SpeechTortimer,
        NpcClip::default(),
        NpcGait::default(),
        NpcModel(TORTIMER_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y),
    ));
    info!("Tortimer waits for the groundhog at {at:?}");
}

/// The show: `aGHC_birth_reset` and the actors' speeches, in order.
#[allow(clippy::too_many_arguments)]
fn run(
    mut commands: Commands,
    time: Res<Time>,
    disc: Option<Res<GameDisc>>,
    town: Res<Town>,
    today: Res<TodayEvents>,
    mut show: ResMut<Show>,
    mut dialogue: ResMut<DialogueState>,
    mut focus: ResMut<InputFocus>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut meshes: ResMut<Assets<Mesh>>,
    player: Query<&Transform, With<Player>>,
    tortimer: Query<Entity, With<SpeechTortimer>>,
    groundhog: Query<Entity, With<Groundhog>>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    let (Ok(tortimer), Ok(player)) = (tortimer.single(), player.single()) else {
        return;
    };
    let now = crate::clock::now();
    match show.phase {
        Phase::Waiting => {
            // 08:00, the player in the shrine acre, free to be spoken to.
            if now.hour >= 8 && at_shrine(&town, player.translation) && focus.is_world() {
                let Some(at) = crate::tortimer::shrine_cell(&town, 5, 8) else {
                    return;
                };
                commands.spawn((
                    Name::new("Groundhog"),
                    Groundhog,
                    NpcClip::default(),
                    NpcGait::default(),
                    NpcModel(MAJIN_NPC),
                    Mesh3d(meshes.add(Capsule3d::new(0.3, 0.6))),
                    Transform::from_translation(at + Vec3::Y)
                        .with_rotation(Quat::from_rotation_y(0.0)),
                ));
                info!("The groundhog comes up at {at:?}");
                show.phase = Phase::Rising;
                show.timer = RISE;
            }
        }
        Phase::Rising => {
            show.timer -= time.delta_secs();
            if show.timer <= 0.0 && focus.is_world() {
                let Ok(majin) = groundhog.single() else {
                    show.phase = Phase::Waiting;
                    return;
                };
                // `aEMJ_set_force_talk_info`.
                let msg = if snowing(&today, now.hour) {
                    0x3DB2 + rand.0.random(3) as u16
                } else {
                    0x3DAF + rand.0.random(3) as u16
                };
                *focus = InputFocus::Dialogue;
                say(
                    &disc,
                    &mut dialogue,
                    majin,
                    &identity,
                    ("Groundhog", MAJIN_NPC),
                    msg,
                );
                show.phase = Phase::GroundhogSpeaks;
            }
        }
        Phase::GroundhogSpeaks => {
            let Ok(majin) = groundhog.single() else {
                show.phase = Phase::Waiting;
                return;
            };
            if dialogue.speaker == Some(majin) && dialogue.external_result.take().is_some() {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
                show.phase = Phase::Sinking;
                show.timer = SINK;
            }
        }
        Phase::Sinking => {
            show.timer -= time.delta_secs();
            if show.timer <= 0.0 {
                for majin in &groundhog {
                    commands.entity(majin).despawn();
                }
                if focus.is_world() {
                    // `aESS_set_force_talk_info`.
                    *focus = InputFocus::Dialogue;
                    say(
                        &disc,
                        &mut dialogue,
                        tortimer,
                        &identity,
                        ("Tortimer", TORTIMER_NPC),
                        SPEECH,
                    );
                    show.phase = Phase::TortimerSpeaks;
                }
            }
        }
        Phase::TortimerSpeaks => {
            if dialogue.speaker == Some(tortimer) && dialogue.external_result.take().is_some() {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
                show.phase = Phase::Done;
            }
        }
        Phase::Done => {}
    }
}

/// A talk the player starts with Tortimer (`aESS_set_norm_talk_info`).
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    today: Res<TodayEvents>,
    show: Res<Show>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), With<Player>>,
    tortimer: Query<(Entity, &Transform), With<SpeechTortimer>>,
    mut dialogue: ResMut<DialogueState>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    let Ok((actor, actor_at)) = tortimer.single() else {
        return;
    };
    // His own answers end the talk (the show's speeches are handled in `run`).
    if matches!(show.phase, Phase::Waiting | Phase::Done)
        && dialogue.speaker == Some(actor)
        && dialogue.external_result.take().is_some()
    {
        *dialogue = DialogueState::default();
        *focus = InputFocus::World;
        return;
    }
    if !matches!(show.phase, Phase::Waiting | Phase::Done)
        || !focus.is_world()
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    let delta = (actor_at.translation - at.translation).with_y(0.0);
    if delta.length() > 1.8 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    let now = crate::clock::now();
    let msg = if now.hour < 8 {
        SPEECH + 1 + rand.0.random(5) as u16
    } else if snowing(&today, now.hour) {
        SPEECH + 11 + rand.0.random(5) as u16
    } else {
        SPEECH + 6 + rand.0.random(5) as u16
    };
    *focus = InputFocus::Dialogue;
    say(
        &disc,
        &mut dialogue,
        actor,
        &identity,
        ("Tortimer", TORTIMER_NPC),
        msg,
    );
}

pub struct GroundhogPlugin;

impl Plugin for GroundhogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Show>()
            .add_systems(
                Update,
                visit.after(crate::event_today::refresh).in_set(GameplaySet),
            )
            .add_systems(Update, run.after(visit).in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
