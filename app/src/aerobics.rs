//! The radio calisthenics at the shrine (`ac_taisou_npc0.c`, `ac_taisou_npc0_talk.c_inc`, `m_soncho.c`).
//!
//! Two events share the line-up (`l_rc_map`, `l_fdd_radio_map`): the morning aerobics (`mEv_EVENT_MORNING_AEROBICS`,
//! July 25 to August 31 at 06:00) and the sports fair's (`mEv_EVENT_SPORTS_FAIR_AEROBICS`, 09:00-10:59 on the equinox
//! days). In the shrine block four villagers stand at units (4,10) (6,10) (8,10) (10,10) facing south, Copper
//! (`SP_NPC_EV_TAISOU_0`) at (7,12) facing north, and Tortimer (`SP_NPC_SONCHO_D078`) at (9,8). Their talk
//! (`aTS0_set_talk_info`): Copper `0x2665` (morning) / `0x2668` (sports fair) `+ RANDOM(3)`, a villager
//! `msg_base[event][looks] + RANDOM(2) + 3 * place` (place 1 to 4).
//!
//! The sports fair's Tortimer is `soncho.rs`'s (the commemorative medal). The morning aerobics' is the radio card
//! (`mSC_Radio_Set_Talk_Proc`): 13 mornings stamp a card (`radiocard`, kept with the player), the last gives the radio
//! (`FTR_RADIO_TEST`). Talking: the first morning of a card `0x3422` (`0x3423` with the calendar mark) and a card;
//! each later new morning the stamp `0x3427`; on the thirteenth `0x342A` and the radio; the same morning `0x3433` and
//! a closing line `0x3434..=0x3436`; a missed morning (`mSCR_ACTION_LOST_CARD`) `0x342E` / `0x342F` and the card
//! advances anyway; after 18 August without a card `0x3437` and `0x3438..=0x343A`; two cards in the pockets `0x343C`
//! (they are taken). Free pockets are needed for a new card (`0x3424` otherwise).
//!
//! Not built: the radio music and its sync, the exercise animations (they stand), the failures in the routine
//! (`gymnastic_state` 2/3) and Tortimer's tiredness, the radio set, the player's own exercises.

use std::collections::VecDeque;

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::inventory::EMPTY_NO;

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
use crate::shop_nook::handover::{HandOver, Item, Party, Request};
use crate::states::GameplaySet;
use crate::town::Town;

/// ALL_NPC_NUM + (SP_NPC_EV_TAISOU_0 - SP_NPC_START): Copper.
const COPPER_NPC: u32 = 238 + 78;
/// ALL_NPC_NUM + (SP_NPC_SONCHO_D078 - SP_NPC_START): Tortimer in the line.
const LEADER_NPC: u32 = 238 + 120;
const SOUND: u8 = 2;
/// `ITM_EXCERCISE_CARD00` (13 cards) and `FTR_RADIO_TEST`.
const CARD00: u16 = 0x2523;
const RADIO: u16 = 0x1FCC;
const CARDS: u16 = 13;
/// `mSC_RADIO_DAYS`.
const RADIO_DAYS: u8 = 13;
/// `mSC_SPECIAL_EVENT_MORNING_AEROBICS`: the calendar's event number of the morning aerobics.
const MORNING_EVENT: u8 = 103;
/// The villagers' units, Copper's and Tortimer's (`l_rc_map`).
const VILLAGER_UNITS: [(usize, usize); 4] = [(4, 10), (6, 10), (8, 10), (10, 10)];
const COPPER_UNIT: (usize, usize) = (7, 12);
pub(crate) const LEADER_UNIT: (usize, usize) = (9, 8);
/// `msg_base`: by event (morning, sports fair) and looks.
const MSG_BASE: [[u16; 6]; 2] = [
    [0x1A3C, 0x1A4B, 0x1A2D, 0x1A5A, 0x1A69, 0x1A78],
    [0x1A96, 0x1AA5, 0x1A87, 0x1AB4, 0x1AC3, 0x1AD2],
];
const COPPER_MSG: [u16; 2] = [0x2665, 0x2668];

