# Train arrival effects implementation plan

**Goal:** Finish the selected engineer, smoke/steam and train-sound slice.

**Architecture:** Keep train_control as the movement authority. A native-tick
disc-independent presentation kernel supplies smoke/steam and rhythmic sound
events; the app renders original display lists and follows the cab with NpcModel.
The existing ambience mixer owns the train's stable steam-loop source.

**Scope/design:** .claude-agents/completed/intro-train-effects.md. Philip selected
the source-fidelity slice; routine implementation choices follow existing code.

## Constraints

Bevy 0.19.1, existing checkout/main workflow, no new dependency/save schema,
runtime disc art/audio, preserve actual save and unrelated .gitignore, no tests.

## Tasks

- [x] Trace original source, record offsets/cadence/audio IDs and asset encoding.
- [x] Add native-tick presentation kernel to logic/src/train_control.rs and
  integrate cab actor/audio events with app/src/intro_arrival.rs.
- [x] Add focused app/src/intro_arrival/effects.rs for cached disc particle
  meshes/textures, private per-particle alpha/LOD, billboard motion and cleanup.
- [x] Add the stable train level source to app/src/ambience.rs.
- [x] Verify check/clippy/fmt sequentially; build capture, preserve isolated EXE,
  then default/release. No tests.
- [x] Drive arrival, stopped wait, Porter dialogue, departure, reload and no-disc
  fallback in Temp. Inspect captures; preserve save hash. Scene-exit condition
  reviewed; direct house setup intercepted by Nook before entering.
- [x] Review source/diff, update both roadmaps/comms/task note, complete ownership,
  commit only task files and push. Report acceptance limits.

## Outcome

See the completed intro-train-effects task for exact build/runtime evidence and
limits. Two source-review corrections: saved wheel frame trigger and both scene
states for cleanup. Runtime corrected Town self-transition cleanup and optional
sub-state resources during startup. No tests, new dependencies or save schema.

## Review focus

Render rates must not alter puff count; stopped trains must not emit new puffs;
steam blends use both native textures; effects keep world positions after birth;
engineer uses capsule-centre compensation; reload/scene exit retires the train
and its sounds; missing assets must not panic or change inventory.
