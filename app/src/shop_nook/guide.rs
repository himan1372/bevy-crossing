//! The part-time job's guide: Nook while `mEv_CheckFirstJob()` holds
//! (`ac_npc_rcn_guide2*.c_inc`). He greets the player on the first visit of a
//! session (`aNRG2_think_init_proc`), starts and ends the jobs, and refuses to sell
//! while the job runs (`aNRG2_goods_talk_request`). The job's state and tables are
//! `rustimal_logic::first_job`; this module is the talk flow.
//!
//! Jobs 1 (change clothes), 2 (plant flowers), 3 (deliver furniture), 4 (send a
//! letter), 10 (introductions), 11 (the open quest), 5 (carpet), 6 (axe) and 7 (post a notice)
//! are covered, and so are the second letter and second axe jobs, which the "recipient moved
//! away" redirects lead to (`Job::moved_away`).

use bevy::prelude::*;
use rustimal_logic::first_job::{self as fj, Cond, FirstJob, Job, Picks};

use crate::interaction::PlayerInventory;

/// `aNRG2_goods_talk_request`: the goods cannot be touched.
pub(super) const GOODS_MESSAGE: usize = 0x081C;
/// `mPr_ITEM_COND_QUEST`.
const QUEST: u32 = 2;

/// What the guide says: the messages in order (each starts when the last ends),
/// the message strings the jobs need (`aNRG2_set_str_proc`: FREE3 acre row, FREE4
/// acre number, FREE5 recipient, FREE6 post-office acre) and a villager the job
/// makes the player's acquaintance (`mNpc_SetAnimalPersonalID2Memory`).
#[derive(Default)]
pub(super) struct Talk {
    /// The last job's end talk: afterwards the guide sends the player out (`aNRG2_all_job_end`).
    pub end_job: bool,
    pub chain: Vec<usize>,
    pub free: Vec<(usize, String)>,
    pub meet: Option<String>,
}

impl Talk {
    fn said(chain: Vec<usize>) -> Self {
        Self {
            chain,
            ..Self::default()
        }
    }
}

/// The residents the jobs can name: villager-table index, name and home acre.
#[derive(Default)]
pub(super) struct Residents {
    ids: Vec<u16>,
    names: Vec<String>,
    /// (x, z) of the home's acre in the town plan.
    acres: Vec<(usize, usize)>,
    /// The post office's acre column (`mFI_BlockKind2BkNum`), 1-based.
    post_column: usize,
    /// `mNpc_CheckFriendAllAnimal && mSC_check_ArbeitPlayer`: every resident met and Tortimer talked to.
    pub met_everyone: bool,
    /// The errand's recipient when they no longer live in town: id and name.
    gone: Option<(u16, String)>,
}

pub(super) fn residents(
    roster: Option<&crate::roster::TownRoster>,
    town: Option<&crate::town::Town>,
    trophy: bool,
) -> Residents {
    let mut out = Residents::default();
    let Some(roster) = roster else {
        return out;
    };
    out.met_everyone = trophy
        && roster
            .residents
            .iter()
            .all(|e| roster.met.contains(&e.index));
    for (i, entry) in roster.residents.iter().enumerate() {
        out.ids.push(entry.index as u16);
        out.names.push(entry.name.clone());
        let acre = town
            .and_then(|t| t.plan.house_acres.get(i).copied())
            .map_or(0, usize::from);
        out.acres.push((
            acre % rustimal_logic::town_gen::ACRE_WIDTH,
            acre / rustimal_logic::town_gen::ACRE_WIDTH,
        ));
    }
    if let Some(town) = town {
        out.post_column = town
            .plan
            .acres
            .iter()
            .position(|a| a.feature == rustimal_logic::town_gen::Feature::PostOffice as u8)
            .map_or(1, |i| i % rustimal_logic::town_gen::ACRE_WIDTH + 1);
    }
    out
}

