# Retail character-grid entry (shared editor overlay)

Unclaimed. Philip (2026-10-26): pull graphics from the disc rather than redraw
them; any rewritten logic must be a pure kernel in `logic/` that does not depend on
the disc so it can move to a new project; reuse rustimal-crossing kernels.

Findings: pinned/sibling `rust/src/name_editor.rs` has only buffer helpers and
constants (`buffer::clear`, `is_all_spaces`, entry types, 10x4 keyboard size);
its note says keyboard tables, insert/delete and the state machine stayed in C.
Password entry lives in `m_editor_ovl.c` (2635 lines): `mED_get_code`
(grid code lookup, line 63), cursor/stick area and movement (140-420), shift/
arrange/input-mode and convert (422-520), `mED_input_pw_chk_line`/`pw_make_line`
(already in `logic/password.rs`), pw table moves (1572-1760) and the per-key
button draws (`kai_sousa_*` models/textures, 1905-2300, loaded from the disc).
Letters and the diary use the same overlay.

Plan: 1) new `logic/src/editor_keyboard.rs`: grid tables, cursor/shift/mode
state machine and key outcomes as pure functions (data copied from C source,
no disc). 2) app overlay that draws the grid/buttons with disc models
(`disc_gfx`) and feeds the kernel from keyboard/gamepad, replacing the typed
adapter for passwords first, then letters/diary. 3) Gates, Temp QA, docs.
