# Retail grid leftovers

Claimed by claude 2026-10-27. Philip chose "grid leftovers" after shop-rare-sold-sign.

Scope (from the handoff): the X exchange (accent) key with its plate, the typed-character voice, the
R pressed face. Left alone unless cheap: the scrolling window texture, the cursor pulse, pad rumble.
Source: `m_editor_ovl.c` mED_ornament_table (19), mED_get_exchange_code/mED_exchange_code_func
(1081-1100), mED_set_se (754-800), mED_KeyDraw_X_button/R_button (1955-2138),
mED_StringsDraw_ornament (2497); `m_msg_sound.c_inc` mMsg_sound_voice_get_for_editor.
Hosts: diary and letters (multi-line); the password windows keep their plain codes.

## Implemented (2026-10-27, claude)

Kernel `logic/src/editor_keyboard.rs`: `ORNAMENT` (mED_ornament_table), `exchange_code`/`exchange`
(mED_get_exchange_code / mED_exchange_code_func), `EXCHANGE_VOICE` list, `editor_voice`,
`output_voice`, `exchange_voice` (mMsg_sound_voice_get_for_editor and mED_set_se). Tables generated
from the C source. Pure, disc-free.

App `shop_nook/editor.rs`: `Held` gains `r`, `x`, `exchange`; the R face is pressed with Space, the
X face with its pressed texture, and when the character before the cursor has a swap the
`kai_sousa_yajirushi_model` arrow and the two characters (before dim, after white at 259/281,190)
are drawn. `speak(code, exchange)` plays the editor voice (voice_begin spec 2, character
[voice, FF, FF], scale 32, default melody) for typed keys and swaps, and the 0x36 refusal when an
exchange stays quiet. Diary and letters: Ctrl+X is the X button; typed keys and the grid speak;
the password windows speak but have no exchange (their codes are fixed letters).

Verified (Temp, disc, synthetic saves): diary "cafe" + Ctrl+X became "cafè" (byte 0x7B), the plate
showed the pending swap with its arrow, hint line updated; a letter written the same way saved the
body "cafè". No WARN/ERROR. Not verified by ear: the voices. Not done: the scrolling window
texture, the cursor pulse, pad rumble/other pads; the password windows' exchange.
