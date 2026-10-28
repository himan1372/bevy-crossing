//! The villagers' side of the part-time job (`ac_quest_talk_fj_init.c`,
//! `aQMgr_talk_fj_*`). While the job runs every villager greets the player with a
//! three-answer question (`aQMgr_talk_fj_select_talk`); the villager an errand is
//! addressed to also offers "Delivery!", takes the quest item and answers with the
//! errand's messages (`l_set_errand_data[kind].msg_start`, plus `looks * 2`).
//! Only the furniture delivery (job 3) is covered.

use bevy::ecs::message::{MessageCursor, Messages};
use bevy::prelude::*;
use rustimal_logic::first_job::{Job, FIRST_JOB_GREETING};

use crate::dialogue::DialogueState;
use crate::input::InputFocus;
use crate::interaction::{PlayerInventory, TalkEvent};
use crate::npc::Villager;

/// `mPr_ITEM_COND_QUEST`.
const QUEST: u32 = 2;

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// The three-answer greeting is up. `slot` is the pocket holding the errand item.
    Greeting { own_slot: Option<usize> },
    /// Messages still to say after the current one.
    Rest(Vec<usize>),
}

#[derive(Resource, Default)]
pub(crate) struct FirstJobTalk {
    step: Step,
    villager: Option<Entity>,
    /// A talk handed back to the ordinary dialogue (the "hello" answers).
    passed_on: Option<Entity>,
}

/// `l_set_errand_data[kind].msg_start[COMPLETE_INIT / COMPLETE_END]` (the table's base numbers;
/// the villager's looks * 2 is added).
fn complete_messages(job: Job) -> Option<(usize, usize)> {
    match job {
        Job::DeliverFurniture => Some((0x08EF, 0x08F0)),
        Job::DeliverCarpet => Some((0x08FB, 0x08FC)),
        Job::DeliverAxe | Job::DeliverAxe2 => Some((0x0907, 0x0908)),
        _ => None,
    }
}

/// `aQMgr_get_select_hellow_no`: the greeting answer by the time of day.
fn hello_choice(hour: u8) -> usize {
    match hour {
        4..=9 => 0x3E,
        10..=17 => 0x3F,
        _ => 0x40,
    }
}

