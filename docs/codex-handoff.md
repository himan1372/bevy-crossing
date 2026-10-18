# Codex continuation prompt - 2026-10-18

Continue in `C:\Users\phili\game c\bevy-crossing`, Windows PowerShell, branch
`main`. You are Codex. Philip authorizes committing and pushing completed work;
never force push. Read current repository state before acting. This prompt is
continuity, not a replacement for the coordination files or Philip's new message.

Latest gameplay commit is `686a09db`, pushed to origin/main. A documentation-only
handoff commit follows it. All gameplay work below is committed; there is no
interrupted source edit to resume. Expected handoff state: no Codex task claim,
running Cargo process or own GUI preview. Verify rather than assuming.

## Read first

1. Applicable ancestor/repository `AGENTS.md` files. None were found in the
   searches so far; check current state.
2. Project skills, starting with:
   - `.claude/skills/rust-workflow/SKILL.md`
   - `.claude/skills/task-coordination/SKILL.md`
   - `.claude/skills/bevy-019/SKILL.md`
   - `.claude/skills/gc-iso-reader/SKILL.md` (pointer), then
     `skills/gc-iso-reader/SKILL.md` (actual instructions/tool).
   Read `.claude/skills/bevy-animation/SKILL.md` for animation work and relevant
   `.claude/skills/asset-pipeline/SKILL.md` / `.claude/skills/blender-gltf/SKILL.md` guidance.
   These contain outdated GLB and edition-2024 assumptions: verify against the
   actual runtime-disc architecture and installed APIs. Do not restart GLB import.
3. `.claude-agents/ROADMAP.md`, especially its latest migration/furniture entries,
   then `.claude-agents/backlog/migrate-rustimal-modules.md` and relevant task notes.
   Historical sections can describe limits superseded by later task evidence.
4. `comms.txt`. Announce identity/work there; append only. Inspect claims using
   the root script before claiming work.
5. `Model-research/report.md` and `Model-research/index.html`. Earlier agents
   actually read them. Four referenced research notes were absent; never claim
   to have read missing material. Ask Philip for paths only if needed.

Relevant backlog notes, all under `.claude-agents/backlog/`:

- `furniture-movement-animation.md`: original bounded movement adapter/limits.
- `furniture-birth-pickup-animation.md`: accepted bounded leaf/birth/pickup slice.
- `furniture-storage-animation.md`, `furniture-playtest-access.md`.
- `player-home-entrance.md`, `furniture-drawer-animation.md`.
- `room-loose-diary.md`, `diary-placement.md`, `stationery-room.md`.
- `fruit-shell-room.md`, `furniture-carry-loose.md`, `furniture-turn-loose.md`.
- `compiler-memory-failure.md`: resolved Windows allocation incident.
- `audio-acceptance.md`, `model-actions-net-swing.md` for other open feedback.

## Standing rules

- Claim one task at a time through root `task-claimer.sh`, identity `codex`.
  Never manually edit `.claude-agents/claims.jsonl`. Release the claim at handoff;
  check ownership after a crash. Some historical Muse records use `status`
  instead of `action`; do not rewrite them or infer completion from them.
- Example PowerShell invocation:
  `& "C:/Program Files/Git/bin/bash.exe" task-claimer.sh my-task codex`.
  Use the same root script for `claim <slug> codex` and `release <slug> codex`.
- Preserve unrelated modified `.gitignore`, untracked `savegame.json`, and user
  settings. Never stage, overwrite, delete, or load the actual save for automation.
  Take a fresh save hash at the start/end of work; user play can legitimately
  change it between sessions. Last observed SHA256 was
  `26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB`.
  Older ECD1.../DD897... baselines in historical notes are not current baselines.
- Models, textures, fonts and audio load at runtime from Philip's disc. Never
  commit extracted bytes, derived assets, screenshots or synthetic save fixtures.
  Keep no-disc fallback playable and preserve numeric room data without profiles.
  Content is N64-derived Gfx display lists, CI textures and cKF rigs/clips from
  main.dol/foresta.rel.szs. Existing `app/src/disc_gfx.rs`, `app/src/skeleton.rs`
  and `disc/src/rom.rs` implement this path; no GLB conversion is needed.
