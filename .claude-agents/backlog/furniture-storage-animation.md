# Furniture storage opening and closing

2026-10-17, codex. Proposal following Philip's acceptance of bounded furniture
placement/pickup presentation. Philip authorized implementation ("lets get it").

## Bounded scope

Connect the original wardrobe/closet open and close sequence to the existing E
storage interaction, using runtime disc furniture rigs/clips and the applicable
original player clips. Keep Put in/Take out transactions, item eligibility,
capacity checks and cancellation atomic. Close on completion/cancel according
to source lifecycle; do not invent persistent open state in numeric room saves.

Audit supported constructors first and explicitly identify supported furniture
families. Drawers have separate obstruction rules and must not silently share a
wardrobe approximation. Music disk players remain a separate interaction.

## Source and implementation audit

ac_my_room_action.c_inc aMR_SetOpenFtrDemoData sets demo_status, WAIT_MSG and
demo_ftrID. aMR_JudgeDemoStart gates wardrobes/closets on front contact; drawers
have a separate aMR_CheckHikidashi obstruction path. These callers were inspected
for this proposal. The following lifecycle and constructor audit is still pending:

- Trace demo_status/message completion through ac_my_room_move.c_inc and each
  selected furniture constructor/move/draw callback; establish exact clips,
  timing, sounds and closure conditions.
- Read player open_furniture/wait_open_furniture/close_furniture states and
  relevant existing logic translations before choosing the adapter.
- Audit app/src/furniture.rs profile_skeleton and current fixed-pose joint mesh
  construction before introducing animated furniture roots. Reuse existing
  runtime cKF decoding; keep static furniture, stacks and movement working.
- Integrate exclusive focus with player_room storage/dialogue/pockets ownership;
  clear transient state on F9, scene changes and missing-disc startup.

## Evidence and acceptance

- [x] Claim this task before implementation; audit source/callers/translations.
- [x] Implement the bounded supported families and document limitations.
- [x] Workspace check, warnings-denied lint, format and appropriate builds,
  one Cargo process at a time. No tests added/run without Philip's request.
- [x] Disposable Temp previews with synthetic saves and explicit --disc: open,
  store/retrieve, cancel, full pockets/storage, repeated interactions, pause,
  F5/F9/restart, stacked furniture and no-disc numeric preservation.
- [ ] Philip's actual playtest acceptance, separately from automated evidence.

Preserve actual save/settings/.gitignore. No extracted or derived disc assets in
Git. Exact dust, long-piece retail pivots, loose diary, broader per-type behavior,
M4 and module migration remain separate/open.

## Implementation and source audit, 2026-10-17

app/src/furniture/storage.rs retains cKF joints for 19 original common door
constructors, reusing character::skinned_part_mesh and identity bind poses.
Original joint display lists, textures, skeletons and clips load from the disc.
Other furniture retains its existing renderer. Audited common constructors all
initialize standard_stop at speed0 and draw with null callbacks. Source endpoints:

- 10: sum_blue_cab01.
- 12: nog_tri_chest02, sum_asi_chest03, sum_bla_chest03, sum_cont_chest02,
  sum_fruitchest03, sum_gre_chest02, sum_liccachest, sum_log_chest02,
  sum_ratan_chest02, sum_whi_chest02, sum_x_chest01, yaz_turkey_closet.
- 16: kob_locker1, kon_snowbox, nog_tri_chest03, sum_blue_bureau01,
  sum_clchest03, sum_hal_chest02.

Original ac_my_room_action front-contact demo gates, ac_my_room_move common
open/close and action->player mapping, msg_ctrl close-on-submenu/notice/choice
completion, and player hold/open/wait/close states were inspected. Closet maps
to kagu_*_d1; wardrobe to kagu_*_k1; drawers h1 are separate. Native player clips
run0.5 frames per60Hz tick, with existing five-frame pose morph. Common furniture
case3/7 calls play twice at speed0.5 (initial moving tick once), so the adapter
samples that pace rather than treating speed0.5 as the complete door rate.
NA_SE_FTR_DOOR_OPEN0x42C starts opening; CLOSE0x42D fires once when shut.
Runtime disc symbol headers confirmed player frames and source table/profile IDs.
Existing logic and pinned Rustimal furniture contain demo_status metadata but no
equivalent open/close renderer or lifecycle adapter to copy.

app/src/player_room/storage.rs owns temporary opening/waiting/closing state,
with front-facing interaction required for supported models. The existing item
transaction logic is retained; doors wait for pockets' closing slide before
closing, and notices keep them open until acknowledged. Player locomotion and
competing actions are held through the demo. P pauses it; F5/pause Save snapshots
numeric state; F9 and scene changes clear it. Door poses reset to frame1 on load
and are never saved as persistent switches. Missing player clips retain immediate
storage access. Drawers, special constructors, retail dialogue scripts and exact
player root motion/contact snapping remain separate; this is not every retail
storage behavior or every model's visual acceptance.

## Automated evidence (separate from Philip acceptance)

Workspace check, all-target clippy -D warnings with -j1, and format pass. Initial
parallel lint/build attempts hit Windows paging-file exhaustion (os1455); single
job lint passed, and unloading the model used by our timed-out local Ollama
request let the capture build pass. Ollama supplied no review evidence. Final
workspace check, all-target lint -D warnings, format, capture build and normal
workspace build pass, all Cargo processes sequential. No tests added/run.

Disposable Temp/bevy-crossing-furniture-storage-review checks so far:
- Closet4096 at cell34: closed/open/held/closing poses inspected; an intermediate
  paused opening screenshot shows the swinging door. Put in10240 saved pockets
  [0,8960,9216,0] and owner layers [4096,10240,0,0]; retrieval restored pockets
  [10240,8960,9216,0] and [4096,0,0,0].
- Pause-menu Save during opening followed by F9 returned to town without modal
  or presentation state. Fresh entry restored the closed model.
- Full storage and full pockets displayed the correct notices, held the open
  door until acknowledged, then closed. F5 retained all15 pocket items and owner
  layers [4096,10240,8960,9216].
- Put-in pockets cancellation and Take-list cancellation both closed the door;
  F5 retained pockets [10240,8960,9216,0] and [4096,10240,0,0].
- One early capacity-check attempt entered the mailbox instead of the house;
  it was excluded, then repeated after confirming house entry in log/screenshot.

- Final capture binary/fresh process: rotated blue wardrobe4145 at cell34 loaded
  closed with stored10240; back-side E refused without opening dialogue, front E
  opened its original doors and kagu_*_k1 player flow. Cancel shut it. G+D pulled
  it to cell35, preserving [4145,10240,0,0] and clearing cell34. Reopened at its
  new cell; retrieval added10240 once and saved [4145,0,0,0].
- F9 restored tape7768 on box12536 at cell17 alongside the closed wardrobe;
  screenshot inspected. Backspace recovered tape7768 and preserved box12536,
  secondary cell17 empty and wardrobe cell35 unchanged.
- Explicit missing-disc startup in Temp/bevy-crossing-furniture-storage-nodisc
  remained playable in town. F5/F9/F5 preserved all home_furniture layers,
  switches and pockets exactly. Placeholder-house entry was not rechecked.

All own preview processes closed. Screenshots, logs and synthetic saves remain
in Temp. Actual save current-turn
baseline DD89745CCFEDDEAEA619A1C931FEE5233EC533C1860E3B5A910A00B8121E5721
is unchanged at final hash check. Settings and unrelated .gitignore preserved.
Philip's storage-animation playtest acceptance remains pending.
