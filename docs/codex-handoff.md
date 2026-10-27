# Codex handoff - 2026-10-25

## Claude continuation - 2026-10-27 (read this first)

Identity `claude`, branch `main`, all work below is committed and pushed (last `3125bb56`);
only `.gitignore` and `savegame.json` are untracked/modified and must NOT be committed.
Workflow that held all session: read the repo skills; write `.claude-agents/backlog/<slug>.md`
and `bash task-claimer.sh claim <slug> claude`; sequential `cargo -j 1`; QA only in a Temp
dir with a copy of the capture exe (`cargo build -p bevy-crossing --features capture`),
synthetic `savegame.json`, BRP keys/screenshots (never touch the real save); gates =
`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -j 1 -- -D warnings`,
default and `--release` builds; no test runs except a ported kernel's own tests; then update
roadmap docs/`comms.txt`, move the note to `completed/`, `task-claimer.sh complete`, commit
explicit files with the `Co-Authored-By: Claude Sonnet 5.5` line, push. Philip's rules: graphics
from the disc; new logic as disc-independent kernels in `logic/` (check rustimal-crossing
`rust/src` first); decomp reference is `../shmanimal crossing/ACGC-PC-Port/src` (== ac-decomp).
QA tips: enter shops by `teleport(-15.9,4,-35.9)` then `KeyW+KeyD` 400 ms, `KeyW` 900 ms
(teleporting inside breaks state); Nook talk from his north side; launch with `-Hour 14` for
sale-day tests; never pipe `launch.ps1` (it hangs).

Done this session (each has `completed/<slug>.md`): password service (user/popular/famicom/
magazine/card-e codes, send, hand-over, arm clips, retail letter grid), Joan the Sunday
turnip peddler (`app/src/joan.rs`, money/turnip hand-over demos, walks away after a talk),
generic hand-over (`app/src/shop_nook/handover.rs`: Party/Item/Request; player plays arm clips
via `NpcClip`), cuckoo catalog fit (`furniture::FitBounds`), sale-day balloon hand-over,
field rank + player id (`logic/src/field_assessment.rs`, `app/src/field_rank.rs`,
`PlayerIdentity.id`, saved `dust_flag`; feeds bargain scheduling and insect hatching),
shop spotlight rare furniture (tiers 2/3, `shop_rare` save field, FE09 display).

Also done after that (each has `completed/<slug>.md`): diary and letter writing on the retail grid
(`diary-retail-grid`, `letters-retail-grid`; text-cursor kernel `logic/src/diary.rs` with `Limits`; Ctrl+arrows
text cursor, F4 Start, PgUp/PgDn addressee), rare-furniture notice letter (`shop-rare-notice`,
`logic/src/leaflet.rs::rare_notice_gate`, saved `shop_progress.played`), rare piece purchase and sold sign
verified (`shop-rare-sold-sign`, no code change), villager move-in by field rank (`villager-move-in-rank`,
`field_rank::rank_at_start`), fish spawn by field rank (`fish-spawn-field-rank`, faithful
`logic::ecology::fish_select`), grid leftovers (`grid-leftovers`: Ctrl+X accent exchange with its plate,
typed-key voices, R face) and the sliding cloth background of the secret-code windows (`grid-window-scroll`).

