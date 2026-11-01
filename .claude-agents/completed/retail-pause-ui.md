# Retail pause/save presentation

2026-11-01, codex. Philip selected pause/save. Implementation and runtime verification complete; Philip comparison pending.

Fourth UI task. Existing Save/Settings/Quit functionality is implemented.

## Scope

Trace the original pause/save flow before choosing layout and assets. Recreate
applicable retail presentation while keeping port-specific Settings and Quit
accessible. Identify those port controls explicitly; title remains deferred.

Relevant files: app/src/pause_menu.rs, states.rs, save.rs, input.rs, msg_window.rs;
original pause/save source identified during tracing.

## Acceptance and boundaries

Philip accepts the applicable retail pause/save presentation and usable port
controls. Pause/resume and cancel preserve gameplay state. Review save behavior
only with disposable saves; preserve the user's save and settings.

Load original assets at runtime from Philip's own disc; never commit extracted
bytes. Keep the no-disc placeholder usable. Use source and existing translations,
not guessed layouts or gameplay rules. Claim before implementation. Run suitable
compile/lint/format/build checks for code changes; no tests unless Philip requests
them. Record implementation evidence separately from Philip's actual acceptance.
M4 remains open; title and rename remain deferred.

## Implementation

Source audit: save_menu.c is the developer FlashROM/Pak menu and m_pause.c is
a debug pause, not the retail save UI. ac_npc_restart_talk.c_inc starts30CD and
select entries1AE/1AF (Save & Quit / Save & Continue); successful continue and
quit messages are30D3/30E0. Read those banks at runtime and reuse the existing
con_kaiwa2/con_sentaku2 message/choice models, font and cursor. The PC overlay
places choices above the message so long save labels cannot hide the question.

P opens pause; P/Escape resume without saving. F1 opens explicitly PC-only
Settings / Quit without saving / Back. Quitting without saving has a safe
Keep playing default. Settings keeps other JSON keys, accepts P as text and
handles Enter/NumpadEnter, Escape and Backspace by physical key code. Its
write failures remain visible and cancel does not write settings.

Save requests now use the same numeric collector while paused, with an actual
write-result handshake. Continue/quit acknowledgements only follow success;
errors offer retry/back and cannot automatically exit. Simultaneous P+save or
retry is consumed so the world stays frozen. The existing synchronous writer
now serializes first, exclusively creates a unique temporary beside the save,
writes/syncs/closes it and rename-replaces the destination. Failed commits keep
the previous save and remove only a temporary created by that attempt.

Independent Astra source/code review found the simultaneous-P race and the
inherited truncate-on-failure risk; both are fixed and reviewed. No new title,
gyroid-to-house walk or memory-card emulation is claimed; this adapts the retail
save conversation to the PC pause entry. Its closing address uses the player's
name in place of an absent speaker's catchphrase. No tests added/run; Philip
visual acceptance stays open.

## Verification

Final cargo check/clippy/fmt and capture/default/release builds pass (release5m20s).
Evidence: Temp/bevy-pause-save-20261101, synthetic completed town with Bells543,
loan17400, house2 and a quest-conditioned item in slot7 (pocket_conds32768).

- Inspected native prompt, PC options, path editing, confirmation, saved and
  error screens. Choice geometry moved above the question after the first
  capture exposed overlap; keyboard controls use physical codes after preview
  input exposed logical-key ambiguity.
- Pause kept player and world-camera transforms fixed through movement input;
  P/Escape/cancel did not change save bytes. F5 while paused did not save.
- Settings P typed as text; cancel preserved the file; apply retained another
  key42. Failed settings writes remained visible and recoverable.
- Save & Continue saved the frozen state, resumed on acknowledgement and F9
  loaded it. Numeric house/Bells/loan/pockets/conditions/room fields were retained.
- Save & Quit stayed open after a failed write; retry succeeded, with simultaneous
  P+Enter remaining paused until acknowledgement. Enter then exited successfully.
- Destination-directory failure cleaned the temporary. A separate Win32
  read-sharing/no-delete lock on the original synthetic save forced rename
  failure: the existing target stayed byte-for-byte intact, with0 temporary files;
  unlocking and retrying succeeded and the saved quit acknowledgement exited.
- No-disc fallback Save & Continue/F9 retained Bells/loan/pockets/conditions,
  house/furniture/floors/identity. Its Settings and explicit Quit without saving
  worked; safe confirmation default and unchanged file bytes were verified.

All owned previews exited. Actual save baseline/final SHA256:
0495927D27B005E9FD0A4AED424EAAD883F56BBD8A9C4F5910167C1F9BBB1BD9.
Actual save/settings and unrelated .gitignore were not used for automation.
No assets exported, dependency/profile changes, save schema changes or tests.
