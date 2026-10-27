# Nook's arm gesture for the present hand-over

2026-10-26 claude, Philip: "arm gesture". Follow-up to completed/shop-password-handover.md.

Source: `ac_npc_shop_master_anime.c` row order follows `aNSC_action`; the second
TRANSFER1/TRANS_WAIT1 rows are `PC_PRESENT_TRANS_TAKEOUT` / `_TRANS_WAIT`, then
`PC_PRESENT_END_WAIT` = WAIT1. Clips `cKF_ba_r_npc_1_transfer1` (0x1A) and
`cKF_ba_r_npc_1_trans_wait1` (0x1B) from `ac_npc_data.c_inc`.

- [x] Nook gets an `NpcClip`; hand-over plays TRANSFER1 once, then TRANS_WAIT1 looped,
      then stands again when the player reaches for the present.
- [x] Gates, Temp QA, docs/comms, release claim, commit/push.

## Implemented (2026-10-26, claude)

`shop.rs` spawns the shopkeeper with an `NpcClip`. `shop_nook/handover.rs` starts
`cKF_ba_r_npc_1_transfer1` once with the transfer (8 frame blend, as the
`animation_init_proc` morph), switches to a looped `cKF_ba_r_npc_1_trans_wait1` during
the hold, and stands him up again when the player reaches for the present (also
on leaving the shop). Mapping follows `ac_npc_shop_master_anime.c` row order: the
second TRANSFER1/TRANS_WAIT1 rows are the password present actions, END_WAIT is
WAIT1. The clips load through the existing special-NPC clip path (npc_1 clips on
his rcn rig worked in the preview).

Verified (Temp, disc, synthetic save): fourteen frames after the last message page
show his left arm swinging out as the wrapped present appears in his hand, held
out, then passed to the player's side. No WARN/ERROR in the log. Not checked:
frame-exact sync of the clip length with the 78-frame kernel transfer, the
Timmy/department counter, or the original's spring chase.
