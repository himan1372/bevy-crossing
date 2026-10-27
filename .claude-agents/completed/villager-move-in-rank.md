# Villager move-in uses the field rank

Claimed by claude 2026-10-27. Philip chose it after shop-rare-notice.

Source: `m_npc.c` mNpc_CheckGrowFieldRank (npc_grow_prob 40..100 by rank 0..6, kernel
`logic/villager_home.rs::check_grow_field_rank`), mNpc_CheckGrow, start-up order in
`m_start_data_init.c` (mFAs_SetFieldRank, then mNpc_Grow).

Problem: `roster::init_roster` runs at Startup before the town exists, so the check used a fixed 70%.
Plan: `grow` takes a lazy `rank` closure, called only once the cheap gates (room, new day, everyone
met) pass; `field_rank::rank_at_start` builds the retail layout's field items from the seed,
applies the save's field changes and trash flag, and scores them with the existing kernel.

## Implemented (2026-10-27, claude)

`roster.rs`: `TownRoster::grow` takes a lazy `rank` closure and applies `mNpc_CheckGrowFieldRank`
(`npc_grow_prob` 40/50/60/70/80/90/100 by rank 0..6, `rustimal_logic::check_grow_field_rank`,
re-exported from the private `villager_home` module) after the cheap gates; the fixed 70% is gone.
`field_rank.rs::rank_at_start` builds the retail layout's field items for the seed, applies the
save's field changes and trash flag and scores them with the existing kernel; the closure runs
only when a move-in could happen. The existing roster unit tests pass `|| Some(3)` (same 70%).

Verified (Temp, disc, synthetic save with every villager met and an old last move-in day): the
start-up rank logged as 5, the same rank the in-game scan produced a moment later; Anicotti moved
in. No WARN/ERROR. Not driven: a low-rank town (needs a real trashed field), which only changes
the table lookup; fish spawn rank scaling is still open.
