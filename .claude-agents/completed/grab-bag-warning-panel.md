# Grab-bag warning panel

Philip chose "warning panel" (2026-10-27). Source: `m_warning_ovl.c` (mWR): `att_win_model` frame scaled by
(scale*sx, scale*sy) about the screen centre, `scale` +0.2 per 60 Hz frame in, A/B/Start starts -0.2 out;
lines `wr_hukubukuro_open_line` (48,37 / 50,24 / 52,24, stretch 1.0 x 1.15) in yellow 255,255,0 via
`mFont_SetLineStrings` at `x = 160 - sx*120*scale + pos*scale`, `y = 120 - sy*60*scale + sum*scale`.
Replaces the dialogue-window stand-in in `pockets.rs`.
Kernel `logic/src/warning.rs`; app `app/src/warning_panel.rs`.

## Result (2026-10-27, claude)
Implemented: `logic/src/warning.rs` (HUKUBUKURO_OPEN lines/stretch, `Warning::line_origin`, `Panel` grow/shrink at
0.2 per 60 Hz frame, dismiss) and `app/src/warning_panel.rs` (disc `att_win_mode`+`att_win_model` frame and yellow
text, grows about the screen centre; A/B/Start/Enter close it). `pockets.rs` opens it instead of the dialogue
stand-in; the pockets stay blocked while it is up and answer again after it shrinks; reset/exit clear it.
Gates: fmt, clippy -D warnings, check, capture and default builds. No release build, no tests.
Temp QA (disc, synthetic save, 14 apples + a grab bag): Open on the bag shows the panel, ArrowRight does not move
the pockets cursor, Enter closes it and the pockets return intact; no WARN/ERROR. Not done: panel frames mid-grow
were not inspected (screenshots landed after the 5-frame grow), the blue frame tint is the disc model's but
unchecked against retail footage, Escape/B close not driven, and the other mWR warnings (mailbox Yes/No,
sell-quest, put-item, ...) still use their stand-ins; only the grab-bag entry exists in the kernel.
