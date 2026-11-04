//! Crazy Redd (`SP_NPC_BROKER`), the dealer with the tent.
//!
//! He is out while the original scheduler says `mEv_EVENT_BROKER_SALE` is active (18:00 on his day until 17:59 the
//! next), on free ground like the other visitors. The event area (`mEv_broker_c`) is rolled when the shop calendar
//! rolls the event (`init_sp_broker`: three different furniture pieces, each from the event list with 50%, else the
//! common list) and starts empty with the next one.
//!
//! Talking outside (`ac_ev_broker_move.c_inc`): `0x788`, `0x789` once the player has been in the tent, `0x78A` after a
//! purchase. Inside (`ac_ev_broker2_move.c_inc`): `0x78B`; each piece is explained (`0x793 + RANDOM(5)` the first time,
//! `0x7A1` after) with its name and price (four times Nook's) and a question: Buy goes on with `0x798 + RANDOM(5)`, then
//! too poor `0x7A0`, no free pocket `0x79F`, else the piece goes into the first free pocket, the price is paid and
//! `0x79E`; No is `0x79D`. Leaving: `0x792` after a purchase, else `0x791`; a sold-out tent says `0x7A2`.
//!
//! Port adaptation: the tent is not built. After the outside talk the stall is a menu of the pieces left (name and
//! price) and Leave, the rest through the original messages; Redd stays outside. The menu's own words are the port's.
//!
//! Not built: the tent and its interior, Redd walking in, money bags as payment, the handbills.

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::inventory::EMPTY_NO;
use serde::{Deserialize, Serialize};

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::messages::{MsgChoice, MsgContext};
use crate::player::Player;
use crate::save::LoadedSave;
use crate::shop::ShopLocation;
use crate::states::GameplaySet;
use crate::town::Town;

/// ALL_NPC_NUM + (SP_NPC_BROKER - SP_NPC_START).
const REDD_NPC: u32 = 238 + 1;
/// `l_sp_actor_name` sound id of `SP_NPC_BROKER`.
const REDD_SOUND: u8 = 2;
/// `mSP_LIST_EVENT`.
const EVENT_LIST: usize = 3;
/// `aEBR2_PRICE_MULT`.
const PRICE_MULT: u32 = 4;
/// `mEv_BROKER_ITEM_NUM`.
const ITEMS: usize = 3;
/// Message base of the tent talk.
const BASE: usize = 0x0788;

/// The event's save area (`mEv_broker_c`) for the one player.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrokerEvent {
    /// `dates[SPECIAL0]` and `dates[SPECIAL1]` of the event this belongs to.
    pub rolled: (u16, u16),
    /// `items[3]` (0: none, which is a sold slot).
    pub items: [u16; ITEMS],
    /// `sold_items[..used]`: what this player bought.
    pub sold_items: Vec<u16>,
    /// The player has been in the tent (`entered_pid`).
    pub entered: bool,
}

impl BrokerEvent {
    fn left(&self) -> Vec<usize> {
        (0..ITEMS).filter(|&i| self.items[i] != 0).collect()
    }
}

#[derive(Component)]
struct Redd {
    /// The game day he came out for.
    day: i64,
}

/// Startup: the saved event area.
fn apply_loaded(loaded: Res<LoadedSave>, mut event: ResMut<BrokerEvent>) {
    if let Some(save) = loaded.0.as_ref() {
        *event = save.broker_event.clone().unwrap_or_default();
    }
}

/// Brings him out for his day and sends him off when the event ends.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    items: Option<Res<crate::field_items::FieldItems>>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Redd)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut flow: ResMut<Talk>,
) {
    let now = crate::clock::now();
    let out = today.is_active(ev::BROKER_SALE, now.hour)
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    // Until 18:00 of the next day he is still the previous day's visitor.
    let day = if now.hour < 18 {
        now.day_number - 1
    } else {
        now.day_number
    };
    for (entity, r) in &existing {
        if !out || r.day != day {
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
        Name::new("Crazy Redd"),
        Redd { day },
        NpcClip::default(),
        NpcGait::default(),
        NpcModel(REDD_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y),
    ));
    // `broker2->explain_flag` and `sell_flag` belong to the actor.
    flow.explained = 0;
    flow.sold_now = false;
    info!("Crazy Redd is out at {at:?}");
}