impl Residents {
    /// Names a recipient who has moved away (`mQst_CheckRemoveTarget`) from the disc's table.
    pub(super) fn with_gone(
        mut self,
        recipient: Option<u16>,
        disc: &crate::game_disc::GameDisc,
    ) -> Self {
        if let Some(id) = recipient.filter(|id| !self.ids.contains(id)) {
            self.gone = crate::roster::VillagerTables::from_disc(disc)
                .and_then(|tables| tables.entry(u32::from(id)))
                .map(|entry| (id, entry.name));
        }
        self
    }

    pub(super) fn ids(&self) -> &[u16] {
        &self.ids
    }

    fn name_of(&self, id: u16) -> Option<&str> {
        let i = self.ids.iter().position(|&r| r == id)?;
        self.names.get(i).map(String::as_str)
    }

    /// `aNRG2_set_str_j3` / `_j4`.
    fn job_strings(&self, job: &FirstJob) -> Vec<(usize, String)> {
        let Some(kind) = job.job else {
            return Vec::new();
        };
        let letter = matches!(kind, Job::SendLetter | Job::SendLetter2);
        let delivery = matches!(
            kind,
            Job::DeliverFurniture | Job::DeliverCarpet | Job::DeliverAxe | Job::DeliverAxe2
        );
        if !(letter || delivery) {
            return Vec::new();
        }
        let Some(id) = job.recipient else {
            return Vec::new();
        };
        let Some(i) = self.ids.iter().position(|&r| r == id) else {
            // Moved away: only the name is left (FREE5).
            return self
                .gone
                .iter()
                .filter(|(gone, _)| *gone == id)
                .map(|(_, name)| (5, name.clone()))
                .collect();
        };
        let (x, z) = self.acres[i];
        let mut free = vec![
            (3, ['A', 'B', 'C', 'D', 'E', 'F'][z.min(5)].to_string()),
            (4, (x + 1).to_string()),
            (5, self.names[i].clone()),
        ];
        if letter {
            free.push((6, self.post_column.to_string()));
        }
        free
    }
}

/// Whether the guide runs the shop.
pub(super) fn active(identity: &crate::save::PlayerIdentity) -> bool {
    identity.events.first_job
}

/// The jobs the guide can run so far.
fn handled(kind: Job) -> bool {
    matches!(
        kind,
        Job::ChangeCloth
            | Job::PlantFlower
            | Job::DeliverFurniture
            | Job::SendLetter
            | Job::Open
            | Job::DeliverCarpet
            | Job::DeliverAxe
            | Job::SendLetter2
            | Job::DeliverAxe2
            | Job::PostNotice
            | Job::Introductions
    )
}

fn free_slots(inventory: &PlayerInventory) -> usize {
    inventory
        .pockets
        .pockets
        .iter()
        .filter(|&&i| i == 0)
        .count()
}

fn worn(look: &crate::character::PlayerLook) -> u16 {
    0x2400 + look.cloth as u16
}

/// `aNRG2_cloth_check`: changed out of the uniform on a later job.
fn cloth_check(job: &FirstJob, look: &crate::character::PlayerLook) -> bool {
    job.job.is_some_and(|j| j != Job::ChangeCloth)
        && !job.wrong_cloth
        && worn(look) != fj::WORK_UNIFORM
}

/// The part of `aNRG2_set_possession` that puts items in the pockets.
fn grant(
    job: Job,
    item: u16,
    inventory: &mut PlayerInventory,
    rand: &mut rustimal_logic::qrand::Qrand,
) {
    for (item, cond) in fj::possession(job, item, &mut |n| rand.random(n)) {
        let cond = if cond == Cond::Quest { QUEST } else { 0 };
        inventory.pockets.put(item, cond);
    }
}

/// `aNRG2_say_hello_talk_proc`: a chosen job starts if the pockets allow it.
/// Returns the message that follows.
fn say_hello(
    job: &mut FirstJob,
    inventory: &mut PlayerInventory,
    residents: &Residents,
    state: &mut GuideState,
    meet: &mut Option<String>,
    rand: &mut rustimal_logic::qrand::Qrand,
) -> Option<usize> {
    if job.progress != 1 {
        return None;
    }
    let kind = job.job?;
    if !handled(kind) {
        return None;
    }
    if kind.can_start(free_slots(inventory)) {
        begin_job(kind, job, inventory, residents, state, meet, rand, true)?;
        Some(kind.start_message())
    } else {
        kind.refuse_message()
    }
}

