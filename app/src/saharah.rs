//! Saharah (`SP_NPC_CARPETPEDDLER`), the carpet and wallpaper peddler.
//!
//! She is out while the original scheduler says `mEv_EVENT_CARPET_PEDDLER` is active (06:00 on her day until 05:59
//! the next morning; `event_schedule` with the special-event calendar), on free ground like Joan and Katrina.
//!
//! Talking (`ac_ev_carpetPeddler.c_inc`, kernel `event_visitors`): `0x48A` offers a random carpet of the event list
//! (named in the message) for `3000 * 2^used` Bells, `used` being the trade-ins of this special event (capped at 4).
//! Two questions: the first No ends it; at the second, No is `+1`, and Yes is `+4` without a carpet in the pockets,
//! `+3` without the Bells, `+5` otherwise and the pockets open. Nothing picked: `+6`. A carpet picked (`ITEM1`
//! category 6): `used` goes up, the pocket takes the new carpet, the price is paid, the old one is handed over, `+7`,
//! she hands over the new one. Any other item: handed over, `+9`, handed back into the first free pocket. The picked
//! item leaves the pockets either way. The event's save (`arabian.used`) is kept as [`CarpetEvent`].
//!
//! Not built: money bags in the pockets as payment (`mSP_money_check` counts them; the wallet only here), the
//! money-bag hand-over visual and the event manager's walk-away after the talk.

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::event_visitors as native;
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
use crate::pockets::PocketsUi;
use crate::save::LoadedSave;
use crate::shop::ShopLocation;
use crate::shop_nook::handover::{HandOver, Item, Party, Request};
use crate::states::GameplaySet;
use crate::town::Town;

/// ALL_NPC_NUM + (SP_NPC_CARPETPEDDLER - SP_NPC_START).
const SAHARAH_NPC: u32 = 238 + 6;
/// `l_sp_actor_name` sound id of `SP_NPC_CARPETPEDDLER`.
const SAHARAH_SOUND: u8 = 2;
/// `mSP_LIST_EVENT`: the carpet list she draws from.
const EVENT_LIST: usize = 3;
/// `ITEM1_CAT_CARPET`.
const CAT_CARPET: u16 = 6;

/// The event's save area (`arabian.used`): the trade-ins so far. It belongs to one special event (`rolled`: when
/// the calendar rolled it) and starts at 0 with the next.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CarpetEvent {
    /// `dates[SPECIAL0]` and `dates[SPECIAL1]` of the event this belongs to.
    pub rolled: (u16, u16),
    pub used: u8,
}

#[derive(Component)]
struct Saharah {
    /// The game day she came out for.
    day: i64,
}

/// `ITEM_NAME_GET_TYPE == ITEM1 && ITEM_NAME_GET_CAT == ITEM1_CAT_CARPET`.
fn is_carpet(item: u16) -> bool {
    item >> 12 == 2 && (item >> 8) & 0xF == CAT_CARPET
}

/// Startup: the saved event area.
fn apply_loaded(loaded: Res<LoadedSave>, mut event: ResMut<CarpetEvent>) {
    if let Some(save) = loaded.0.as_ref() {
        *event = save.carpet_event.clone().unwrap_or_default();
    }
}

/// Brings her out for her day and sends her off when the event ends.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    items: Option<Res<crate::field_items::FieldItems>>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Saharah)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let out = today.is_active(ev::CARPET_PEDDLER, now.hour)
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    // After midnight she is still the previous day's visitor.
    let day = if now.hour < 6 {
        now.day_number - 1
    } else {
        now.day_number
    };
    for (entity, s) in &existing {
        if !out || s.day != day {
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
        Name::new("Saharah"),
        Saharah { day },
        NpcClip::default(),
        NpcGait::default(),
        NpcModel(SAHARAH_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y),
    ));
    info!("Saharah is out at {at:?}");
}

/// A new special event starts the event area empty (`mEv_reserve_save_area`).
fn reset_for_new_event(shop: Option<Res<crate::shop::ShopData>>, mut event: ResMut<CarpetEvent>) {
    let Some((ev::CARPET_PEDDLER, slots)) = shop.and_then(|s| s.events.special_event()) else {
        return;
    };
    let rolled = (slots[0], slots[1]);
    if event.rolled != rolled {
        *event = CarpetEvent { rolled, used: 0 };
    }
}