Known gaps (the grid cursor pulse is done as the text-caret fade, `completed/grid-caret-pulse.md`; the source has no key pulse or editor rumble): escort/twin audited (`completed/shop-twin-escort-audit.md`: tiers 0-3 follow fixed; twins' upper floor built in `completed/shop-twins-upper-floor.md`; stairs goodbye done in `completed/shop-twins-finish.md`; forced greeting remains); grab-bag capacity warning panel (done: `completed/grab-bag-warning-panel.md`, other mWR warnings not converted); Nook
present/hand-over frames not eyeballed; bargain RNG still a local adapter; grid cursor pulse and pad
rumble; accent swap in the secret-code windows; the voices and receiver-window scroll were not
heard/driven; Rewrite of a letter was not driven; paper roll-in animations. Candidate next tasks: the
grid cursor pulse, the escort/twin choreography audit, the grab-bag warning panel. Parent retail/M4 and
Philip's acceptance stay open; Philip chooses the next task.
QA traps learned: a capture exe is needed for BRP (a default `cargo build` overwrites `target/debug`);
a first shop visit with a loan starts Nook's house talk and Space on its Yes/No orders a house
expansion; face a shop display with a short `KeyW` (20 ms) before `E`; a modal (mail notice) can block
movement keys until Space/Escape.

## Latest continuation - 2026-10-26

Halloween stock (`0e0e1e90`) and November grab bags (`cd06b1e0`) are pushed.
Scheduled bargain sales (`ce5a3583`) and festive-candle rendering (`69a8c092`)
are pushed. Turnip price inquiry/selling (`a509cab3`) is pushed; consult
`completed/shop-turnip-service.md` and latest comms/claims for publication and
validation. Native daily weekly-trend prices, bundles and Sunday/date spoilage
are connected. Legacy saves without an age clock adopt their first observed
day. Joan (`app/src/joan.rs`, `completed/joan-turnip-peddler.md`) now sells turnips on Sunday mornings with the money/turnip hand-over animations (generalised `shop_nook/handover.rs`: Party/Item/Request, player arm clips via NpcClip on the player) and walks to a new spot after a talk. Original global RNG remains separate.
2026-10-26 claude: the counter's **Say code** check is connected for user and
popularity-contest passwords (`completed/shop-password-service.md`, codec in
`logic/src/password.rs`, flow in `app/src/shop_nook/password.rs`) and the **Send a
code** direction (same files, `backlog/shop-password-codes.md` has the evidence).
Famicom, magazine and mini card-e codes redeem too (`backlog/shop-password-codes.md`);
the retail letter grid is in (`completed/shop-password-grid.md`, kernel
`logic/src/editor_keyboard.rs`, disc drawing `app/src/shop_nook/editor.rs`) and can be reused
for the diary and letters. The present
hand-over now plays (`shop_nook/handover.rs`, kernel `logic/src/hand_over_item.rs`,
pure and disc-free); Nook's transfer/hold arm clips play too
(`completed/shop-password-arm.md`). Frame sync with the kernel, the spring chase and the
mid-message start remain. Philip's rule:
graphics come from the disc, rewritten logic stays disc-independent in `logic/`,
and check rustimal-crossing/pinned Rustimal for existing kernels first. The catalog cuckoo-clock preview fit
is fixed (`completed/cuckoo-catalog-fit.md`).
The original task limits below are historical where these
newer task notes supersede them. Parent retail/M4/Philip acceptance stay open.

Philip's latest Qwen instruction: **optional summaries or explanations only**.
Explicitly request English every time, give only relevant supplied code, keep
outputs short and verify against source. Report response time, useful findings
and errors. A prior six-second sales interval check was wrong and discarded;
the earlier broad grab-bag audit took about 6.5 minutes and missed explicit
duplicate exclusions. Qwen was not needed for the candle or turnip slices. Keep code
decisions, multi-file reasoning, final review and validation with Codex.

Continue in `C:/Users/phili/game c/bevy-crossing`, Windows PowerShell, branch
`main`, identity `codex`. Philip authorizes committing and pushing finished work;
never force push. His latest message controls the task. This file preserves
continuity; it does not authorize starting every proposal below. Philip's usage
budget is tight: prefer targeted greps over whole-file reads, and few rebuilds.

**NEXT INSTANCE: actually read the relevant `SKILL.md` files before implementation.**
Finish the selected task, validation, task/comms/roadmap updates, claim release,
commit and push before telling Philip what is next.

**End every final reply by saying what is needed next.** Name the next concrete
task or required input in a short final sentence, using the current task notes
and roadmaps rather than stale proposals. If nothing remains, say so. This is
a status handoff, not authorization to start another task automatically.
Currently next: the rest of the seasonal work (escort/twin, rare-furniture leaflet, grab-bag warning panel) is blocked or optional; Philip chooses. The field rank and player id inputs are done (`completed/field-rank-player-id.md`).

## Current state (2026-10-25)

Latest gameplay commit **`675ef4df`**, pushed to `origin/main`. All selected
Codex gameplay tasks are completed and claims closed. Parent `retail-shop-ui`,
M4, migration and Philip's playtest remain open. Inspect current claims before
claiming new work; do not revive the old catalog ownership instructions.

The only unrelated working-tree changes are modified `.gitignore` and untracked
`savegame.json`. Preserve both, unstaged. No catalog/furniture source work is
waiting for review or commitment. The guessed green catalog recolour was removed;
keep original disc palettes.

Philip requests token efficiency: targeted reads, reuse evidence, no unnecessary
rebuilds or questions. He allowed **at most one subagent**, only where it saves
tokens; the last implementation used one reused agent for assets/review. Do not
start a team or resume historical agents automatically.

### What was completed since the old handoff

| Commit | Implemented work |
|---|---|
| `372e29bf`, `a51927c9` | Catalog fitting/native palette and transaction verification. Purchase tickets/full-pocket ticket mail, insufficient Bells, five orders full, paid orders and restart delivery checked. |
| `f6f339e6` | Raffle ball scale/roll onto tray, drum/table placement and player clearance including tray overhang. Preserve these fixes. |
| `d6a1d96c` | Empty text meshes caused reproduced letter/raffle renderer faults; skip meshes without glyph geometry. |
| `3a98734d` | Original wallpaper, carpet, stationery and animated music catalog previews. Exact clock preview fitting remains separate. |
| `29ce59a1`, `b4b896ba` | Original pockets/mail tag artwork, name pointers, edge placement and popup motion/input guards. |
| `6dd71f34` | [Shop previews](../.claude-agents/completed/shop-category-previews.md): shirt try-on, wallpaper/carpet room previews, native camera/gesture and restoration before final payment. |
| `d117124f` | [Clothing effects](../.claude-agents/completed/shop-change-effects.md): original sparkle spiral and wear sound on both try-on and return. |
| `bb3243b8` | [Paint](../.claude-agents/completed/shop-paint-service.md): exact display purchase, no pocket item/ticket, pending roof palette applied on startup/load; bounded own-roof palette cache. |
| `895ca05b` | [Multi-selling](../.claude-agents/completed/shop-multi-selling.md): M marks/Sell All, duplicate all/one, paper sheets/remainders and atomic wallet/overflow bags. |
| `42020f3c` | [Selling cases](../.claude-agents/completed/shop-selling-warnings.md): quest refusal, wrapped-slot sale, free junk disposal and mixed priced/junk batches. |
| `675ef4df` | [Higher tiers](../.claude-agents/completed/shop-higher-tiers.md): all four native tier rooms/exteriors/uniforms/clocks, both department floors/stairs, lower paint and upper Timmy services, saved two-day renovations and native hours. |

### Important current behavior

- Normal hours: 09:00-22:00; Nook 'n Go 07:00-23:00; monthly raffle opens10:00.
- Crossing a sales threshold does not instantly upgrade. In town, schedule +2
  calendar dates, close the intervening day, complete/restock when due in town.
  Raffle and November-sale lookahead exclusions apply. Optional version1
  `shop_progress` saves pending/date/notice/visitor. Legacy saves load safely.
- The original notice letter works. Nookington's visitor gate is retained;
  foreign-town travel itself is not implemented. Do not silently waive the gate.
- Paint/tools are downstairs; furniture/clothing upstairs. Upper uses Timmy's
  original message bank, voice and preview camera. Floor state is transient.
- Any quest item in an offer refuses the entire batch. Wrapped presents sell
  directly without invented warning text and are excluded from duplicate totals.
  Zero-value disposal asks confirmation and pays nothing; mixed batches pay only
  priced goods. Turnips remain excluded until their separate service is built.
- All slot/item/condition checks and wallet/removal/bag changes commit together
  on final Yes. One sheet affects the actual selected stack. Overflow uses
  30,000-Bell bags, preferring freed sale slots; no capacity means no mutation.

### Latest verification and limits

Latest tier slice passed check3.05s, warnings-denied all-target clippy8.88s, fmt,
capture19.11s, default19.63s and release4m36s. No test suites added/run.
Synthetic Temp checks cover all four entry/exits and40 saved fields unchanged
except position/playtime; both stair directions; renovation dates/closure/notice/
completion/visitor gate; department paint980; Timmy bed2100 with exact stock,
pocket, ledger and ticket; wallpaper preview/cancel;07/22/23 hour gates; raffle
09 closed/10 open; raffle lookahead exclusion; missing-disc numeric retention.
Screenshots were inspected. Final disc logs had no ERROR/WARN/panic; missing-disc
warning was expected. Owned previews are closed. Do not rerun these builds for
this documentation-only handoff.

Still open: twins' full escort/secondary echo choreography (native echo text
already works), renovation signs/board presentation, native neighboring-field
rewrites, original pockets pre-transfer quest-warning window, remaining poses/
camera/acceptance. Bargain-event calendar and forced chore opening are not yet
modeled. Full diary/custom patterns/further tool actions and remaining M4 UI/
audio/render fidelity stay on their existing roadmaps. Title, rename and held-net
appearance remain deferred by Philip.

