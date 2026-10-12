# bevy-crossing roadmap (revised 2026-10-12)

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

## Direction (Philip, 2026-10-12): OpenMW model

bevy-crossing rebuilds the **engine** in Bevy. All game **content**
(models, textures, sounds, maps) loads at runtime from the player's own
Animal Crossing GameCube disc image. This replaces the earlier
"make our own art" plan: art-direction v2 is retired, and
`tools/concept-art/` and `docs/art-direction.md` are leftovers.

**Hard rule:** no data copied from the disc goes in any repo, ever. That
means no extracted models, textures, audio, `.bin` dumps, or test files cut
from the disc. Tests use made-up data built inside the test. Tests that need
a real disc are `#[ignore]` and read the disc path from `BEVY_CROSSING_ISO`;
CI never has a disc. The game must still start without a disc (placeholder
art plus a clear message).

## Vertical slice plan (revised 2026-10-12)

**Philip's scope:** a playable area with animations, drawn from the player's
own disc, and the main game loop of Animal Crossing (GameCube):
**walk/run, talk, get quests, and an inventory.**

**Slice = done when:** pointed at the player's disc image, the game opens
into a town drawn with the original models and textures (no placeholder
capsules or boxes). An animated player walks and runs, talks to animated
villagers, accepts a quest, carries items in a 15-slot pocket inventory,
and completes the quest for a reward. Steady 60 fps in a release build on
Philip's PC. Without a disc, the same loop runs on placeholder art.

**Build on the code Philip already translated.** `logic/` is the Rust
translation from **rustimal-crossing** (`rust/src/`). Gameplay tasks
**must** call the named `logic::` module instead of writing new logic in
`app/`. After the direction change, `logic::graphics` is now the core of
rendering. It holds the translated emu64/GBI display-list interpreter, the
texture conversions and the `GxBackend` trait. The disc's models are
N64-style `Gfx` display lists, and Bevy implements the GX backend.

**Port existing code from rustimal-crossing instead of rewriting it:**
- `rust/src/lib.rs` (`Disc`, `FstFile`, `decode_yaz0`): reads ISO/GCM/CISO,
  checks the GCM header, parses the disc's file table (FST), finds the DOL
  size, and decodes Yaz0. `src/pc_disc.c` is the C version, for reference.
- `.claude/skills/asset-pipeline/SKILL.md` + `tools/gen_runtime_assets.py` +
  `src/pc_assets.c`: the asset table (~2,500 assets, each with ROM offset,
  size, source DOL/REL, and byte-swap type). The assets are packed inside
  `main.dol` and `foresta.rel.szs`, not stored as separate files on the disc.
- `logic/src/dvd.rs`, `aram.rs` and `gbi_runtime.rs` are C-ABI shims for the
  C port; bevy-crossing should use plain Rust APIs instead.

Task slugs below become `backlog/<slug>.md` files when they're ready to
claim. **(P)** = needs Philip (decision or playtest). **[blocked: x]** = do x first.
Each milestone ends with a Philip playtest.

### M0 — Decisions and foundations
- [x] **slice-scope** — Philip set the scope (2026-10-10; graphics source changed 2026-10-12).
- [ ] **ip-rename** (P, deferred by Philip 2026-10-10: keep names for now) — Nintendo names in `app/src/shop.rs` and the villager list. Revisit before any release.
- [x] **app-states** — done (claude): `AppState` (Loading / Title / InGame) + `PauseState`; HouseState/ShopLocation are sub-states.
- [x] **input-actions** — done (claude): one action map, a single input-focus rule, consumable presses (`app/src/input.rs`).
- [x] **assets-scaffold** — done (claude): `app/assets/`, loading gate, placeholder fallback. (Now holds only our own UI and placeholder files; disc content never goes here.)
- [x] **ci-gate** — done (claude): `.github/workflows/ci.yml` runs the full gate on every push to `main`.
- [x] ~~**art-direction**~~ — replaced 2026-10-12 by the disc-asset direction; v2 sign-off no longer needed.
- [ ] **disc-setup** (P) — Philip decides three things. Which disc formats to accept: ISO/GCM/CISO as in rustimal-crossing, with RVZ/GCZ maybe later. Which regions: NTSC-U `GAFE01` only to start? How the player points the game at the disc: a settings file, a `--disc` flag, a file picker, or a `rom/` folder next to the exe. Proposed default: `GAFE01`, ISO/GCM/CISO, look in `rom/` first, then use a path from the settings file.

### M1 — Main loop on the translated code (placeholder art) — DONE 2026-10-10 (Philip playtested; U-turn drift fixed in 6bb80f7)
- [x] **logic-expose** — done (claude): slice modules `pub` in `logic/src/lib.rs`, one test each.
- [x] **walk-run** — done (claude, playtested; walk ~3.9 u/s, run 6.8 u/s): `logic::player_move`, Shift = run.
- [x] **pockets-inventory** — done (claude, playtested): 15-slot pockets on `logic::inventory`.
- [x] **talk-system** — done (claude, playtested): `logic::talk_request` / `talk_topics` / `dialogue_topics`.
- [x] **quests-full** — done (claude, playtested): `logic::quest` / `quest_gen` / `request_selector`.
- [x] **villager-routines** — done (claude, playtested): `logic::npc_ai` / `behavior` / `movement`.
- [x] **game-clock** — done (claude): `app/src/clock.rs` on `logic::game_time`.

