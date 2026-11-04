//! Jingle (`SP_NPC_SANTA`), the Toy Day reindeer (`ac_ev_santa.c`, `ac_ev_santa_move.c_inc`).
//!
//! He is out while the original scheduler says `mEv_EVENT_TOY_DAY_JINGLE` is active (December 24, 20:00 until
//! midnight) and stands on the beach like Gulliver (`christmas_in` shows him at the wade). After every talk he walks
//! to another beach spot (`christmas_behind` / `walk_actor_at_wade`); the port moves him there directly.
//!
//! Talking, for the player he has not given a present yet: a talk in the acre of the last talk gets
//! `0x7BC + RANDOM(3)`; otherwise the talk counter picks the line: hello `0x7AA`, first wish question `0x7AB`,
//! second `0x7AE`, final question `0x7B2 + wish` (wish bits: the first answer is bit 0, the second bit 1, the final
//! answer picks furniture / clothes or wallpaper / carpet), and with the counter at 4 the present: `0x7BA`, a piece
//! of the Christmas list of the wished kind goes into the pockets (`0x7B9` when none is free). After the present
//! he does not know the player: a shirt he has not seen starts the check again (`0x2B4C`: the choices lead to the
//! wish messages `0x2B54..0x2B57`, `ALMOST_RECOGNIZE`), the same shirt twice in a row gets a second present
//! (`0x2B59`, `0x2B58` when the pockets are full), a shirt he has already given for gets `0x7BF + RANDOM(3)`, and ten
//! presents end it (`0x2B5B + RANDOM(3)`).
//!
//! The original compares the acre of the *player* with the last talk's (its own `@BUG`, kept: retail behaviour).
//!
//! Not built: his wandering between talks, the border-acre rule, the present animation (the hand-over plays instead),
//! other players (`0x7C2..0x7C4`; the port has one player).

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use serde::{Deserialize, Serialize};

use crate::character::{NpcClip, NpcGait, NpcModel, PlayerLook};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::goods::Kind;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::messages::MsgContext;
use crate::player::Player;
use crate::save::LoadedSave;
use crate::shop::ShopLocation;
use crate::shop_nook::handover::{HandOver, Item, Party, Request};
use crate::states::GameplaySet;
use crate::town::{Town, ACRE_SIZE};
use rustimal_logic::inventory::EMPTY_NO;
use rustimal_logic::town_gen::{ACRE_DEPTH, ACRE_WIDTH};

/// ALL_NPC_NUM + (SP_NPC_SANTA - SP_NPC_START).
const JINGLE_NPC: u32 = 238 + 15;
/// `l_sp_actor_name` sound id of `SP_NPC_SANTA`.
const JINGLE_SOUND: u8 = 2;
/// `mSP_LIST_CHRISTMAS`.
const CHRISTMAS_LIST: usize = 8;
/// `mEv_SANTA_CLOTH_NUM_MAX`.
const CLOTH_MAX: usize = 10;

/// `aESNT_WISH_*`: furniture, clothes, wallpaper, carpet.
const WISH_KINDS: [Kind; 4] = [Kind::Furniture, Kind::Cloth, Kind::Wallpaper, Kind::Carpet];

/// The event's save area (`mEv_santa_event_c` and `mEv_santa_event_common_c`) for the one player.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JingleEvent {
    /// The game day this belongs to (the event lasts one evening).
    pub day: i64,
    /// A present has been given (`pid` is no longer null).
    pub given: bool,
    /// `present_count`.
    pub present_count: u8,
    /// `cloth[..present_count]`: the shirts he has given for.
    pub cloths: Vec<u16>,
    /// `present`: the wished kind.
    pub present: u8,
    /// `talk_counter`.
    pub talk_counter: u8,
    /// The acre of the last talk (`bx`, `bz`).
    pub block: (i32, i32),
    /// `last_talk_cloth` (a shirt index plus one; 0 is none).
    pub last_cloth: u16,
}

#[derive(Component)]
struct Jingle {
    /// The game day he came out for.
    day: i64,
    /// Talks so far, to pick the next spot.
    moves: i64,
}

/// Startup: the saved event area.
fn apply_loaded(loaded: Res<LoadedSave>, mut event: ResMut<JingleEvent>) {
    if let Some(save) = loaded.0.as_ref() {
        *event = save.jingle_event.clone().unwrap_or_default();
    }
}

