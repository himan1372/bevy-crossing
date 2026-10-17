# Codex handoff prompt

Continue work in `C:\Users\phili\game c\bevy-crossing`, Windows PowerShell,
branch `main`. You are Codex. Philip authorizes commits and pushes of completed
work; never force push. Read the repository's current state before acting: this
prompt is context, not a replacement for the coordination files.

## Read first

1. Applicable `AGENTS.md` files (none found in the prior workspace search).
2. Project skills in `.claude/skills/` and `skills/`, starting with
   `rust-workflow`, `task-coordination`, `bevy-019`, and both the gc-iso-reader
   pointer and `skills/gc-iso-reader/SKILL.md`. Read relevant asset/animation
   skills. Verify their claims against the current runtime disc architecture:
   GLB conversion instructions and edition-2024 claims are outdated here.
3. `.claude-agents/ROADMAP.md`, then the relevant backlog notes, especially
   `.claude-agents/backlog/migrate-rustimal-modules.md` and its latest evidence.
4. `comms.txt`. Announce yourself and your work there; **append only**.
5. `Model-research/report.md` and `Model-research/index.html` in the repository.
   Earlier agents actually read these. Four research notes they reference were
   absent; do not claim to have read missing material. Ask Philip for paths if
   they become necessary.

## Standing rules

- Claim one task at a time through root `task-claimer.sh` before implementation.
  Use identity `codex`; never manually edit `.claude-agents/claims.jsonl`.
  Release the claim when handing off. Check ownership after crashes.
- Preserve unrelated modified `.gitignore` and untracked `savegame.json`.
  Do not stage, overwrite, delete or load the actual save for automated work.
  Preserve user settings too. Take current save hashes, not historical baselines.
- Models, textures, fonts and audio load at runtime from Philip's disc.
  Never commit extracted disc bytes or derived assets. Keep no-disc fallback
  playable and preserve its numeric saved room data even without disc profiles.
- Reuse original C and existing `logic/` translations; audit callers and source
  semantics before copying. Do not assume the newest source automatically wins.
- Manifests use Bevy 0.19 (installed 0.19.1) and Rust edition 2021. Check installed
  APIs. One Cargo process at a time; poll running sessions until completion.
- Check, lint with warnings denied, format and build appropriately. Do not add
  or run tests unless Philip requests testing/implementation verification.
  Historical pilot-test authorization is not blanket authorization for new work.
- Give meaningful progress updates roughly once a minute. Use local Ollama
  qwen2.5-coder:7b for useful bounded mechanical work when available, and review
  all results against source; it has omitted cases and invented comparisons.
- GUI previews use a disposable Temp working directory and synthetic saves,
  with `--disc "C:/Users/phili/game c/bevy-crossing/rom/Animal Crossing (USA).iso"`.
  The application loader uses the flag/settings; setting BEVY_CROSSING_ISO alone
  does not select the disc for an app preview. Close only your own preview before
  rebuilding. Launch Start-Process with -WindowStyle Hidden.
- Keep roadmap/task notes current. Separate implementation, automated evidence
  and **Philip's actual playtest acceptance**. Do not mark M4 or migration complete.

## Source locations and migration

Original C: `C:\Users\phili\game c\shmanimal crossing\ACGC-PC-Port`.
Public decomp: https://github.com/ACreTeam/ac-decomp (Philip supplied it).
Rustimal checkout: `C:\Users\phili\game c\rustimal-crossing\rust\src`.
Canonical migration snapshot is pinned in this repository's Git objects at
`122867f762122eb266f04e319af0f0c31d54f476`; inspect with `git show`, never merge
the source repository. The local source checkout can be older.

`docs/rustimal-migration-inventory.md` audits 194 source module paths: 104 source
only, 90 overlapping (88 differing/2 matching at the inventory snapshot), and
27 Bevy-only paths to preserve. All nine graphics modules already overlap.
This is a module-by-module review, not 194 new imports or wholesale replacements.

Muse's furniture pilot was reviewed and corrected. Commits `0cfdc4a8` and
`25481048` record that work: native save-angle parity, correct FTR0/FTR1 bank
encoding, actor loading/budget/profile gates and independent layer occupancy,
owner-cell storage save-back and restoration. Historical focused evidence:
18 furniture tests; full logic 528 unit + 10 integration tests. Those counts
are previous verification, not proof that every furniture action is retail.
Some Muse historical claim records use `status` instead of script `action`;
do not manually rewrite them or infer completion from the malformed entry.

## Current feature and relevant files

Philip authorized connecting corrected furniture logic to playable placement
and storage. A PC crash interrupted research before implementation; work resumed
without partially written source. Read the latest task-note section for the
finished slice, exact validation evidence and remaining acceptance checks.

The slice passed workspace check, all-target clippy with warnings denied, format,
capture and default builds. No tests were added/run for it. Disposable previews
covered placement/pickup, ordinary storage transfers, repeated save retaining
live contents, F9 and fresh-process three-content restoration, full pockets/full
storage/cancel, surface stacking/top-first pickup, explicit empty rooms, collision
removal, blocked placement retaining the item, and no-disc numeric preservation.
These are bounded implementation checks, not Philip's playtest acceptance.
Preview folders are under Temp: bevy-crossing-furniture-review and
bevy-crossing-furniture-nodisc-review; screenshots/logs/synthetic saves stay there.
Actual save hash stayed ECD1F9A7D016A0E5862882A704FDB3B88B16AAB51183CA1E7B29084139359BDC.