### M2 — Reading the disc (pure Rust, no Bevy; testable without a disc)
- [ ] **disc-reader** — open ISO/GCM/CISO, check the GameCube magic number and game ID, parse the FST, and find and read files by path. Port of rustimal-crossing's `Disc`/`FstFile`. Tests build a tiny made-up GCM in memory. [blocked: disc-setup]
- [ ] **yaz0** — Yaz0 (`.szs`) decoder, port of `decode_yaz0`. Tests round-trip through a small encoder written for the test.
- [ ] **dol-rel-image** — extract `main.dol` and decompress `foresta.rel.szs`; map the DOL and REL sections so an asset's ROM offset resolves to bytes. [blocked: disc-reader, yaz0]
- [ ] **asset-table** — port the asset table from rustimal-crossing (`pc_assets.c` / `gen_runtime_assets.py`) into Rust: name → ROM offset, size, source (DOL/REL), swap type. It holds metadata only, no disc bytes. Plus byte-swap helpers (u16 / u32 / `Vtx`). [blocked: dol-rel-image]
- [ ] **disc-loader-app** — Bevy side: find and open the disc at startup as decided in disc-setup. If it's missing or wrong, show a clear message and fall back to placeholder art. Expose the disc to the renderer as an `AssetSource` or resource. [blocked: asset-table]

### M3 — Rendering the original assets
- [ ] **gx-backend-bevy** — implement `logic::graphics::gx_backend` for Bevy: run display lists through `logic::graphics::interpreter`, turn the decoded vertices into Bevy `Mesh`es and the converted textures (`logic::graphics::texture`) into `Image`s, and cache them per asset. [blocked: disc-loader-app]
- [ ] **model-viewer** (P) — debug scene (F-key or `--view <asset>`) that loads one asset by name (a tree, then a building) and orbits the camera around it. This is the first proof for Philip that disc rendering works. [blocked: gx-backend-bevy]
- [ ] **tev-materials** — turn the GameCube's colour-combiner settings (TEV, `logic::graphics::combine`) into Bevy materials. Start with texture × vertex colour, then add the modes the town actually uses, then alpha test/blend and fog. [blocked: gx-backend-bevy]
- [ ] **town-from-disc** — draw the town ground, cliffs, river and beach from the original acre/field models for the generated town (`logic::field_gen` / `town_gen`), replacing the per-cell boxes. Collision stays on `logic::collision`. [blocked: tev-materials]
- [ ] **world-objects-from-disc** — trees, rocks, flowers, houses, shop and mailbox from the disc. [blocked: tev-materials]
- [ ] **skeletal-animation** — research first: how the original stores character skeletons and animations (cKF-style keyframes in the decomp; `logic/` has no translation yet). Then decode them and drive Bevy transforms. May need more decomp code translated first. [blocked: gx-backend-bevy]
- [ ] **player-from-disc** — player model with idle / walk / run animations matched to the speed from `logic::player_move`. [blocked: skeletal-animation]
- [ ] **villagers-from-disc** — villager models and animations per species/villager. [blocked: skeletal-animation]
- [ ] **render-look** — camera angle and follow, the curved-horizon world, and lighting, matched to the original. Compare against Philip playing his own copy of the real game, not ripped screenshots kept in the repo. [blocked: town-from-disc]
- [ ] **render-perf** — batching/instancing, 60 fps in a release build. [blocked: town-from-disc, world-objects-from-disc]

### M4 — UI, audio, feel
- [ ] **ui-from-disc** — research: fonts, dialogue window and HUD textures from the disc. Fall back to our own open-licence UI when there's no disc.
- [ ] **dialogue-polish** — letter-by-letter text, name plate, speaker voice blips.
- [ ] **pause-menu** — Save / Settings / Quit (Settings holds the disc path). [blocked: app-states]
- [ ] **title-screen** — back burner; the flow can follow rustimal-crossing's `title_demo.rs`.
- [ ] **audio-from-disc** — research: the original's audio formats (sound banks, sequenced music) and how much `logic::audio` already covers. Then footsteps, UI sounds and music. Large; may be split later.

### Later (after the slice)
Tree shaking, furniture placement, day/night lighting, daily rollover, mail
and shop polish, new-game flow, more disc formats/regions.

### M5 — Slice hardening
- [ ] **playtest-script** (P) — a step-by-step checklist for the full "done when" list; Philip runs it with his disc.
- [ ] **bug-bash** — fix everything the playtest turns up.
- [ ] **windows-release** — a release build zip (no disc content in it) and plain-English steps: install, then point the game at your own disc image.

### Suggested order
disc-setup decision → M2 disc tasks (disc-reader and yaz0 can run in
parallel) → gx-backend-bevy → model-viewer playtest → the rest of M3, with
M4 in parallel → M5.

## Done (pre-board, with Philip)
- Bevy 0.14 → 0.19 upgrade (branch bevy-0.19-upgrade, merged to main).
- B0001 query-conflict fix (Without<> disambiguators in mail.rs / quest.rs).
- Phases 0–3: plane, town render, villagers + dialogue, save/load.
- Workbook bevy-upgrade-workbook.xlsx through finding #21.