- Reuse original C and existing `logic/` translations; audit callers and semantics
  before copying. Do not assume the newest checkout/source automatically wins.
  Use existing native kernels; do not replace them with parallel gameplay rules.
- Manifests use Rust edition 2021 and Bevy 0.19; installed Bevy is 0.19.1.
  Last checked compiler: rustc 1.99.0 stable. Verify installed APIs for changes.
- One Cargo process at a time. Poll running sessions to completion. Typical gate:
  `cargo check --workspace -j 1`,
  `cargo clippy --workspace --all-targets -j 1 -- -D warnings`,
  `cargo fmt --all -- --check`, then appropriate builds. Capture/default/release
  builds are used for playable changes; Philip runs `cargo run --release` himself.
- Do not add or run tests without Philip's request. Historical pilot authorization
  is not blanket authorization. The skill's test gate does not override this.
  Documentation-only updates need no Cargo rebuild/test run.
- Give meaningful updates about once a minute. Philip requested efficiency after
  reporting 14% usage left for five hours; do not assume that historical quota is current.
  Keep scope bounded, avoid redundant audits/builds and broad module batches.
- Use local Ollama qwen2.5-coder:7b for useful bounded mechanical work when
  available; review output against source. It has omitted cases and invented
  comparisons. Recent command/localhost:11434 availability checks failed; direct
  source review was used. Do not install/restart it or keep retrying gratuitously.
- No subagent delegation was authorized in this session. Follow current user and
  system/developer instructions about delegation.
- Keep roadmap/task notes current. Separate implementation, bounded checks and
  Philip's actual acceptance. Never mark M4 or migration complete from compilation.
- Git index/commit, Git Bash startup and GUI launch may require tool escalation.
  Use the tool approval mechanism; existing commit/push authorization persists.

## Sources and migration

Original C: `C:\Users\phili\game c\shmanimal crossing\ACGC-PC-Port`.
Public decomp supplied by Philip: https://github.com/ACreTeam/ac-decomp.
Local Rustimal: `C:\Users\phili\game c\rustimal-crossing\rust\src`.
Canonical migration snapshot is pinned in this repo's Git objects at
`122867f762122eb266f04e319af0f0c31d54f476`.
Inspect with `git show <hash>:rust/src/<module>.rs`; never merge the source repo.
The local source checkout can be older than the pinned snapshot.

`docs/rustimal-migration-inventory.md` audits 194 module paths: 104 source-only,
90 overlapping (88 differing/2 matching at inventory snapshot), plus 27 Bevy-only
paths to preserve. All nine graphics modules already overlap. This means review
module by module, not 194 new imports or wholesale replacements.

Muse's furniture pilot was corrected in `0cfdc4a8` / `25481048`: native save-angle
parity, FTR0/FTR1 bank encoding, actor loading/budget/profile gates, independent
layer occupancy and owner-cell storage save-back/restore. Historical evidence:
18 focused furniture tests; 528 unit + 10 integration logic tests. These were
previous verification, not new tests or proof of complete retail behavior.

## Current playable implementation

| Commit | Bounded change |
|---|---|
| `fd9516df` | Walkable home landing and facing-required interior exit. |
| `b5fc1928` | 16 common drawer families, original clips/sounds, front obstruction. |
| `c860d4a3` | Original room diary rendering and atomic pickup. |
| `8430a444` | Diary placement from pockets on floor/surfaces. |
| `da7b37a1` | Stationery bundles use the same path with quantity/design intact. |
| `f9bdad10` | Six fruit and eight shells/coral use that path. |
| `bf248ca1` | Push/pull carries supported loose surface items, including mixed stacks. |
| `686a09db` | Loaded single-cell/square turns, relative offsets and source yaw flags. |