/// `aNRG2_stepup_job` and `aNRG2_set_possession` for a chosen job: the picks, the state change,
/// the acquaintance and the items. `None` if the job is not at progress 1.
fn begin_job(
    kind: Job,
    job: &mut FirstJob,
    inventory: &mut PlayerInventory,
    residents: &Residents,
    state: &mut GuideState,
    meet: &mut Option<String>,
    rand: &mut rustimal_logic::qrand::Qrand,
    give: bool,
) -> Option<()> {
    let mut picks = Picks {
        slot: inventory.pockets.find_free_slot().unwrap_or(0),
        ..Picks::default()
    };
    if matches!(
        kind,
        Job::DeliverFurniture | Job::SendLetter | Job::SendLetter2 | Job::DeliverCarpet
    ) {
        // `aNRG2_stepup_j3` / `_j4` / `_j5`: an unused villager.
        picks.villager =
            fj::pick_other_villager(&residents.ids, &job.used, &mut |n| rand.random(n));
    }
    if matches!(kind, Job::DeliverFurniture | Job::DeliverCarpet) {
        // ... and a rolled piece of furniture (the walk-on ones excluded) or carpet.
        let (goods, exclude) = if kind == Job::DeliverFurniture {
            (
                crate::goods::Kind::Furniture,
                &fj::NO_DELIVERY_FURNITURE[..],
            )
        } else {
            (crate::goods::Kind::Carpet, &[][..])
        };
        picks.item =
            crate::goods::delivery_item(goods, exclude, &mut |n| rand.random(n as i32) as u32)?;
    }
    let out = job.step_up(picks)?;
    if matches!(kind, Job::SendLetter | Job::SendLetter2) {
        // The recipient is put in the address book even if the player never met them.
        *meet = job
            .recipient
            .and_then(|id| residents.name_of(id))
            .map(str::to_string);
    }
    state.ask_again = out.can_ask_again;
    if give {
        grant(kind, job.item, inventory, rand);
    }
    Some(())
}

/// A talk that ends with the job's start (or refusal) message appended.
fn with_hello(
    mut chain: Vec<usize>,
    job: &mut FirstJob,
    inventory: &mut PlayerInventory,
    residents: &Residents,
    state: &mut GuideState,
    rand: &mut rustimal_logic::qrand::Qrand,
) -> Talk {
    let mut meet = None;
    chain.extend(say_hello(job, inventory, residents, state, &mut meet, rand));
    Talk {
        chain,
        free: residents.job_strings(job),
        meet,
        ..Talk::default()
    }
}

/// `aNRG2_think_init_proc` and `aNRG2_set_force_talk_info`: the first visit of a
/// session, or the scolding for a changed shirt.
pub(super) fn arrival(
    identity: &mut crate::save::PlayerIdentity,
    inventory: &mut PlayerInventory,
    look: &crate::character::PlayerLook,
    residents: &Residents,
    state: &mut GuideState,
    rand: &mut rustimal_logic::qrand::Qrand,
    first_visit: bool,
) -> Option<Talk> {
    if identity.first_job.is_none() {
        // The new-game intro starts the job without the map (`ac_npc_p_sel_schedule.c_inc`).
        identity.map_flag = false;
    }
    let job = identity.first_job.get_or_insert_with(FirstJob::start);
    if cloth_check(job, look) {
        job.wrong_cloth = true;
        return Some(Talk::said(vec![0x0832]));
    }
    if !first_visit {
        return None;
    }
    let mut chain = Vec::new();
    if job.job.is_none() {
        // `mQst_ERRAND_FIRSTJOB_START`: wearing the uniform already skips the first job.
        if worn(look) == fj::WORK_UNIFORM {
            job.setup_job(Job::PlantFlower);
            chain.extend([0x07EE, 0x0837]);
        } else {
            job.setup_job(Job::ChangeCloth);
            chain.push(0x07EE);
        }
    } else {
        chain.push(0x07EF);
    }
    Some(with_hello(chain, job, inventory, residents, state, rand))
}