The real save was only hashed, unchanged throughout the work:
`C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA`.
Take a fresh hash for future runtime work; never read/copy/load it for automation.

## Read before continuing

1. Check current branch/status and applicable ancestor/repository `AGENTS.md`.
   None were found during this update. Discover hidden files with `rg --hidden`.
2. Read the [shared project workflow](../.claude/skills/references/project-workflow.md).
   It consolidates current source, claims, runtime isolation and validation rules.
3. Read **both roadmaps**:
   - [docs/roadmap.html](roadmap.html): Philip's readable status/search page;
     begin with **Latest work and next steps**, including acceptance and deferrals.
   - [.claude-agents/ROADMAP.md](../.claude-agents/ROADMAP.md): engineering plan
     and appended migration/implementation evidence. Its older heading/sections
     do not override later dated entries.
   For migration, also read the [inventory](rustimal-migration-inventory.md) and
   [migration task](../.claude-agents/backlog/migrate-rustimal-modules.md).
4. Read the latest entries in [root comms.txt](../comms.txt), then the task note
   for the selected work. Older task limits may be superseded by follow-up notes.
   Inspect claims using root `task-claimer.sh` before claiming one bounded task.
5. **Read relevant skills before implementation.** Use the guide below; load
   conditional references only when needed. Do not assume all skills live in the
   top-level `skills/` folder: most are under `.claude/skills/`.

