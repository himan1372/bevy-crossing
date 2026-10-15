# ui-mailbox-sounds — M4 interface and mailbox audio

Requested by Philip 2026-10-14; keep work focused and token efficient.

- [x] Existing disc queue: pockets/pause open/close; pause settings save/cancel; actual selection changes only; dialogue choice open/confirm/cancel; shop tabs/navigation and successful register sounds, with cancel feedback on rejected purchases.
- [x] Mailbox panel opens with inventory mailbox ZOOMUP (0x8004), and closes with inventory NA_SE_31 (0x31); actor lid effects 0x146/147 are not attached to the standalone menu. Flag up/down sounds (0x433/434) cross frame 15 once, attenuated within 16 world units. Letter read/back/compose/send feedback implemented.
- [x] Game build passed. Check, workspace clippy --all-targets -- -D warnings, formatting and workspace tests passed (609 passed, 20 ignored). Extended the existing ignored disc probe rather than adding a separate fixture: all 29 outdoor/floor/UI/mailbox effect IDs render non-silent finite audio. WAVs stay in Windows temp/bevy-crossing-effects.
- [ ] Philip: recheck corrected exit/mailbox/register sounds, then restart the game; open/close pockets and pause, move the pause cursor and select Settings, navigate/answer a dialogue choice, buy/sell or attempt an unaffordable item, open the mailbox, move through letters, read/back, compose/cancel/send and close. Check for one sound per action and no cursor clicks at list boundaries. Reading the last unread letter lowers the flag; its sound should occur once nearby.

Source: audio_defs.h, m_choice_sound.c_inc, m_inventory_ovl.c, mTG_move_cancel (m_tag_ovl.c), and ac_mailbox_move.c_inc. Inventory closing uses NA_SE_31. Player mailbox interaction opens mSM_OVL_INVENTORY with mSM_IV_OPEN_MAILBOX, so it closes with NA_SE_31 too; ZOOMDOWN_SHORT belongs to the separate mailbox overlay. Lid audio belongs with the unimplemented actor lid animation. Existing mixer lacks reverb. Listening acceptance remains pending.

Playtest correction (2026-10-14): Philip reports exits sounded like letter-recipient selection, register sounded incomplete, and the standalone mailbox-menu effect was unfamiliar. Cursor navigation and pocket opening are confirmed good. Fixes: MENU_EXIT now inventory's NA_SE_31 (0x31); mailbox transitions use 0x8004/0x8005; SE_REGISTER retains MONO(0x50) and routes to dedicated subtrack 14, avoiding ordinary-slot replacement. Voice clicks also preserve their existing MONO flag.

Improved validation: mixer update ADDS into its output; reused probe buffers now clear before each update, matching live NasSource. Earlier repeated-buffer RMS figures were inflated and established only activity. Fresh-buffer probe checks all 29 IDs and a separate register tail check uses a silent baseline, one-shot completion, and ordinary-slot traffic. Reproduced ordinary register cut from 1.163 s to 0.437 s; MONO remains 1.163 s under traffic and returns to silence. Three real-disc probes and clippy passed; workspace tests: 609 passed, 21 ignored. Playback corrections still need Philip's listening approval.

2026-10-14 follow-up: Philip accepts pockets; mailbox still sounds a little off.
Traced ac_mailbox's player interaction to mSM_OVL_INVENTORY/mSM_IV_OPEN_MAILBOX.
Opening ZOOMUP was correct, but closing ZOOMDOWN_SHORT belonged to a different
overlay. mail.rs now closes with NA_SE_31. Physical lid frame-18/frame-4 effects
remain deferred until the actor opening/closing animation is implemented.
Listening recheck remains pending; proceeded to step 2 as requested.

Philip listening update (2026-10-14): letters sound okay; asked to move on for
now. Pockets/letters accepted. This does not imply physical lid animation or
general trigger overlap stress validation is finished.
