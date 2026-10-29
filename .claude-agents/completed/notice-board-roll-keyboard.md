# Notice board: cursor roll and keyboard slide-in

Philip chose these (2026-10-29) from the third-session gaps. Source: `m_notice_ovl.c`
(`mNT_roll_control`, `mNT_roll_control2`, `mNT_set_page_dl`), `m_editor_ovl.c`
(`mED_editor_ovl_init`: `mSM_MOVE_IN_BOTTOM`, `mED_..` exit `mSM_MOVE_OUT_BOTTOM`) and
`m_submenu_ovl.c::mSM_move_Move` (the same curve, direction -1).

Plan:
- `logic/src/notice_board.rs::Roll`: `follow(line)` (page_center window of three lines, speed 1..4,
  chase) and `release()` (the end check rolls the page back down, speed up to 16).
- App: the page group is lifted by the roll instead of the fixed `LIFT`; all lines are drawn
  (no five-line window); the keyboard sits in its own group that slides in from the bottom
  (`Slide`, negated) when editing starts and out when the end check opens.

Also from the same request, found moot: the "demolition animation of a removed villager's house"
has no source (`ac_house.c` only has door/open requests; removal is item clearing in
`m_all_grow_ovl.c`). Per-villager memory counts is the next task, separate claim.

## Done (2026-10-29)
- `logic/src/notice_board.rs::Roll`: `follow` (`mNT_roll_control`) and `release` (`mNT_roll_control2`), one own test.
- App: the fixed 100 px lift and the five-line window are gone. The page group is lifted by `roll.position`, every line (up to six) is drawn, the keyboard is its own group sliding from the bottom
  (`Slide`, `mSM_MOVE_IN_BOTTOM`) when editing starts and out when the edit ends (F4 / Esc); input waits for it like the retail editor.
- Driven in a Temp copy of the capture exe (synthetic save): keyboard slide in and out, six lines typed; the page stays put through line 3, then rolls up so the cursor line stays above the keyboard.
- Not done: the editor's own rumble/sound at open (`sAdo_SysTrgStart(0x59)` is for other editor types); the port's own help text rows at the screen top overlap the header once the page has rolled up.
