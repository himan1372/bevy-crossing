//! Halloween trick-or-treat (`ac_halloween_npc.c`, `ac_halloween_npc_talk.c_inc`, `_schedule.c_inc`).
//!
//! While the scheduler says `mEv_EVENT_HALLOWEEN` is active (October 31, 18:00 until midnight), up to five of
//! the town's villagers go out in costume (`mEvMN_GetHalloweenNpcName` registers each as
//! `SP_NPC_EV_HALLOWEEN_0` + the next free event-NPC slot: the costume's draw entry is their texture, the villager
//! stays the animation and identity). They run at the player (`aHWN_approach`: `aNPC_ACT_RUN` at the player while
//! the player is in their block) and, within 70 units (1.75 here) in the same block, force a talk
//! (`aHWN_force_talk_request`): the greeting `2442` (`0x98A`), then the question `0x98B + looks * 6`.
//!
//! The answer: Treat opens the pockets in the give mode (`mSM_IV_OPEN_GIVE`, "Give" / "Quit" for any item). Nothing
//! picked: `0x98C` and the trick. Candy (`ITM_FOOD_CANDY`): taken, `0x98F`, no trick. A tool (net, rod, golden net,
//! golden rod): handed over, `0x990`, handed back, then the trick. Anything else: taken, `0x98E`, the trick. Trick
//! (`aHWN_get_trick_type`): if the pockets hold a swappable item (furniture or an item that is not a tool, not a
//! quest item, not a jack-o'-lantern or jack-in-the-box) and `RANDOM_F < 0.5`, one of them becomes a random piece of
//! the Halloween2 furniture list; otherwise the player puts on the costume shirt (`ITM_CLOTH017`). Then
//! `0x98D + looks * 6`. All the `looks * 6` lines are the villager's personality (`mNpc_GetLooks`).
//!
//! Afterwards a talk the player starts (`aHWN_norm_talk_request`) is the single line `2479 + looks`; one started
//! before the forced greeting is the greeting (the approach think still has `aHWN_force_talk_request`).
//!
//! Port adaptations: a hand-over plays before the line that follows it (as with K.K. Slider), the quest-item
//! condition keeps an item from being offered, and the shirt change has no change animation or sparkle (nor does the
//! item trick have its player animation, order `0xFD`).

use std::collections::HashSet;

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::halloween as native;
use rustimal_logic::inventory::EMPTY_NO;

use crate::character::{Costume, NpcClip, PlayerLook};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::input::{InputFocus, InteractSet};
use crate::interaction::PlayerInventory;
use crate::interaction::TalkEvent;
use crate::messages::MsgChoice;
use crate::npc::{Indoors, Villager};
use crate::player::Player;
use crate::pockets::PocketsUi;
use crate::shop::ShopLocation;
use crate::shop_nook::handover::{HandOver, Item, Party, Request};
use crate::states::GameplaySet;
use crate::town::Town;

/// ALL_NPC_NUM + (SP_NPC_EV_HALLOWEEN_0 - SP_NPC_START).
const COSTUME_BASE: u32 = 238 + 32;
/// `SP_NPC_EV_HALLOWEEN_0..=4`: five event-NPC slots.
const MAX_COSTUMES: usize = 5;
/// `aHWN_force_talk_request`: `player_distance_xz < 70` game units, 40 to a cell.
const FORCE_TALK_DISTANCE: f32 = native::schedule::FORCE_TALK_DISTANCE / 40.0;
/// `ABS(player_distance_y) < 20`.
const FORCE_TALK_HEIGHT: f32 = native::schedule::FORCE_TALK_HEIGHT / 40.0;
/// `mSP_LIST_HALLOWEEN2`: the furniture list of the item trick.
const HALLOWEEN2_LIST: usize = 10;
/// The costume shirt's index in the clothing bank (`ITM_CLOTH017`).
const COSTUME_SHIRT: usize = 17;
/// `mPr_ITEM_COND_QUEST`.
const COND_QUEST: u32 = 2;