/// `init_sp_broker`: a new ROLLED event gets its three pieces.
fn reset_for_new_event(
    shop: Option<Res<crate::shop::ShopData>>,
    mut event: ResMut<BrokerEvent>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some((ev::BROKER_SALE, slots)) = shop.and_then(|s| s.events.special_event()) else {
        return;
    };
    let rolled = (slots[0], slots[1]);
    if event.rolled == rolled {
        return;
    }
    let events = crate::goods::list(crate::goods::Kind::Furniture, EVENT_LIST);
    let mut items = [0u16; ITEMS];
    for i in 0..ITEMS {
        let taken: Vec<u16> = items[..i].to_vec();
        // `RANDOM_F(100.0f) <= event_chance` (50 at the shop's goods power of 0): the event list, else the common list.
        let from_event = rand.0.fqrand() * 100.0 <= 50.0;
        let fresh: Vec<u16> = events
            .iter()
            .copied()
            .filter(|p| !taken.contains(p))
            .collect();
        items[i] = if from_event && !fresh.is_empty() {
            fresh[rand.0.random(fresh.len() as i32) as usize]
        } else {
            crate::goods::select_common(crate::goods::Kind::Furniture, &taken, &mut |n| {
                rand.0.random(n as i32) as u32
            })
            .unwrap_or(0)
        };
    }
    info!("Redd's pieces for this event: {items:#06x?}");
    *event = BrokerEvent {
        rolled,
        items,
        ..default()
    };
}

#[derive(Clone, Copy, Debug)]
enum After {
    Close,
    /// Redd's `0x78B` follows, then the stall.
    Hello,
    /// The stall menu.
    Menu,
    /// The purchase of this slot.
    Buy(usize),
    /// The goodbye line.
    Goodbye,
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// A line is up.
    Said(After),
    /// The menu of the slots left is up.
    Menu(Vec<usize>),
    /// A piece's explanation with its question (the slot).
    Explain(usize),
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
    /// `broker2->explain_flag`.
    explained: u8,
    /// `broker2->sell_flag`.
    sold_now: bool,
}