## Skills to use

The eight skills added in `b0f182b2` were refined in `c163b42f` against actual
project code. Their entrypoints are 71% shorter, with shared rules and focused
BRP/packaging references. Start with the relevant one:

| Work | Read |
|---|---|
| Rustimal port/merge | [module-port](../.claude/skills/module-port/SKILL.md) |
| Disc textures/materials | [gc-texture-pipeline](../.claude/skills/gc-texture-pipeline/SKILL.md) |
| Original audio/triggers | [gc-audio-pipeline](../.claude/skills/gc-audio-pipeline/SKILL.md) |
| Runtime driving/screenshots | [brp-visual-qa](../.claude/skills/brp-visual-qa/SKILL.md) |
| Menus/dialogue/HUD | [bevy-ui-screens](../.claude/skills/bevy-ui-screens/SKILL.md) |
| Windows build/package | [windows-build](../.claude/skills/windows-build/SKILL.md) |
| JSON save/load | [save-persistence](../.claude/skills/save-persistence/SKILL.md) |
| ECS/schedule/runtime fault | [bevy-debug](../.claude/skills/bevy-debug/SKILL.md) |

Also read [task-coordination](../.claude/skills/task-coordination/SKILL.md) for
coordination, and [rust-workflow](../.claude/skills/rust-workflow/SKILL.md) plus
[bevy-019](../.claude/skills/bevy-019/SKILL.md) for Rust/API work. Consult
[bevy-animation](../.claude/skills/bevy-animation/SKILL.md) for animation changes.
Some older skills retain edition-2024, mandatory-test, GLB and ECS-access claims
that conflict with current manifests/source or Philip's instructions. Use the
current rules below and verify the actual API; a skill does not override them.

