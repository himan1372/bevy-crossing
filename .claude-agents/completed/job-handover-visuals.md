# Job item hand-over visuals

Philip: "hand over and wipe" (2026-10-28). The guide gives the job's items directly (no hand-over frames); build the source's hand-over for them with the generic `shop_nook/handover.rs`, then the scene wipe.

## Result (2026-10-28, claude)
Source: `aNRG2_demo_start_wait_talk_proc` .. `_demo_end_wait2_talk_proc` (the start message raises order NPC0/1 = 4, the guide hands over `itemNo[job]` with after-mode 7 and locks the message until the hand-over ends), `ac_handOverItem*.c_inc` (`mNT_get_itemTableNo` draw types, `mode_DL_table` / `vtx_DL_table`).
- `shop_nook/handover.rs`: `Item::Goods(item)` draws any item as its hand-over table model (item1 categories via `item1_tableNo$430`; furniture is the leaf), lists read from the disc's `mode_DL_table`/`vtx_DL_table`.
- `msg_window.rs`: `DemoOrders::take_raised` hands raised orders to actors once. `dialogue.rs`: `DialogueState.held` (`mMsg_SET_LOCKCONTINUE`) blocks continuing a page.
- `shop_nook/guide.rs::hand_over_job_item`: on (1, 1, 4) during the job it starts Nook -> player with `Job::display()` and holds the page until the hand-over is done (pockets were already filled by the start).
Verified (Temp capture copy, synthetic save, disc): job 1 start: "hand-over of the job item 0x2410", phases Out -> Wait -> Get -> Away ran (log), the message stayed locked until the end, a model appeared at the player's hand; the pockets held 0x2410. Gates: fmt, clippy -D warnings, capture build. Real save unchanged.
Not checked: the other items' models (bag, paper, carpet, axe, furniture leaf), Nook's hand is hidden by the shop camera, the item's texture is whatever the model's lists carry (the original sets no segment either).
