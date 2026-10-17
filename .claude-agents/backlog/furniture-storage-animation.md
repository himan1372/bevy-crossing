# Furniture storage opening and closing

2026-10-17, codex. Proposal following Philip's acceptance of bounded furniture
placement/pickup presentation. Planning only; implementation not started.

## Bounded scope

Connect the original wardrobe/closet open and close sequence to the existing E
storage interaction, using runtime disc furniture rigs/clips and the applicable
original player clips. Keep Put in/Take out transactions, item eligibility,
capacity checks and cancellation atomic. Close on completion/cancel according
to source lifecycle; do not invent persistent open state in numeric room saves.

Audit supported constructors first and explicitly identify supported furniture
families. Drawers have separate obstruction rules and must not silently share a
wardrobe approximation. Music disk players remain a separate interaction.

## Source and implementation audit

ac_my_room_action.c_inc aMR_SetOpenFtrDemoData sets demo_status, WAIT_MSG and
demo_ftrID. aMR_JudgeDemoStart gates wardrobes/closets on front contact; drawers
have a separate aMR_CheckHikidashi obstruction path. These callers were inspected
for this proposal. The following lifecycle and constructor audit is still pending:

- Trace demo_status/message completion through ac_my_room_move.c_inc and each
  selected furniture constructor/move/draw callback; establish exact clips,
  timing, sounds and closure conditions.
- Read player open_furniture/wait_open_furniture/close_furniture states and
  relevant existing logic translations before choosing the adapter.
- Audit app/src/furniture.rs profile_skeleton and current fixed-pose joint mesh
  construction before introducing animated furniture roots. Reuse existing
  runtime cKF decoding; keep static furniture, stacks and movement working.
- Integrate exclusive focus with player_room storage/dialogue/pockets ownership;
  clear transient state on F9, scene changes and missing-disc startup.

## Evidence and acceptance

- [ ] Claim this task before implementation; audit source/callers/translations.
- [ ] Implement the bounded supported families and document limitations.
- [ ] Workspace check, warnings-denied lint, format and appropriate builds,
  one Cargo process at a time. No tests added/run without Philip's request.
- [ ] Disposable Temp previews with synthetic saves and explicit --disc: open,
  store/retrieve, cancel, full pockets/storage, repeated interactions, pause,
  F5/F9/restart, stacked furniture and no-disc numeric preservation.
- [ ] Philip's actual playtest acceptance, separately from automated evidence.

Preserve actual save/settings/.gitignore. No extracted or derived disc assets in
Git. Exact dust, long-piece retail pivots, loose diary, broader per-type behavior,
M4 and module migration remain separate/open.
