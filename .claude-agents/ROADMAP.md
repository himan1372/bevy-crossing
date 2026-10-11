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

## Vertical slice plan (revised 2026-10-10 with Philip's scope)

**Philip's scope:** real graphics from our own assets, a built rendering
pipeline, a playable area with animations (they don't need to be perfect),
and the main game loop of Animal Crossing (GameCube): **walk/run, talk,
get quests, and an inventory.**

**Slice = done when:** the game opens into a lit, textured town made of our
own Blender models (no placeholder capsules or boxes). An animated player
walks and runs (idle/walk/run animations), talks to animated villagers,
accepts a quest, carries items in a 15-slot pocket inventory, and completes
the quest for a reward. Steady 60 fps in a release build on Philip's PC.

**Build on the code Philip already translated.** `logic/` is the Rust
translation from **rustimal-crossing** (`rust/src/`, checked 2026-10-10: up
to date apart from formatting/lint cleanup; the only upstream modules not
copied are `vi`, `profiler` — C-runtime glue, not needed — and
`title_demo`, kept for the title screen). Gameplay tasks below **must** call
the named `logic::` module instead of writing new logic in `app/`. Most of
these modules are private today, so they need to be made `pub` first
(additive, no API break).
`logic::graphics` (the translated display-list renderer) is **not** the
rendering pipeline: it draws the original game's data, which we don't use.
Our pipeline is Bevy rendering fed by our own `.glb` assets.

Task slugs below become `backlog/<slug>.md` files when they're ready to
claim. **(P)** = needs Philip (decision or playtest). **[blocked: x]** = do x first.
Each milestone ends with a Philip playtest.

### M0 — Decisions and foundations
- [x] **slice-scope** — Philip set the scope above (2026-10-10).
- [ ] **ip-rename** (P, deferred by Philip 2026-10-10: keep names for now) — the shop uses Nintendo names (Tom Nook, Nook's Cranny, Nook 'n Go, Nookway, Nookington's in `app/src/shop.rs`); villagers Maple, Pippy, Bob, Chief are real Animal Crossing villagers. The project rules say no Nintendo IP, so Philip should pick original names; then a quick find/replace.
- [x] **app-states** — done (claude): `AppState` (Loading / Title / InGame) + `PauseState`; gameplay runs only in game and unpaused; HouseState/ShopLocation are sub-states.
- [x] **input-actions** — done (claude): one action map, a single input-focus rule, and consumable presses (`app/src/input.rs`).
- [x] **assets-scaffold** — done (claude): `app/assets/` (models, textures, audio, fonts), loading gate, placeholder fallback.
- [x] **ci-gate** — done (claude): `.github/workflows/ci.yml` runs the full gate on every push to `main`.
- [ ] **art-direction** (P) — Philip chose: look similar to Animal Crossing GameCube (soft pastel palette, chunky rounded low-poly, big-headed characters, painterly ground) built from scratch: no game textures or traced assets as reference. Next: one reference render for Philip to approve. [blocks all of M2]

### M1 — Main loop on the translated code (placeholder art is fine)
- [x] **logic-expose** — done (claude): make the slice modules `pub` in `logic/src/lib.rs`: `player_move`, `inventory`, `item_prefs`, `talk_request`, `talk_topics`, `request_selector`, `npc_ai`, `behavior`, `collision`, `game_time`. Add a small test per module proving the app-facing call works.
- [x] **walk-run** — done (claude, playtested; walk ~3.9 u/s, run 6.8 u/s): player locomotion through `logic::player_move` (walk/run/dash speeds and acceleration from the translated controller), with a Run action (hold Shift / gamepad B). Replaces the fixed speed in `player.rs`, `house.rs` and `shop.rs`. [blocked: logic-expose]
- [x] **pockets-inventory** — done (claude, playtested): 15-slot pockets backed by `logic::inventory` (+ `item_prefs` for item types); pockets UI; catches, fruit, furniture and quest items go into pockets; selling takes from pockets. Replaces the HUD counters. [blocked: logic-expose]
- [x] **talk-system** — done (claude, playtested): conversations driven by `logic::talk_request` / `talk_topics` / `dialogue_topics` (topic choice per personality and situation) instead of the placeholder lines in `dialogue.rs`. Dialogue text stays original (no game script). [blocked: logic-expose]
- [x] **quests-full** — done (claude; Philip playtest batched at end of M1): quest flow through `logic::quest` / `quest_gen` / `request_selector`: villager requests, delivery items carried in pockets, turn-in, rewards. Builds on today's `quest.rs`. [blocked: pockets-inventory, talk-system]
- [x] **villager-routines** — done (claude, playtested): villager schedules and wandering from `logic::npc_ai` / `behavior` / `movement` (field / in-house / sleep, greeting reactions) instead of the random walk in `npc.rs`. [blocked: logic-expose]
- [ ] **game-clock** — partly done (claude): `app/src/clock.rs` gives local time (chrono) + F6 debug hour; still to do: the `logic::game_time` calendar engine. — in-game clock and calendar from `logic::game_time`; villager schedules and ecology follow it; clock on the HUD. [blocked: logic-expose]

### M2 — Graphics: assets, rendering pipeline, animation
- [ ] **render-pipeline** — the Bevy look: camera angle and follow, sun + ambient light with shadows, a flat/toon material for our models, the curved-horizon ("rolling world") effect as our own shader, colour grading, MSAA. Tuned on placeholder geometry first. [blocked: art-direction]
- [ ] **terrain-mesh** — build the town ground as real meshes from the generated town (grass, sand, cliffs, river, ocean edge) instead of per-cell boxes; one shared material per terrain type. Ground height via `logic::collision`. [blocked: render-pipeline]
- [ ] **player-model** — rigged original player character with idle / walk / run / talk animations; animation speed follows walk/run (bevy-animation skill). [blocked: art-direction, walk-run]
- [ ] **villager-model** — one shared rig with per-villager colours and accessories; idle / walk / talk animations. [blocked: art-direction]
- [ ] **world-models** — player house, shop, mailbox, villager houses, trees, rocks, flowers. [blocked: art-direction]
- [ ] **render-perf** — instancing for trees and terrain pieces; check 60 fps in a release build. [blocked: terrain-mesh, world-models]

### M3 — UI and feel
- [ ] **ui-theme** — one UI style (open-licence font such as OFL, panels, colours) for dialogue, pockets, quest panel and HUD. [blocked: art-direction]
- [ ] **dialogue-polish** — letter-by-letter text, name plate, speaker sound blips.
- [ ] **pause-menu** — Save / Settings / Quit. [blocked: app-states]
- [ ] **title-screen** — this repo owns it (back burner): logo, New Game / Continue / Quit; flow can follow `rustimal-crossing`'s `title_demo.rs` (with our own logo, not Nintendo's).
- [ ] **audio-pass** — footsteps (walk vs run), UI blips, quest jingle, background music. Original or CC0 only.

### Later (after the slice)
Tree shaking, furniture placement, day/night lighting, daily rollover, mail
and shop polish, new-game flow — the systems exist in some form today but
aren't part of Philip's slice scope.

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