/// Puts costumes on the first five villagers while the event runs and takes them off after.
fn assign_costumes(
    mut commands: Commands,
    today: Res<TodayEvents>,
    villagers: Query<(Entity, &Villager, Option<&Costume>)>,
    mut talk: ResMut<Talk>,
) {
    let on = today.is_active(ev::HALLOWEEN, crate::clock::now().hour);
    if !on {
        for (entity, _, costume) in &villagers {
            if costume.is_some() {
                commands.entity(entity).remove::<Costume>();
            }
        }
        talk.greeted.clear();
        return;
    }
    let mut used: HashSet<u32> = villagers
        .iter()
        .filter_map(|(_, _, c)| c.map(|c| c.0))
        .collect();
    let mut order: Vec<(Entity, u32)> = villagers
        .iter()
        .filter(|(_, _, c)| c.is_none())
        .map(|(e, v, _)| (e, v.npc_index.unwrap_or(u32::MAX)))
        .collect();
    order.sort_by_key(|&(_, index)| index);
    for (entity, _) in order {
        // `mNpc_GetFreeEventNpcIdx`: the lowest free slot.
        let Some(slot) = (0..MAX_COSTUMES as u32).find(|s| !used.contains(&(COSTUME_BASE + s)))
        else {
            break;
        };
        used.insert(COSTUME_BASE + slot);
        // The arm clips of a hand-over play through `NpcClip`.
        commands
            .entity(entity)
            .insert((Costume(COSTUME_BASE + slot), NpcClip::default()));
        info!("Halloween: a villager puts on costume {slot}");
    }
}

/// `aHWN_approach`: a costumed villager runs at the player while they share an acre, unless a talk is going.
fn approach(
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    dialogue: Res<DialogueState>,
    talk: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<Villager>)>,
    mut villagers: Query<(&mut Villager, &Transform), (With<Costume>, Without<Indoors>)>,
) {
    if dialogue.active
        || !matches!(talk.step, Step::Idle)
        || *state.get() != ShopLocation::Town
        || *house.get() != crate::house::HouseState::Town
    {
        return;
    }
    let Ok(player) = player.single() else { return };
    let block = Town::cell_indices_of(player.translation).map(|(ax, az, ..)| (ax, az));
    for (mut villager, at) in &mut villagers {
        let theirs = Town::cell_indices_of(at.translation).map(|(ax, az, ..)| (ax, az));
        if block.is_some() && block == theirs {
            villager.run_at(player.translation);
        }
    }
}

/// What happens when the line that is up has been read.
#[derive(Clone, Copy, Debug)]
enum After {
    /// `aHWN_trick_timing_wait_talk_proc`: the trick.
    Trick,
    /// The talk ends.
    Close,
    /// A tool was taken for show (`aHWN_receive_tool_item_start_wait_talk_proc`): it is handed back, then the trick.
    ReturnTool(u16),
}

/// What follows a hand-over.
#[derive(Clone, Copy, Debug)]
enum Then {
    /// The line (a message number before `looks * 6`) and what follows it.
    Say(u16, After),
    /// The trick.
    Trick,
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// `2442` is up: the question follows.
    Greeting,
    /// The question is up (`next`: each answer's own follow-up line).
    Asked(Vec<Option<usize>>),
    /// The pockets are open in the give mode.
    Picking,
    /// A line is up.
    Said(After),
    /// An item is crossing.
    Hand(Then),
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
    /// The villager talking.
    villager: Option<Entity>,
    /// Villagers whose forced greeting is done (afterwards `aHWN_norm_talk_request` takes over).
    greeted: HashSet<Entity>,
}

