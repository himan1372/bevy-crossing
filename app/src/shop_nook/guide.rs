//! The part-time job's guide: Nook while `mEv_CheckFirstJob()` holds
//! (`ac_npc_rcn_guide2*.c_inc`). He greets the player on the first visit of a
//! session (`aNRG2_think_init_proc`), starts and ends the jobs, and refuses to sell
//! while the job runs (`aNRG2_goods_talk_request`). The job's state and tables are
//! `rustimal_logic::first_job`; this module is the talk flow.
//!
//! Jobs 1 (change clothes) and 2 (plant flowers) are covered; the later jobs need
//! villager picks and the quest manager and stop at their start.

use bevy::prelude::*;
use rustimal_logic::first_job::{self as fj, Cond, FirstJob, Job, Picks};

use crate::interaction::PlayerInventory;

/// `aNRG2_goods_talk_request`: the goods cannot be touched.
pub(super) const GOODS_MESSAGE: usize = 0x081C;
/// `mPr_ITEM_COND_QUEST`.
const QUEST: u32 = 2;

/// What the guide says: the messages in order (each starts when the last ends)
/// and whether he walks forward while speaking.
pub(super) struct Talk {
    pub chain: Vec<usize>,
}

/// The residents by villager-table index (`Save_Get(animals)` order).
pub(super) fn residents(roster: Option<&crate::roster::TownRoster>) -> Vec<u16> {
    roster.map_or_else(Vec::new, |r| {
        r.residents.iter().map(|e| e.index as u16).collect()
    })
}

/// Whether the guide runs the shop: the job is on and its first two jobs are in play.
pub(super) fn active(identity: &crate::save::PlayerIdentity) -> bool {
    identity.events.first_job
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
    residents: &[u16],
    rand: &mut rustimal_logic::qrand::Qrand,
) -> Option<usize> {
    if job.progress != 1 {
        return None;
    }
    let kind = job.job?;
    if kind.index() > Job::DeliverFurniture.index() {
        return None;
    }
    if kind.can_start(free_slots(inventory)) {
        let mut picks = Picks {
            slot: inventory.pockets.find_free_slot().unwrap_or(0),
            ..Picks::default()
        };
        if kind == Job::DeliverFurniture {
            // `aNRG2_stepup_j3`: an unused villager and a rolled piece of furniture.
            picks.villager = fj::pick_other_villager(residents, &job.used, &mut |n| rand.random(n));
            picks.item =
                crate::goods::furniture_for_delivery(&fj::NO_DELIVERY_FURNITURE, &mut |n| {
                    rand.random(n as i32) as u32
                })?;
        }
        job.step_up(picks)?;
        grant(kind, job.item, inventory, rand);
        Some(kind.start_message())
    } else {
        kind.refuse_message()
    }
}

/// `aNRG2_think_init_proc` and `aNRG2_set_force_talk_info`: the first visit of a
/// session, or the scolding for a changed shirt.
pub(super) fn arrival(
    identity: &mut crate::save::PlayerIdentity,
    inventory: &mut PlayerInventory,
    look: &crate::character::PlayerLook,
    residents: &[u16],
    rand: &mut rustimal_logic::qrand::Qrand,
    first_visit: bool,
) -> Option<Talk> {
    let job = identity.first_job.get_or_insert_with(FirstJob::start);
    if cloth_check(job, look) {
        job.wrong_cloth = true;
        return Some(Talk {
            chain: vec![0x0832],
        });
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
    chain.extend(say_hello(job, inventory, residents, rand));
    Some(Talk { chain })
}

/// `aNRG2_norm_talk_request`: the player talks to the guide.
/// `None` when the job in play is not one of the first two.
pub(super) fn talk(
    identity: &mut crate::save::PlayerIdentity,
    inventory: &mut PlayerInventory,
    look: &crate::character::PlayerLook,
    residents: &[u16],
    rand: &mut rustimal_logic::qrand::Qrand,
) -> Option<Talk> {
    let job = identity.first_job.get_or_insert_with(FirstJob::start);
    if cloth_check(job, look) {
        job.wrong_cloth = true;
        return Some(Talk {
            chain: vec![0x0832],
        });
    }
    let kind = job.job?;
    if kind.index() > Job::PlantFlower.index() {
        return None;
    }
    // `aNRG2_talk_start_wait` / `aNRG2_job_start_wait`.
    if job.progress == 1 {
        let chain = say_hello(job, inventory, residents, rand)
            .or_else(|| kind.refuse_message())
            .into_iter()
            .collect();
        return Some(Talk { chain });
    }
    if job.finished() {
        // The job_end talks (J1_END 0x07F4, J2_END 0x07F9), then the next job.
        let end = [0x07F4, 0x07F9, 0x0801][kind.index()];
        let mut chain = vec![end];
        let next = job.next_job(false)?;
        job.setup_job(next);
        chain.extend(say_hello(job, inventory, residents, rand));
        return Some(Talk { chain });
    }
    // The job is running: `aNRG2_setup_j1_cont` / `_j2_cont`.
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
        _ => vec![0x07FE],
    };
    Some(Talk { chain })
}

/// The quest manager's per-frame rules for the running job
/// (`aQMgr_move_own_errand_*`).
pub(crate) fn track(
    mut identity: ResMut<crate::save::PlayerIdentity>,
    look: Res<crate::character::PlayerLook>,
    inventory: Res<PlayerInventory>,
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
        _ => {}
    }
    if next != *job {
        info!("first job progress {} -> {}", job.progress, next.progress);
        identity.first_job = Some(next);
    }
}

/// A flag kept for the session only (`Common_Get(quest).work`).
#[derive(Resource, Default)]
pub(crate) struct GuideState {
    pub greeted: bool,
}
