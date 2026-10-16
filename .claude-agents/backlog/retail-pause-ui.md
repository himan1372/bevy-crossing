# Retail pause/save presentation

2026-10-16, codex. Planned only; no implementation or Philip acceptance.

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

