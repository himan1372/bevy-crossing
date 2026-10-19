# Codex handoff - 2026-10-19

Continue in `C:/Users/phili/game c/bevy-crossing`, Windows PowerShell, branch
`main`, identity `codex`. Philip authorizes committing and pushing finished work;
never force push. His latest message controls the task. This file preserves
continuity; it does not authorize starting every proposal below.

Latest gameplay commit: `2fae13d2` (ten furniture clocks and antique ticking).
Latest skill updates: `d9117594` (top-level pair) and `c163b42f` (eight new project
skills). Those commits are pushed. This handoff update is documentation only;
there is no interrupted gameplay edit to resume.

**NEXT INSTANCE: actually read the relevant `SKILL.md` files before implementation.**
Reading this handoff or a skill catalog does not replace reading the skills.
Finish the selected authorized task, required validation, task/comms/roadmap
updates, claim release, commit and push before telling Philip what is next.
If asked only to read this handoff, read the context and report the next proposal;
a subsequent "le go" authorizes that bounded proposal. Do not silently begin
every remaining roadmap item. All previous owned previews and Cargo processes
are closed; historical task-note PIDs are not current process targets.

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
| `686a09db` | [Single-cell/square turns with loose tops](../.claude-agents/backlog/furniture-turn-loose.md), following push/pull carry. |
| `2edebb0a` | [Long furniture endpoint turns](../.claude-agents/backlog/furniture-turn-long.md), all four orientations/both endpoints, mixed tops and shifted owner storage. |
| `ecffb7d5` | [Loose-room placement/pickup animation](../.claude-agents/backlog/room-loose-animation.md), original throw/growth/bounce and floor/surface pickup clips. |
| `59c548fe` | [Mushroom/candy room goods](../.claude-agents/backlog/mushroom-candy-room.md), completing loose food IDs `0x2800..0x2807`. |
| `83bd9604` | [Money bags/turnip bundles](../.claude-agents/backlog/money-turnip-room.md), exact denomination/bundle IDs and shared room transactions. |
| `ed49741b` | [Music items, fans and pinwheels in rooms](../.claude-agents/backlog/music-fan-pinwheel-room.md). |
| `63bc0caa` | [Home music registration/playback](../.claude-agents/backlog/room-music-player.md), 55-song collection and persistence. |
| `c548e88b` | [Original music grid and handheld toys](../.claude-agents/backlog/music-presentation-toys.md), disc frame/font/cloth, fan use and pinwheel motion/sound. |
| `79250311` | [Music source-fidelity follow-up](../.claude-agents/backlog/music-source-fidelity.md): M marking, ordered/partial take-out, panel/cloth motion, placement selection/off, front access/Start-Stop and disc-fed device filters/hiss. |
| `5cac5459` | [Standard-shirt room conversion](../.claude-agents/backlog/clothing-room-conversion.md): `0x2400..0x24FE` to original mannequin indices `0x1EB..0x2E9`, exact shirt pickup after rotation. Reserved `0x24FF` and custom designs excluded. |
| `6c405ca2` | [Umbrellas/tools](../.claude-agents/backlog/umbrella-tool-room.md): 32 standard umbrellas become original stands with exact reverse pickup. Normal/golden tools and worn axes remain loose with exact IDs; existing net/rod Hold retained. Custom designs/new tool actions excluded. |
| `db6e3301` | [Room decorating](../.claude-agents/backlog/room-decorating.md): 71 standard wallpapers/carpets per bank, atomic slot swaps even with full pockets, original room textures and floor footsteps, optional saved decoration. Loose rolls support indices 0..66; shop patterns 67..70 decorate but unsupported loose models refuse. |
| `8dcd0039` | [Monthly diary](../.claude-agents/backlog/diary-writing.md): shared 12-month notebook, original paper/font, bounded editing, Yes/Rewrite and optional saved diary independent of physical item pickup. Partial native port; full retail calendar/editor/locks remain open. |
| `de9a704a` | [Three TVs](../.claude-agents/backlog/television-actions.md): E power, cached original screen cycles, positional audio and native saved switches for `sum_tv01`, `sum_tv02`, `sum_fruittv01`. Movement, turning and pickup preserve ownership/lifecycle. |
| `76ba1b4a` | [Snowman TV and four lamps](../.claude-agents/backlog/snow-tv-lamp-draw.md): fourth TV power/screens/audio, full TV placement asset preflight and missing original opaque model parts for bla/whi/gre/hal lamps. Lamp callbacks are static; no lamp switch or illumination invented. |
| `2fae13d2` | [Ten clocks](../.claude-agents/backlog/furniture-clocks.md): native local-time hand math, original half-speed cKF animation, extra snow/meza hand lists and antique positional ticking with room/presentation/pause/load gates. No configurable alarm or power action found in audited callbacks. |

