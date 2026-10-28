# Map hand-over animation

Philip: "fade out and map handover" (2026-10-28). The guide gives the map after the third job (0x0801 end talk, 0x0816 demo3) with a hand-over; the app sets `map_flag` with no item model. Build the hand-over (map item model) with the generic `handover.rs`.

## Result (2026-10-28, claude)
Source: `aNRG2_3rd_job_end_talk_proc` (order (NPC0, 9) of the third job's end talk 0x0801 -> job_end, order (NPC0,1) = 2, item ITM_TOWN_MAP (0x251D) with after-mode 7, `map_flag` on), the generic `ac_handOverItem`.
- `guide.rs`: the third job's end sets `GuideState.map_handover`; when the message raises order (1, 9) (`hand_over_job_item`) Nook hands over `Item::Goods(0x251D)` and the page is held until it is put away. `map_flag` is still set when the talk is built.
Verified (Temp capture copy, synthetic save: job 3 done, map_flag off): log `message order kind 1 index 9 value 0x0001` -> `hand-over of the town map` -> phases 2, 3, 7 -> next message 0x0802. Screenshots do not show the map clearly (the shop camera hides Nook's hand; frames were taken as the item went into the pockets). Gates: fmt, clippy, capture build.
Not covered: the map model's appearance (0x251D's table entry draws whatever `item1_tableNo` gives) and the map item in the pockets.
