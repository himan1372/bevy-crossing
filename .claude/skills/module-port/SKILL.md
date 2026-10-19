---
name: module-port
description: "Port or reconcile Rustimal modules in logic/ using the pinned migration inventory, existing callers and original C semantics."
---

# Module port

Consult [project workflow](../references/project-workflow.md) once per task for claims, source authority, save isolation and validation boundaries.

## Choose the slice

Start with `docs/rustimal-migration-inventory.md` and the current task note. Its counts describe a snapshot, not remaining work. Inspect the pinned source with:

```powershell
git show 122867f762122eb266f04e319af0f0c31d54f476:rust/src/<module>.rs
```

Compare the target module, its public signatures, dependencies and callers before copying. Read relevant functions and invariants rather than loading unrelated modules. Use `rg` and Git diffs first; API-diff tools are optional when already available and useful. A source-only file may be test support or unsuitable for production.

## Reconcile behavior

For each difference, record the decision and source evidence in the task note: retain the Bevy implementation, import the source behavior, or combine them. Original C and actual callers resolve semantics; checkout age alone does not. Preserve native kernels and intentional Bevy additions. In furniture code, retain per-layer occupancy, owner-cell storage and bank/angle encoding.

Adapt only the authorized module/dependency slice. Keep required ABI adapters; remove obsolete wrappers only after checking callers. Follow `logic/src/lib.rs` exports (`pub mod` where public) and real package names: `rustimal-logic`, `bevy-crossing`, `disc`. Update callers together with intentional signature changes.

A compatibility shim needs an actual remaining caller and a retirement condition. Editing the sibling repository requires that scope to be authorized; do not create a speculative cross-repository dependency.

## Finish

Inspect the resulting API and call sites, then apply the shared validation rules. Do not import, add or run tests without Philip's request. Record source revision, decisions, integration coverage and unresolved behavior. Compilation alone does not establish runtime fidelity or finish the migration.