/// `aNRG2_norm_talk_request`: the player talks to the guide.
/// `None` when the job in play is not one the guide runs yet.
pub(super) fn talk(
    identity: &mut crate::save::PlayerIdentity,
    inventory: &mut PlayerInventory,
    look: &crate::character::PlayerLook,
    residents: &Residents,
    state: &mut GuideState,
    rand: &mut rustimal_logic::qrand::Qrand,
) -> Option<Talk> {
    let job = identity.first_job.get_or_insert_with(FirstJob::start);
    if cloth_check(job, look) {
        job.wrong_cloth = true;
        return Some(Talk::said(vec![0x0832]));
    }
    let kind = job.job?;
    if !handled(kind) {
        return None;
    }
    // `aNRG2_talk_start_wait` / `aNRG2_job_start_wait`.
    if job.progress == 1 {
        let mut meet = None;
        let chain = say_hello(job, inventory, residents, state, &mut meet, rand)
            .or_else(|| kind.refuse_message())
            .into_iter()
            .collect();
        return Some(Talk {
            chain,
            free: residents.job_strings(job),
            meet,
            ..Talk::default()
        });
    }
    if job.finished() {
        // The job_end talks (`dt_tbl` J1_END ... J11_END), then the next job.
        let end = match kind {
            Job::ChangeCloth => 0x07F4,
            Job::PlantFlower => 0x07F9,
            Job::DeliverFurniture => 0x0801,
            Job::SendLetter | Job::SendLetter2 => 0x080B,
            Job::DeliverCarpet => 0x0812,
            Job::DeliverAxe | Job::DeliverAxe2 => 0x0818,
            Job::PostNotice => 0x081A,
            Job::Introductions => 0x0828,
            _ => 0x0830, // Job::Open
        };
        state.ask_again = false;
        if kind == Job::DeliverFurniture {
            // `aNRG2_3rd_job_end_talk_proc`: the guide hands over the map (`map_flag`) when the
            // message reaches its order (`hand_over_job_item`).
            identity.map_flag = true;
            state.map_handover = true;
        }
        if kind == Job::PostNotice {
            // `aNRG2_all_job_end_talk_proc`: the last job. The guide sends the player away and
            // the job is over (`mEv_UnSetFirstJob`): the next day Nook greets them (HRATALK).
            identity.events.unset_first_job();
            identity.first_job = None;
            return Some(Talk {
                end_job: true,
                ..Talk::said(vec![end])
            });
        }
        let next = job.next_job(residents.met_everyone)?;
        job.setup_job(next);
        return Some(with_hello(
            vec![end],
            job,
            inventory,
            residents,
            state,
            rand,
        ));
    }
    // The job is running: `aNRG2_setup_j*_cont`.
    let letter = matches!(kind, Job::SendLetter | Job::SendLetter2);
    // `mQst_CheckRemoveTarget`: the recipient no longer lives in town (a letter job asks
    // again first).
    if let Some((message, next)) = kind.moved_away() {
        if job.recipient_gone(residents.ids()) && !(letter && state.ask_again) {
            state.redirect = Some(Redirect {
                next,
                give_back: kind.given_back(job.item),
                grant: !letter,
            });
            let mut chain = vec![message];
            chain.extend(kind.moved_away_next());
            return Some(Talk {
                chain,
                free: residents.job_strings(job),
                ..Talk::default()
            });
        }
    }
    let chain = match kind {
        Job::ChangeCloth => {
            if inventory.pockets.find_item(fj::WORK_UNIFORM).is_some() {
                vec![0x07F3]
            } else if free_slots(inventory) == 0 {
                vec![0x0835]
            } else {
                // 0x081F: he hands over another uniform.
                grant(kind, 0, inventory, rand);
                vec![0x081F]
            }
        }
        Job::PlantFlower => vec![0x07F8],
        Job::DeliverFurniture => vec![0x07FE],
        Job::DeliverCarpet => vec![0x080F],
        Job::DeliverAxe | Job::DeliverAxe2 => vec![0x0815],
        Job::PostNotice => vec![0x0819],
        Job::Introductions => vec![0x0822],
        // `aNRG2_setup_j4_cont`: asked again (0x0836) until the flag drops, else the paper talk.
        Job::SendLetter | Job::SendLetter2 if state.ask_again => vec![0x0836],
        // `aNRG2_give_new_paper_talk_proc`: a letter went to the wrong person (progress 3).
        Job::SendLetter | Job::SendLetter2 if job.progress == 3 => {
            if kind.can_start(free_slots(inventory)) {
                grant(kind, job.item, inventory, rand);
                state.ask_again = true;
                vec![0x0804, 0x0805]
            } else {
                state.ask_again = false;
                vec![0x0804, 0x07FA]
            }
        }
        // 0x0807 goes on to the question about more paper (0x080A), answered by `paper_answer`.
        Job::SendLetter | Job::SendLetter2 => {
            state.paper_ask = true;
            vec![0x0807]
        }
        _ => vec![0x0829], // Job::Open
    };
    Some(Talk {
        chain,
        free: residents.job_strings(job),
        ..Talk::default()
    })
}

