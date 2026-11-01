# Intro dialogue reactions implementation plan

**Goal:** Finish native intro gestures/expressions at message reveal orders.
**Spec:** .claude-agents/completed/intro-dialogue-reactions.md.
**Architecture:** Small source-driven reaction state separate from scripted
NpcClip ownership. Character/face renderer selects it while talking; existing
movement and hand-over clips retain priority. Native data selects clip repeat,
morph, facial sequences and related effects; no asset exports or new framework.

**Constraints:** Existing Bevy0.19.1/edition2021; no tests/dependencies/schema;
runtime disc artwork; targeted materials; isolated Temp saves; main workflow.

- [x] Inventory actual intro commands from disc plus source behavior.
- [x] Implement reaction/order lifecycle and start-to-loop/reset transitions.
- [x] Extend face metadata consumption for native expressions and mouth poses.
- [x] Add the relevant native effect presentation/audio using existing renderer.
- [x] Verify movement/hand-over priority, dialogue branches, scene/load reset,
  missing-disc fallback and smoothness with synthetic scenarios.
- [x] Run check/clippy/fmt and capture/default/release sequentially; no tests.
- [x] Source review and records/roadmaps; complete ownership and publish explicit files.

Review focus: repeated identical commands preserve the native current frame; same-frame reveal
commands use native last value; reset preserves seated/phone context; no stale
face/effect after talk/load; reaction clips cannot consume hand-over orders.

Completed2026-11-01. Source review corrected identical-order frame preservation,
0.5/tick morph timing, continuous effect deduplication and NPC-only facial metadata.
Visual QA also required releasing Rover's final completed sit clip to seated idle.
Native Gfx-authored colors/LOD remain authoritative inside effect models.
Full train/Nook flow, refusal/repeat/cancel/transfer, F9 and missing-disc completed
save verified; all15 pooled effect meshes hidden after completion. Bounded
smoke/reaction/walking samples and remaining acceptance limits are in the task note.
Release7m36s; no tests. Actual save unchanged; all owned previews closed.
