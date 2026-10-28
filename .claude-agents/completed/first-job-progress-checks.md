# First job: the per-frame progress checks

Philip: "go" on slice 2 (guide NPC + stage 1), 2026-10-28. Reading the source for it showed the finish rules live in the quest manager (`ac_quest_errand.c_inc`
`aQMgr_move_own_errand_*`), not the guide, and that finishing stage 1 needs the player to wear a shirt from the pockets, which the app cannot do yet. This task ports those
rules into `logic/src/first_job.rs` (the part of slice 2 that is independent) and records the blockers.

## Result (2026-10-28, claude)
`logic/src/first_job.rs` now has the quest manager's finish rules: `check_cloth(worn)` (running + wearing the errand shirt = done; taking it off reopens it), `check_seed(plant_in_pockets)`,
`check_letter(recipient_has_letter)`, `check_hello(met_everyone)`; one more own test (seven pass). Gates: fmt, clippy -D warnings, `cargo test -p rustimal-logic first_job`.

Blockers found for slice 2 (guide NPC + stage 1), not built:
1. **The player cannot wear a shirt from the pockets.** Stage 1 finishes when `cloth.item == ITM_CLOTH016`. The app's shirt is a fixed `Look.cloth` (random new-boy look) plus the shop try-on preview;
   there is no Wear action in the pockets menu and no persisted worn shirt that drives the model (`player_cloth` in the save is not applied at runtime). A pockets "Change clothes" action, persistence and
   the player's texture swap are a prerequisite (also needed for `aNRG2_cloth_check`, the "why did you change out of your uniform" scolding).
2. **Demo-order codes in the messages.** The guide's messages carry `SET_DEMO_ORDER_NPC0` codes (code 0x09, `[index:u8][value:u16]`; e.g. 0x07F1 has `01 0004` = "show the job's item now", 0x07EE/0x0837 set emote values
   0x16/0x15/0xFF). `app/src/messages.rs::render` drops them, so the hand-over cannot be timed to the page where the original shows the item; they would need to be kept as a page marker.
3. **Hand-over of an arbitrary item.** `shop_nook/handover.rs` only draws the present, money bag and turnips; the guide hands over the uniform / flower bag / paper / axe item icons (`mNT_ITEM_TYPE` item draw).
4. **The guide's talk table** (46 normal and 5 forced entries, 21 talk procs) and the shop's behaviour change while the job runs (guide NPC instead of the shop master; goods talk 0x081C; the stair block 0x0834)
   are still to port; the message numbers are listed in the source note above and in `ac_npc_rcn_guide2*.c_inc`.
Suggested order: (a) wear a shirt from the pockets, (b) order markers in messages, (c) the guide flow for stage 1 with direct item grants (hand-over visual later), then stages 2 and 3.
