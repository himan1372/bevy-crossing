//! The new-game part-time job (`mQst_ERRAND_TYPE_FIRST_JOB`): the errand state in
//! `m_quest.c` (`mQst_SetFirstJob*`, `mQst_CheckFirstJobFin`) and the guide's job
//! tables in `ac_npc_rcn_guide2_talk.c_inc` (`aNRG2_*`).
//!
//! The guide gives the player eleven jobs in a chain. Each job passes through
//! progress 1 (chosen, not yet running: `aNRG2_setup_job`), 2 (running: set by
//! `aNRG2_stepup_job`) and 0 (finished). Picking villagers and furniture,
//! handing items over and the messages' text belong to the caller; this module
//! owns the state and every table.

/// `ITM_CLOTH016`, the work uniform.
pub const WORK_UNIFORM: u16 = 0x2410;
/// `ITM_PAPER55`, plain stationery.
pub const PAPER55: u16 = 0x2037;
/// `ITM_AXE`.
pub const AXE: u16 = 0x2201;
/// `ITM_SAPLING`.
pub const SAPLING: u16 = 0x2900;
/// `ITM_WHITE_PANSY_BAG`, the first of nine flower bags.
pub const WHITE_PANSY_BAG: u16 = 0x2902;
/// `ITM_CARPET00`.
pub const CARPET00: u16 = 0x2600;

/// `mQst_ERRAND_FIRST_JOB_ANIMAL_NUM`.
pub const USED_VILLAGERS: usize = 2;
/// `FTR_START(FTR_TAK_HOLE01)` and `FTR_START(FTR_KON_MAT)`: the furniture the delivery never rolls
/// (`ng_list` of `aNRG2_stepup_j3`).
pub const NO_DELIVERY_FURNITURE: [u16; 2] = [0x1444, 0x154C];
/// `FTR_START(FTR_NOG_FAN01)`, shown while the furniture job is offered.
pub const FAN: u16 = 0x1088;

/// `mNpc_GetOtherAnimalPersonalID`: a resident not yet used for this job, chosen uniformly
/// (`RANDOM(n)` over those left). `None` when everyone has been used.
pub fn pick_other_villager(
    residents: &[u16],
    used: &[Option<u16>],
    random: &mut dyn FnMut(i32) -> i32,
) -> Option<u16> {
    let left: Vec<u16> = residents
        .iter()
        .copied()
        .filter(|r| !used.contains(&Some(*r)))
        .collect();
    if left.is_empty() {
        return None;
    }
    left.get(random(left.len() as i32).max(0) as usize).copied()
}

/// The eleven jobs, in `mQst_ERRAND_FIRSTJOB_CHANGE_CLOTH..OPEN` order (the
/// guide's `aNRG2_JOB1..JOB11`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Job {
    ChangeCloth,
    PlantFlower,
    DeliverFurniture,
    SendLetter,
    DeliverCarpet,
    DeliverAxe,
    PostNotice,
    SendLetter2,
    DeliverAxe2,
    Introductions,
    Open,
}

pub const JOBS: [Job; 11] = [
    Job::ChangeCloth,
    Job::PlantFlower,
    Job::DeliverFurniture,
    Job::SendLetter,
    Job::DeliverCarpet,
    Job::DeliverAxe,
    Job::PostNotice,
    Job::SendLetter2,
    Job::DeliverAxe2,
    Job::Introductions,
    Job::Open,
];