Controls: `I` opens pockets, select a supported normal item then **Place in room**.
`Backspace` picks up visible loose goods/top furniture before the supporting base.
With empty hands, hold `G` beside furniture: toward/away push/pull, sideways turn;
release to let go. `P` pauses, `F5` saves, `F9` reloads.
Inside the player's house, `F3` grants a blue bureau (drawers) and `F4` grants a
blue cabinet (wardrobe), to an empty pocket only. No overwriting full pockets.
These are playtest aids, not general item spawning or retail progression.

Important behavior/invariants:

- Player-home wood is source geometry x[-1,1], z[2,4] relative to room origin.
  Movement uses radius-inset carpet plus landing. Exit requires feet on landing,
  facing the south doorway (cosine>=0.9) and hitting its finite interaction panel
  with `E`. Furniture placement stays on carpet. Villager-room bounds/collision
  remain independent; no-disc room has its own playable central exit area.
- 19 audited wardrobe/closet rigs and 16 drawer rigs use original cKF open/wait/close
  and player clips. Doors/drawers remain open through choices/notices/pockets and
  close after completion/cancel. Source front contact gates remain.
- Blue bureau only opens the bottom drawer in the original model: other fronts
  belong to its static base. Do not add independent upper-drawer controls.
- Wide drawers check two MAIN cells in front, rejecting furniture-bank items or
  RSV_FE1F, not arbitrary collision bits/loose goods. Single-cell TypeA bypasses
  this check. Source BUBU feedback is currently a short blocking notice.
- Drawer geometry uses DynamicSkinnedMeshBounds: large original joint/vertex
  offsets otherwise culled the blue bureau. NoFrustumCulling is only an error
  fallback for failed bound generation. Classic drawers use their source
  sum_clchest01 rig alias. Do not undo these fixes.
- Room saves retain four 16x16 numeric layers and switches. Absent home_furniture
  means defaults; explicit empty means empty. Only default rooms get source
  starter diary 0x2B00 at secondary(1,1), above the orange box. Existing rooms are
  not backfilled or altered to grant a diary.
- Supported loose families: diaries 0x2B00..2B0F; stationery 0x2000..20FF (64 designs,
  four encoded quantities); fruit 0x2800..2804 and 0x2807; shells 0x2514..251B,
  including coral. Mushroom/candy and other loose categories remain unported.
  Never strip stationery quantity bits while moving/picking up a bundle.
- Loose goods use original shop-goods indoor display lists. Source tries front
  surface first, then current floor cell and eight alternating adjacent cells.
  Empty numeric/actor cells, eligible Surface metadata and room bounds matter.
  Placement/pickup/refused moves are atomic. Stored goods are not visible tops.
- Storage accepts normal ITEM1 goods except fish, insects, turnips, exercise
  cards and knife/fork, per source mSM_check_item_for_furniture. Furniture leaves
  are excluded by retail. Music-player disks have a separate flow; wardrobe
  storage is not music playback. Rendering support is not the storage whitelist.
- F5 snapshots cloned actors because KeepItem2Fg clears written actor contents;
  saving must never empty a live wardrobe. Preserve owner-cell/layer storage,
  independent occupancy, conditions, switch masks and explicit-empty data.
- Carrying collects supported loose tops on copied layers, reuses native actor
  relocation, restores destination cells after ownership/empty-cell checks, then
  validates the player path before publish. Items animate with base progress.
  Mixed furniture/loose tops are supported. Unsupported tops still refuse.
- Loaded turns now work for TypeA (single cell) and TypeC (square), orbiting around
  the same centers as native top actors. Loaded TypeB/long-piece loose turns
  still refuse. Do not merely remove that refusal before fixing contact pivots.
- Original goods flag 1 controls secondary-layer model yaw. Diary/paper/shell
  models turn; fruit positions orbit while their own model facing stays fixed.
  Loose angle state follows runtime SG_angle_y: retained through movement and
  refresh, reset on placement/pickup, F9 restore or fresh process. Numeric item
  cells and base angle save; loose-facing angles are not serialized. This is not
  a claim that F9 itself exists or behaves identically in retail.

