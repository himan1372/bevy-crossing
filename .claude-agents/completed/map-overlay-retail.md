# Retail town map overlay

Philip: "do two" (2026-10-28), candidate 2 of the handoff: the retail map and notice overlays, both still plain panels.
Order: the map first (`m_map_ovl.c`, replaces `app/src/town_map.rs`'s text grid), then the notice board (`m_notice_ovl.c`, `app/src/notice_board.rs`).

## Plan
1. Source audit of `m_map_ovl.c` draw path (acre textures `kan_tizu_*` by block type and palette, `kan_win_*` frame/labels/cursor, house markers, cursor motion `add_calc` 0.7/19/1.8, close sounds).
2. Disc lookups with `skills/gc-iso-reader` (symbols, sizes, formats) before any Rust.
3. Build the overlay on the disc renderer used by pockets/messages; keep the plain panel as the no-disc fallback.
4. QA in a Temp capture copy with a synthetic save; gates: fmt, clippy -D warnings, capture build.
5. Notice overlay the same way.

## Implemented (2026-10-28, claude) - map half
Source: `m_map_ovl.c` (mMP_*), `m_submenu_ovl.c` (`mSM_move_menu`, slide data). All pictures, window, labels and cursor come from the disc's display lists.
- Kernel `logic/src/town_map.rs` (2 own tests): acre pictures by block type (`mMP_make_max_no_table`: types with no picture are skipped, not blanked), building labels (`mMP_set_field_data`), label words/offsets,
  house-mark index from `mMP_house_pos_list` (parsed from the disc), house layer (`mMP_check_layer`), cursor step/pulse/glide (`add_calc` 0.7/19/1.8, 18-frame pulse tables).
- `app/src/town_map.rs`: retail overlay when the disc and a retail layout exist (plain text grid kept as the fallback). Window `kan_win_model`, acre pictures `kan_tizu_model` with
  `l_map_texture`/`l_map_pal`/`kan_tizu{1,2}_pal` in segments 8/9, coordinates and label bubble `kan_win_model2` (+ `kan_win_waku*` in segment A, number/letter textures in 8/9),
  label icons, villagers' house marks (`kan_win_npc2T_*`), the "you are here" mark, the pulsing cursor, the town name (blue, X-opened colour). Slides in from the top and out upward
  (`mSM_move_menu` with the source's speeds), sounds 0x17C / 0x17D / cursor, holding a direction repeats (25 then every 3 ticks).
- `app/src/disc_gfx.rs`: `RelGfx::with_dl_segments` + `run_display_lists_with_segment_lists`: a `G_DL` into a runtime segment that game code points at a display list (the window's colour list in segment B,
  the label area's list in segment A) is followed instead of skipped.
Verified (Temp capture copy, synthetic save, disc): X opened the map, screenshots show the window, A-F / 1-5 edges, acre pictures, cursor on the player's acre (A-2 "Shop"), B-3 listing
"QA, free, free, free" with the four player houses, C-5 listing the two villagers ("Freya", "Grizzly") with their marks; moving the cursor with W/A/S/D glides; X closes and gives control back. No ERROR/panic in the log.
Gates: fmt, clippy -D warnings (workspace, all targets), capture build. Real save hash unchanged.
Not done / uncertain: the house mark index bias (`HOUSE_UT_Z_BIAS`, the plan's house unit vs the source's `home_info.ut_z`; both villager marks looked plausible but were not compared with retail),
the house layer heights (`tier_height` 0/160/320 chosen from the layer thresholds), the bridge picture (`pluss_bridge` is read but the app has no built-bridge state, so never swapped),
opening from the pause menu (`data0 == 0`, green colour list and red town name), text anchor/size checked only by eye, the real X-button open/close timing, and the pad.