For disc/source inspection, use the canonical top-level
[gc-iso-reader](../skills/gc-iso-reader/SKILL.md) and
[gamedb-cli](../skills/gamedb-cli/SKILL.md). The hidden GC skill is a pointer.
`gamedb` was absent from PATH: use direct `rg`/source inspection rather than
installing it automatically. Skill-creator's `quick_validate.py` was unavailable
because PyYAML is missing; restricted metadata/link/example checks passed instead.
No need to install dependencies merely to repeat documentation checks.

## Current implementation and evidence

Read the linked task for source audit, bounded checks and remaining limits.
These entries describe implementation, not comprehensive Philip acceptance.

| Commit | Implemented slice / task evidence |
|---|---|
| `686a09db` | [Single-cell/square turns with loose tops](../.claude-agents/completed/furniture-turn-loose.md), following push/pull carry. |
| `2edebb0a` | [Long furniture endpoint turns](../.claude-agents/completed/furniture-turn-long.md), all four orientations/both endpoints, mixed tops and shifted owner storage. |
| `ecffb7d5` | [Loose-room placement/pickup animation](../.claude-agents/completed/room-loose-animation.md), original throw/growth/bounce and floor/surface pickup clips. |
| `59c548fe` | [Mushroom/candy room goods](../.claude-agents/completed/mushroom-candy-room.md), completing loose food IDs `0x2800..0x2807`. |
| `83bd9604` | [Money bags/turnip bundles](../.claude-agents/completed/money-turnip-room.md), exact denomination/bundle IDs and shared room transactions. |
| `ed49741b` | [Music items, fans and pinwheels in rooms](../.claude-agents/completed/music-fan-pinwheel-room.md). |
| `63bc0caa` | [Home music registration/playback](../.claude-agents/completed/room-music-player.md), 55-song collection and persistence. |
| `c548e88b` | [Original music grid and handheld toys](../.claude-agents/completed/music-presentation-toys.md), disc frame/font/cloth, fan use and pinwheel motion/sound. |
| `79250311` | [Music source-fidelity follow-up](../.claude-agents/completed/music-source-fidelity.md): M marking, ordered/partial take-out, panel/cloth motion, placement selection/off, front access/Start-Stop and disc-fed device filters/hiss. |
| `5cac5459` | [Standard-shirt room conversion](../.claude-agents/completed/clothing-room-conversion.md): `0x2400..0x24FE` to original mannequin indices `0x1EB..0x2E9`, exact shirt pickup after rotation. Reserved `0x24FF` and custom designs excluded. |
| `6c405ca2` | [Umbrellas/tools](../.claude-agents/completed/umbrella-tool-room.md): 32 standard umbrellas become original stands with exact reverse pickup. Normal/golden tools and worn axes remain loose with exact IDs; existing net/rod Hold retained. Custom designs/new tool actions excluded. |
| `db6e3301` | [Room decorating](../.claude-agents/completed/room-decorating.md): 71 standard wallpapers/carpets per bank, atomic slot swaps even with full pockets, original room textures and floor footsteps, optional saved decoration. Loose rolls support indices 0..66; shop patterns 67..70 decorate but unsupported loose models refuse. |
| `8dcd0039` | [Monthly diary](../.claude-agents/completed/diary-writing.md): shared 12-month notebook, original paper/font, bounded editing, Yes/Rewrite and optional saved diary independent of physical item pickup. Partial native port; full retail calendar/editor/locks remain open. |
| `de9a704a` | [Three TVs](../.claude-agents/completed/television-actions.md): E power, cached original screen cycles, positional audio and native saved switches for `sum_tv01`, `sum_tv02`, `sum_fruittv01`. Movement, turning and pickup preserve ownership/lifecycle. |
| `76ba1b4a` | [Snowman TV and four lamps](../.claude-agents/completed/snow-tv-lamp-draw.md): fourth TV power/screens/audio, full TV placement asset preflight and missing original opaque model parts for bla/whi/gre/hal lamps. Lamp callbacks are static; no lamp switch or illumination invented. |
| `2fae13d2` | [Ten clocks](../.claude-agents/completed/furniture-clocks.md): native local-time hand math, original half-speed cKF animation, extra snow/meza hand lists and antique positional ticking with room/presentation/pause/load gates. No configurable alarm or power action found in audited callbacks. |

