# Drive the unverified warning panels

Philip (2026-10-27): "drive the unverified warnings": MUSIC, MUSIC2, PUT_MAX_FURNITURE, PUT_FURNITURE, PUT_ITEM, WORD_OVER
(diary and letters), and look at the Yes/No cursor model. Evidence only; fix what the runs show.

## Result (2026-10-27, claude)
Driven in Temp (capture build, disc, synthetic home/town saves; screenshots inspected; no WARN/ERROR in the logs):
- PUT_MAX_FURNITURE ("You can't put any more items out!"): home layer 0 filled with 256 furniture cells, Place in room on a bed.
- PUT_FURNITURE ("...any more items there!"): layer 0 filled with multi-cell filler values (65055), same action.
- PUT_ITEM ("You can't drop any more items here!"): the same filled room with an apple.
- MUSIC2 ("There's already music in there!"): mini-disc player placed from the pockets (furniture item 0x1E5C = `FTR_SUM_MD01`), device menu
  (stand in front of it, facing it: its front is +z), Register a song with a song that is already in `music_box`.
- MUSIC ("You can't pull anything out when your item screen is full!"): two songs registered, take one out (fills the last free
  pocket), take the second.
- WORD_OVER ("You don't have room to write any more."): diary at 992/992 and the letter compose view (1200 typed characters).
Each panel closed with Enter and returned the screen (furniture placement, pockets, grid, editor) to a usable state.
Found and fixed: the music collection grid kept stale visuals after its warning (black cloth only); its draw key now includes
`grid.warning`, so it rebuilds when the warning flag flips.
Not resolved: the Yes/No cursor model (`att_win_cursor_model`, a red 10-pixel triangle) is created (one mesh at the expected
spot, prim colour now red) but is not visible in screenshots; the cause is not found (mode list, render mode or culling).
Harness notes: turning to face something needs a held key (a 100 ms tap does not set the facing), and the mini-disc player only
answers E from its front (`facing . front <= -0.9`).
Gates: fmt, clippy -D warnings, default build; no release build, no tests.
