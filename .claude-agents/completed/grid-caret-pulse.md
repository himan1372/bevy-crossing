# Editor text caret pulse

Philip chose "grid cursor pulse" (2026-10-27). Source audit: `m_editor_ovl.c` has no pulse on the key
grid and no pad rumble in the editor; the pulse is the text caret, `mED_cursol_draw` (2549) with
`cursol_opacity_step` (0..34, +1 per 60 Hz frame, reset on cursor move/backspace/output, 1798-1865),
alpha `(17 - s) * 255 / 17` with `s = step > 17 ? 35 - step : step`, ink 195,80,80.

Kernel `editor_keyboard::caret_alpha`; app `shop_nook::editor::{Caret, CaretClock, pulse}` (PostUpdate),
used by the diary, letters and both password windows. Reset happens when the caret lands elsewhere.

## Result (2026-10-27, claude)
Implemented; gates: fmt, clippy -D warnings, check, default build pass. NOT driven at runtime (no
capture QA); the fade is unverified by eye. Correction to the handoff: the key grid has no pulse and
the editor has no rumble in the source, so "grid cursor pulse and pad rumble" reduces to this caret.
Diary/letter carets changed from ink colour to the retail red.

## Runtime check (2026-10-27, claude)
Capture build in Temp with a synthetic save and the disc, Say-code window at Nook's counter. Ten screenshots
~0.3 s apart: the caret's red-over-green strength cycled 54..116 (beating against the 35-frame cycle), so it
fades rather than staying solid; after typing "a" the caret sat right of the letter at near-full opacity.
No WARN/ERROR in the log. Not driven: diary and letter carets (same code path, different host), by ear/eye
timing against the retail cycle.
