# Warning panel Yes/No cursor arrow

Philip: "the cursor arrow" (2026-10-27). `att_win_cursor_model` (red 10 px triangle) is built but invisible. Find why.

## Result (2026-10-27, claude)
Cause: `att_win_cursor_model` sets `G_SETCOMBINE fcffffff fffdfeff` = colour PRIMITIVE, alpha ZERO, under an opaque render mode
(`e2 00001c 00552048`); the retail opaque mode ignores the combiner's alpha. `N64Material` alpha-blended it (alpha 0, invisible) and,
non-blended, masks at 0.5 unless `FLAG_OPAQUE` is set. A debug log confirmed the mesh was right (3 vertices, +-80 overlay units = a 10 px triangle).
Fix: `Skin::solid` / `model_with(blend: false)` in `shop_nook/editor.rs` builds the model non-blended with `FLAG_OPAQUE` set; the warning
panel uses it (prim red). Verified (Temp, disc, mailbox warning): the red arrow sits left of the selected answer, Yes and No, and moves with
Left/Right. No WARN/ERROR. Gates: fmt, clippy -D warnings, default build; no release, no tests.