/// `aHWN_get_trick_type`: the trick this time. An item trick changes one pocket now and returns true.
fn trick(
    inventory: &mut PlayerInventory,
    look: &mut PlayerLook,
    rand: &mut crate::game_rand::GameRand,
) -> bool {
    let mut pockets = [native::TrickPocket {
        item_type: 0,
        cat: 0,
        is_quest: false,
        ftr_idx: u16::MAX,
    }; native::POCKETS_SLOT_COUNT];
    for (slot, entry) in pockets.iter_mut().enumerate() {
        let item = inventory.pockets.pockets[slot];
        *entry = native::TrickPocket {
            item_type: (item >> 12) as u8,
            cat: ((item >> 8) & 0xF) as u8,
            is_quest: inventory.pockets.item_cond(slot) & COND_QUEST != 0,
            ftr_idx: crate::furniture::furniture_index(item).map_or(u16::MAX, |i| i as u16),
        };
    }
    let (count, slots) = native::scan_trick_candidates(&pockets);
    // `count > 0 && RANDOM_F(1.0f) < 0.5f`: the roll is only made with a candidate.
    let item_trick =
        count > 0 && native::trick_decision(count, rand.0.fqrand()) == native::trick::CHG_ITEM;
    let list = crate::goods::list(crate::goods::Kind::Furniture, HALLOWEEN2_LIST);
    if item_trick && !list.is_empty() {
        let item = list[rand.0.random(list.len() as i32) as usize];
        let slot = usize::from(slots[rand.0.random(count as i32) as usize]);
        inventory.pockets.pockets[slot] = item;
        inventory.pockets.set_item_cond(slot, 0);
        info!("Halloween trick: pocket {slot} becomes {item:#06x}");
        true
    } else {
        look.cloth = COSTUME_SHIRT;
        crate::sound::se(crate::sound::se_id::WEAR);
        info!("Halloween trick: the player is in the costume shirt");
        false
    }
}

fn is_tool(item: u16) -> bool {
    native::items::is_tool_range(item)
}

