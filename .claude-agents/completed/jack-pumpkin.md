# Jack the pumpkin (Halloween)

Scope: `SP_NPC_HALLOWEEN` (`ac_ev_pumpkin*.c`) on the beach while HALLOWEEN is active; greeting 0x98A, question 0x9B5,
treat (pockets give mode: candy -> present, tool -> back + trick, other -> trick), trick, same-acre lines 0x9BC+r(6),
moves to another beach spot after each talk. No save area (common area only).

- [ ] app/src/jack.rs, wire in main.rs; halloween::trick / is_tool pub(crate)
- [ ] gates, Temp run Oct 31 evening, result, comms, commit

## Result
Run in game (Temp save, disc, Oct 31 evening): Jack (pumpkin head) on the beach; E talk: 0x98A, question 0x9B5, Treat
opened the pockets, candy picked: taken (hand-over), 0x9B9, a Halloween-list piece (spooky table 0x14D8) handed over,
0x9BA, then he walked to another beach spot (different acre).
Unverified: Trick answer, nothing picked (0x9B6), tool (0x9BB), other item (0x9B8), the trick itself (shared with
`halloween::trick`), same-acre lines 0x9BC.., carpet/wall presents.
Not built: wandering between talks, present box animation (the piece itself is handed over).
