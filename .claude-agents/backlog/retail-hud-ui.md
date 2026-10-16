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

