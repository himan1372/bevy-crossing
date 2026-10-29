//! `mNpc_ForceRemove` and `mNpc_GetGoodbyAnimalIdx` (`m_npc.c`): when the town is full and no
//! one has moved in or out for ten days, the villager the player knows least leaves at the
//! next start.

/// `ANIMAL_NUM_MAX`.
pub const ANIMAL_NUM_MAX: usize = 15;
/// `mNpc_MINIMUM_DAYS_BEFORE_FORCE_REMOVAL`.
pub const MINIMUM_DAYS: i64 = 10;
/// `0x20E`: the first goodbye letter (`mNpc_SetGoodbyMailData`); three per personality.
const GOODBYE_MAIL: usize = 0x20E;

/// One resident as `mNpc_CheckGoodbyAnimalMemoryNum` compares them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Candidate {
    /// The animal slot (`animals[i]`).
    pub slot: usize,
    /// `mNpc_GetAnimalMemoryNum`: players who have met them.
    pub memories: u32,
    /// `mNpc_GetAnimalMemoryLetterNum`: memories with a letter.
    pub letters: u32,
    /// `mNpc_GetAnimalMemoryLandKindNum`: different towns among the memories.
    pub lands: u32,
    /// `mNpc_GetSameLooksNum`: residents with their personality.
    pub same_looks: u32,
}

/// `mNpc_ForceRemove`'s gate: a full town and a removal date at least ten days old.
pub fn due(residents: usize, remove_day: Option<i64>, today: i64) -> bool {
    residents == ANIMAL_NUM_MAX && remove_day.is_some_and(|d| (today - d).abs() >= MINIMUM_DAYS)
}

/// `mNpc_CheckGoodbyAnimalMemoryNum(a0, a1)`: 1 if `new` replaces `best`, -1 if it joins it as
/// a candidate, 0 if it is passed over.
fn compare(new: &Candidate, best: &Candidate) -> i32 {
    // The source's order: memories, letters, towns, then (as written) the personality count.
    for (a, b) in [
        (new.memories, best.memories),
        (new.letters, best.letters),
        (new.lands, best.lands),
    ] {
        if b > a {
            return 1;
        }
        if b < a {
            return 0;
        }
    }
    if best.same_looks < new.same_looks {
        1
    } else {
        -1
    }
}

/// `mNpc_GetGoodbyAnimalIdx`: the slot of the villager who leaves. `random(n)` is `RANDOM(n)`.
pub fn goodbye_slot(candidates: &[Candidate], random: &mut dyn FnMut(u32) -> u32) -> Option<usize> {
    let mut best: Option<Candidate> = None;
    let mut chosen = Vec::new();
    for c in candidates {
        match best {
            None => {
                best = Some(*c);
                chosen = vec![c.slot];
            }
            Some(b) => match compare(c, &b) {
                1 => {
                    best = Some(*c);
                    chosen = vec![c.slot];
                }
                -1 => chosen.push(c.slot),
                _ => {}
            },
        }
    }
    match chosen.len() {
        0 => None,
        1 => Some(chosen[0]),
        n => Some(chosen[random(n as u32) as usize % n]),
    }
}

/// The goodbye letter's mail number (`0x20E + looks * 3 + RANDOM(3)`).
pub fn goodbye_mail(looks: u8, roll: u32) -> usize {
    GOODBYE_MAIL + usize::from(looks) * 3 + (roll as usize).min(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(slot: usize, memories: u32, letters: u32, same_looks: u32) -> Candidate {
        Candidate {
            slot,
            memories,
            letters,
            lands: memories.min(1),
            same_looks,
        }
    }

    #[test]
    fn it_needs_a_full_town_and_ten_days() {
        assert!(!due(14, Some(0), 100));
        assert!(!due(15, None, 100));
        assert!(!due(15, Some(95), 100));
        assert!(due(15, Some(90), 100));
    }

    #[test]
    fn the_least_known_villager_leaves() {
        let all = [c(0, 1, 1, 2), c(1, 0, 0, 1), c(2, 1, 0, 2)];
        assert_eq!(goodbye_slot(&all, &mut |_| 0), Some(1));
        // Equal in everything: the more common personality leaves; ties pick by RANDOM.
        let tie = [c(0, 1, 1, 1), c(1, 1, 1, 3), c(2, 1, 1, 3)];
        assert_eq!(goodbye_slot(&tie, &mut |_| 0), Some(1));
        assert_eq!(goodbye_slot(&tie, &mut |n| n - 1), Some(2));
        assert_eq!(goodbye_slot(&[], &mut |_| 0), None);
    }

    #[test]
    fn the_letter_depends_on_personality() {
        assert_eq!(goodbye_mail(0, 0), 0x20E);
        assert_eq!(goodbye_mail(5, 2), 0x20E + 15 + 2);
    }
}
