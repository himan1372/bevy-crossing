# Island cottage and naming plan

Philip selected Next30 tasks5 and6. Restore the original cottage room (aMI_ROOM_KIND_L, usable8x8 within10x10 model), native default floor/wall13 and four aloha mannequins from mHm_SetDefaultCottageData. Reuse the existing editable PlayerRoom renderer at its isolated origin, swapping independent numeric cottage/home records at entry. Keep mainland floors and house-owned collections separate. Add optional compatible cottage save; no new version, dependencies or extracted art.

Kapp'n first-visit flow uses0x3079, native island name entry, then0x30BE with the committed name. Store optional accepted island_name on identity; legacyNone prompts only after crossing consent. Reuse name-entry native window and8-byte restrictions; generic intro callers remain intact. Do not implement other island layout/life work.

- [x] Parent: explicit cottage context, independent room snapshot/restore, native room dimensions/defaults, correct shell/stairs and existing room actions.
- [x] Subagent: source-backed naming flow in kappn/name_entry; parent adds identity field.
- [x] Integrate F9/scene/audio cleanup and save ownership; source review then sequential check/clippy/fmt/capture.
- [x] Temp QA cottage edit/exit/revisit/save/F9/restart/mainland isolation, naming cancel/accept/repeat/reload, no-disc numeric retention; inspect screenshots. No tests without request.
- [x] Default/release builds, docs/claims/publication. Keep acceptance separate from bounded automation.

Completed with source review and bounded synthetic runtime checks; see completed task note for evidence and limits. Philip acceptance remains open.