/// The forced talk and its steps (`aHWN_force_talk_request`, `aHWN_first_call_talk_proc`,
/// `aHWN_trick_or_treat_talk_proc`, the item and trick procs).
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    player: Query<&Transform, (With<Player>, Without<Villager>)>,
    villagers: Query<(Entity, &Villager, &Transform), (With<Costume>, Without<Indoors>)>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut pockets: ResMut<PocketsUi>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    mut look: ResMut<PlayerLook>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut claim: ResMut<crate::quest::QuestClaim>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    let Some(text) = disc.text.as_ref() else {
        return;
    };
    if *state.get() != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let now = crate::clock::now();
    let say = |dialogue: &mut DialogueState,
               entity: Entity,
               villager: &Villager,
               msg: u16|
     -> Option<MsgChoice> {
        let ctx = crate::dialogue::disc_message_context(&disc, villager, None, &now);
        let (pages, asked) = text.talk_asking(usize::from(msg), &ctx, &mut |_| 0);
        dialogue.begin_special(
            entity,
            villager.name,
            (villager.npc_index.unwrap_or(0), 0),
            pages,
            asked.as_ref().map(|(_, c)| c.labels.clone()),
        );
        info!("Halloween: {} says message {msg:#06x}", villager.name);
        asked.map(|(_, c)| c)
    };
    let end = |dialogue: &mut DialogueState,
               focus: &mut InputFocus,
               flow: &mut Talk,
               claim: &mut crate::quest::QuestClaim| {
        *dialogue = DialogueState::default();
        *focus = InputFocus::World;
        flow.villager = None;
        flow.step = Step::Idle;
        // The ordinary talk of this villager was ours (see `intercept_talk`).
        claim.0 = None;
        info!("Halloween: the talk ends");
    };

    // A talk in progress belongs to one villager.
    if let Some(entity) = flow.villager {
        let Ok((_, villager, _)) = villagers.get(entity) else {
            // They took the costume off or went away: the talk is over.
            pockets.give_any = false;
            end(&mut dialogue, &mut focus, &mut flow, &mut claim);
            return;
        };
        let looks = villager.looks;

        // The pockets are open: `aHWN_menu_close_wait_talk_proc`.
        if matches!(flow.step, Step::Picking) {
            if pockets.open {
                return;
            }
            pockets.give_any = false;
            *focus = InputFocus::Dialogue;
            match pockets.sell_slot.take() {
                None => {
                    // Nothing picked: `aHWN_restart_msg_win(0x098C)`, then the trick.
                    say(
                        &mut dialogue,
                        entity,
                        villager,
                        native::msg::restart(looks, 0x098C),
                    );
                    flow.step = Step::Said(After::Trick);
                }
                Some(slot) => {
                    let item = inventory.pockets.pockets[slot];
                    info!("Halloween: {} is offered {item:#06x}", villager.name);
                    let (msg, after) = if item == native::items::CANDY {
                        (0x098F, After::Close)
                    } else if is_tool(item) {
                        (0x0990, After::ReturnTool(item))
                    } else {
                        (0x098E, After::Trick)
                    };
                    if !is_tool(item) {
                        // `mPr_SetPossessionItem(.., slot, EMPTY_NO, ..)`: tools stay in the pockets.
                        inventory.pockets.pockets[slot] = EMPTY_NO;
                        inventory.pockets.set_item_cond(slot, 0);
                    }
                    handover.start(Request {
                        item: Item::Goods(item),
                        giver: Party::Player,
                        taker: Party::Npc(entity),
                    });
                    flow.step = Step::Hand(Then::Say(msg, after));
                }
            }
            return;
        }

        // An item is crossing.
        if let Step::Hand(then) = flow.step {
            if handover.busy() {
                *focus = InputFocus::Dialogue;
                return;
            }
            *focus = InputFocus::Dialogue;
            match then {
                Then::Say(msg, after) => {
                    say(
                        &mut dialogue,
                        entity,
                        villager,
                        native::msg::restart(looks, msg),
                    );
                    flow.step = Step::Said(after);
                }
                Then::Trick => {
                    trick(&mut inventory, &mut look, &mut rand);
                    say(
                        &mut dialogue,
                        entity,
                        villager,
                        native::msg::trick_cloth_done(looks),
                    );
                    flow.step = Step::Said(After::Close);
                }
            }
            return;
        }

        // A line was read (or the player answered a question).
        if dialogue.speaker == Some(entity) {
            if let Some(choice) = dialogue.external_result.take() {
                *focus = InputFocus::Dialogue;
                match std::mem::take(&mut flow.step) {
                    Step::Greeting => {
                        // `aHWN_first_call_talk_proc`: the continuing message is the question.
                        let asked = say(
                            &mut dialogue,
                            entity,
                            villager,
                            native::msg::first_call_continue(looks),
                        );
                        flow.step = Step::Asked(asked.map(|c| c.next).unwrap_or_default());
                    }
                    Step::Asked(next) => {
                        if choice == 0 {
                            // `aHWN_trick_or_treat_talk_proc`: Treat opens the pockets in the give mode.
                            *pockets = PocketsUi {
                                open: true,
                                give_any: true,
                                ..default()
                            };
                            *dialogue = DialogueState::default();
                            *focus = InputFocus::Pockets;
                            crate::sound::se(crate::sound::se_id::MENU_PAUSE);
                            flow.step = Step::Picking;
                        } else if let Some(Some(msg)) = next.get(choice).copied() {
                            // Trick: the answer's own line, then the trick.
                            let ctx =
                                crate::dialogue::disc_message_context(&disc, villager, None, &now);
                            let (pages, _) = text.talk_asking(msg, &ctx, &mut |_| 0);
                            dialogue.begin_special(
                                entity,
                                villager.name,
                                (villager.npc_index.unwrap_or(0), 0),
                                pages,
                                None,
                            );
                            flow.step = Step::Said(After::Trick);
                        } else {
                            // No line of its own: straight to the trick.
                            *dialogue = DialogueState::default();
                            trick(&mut inventory, &mut look, &mut rand);
                            say(
                                &mut dialogue,
                                entity,
                                villager,
                                native::msg::trick_cloth_done(looks),
                            );
                            flow.step = Step::Said(After::Close);
                        }
                    }
                    Step::Said(After::Trick) => {
                        *dialogue = DialogueState::default();
                        trick(&mut inventory, &mut look, &mut rand);
                        say(
                            &mut dialogue,
                            entity,
                            villager,
                            native::msg::trick_cloth_done(looks),
                        );
                        flow.step = Step::Said(After::Close);
                    }
                    Step::Said(After::ReturnTool(item)) => {
                        // `aNPC_DEMO_GIVE_ITEM(item, aHOI_REQUEST_PUTAWAY)`: the tool goes back, then the trick.
                        *dialogue = DialogueState::default();
                        handover.start(Request {
                            item: Item::Goods(item),
                            giver: Party::Npc(entity),
                            taker: Party::Player,
                        });
                        flow.step = Step::Hand(Then::Trick);
                    }
                    Step::Said(After::Close) | Step::Idle | Step::Picking | Step::Hand(_) => {
                        end(&mut dialogue, &mut focus, &mut flow, &mut claim);
                    }
                }
            }
        }
        return;
    }

    // `aHWN_force_talk_request`.
    if !matches!(flow.step, Step::Idle) || !focus.is_world() || dialogue.active {
        return;
    }
    let Ok(player) = player.single() else { return };
    let block = Town::cell_indices_of(player.translation).map(|(ax, az, ..)| (ax, az));
    for (entity, villager, at) in &villagers {
        if flow.greeted.contains(&entity) {
            continue;
        }
        let delta = (at.translation - player.translation).with_y(0.0);
        let theirs = Town::cell_indices_of(at.translation).map(|(ax, az, ..)| (ax, az));
        if delta.length() < FORCE_TALK_DISTANCE
            && (at.translation.y - player.translation.y).abs() < FORCE_TALK_HEIGHT.max(1.0)
            && block.is_some()
            && block == theirs
        {
            *focus = InputFocus::Dialogue;
            flow.greeted.insert(entity);
            flow.villager = Some(entity);
            flow.step = Step::Greeting;
            say(&mut dialogue, entity, villager, native::msg::FIRST_GREETING);
            return;
        }
    }
}

