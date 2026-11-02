# Inventory/mail Implementation Plan

**Goal:** Finish supported special tags and native letter/editor motion.
**Architecture:** Extend retained tags and mail disc presentation; reuse existing native motion kernels. Preserve all numeric gameplay and save behavior.
**Spec:** `docs/superpowers/specs/2026-11-01-inventory-mail-design.md`.

- [x] Audit completed notes/current code/native C; ordinary popups and editor grid already exist.
- [ ] Parent: add special tag label adapter, native geometry/pointers/fonts/name colors and after-parent placement in pockets_disc tags modules. Preserve ordinary/collection labels and existing actions.
- [ ] Reused subagent: correct stationery selection and implement retained native paper/keyboard motion in mail_disc.rs. Parent: integrate transition input/focus and reload cleanup.
- [ ] Sequential check/clippy/fmt/capture; synthetic Temp visual/numeric QA for supported tags and read/write/rewrite/cancel/paging/reload/no-disc. No tests requested.
- [ ] Review, default/release builds, task/comms/roadmap updates, ownership completion and explicit-file publication.