/// What the guide shows while he offers the job (`aNRG2_demo_start_wait_talk_proc`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Display {
    None,
    Item(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cond {
    Normal,
    Quest,
}

impl Job {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Option<Job> {
        JOBS.get(i).copied()
    }

    /// `job_start_msg_no`.
    pub fn start_message(self) -> usize {
        [
            0x07F1, 0x07F6, 0x07FB, 0x0802, 0x080C, 0x0813, 0x0818, 0x0802, 0x0813, 0x0821, 0x0827,
        ][self.index()]
    }

    /// `job_start_refuse_msg_no` (`-1`, no message, for the last two).
    pub fn refuse_message(self) -> Option<usize> {
        [
            Some(0x07F0),
            Some(0x07F5),
            Some(0x07FA),
            Some(0x07FA),
            Some(0x07FA),
            Some(0x07FA),
            Some(0x0818),
            Some(0x07FA),
            Some(0x07FA),
            None,
            None,
        ][self.index()]
    }

    /// `aNRG2_demo_start_wait_talk_proc`'s `itemNo`.
    pub fn display(self) -> Display {
        match self {
            Job::ChangeCloth => Display::Item(WORK_UNIFORM),
            Job::PlantFlower => Display::Item(WHITE_PANSY_BAG),
            Job::DeliverFurniture => Display::Item(FAN),
            Job::SendLetter | Job::SendLetter2 => Display::Item(PAPER55),
            Job::DeliverCarpet => Display::Item(CARPET00),
            Job::DeliverAxe | Job::DeliverAxe2 => Display::Item(AXE),
            Job::PostNotice | Job::Introductions | Job::Open => Display::None,
        }
    }

    /// `aNRG2_check_job_start`: pocket room needed to begin the job.
    pub fn can_start(self, free_slots: usize) -> bool {
        match self {
            Job::PlantFlower => free_slots >= 10,
            Job::PostNotice | Job::Introductions | Job::Open => true,
            _ => free_slots > 0,
        }
    }

    /// `next_job_no` of `aNRG2_job_end_talk_proc`.
    fn next(self) -> Job {
        [
            Job::PlantFlower,
            Job::DeliverFurniture,
            Job::SendLetter,
            Job::Open,
            Job::DeliverAxe,
            Job::PostNotice,
            Job::PostNotice,
            Job::DeliverAxe2,
            Job::PostNotice,
            Job::DeliverFurniture,
            Job::DeliverCarpet,
        ][self.index()]
    }
}

/// `aNRG2_set_possession`: what the guide puts in the pockets when a job starts.
/// `errand_item` is the errand's item (the furniture or carpet to deliver);
/// `random(n)` is `RANDOM(n)`.
pub fn possession(
    job: Job,
    errand_item: u16,
    random: &mut dyn FnMut(i32) -> i32,
) -> Vec<(u16, Cond)> {
    match job {
        Job::PlantFlower => {
            let mut items = Vec::new();
            for _ in 0..7 {
                items.push((WHITE_PANSY_BAG + random(9) as u16, Cond::Normal));
            }
            for _ in 0..3 {
                items.push((SAPLING, Cond::Normal));
            }
            items
        }
        Job::DeliverFurniture | Job::DeliverCarpet => vec![(errand_item, Cond::Quest)],
        Job::ChangeCloth => vec![(WORK_UNIFORM, Cond::Normal)],
        Job::SendLetter | Job::SendLetter2 => vec![(PAPER55, Cond::Normal)],
        Job::DeliverAxe | Job::DeliverAxe2 => vec![(AXE, Cond::Quest)],
        // PostNotice sets `EMPTY_NO`, a no-op; the last two give nothing.
        Job::PostNotice | Job::Introductions | Job::Open => Vec::new(),
    }
}

/// A villager picked for a job (`mNpc_GetOtherAnimalPersonalID`), the item rolled
/// for a delivery (`mSP_SelectRandomItem_New`) and the first free pocket.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Picks {
    pub villager: Option<u16>,
    pub item: u16,
    pub slot: usize,
}

/// What `aNRG2_stepup_job` asks the caller to do besides the state change.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepUp {
    /// `can_ask_again_flag = TRUE` (the letter jobs).
    pub can_ask_again: bool,
    /// `mEv_EventON(mEv_SAVED_FJOPENQUEST_PLRn)` (the open job).
    pub open_quest_event: bool,
}

/// `mQst_errand_c` for the first job. `job == None` is `mQst_ERRAND_FIRSTJOB_START`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FirstJob {
    pub job: Option<Job>,
    pub progress: u8,
    pub recipient: Option<u16>,
    pub used: [Option<u16>; USED_VILLAGERS],
    pub used_num: u8,
    pub item: u16,
    pub pockets_idx: usize,
    /// `wrong_cloth`: set when the player changes out of the uniform mid-job.
    pub wrong_cloth: bool,
}

impl FirstJob {
    /// `mQst_SetFirstJobStart`.
    pub fn start() -> Self {
        Self::default()
    }

    /// `aNRG2_setup_job`: the job is chosen (progress 1, not yet running).
    pub fn setup_job(&mut self, job: Job) {
        self.job = Some(job);
        self.progress = 1;
    }

    /// `mQst_CheckFirstJobFin`: finished iff progress is 0.
    pub fn finished(&self) -> bool {
        self.progress == 0
    }

    /// The player has done the job (`progress = 0`).
    pub fn finish(&mut self) {
        self.progress = 0;
    }

