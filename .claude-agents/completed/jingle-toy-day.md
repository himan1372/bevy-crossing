# Jingle (Santa) on Toy Day

Scope: `SP_NPC_SANTA` while `TOY_DAY_JINGLE` is active (Dec 24 20:00 - Dec 25 00:00): he stands on the beach, answers
with the hide-and-seek talk of `ac_ev_santa_move.c_inc` (hello, two wish questions, a final question, then the
present from the Christmas list by the wished kind) and moves to another beach spot after each talk
(`walk_actor_at_wade`). Per-event save area in `SaveData.jingle_event`.

- [ ] app/src/jingle.rs (spawn, talk, relocation, save)
- [ ] gates, Temp run on Dec 24 evening, backlog result, comms, commit

## Result
Run in game (Temp save, disc, Dec 24 21:00): Jingle (disc model, Santa hat) on the beach; talks in order 0x7AA hello,
0x7AB, 0x7AE, 0x7B2 (questions, first choice answered), then 0x7BA with a Christmas-list piece (0x1E18) handed over;
a later talk in a new acre with the same shirt gets 0x7BF. After each talk he moves to another beach spot (different
acre). State saved as `SaveData.jingle_event`.
Unverified: choosing the second option of the wish questions, the shirt-change path (`0x2B4C`, wish messages, second
present `0x2B59`), full pockets (`0x7B9`/`0x2B58`), ten presents (`0x2B5B..`), same-acre talk (`0x7BC..`).
Not built: wandering between talks, border-acre rule, present animation, other players (0x7C2..0x7C4).
