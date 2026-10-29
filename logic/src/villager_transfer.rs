//! A villager moving between towns (`mNpc_GetRemoveAnimal`, `m_npc.c`): the villager who is
//! thinking of moving (`remove_animal_idx`) leaves with the player who goes to another town, and
//! the other town takes them in. Pinned Rustimal `npc_life.rs` has the same two rules.

/// `mNpc_GetRemoveAnimal`, the town that receives the villager: refused when they are already
/// on their way out, live here, were the last villager to leave, or are the summer camper.
pub fn may_move_in(
    incoming: u32,
    removing: bool,
    town: &[u32],
    last_removed: Option<u32>,
    summer_camper: Option<u32>,
) -> bool {
    !removing
        && !town.contains(&incoming)
        && last_removed != Some(incoming)
        && summer_camper != Some(incoming)
}

/// Where an accepted villager goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveInSlot {
    /// A free slot: the town grows.
    Free(usize),
    /// Replaces a resident, whose house is destroyed and who sends a goodbye letter.
    Replace(usize),
}

/// The first free slot, else the goodbye villager's, else slot 0 (`mNpc_GetFreeAnimalInfo`,
/// `mNpc_GetGoodbyAnimalIdx`).
pub fn move_in_slot(free: Option<usize>, goodbye: Option<usize>) -> MoveInSlot {
    match free {
        Some(slot) => MoveInSlot::Free(slot),
        None => MoveInSlot::Replace(goodbye.unwrap_or(0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_villager_may_move_in_unless_a_rule_refuses() {
        assert!(may_move_in(7, false, &[1, 2], None, None));
        assert!(!may_move_in(7, true, &[1, 2], None, None));
        assert!(!may_move_in(2, false, &[1, 2], None, None));
        assert!(!may_move_in(7, false, &[1, 2], Some(7), None));
        assert!(!may_move_in(7, false, &[1, 2], None, Some(7)));
    }

    #[test]
    fn a_full_town_replaces_the_goodbye_villager_or_slot_zero() {
        assert_eq!(move_in_slot(Some(3), Some(5)), MoveInSlot::Free(3));
        assert_eq!(move_in_slot(None, Some(5)), MoveInSlot::Replace(5));
        assert_eq!(move_in_slot(None, None), MoveInSlot::Replace(0));
    }
}
