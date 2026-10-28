//! The part-time job's guide: Nook while `mEv_CheckFirstJob()` holds
//! (`ac_npc_rcn_guide2*.c_inc`). He greets the player on the first visit of a
//! session (`aNRG2_think_init_proc`), starts and ends the jobs, and refuses to sell
//! while the job runs (`aNRG2_goods_talk_request`). The job's state and tables are
//! `rustimal_logic::first_job`; this module is the talk flow.
//!
//! Jobs 1 (change clothes), 2 (plant flowers), 3 (deliver furniture), 4 (send a
//! letter), 11 (the open quest), 5 (carpet), 6 (axe) and 7 (post a notice) are covered;
//! the second letter, second axe and introductions jobs are not.

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
}

pub(super) fn residents(
    roster: Option<&crate::roster::TownRoster>,
    town: Option<&crate::town::Town>,
) -> Residents {
    let mut out = Residents::default();
    let Some(roster) = roster else {
        return out;
    };
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
            return Vec::new();
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
            | Job::PostNotice
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
        let mut picks = Picks {
            slot: inventory.pockets.find_free_slot().unwrap_or(0),
            ..Picks::default()
        };
        if matches!(
            kind,
            Job::DeliverFurniture | Job::SendLetter | Job::DeliverCarpet
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
        if kind == Job::SendLetter {
            // The recipient is put in the address book even if the player never met them.
            *meet = job
                .recipient
                .and_then(|id| residents.name_of(id))
                .map(str::to_string);
        }
        state.ask_again = out.can_ask_again;
        grant(kind, job.item, inventory, rand);
        Some(kind.start_message())
    } else {
        kind.refuse_message()
    }
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
        });
    }
    if job.finished() {
        // The job_end talks (`dt_tbl` J1_END ... J11_END), then the next job.
        let end = match kind {
            Job::ChangeCloth => 0x07F4,
            Job::PlantFlower => 0x07F9,
            Job::DeliverFurniture => 0x0801,
            Job::SendLetter => 0x080B,
            Job::DeliverCarpet => 0x0812,
            Job::DeliverAxe => 0x0818,
            Job::PostNotice => 0x081A,
            _ => 0x0830, // Job::Open
        };
        state.ask_again = false;
        if kind == Job::PostNotice {
            // `aNRG2_all_job_end_talk_proc`: the last job. The guide sends the player away and
            // the job is over (`mEv_UnSetFirstJob`): the next day Nook greets them (HRATALK).
            identity.events.unset_first_job();
            identity.first_job = None;
            return Some(Talk::said(vec![end]));
        }
        let next = job.next_job(false)?;
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
        Job::DeliverAxe => vec![0x0815],
        Job::PostNotice => vec![0x0819],
        // `aNRG2_setup_j4_cont`: asked again (0x0836) until the flag drops, else the paper talk.
        Job::SendLetter if state.ask_again => vec![0x0836],
        Job::SendLetter => vec![0x0807],
        _ => vec![0x0829], // Job::Open
    };
    Some(Talk {
        chain,
        free: residents.job_strings(job),
        meet: None,
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
) {
    if !identity.events.first_job {
        return;
    }
    let Some(job) = identity.first_job.as_ref() else {
        return;
    };
    let mut next = job.clone();
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
        Some(Job::SendLetter) => {
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

/// Session-only guide state (`Common_Get(quest).work`, `can_ask_again_flag`).
#[derive(Resource, Default)]
pub(crate) struct GuideState {
    pub greeted: bool,
    /// `can_ask_again_flag`: the letter job's reminder.
    pub ask_again: bool,
}