/// A talk the player starts with a costumed villager is ours, not the ordinary villager talk
/// (`aHWN_norm_talk_request` / `aHWN_force_talk_request`): the single line `2479 + looks` once they have greeted,
/// the forced greeting if they have not.
#[allow(clippy::too_many_arguments)]
fn intercept_talk(
    mut events: MessageReader<TalkEvent>,
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    villagers: Query<&Villager, (With<Costume>, Without<Indoors>)>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut claim: ResMut<crate::quest::QuestClaim>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        events.clear();
        return;
    };
    let Some(text) = disc.text.as_ref() else {
        return;
    };
    if *state.get() != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        events.clear();
        return;
    }
    for event in events.read() {
        let Ok(villager) = villagers.get(event.villager) else {
            continue;
        };
        if !matches!(flow.step, Step::Idle) || dialogue.active {
            continue;
        }
        let entity = event.villager;
        let (msg, step) = if flow.greeted.contains(&entity) {
            (
                native::msg::normal_talk(villager.looks),
                Step::Said(After::Close),
            )
        } else {
            flow.greeted.insert(entity);
            (native::msg::FIRST_GREETING, Step::Greeting)
        };
        let ctx =
            crate::dialogue::disc_message_context(&disc, villager, None, &crate::clock::now());
        let (pages, asked) = text.talk_asking(usize::from(msg), &ctx, &mut |_| 0);
        dialogue.begin_special(
            entity,
            villager.name,
            (villager.npc_index.unwrap_or(0), 0),
            pages,
            asked.map(|(_, c)| c.labels),
        );
        info!(
            "Halloween: {} says message {msg:#06x} (asked)",
            villager.name
        );
        *focus = InputFocus::Dialogue;
        claim.0 = Some(entity);
        flow.villager = Some(entity);
        flow.step = step;
        break;
    }
}

/// While a talk is in any step the villagers stand still, as in a conversation.
fn pause_villagers(talk: Res<Talk>, mut pause: ResMut<crate::npc::VillagerPause>) {
    pause.0 = !matches!(talk.step, Step::Idle);
}

pub struct HalloweenPlugin;

impl Plugin for HalloweenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .add_systems(
                Update,
                (assign_costumes, approach)
                    .chain()
                    .after(crate::event_today::refresh)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                intercept_talk
                    .in_set(crate::quest::QuestInputSet)
                    .after(crate::interaction::interact)
                    .in_set(GameplaySet),
            )
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet))
            .add_systems(Update, pause_villagers.after(talk).in_set(GameplaySet));
    }
}