fn context(
    disc: &GameDisc,
    identity: &crate::save::PlayerIdentity,
    event: &BrokerEvent,
    slot: Option<usize>,
) -> MsgContext {
    let now = crate::clock::now();
    let mut items = vec![String::new(); 5];
    let mut articles = vec![0u8; 5];
    let mut free = vec![String::new(); 20];
    if let Some(slot) = slot {
        let item = event.items[slot];
        items[0] = crate::items::name(item);
        articles[0] = crate::shop_nook::item_article(disc, item);
        free[0] = (crate::items::price(item).unwrap_or(0) * PRICE_MULT).to_string();
    }
    // `mMsg_ITEM_STR2`: what the player bought.
    if let Some(&bought) = event.sold_items.first() {
        items[2] = crate::items::name(bought);
        articles[2] = crate::shop_nook::item_article(disc, bought);
    }
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Crazy Redd".into(),
        free,
        items,
        item_articles: articles,
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

/// Shows disc message `msg` (absolute) in Redd's voice.
fn say_abs(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &MsgContext,
    msg: usize,
) -> Option<MsgChoice> {
    let text = disc.text.as_ref()?;
    let (pages, asked) = text.talk_asking(msg, ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "Crazy Redd",
        (REDD_NPC, REDD_SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Redd message {msg:#06x}");
    asked.map(|(_, c)| c)
}

/// He turns to face the player while they talk.
fn face(
    time: Res<Time>,
    flow: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<Redd>)>,
    mut redd: Query<&mut Transform, With<Redd>>,
) {
    let (Ok(at), Ok(mut transform)) = (player.single(), redd.single_mut()) else {
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

/// The talk and the stall.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), With<Player>>,
    redd: Query<(Entity, &Transform), With<Redd>>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut event: ResMut<BrokerEvent>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, actor_at)) = redd.single() else {
        return;
    };
    let end = |dialogue: &mut DialogueState, focus: &mut InputFocus, flow: &mut Talk| {
        *dialogue = DialogueState::default();
        *focus = InputFocus::World;
        flow.step = Step::Idle;
    };

    // Only his own answers; other talkers in town take theirs.
    let answer = if dialogue.speaker == Some(actor) {
        dialogue.external_result.take()
    } else {
        None
    };
    if let Some(choice) = answer {
        *focus = InputFocus::Dialogue;
        let ctx = context(&disc, &identity, &event, None);
        // The menu of what is left.
        let show_menu = |dialogue: &mut DialogueState, flow: &mut Talk, event: &BrokerEvent| {
            let slots = event.left();
            let labels: Vec<String> = slots
                .iter()
                .map(|&s| {
                    let item = event.items[s];
                    format!(
                        "{} - {} Bells",
                        crate::items::name(item),
                        crate::items::price(item).unwrap_or(0) * PRICE_MULT
                    )
                })
                .chain(["Leave".to_string()])
                .collect();
            dialogue.begin_special(
                actor,
                "Crazy Redd",
                (REDD_NPC, REDD_SOUND),
                vec!["Which one?".to_string()],
                Some(labels),
            );
            flow.step = Step::Menu(slots);
        };
        match std::mem::take(&mut flow.step) {
            Step::Said(After::Hello) => {
                // The player walks in: `aEBR2_say_hello_init` records them (`entered_pid`), then `0x78B`.
                event.entered = true;
                say_abs(&disc, &mut dialogue, actor, &ctx, BASE + 3);
                flow.step = Step::Said(After::Menu);
            }
            Step::Said(After::Menu) => {
                if event.left().is_empty() {
                    // `save_p->used == mEv_BROKER_ITEM_NUM`.
                    say_abs(&disc, &mut dialogue, actor, &ctx, 0x07A2);
                    flow.step = Step::Said(After::Goodbye);
                } else {
                    show_menu(&mut dialogue, &mut flow, &event);
                }
            }
            Step::Said(After::Goodbye) => {
                // `aEBR2_set_talk_info_goodbye_wait`.
                let m = if flow.sold_now { 0x0792 } else { 0x0791 };
                say_abs(&disc, &mut dialogue, actor, &ctx, m);
                flow.step = Step::Said(After::Close);
            }
            Step::Menu(slots) => {
                if let Some(&slot) = slots.get(choice) {
                    // The piece's explanation (`aEBR2_set_talk_info_sell_item`).
                    let ctx = context(&disc, &identity, &event, Some(slot));
                    let bit = 1u8 << slot;
                    let msg = if flow.explained & bit != 0 {
                        0x07A1
                    } else {
                        flow.explained |= bit;
                        0x0793 + rand.0.random(5) as usize
                    };
                    let asked = say_abs(&disc, &mut dialogue, actor, &ctx, msg);
                    flow.step = if asked.is_some() {
                        Step::Explain(slot)
                    } else {
                        Step::Said(After::Menu)
                    };
                } else {
                    // Leave.
                    let m = if flow.sold_now { 0x0792 } else { 0x0791 };
                    say_abs(&disc, &mut dialogue, actor, &ctx, m);
                    flow.step = Step::Said(After::Close);
                }
            }
            Step::Explain(slot) => {
                let ctx = context(&disc, &identity, &event, Some(slot));
                if choice == 0 {
                    // `aEBR2_sell_check`: Buy.
                    say_abs(
                        &disc,
                        &mut dialogue,
                        actor,
                        &ctx,
                        0x0798 + rand.0.random(5) as usize,
                    );
                    flow.step = Step::Said(After::Buy(slot));
                } else {
                    say_abs(&disc, &mut dialogue, actor, &ctx, 0x079D);
                    flow.step = Step::Said(After::Menu);
                }
            }
            Step::Said(After::Buy(slot)) => {
                // `aEBR2_sell_after`.
                let item = event.items[slot];
                let price = crate::items::price(item).unwrap_or(0) * PRICE_MULT;
                let ctx = context(&disc, &identity, &event, Some(slot));
                let msg = if inventory.bells < price {
                    0x07A0
                } else if !inventory.pockets.pockets.contains(&EMPTY_NO) {
                    0x079F
                } else {
                    inventory.add(item);
                    inventory.bells -= price;
                    event.items[slot] = 0;
                    event.sold_items.push(item);
                    flow.sold_now = true;
                    info!("Redd sells {item:#06x} for {price} Bells");
                    0x079E
                };
                say_abs(&disc, &mut dialogue, actor, &ctx, msg);
                flow.step = Step::Said(After::Menu);
            }
            Step::Said(After::Close) | Step::Idle => end(&mut dialogue, &mut focus, &mut flow),
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
    // `aEBRK_set_talk_info_talk_request`.
    let msg = if !event.sold_items.is_empty() {
        0x078A
    } else if event.entered {
        0x0789
    } else {
        0x0788
    };
    let ctx = context(&disc, &identity, &event, None);
    say_abs(&disc, &mut dialogue, actor, &ctx, msg);
    flow.step = Step::Said(After::Hello);
}

pub struct ReddPlugin;

impl Plugin for ReddPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .init_resource::<BrokerEvent>()
            .add_systems(Startup, apply_loaded)
            .add_systems(
                Update,
                (
                    reset_for_new_event,
                    visit.after(crate::event_today::refresh),
                )
                    .chain()
                    .in_set(GameplaySet),
            )
            .add_systems(Update, face.in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
