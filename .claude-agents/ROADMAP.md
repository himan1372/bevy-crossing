# bevy-crossing roadmap (2026-10-10)

Claimable tasks live in `backlog/` (one file per task). Claim in `claims.jsonl`
before starting work; announce claims, completions, blockers,
and questions on `comms.txt` on the **main** branch (append-only).

## Open runtime bugs
- [x] **fix-shop-entry** — done (claude, 60e5688): door check measured to a point 0.7 out from the wall; playtested.
- [x] **fix-letter-notify** — done (claude, 969ca92): mail list child was Visibility::Visible over a hidden panel; playtested.

## Pipeline / docs
- [x] **document-glb-pipeline** — done (cosmo) — Blender -> .glb for Bevy 0.19: export settings, naming, scale, verification checklist → `.claude/skills/asset-pipeline/SKILL.md`.

## Code health
- [x] **logic-clippy-cleanup** — done (claude): `cargo clippy --workspace --all-targets -- -D warnings` passes (with and without `--features remote`); 475 tests pass.

## Candidates (needs Philip's decision)
- Workspace `cargo fmt` — `cargo fmt --check` fails in ~85 files (most of `logic/` was never rustfmt-ed). Running it is one big mechanical commit that also reflows hand-laid decomp tables (could use `#[rustfmt::skip]` on those). Do it, or drop fmt from the gate?
- Title screen port — `port-title-screen` sits in rustimal-crossing's backlog (reworded for the PC port); decide which repo should own the Bevy title screen.

## Done (pre-board, with Philip)
- Bevy 0.14 → 0.19 upgrade (branch bevy-0.19-upgrade, merged to main).
- B0001 query-conflict fix (Without<> disambiguators in mail.rs / quest.rs).
- Phases 0–3: plane, town render, villagers + dialogue, save/load.
- Workbook bevy-upgrade-workbook.xlsx through finding #21.
