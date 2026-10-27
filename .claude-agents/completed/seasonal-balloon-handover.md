# Sale-day balloon hand-over

2026-10-27 claude, Philip: "seasonal services". Slice of the remaining seasonal work (see
completed/shop-bargain-sales.md limits). Source: `ac_npc_shop_common.c` aNSC_present_balloon_*
(START_WAIT -> TRANS_MOVE -> TRANS_TAKEOUT -> TRANS_WAIT -> END_WAIT; `birth_proc(ITM_BALLOON_START,
PUTAWAY, aHOI_PRESENT)`), `aNSC_check_present_balloon` (event shop status, first-talk flag, free pocket).

- [x] Play the generic hand-over (`Item::Present`) when the sale-day balloon is given.
- [x] Confirm TRANS_MOVE (step to aNSC_POS_Z_MAX) is covered by the existing counter follow.
- [x] Temp QA: gift (log hand-over Present -> 2,3,7, pocket 0x2244..0x224B, latch saved, no second gift); full pockets: none.
- [x] Also covers the Nook-present path with the new player arm clips (not re-QA'd last task).
- [x] Record blocked items: identity/rank/assessment RNG inputs, spotlight rare_item exclusion, escort/twin (unverified meaning), grab-bag mWR panel.
- [x] Gates, docs/comms, release claim, commit/push.

## Implemented (2026-10-27, claude)
`shop_nook.rs` Step::BargainBalloon now also starts `handover.start(Request::nook_present())` after the pocket write (item 0x2244.. cond wrapped), so the sale-day first-talk balloon uses the generic hand-over (Nook transfer/wait clips, present model, player get/put-away clips). TRANS_MOVE (step to aNSC_POS_Z_MAX) is covered by the existing counter follow (Nook already tracks the player along the counter); not separately choreographed. The hand-over starts after the 218/220 message ends, not mid-message as the original's demo order does (message text order not audited).
Verified (Temp, disc, synthetic save: sale today 14:xx, balloon not yet given): log `hand-over Present -> 2, 3, 7`; pocket 0x2244..224B with condition 1; Bells and sales unchanged; F9 reload shows no second gift; with full pockets no gift and no hand-over; no WARN/ERROR. This run also exercises the Nook present path with the player's new arm clips (the earlier gap) without errors; frames were not eyeballed.
Blocked/not done (unchanged): identity/rank/assessment RNG inputs (need personal IDs and field assessment), spotlight rare_item exclusion (no spotlight stock field), escort/twin choreography (meaning not verified), grab-bag capacity warning panel geometry.