    /// `aNRG2_stepup_job`: when the job is chosen (progress 1) the `mQst_SetFirstJob*`
    /// setter for it runs and the job starts (progress 2). Returns `None` if the
    /// job is not at progress 1.
    pub fn step_up(&mut self, picks: Picks) -> Option<StepUp> {
        if self.progress != 1 {
            return None;
        }
        let job = self.job?;
        let mut out = StepUp::default();
        self.progress = 2;
        match job {
            Job::ChangeCloth => self.item = WORK_UNIFORM,
            Job::PlantFlower | Job::Introductions => {}
            Job::PostNotice => self.recipient = None,
            Job::DeliverFurniture => {
                self.recipient = picks.villager;
                self.used[0] = picks.villager;
                self.used_num = 1;
                self.pockets_idx = picks.slot;
                self.item = picks.item;
            }
            Job::SendLetter | Job::SendLetter2 => {
                self.recipient = picks.villager;
                self.used[1] = picks.villager;
                self.used_num = 2;
                out.can_ask_again = true;
            }
            Job::DeliverCarpet => {
                self.recipient = picks.villager;
                self.pockets_idx = picks.slot;
                self.item = picks.item;
            }
            Job::DeliverAxe | Job::DeliverAxe2 => {
                // The axe goes to the villager who got the letter.
                self.recipient = self.used[1];
                self.pockets_idx = picks.slot;
                self.item = AXE;
            }
            Job::Open => {
                self.recipient = None;
                out.open_quest_event = true;
            }
        }
        Some(out)
    }

    /// `aQMgr_move_own_errand_cloth`: running (2) and wearing the errand's shirt
    /// finishes it; finished (0) and not wearing it any more puts it back to 2.
    pub fn check_cloth(&mut self, worn: u16) {
        match self.progress {
            2 if worn == self.item => self.progress = 0,
            0 if worn != self.item => self.progress = 2,
            _ => {}
        }
    }

    /// `aQMgr_move_own_errand_seed`: running, and no plant item (flower bag,
    /// sapling, ...) left in the pockets, finishes it.
    pub fn check_seed(&mut self, plant_in_pockets: bool) {
        if self.progress == 2 && !plant_in_pockets {
            self.progress = 0;
        }
    }

    /// `aQMgr_move_own_errand_letter`: the recipient has a letter from the player.
    pub fn check_letter(&mut self, recipient_has_letter: bool) {
        if matches!(self.progress, 2 | 3) && recipient_has_letter {
            self.progress = 0;
        }
    }

    /// `aQMgr_move_own_errand_hello`: the player has met every villager (and
    /// Tortimer's island visit is allowed, `mSC_check_ArbeitPlayer`).
    pub fn check_hello(&mut self, met_everyone: bool) {
        if self.progress == 2 && met_everyone {
            self.progress = 0;
        }
    }