Earlier foundation remains: walkable home landing/facing-required exit, original
wardrobe/drawer rigs, atomic diary/stationery/fruit/shell placement and pickup,
owner-cell storage and loose-item carry. See their task notes when editing those
paths. The original blue bureau opens its bottom drawer only; this was verified
against its model. Do not invent independent upper drawers.

Earlier furniture verification remains in the linked completed tasks above.
The newer shop evidence at the top supersedes the old clock-only build summary.
Keep bounded development checks distinct from Philip's actual acceptance.

Philip said **"music is good"**, accepting earlier playback. Preserve his earlier
audio report exactly: **"there is absolutely zero audio overlap."** Voices,
accents and music-note playback also received positive feedback. No town tune
is configured. These reports do not close all audio fidelity/lifecycle work.

Clothing, umbrellas/tools, decorating, diary, TVs, lamp appearance, clocks,
recent loose-item/long-turn behavior, new music presentation/device effects and
toy use still await the corresponding playtest acceptance. Earlier
placement/storage, bounded movement and birth/pickup presentation were accepted;
entrance feedback was specifically that the walkable area is good. Repeated
"next" authorizes progress, not blanket retail acceptance. M4 and migration
remain open; M5 follows acceptance/playtest, fixes and release work.

## Next instance instructions

Read this handoff, both roadmaps and the latest comms first. Then tell Philip you
are caught up and **end your first reply with the exact three numbered priorities
at the end of this file**. They are remaining work, not three completed features.
Do not silently start one; wait for Philip's selection unless his new message
already selects or directs work. Prioritize reported defects over the list.

Keep both roadmaps, task notes and root comms current for gameplay changes.
Documentation maintenance does not change gameplay counts or close M4. Do not
restart implemented previews, paint, selling, tiers, clocks or old room-light
proposals merely because an older handoff called them missing.

## Architecture and invariants to preserve

- Rebuild the engine in Bevy; read models, textures, fonts, maps and audio at
  runtime from Philip's own disc. Content uses N64-derived Gfx/CI/cKF in
  `main.dol`/`foresta.rel.szs`, not a generic BMD/GLB export pipeline. No extracted
  or derived disc assets, synthetic saves or screenshots in commits. No-disc
  fallback stays playable and retains numeric room data without asset profiles.
- Manifests use edition 2021 and Bevy 0.19; installed Bevy source is 0.19.1.
  Packages are `bevy-crossing`, `rustimal-logic`, `disc`. App `capture` implies
  `remote`; neither is enabled by default. No app `dynamic_linking` feature.
  Extras declares compatible `0.22.1`; installed source checked was 0.22.10.
- Original C: `C:/Users/phili/game c/shmanimal crossing/ACGC-PC-Port`.
  Pinned migration source: `122867f762122eb266f04e319af0f0c31d54f476`;
  inspect with `git show <hash>:rust/src/<module>.rs`. The local Rustimal checkout
  can be older. Never merge it wholesale or edit it just to create a shim.
  Compare callers/semantics; preserve Bevy fixes and reuse native kernels.
- Inventory snapshot: 194 source paths, 104 source-only, 90 overlapping
  (88 differing/2 matching), plus 27 Bevy-only paths. These are inventory counts,
  not the current remaining-work total. All nine graphics modules overlap.
