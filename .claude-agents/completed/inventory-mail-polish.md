# Inventory/mail tags and paper motion

2026-11-01, codex. Philip selected inventory/mail polish after accepting the inventory collections and wallet work in915ede8c. Audit native special quest/mail tags and paper animations; preserve existing native art, numeric state, input ownership and no-disc fallback. No tests unless requested.

- [x] Compare completed tags/editor notes with current app and original C.
- [ ] Restore supported native multiline mail/delivery tags, geometry, pointers, typography and after-tag action placement.
- [ ] Restore source-backed letter paper/editor motion and correct stationery selection for writing/rewrite.
- [ ] Sequential check/clippy/fmt/capture/default/release and bounded synthetic Temp checks; protect real save/settings.
- [ ] Update roadmaps/comms, complete claim and commit/push explicit files. Keep M4 and Philip acceptance open.

One reused subagent audits/implements mail_disc.rs only; parent owns tag presentation and integration.

## Budget handoff 2026-11-02

Philip requested immediate wrap-up with under3% weekly usage. Partial implementation
is uncommitted; cargo fmt and cargo check --workspace -j1 passed. No final clippy,
builds or runtime QA, and no tests. Final reviewer hit usage limit; review remains
open. No owned previews or Cargo processes running. See the CURRENT top section of
docs/codex-handoff.md for exact files/source/resume concerns. Release claim without
completion. Do not report this slice as playable/accepted yet.

## Completion 2026-11-02, claude

Resumed the uncommitted partial work and finished it. Review fix: `tags/label.rs::delivery_label`
no longer requires `job.pockets_idx == slot` (the port's pocket moves never update that field and
`first_job_talk` finds the item by item + QUEST condition); it now matches item and quest condition.
Second fix found in QA: the Enter that chose Write also typed the first keyboard cell ("!") because
`mail_ui_input` read the same key event on the compose view's first frame; `mail.rs` now clears the
queue there.

Verified (synthetic Temp save, `Temp/bevy-inv-mail-20261102`, capture exe, real save untouched, SHA256
0495927D...BB1BD9 before/after): first-job delivery item with a stale `pockets_idx` shows the native
three-line "Delivery for / Kabuki / from Tom Nook" special tag with coloured names, pointer and Grab
action placement; ordinary items keep the normal tag; letter slot shows "Letter to QA / from Long
Neighbor QA"; Read opens the stationery with two pages; rapid Escape/Enter/Enter during the slide is
swallowed with no leaked input; Write opens with the actual stationery and keyboard slide, long text
shows the roll, Escape slides the keyboard and paper out and cleans up; cancel leaves pockets,
conditions, letters and Bells identical. check/clippy(-D warnings)/fmt/capture/default/release all pass.
No tests added or run.

Not driven: Rewrite cancel/commit, F4 commit, legacy multi-page long-letter paging beyond page 1,
second/third special arrow variants and right-edge flip, Museum recipient colours, Write refusal.
F4 still commits directly (native edit-end confirmation is a separate gap). M4 and Philip's play
acceptance remain open.