/// The quest manager's per-frame rules for the running job
/// (`aQMgr_move_own_errand_*`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn track(
    mut identity: ResMut<crate::save::PlayerIdentity>,
    look: Res<crate::character::PlayerLook>,
    inventory: Res<PlayerInventory>,
    post: Res<crate::post::Post>,
    mut quest: ResMut<crate::quest::QuestState>,
    roster: Option<Res<crate::roster::TownRoster>>,
    mut letters: Local<Option<u32>>,
) {
    // A letter handed in while a letter job runs (`mNpc_SetAnimalMail`).
    let delivered = letters
        .replace(post.letters_given)
        .is_some_and(|before| post.letters_given != before);
    if !identity.events.first_job {
        return;
    }
    let Some(job) = identity.first_job.as_ref() else {
        return;
    };
    let mut next = job.clone();
    if delivered {
        next.letter_sent();
    }
    match next.job {
        Some(Job::ChangeCloth) => next.check_cloth(worn(&look)),
        // `ITEM1_CAT_PLANT`: saplings and flower bags (0x2900-0x29FF).
        Some(Job::PlantFlower) => next.check_seed(
            inventory
                .pockets
                .pockets
                .iter()
                .any(|&item| (0x2900..0x2A00).contains(&item)),
        ),
        // The recipient holds a letter from the player (`letter_info.exists`).
        Some(Job::SendLetter | Job::SendLetter2) => {
            let name = roster.as_deref().and_then(|r| {
                let id = next.recipient?;
                r.residents
                    .iter()
                    .find(|e| e.index == u32::from(id))
                    .map(|e| e.name.clone())
            });
            next.check_letter(name.is_some_and(|n| post.sent_to.contains(&n)));
        }
        // The villagers use the ordinary quest talks; taking a quest finishes the job.
        // Every resident met and Tortimer talked to (`aQMgr_move_own_errand_hello`).
        Some(Job::Introductions) => {
            let met = roster
                .as_deref()
                .is_some_and(|r| r.residents.iter().all(|e| r.met.contains(&e.index)));
            next.check_hello(met && identity.arbeit_trophy);
        }
        Some(Job::Open) => {
            if next.open_quest_running() {
                quest.offer_now();
            }
            next.check_open_quest(quest.active.is_some());
        }
        _ => {}
    }
    if next != *job {
        info!("first job progress {} -> {}", job.progress, next.progress);
        identity.first_job = Some(next);
    }
}