- Room save data includes four 16x16 layers, switches, owner-cell storage and
  music collection, plus optional decoration and diary in the existing version-1
  schema. Clock pose/phase are transient; no new clock save schema was added.
  Missing `home_furniture` means legacy defaults; explicit
  empty means empty. Keep independent layer occupancy, bank/angle encoding,
  pocket conditions, quantities, wallet and hidden storage intact. F5 must not
  empty live actor storage. Refused transactions must not partially mutate it.
- Carry/turn adapters use copied layers plus native relocation and publish only
  after occupancy/ownership/player-path checks. Source goods yaw flags matter:
  fruit can orbit without turning its model. Loose runtime angles reset on
  placement/pickup/reload; do not serialize them casually as a fidelity fix.
- Retail UI uses disc meshes/materials/font metrics as well as native Bevy UI.
  Preserve modal input focus and action consumption; scene/reload must clear
  transient menus/hands/voices. Existing Nas audio engine and manual serde JSON
  save system are the starting points, not new audio/save frameworks.

Core app entry points: `app/src/player_room.rs` and its movement/loose/presentation/
storage/music modules; `pockets.rs`, `pockets_disc.rs`, `save.rs`, `input.rs`,
`dialogue.rs`, `disc_gfx.rs`, `skeleton.rs`, `n64_material.rs`, `sound.rs` and
`sound_rom.rs`. Core native furniture paths: `logic/src/furniture.rs` and
`logic/src/furniture_motion.rs`. Source authority and deeper per-task functions
are documented in the skills/task notes; read the affected callers before edits.
Recent entry points include `app/src/player_room/decorating.rs`,
`app/src/player_room/diary.rs`, `app/src/furniture/television.rs`,
`app/src/furniture/clocks.rs`, and the native `logic/src/diary.rs`,
`logic/src/television.rs`, `logic/src/furniture_clock.rs` kernels.

## Current shop entry points and runtime lessons

- `shop.rs`, `shop/schedule.rs`, `shop/layout.rs`, `shop/layout/follow.rs`:
  persisted renewal/hours, native rooms/portals/routes and owned-root rebuilding.
  `upgrading_today` means pending; use the date-aware renovation helper for closure.
- `disc_town/shop_exterior.rs`, `interior.rs`, `furniture/house_clock.rs`,
  `shop_display.rs`: native tier assets/footprints/clocks and floor-aware displays.
- `shop_nook.rs`, `shop_nook/sales.rs`, `shop_nook/preview.rs`, `raffle.rs`:
  native conversations, exact transactions, temporary previews and tier offsets.
- Insect category is0x2D; turnips0x2F. Do not accidentally block insects again.
  `aNSC_item_tanni_type` is a relocated pointer table, not a flat byte table.
- MovementSet follows Doors/Objects/Tools: ordering room rebuild after movement
  and before Doors creates a cycle. Initialize ShopObstacles before Update.
  Keep ownership markers on room/clock/attendant/HUD roots to avoid duplicates.
- Native door centres and narrow corridors vary by tier. Stop before the portal
  plane, and guard stair reentry. Allow turning time; a wide U-turn can miss stairs.
- Match actual final question text before choosing. Hidden/stale Text and prices
  on earlier pages caused QA to select the wrong thing. Query fresh named entities;
  the blue-bed model is `sum_blue_bed01`, not a guessed name.
- Existing disposable helpers/evidence are under
  `%TEMP%/bevy-crossing-room-critters/higher-tiers-20261025` and the earlier task
  folders named in their notes. Historical PIDs never authorize future stops.
  `install.ps1` copies target/debug: rebuild capture before using it, and copy the
  capture EXE before a default build replaces it. Never launch from repo cwd.
- Debug HOUR_OFFSET wraps modulo24: negative hours advance across midnight.
  The latest Temp launcher accepts target Hour and compensates DayOffset. Set
  explicit dates/opening hours and ISO, not merely BEVY_CROSSING_ISO.
- Same-size own roof reload is cached; size-changing exterior reload still needs
  restart. An unrelated fresh-start empty `field_items` list repopulates20
  records; this was recorded, not fixed by paint work.

