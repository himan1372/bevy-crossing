# Convert the other mWR warnings to the retail warning panel

Philip (2026-10-27): "convert the other warnings". `completed/grab-bag-warning-panel.md` built the panel for the
grab-bag capacity refusal only. Inventory of stand-ins and conversion follows.

## Result (2026-10-27, claude)
Source: `m_warning_ovl.c` (`wr_win_data`, `mWR_move_Play`, `mWR_set_strings`, `mWR_set_frame_dl`) and each `mTG_open_warning_window` / `mED_open_warning_window` caller.
Kernel `logic/src/warning.rs`: all 24 non-null warnings generated from the C (script), `has_choice`. Panel
(`app/src/warning_panel.rs`): Yes/No (cursor model, colours white/pink, C-left/right, B answers No, A/Start the selection,
0x435 on Yes), `take_answer`, one panel at a time, runs in PreUpdate before `toggle_pockets`; `update_focus` holds the
player while a panel is up; the pockets toggle, pockets/mail/diary/letter input yield to it (Escape used to close the
screen underneath because `consume` does not hide `just_pressed`).
Wired (the app's stand-in -> panel): grab bag capacity (earlier), WRITE (stationery with no free letter slot),
PR_LEAVE (letter present with no free pocket), MAILBOX (house mailbox cannot empty into full letters; once per visit; Yes/No
only logged), WORD_OVER (diary and letter editor when a character does not fit), MUSIC (taking music out with no free pocket,
music grid), MUSIC2 (a song already registered), PUT_MAX_FURNITURE / PUT_FURNITURE (placing furniture), PUT_ITEM (a loose
item with no spot).
Verified (Temp, disc, synthetic saves, capture build): WRITE, PR_LEAVE and MAILBOX screenshots (wording, positions, stretch,
colours, left/right, Enter/Escape closing only the panel, pockets back, no re-open); no WARN/ERROR. Gates: fmt, clippy -D
warnings, default build; no release, no tests.
Not verified at runtime: MUSIC, MUSIC2, PUT_MAX_FURNITURE, PUT_FURNITURE, PUT_ITEM, WORD_OVER (wired by code path only);
the Yes/No cursor model (`att_win_cursor_model`) is built but did not show in the screenshots.
Not wired (no trigger in the app): MONEY and the mail-attach refusals PR_FOOD/PR_QUEST/PRESENT_MAIL/PR_CARD/PR_FORK (no
hand-drop onto letters or the wallet), PUT_PLANT, PUT_SIGN*, PUT_FAMI (no field drop of those), ORIGINAL (custom designs),
LOCK_DIARY (no diary lock), PW_CHK and the password windows' full-line cases (which failures map is unclear).