#[allow(clippy::too_many_arguments)]
fn talk(
    mut messages: ResMut<Messages<TalkEvent>>,
    mut cursor: Local<MessageCursor<TalkEvent>>,
    villagers: Query<&Villager>,
    mut state: ResMut<FirstJobTalk>,
    mut identity: ResMut<crate::save::PlayerIdentity>,
    mut inventory: ResMut<PlayerInventory>,
    mut dialogue: ResMut<DialogueState>,
    mut claim: ResMut<crate::quest::QuestClaim>,
    mut focus: ResMut<InputFocus>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut roster: Option<ResMut<crate::roster::TownRoster>>,
    mut memory: ResMut<crate::villager_memory::VillagerMemory>,
    house: Res<State<crate::house::HouseState>>,
    shop: Res<State<crate::shop::ShopLocation>>,
    disc: Option<Res<crate::game_disc::GameDisc>>,
) {
    // Always read the talks, so an unanswered one cannot linger.
    let talks: Vec<Entity> = cursor.read(&messages).map(|e| e.villager).collect();
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *house.get() != crate::house::HouseState::Town
        || *shop.get() != crate::shop::ShopLocation::Town
    {
        return;
    }
    let Some(text) = disc.text.as_ref() else {
        return;
    };

    // The player answered (only our own dialogue: other talkers take theirs).
    if state.villager.is_some() && dialogue.speaker == state.villager {
        if let Some(choice) = dialogue.external_result.take() {
            let entity = state.villager.expect("checked");
            let Ok(villager) = villagers.get(entity) else {
                *state = FirstJobTalk::default();
                return;
            };
            let looks = usize::from(villager.looks);
            let ctx =
                crate::dialogue::disc_message_context(&disc, villager, None, &crate::clock::now());
            let say =
                |dialogue: &mut DialogueState, msg: usize, ctx: &crate::messages::MsgContext| {
                    let (pages, _) = text.talk(msg, ctx, &mut |_| 0);
                    dialogue.begin_special(
                        entity,
                        villager.name,
                        (villager.npc_index.unwrap_or(0), 0),
                        pages,
                        None,
                    );
                };
            match std::mem::take(&mut state.step) {
                Step::Greeting { own_slot } => {
                    let hint = FIRST_JOB_GREETING.hint + looks * 2;
                    let ordinary =
                        choice == 1 && own_slot.is_some() || (own_slot.is_none() && choice < 2);
                    if choice == 0 && own_slot.is_some() {
                        // The errand is done (`aQMgr_talk_fj_reward_to`: progress 0). The villager's
                        // reward is `quest_item` = the errand's own item (`CURRENT_ITEM`), set back into
                        // the pocket as an ordinary item: the player keeps the furniture.
                        if let (Some(slot), Some(job)) = (own_slot, identity.first_job.as_mut()) {
                            inventory.pockets.set_item_cond(slot, 0);
                            job.finish();
                            if let Some((init, end)) = job.job.and_then(complete_messages) {
                                let mut ctx = ctx.clone();
                                ctx.items = vec![crate::items::name(job.item)];
                                say(&mut dialogue, init + looks * 2, &ctx);
                                state.step = Step::Rest(vec![end + looks * 2]);
                                crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
                                info!("first job delivery to {}", villager.name);
                            }
                        }
                    } else if ordinary {
                        // `aQMgr_TALK_COMMON_CHANGE_TALK_NORMAL`.
                        *dialogue = DialogueState::default();
                        *focus = InputFocus::World;
                        claim.0 = None;
                        state.passed_on = Some(entity);
                        messages.write(TalkEvent { villager: entity });
                        state.villager = None;
                        return;
                    } else {
                        say(&mut dialogue, hint, &ctx);
                        state.step = Step::Rest(Vec::new());
                    }
                }
                Step::Rest(mut rest) => {
                    if rest.is_empty() {
                        *dialogue = DialogueState::default();
                        *focus = InputFocus::World;
                        claim.0 = None;
                        state.villager = None;
                        return;
                    }
                    let msg = rest.remove(0);
                    say(&mut dialogue, msg, &ctx);
                    state.step = Step::Rest(rest);
                }
                Step::Idle => {}
            }
        }
        return;
    }

    // A new talk. While the open-quest job runs (FJOPENQUEST) villagers use the ordinary quest talks.
    if !identity.events.first_job
        || dialogue.active
        || identity
            .first_job
            .as_ref()
            .is_some_and(|j| j.open_quest_running())
    {
        return;
    }
    for entity in talks {
        if state.passed_on == Some(entity) {
            state.passed_on = None;
            continue;
        }
        let Some(job) = identity.first_job.as_mut() else {
            continue;
        };
        let Ok(villager) = villagers.get(entity) else {
            continue;
        };
        let looks = usize::from(villager.looks);
        // `aQMgr_fj_check_own_quest`: an errand for this villager, running, with its item in the pockets.
        let own_slot = job
            .job
            .filter(|j| complete_messages(*j).is_some())
            .filter(|_| job.progress >= 2 && job.recipient.is_some())
            .filter(|_| job.recipient.map(u32::from) == villager.npc_index)
            .and_then(|_| {
                (0..inventory.pockets.pockets.len()).find(|&i| {
                    inventory.pockets.pockets[i] == job.item
                        && inventory.pockets.item_cond(i) == QUEST
                })
            });
        let hello = text
            .choice(hello_choice(crate::clock::now().hour))
            .unwrap_or_default();
        let sorry = text
            .choice(FIRST_JOB_GREETING.sorry + rand.0.random(5) as usize)
            .unwrap_or_default();
        let labels: Vec<String> = if own_slot.is_some() {
            vec![
                text.choice(FIRST_JOB_GREETING.delivery).unwrap_or_default(),
                hello,
                sorry,
            ]
        } else {
            vec![
                hello,
                text.choice(FIRST_JOB_GREETING.tips).unwrap_or_default(),
                sorry,
            ]
        };
        let ctx =
            crate::dialogue::disc_message_context(&disc, villager, None, &crate::clock::now());
        let (pages, _) = text.talk(FIRST_JOB_GREETING.greeting + looks * 2, &ctx, &mut |_| 0);
        dialogue.begin_special(
            entity,
            villager.name,
            (villager.npc_index.unwrap_or(0), 0),
            pages,
            Some(labels),
        );
        // Greeting a villager is meeting them (`mNpc_CheckFriendAllAnimal` counts the memory).
        if let (Some(roster), Some(index)) = (roster.as_deref_mut(), villager.npc_index) {
            if !roster.met.contains(&index) {
                roster.met.push(index);
            }
        }
        memory.meet(villager.name);
        *focus = InputFocus::Dialogue;
        claim.0 = Some(entity);
        state.villager = Some(entity);
        state.step = Step::Greeting { own_slot };
        info!(
            "first job greeting from {} #{:?} (delivery: {})",
            villager.name,
            villager.npc_index,
            own_slot.is_some()
        );
        break;
    }
}

pub struct FirstJobTalkPlugin;

impl Plugin for FirstJobTalkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FirstJobTalk>().add_systems(
            Update,
            talk.in_set(crate::quest::QuestInputSet)
                .after(crate::interaction::interact)
                .in_set(crate::states::GameplaySet),
        );
    }
}
