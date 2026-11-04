//! Halloween trick-or-treat (`ac_halloween_npc.c`, `ac_halloween_npc_talk.c_inc`, `_schedule.c_inc`).
//!
//! While the scheduler says `mEv_EVENT_HALLOWEEN` is active (October 31, 18:00 until midnight), up to five of
//! the town's villagers go out in costume (`mEvMN_GetHalloweenNpcName` registers each as
//! `SP_NPC_EV_HALLOWEEN_0` + the next free event-NPC slot: the costume's draw entry is their texture, the villager
//! stays the animation and identity). They run at the player (`aHWN_approach`: `aNPC_ACT_RUN` at the player while
//! the player is in their block) and, within 70 units (1.75 here) in the same block, force a talk
//! (`aHWN_force_talk_request`): the greeting `2442` (`0x98A`), then the question `0x98B + looks * 6`.
//!
//! This is the first slice: costumes, the chase, the forced greeting and the question. The treat (an item from the
//! pockets: candy, a tool, anything else), the trick (an item swapped for a Halloween piece or the player in the
//! costume shirt) and the ordinary talk afterwards (`2479 + looks`) follow in later slices.

use std::collections::HashSet;

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::halloween as native;

use crate::character::Costume;
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::input::{InputFocus, InteractSet};
use crate::npc::{Indoors, Villager};
use crate::player::Player;
use crate::shop::ShopLocation;
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
        commands.entity(entity).insert(Costume(COSTUME_BASE + slot));
        info!("Halloween: a villager puts on costume {slot}");
    }
}

/// `aHWN_approach`: a costumed villager runs at the player while they share an acre, unless a talk is going.
fn approach(
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    dialogue: Res<DialogueState>,
    player: Query<&Transform, (With<Player>, Without<Villager>)>,
    mut villagers: Query<(&mut Villager, &Transform), (With<Costume>, Without<Indoors>)>,
) {
    if dialogue.active
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

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// `2442` is up: the question follows.
    Greeting,
    /// The question is up (`next`: each answer's own follow-up, if the message gives one).
    Asked,
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
    /// The villager talking.
    villager: Option<Entity>,
    /// Villagers whose forced greeting is done (afterwards `aHWN_norm_talk_request` takes over).
    greeted: HashSet<Entity>,
}

/// The forced talk and its steps (`aHWN_force_talk_request`, `aHWN_first_call_talk_proc`,
/// `aHWN_trick_or_treat_talk_proc`).
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
     -> Option<Vec<String>> {
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
        asked.map(|(_, c)| c.labels)
    };

    // The player answered (only our own dialogue: other talkers take theirs).
    if let Some(entity) = flow.villager.filter(|&e| dialogue.speaker == Some(e)) {
        if let Some(choice) = dialogue.external_result.take() {
            *focus = InputFocus::Dialogue;
            let Ok((_, villager, _)) = villagers.get(entity) else {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
                *flow = Talk {
                    greeted: std::mem::take(&mut flow.greeted),
                    ..default()
                };
                return;
            };
            match std::mem::take(&mut flow.step) {
                Step::Greeting => {
                    // `aHWN_first_call_talk_proc`: the continuing message is the trick-or-treat question.
                    say(
                        &mut dialogue,
                        entity,
                        villager,
                        native::msg::first_call_continue(villager.looks),
                    );
                    flow.step = Step::Asked;
                }
                Step::Asked => {
                    // `aHWN_trick_or_treat_talk_proc`: Treat (0) opens the pockets, anything else is the trick.
                    // Both are later slices; the talk ends here.
                    info!(
                        "Halloween: {} answered {choice} (treat / trick not built yet)",
                        villager.name
                    );
                    *dialogue = DialogueState::default();
                    *focus = InputFocus::World;
                    flow.villager = None;
                }
                Step::Idle => {
                    *dialogue = DialogueState::default();
                    *focus = InputFocus::World;
                    flow.villager = None;
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
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