/// `mDemo_ORDER_NPC0`, and the order the job-start messages raise at the hand-over point
/// (`aNRG2_demo_start_wait_talk_proc`: index 1, value 4).
const ORDER_NPC0: u8 = 1;
/// `ITM_TOWN_MAP`.
const TOWN_MAP: u16 = 0x251D;

/// `aNRG2_demo_start_wait_talk_proc` .. `aNRG2_demo_end_wait2_talk_proc`: when the guide's
/// message reaches its hand-over order, he holds out the job's item (already in the pockets),
/// the player takes it, and the message cannot continue until it is put away.
pub(crate) fn hand_over_job_item(
    mut orders: ResMut<crate::msg_window::DemoOrders>,
    mut handover: ResMut<super::handover::HandOver>,
    mut dialogue: ResMut<crate::dialogue::DialogueState>,
    mut state: ResMut<GuideState>,
    identity: Res<crate::save::PlayerIdentity>,
    mut inventory: ResMut<PlayerInventory>,
) {
    for (kind, index, value) in orders.take_raised() {
        // `aNRG2_3rd_job_end_talk_proc`: order (NPC0, 9) while the third job's end is spoken.
        if (kind, index) == (ORDER_NPC0, 9) && value != 0 && state.map_handover {
            state.map_handover = false;
            info!("hand-over of the town map");
            dialogue.held = true;
            handover.start(super::handover::Request {
                item: super::handover::Item::Goods(TOWN_MAP),
                giver: super::handover::Party::Nook,
                taker: super::handover::Party::Player,
            });
            continue;
        }
        // `aNRG2_demo2_start_wait_talk_proc` / `demo3_start_wait`: the redirect message's order
        // takes the undelivered item back from the player's pockets and hands it over.
        if (kind, index) == (ORDER_NPC0, 9) && value != 0 {
            if let Some(Redirect {
                give_back: Some(item),
                ..
            }) = state.redirect
            {
                // The source clears the first pocket if the quest item cannot be found.
                let slot = inventory
                    .pockets
                    .find_item_with_cond(item, QUEST)
                    .unwrap_or(0);
                inventory.pockets.pockets[slot] = 0;
                inventory.pockets.set_item_cond(slot, 0);
                info!("hand-back of the job item {item:#06x}");
                dialogue.held = true;
                handover.start(super::handover::Request {
                    item: super::handover::Item::Goods(item),
                    giver: super::handover::Party::Player,
                    taker: super::handover::Party::Nook,
                });
                // Only the first order of the message (the later ones have other values).
                state.redirect = state.redirect.map(|r| Redirect {
                    give_back: None,
                    ..r
                });
                continue;
            }
        }
        if (kind, index, value) != (ORDER_NPC0, 1, 4) || !identity.events.first_job {
            continue;
        }
        let Some(job) = identity.first_job.as_ref().and_then(|j| j.job) else {
            continue;
        };
        if let fj::Display::Item(item) = job.display() {
            info!("hand-over of the job item {item:#06x}");
            // The lock starts with the order, before the next frame can continue the page.
            dialogue.held = true;
            handover.start(super::handover::Request {
                item: super::handover::Item::Goods(item),
                giver: super::handover::Party::Nook,
                taker: super::handover::Party::Player,
            });
        }
    }
    if dialogue.held && !handover.busy() {
        dialogue.held = false;
    }
}

/// The redirect messages are shown on their own: the new recipient's name is put in before
/// the message that follows them.
pub(super) fn single(message: usize) -> bool {
    matches!(message, 0x07FF | 0x0810 | 0x0816 | 0x0808)
}