Earlier foundation remains: walkable home landing/facing-required exit, original
wardrobe/drawer rigs, atomic diary/stationery/fruit/shell placement and pickup,
owner-cell storage and loose-item carry. See their task notes when editing those
paths. The original blue bureau opens its bottom drawer only; this was verified
against its model. Do not invent independent upper drawers.

Recent gameplay slices passed check, warnings-denied all-target clippy, fmt and
capture/default/release builds. The latest clock slice passed check (8.98s),
clippy (24.90s), fmt, capture (1m01s), default (1m02s) and release (3m25s).
Disposable checks cover all ten clock variants, F6 hour changes, minute
continuity, authored motion, exact pause, F5/F9, placement/pickup, hidden/surface
items, mixed powered-TV state, fresh launch and missing-disc numeric retention.
Clear screenshots were inspected after repeating initially obscured views.
Audio diagnostics settled to zero voices/nonfinite samples; this does not
establish audible acceptance. No tests were added/run
for these slices. Do not rerun builds for this documentation-only handoff or
substitute old furniture-pilot test counts for current behavioral evidence.

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

## Next gameplay work

Follow Philip's current request and prioritize reported defects. The current
proposal in `docs/roadmap.html`, **planned and not started**, is:

1. Audit original room-light controls/illumination and remaining special furniture
   callbacks, then select a bounded source-correct slice using native kernels.
   Static lamp callbacks and on/off model names alone do not establish a power
   action or a Bevy light. Audited clocks do not expose configurable alarms.
2. Remaining fish/insects/balloons require source audits; custom designs require
   player-pattern support. Further held-tool actions and full diary calendar,
   editor presentation/contact pose and other-player locks are separate work.
3. Further retail comparison of music/toys: exact hardware DSP/rounding/resampling
   and full warning/choice-window animation remain open. Pinwheel environmental
   wind depends on weather.
4. Finish remaining pockets/mail/shop/pause/HUD and audio/render M4 acceptance,
   then M5 hardening/playtest/release.

**Deferred by Philip:** held-net appearance, title screen and rename. Do not
resume net appearance just because it remains on the roadmap. Long-furniture
turning and loose throw/pickup animation are implemented, not the next proposal.
Exact player root/contact/slip, conservative rotation/player sweeps and special
per-type furniture behavior still have fidelity limits described in task notes.

Keep both roadmaps, the relevant task and comms current for gameplay changes.
The HTML distinguishes implemented work from acceptance and proposed work;
internal skill/handoff maintenance must not inflate gameplay progress counts.
Current roadmap counts are 79 items: 46 implemented, 6 in progress, 27 planned.
These are tracked-item counts, not a game completion percentage. Umbrellas,
decorating, the bounded diary, four TVs and ten clocks are already implemented;
do not restart those slices from the previous handoff's obsolete proposals.

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
Append only to root `comms.txt`. No subagent delegation is authorized by this
handoff; follow the current session's delegation instructions.

Preserve unrelated modified `.gitignore`, untracked `savegame.json` and settings.
Never load the actual save for automation. Last read-only SHA256:
`26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB`.
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
then tell Philip the next concrete step. At this handoff the next proposal is
the original room-light control/illumination audit, not another clock alarm UI.