    /// `aNRG2_job_end_talk_proc`: the job that follows the finished one. After the
    /// planting job, a player who has not met every villager (and Tortimer) goes to
    /// the introductions first.
    pub fn next_job(&self, met_everyone: bool) -> Option<Job> {
        let job = self.job?;
        Some(if job == Job::PlantFlower && !met_everyone {
            Job::Introductions
        } else {
            job.next()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jobs_run_through_chosen_running_finished() {
        let mut fj = FirstJob::start();
        assert_eq!((fj.job, fj.progress), (None, 0));
        fj.setup_job(Job::ChangeCloth);
        assert_eq!(fj.progress, 1);
        assert!(fj.step_up(Picks::default()).is_some());
        assert_eq!((fj.progress, fj.item), (2, WORK_UNIFORM));
        assert!(fj.step_up(Picks::default()).is_none(), "only at progress 1");
        assert!(!fj.finished());
        fj.finish();
        assert!(fj.finished());
        assert_eq!(fj.next_job(true), Some(Job::PlantFlower));
    }

    #[test]
    fn start_checks_need_pocket_room() {
        assert!(!Job::PlantFlower.can_start(9));
        assert!(Job::PlantFlower.can_start(10));
        assert!(!Job::ChangeCloth.can_start(0));
        assert!(Job::ChangeCloth.can_start(1));
        assert!(Job::Open.can_start(0) && Job::PostNotice.can_start(0));
        assert_eq!(Job::PlantFlower.refuse_message(), Some(0x07F5));
        assert_eq!(Job::Open.refuse_message(), None);
        assert_eq!(Job::Open.start_message(), 0x0827);
    }

    #[test]
    fn the_planting_job_gives_seven_flower_bags_and_three_saplings() {
        let mut roll = 0;
        let mut random = |n: i32| {
            roll = (roll + 4) % n;
            roll
        };
        let items = possession(Job::PlantFlower, 0, &mut random);
        assert_eq!(items.len(), 10);
        assert!(items[..7].iter().all(|&(i, c)| {
            (WHITE_PANSY_BAG..WHITE_PANSY_BAG + 9).contains(&i) && c == Cond::Normal
        }));
        assert!(items[7..]
            .iter()
            .all(|&(i, c)| i == SAPLING && c == Cond::Normal));
    }

    #[test]
    fn deliveries_carry_quest_items_and_record_the_villager() {
        let picks = Picks {
            villager: Some(7),
            item: 0x1234,
            slot: 3,
        };
        let mut fj = FirstJob::start();
        fj.setup_job(Job::DeliverFurniture);
        fj.step_up(picks);
        assert_eq!(
            (fj.recipient, fj.used, fj.used_num, fj.pockets_idx, fj.item),
            (Some(7), [Some(7), None], 1, 3, 0x1234)
        );
        assert_eq!(
            possession(Job::DeliverFurniture, fj.item, &mut |_| 0),
            vec![(0x1234, Cond::Quest)]
        );
        // The letter takes the second used slot and lets the guide be asked again.
        fj.setup_job(Job::SendLetter);
        let out = fj
            .step_up(Picks {
                villager: Some(9),
                ..Picks::default()
            })
            .unwrap();
        assert!(out.can_ask_again);
        assert_eq!((fj.used, fj.used_num), ([Some(7), Some(9)], 2));
        // The axe goes to the letter's recipient.
        fj.setup_job(Job::DeliverAxe);
        fj.step_up(Picks {
            slot: 1,
            ..Picks::default()
        });
        assert_eq!((fj.recipient, fj.item), (Some(9), AXE));
        assert_eq!(
            possession(Job::DeliverAxe, AXE, &mut |_| 0),
            vec![(AXE, Cond::Quest)]
        );
    }

    #[test]
    fn the_chain_follows_the_source_table() {
        let next = |job: Job, met| {
            let mut fj = FirstJob::start();
            fj.setup_job(job);
            fj.next_job(met).unwrap()
        };
        assert_eq!(next(Job::PlantFlower, true), Job::DeliverFurniture);
        assert_eq!(next(Job::PlantFlower, false), Job::Introductions);
        assert_eq!(next(Job::DeliverFurniture, true), Job::SendLetter);
        assert_eq!(next(Job::SendLetter, true), Job::Open);
        assert_eq!(next(Job::Introductions, true), Job::DeliverFurniture);
        assert_eq!(next(Job::Open, true), Job::DeliverCarpet);
        assert_eq!(next(Job::DeliverAxe2, true), Job::PostNotice);
    }

    #[test]
    fn the_quest_manager_finishes_jobs_by_what_the_player_does() {
        let mut fj = FirstJob::start();
        fj.setup_job(Job::ChangeCloth);
        fj.step_up(Picks::default());
        fj.check_cloth(0x2400);
        assert_eq!(fj.progress, 2);
        fj.check_cloth(WORK_UNIFORM);
        assert!(fj.finished());
        fj.check_cloth(0x2400);
        assert_eq!(fj.progress, 2, "taking the uniform off reopens the job");

        fj.setup_job(Job::PlantFlower);
        fj.step_up(Picks::default());
        fj.check_seed(true);
        assert_eq!(fj.progress, 2);
        fj.check_seed(false);
        assert!(fj.finished());

        fj.setup_job(Job::SendLetter);
        fj.step_up(Picks::default());
        fj.progress = 3;
        fj.check_letter(false);
        assert_eq!(fj.progress, 3);
        fj.check_letter(true);
        assert!(fj.finished());

        fj.setup_job(Job::Introductions);
        fj.step_up(Picks::default());
        fj.check_hello(true);
        assert!(fj.finished());
    }

    #[test]
    fn the_picker_skips_used_villagers_and_ends_when_all_are_used() {
        let residents = [4, 8, 15, 16];
        let used = [Some(8), None];
        let mut seen = std::collections::BTreeSet::new();
        for roll in 0..3 {
            seen.insert(pick_other_villager(&residents, &used, &mut |n| roll.min(n - 1)).unwrap());
        }
        assert_eq!(seen, [4, 15, 16].into_iter().collect());
        assert_eq!(pick_other_villager(&[8], &used, &mut |_| 0), None);
        assert_eq!(pick_other_villager(&[], &[None, None], &mut |_| 0), None);
    }

    #[test]
    fn the_open_job_raises_its_event_and_has_no_recipient() {
        let mut fj = FirstJob::start();
        fj.recipient = Some(3);
        fj.setup_job(Job::Open);
        let out = fj.step_up(Picks::default()).unwrap();
        assert!(out.open_quest_event && fj.recipient.is_none());
    }
}
