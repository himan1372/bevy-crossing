# Map from the pause menu

Philip: "do job gaps and open from pause" (2026-10-28). The retail map (`m_map_ovl.c`) opens with data0 == 1 from X and data0 == 0 from the menu: colour list `kan_win_color0_mode`, town name (255,0,255), and X does not close it. Read `pause_menu.rs` for the menu's map entry.

## Result (2026-10-28, claude)
Correction: the source's `data0 == 0` map (`mSM_OVL_MAP, 0`) is opened from the map signboards (`MAP_BOARD0/1` = field items 0x000C/0x000D, `mSM_submenu_ctrl`: the action button facing one), not from the pause menu. X is `data0 == 1`.
`town_map.rs`: the action button facing a map signboard opens the map without needing `map_flag`; it uses `kan_win_color0_mode` (yellow/orange window) and the magenta town name, and X does not close it (Cancel/Back/Confirm/Interact still do).
Verified (Temp capture copy, synthetic save, board at (-1.5, -40.5)): window and town name colours as above, X left it open, Escape closed it, no panic. Gates: fmt, clippy, capture build. Not checked: the 45-degree facing test (the board opens from any facing like the notice board does).
