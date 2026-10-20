# Retail HUD visuals

2026-10-16, codex. Planned only; no implementation or Philip acceptance.

Fifth UI task, following the shared menu primitives.

## Scope

Trace the retail clock/date, bells and context prompts, their visibility and
runtime assets. Replace development-style text where retail has an equivalent;
keep debug information explicitly separate and reuse existing game state.

Relevant files: app/src/clock.rs, ecology.rs, pockets.rs, interaction.rs, input.rs,
msg_window.rs; original HUD source identified during tracing.

## Acceptance and boundaries

Philip compares clock/date, bells and applicable context prompts with retail
in town and menus. Verify readability, correct values and expected visibility
without obstructing play or capturing gameplay input.

Load original assets at runtime from Philip's own disc; never commit extracted
bytes. Keep the no-disc placeholder usable. Use source and existing translations,
not guessed layouts or gameplay rules. Claim before implementation. Run suitable
compile/lint/format/build checks for code changes; no tests unless Philip requests
them. Record implementation evidence separately from Philip's actual acceptance.
M4 remains open; title and rename remain deferred.

## Result (claude, 2026-10-19)

Source trace (ACGC-PC-Port / ac-decomp): no field HUD exists. The symbol map
has no clock/bells display routine; time is shown by ac_house_clock (scene-
keyed interior wall clocks), furniture clocks and the calendar; bells only in
the pockets screen (m_inventory_ovl); BUTTON_Z drives the light switch,
editors and debug, not a time display; no context prompts. Philip chose:
hide the development text by default, F10 toggles it.

Implemented app::dev_overlay (DevOverlay marker on HudText, ToolText,
ClockText, HouseHudText, QuestHudText; DevOverlayShown resource, F10
Action::DevOverlay, Node::display so those texts' own Visibility logic is
untouched). Transient Saved/Loaded, catch and quest notices and the real UI
panels (pockets, mail, shop, pause, dialogue) are unchanged.
Verified (Temp preview, real save C15154C0... unchanged): town screenshot
clean, F10 shows bells/pockets/tool/clock lines, F10 hides; house hint lines
hidden. Gates: check, clippy -D warnings, fmt, capture, default 39s, release
2m52s; no tests. The shop wall clock moved to backlog/shop-wall-clock.md
(legacy N64 texture blocker).
