//! The museum's collection (m_museum_display) and Blathers' answers when
//! something is offered (ac_npc_curator: aCR_get_idx_to_donate_*,
//! aCR_msg_win_open_wait, aCR_set_talk_info).
//!
//! Each of the 25 fossils, 15 paintings, 40 insects and 40 fish records
//! who gave it: 0 nobody, 1-4 a player, 5 a player since removed.

/// mMmd_FOSSIL_NUM / ART / INSECT / FISH.
pub const FOSSIL_NUM: usize = 25;
pub const ART_NUM: usize = 15;
pub const INSECT_NUM: usize = 40;
pub const FISH_NUM: usize = 40;
pub const TOTAL: usize = FOSSIL_NUM + ART_NUM + INSECT_NUM + FISH_NUM;

/// FTR_START(FTR_DIN_TRIKERA_HEAD) .. FTR_END(FTR_DIN_TRILOBITE).
pub const FOSSIL_FIRST: u16 = 0x1EEC;
pub const FOSSIL_LAST: u16 = 0x1F4F;
/// FTR_START(FTR_SUM_ART01) .. FTR_END(FTR_SUM_ART15).
pub const ART_FIRST: u16 = 0x12AC;
pub const ART_LAST: u16 = 0x12E7;
/// ITM_INSECT_START .. ITM_SPIRIT0 (exclusive); ITM_FISH_START .. END.
pub const INSECT_FIRST: u16 = 0x2D00;
pub const FISH_FIRST: u16 = 0x2300;
/// ITM_FOSSIL: an unidentified fossil.
pub const UNIDENTIFIED_FOSSIL: u16 = 0x2511;
/// ITM_INSECT28: the cockroach.
const COCKROACH: u16 = INSECT_FIRST + 28;
/// FTR_START of the single-piece fossils.
const TRILOBITE: u16 = 0x1EEC + 24 * 4;
const AMMONITE: u16 = 0x1EEC + 22 * 4;
const EGG: u16 = 0x1EEC + 23 * 4;
const STUMP: u16 = 0x1EEC + 21 * 4;
const AMBER: u16 = 0x1EEC + 20 * 4;
/// FTR_START(FTR_SUM_ART02 / ART03): Blathers turns these away.
const ART02: u16 = ART_FIRST + 4;
const ART03: u16 = ART_FIRST + 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Fossil,
    Art,
    Insect,
    Fish,
}

/// The exhibit an item belongs to, and its index there.
pub fn exhibit(item: u16) -> Option<(Category, usize)> {
    match item {
        FOSSIL_FIRST..=FOSSIL_LAST => {
            Some((Category::Fossil, ((item - FOSSIL_FIRST) >> 2) as usize))
        }
        ART_FIRST..=ART_LAST => Some((Category::Art, ((item - ART_FIRST) >> 2) as usize)),
        i if (INSECT_FIRST..INSECT_FIRST + INSECT_NUM as u16).contains(&i) => {
            Some((Category::Insect, (i - INSECT_FIRST) as usize))
        }
        i if (FISH_FIRST..FISH_FIRST + FISH_NUM as u16).contains(&i) => {
            Some((Category::Fish, (i - FISH_FIRST) as usize))
        }
        _ => None,
    }
}

/// `mMmd_info_c`: who gave each exhibit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collection {
    pub fossil: [u8; FOSSIL_NUM],
    pub art: [u8; ART_NUM],
    pub insect: [u8; INSECT_NUM],
    pub fish: [u8; FISH_NUM],
}

impl Default for Collection {
    fn default() -> Self {
        Self {
            fossil: [0; FOSSIL_NUM],
            art: [0; ART_NUM],
            insect: [0; INSECT_NUM],
            fish: [0; FISH_NUM],
        }
    }
}

/// What Blathers says to an offered item, and whether he keeps it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Answer {
    pub msg: usize,
    pub accepted: bool,
}

impl Collection {
    fn slot(&mut self, cat: Category, idx: usize) -> Option<&mut u8> {
        match cat {
            Category::Fossil => self.fossil.get_mut(idx),
            Category::Art => self.art.get_mut(idx),
            Category::Insect => self.insect.get_mut(idx),
            Category::Fish => self.fish.get_mut(idx),
        }
    }

