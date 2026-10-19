---
name: save-persistence
description: "Extend or diagnose the existing JSON save/load path, compatibility and transient cleanup while protecting Philip's actual save."
---

# Save persistence

Consult [project workflow](../references/project-workflow.md) once per task for claims, source authority, save isolation and validation boundaries.

## Extend the existing format

`app/src/save.rs` manually snapshots serde `SaveData` into working-directory `savegame.json`; it does not use a reflective save framework. `logic` also has retail save/GCI code, which is a different interface. Follow the actual snapshot/apply callers rather than replacing either system.

Current JSON version is 1. The loader rejects a mismatched version and starts fresh; do not bump it for every optional field. Use appropriate serde defaults/migration for compatible additions and design an explicit migration for incompatible changes. Legacy item-number/pocket conversion already exists: preserve it.

`home_furniture: Option<RoomSave>` distinguishes missing legacy room data from an intentionally empty room. Preserve numeric IDs, rotations, all room layers, owner-cell storage, switches, music selection, packed pocket conditions and unrelated wallet/pocket fields. Profiles and disc assets may be absent; that must not erase numeric contents. Rebuild runtime entities/handles from saved data rather than persisting ECS entity IDs.

## Keep persistence and presentation consistent

F5/F9 are existing manual save/load controls. Do not add implicit autosave triggers as part of a field change. Inspect the existing `LoadSet`, `ResetAudio` and scene/transient cleanup ordering. Mid-animation saves must represent the committed gameplay transaction, not half of its visual interpolation.

The current writer uses `fs::write`; atomic replacement or backups are not existing guarantees. If write durability is in scope, design it against Windows replacement/failure semantics and preserve the prior valid save on failure. Do not silently introduce a new persistence dependency for an ordinary data addition.

## Verify safely

Use only disposable working-directory saves. Compare complete relevant numeric state across snapshot/load, legacy missing fields, intentionally empty values and missing-disc/profile cases. For a refused operation, check equality of every affected collection and condition field. Exercise QuickLoad cleanup when the change touches live state. Add/run tests only if requested; keep synthetic files in Temp and record verification limits.
