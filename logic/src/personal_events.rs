//! The part-time-job flags of `m_event.c` (`mEv_SAVED_FIRSTJOB_PLRn`,
//! `_HRAWAIT_PLRn`, `_HRATALK_PLRn`), for one player.
//!
//! `mEv_SetFirstJob` raises FIRSTJOB; `mEv_UnSetFirstJob` (the job is done)
//! drops it and raises HRAWAIT; the next daily renewal
//! (`mEv_RenewalDataEveryDay`) turns HRAWAIT into HRATALK; Tom Nook's start
//! wait (`aNSC_start_wait`) then greets the player with message `-1` once and
//! clears HRATALK. `mEv_ClearPersonalEventFlag` clears all three.

/// First message Nook says on the day after the part-time job
/// (`aNSC_get_msg_no(-1)`).
pub const HRATALK_MESSAGE: usize = 0x082A;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PersonalEvents {
    pub first_job: bool,
    pub hra_wait: bool,
    pub hra_talk: bool,
}

impl PersonalEvents {
    /// `mEv_SetFirstJob`.
    pub fn set_first_job(&mut self) {
        self.first_job = true;
    }

    /// `mEv_UnSetFirstJob`.
    pub fn unset_first_job(&mut self) {
        self.first_job = false;
        self.hra_wait = true;
    }

    /// `mEv_RenewalDataEveryDay`, for a day that has just begun.
    pub fn renew_daily(&mut self) {
        if self.hra_wait {
            self.hra_wait = false;
            self.hra_talk = true;
        }
    }

    /// `mEv_ClearPersonalEventFlag` (these three flags).
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// `mEv_CheckRealArbeit`.
    pub fn real_arbeit(&self) -> bool {
        self.first_job || self.hra_wait
    }

    /// `mEv_CheckArbeit`.
    pub fn arbeit(&self) -> bool {
        self.first_job || self.hra_wait || self.hra_talk
    }

    /// The HRATALK arm of `aNSC_start_wait`: the flag is on, so Nook greets
    /// with `HRATALK_MESSAGE` and the flag goes off as the talk starts.
    pub fn take_hra_talk(&mut self) -> bool {
        std::mem::take(&mut self.hra_talk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_job_leads_to_one_nook_greeting_the_next_day() {
        let mut ev = PersonalEvents::default();
        ev.set_first_job();
        assert!(ev.real_arbeit());
        ev.renew_daily();
        assert!(!ev.hra_talk, "no talk while the job is running");
        ev.unset_first_job();
        assert!(ev.real_arbeit() && ev.arbeit() && !ev.hra_talk);
        ev.renew_daily();
        assert!(!ev.real_arbeit() && ev.arbeit() && ev.hra_talk);
        assert!(ev.take_hra_talk());
        assert!(!ev.take_hra_talk() && !ev.arbeit());
    }

    #[test]
    fn clearing_drops_every_flag() {
        let mut ev = PersonalEvents {
            first_job: true,
            hra_wait: true,
            hra_talk: true,
        };
        ev.clear();
        assert_eq!(ev, PersonalEvents::default());
    }
}