/// The end of a redirect message (`aNRG2_demo2_end_wait_talk_proc` -> `aNRG2_job_start_talk_proc`,
/// `aNRG2_decide_new_target_talk_proc`): the replacing job is chosen and started with a new
/// pick, and the message that follows needs the new recipient's strings.
pub(super) fn redirect_effect(
    identity: &mut crate::save::PlayerIdentity,
    inventory: &mut PlayerInventory,
    residents: &Residents,
    state: &mut GuideState,
    rand: &mut rustimal_logic::qrand::Qrand,
) -> Option<(Vec<(usize, String)>, Option<String>)> {
    let redirect = state.redirect.take()?;
    let job = identity.first_job.as_mut()?;
    job.setup_job(redirect.next);
    let mut meet = None;
    // A letter's new target does not give paper yet: the guide asks (`check_more_paper`).
    begin_job(
        redirect.next,
        job,
        inventory,
        residents,
        state,
        &mut meet,
        rand,
        redirect.grant,
    )?;
    // The message that follows ends on the question about more paper.
    state.paper_ask = !redirect.grant;
    info!(
        "first job redirected to {:?}, recipient {:?}",
        redirect.next, job.recipient
    );
    Some((residents.job_strings(job), meet))
}

/// `aNRG2_check_more_paper_talk_proc`: the answer to 0x080A. Yes gives another sheet if the
/// pockets allow it (`aNRG2_give_new_paper_talk_proc`); the message that follows is returned.
pub(super) fn paper_answer(
    answer: usize,
    identity: &mut crate::save::PlayerIdentity,
    inventory: &mut PlayerInventory,
    state: &mut GuideState,
    rand: &mut rustimal_logic::qrand::Qrand,
) -> usize {
    state.paper_ask = false;
    let Some(job) = identity.first_job.as_mut() else {
        return 0x0806;
    };
    let kind = job.job.unwrap_or(Job::SendLetter);
    if answer != 0 {
        state.ask_again = false;
        return 0x0806;
    }
    job.progress = 2;
    if kind.can_start(free_slots(inventory)) {
        grant(kind, job.item, inventory, rand);
        state.ask_again = true;
        0x0805
    } else {
        state.ask_again = false;
        0x07FA
    }
}

/// Session-only guide state (`Common_Get(quest).work`, `can_ask_again_flag`).
#[derive(Resource, Default)]
pub(crate) struct GuideState {
    pub greeted: bool,
    /// `can_ask_again_flag`: the letter job's reminder.
    pub ask_again: bool,
    /// The third job's end talk is up: its order (NPC0, 9) hands the map over.
    pub map_handover: bool,
    /// Seconds since the last talk ended, once the whole job is over.
    pub ending: Option<f32>,
    /// The guide is giving a job another go (`mQst_CheckRemoveTarget`): set when the redirect
    /// message is chosen, carried out as it ends.
    pub redirect: Option<Redirect>,
    /// The talk is on 0x0807 / 0x080A: the question about more paper follows.
    pub paper_ask: bool,
}

/// What a "recipient moved away" talk does: the job that takes this one's place, and the item
/// the player hands back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Redirect {
    pub next: Job,
    pub give_back: Option<u16>,
    /// The job's items are given too (not for a letter's new target, which asks about paper).
    pub grant: bool,
}

/// `aNRG2_all_job_end` / `aNRG2_send_away_init`: a second after the last talk the guide sends the
/// player out of the shop (the goodbye is skipped), and outside the cheer follows
/// (`crate::job_end`).
pub(crate) fn send_away(
    time: Res<Time>,
    focus: Res<crate::input::InputFocus>,
    mut state: ResMut<GuideState>,
    mut twins: ResMut<super::Twins>,
    mut payment: ResMut<crate::job_end::CompletePayment>,
    mut wipe: ResMut<crate::scene_wipe::SceneWipe>,
) {
    let Some(waited) = state.ending.as_mut() else {
        return;
    };
    if !focus.is_world() {
        *waited = 0.0;
        return;
    }
    *waited += time.delta_secs();
    if *waited >= 1.0 {
        // The picture fades to black before he sends the player out (`FADE_TYPE_OUT`).
        if wipe.clear() {
            wipe.fade_out();
        } else if wipe.black() {
            state.ending = None;
            // `Common_Get(transition).wipe_type = WIPE_TYPE_CIRCLE_LEFT`: the town opens as a circle.
            wipe.circle_next(false);
            twins.goodbye = super::Goodbye::Done;
            payment.begin();
        }
    }
}