    pub fn donor(&self, cat: Category, idx: usize) -> u8 {
        let list: &[u8] = match cat {
            Category::Fossil => &self.fossil,
            Category::Art => &self.art,
            Category::Insect => &self.insect,
            Category::Fish => &self.fish,
        };
        list.get(idx).copied().unwrap_or(0)
    }

    /// mMmd_CountDisplayed* over every exhibit.
    pub fn count(&self) -> usize {
        [&self.fossil[..], &self.art, &self.insect, &self.fish]
            .iter()
            .flat_map(|l| l.iter())
            .filter(|&&d| (1..=5).contains(&d))
            .count()
    }

    /// mSM_check_item_for_curator: may the item be picked to offer?
    pub fn offerable(&self, item: u16) -> bool {
        item == UNIDENTIFIED_FOSSIL || exhibit(item).is_some()
    }

    /// aCR_msg_win_open_wait for an item offered by `player` (1-4):
    /// Blathers' line, and the record set if he keeps it
    /// (mMmd_RequestMuseumDisplay). `pick3` chooses among his three thanks
    /// for a painting (RANDOM(3)).
    pub fn offer(&mut self, item: u16, player: u8, pick3: u32) -> Answer {
        const REFUSE: [usize; 4] = [0x2F6B, 0x2F6C, 0x2F6E, 0x2F6D];
        const OTHER: [usize; 4] = [0x2F67, 0x2F68, 0x2F6A, 0x2F69];
        const DELETED: [usize; 4] = [0x2F90, 0x2F91, 0x2F93, 0x2F92];
        let Some((cat, idx)) = exhibit(item) else {
            let msg = if item == UNIDENTIFIED_FOSSIL {
                0x2F65
            } else {
                0x2F64
            };
            return Answer {
                msg,
                accepted: false,
            };
        };
        let k = cat as usize;
        if cat == Category::Art && (item == ART02 || item == ART03) {
            return Answer {
                msg: 0x3A83,
                accepted: false,
            };
        }
        let donor = self.donor(cat, idx);
        let msg = match donor {
            d if d == player => REFUSE[k],
            0 => match cat {
                Category::Fossil => match item {
                    TRILOBITE => 0x2F7F,
                    AMMONITE => 0x2F80,
                    EGG => 0x2F81,
                    STUMP => 0x2F82,
                    AMBER => 0x2F83,
                    _ => 0x2F8F,
                },
                Category::Art => 0x2F70 + (pick3 % 3) as usize,
                Category::Insect if item == COCKROACH => 0x2F87,
                Category::Insect => 0x2F86,
                Category::Fish => 0x2F89,
            },
            5 => DELETED[k],
            _ => OTHER[k],
        };
        let accepted = donor == 0;
        if accepted {
            if let Some(slot) = self.slot(cat, idx) {
                *slot = player;
            }
        }
        Answer { msg, accepted }
    }
}

impl Collection {
    /// aCR_get_msg_no_after_talk: how the collection is coming along.
    pub fn after_talk(&self) -> usize {
        match self.count() {
            0 => 0x2F58,
            1..=36 => 0x2F59,
            37..=84 => 0x2F5A,
            _ => 0x2F5B,
        }
    }

    fn complete(&self, cat: Category) -> bool {
        let list: &[u8] = match cat {
            Category::Fossil => &self.fossil,
            Category::Art => &self.art,
            Category::Insect => &self.insect,
            Category::Fish => &self.fish,
        };
        list.iter().all(|d| (1..=5).contains(d))
    }

