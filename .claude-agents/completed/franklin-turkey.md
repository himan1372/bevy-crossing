# Franklin the turkey (Harvest Festival)

`app/src/franklin.rs`: Franklin (`SP_NPC_TURKEY`, disc model) hides in a cell with blocked neighbours while
`HARVEST_FESTIVAL_FRANKLIN` is active (4th Thursday of November, 15:00-20:00). He speaks first when the player is within
55 units; explanation 0x3BFE..0x3C01, then with a knife and fork (0x2530) in the pockets: 0x3C11+present, the fork is
handed to him, the present (random of the 12 not yet given, 0x32d0.. turkey set, carpet 0x2642, wall 0x2742) is handed
back; without a fork 0x3C02+r(3). He then hides in another spot. Given-bitfield saved as `SaveData.franklin_event`.

## Result
Run in game (Temp save, disc, Nov 26 test day): hides at a tree-side cell, speaks on approach, four explanation
messages, fork message 0x3C15 (harvest chair), fork hand-over, chair hand-over (0x32ec), hides again.
Unverified: no-fork lines, second talk (0x3C05+present), all-12-given reset, the present table ids (checked only for
the chair's name).
Not built: walking/look-around animations, guide arrow, 0x3C1D chatter, other players.
