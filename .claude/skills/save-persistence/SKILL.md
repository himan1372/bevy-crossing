---
name: save-persistence
description: Town and game-state persistence for the life-sim — choosing a save framework, Reflect registration, save/load triggers, round-trip tests, and save-format versioning. Use when implementing or changing anything about saving/loading the town, player data, or settings; getting this wrong late is expensive.
---

# save-persistence

An Animal Crossing-style game *is* a persistent town simulation. Save/load correctness is core gameplay, not plumbing. This skill covers the full loop: what to save, how, when, and how to prove it round-trips.

## Step 1 — Choose the persistence approach

Different data wants different tools. Don't use one hammer for all three:

| Data | Tool | Why |
|---|---|---|
| **Settings / preferences** (volume, keybinds, graphics options) | `bevy_pkv` — persistent key-value store | Tiny, schema-less, no entity graph involved |
| **Full town snapshot** (entities, villagers, items, placements, time) | `moonshine-save` (Save/Load framework) **or** `bevy_save` (snapshot framework with entity-id mapping) | Both handle the hard part: serializing the entity graph and remapping entity IDs on load |
| **Human-readable / schema-driven saves** (debugging, modding-friendly formats) | `bevy_serde_lens` — schema-based serialization | When you want saves you can read and hand-edit |

> **Verify Bevy 0.19 compatibility** for whichever crate you pick (check [bevy.org/assets](https://bevy.org/assets/) Persistence section — ecosystem crates track Bevy versions closely and the compatible release moves with each Bevy release). Don't pin a version from memory; check at implementation time.

Decision guidance:
- Start with `bevy_pkv` for settings — it's the smallest possible step and de-risks the pattern.
- For the town, prefer the framework whose entity-ID remapping story you understand best after reading its docs; both `moonshine-save` and `bevy_save` solve it, but their APIs differ. Prototype on a tiny test world (3 entities, 2 components) before committing the whole town to one.
- The existing `savegame.json` quick-save/load (F5/F9) is the incumbent — any new framework must either replace it cleanly or interoperate; don't run two save systems side by side.

## Step 2 — Reflect registration (the step everyone forgets)

Bevy's serialization works through `Reflect`. Every component and resource that participates in a save **must** be `Reflect`-registered, or it silently won't serialize — the save "works" and the data is just missing on load.

Checklist per saveable type:

- [ ] Type derives `Reflect` (and `Component`/`Resource` as appropriate).
- [ ] Type is registered in the app's `TypeRegistry` (via `app.register_type::<T>()` or the persistence framework's registration helper).
- [ ] Custom types inside components (enums, nested structs) are also `Reflect` — registration is transitive in requirement, not automatic.
- [ ] A test exists that serializes one entity with this component and asserts the data survives (feeds Step 4).

When a load "loses" data, check this step before anything else. Unregistered types are the #1 cause and they fail silently.

## Step 3 — Save/load trigger patterns

Decide *when* saves happen; don't sprinkle save calls through gameplay code:

- **Explicit player action** — menu "Save", F5 quick-save: event → save system. Keep the existing F5/F9 bindings working or deliberately replace them (Philip's call — he removed auto-save by preference; don't reintroduce it unasked).
- **State transitions** — entering/exiting buildings, day rollover, quitting to title: these are natural save points. Implement as: transition event → save system runs → transition completes. Never save *during* a transition's mid-frame state.
- **Never** save from inside gameplay logic systems directly. Gameplay emits events / changes state; a dedicated save system reacts. Same separation as `gc-audio-pipeline`'s event→sound pattern.

Save format rules:
- One canonical save file (+ versioned backups, e.g. keep the previous save as `.bak` — a corrupted write must never destroy the only copy).
- Write atomically: write to temp file, then rename. A crash mid-write must leave the previous save intact.
- Include a **format version number** in the file (Step 5).

## Step 4 — Round-trip test recipe

Every saveable unit gets a round-trip test. This is non-negotiable for a life-sim — it's the test that catches the silent data loss from Step 2:

```
1. Build a known world state (spawn entities with known components/values).
2. Serialize (save).
3. Clear the world (despawn everything saveable).
4. Deserialize (load).
5. Assert: entity count equal, component values equal, relationships (parent/child, references) intact.
```

Wire these into the per-module test conventions (`rust-workflow`, `module-port`'s checklist): the `logic/` side tests the data model round-trips; the `app/` side tests the framework integration. A round-trip test that only covers 3 entities is infinitely better than no test — start small, extend as new saveable types land.

Reference pattern: [chadnauseam — automated testing in Bevy](https://chadnauseam.com/coding/gamedev/automated-testing-in-bevy) (setup/run/check test worlds).

## Step 5 — Save-format versioning

The town data model *will* change (that's the whole migration). Plan for it now:

1. Every save file carries a `format_version: u32`, bumped whenever a saveable type changes shape.
2. On load: if `file_version < CURRENT`, run the migration chain (`v1→v2→v3…`, one small function per step, each tested). If `file_version > CURRENT` (save from a newer build), refuse to load with a clear message — never silently misread.
3. Keep one test save file per old version in the repo (under test fixtures) so the migration chain is exercised in CI, not discovered by players.

## Checklist (per persistence task)

- [ ] Approach chosen per data class (settings vs town vs readable); 0.19 compatibility verified.
- [ ] Every saveable type `Reflect`-registered, including nested types.
- [ ] Save triggers defined (explicit action / state transition); no saves from gameplay systems; no auto-save unless Philip asks.
- [ ] Atomic writes + `.bak` of previous save.
- [ ] Round-trip test written and passing for every new saveable type.
- [ ] Format version bumped if any saveable shape changed; migration function + fixture test added.

## Sources

- [bevy.org/assets — Persistence section](https://bevy.org/assets/)
- [bevy_serde_lens quick-start tutorial](https://deepwiki.com/mintlu8/bevy_serde_lens/2.2-quick-start-tutorial)
- [chadnauseam — automated testing in Bevy](https://chadnauseam.com/coding/gamedev/automated-testing-in-bevy)
