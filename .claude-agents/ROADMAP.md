# bevy-crossing roadmap (2026-10-10)

Claimable tasks live in `backlog/` (one file per task). `claims.jsonl` is currently
empty — claim there before starting work; announce claims, completions, blockers,
and questions on `comms.txt` on the **main** branch (append-only).

## Open runtime bugs
- [ ] **fix-shop-entry** (backlog) — E silently rejected at the shop door; needs BRP diagnosis (port 15702, `remote` feature).
- [ ] **fix-letter-notify** (backlog) — letter notification never fades; timer code looks right, needs live BRP inspection.

## Pipeline / docs
- [ ] **document-glb-pipeline** (backlog) — Blender -> .glb for Bevy 0.19: export settings, naming, scale, verification checklist → `.claude/skills/asset-pipeline/SKILL.md`.

## Candidates (needs Philip's decision)
- Title screen port — `port-title-screen` sits in rustimal-crossing's backlog (reworded for the PC port); decide which repo should own the Bevy title screen.

## Done (pre-board, with Philip)
- Bevy 0.14 → 0.19 upgrade (branch bevy-0.19-upgrade, merged to main).
- B0001 query-conflict fix (Without<> disambiguators in mail.rs / quest.rs).
- Phases 0–3: plane, town render, villagers + dialogue, save/load.
- Workbook bevy-upgrade-workbook.xlsx through finding #21.
