//! Katrina (`SP_NPC_GYPSY`), the fortune teller.
//!
//! She is out while the original scheduler says `mEv_EVENT_GYPSY` is active (from 21:00 on her day until the
//! morning after; `event_schedule` with the special-event calendar). The original puts her in a fortune tent on a
//! lot for sale (`gypsy_start`); the tent is not built, so she stands on free ground like Joan.
//!
//! Talking (`ac_ev_gypsy_move.c_inc`, kernel `event_visitors`): with a normal destiny she offers a fortune for 50
//! Bells (`0x970`, Yes / No): No `0x971`, Yes without the Bells `0x972`, Yes `0x973`; then the fee is paid, four
//! random words (`0x1A4`..`0x164` + RANDOM(32)) fill the message's item strings and `RANDOM(10)` picks the destiny
//! and its message (`0x974` + index; half of the rolls are "normal"). Asked again after the fortune she answers
//! `0x982`. With a destiny already saved today she says `0x97B` and then `0x97C` + destiny - 1.
//! The destiny (and the day it was told) is saved with the player and goes back to normal on a new day
//! (`Game_play_Reset_destiny`).
//!
//! Not built: the tent, and the destiny's effects on prices and goods (`mPr_GetMoneyPower` / `mPr_GetGoodsPower`).

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::event_visitors as native;

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::messages::MsgContext;
use crate::player::Player;
use crate::shop::ShopLocation;
use crate::states::GameplaySet;
use crate::town::Town;

/// ALL_NPC_NUM + (SP_NPC_GYPSY - SP_NPC_START).
const KATRINA_NPC: u32 = 238 + 12;
/// `l_sp_actor_name` sound id of `SP_NPC_GYPSY`.
const KATRINA_SOUND: u8 = 2;

#[derive(Component)]
struct Katrina {
    /// The game day she came out for.
    day: i64,
}