/// Brings him out for the evening and sends him off when the event ends.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Jingle)>,
    mut event: ResMut<JingleEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let out = today.is_active(ev::TOY_DAY_JINGLE, now.hour)
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    for (entity, j) in &existing {
        if !out || j.day != now.day_number {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    // A new evening starts a new save area (`mEv_reserve_save_area`).
    if event.day != now.day_number {
        *event = JingleEvent {
            day: now.day_number,
            ..default()
        };
    }
    let Some(at) = crate::gulliver::beach_spot(&town, now.day_number ^ 0x4a49) else {
        return;
    };
    commands.spawn((
        Name::new("Jingle"),
        Jingle {
            day: now.day_number,
            moves: 0,
        },
        NpcClip::default(),
        NpcGait::default(),
        NpcModel(JINGLE_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y),
    ));
    info!("Jingle is out at {at:?}");
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// A question is up; the choices' follow-up messages are kept.
    Ask(Question, Vec<Option<usize>>),
    /// A line is up; the talk ends with it.
    Close,
    /// The present's line is up; the piece changes hands when it ends.
    Giving(u16),
    /// The piece is crossing; the talk ends after.
    Handing,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Question {
    /// `aESNT_chk_wish_1st` (bit 0) and the second (bit 1).
    First,
    Second,
    /// `aESNT_chk_wish_final`.
    Final,
    /// `aESNT_chk_wish_more` (the message the choice leads to is the wish).
    More,
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
}

fn context(identity: &crate::save::PlayerIdentity, item: Option<u16>) -> MsgContext {
    let now = crate::clock::now();
    let name = item.map(crate::items::name).unwrap_or_default();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Jingle".into(),
        free: vec![String::new(); 20],
        items: vec![name; 5],
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

/// Shows disc message `msg` in Jingle's voice; the choices' follow-ups come back when it asks.
fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &MsgContext,
    msg: u16,
) -> Option<Vec<Option<usize>>> {
    let text = disc.text.as_ref()?;
    let (pages, asked) = text.talk_asking(usize::from(msg), ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "Jingle",
        (JINGLE_NPC, JINGLE_SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Jingle message {msg:#06x}");
    asked.map(|(_, c)| c.next)
}

/// He turns to face the player while they talk.
fn face(
    time: Res<Time>,
    flow: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<Jingle>)>,
    mut jingle: Query<&mut Transform, With<Jingle>>,
) {
    let (Ok(at), Ok(mut transform)) = (player.single(), jingle.single_mut()) else {
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

/// The acre a world position is in (`play->block_table`).
fn acre_of(at: Vec3) -> (i32, i32) {
    let ox = ACRE_WIDTH as f32 * ACRE_SIZE / 2.0;
    let oz = ACRE_DEPTH as f32 * ACRE_SIZE / 2.0;
    (
        ((at.x + ox) / ACRE_SIZE).floor() as i32,
        ((at.z + oz) / ACRE_SIZE).floor() as i32,
    )
}

/// What the talk shows, decided by `aESNT_getP_talk_data`.
enum Line {
    /// A plain line.
    Plain(u16),
    /// A question line.
    Question(u16, Question),
    /// The present line, with the piece (already in the pockets).
    Present(u16, u16),
}

/// `aESNT_before_talk_present_decide_talk_data_idx`; `second` picks the `*2` lines.
fn present_line(
    event: &mut JingleEvent,
    inventory: &mut PlayerInventory,
    rand: &mut crate::game_rand::GameRand,
    cloth: u16,
    second: bool,
) -> Line {
    let (yes, no) = if second {
        (0x2B59, 0x2B58)
    } else {
        (0x07BA, 0x07B9)
    };
    if !inventory.pockets.pockets.contains(&EMPTY_NO) {
        return Line::Plain(no);
    }
    if event.present >= 4 {
        event.present = 3;
    }
    let list = crate::goods::list(WISH_KINDS[usize::from(event.present)], CHRISTMAS_LIST);
    if list.is_empty() {
        return Line::Plain(no);
    }
    let item = list[rand.0.random(list.len() as i32) as usize];
    if !inventory.add(item) {
        return Line::Plain(no);
    }
    event.given = true;
    event.talk_counter = 0;
    event.cloths.push(cloth);
    event.present_count += 1;
    Line::Present(yes, item)
}

/// `aESNT_getP_talk_data`.
fn decide(
    event: &mut JingleEvent,
    inventory: &mut PlayerInventory,
    rand: &mut crate::game_rand::GameRand,
    block: (i32, i32),
    cloth: u16,
) -> Line {
    if !event.given {
        let line = if block == event.block {
            Line::Plain(0x07BC + rand.0.random(3) as u16)
        } else if event.talk_counter >= 4 {
            present_line(event, inventory, rand, cloth, false)
        } else {
            let line = match event.talk_counter {
                0 => Line::Plain(0x07AA),
                1 => Line::Question(0x07AB, Question::First),
                2 => Line::Question(0x07AE, Question::Second),
                _ => Line::Question(0x07B2 + u16::from(event.present.min(3)), Question::Final),
            };
            event.talk_counter += 1;
            line
        };
        // The acre is kept unless the pockets were full (`aESNT_TALK_FINAL_NO_PRESENT`).
        if !matches!(line, Line::Plain(0x07B9)) {
            event.block = block;
        }
        return line;
    }
    // After a present: the same player.
    if event.cloths.contains(&cloth) {
        event.talk_counter = 0;
        return Line::Plain(0x07BF + rand.0.random(3) as u16);
    }
    if usize::from(event.present_count) >= CLOTH_MAX {
        return Line::Plain(0x2B5B + rand.0.random(3) as u16);
    }
    let recognise = |event: &mut JingleEvent| {
        event.last_cloth = cloth + 1;
        event.talk_counter += 1;
        Line::Question(0x2B4C, Question::More)
    };
    if event.talk_counter == 0 {
        recognise(event)
    } else if event.last_cloth == cloth + 1 {
        present_line(event, inventory, rand, cloth, true)
    } else {
        let line = recognise(event);
        event.talk_counter = 1;
        line
    }
}

/// The talk, as `aESNT_talk_request` and the talk procs.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    town: Res<Town>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), (With<Player>, Without<Jingle>)>,
    mut jingle: Query<(Entity, &mut Transform, &mut Jingle)>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    identity: Res<crate::save::PlayerIdentity>,
    look: Res<PlayerLook>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut event: ResMut<JingleEvent>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, mut actor_at, mut me)) = jingle.single_mut() else {
        return;
    };
    let ctx = context(&identity, None);
    let answered = if dialogue.speaker == Some(actor) {
        dialogue.external_result.take()
    } else {
        None
    };
    if let Some(choice) = answered {
        *focus = InputFocus::Dialogue;
        match std::mem::take(&mut flow.step) {
            Step::Ask(question, next) => {
                let wish = |pick: usize| next.get(pick).copied().flatten();
                let chosen = if choice == usize::MAX { 0 } else { choice };
                let mut follow = wish(chosen);
                match question {
                    Question::First => event.present |= u8::from(chosen != 0),
                    Question::Second => event.present |= u8::from(chosen != 0) << 1,
                    Question::Final => {
                        event.present = if chosen == 0 {
                            event.present & 1
                        } else if event.present & 2 != 0 {
                            2
                        } else {
                            3
                        };
                    }
                    Question::More => {
                        // The wish message the choice leads to names the kind.
                        event.present = match follow {
                            Some(0x2B55) => 2,
                            Some(0x2B56) => 3,
                            Some(0x2B57) => 1,
                            _ => 0,
                        };
                        follow = follow.or(Some(0x2B54));
                    }
                }
                match follow {
                    Some(msg) => {
                        say(&disc, &mut dialogue, actor, &ctx, msg as u16);
                        flow.step = Step::Close;
                    }
                    None => {
                        *dialogue = DialogueState::default();
                        *focus = InputFocus::World;
                        relocate(&town, &mut actor_at, &mut me);
                    }
                }
            }
            Step::Giving(item) => {
                *dialogue = DialogueState::default();
                handover.start(Request {
                    item: Item::Goods(item),
                    giver: Party::Npc(actor),
                    taker: Party::Player,
                });
                flow.step = Step::Handing;
            }
            Step::Close | Step::Handing | Step::Idle => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
                relocate(&town, &mut actor_at, &mut me);
            }
        }
        return;
    }
    if matches!(flow.step, Step::Handing) {
        if handover.busy() {
            *focus = InputFocus::Dialogue;
        } else {
            *dialogue = DialogueState::default();
            *focus = InputFocus::World;
            flow.step = Step::Idle;
            relocate(&town, &mut actor_at, &mut me);
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
    let cloth = look.cloth as u16;
    let line = decide(
        &mut event,
        &mut inventory,
        &mut rand,
        acre_of(at.translation),
        cloth,
    );
    match line {
        Line::Plain(msg) => {
            say(&disc, &mut dialogue, actor, &ctx, msg);
            flow.step = Step::Close;
        }
        Line::Question(msg, question) => {
            flow.step = match say(&disc, &mut dialogue, actor, &ctx, msg) {
                Some(next) => Step::Ask(question, next),
                None => Step::Close,
            };
        }
        Line::Present(msg, item) => {
            let ctx = context(&identity, Some(item));
            say(&disc, &mut dialogue, actor, &ctx, msg);
            info!("Jingle gives {item:#06x}");
            flow.step = Step::Giving(item);
        }
    }
}

/// `walk_actor_at_wade`: he goes to another beach spot after a talk.
fn relocate(town: &Town, at: &mut Transform, me: &mut Jingle) {
    me.moves += 1;
    let here = acre_of(at.translation);
    for step in 0..4 {
        let seed = (me.day ^ 0x4a49).wrapping_add((me.moves + step) * 7919);
        if let Some(spot) = crate::gulliver::beach_spot(town, seed) {
            if acre_of(spot) != here || step == 3 {
                at.translation = spot + Vec3::Y;
                info!("Jingle walks to {spot:?}");
                return;
            }
        }
    }
}

pub struct JinglePlugin;

impl Plugin for JinglePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .init_resource::<JingleEvent>()
            .add_systems(Startup, apply_loaded)
            .add_systems(
                Update,
                visit.after(crate::event_today::refresh).in_set(GameplaySet),
            )
            .add_systems(Update, face.in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