## Files and source entry points

- `app/src/player_room.rs`: room ownership/transactions, profiles, numeric DTO,
  snapshot/restore, occupancy, collision refresh, pockets/storage handoff.
- `app/src/player_room/movement.rs`: input grip, atomic app carry/turn adapter,
  player-path gates, transient carried visuals/angles and source clip progress.
- `logic/src/furniture_motion.rs`: unchanged native actor/stack relocation,
  switch save-back and current conservative rotation gates. Primary-cell TypeB
  pivot is an acknowledged approximation, not full retail contact-dependent pivot.
- `logic/src/furniture.rs`: corrected kernels including second-layer placement
  and drawer obstruction. Inherited judge_place_furniture approximates search;
  app furniture search uses C adjacent-unit offsets/shared footprint checks.
- `app/src/player_room/loose.rs`: supported goods, visible-layer/storage filtering,
  placement/pickup targets, runtime angle table and source model spawning.
- `app/src/player_room/presentation.rs`, `app/src/player_room/storage.rs`: furniture birth/pickup
  and wardrobe/closet/drawer demos. Loose placement/pickup is still immediate.
- `app/src/furniture.rs`, `app/src/furniture/storage.rs`: runtime profiles/rigs,
  animated skin bounds. aMR_layer_set_info is separate from aFTR_PROFILE.
- `app/src/shop_display.rs`: shared goods-entry/model lookup and rotation flags;
  preserve ordinary shop behavior when editing helpers.
- `app/src/pockets.rs`, `app/src/pockets_disc.rs`, `app/src/save.rs`,
  `app/src/input.rs`, `app/src/dialogue.rs`:
  existing retail renderer, isolated mail/selling/storage branches, persistence
  and exclusive input focus. `app/src/house.rs` / `app/src/interior.rs` own doors/room geometry.

Original source under the C root:
`src/actor/ac_my_room.c`, `ac_my_room_action.c_inc`, `ac_my_room_move.c_inc`,
`ac_my_room_draw.c_inc`; `src/actor/ac_shop_goods.c` / `ac_shop_goods_data.c_inc`;
`src/game/m_tag_ovl.c` and `m_home.c`; `src/furniture/` constructors.
Fit/UnFit/GetItemPosOnMovingFurniture and DrawItemOnMovingFurniture define
carried offsets/angles; mTG_room_put_proc defines loose placement routing.

## Validation and actual acceptance

All recent slices passed workspace check, warnings-denied all-target clippy,
format, capture/default builds and release builds. Latest release build passed
in 2m 38s. No tests were added/run for these recent slices. Detailed evidence/limits
are in each task note; do not substitute old pilot-test counts for current proof.

Latest disposable previews cover single-cell and four-cell carrying/turning,
diary/paper/fruit/shell flags, mixed tape-deck tops, switch movement, orbital
mid-turn poses, left/push/pull/right numeric roundtrip, mid-motion F5/F9, angle
reset on replacement/reload, and long/unsupported refusal without mutation.
No WARN/ERROR in latest preview log. Older notes contain bounded no-disc checks;
not every later slice repeated a new no-disc/fresh-process cohort.

Philip explicitly accepted the first placement/storage slice (collision,
remembered furniture, saving), bounded movement, and bounded birth/pickup
presentation. He said the entrance **walkable area is good**; that does not
independently certify every exit/facing edge. He observed the bottom-only drawer
and was fine with it if original; the original model was verified. Recent
diary/stationery/fruit-shell/carry/loaded-turn slices have implementation evidence,
but no explicit comprehensive Philip playtest acceptance. Repeated "next/do it"
messages authorize work, not blanket retail acceptance. Prioritize reported defects.

## Disposable preview and Windows build continuity

Never automate `cargo run` from repo root: it can load the real save. Build first,
then launch your executable with a disposable Temp WorkingDirectory/synthetic save,
explicit `--disc "C:/Users/phili/game c/bevy-crossing/rom/Animal Crossing (USA).iso"`,
and `Start-Process -WindowStyle Hidden`. BEVY_CROSSING_ISO alone does not select the
disc for an app preview. Close only your own preview before rebuilding.

