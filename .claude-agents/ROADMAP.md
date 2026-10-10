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
- [x] **workspace-fmt** — done (claude): `cargo fmt --all --check` clean. 38 hand-laid decomp tables and the two generated modules (fish/insect tables) are `#[rustfmt::skip]`. The full rust-workflow gate now passes.

## Vertical slice plan (drafted by claude, 2026-10-10)

**Slice = done when:** a new player launches the game → title screen →
New Game / Continue → wakes up in town → plays one full in-game day of the
core loop (catch fish + bugs, shake trees, sell at the shop, buy furniture,
place it at home, talk to villagers, finish one errand, get a letter) →
saves from the pause menu → quits. Original stand-in art for every on-screen
thing, basic audio, steady 60 fps in a release build on Philip's PC, and no
debug keys needed.

Task slugs below become `backlog/<slug>.md` files when they're ready to
claim. **(P)** = needs Philip (decision or playtest). **[blocked: x]** = do x first.
Each milestone ends with a Philip playtest.

### M0 — Decisions and foundations (do first)
- [ ] **slice-scope** (P) — Philip confirms the "done when" list above, town size, and one art direction (e.g. low-poly, flat colours).
- [ ] **ip-rename** (P) — the shop uses Nintendo names (Tom Nook, Nook's Cranny, Nook 'n Go, Nookway, Nookington's in `app/src/shop.rs`). The project rules say no Nintendo IP, so Philip should pick original names; then a quick find/replace. Check villager names too.
- [ ] **app-states** — top-level `AppState` (Title / InGame / Paused). Gameplay plugins run only `InGame`; HouseState/ShopLocation become sub-states.
- [ ] **input-actions** — one action map (Interact, Tool1-3, Menu, Save) instead of raw `KeyCode` checks, plus a single "who owns E right now" rule. That replaces the chain of "is mail/shop/quest UI open" checks that caused the door bugs, and makes a gamepad easy later.
- [ ] **assets-scaffold** — create `assets/` (models, textures, audio, fonts), a loading step, and a placeholder fallback so missing art never crashes the game.
- [ ] **ci-gate** — GitHub Action on `main` that runs check → clippy -D warnings → fmt --check → test, so a broken push is caught right away.

### M1 — Core loop complete (gameplay)
- [ ] **game-clock** — in-game clock and calendar (use `logic::game_time`); ecology spawns follow it instead of the PC clock; clock on the HUD. [blocked: app-states]
- [ ] **day-night-lighting** — sun angle and colour driven by the game clock; lights on at night. [blocked: game-clock]
- [ ] **tree-shake** — E at a tree shakes it: fruit drops (sometimes bells; bees optional via `logic::bee_ant`). Fruit regrows over days.
- [ ] **pockets-ui** — a real pocket inventory (fixed slots, item list) instead of the HUD counters; selling picks from pockets. [blocked: input-actions]
- [ ] **furniture-placement** — bought furniture shows up in the house: place, rotate, pick up (uses `FurnitureSize` footprints); saved and loaded. [blocked: pockets-ui]
- [ ] **new-game-flow** — New Game (fresh seed, name entry) vs Continue (load save); saving moves from F5 to the pause menu (F5 stays as a debug key). [blocked: app-states]
- [ ] **daily-loop** — the day rolls over (new shop stock, regrown fruit, new errands, villager letters arrive by mail). [blocked: game-clock]

### M2 — Art pass (Blender → .glb, see blender-gltf / asset-pipeline skills)
- [ ] **art-style-sheet** (P) — palette, proportions, poly budget; one reference image Philip approves. [blocked: slice-scope]
- [ ] **player-model** — rigged player, idle and walk animations (bevy-animation skill). [blocked: art-style-sheet, assets-scaffold]
- [ ] **villager-model** — one shared rig with per-villager colours/accessories; idle, walk and talk animations.
- [ ] **building-models** — player house, shop, mailbox, villager houses.
- [ ] **nature-models** — trees (with fruit variants), rocks, flowers; terrain materials (grass, sand, cliff, river water).
- [ ] **interior-models** — house and shop interiors, plus the starter furniture set.
- [ ] **render-perf** — ~1000 separate meshes today: share meshes/materials (instancing) for trees and terrain. Check the frame rate in a release build.

### M3 — UI, audio, feel
- [ ] **ui-theme** — one UI style (open-licence font such as OFL, panels, colours) for dialogue, mail, shop, pockets and HUD. [blocked: art-style-sheet]
- [ ] **dialogue-polish** — text appears letter by letter, villager name plate, speaker sound blips.
- [ ] **pause-menu** — Save / Settings (volume, fullscreen) / Quit. [blocked: app-states]
- [ ] **title-screen** — Philip confirmed this repo owns it (on the back burner for now): logo, New Game / Continue / Quit. [blocked: app-states, new-game-flow]
- [ ] **audio-pass** — footsteps, UI clicks, catch jingle, background music by hour. Original or CC0 only.
- [ ] **camera-feel** — tune the camera angle, follow smoothing, and indoor framing.

### M4 — Slice hardening
- [ ] **playtest-script** (P) — a step-by-step checklist that walks the full "done when" list; Philip runs it.
- [ ] **bug-bash** — fix everything the playtest script turns up.
- [ ] **windows-release** — release build plus a zip with assets, and plain-English install/run steps for Philip.

### Suggested order
M0 decisions in parallel with app-states / input-actions / ci-gate →
M1 (gameplay can use placeholder art) → M2 + M3 in parallel once the style
sheet is approved → M4. Art and gameplay tasks are independent enough for
different agents to work on them at the same time.

## Done (pre-board, with Philip)
- Bevy 0.14 → 0.19 upgrade (branch bevy-0.19-upgrade, merged to main).
- B0001 query-conflict fix (Without<> disambiguators in mail.rs / quest.rs).
- Phases 0–3: plane, town render, villagers + dialogue, save/load.
- Workbook bevy-upgrade-workbook.xlsx through finding #21.
