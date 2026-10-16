# Retail pockets UI

2026-10-16, codex. Planned only; no implementation or Philip acceptance.

First remaining retail UI task in M4; existing inventory behavior is implemented.

## Scope

Trace original inventory layout, runtime icon/texture assets, selection and
available action windows. Reuse disc font/window rendering and translated
inventory logic; retain existing inventory capacity, tool use and input ownership.

Relevant files: app/src/pockets.rs, items.rs, input.rs, msg_window.rs, game_disc.rs;
logic inventory and original inventory/window code identified during tracing.

## Acceptance and boundaries

Philip compares the 15-slot pocket layout, item icons and selection/action
windows with retail. Existing item actions and closing/back navigation remain
usable; opening/closing must not leak input to gameplay.

Load original assets at runtime from Philip's own disc; never commit extracted
bytes. Keep the no-disc placeholder usable. Use source and existing translations,
not guessed layouts or gameplay rules. Claim before implementation. Run suitable
compile/lint/format/build checks for code changes; no tests unless Philip requests
them. Record implementation evidence separately from Philip's actual acceptance.
M4 remains open; title and rename remain deferred.

