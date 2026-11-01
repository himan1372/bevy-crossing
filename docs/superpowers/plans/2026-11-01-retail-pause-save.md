# Retail pause/save presentation

Philip selected pause/save after the M4 remaining-work summary. Existing scope:
.claude-agents/completed/retail-pause-ui.md; use current main checkout and shared
project workflow. No tests, no dependencies or save schema changes. Preserve
actual save/settings and unrelated .gitignore; automation uses synthetic Temp data.

## Design

Use the retail message/choice assets already decoded at runtime. Source
ac_npc_restart_talk.c_inc starts message30CD, whose two choices are select1AE
Save & Quit and1AF Save & Continue. The save_menu.c FlashROM/Pak screen is a
developer screen, and m_pause.c is a debugging pause. Do not copy either as retail.
The PC pause stays on P, freezes the existing world and leaves current location
intact. Native prompt and two choices occupy the existing talk/choice layout.
P/Escape resume. F1 opens explicitly named PC options: Settings, Quit without
saving, Back. Quit without saving defaults to cancel in its confirmation.

Keep the current numeric save collector/schema. A menu request is processed
while paused, with a result returned to the menu; only a successful result can
reach the saved-and-quit acknowledgement. Errors retain pause and offer retry
or back. Saving ignores close/quit input until its result. F5 remains available
only in its existing world/furniture contexts. No actual title or gyroid-walk
scene is added. Settings editing preserves other keys and reports write failure.
Native completed-save messages are read at runtime; Memory Card instructions
are replaced by truthful PC save/error wording. No-disc mode keeps a usable
text interface with the same actions and result semantics.

Source/code review found that the inherited fs::write could truncate the prior
save before reporting failure. Commit JSON via a unique, exclusively created
same-directory temporary, sync and close, then rename-replace; clean only a
temporary actually created by this attempt. A failed commit preserves the old
save. Also consume simultaneous Pause when starting/retrying a save.

## Work

- [x] Implement pause pages/input and save-result handshake.
- [x] Draw disc message/choice/font/cursor assets, retain accessible fallback.
- [x] Inspect source and integration for modal input leaks and save failure exits.
- [x] Sequential check/clippy/fmt; capture/default/release builds; no tests.
- [x] Synthetic Temp resume/cancel/settings/save/quit/error/retry/reload/fallback QA.
- [x] Update task/roadmaps/comms; complete claim and publish explicit files.

Preview findings corrected: physical Settings control keys, choice placement
above the message for readable long labels, and player name as the actorless
closing address. Native and no-disc flows passed, including nonzero condition
flags, locked-target old-byte retention, temporary cleanup and retry. No async
I/O or complete title/gyroid save scene is claimed. See task note for evidence.
Final release5m20s passed. Actual save unchanged; all owned previews exited.