    /// What Blathers goes on to say after keeping `item` (already
    /// recorded): aCR_putaway_demo_end_wait and what follows it. A fossil
    /// part may finish its skeleton (0x2F78 + kind, else 0x2F84); an insect
    /// or fish gets his remarks on it; then either "anything else?"
    /// (0x2F73, a question) or that category's completion and 0x2F76 /
    /// 0x2F77 when everything is.
    pub fn after_donation(&self, item: u16) -> Vec<usize> {
        let Some((cat, idx)) = exhibit(item) else {
            return Vec::new();
        };
        let mut chain = Vec::new();
        match cat {
            Category::Fossil if idx < 20 => {
                // aCR_chk_fossil_parts_complete: the skeletons' pieces.
                const KINDS: [std::ops::Range<usize>; 7] =
                    [0..3, 3..6, 6..9, 9..12, 12..15, 15..18, 18..20];
                let kind = KINDS.iter().position(|r| r.contains(&idx));
                let done = kind
                    .is_some_and(|k| KINDS[k].clone().all(|i| (1..=5).contains(&self.fossil[i])));
                chain.push(match kind {
                    Some(k) if done => 0x2F78 + k,
                    _ => 0x2F84,
                });
            }
            Category::Insect if item != COCKROACH => chain.push(INSECT_TALK[idx]),
            Category::Fish => chain.push(FISH_TALK[idx]),
            _ => {}
        }
        const COMPLETE: [usize; 4] = [0x2F85, 0x2F75, 0x2F88, 0x2F8A];
        if self.complete(cat) {
            chain.push(COMPLETE[cat as usize]);
            chain.push(if self.count() == TOTAL {
                0x2F77
            } else {
                0x2F76
            });
        } else {
            chain.push(0x2F73);
        }
        chain
    }
}

/// aCR_set_after_explain_insect: his remarks on each insect.
const INSECT_TALK: [usize; INSECT_NUM] = [
    0x3A56, 0x3A56, 0x3A56, 0x3A56, 0x3A57, 0x3A57, 0x3A5D, 0x3A57, 0x3A5E, 0x3A58, 0x3A58, 0x3A58,
    0x3A5F, 0x3A59, 0x3A59, 0x3A5A, 0x3A5A, 0x3A5A, 0x3A5A, 0x3A5B, 0x3A5B, 0x3A5B, 0x3A61, 0x3A62,
    0x3A5C, 0x3A5C, 0x3A63, 0x3A64, 0x3A56, 0x3A5B, 0x3A5B, 0x3A60, 0x3A6B, 0x3A69, 0x3A65, 0x3A6C,
    0x3A68, 0x3A6A, 0x3A66, 0x3A67,
];

/// aCR_putaway_demo_start_wait5_2: his remarks on each fish.
const FISH_TALK: [usize; FISH_NUM] = [
    0x3A6D, 0x3A6E, 0x3A6D, 0x3A80, 0x3A6D, 0x3A6D, 0x3A6E, 0x3A6F, 0x3A6D, 0x3A70, 0x3A71, 0x3A6D,
    0x3A6E, 0x3A72, 0x3A6D, 0x3A6D, 0x3A6D, 0x3A73, 0x3A6E, 0x3A81, 0x3A6E, 0x3A74, 0x3A6D, 0x3A7D,
    0x3A75, 0x3A76, 0x3A6E, 0x3A6E, 0x3A7E, 0x3A77, 0x3A7D, 0x3A79, 0x3A7C, 0x3A82, 0x3A77, 0x3A7F,
    0x3A6D, 0x3A7A, 0x3A7B, 0x3A78,
];

/// aCR_check_sleep_time: Blathers sleeps from 06:00 to 18:00.
pub fn asleep(day_secs: u32) -> bool {
    (6 * 3600..18 * 3600).contains(&(day_secs % 86400))
}

/// aCR_set_talk_info's greeting for a local player. In his sleep hours
/// `standing` (aCR_ACTION_SLEEP_WAIT, not yet nodded off) gets the plain
/// welcome; asleep he wakes with a start.
pub fn greeting(complete: bool, first_talk: bool, day_secs: u32, standing: bool) -> usize {
    const MSG: [[[usize; 3]; 2]; 2] = [
        [[0x2F4E, 0x2F4F, 0x2F4F], [0x2F51, 0x2F96, 0x2F52]],
        [[0x2F53, 0x2F54, 0x2F54], [0x2F55, 0x2F94, 0x2F56]],
    ];
    let action = if asleep(day_secs) {
        usize::from(standing)
    } else {
        2
    };
    MSG[usize::from(complete)][usize::from(!first_talk)][action]
}