#[derive(Component)]
struct Line {
    day: i64,
    morning: bool,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Role {
    Copper,
    /// A villager at place 1..=4 with their looks.
    Villager(u8, u8),
    /// Tortimer in the morning line.
    Leader,
}

/// What the morning leader's talk goes through after the first line.
#[derive(Clone, Copy, Debug)]
enum Seq {
    Say(u16),
    /// A card or the radio crosses to the player (it is already in the pockets).
    Give(u16),
}

#[derive(Resource, Default)]
struct Talk {
    queue: VecDeque<Seq>,
    active: bool,
    handing: bool,
}

fn morning(today: &TodayEvents, hour: u8) -> bool {
    today.is_active(ev::MORNING_AEROBICS, hour)
}

fn sports_fair(today: &TodayEvents, hour: u8) -> bool {
    today.is_active(ev::SPORTS_FAIR_AEROBICS, hour)
}

/// Brings the line to the shrine for the morning aerobics or the sports fair's.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    roster: Option<Res<TownRoster>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Line)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let is_morning = morning(&today, now.hour);
    let on = is_morning || sports_fair(&today, now.hour);
    let out =
        on && *state.get() == ShopLocation::Town && *house.get() == crate::house::HouseState::Town;
    for (entity, l) in &existing {
        if !out || l.day != now.day_number || l.morning != is_morning {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    let mut spawn = |role: Role, model: u32, unit: (usize, usize), yaw: f32, name: &str| {
        let Some(at) = crate::tortimer::shrine_cell(&town, unit.0, unit.1) else {
            return;
        };
        commands.spawn((
            Name::new(name.to_string()),
            Line {
                day: now.day_number,
                morning: is_morning,
            },
            role,
            NpcClip::default(),
            NpcGait::default(),
            crate::event_body::EventBody::default(),
            crate::event_body::Idle("cKF_ba_r_npc_1_taisou1"),
            NpcModel(model),
            Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
            Transform::from_translation(at + Vec3::Y).with_rotation(Quat::from_rotation_y(yaw)),
        ));
    };
    spawn(
        Role::Copper,
        COPPER_NPC,
        COPPER_UNIT,
        std::f32::consts::PI,
        "Copper (aerobics)",
    );
    if let Some(roster) = roster {
        for (place, (entry, unit)) in roster.residents.iter().zip(VILLAGER_UNITS).enumerate() {
            spawn(
                Role::Villager(place as u8 + 1, entry.looks),
                entry.index,
                unit,
                0.0,
                &format!("{} (aerobics)", entry.name),
            );
        }
    }
    // The sports fair's Tortimer is the commemorative one (`soncho.rs`).
    if is_morning {
        spawn(
            Role::Leader,
            LEADER_NPC,
            LEADER_UNIT,
            0.0,
            "Tortimer (aerobics)",
        );
    }
    info!(
        "The {} aerobics line stands at the shrine",
        if is_morning { "morning" } else { "sports fair" }
    );
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
    ctx: &MsgContext,
    (name, model): (&'static str, u32),
    msg: u16,
) {
    let Some(text) = disc.text.as_ref() else {
        return;
    };
    let (pages, asked) = text.talk_asking(usize::from(msg), ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        name,
        (model, SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("{name} message {msg:#06x}");
}

/// `mSC_Radio_time_check` for today: `0` less (or another year), `1` the same day, `2` over.
fn time_check(identity: &crate::save::PlayerIdentity) -> u8 {
    let now = crate::clock::now();
    let ((year, month, day), _) = identity.radio_card;
    if now.year != year {
        return 0;
    }
    match (now.month, now.day).cmp(&(month, day)) {
        std::cmp::Ordering::Less => 0,
        std::cmp::Ordering::Equal => 1,
        std::cmp::Ordering::Greater => 2,
    }
}

fn cards_held(inventory: &PlayerInventory) -> usize {
    inventory
        .pockets
        .pockets
        .iter()
        .filter(|&&i| (CARD00..CARD00 + CARDS).contains(&i))
        .count()
}

fn first_card(inventory: &PlayerInventory) -> u16 {
    inventory
        .pockets
        .pockets
        .iter()
        .copied()
        .find(|i| (CARD00..CARD00 + CARDS).contains(i))
        .unwrap_or(CARD00)
}

/// `mSC_Radio_stamp_card`.
fn stamp(identity: &mut crate::save::PlayerIdentity, days: u8) {
    let now = crate::clock::now();
    identity.radio_card = ((now.year, now.month, now.day), days);
}

/// `mSC_Radio_Set_Talk_Proc` for the morning aerobics: the first line and what follows it.
fn card_talk(
    identity: &mut crate::save::PlayerIdentity,
    inventory: &mut PlayerInventory,
    ctx: &mut MsgContext,
    disc: &GameDisc,
    rand: &mut crate::game_rand::GameRand,
) -> (u16, Vec<Seq>) {
    let now = crate::clock::now();
    let marked = identity
        .soncho_calendar
        .contains(&(MORNING_EVENT, now.year, now.month, now.day));
    let time = time_check(identity);
    let held = cards_held(inventory);
    let free_slot = inventory.pockets.pockets.contains(&EMPTY_NO);
    let days = identity.radio_card.1;
    let number = |ctx: &mut MsgContext, n: u8| ctx.free[0] = n.to_string();
    let radio_name = |ctx: &mut MsgContext| ctx.free[1] = crate::items::name(RADIO);
    let _ = disc;
    if time == 1 {
        // The same morning: `0x3433`, then `0x3434 + RANDOM(3)`.
        return (0x3433, vec![Seq::Say(0x3434 + rand.0.random(3) as u16)]);
    }
    #[derive(PartialEq)]
    enum Act {
        TooLate,
        Delete,
        New,
        Lost,
        Stamp,
        Finish,
        LostFinish,
    }
    let limit_reached = now.month >= 8 && now.day >= 19;
    let act = if held == 0 {
        if limit_reached && (days == RADIO_DAYS || time == 0) {
            Act::TooLate
        } else {
            match time {
                0 => Act::New,
                _ => {
                    if days == RADIO_DAYS - 1 {
                        Act::LostFinish
                    } else if days < RADIO_DAYS {
                        Act::Lost
                    } else {
                        Act::New
                    }
                }
            }
        }
    } else if held > 1 {
        Act::Delete
    } else {
        let card = first_card(inventory);
        match time {
            0 => Act::Delete,
            _ => {
                identity.radio_card.1 = (card - CARD00) as u8 + 1;
                if identity.radio_card.1 >= RADIO_DAYS {
                    Act::Finish
                } else {
                    Act::Stamp
                }
            }
        }
    };
    match act {
        Act::TooLate => (0x3437, vec![Seq::Say(0x3438 + rand.0.random(3) as u16)]),
        Act::Delete => {
            for slot in 0..inventory.pockets.pockets.len() {
                if (CARD00..CARD00 + CARDS).contains(&inventory.pockets.pockets[slot]) {
                    inventory.pockets.pockets[slot] = EMPTY_NO;
                    inventory.pockets.set_item_cond(slot, 0);
                }
            }
            (0x343C, vec![])
        }
        Act::New => {
            let first = if marked { 0x3423 } else { 0x3422 };
            if !free_slot {
                (first, vec![Seq::Say(0x3424)])
            } else {
                inventory.add(CARD00);
                stamp(identity, 0);
                (
                    first,
                    vec![Seq::Say(0x3425), Seq::Give(CARD00), Seq::Say(0x3426)],
                )
            }
        }
        Act::Lost => {
            let first = if marked { 0x342F } else { 0x342E };
            number(ctx, days + 2);
            if !free_slot {
                (first, vec![Seq::Say(0x3430)])
            } else {
                identity.radio_card.1 = days + 1;
                let card = CARD00 + u16::from(days + 1);
                inventory.add(card);
                stamp(identity, days + 1);
                (
                    first,
                    vec![Seq::Say(0x3431), Seq::Give(card), Seq::Say(0x3432)],
                )
            }
        }
        Act::Stamp => {
            let card = first_card(inventory);
            let days = identity.radio_card.1;
            stamp(identity, days);
            if let Some(slot) = inventory.pockets.pockets.iter().position(|&i| i == card) {
                inventory.pockets.pockets[slot] = CARD00 + u16::from(days);
            }
            number(ctx, days + 1);
            (0x3427, vec![Seq::Say(0x3428), Seq::Say(0x3429)])
        }
        Act::Finish => {
            let card = first_card(inventory);
            let days = identity.radio_card.1;
            stamp(identity, days);
            if let Some(slot) = inventory.pockets.pockets.iter().position(|&i| i == card) {
                inventory.pockets.pockets[slot] = RADIO;
            }
            radio_name(ctx);
            (
                0x342A,
                vec![
                    Seq::Say(0x342B),
                    Seq::Say(0x342C),
                    Seq::Give(RADIO),
                    Seq::Say(0x342D),
                ],
            )
        }
        Act::LostFinish => {
            let first = if marked { 0x342F } else { 0x342E };
            number(ctx, days + 2);
            if !free_slot {
                (first, vec![Seq::Say(0x343E)])
            } else {
                identity.radio_card.1 = days + 1;
                inventory.add(RADIO);
                radio_name(ctx);
                stamp(identity, days + 1);
                (
                    first,
                    vec![Seq::Say(0x343D), Seq::Give(RADIO), Seq::Say(0x342D)],
                )
            }
        }
    }
}

/// The talks with the line.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    today: Res<TodayEvents>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), (With<Player>, Without<Line>)>,
    line: Query<(Entity, &Transform, &Role), With<Line>>,
    models: Query<&NpcModel>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    mut identity: ResMut<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let now = crate::clock::now();
    let event = usize::from(!morning(&today, now.hour));
    let leader = line.iter().find(|(_, _, r)| **r == Role::Leader);
    // The leader's script goes on after each line.
    if flow.active {
        let Some((actor, _, _)) = leader else {
            *flow = Talk::default();
            return;
        };
        if flow.handing {
            if handover.busy() {
                *focus = InputFocus::Dialogue;
                return;
            }
            flow.handing = false;
            advance(
                &disc,
                actor,
                &mut flow,
                &mut dialogue,
                &mut focus,
                &mut handover,
                &identity,
            );
            return;
        }
        if dialogue.speaker == Some(actor) && dialogue.external_result.take().is_some() {
            *focus = InputFocus::Dialogue;
            advance(
                &disc,
                actor,
                &mut flow,
                &mut dialogue,
                &mut focus,
                &mut handover,
                &identity,
            );
        }
        return;
    }
    // A single line of anyone else is over when it is read.
    if dialogue.active
        && line.iter().any(|(e, _, _)| dialogue.speaker == Some(e))
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
    let Some((actor, actor_at, role)) = line.iter().find(|(_, t, _)| {
        let delta = (t.translation - at.translation).with_y(0.0);
        delta.length() < 1.8 && facing.dir.dot(delta.normalize_or_zero()) >= 0.5
    }) else {
        return;
    };
    let _ = actor_at;
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    let mut ctx = context(&identity, "Copper");
    match *role {
        Role::Copper => {
            let msg = COPPER_MSG[event] + rand.0.random(3) as u16;
            say(
                &disc,
                &mut dialogue,
                actor,
                &ctx,
                ("Copper", COPPER_NPC),
                msg,
            );
        }
        Role::Villager(place, looks) => {
            ctx.tail =
                crate::dialogue::catchphrase(&disc, models.get(actor).map_or(u32::MAX, |m| m.0));
            let msg = MSG_BASE[event][usize::from(looks).min(5)]
                + rand.0.random(2) as u16
                + u16::from(place) * 3;
            say(
                &disc,
                &mut dialogue,
                actor,
                &ctx,
                ("Villager", COPPER_NPC),
                msg,
            );
        }
        Role::Leader => {
            // `aTS0_talk_init`: the calendar mark is made as the talk starts.
            let mark = (MORNING_EVENT, now.year, now.month, now.day);
            ctx.talk_name = "Tortimer".into();
            let (first, rest) =
                card_talk(&mut identity, &mut inventory, &mut ctx, &disc, &mut rand);
            if !identity.soncho_calendar.contains(&mark) {
                identity.soncho_calendar.push(mark);
            }
            say(
                &disc,
                &mut dialogue,
                actor,
                &ctx,
                ("Tortimer", LEADER_NPC),
                first,
            );
            *flow = Talk {
                queue: rest.into(),
                active: true,
                handing: false,
            };
        }
    }
}

/// The next step of the leader's script.
fn advance(
    disc: &GameDisc,
    actor: Entity,
    flow: &mut Talk,
    dialogue: &mut DialogueState,
    focus: &mut InputFocus,
    handover: &mut HandOver,
    identity: &crate::save::PlayerIdentity,
) {
    match flow.queue.pop_front() {
        Some(Seq::Say(msg)) => {
            let ctx = context(identity, "Tortimer");
            say(disc, dialogue, actor, &ctx, ("Tortimer", LEADER_NPC), msg);
        }
        Some(Seq::Give(item)) => {
            *dialogue = DialogueState::default();
            handover.start(Request {
                item: Item::Goods(item),
                giver: Party::Npc(actor),
                taker: Party::Player,
            });
            flow.handing = true;
        }
        None => {
            *dialogue = DialogueState::default();
            *focus = InputFocus::World;
            *flow = Talk::default();
        }
    }
}

pub struct AerobicsPlugin;

impl Plugin for AerobicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .add_systems(
                Update,
                visit.after(crate::event_today::refresh).in_set(GameplaySet),
            )
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