/// Brings her out for her evening and sends her off when the event ends.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    items: Option<Res<crate::field_items::FieldItems>>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Katrina)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut flow: ResMut<Talk>,
) {
    let now = crate::clock::now();
    let out = today.is_active(ev::GYPSY, now.hour)
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    // After midnight she is still the previous day's visitor.
    let day = if now.hour < 6 {
        now.day_number - 1
    } else {
        now.day_number
    };
    for (entity, k) in &existing {
        if !out || k.day != day {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    let Some(items) = items else { return };
    let Some(at) = crate::joan::spot(&town, &items, day) else {
        return;
    };
    commands.spawn((
        Name::new("Katrina"),
        Katrina { day },
        NpcClip::default(),
        NpcGait::default(),
        NpcModel(KATRINA_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y),
    ));
    // `fortune_given` belongs to the actor: a new visit starts without one.
    flow.fortune_given = false;
    info!("Katrina is out at {at:?}");
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// `0x970` is up with its Yes / No question.
    CallIn,
    /// `0x973` is up: the fee and the fortune follow.
    Deciding,
    /// `0x97B` is up: the saved destiny's message follows.
    Remind,
    /// A closing line is up; the talk ends with it.
    Close,
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
    /// `EV_GYPSY_ACTOR.fortune_given`.
    fortune_given: bool,
}

fn context(identity: &crate::save::PlayerIdentity, words: [String; 4]) -> MsgContext {
    let now = crate::clock::now();
    let [adjective, noun, verb, place] = words;
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Katrina".into(),
        free: vec![String::new(); 20],
        items: vec![adjective, noun, verb, place, String::new()],
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

/// Shows disc message `msg` in Katrina's voice; true when it ends on a question.
fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &MsgContext,
    msg: u16,
) -> bool {
    let Some(text) = disc.text.as_ref() else {
        return false;
    };
    let (pages, asked) = text.talk_asking(usize::from(msg), ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "Katrina",
        (KATRINA_NPC, KATRINA_SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Katrina message {msg:#06x}");
    asked.is_some()
}

/// `aEGPS_set_string`: the four random words of the fortune (adjective, noun, verb, place).
fn fortune_words(disc: &GameDisc, rand: &mut crate::game_rand::GameRand) -> [String; 4] {
    std::array::from_fn(|kind| {
        let no = native::fortune_string_no(kind as u8, rand.0.random(32) as u8);
        disc.text
            .as_ref()
            .and_then(|t| t.string(usize::from(no)))
            .unwrap_or_default()
    })
}

/// `Game_play_Reset_destiny`: a fortune lasts for the day it was told.
fn reset_destiny(mut identity: ResMut<crate::save::PlayerIdentity>) {
    if identity.destiny != native::destiny::NORMAL
        && identity.destiny_day != crate::clock::now().day_number
    {
        identity.destiny = native::destiny::NORMAL;
    }
}

/// She turns to face the player while they talk.
fn face(
    time: Res<Time>,
    flow: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<Katrina>)>,
    mut katrina: Query<&mut Transform, With<Katrina>>,
) {
    let (Ok(at), Ok(mut transform)) = (player.single(), katrina.single_mut()) else {
        return;
    };
    if matches!(flow.step, Step::Idle) {
        return;
    }
    let to = (at.translation - transform.translation).with_y(0.0);
    if to.length() > 0.01 {
        let want = Quat::from_rotation_y(to.x.atan2(to.z));
        transform.rotation = transform
            .rotation
            .slerp(want, (time.delta_secs() * 8.0).min(1.0));
    }
}

/// The talk, as `aEGPS_set_talk_info`, `aEGPS_call_in` and `aEGPS_decide_result`.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), With<Player>>,
    katrina: Query<(Entity, &Transform), With<Katrina>>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut identity: ResMut<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, actor_at)) = katrina.single() else {
        return;
    };
    // Only her own answers; other talkers in town take theirs.
    let answer = if dialogue.speaker == Some(actor) {
        dialogue.external_result.take()
    } else {
        None
    };
    if let Some(choice) = answer {
        *focus = InputFocus::Dialogue;
        let ctx = context(&identity, Default::default());
        match std::mem::take(&mut flow.step) {
            Step::CallIn => match choice {
                // `aEGPS_call_in`: Yes, with the Bells or without; No.
                0 => {
                    let can_afford = inventory.bells >= native::FORTUNE_PRICE;
                    let msg = native::fortune_callin_msg(true, can_afford);
                    say(&disc, &mut dialogue, actor, &ctx, msg);
                    flow.step = if can_afford {
                        Step::Deciding
                    } else {
                        Step::Close
                    };
                }
                1 => {
                    let msg = native::fortune_callin_msg(false, false);
                    say(&disc, &mut dialogue, actor, &ctx, msg);
                    flow.step = Step::Close;
                }
                _ => {
                    *dialogue = DialogueState::default();
                    *focus = InputFocus::World;
                }
            },
            Step::Deciding => {
                // `aEGPS_decide_result_init`, then `aEGPS_decide_result`.
                inventory.bells = inventory.bells.saturating_sub(native::FORTUNE_PRICE);
                let words = fortune_words(&disc, &mut rand);
                flow.fortune_given = true;
                let roll = rand.0.random(10) as u8;
                identity.destiny = native::fortune_destiny(roll);
                identity.destiny_day = crate::clock::now().day_number;
                info!(
                    "Katrina: fortune roll {roll}, destiny {}, words {words:?}",
                    identity.destiny
                );
                let ctx = context(&identity, words);
                say(
                    &disc,
                    &mut dialogue,
                    actor,
                    &ctx,
                    native::fortune_result_msg(roll),
                );
                flow.step = Step::Close;
            }
            Step::Remind => {
                // `aEGPS_decide_result` with a saved destiny.
                let msg = native::fortune_saved_msg(identity.destiny.max(1));
                say(&disc, &mut dialogue, actor, &ctx, msg);
                flow.step = Step::Close;
            }
            Step::Close | Step::Idle => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
            }
        }
        return;
    }
    if !matches!(flow.step, Step::Idle)
        || !focus.is_world()
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    let delta = (actor_at.translation - at.translation).with_y(0.0);
    if delta.length() > 1.6 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    let ctx = context(&identity, Default::default());
    // `aEGPS_set_talk_info` and `aEGPS_talk_init`.
    let msg = native::fortune_talk_msg(flow.fortune_given, identity.destiny);
    let asked = say(&disc, &mut dialogue, actor, &ctx, msg);
    flow.step = if native::fortune_needs_new(identity.destiny) {
        if asked {
            Step::CallIn
        } else {
            Step::Close
        }
    } else if flow.fortune_given {
        Step::Close
    } else {
        Step::Remind
    };
}

pub struct KatrinaPlugin;

impl Plugin for KatrinaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .add_systems(
                Update,
                visit.after(crate::event_today::refresh).in_set(GameplaySet),
            )
            .add_systems(Update, (face, reset_destiny).in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