- `app/src/player_room.rs`: player-owned room, original numeric foreground
  layers, source profile metadata, source adjacent-unit placement search,
  pickup, three-item storage flow, rendering refresh, collision and save DTO.
- `app/src/furniture.rs`: runtime original models and gameplay profile fields;
  `aMR_layer_set_info` is a separate source table, not part of `aFTR_PROFILE`.
- `app/src/interior.rs`: original floor/walls; furniture now comes from room data.
- `app/src/house.rs`: room ownership, doors, movement and collision; preserve
  villager-room collision when updating the player room.
- `app/src/pockets.rs`, `pockets_disc.rs`: existing retail pockets renderer,
  Place in room and Put in action handoffs. Keep ordinary pockets/mail/Nook
  selling isolated and preserve condition bits.
- `app/src/save.rs`: additive `home_furniture` field. Absent means original
  defaults for old saves; explicit empty means an emptied room. F5 snapshots
  clones so save-back cannot empty a live wardrobe. F9 restores the room too.
- `app/src/input.rs`, `dialogue.rs`: exclusive focus and furniture-owned results.
- `logic/src/furniture.rs`: corrected source kernels. The inherited
  `judge_place_furniture` approximates the placement search; the app uses the
  C adjacent-unit table and shared footprint checks instead.

Storage accepts normal-condition ITEM1 goods except fish, insects, turnips,
exercise cards and knife/fork, matching `mSM_check_item_for_furniture`. Furniture
leaves are excluded by retail, not arbitrarily omitted. Music disk players
have a separate source flow. Do not treat ordinary wardrobe storage as music
playback. Keep full-pocket/cancel/blocked placement transactions atomic.

This first slice did not finish retail pushing/pulling/rotation, birth and pickup
animations or opening/closing furniture; bounded follow-ups are recorded below.
Remaining full source fidelity includes per-type constructors and behavior,
loose-item rendering (including the previously omitted starter diary), full
room upgrades/decorating or exact retail storage script text. Consult task notes
before expanding scope; preserve anything already accepted.

## Other continuity and next step

M0–M2 and main M3 loading are complete; M4 remains open. Title and rename remain
deferred. The original disc player/villager rigs and meshes already load; avoid
restarting model import. Net appearance still has Philip's unresolved retail
mismatch feedback; do not mark it accepted. Dedicated retail UI tasks exist:
pockets, mail, shop, pause/save, HUD. Pockets/mail appearance received positive
feedback; shop is a walk-up merchandise/Nook-following flow, not a buy catalog.
Camera follow must target the marked world camera, never pockets preview cameras.

Audio report to preserve exactly: **“there is absolutely zero audio overlap.”**
Voices, accents and music-note playback were accepted. No town tune is configured.
Other lifecycle/fidelity checks remain in `backlog/audio-acceptance.md`; do not
infer acceptance from the overlap statement.

Philip subsequently reported collision, remembered furniture and saving work,
and everything checks out so far. That accepts the first placement/storage slice.
He then authorized furniture-movement-animation implementation. Hold G beside
furniture with empty hands; toward/away directions push/pull, sideways turns;
release G to let go. Numeric destination commits atomically before original
disc-backed player/furniture animation; supported stacked actors, storage goods
and switches travel together. P/F5/F9 are supported during this interaction.
See backlog/furniture-movement-animation.md for current evidence and deliberate
limits: TypeB primary-cell pivots/conservative swept footprints still differ
from full retail contact-dependent pivots/friction, and player root/slip/control
timing is a bounded adapter. Philip reports movement works (2026-10-17).
Next implemented slice: furniture-birth-pickup-animation.md, original runtime
leaf icons/placement flight,46-tick reservation wait, native birth pop/shrink
curves and original pickup1 clip with animated-hand leaf retrieval. Numeric
transactions commit first; P/F5/F9 and restart remain safe. Pickup clears switch
bits; pockets close before placement presentation starts. Philip accepts this
bounded birth/pickup slice (2026-10-17: "wokrs good whats next"). Exact dust
(two-texture material), collision growth/edges, loose diary, upgrades and broader
per-type behavior remain follow-ups.
Keep migration/M4 open; no automatic next import batch.
Latest implementation: furniture-storage-animation.md, authorized by "lets get
it". Nineteen audited original common wardrobe/closet models retain animated cKF
joints using existing character skinning. E from the front plays original
open/wait/close furniture and player clips; doors stay open during choices,
notices and pockets, then close after completion/cancel and pockets' slide.
P/F5/pause Save/F9/restart preserve numeric transactions; transient door state
resets closed. Drawers/special constructors remain on prior immediate storage
flow; exact player root/contact snapping and retail scripts remain open.
Check, all-target clippy -D warnings, format, capture/default builds pass. No
tests added/run. Disposable previews cover storage transfers/cancel/full limits,
pause Save/F9, fresh-process rotated wardrobe, moving stored goods, stack pickup,
and explicit missing-disc F5/F9/F5 numeric preservation. Initial Windows paging
failures resolved with single compiler job and unloading our local Ollama model;
the helper request timed out and supplied no evidence. All own previews closed;
screenshots/logs/synthetic saves stay Temp furniture-storage-review/nodisc.
Obtain Philip's storage-animation playtest before expanding. Migration/M4 open.
Current user-save SHA256 at storage implementation start/end:
DD89745CCFEDDEAEA619A1C931FEE5233EC533C1860E3B5A910A00B8121E5721.