/// What follows a line.
#[derive(Clone, Copy, Debug)]
enum After {
    Close,
    /// The pockets open for the trade-in.
    Pockets,
    /// She hands the new carpet over.
    GiveNew(u16),
    /// She hands the refused item back.
    Refuse(u16),
}

/// What follows a hand-over.
#[derive(Clone, Copy, Debug)]
enum Then {
    /// The line, then `After`.
    Say(u16, After),
    Close,
    /// The refused item goes back into the first free pocket.
    Return(u16),
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// `0x48A` is up with its first question (`next`: each answer's follow-up line).
    First(Vec<Option<usize>>),
    /// The second question is up.
    Second,
    /// The pockets are open.
    Picking,
    /// A line is up.
    Said(After),
    /// An item is crossing.
    Hand(Then),
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
    /// `peddler->price` and `peddler->carpet` of this talk.
    price: u32,
    carpet: u16,
}

fn context(
    identity: &crate::save::PlayerIdentity,
    price: u32,
    carpet: u16,
    article: u8,
) -> MsgContext {
    let now = crate::clock::now();
    let mut free = vec![String::new(); 20];
    free[0] = price.to_string();
    let mut items = vec![String::new(); 5];
    let mut articles = vec![0; 5];
    if carpet != 0 {
        items[0] = crate::items::name(carpet);
        articles[0] = article;
    }
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Saharah".into(),
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

/// Shows disc message `msg` (an absolute message number) in Saharah's voice.
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
        "Saharah",
        (SAHARAH_NPC, SAHARAH_SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Saharah message {msg:#06x}");
    asked.map(|(_, c)| c)
}

/// She turns to face the player while they talk.
fn face(
    time: Res<Time>,
    flow: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<Saharah>)>,
    mut saharah: Query<&mut Transform, With<Saharah>>,
) {
    let (Ok(at), Ok(mut transform)) = (player.single(), saharah.single_mut()) else {
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

/// The talk, as `aECPD_1st_check`, `aECPD_2nd_check` and the trade-in steps.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), With<Player>>,
    saharah: Query<(Entity, &Transform), With<Saharah>>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut pockets: ResMut<PocketsUi>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    mut event: ResMut<CarpetEvent>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, actor_at)) = saharah.single() else {
        return;
    };
    let article = if flow.carpet != 0 {
        crate::shop_nook::item_article(&disc, flow.carpet)
    } else {
        0
    };
    let ctx = context(&identity, flow.price, flow.carpet, article);
    let msg_of = |offset: u16| usize::from(native::saharah::MSG_START + offset);
    let end = |dialogue: &mut DialogueState, focus: &mut InputFocus, flow: &mut Talk| {
        *dialogue = DialogueState::default();
        *focus = InputFocus::World;
        flow.step = Step::Idle;
    };

    // The pockets are open: `aECPD_menu_close_wait`.
    if matches!(flow.step, Step::Picking) {
        if pockets.open {
            return;
        }
        pockets.give_any = false;
        *focus = InputFocus::Dialogue;
        let Some(slot) = pockets.sell_slot.take() else {
            say_abs(&disc, &mut dialogue, actor, &ctx, msg_of(6));
            flow.step = Step::Said(After::Close);
            return;
        };
        let item = inventory.pockets.pockets[slot];
        info!(
            "Saharah is offered {item:#06x} (carpet: {})",
            is_carpet(item)
        );
        // The picked item leaves the pockets either way.
        inventory.pockets.pockets[slot] = EMPTY_NO;
        inventory.pockets.set_item_cond(slot, 0);
        let then = if is_carpet(item) {
            // `aECPD_demo0_end_wait_init`: the count, the new carpet in the slot, the price.
            event.used = native::saharah_next_used(event.used);
            inventory.pockets.pockets[slot] = flow.carpet;
            inventory.pockets.set_item_cond(slot, 0);
            inventory.bells = inventory.bells.saturating_sub(flow.price);
            info!(
                "Saharah: trade-in done, {} Bells paid, trade-ins now {}",
                flow.price, event.used
            );
            Then::Say(7, After::GiveNew(flow.carpet))
        } else {
            Then::Say(9, After::Refuse(item))
        };
        handover.start(Request {
            item: Item::Goods(item),
            giver: Party::Player,
            taker: Party::Npc(actor),
        });
        flow.step = Step::Hand(then);
        return;
    }

    // An item is crossing.
    if let Step::Hand(then) = flow.step {
        *focus = InputFocus::Dialogue;
        if handover.busy() {
            return;
        }
        match then {
            Then::Say(offset, after) => {
                say_abs(&disc, &mut dialogue, actor, &ctx, msg_of(offset));
                flow.step = Step::Said(after);
            }
            Then::Return(item) => {
                // `mPr_SetFreePossessionItem`.
                inventory.add(item);
                end(&mut dialogue, &mut focus, &mut flow);
            }
            Then::Close => end(&mut dialogue, &mut focus, &mut flow),
        }
        return;
    }

    // Only her own answers; other talkers in town take theirs.
    let answer = if dialogue.speaker == Some(actor) {
        dialogue.external_result.take()
    } else {
        None
    };
    if let Some(choice) = answer {
        *focus = InputFocus::Dialogue;
        match std::mem::take(&mut flow.step) {
            Step::First(next) => {
                // `aECPD_1st_check`: Yes goes on to the second question (the message's continuation), No ends.
                if choice == 0 {
                    let follow = next.first().copied().flatten();
                    let asked = follow.and_then(|m| say_abs(&disc, &mut dialogue, actor, &ctx, m));
                    flow.step = if asked.is_some() {
                        Step::Second
                    } else {
                        Step::Said(After::Close)
                    };
                } else {
                    end(&mut dialogue, &mut focus, &mut flow);
                }
            }
            Step::Second => {
                // `aECPD_2nd_check`.
                if choice == 0 {
                    let has_carpet = inventory.pockets.pockets.iter().any(|&i| is_carpet(i));
                    let has_money = inventory.bells >= flow.price;
                    let msg = native::saharah_check_msg(has_carpet, has_money);
                    say_abs(&disc, &mut dialogue, actor, &ctx, usize::from(msg));
                    flow.step = Step::Said(if has_carpet && has_money {
                        After::Pockets
                    } else {
                        After::Close
                    });
                } else {
                    say_abs(&disc, &mut dialogue, actor, &ctx, msg_of(1));
                    flow.step = Step::Said(After::Close);
                }
            }
            Step::Said(After::Pockets) => {
                // `aECPD_menu_open_wait`: the window goes, the pockets open.
                *pockets = PocketsUi {
                    open: true,
                    give_any: true,
                    ..default()
                };
                *dialogue = DialogueState::default();
                *focus = InputFocus::Pockets;
                crate::sound::se(crate::sound::se_id::MENU_PAUSE);
                flow.step = Step::Picking;
            }
            Step::Said(After::GiveNew(item)) => {
                *dialogue = DialogueState::default();
                handover.start(Request {
                    item: Item::Goods(item),
                    giver: Party::Npc(actor),
                    taker: Party::Player,
                });
                flow.step = Step::Hand(Then::Close);
            }
            Step::Said(After::Refuse(item)) => {
                *dialogue = DialogueState::default();
                handover.start(Request {
                    item: Item::Goods(item),
                    giver: Party::Npc(actor),
                    taker: Party::Player,
                });
                flow.step = Step::Hand(Then::Return(item));
            }
            Step::Said(After::Close) | Step::Idle | Step::Picking | Step::Hand(_) => {
                end(&mut dialogue, &mut focus, &mut flow);
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
    // `aECPD_1st_check_init`: the price from the trade-ins so far and a random carpet of the event list.
    let list = crate::goods::list(crate::goods::Kind::Carpet, EVENT_LIST);
    flow.price = native::saharah_price(event.used);
    flow.carpet = if list.is_empty() {
        0
    } else {
        list[rand.0.random(list.len() as i32) as usize]
    };
    let article = if flow.carpet != 0 {
        crate::shop_nook::item_article(&disc, flow.carpet)
    } else {
        0
    };
    let ctx = context(&identity, flow.price, flow.carpet, article);
    info!(
        "Saharah offers {:#06x} for {} Bells (trade-ins {})",
        flow.carpet, flow.price, event.used
    );
    let asked = say_abs(&disc, &mut dialogue, actor, &ctx, msg_of(0));
    flow.step = match asked {
        Some(choice) => Step::First(choice.next),
        None => Step::Said(After::Close),
    };
}

pub struct SaharahPlugin;

impl Plugin for SaharahPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .init_resource::<CarpetEvent>()
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