Latest evidence folders under Temp: bevy-crossing-home-entrance-review,
bevy-crossing-home-entrance-nodisc, bevy-crossing-furniture-drawer-review,
bevy-crossing-diary-review, bevy-crossing-diary-placement,
bevy-crossing-stationery-review, bevy-crossing-fruit-shell-review,
bevy-crossing-carry-review and bevy-crossing-turn-review. Keep evidence there.
All our previews are closed; latest was PID 25604, not a PID to stop blindly later.

Capture enables BRP at localhost:15702. Discover available tools; freshly locate
the player entity instead of reusing old IDs. Key sends are asynchronous holds;
allow them to settle. Confirm house entry before interpreting screenshots.
Seed 12345 synthetic doorstep was (-2.9343145,4,-26.934208), house faces diagonally:
NW facing worked; fully west missed its finite panel. Interior origin is (3000,0,3000).
Old unconfirmed entries/stale-ID attempts were excluded and repeated in task notes.

Past release allocation failure was Windows commit pressure: two unrelated
SteelSeriesCaptureSvc processes held about40 GiB private memory. Philip approved
stopping only then-current PIDs 13580/26448. The same unchanged release build then
passed; restarted capture processes were left running. This does not authorize
stopping new PIDs, changing services/startup/page-file/compiler flags, or assuming
a Rust regression. Diagnose current memory/commit pressure if it recurs; get
permission before interrupting unrelated user capture. Existing Cargo profiles
also deliberately avoid full debug PDBs after an older linker limit incident.

## Broader roadmap and next work

M0-M2 and main M3 loading are complete; M4 remains open. Title and rename stay
deferred. Original disc player/villager rigs and meshes already load; do not
restart model import. Philip's net appearance mismatch remains unresolved.
Dedicated retail pockets/mail/shop/pause/HUD tasks remain on the roadmap;
pockets/mail appearance received positive feedback. Shop is walk-up merchandise
with Nook following, not a buy catalog. Camera follow must target the marked
world camera, never pockets preview cameras.

Preserve Philip's audio report exactly: **"there is absolutely zero audio overlap."**
Voices, accents and music-note playback were accepted; no town tune is configured.
Other lifecycle/listening/fidelity checks remain in audio-acceptance.md; this
statement alone does not close audio or M4. M5 hardening follows the roadmap's
playtest/bug-bash/release work, not merely successful compilation.

Latest proposal, NOT yet authorized/claimed/implemented: source-correct turning
for long 1x2 TypeB furniture, selecting the pivot from grab/contact side and
checking the swept space; then enable supported loose-loaded turns safely.
Philip requested this handoff instead of approving that proposal. Determine
authorization from his new message, rather than silently starting it.

If he authorizes it, create/claim one bounded task and audit original
aMR_DecideKurukuruDirect, aMR_PlaceKurukuruFurniture, aMR_RotatePermission,
aMR_TypeChangeByRotate and aMR_SearchNextSituation plus callers/tables. Compare
the existing native relocate adapter and source contact/base-position transforms.
Keep accepted TypeA/TypeC movement, mixed stacks, storage ownership, switches,
atomic refusal, save/load and no-disc numeric preservation intact. Do not simply
relax TypeB guards or assume a center rotation reproduces retail contact pivots.

Other explicit gaps: exact player root/contact/slip and rotation sweep fidelity,
loose throw/pickup animations, birth dust/material behavior, unsupported loose
families, special per-type furniture constructors, room upgrades/decorating,
diary writing UI and full retail storage scripts. No automatic import batch.

This handoff was rewritten at Philip's explicit request, superseding earlier
instructions to leave it untouched for those earlier tasks. Do not spend future
implementation turns rewriting it unless requested. Keep task notes/roadmap/comms
current instead, and obtain concrete Philip acceptance without inventing it.