## Working safely and efficiently

Claim one task through the **root** script, using your own identity:

```powershell
& "C:/Program Files/Git/bin/bash.exe" task-claimer.sh my-task codex
& "C:/Program Files/Git/bin/bash.exe" task-claimer.sh list-claimed
& "C:/Program Files/Git/bin/bash.exe" task-claimer.sh claim <slug> codex
& "C:/Program Files/Git/bin/bash.exe" task-claimer.sh release <slug> codex
```

Never edit claims manually. Release at handoff. Some historical records use
`status` rather than `action`; do not rewrite them or infer completion from them.
Append only to root `comms.txt`. Philip allows at most one subagent where it
saves tokens; prefer direct work for small tasks. Follow current session limits.

Preserve unrelated modified `.gitignore`, untracked `savegame.json` and settings.
Never load the actual save for automation. Last read-only SHA256:
`C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA`.
Take a fresh baseline for runtime work; user play can legitimately change it.
Never stage the save or copy it into a synthetic test scenario.

For runtime verification, read the local
[BRP launch/protocol reference](../.claude/skills/brp-visual-qa/references/protocol.md).
Build first, launch an absolute EXE with disposable Temp working directory,
synthetic save and explicit `--disc` path. The known local image is
`C:/Users/phili/game c/bevy-crossing/rom/Animal Crossing (USA).iso`; verify it exists.
`BEVY_CROSSING_ISO` alone does not select the app's preview disc. Never automate
`cargo run` from the repository and accidentally load the real save.

BRP uses localhost:15702 and current `world.query`, `world.get_components`,
`world.insert_components` methods plus `brp_extras/send_keys` and screenshot.
Identify an existing listener before attaching. Query fresh entities after scene/
reload; input calls schedule holds, so wait for settled state and modal dismissal.
Preserve exact 64-bit entity IDs (for example, Python integers); do not round
them through JavaScript numbers. Dismiss modals before using F9 in scenario setup.
Inspect screenshots before claiming visual success; check numeric transactions
separately. Keep evidence/helpers in Temp. Close only owned, verified processes;
old task-note PIDs are historical evidence, never current stop targets.

For Rust changes, one Cargo process at a time; poll to completion:

```powershell
cargo check --workspace -j 1
cargo clippy --workspace --all-targets -j 1 -- -D warnings
cargo fmt --all -- --check
```

Then appropriate capture/default/release builds for playable changes. Philip
plays `cargo run --release`; reproduce release-specific faults in that profile.
**Do not add or run tests without Philip's request.** Documentation-only changes
need link/metadata/diff checks, not Cargo. Inspect actual build errors before
changing profiles/toolchains or deleting caches. Earlier Windows failures were
PDB/linker and unrelated capture-service memory pressure; old PID permissions do
not authorize stopping new processes or changing services/page files.
During the snow-TV work, metadata/OOM failures occurred under low available RAM
and commit. Philip freed RAM and explicitly requested the original build plan;
normal check/clippy/capture/default/release builds then passed. Clocks also passed
with normal settings. No permanent profile/toolchain/dependency changes were
needed. Preserve that preference; do not preemptively disable debug info or
incremental compilation, delete caches, or stop user applications.

Use tool escalation when the sandbox blocks required Git/Git Bash/GUI actions;
standing commit/push authorization persists. Stage explicit task files, preserving
unrelated changes. Give meaningful progress updates about once a minute and avoid
redundant reads/builds. Ollama was unavailable; do direct source review rather
than repeatedly retrying, installing or restarting it unasked.

This handoff was updated at Philip's request. Keep implementation turns focused
on the selected task; rewrite this file again when requested or when an actual
handoff needs it. Finish with current evidence, remaining limits and ownership
released; do not invent acceptance or silently start the next proposal.
Complete all authorized repository work and confirm publication/status first,
then tell Philip the next concrete step. End the first catch-up reply with these
three priorities, in this order (do not start them without a selection):

1. Remaining seasonal work (see Currently next above).
