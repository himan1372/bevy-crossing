# Codex handoff - 2026-11-07

## Codex session - 2026-11-08: task16 native music sync (latest)

Philip selected16. Claude's clip-only version had omitted music; Codex completed actual BGM27/shrine-block arbitration and the native sequenceDA u16 counter (tempo ticks, start/opcodeC5 reset). AerobicsClock reads the current BGM handle. The routine now uses logic::taisou native ratio/delay/failure/tired kernels and NpcClip.sync_progress pose seeking, validates failure ownership and resynchronizes skipped updates. Interior/intro/fanfare precedence remains; F9 restarts active aerobics music. No-audio/no-disc participants stand neutral. One reused subagent owned routine/character integration; source review and parent runtime checks completed.

Check/Clippy/fmt and capture/default/release pass (release9m40s). Synthetic Temp/bevy-aerobics-20261108 on BRP25744:563 snapshots/all ten measures/9->0, natural slips and three Tortimer tired recoveries; pause, F9, shrine-block exit/return and event end pass. Sports-fair actual clock/five participants and no-disc neutral/numeric retention pass. Screenshots inspected; music bus peak around.50 with no nonfinite/limiting. Own previews closed; actual save hash unchanged. See `.claude-agents/completed/aerobics-music-sync.md`. Notice sweat/player exercise controls, exhaustive dialogue regression and Philip visual/listening acceptance remain open. No tests requested/run; M4 remains open.

## Codex session - 2026-11-08: task11 radio-effect (latest)

Philip selected task11 and resumed it; implementation and bounded verification are complete. The aerobics radio emits original effect32 KONPU notes every36 ticks (first tick37), with native72-tick velocity/decay/stretch/colour/fade and all three disc note models. It reuses logic::radio and Qrand, pools nine visual entities, freezes on pause and clears on F9, range and radio/event/scene removal. Native arg0=1 notes are silent; exercise BGM/synchronization remains task16 (another Claude agent has claimed it). No save-format change, dependency or extracted asset.

Workspace check/Clippy/fmt and default/capture/release pass; release8m37s. Shared worktree builds overwrote the common debug executable: ordinary Cargo cache hits could yield stale output. Forced cargo rustc metadata rebuilt the capture/release executable and its radio-note code was verified before QA. Synthetic Temp/bevy-radio-20261107 on BRP25743: native screenshots inspected; movement/pool reuse, pause, F9 clear/re-emission, neighbor-acre range,07:00 event end and no-disc fallback/possessions pass. Own previews closed; actual save SHA256 remains0495927D27B005E9FD0A4AED424EAAD883F56BBD8A9C4F5910167C1F9BBB1BD9. No tests requested/run. See `.claude-agents/completed/radio-effect.md` for exact limits; Philip visual acceptance and M4 remain open.

## Codex session - 2026-11-07: Next30 tasks 5 and 6 (latest)

Philip explicitly selected **island-player-cottage-room** and **island-naming**. Do not ask him to select these again. Implementation and bounded verification are complete; see `.claude-agents/completed/island-cottage-naming.md`. Check/Clippy/fmt, capture/default/release builds pass (release9m10s; unrelated tree_palette warning). Cottage editing/exit/reentry, naming No/empty/accept/repeat, F9, upgraded-house isolation, fresh native and no-disc numeric retention pass. Owned previews are closed. Philip acceptance remains pending.

The cottage now swaps an independent numeric room into the editable player-room renderer: native L room, floor/wall13, four aloha mannequins26/27, no house stairs. Furniture, owner-cell storage, switches, music and diary are saved under optional `island_cottage`; mainland floors retain their own records. Cached cottage turnips join the existing spoil pass. Accepted south-facing exterior doors are preserved.

Kapp'n first Yes uses3079, the original shi_win island name window (8 characters,80px), then30BE. Accepted `identity.island_name` survives save/F9; repeat trips use3076. Fixed messages select the player's gender; song IDs retain their existing selection. F9 clears owned naming/voyage state and restores the boat endpoint from the loaded exterior position.

QA uses synthetic Temp/bevy-cottage-naming-20261107 on **BRP25742** (`BRP_EXTRAS_PORT`), because another preview owns15702/15703. Never stop the other preview. Its actual save is untouched (SHA2560495927D27B005E9FD0A4AED424EAAD883F56BBD8A9C4F5910167C1F9BBB1BD9). Other Claude tasks1-3 have unrelated work in the shared tree; stage only our files. No tests requested or run. M4 and Philip acceptance stay open.

## Claude session - 2026-11-05..07 (NEWEST; read first, it supersedes the 2026-11-02/03 section below)

Identity `claude`, `main`. Uncommitted and not to be committed: `.gitignore`, `savegame.json`. The real save was never used: QA ran on Temp
copies of the capture exe (`Temp/bevy-cottage-20261106` and the older `bevy-*` folders; `qa.py` and `talk.py` helpers). `launch.ps1 -DayOffset N -Hour H`
sets the date and hour offsets (from 2026-11-06: 23 Sep 2026 = -44, 4 Jul = -125, 3 Aug = -97, 26 Nov = +20, 6 Apr 2027 = +151).

Done since the 2026-11-03 handoff (about 60 commits; each has a note in `.claude-agents/completed/` and a line in `comms.txt`):
- Migration slices: `game_math` and `calendar`, the event actors (`totakeke`, `gulliver`, `fireworks`, `harvest`, `halloween`, ...), 38 more event and
  minigame modules, and the event scheduler kernel (`logic/src/event_schedule.rs`, 134 rows) feeding `TodayEvents`.
- Event characters in the app: K.K. Slider, Gulliver, Katrina, Halloween trick-or-treat (costumes, Jack, Spooky and his spirits), Wendell, Saharah, Gracie
  (and her car), Crazy Redd (and his tent), Jingle, Franklin, the shrine events (Tortimer's commemorative items, groundhog and audience, hatsumode
  procession, omikuji), the fishing tourney, the sports fair (Tortimer, aerobics, foot race, ball toss, tug of war), fireworks, cherry viewing, harvest
  moon, countdown, meteor shower, harvest festival, the generic event room (kamakura) and `event_body.rs` (bodies, strides, idle clips).
- The island: Kapp'n's boat, the lighthouse (room, switch, lamp, Tortimer's present), the two cottages (turned to face the camera, a deliberate
  departure from the original), the resident with the island chat and her furniture trade and room (verified in game: best piece, 1x1, 1x2, shirt, save
  and restart). The shared quest system no longer offers her villager errands.
- Props from the disc (`event_body::Props`, vertex-segment support in `disc_gfx`): ball-toss baskets and balls, the tug rope, the fireworks stall, the
  cherry-viewing mats, the harvest tables, the aerobics radio. The moon-viewing table is NOT built: the decomp gives no placement (see
  `completed/event-props-moon-radio.md`).

Traps learned: a copied QA save with `first_job` set greets before any island trade check (clear `identity.events` and `first_job`); a villager standing
at her door turns E into a door entry; teleporting while `HouseState::Inside` makes later door tests fail silently (leave through the room's doorway
first); build the capture exe with `--features remote,capture` or BRP stays closed; copy the exe only after the old process has exited.

### FIRST THING TO DO (Philip's instruction, 2026-11-07)
Read the **Next 30** plan in `.claude-agents/ROADMAP.md` (section "Next 30 (planned 2026-11-07)"), give Philip a short status, and ask which numbers to
start with. Items 19-21 wait on Philip himself.

## Claude session - 2026-11-02/03 (older; superseded by the section above)

Identity `claude`, `main`, pushed (last code commit `209c4304`). Uncommitted and not to be committed: `.gitignore`, `savegame.json`. Real save never used for automation (Temp copies of the capture exe,
synthetic saves; `qa.py` helpers in `Temp/bevy-*-20261102`). The inventory-mail-polish work noted below as incomplete was finished by this session (commits `898244b9`..`3f77754c`, see its completed note).

Done, each with a note in `.claude-agents/completed/`:
- **Notice board end-of-edit window** (`notice-board-edit-end-check`): the board's own Post it / Keep writing / Never mind list is replaced by the shared `EditEndCheck::with_answers(3)`
  (`mEE_TYPE_NOTICE`), drawn by `notice_board/render.rs::draw_check`. Driven live: Rewrite, Yes (one notice posted), Throw it out.
- **Mail QA** (`mail-qa-remaining`, `mail-qa-written`): paging past page 1, right-edge tag flip, the three tag arrow variants, Museum / villager colours, Rewrite cancel and commit, full-mailbox Write
  refusal all passed with no code change. **Rewrite now keeps the letter's own addressee** (`mail.rs`; the original opens the board on the letter, only Write uses the address book). Hint line still
  says "PgUp/PgDn: change" for a one-entry list (cosmetic).
- **Catalog circle** (`nook-tickets-catalog`, accepted by Philip): `logic/src/catalog_draw.rs` is the original's `furniture_draw_data` (41) + `mCL_furniture_list` (601); `catalog_screen.rs` draws listed pieces
  at table scale x profile scale and looks at 25 units above the model origin (`change_view_proc(330, 25, ..)`; every piece used to sit too high). Unlisted pieces keep the bounds fit. Open, not scheduled:
  the original's viewport is a 256 px square of a 480 px high screen (larger than the circle); a bigger port viewport made the bed spill over the list, so it is not applied.
- **Migration slice 1** (`migrate-event-day`): `logic/src/event_day.rs` copied from rustimal-crossing (`pc_ed_*` ABI removed), 7 own tests, not called by the app yet. 94 Rustimal modules are still
  source-only (`rust/src` vs `logic/src`): `game_math` + `game_math_tables` are needed by `calendar.rs`; then the event actors (`totakeke`, `gulliver`, `fireworks`, `harvest`, `halloween`, ...). Reuse the
  existing `logic/` modules first (memory rule) and check each against the original C.
- Closed by Philip's acceptance: `retail-mail-ui`, `net-render-fidelity`, `nook-tickets-catalog`. Still open and waiting on Philip: `retail-pockets-ui`, `retail-shop-ui`, the four audio notes. `retire-town-gen`
  is NOT wanted (Philip: town_gen does not need retiring); its backlog note has an audit I wrote, ignore it.

QA traps learned: the Temp QA teleport into the shop room does not enter the shop (`ShopLocation::Inside`), so Nook will neither talk nor follow; walk in through the door (KeyD 0.5 s, KeyW 1.2 s from the plaza
start), wait about 4 s for the greeting, then use the teleport helper. Pockets reopen with the cursor where it was left; navigate by screenshot. `Escape` in the pockets grid closes the whole panel.

### FIRST THING TO DO (Philip's instruction)
In your first reply, after catching up, give Philip a list of **3-4 things we should do next**, each with one line on why, then stop and wait for him to choose. Candidates to weigh: migration slice 2
(`game_math`, then `calendar.rs`) and the event actors; closing the open acceptance notes; the catalog viewport size against retail; wiring `event_day` into the app's holiday handling.

## Budget wrap-up - 2026-11-02 (older)

Philip has under3% weekly usage left and requested an immediate wrap-up/handoff.
Stop further builds/runtime work this session. Main/origin main are **915ede8c**.
The current **inventory-mail-polish** implementation is UNCOMMITTED and INCOMPLETE;
preserve it and resume its existing plan rather than starting another feature.
Philip already selected this work ("inve/mail"); no new scope-selection question
is needed when he resumes. This section supersedes stale instructions below.

### Completed and accepted this session

915ede8c restores native fish/bug inventory pages, saved actual-catch records,
wallet deposit/withdraw controls and the native42-tick counter animation, plus
idle-clock intro/modal/pause/widescreen adapters. Check/clippy/fmt and capture/
default/release passed; release5m36s. Real Cricket catch, exact transactions and
refusals, animated counter intermediates, F9, month rollover, intro and no-disc
were checked. All owned previews closed. Evidence: Temp/bevy-hud-20261101.
Task: .claude-agents/completed/retail-hud-followup.md. Philip said **"they work"**
before selecting inventory/mail next; record that acceptance without closing M4.
Custom design/editor is still absent (he asked whether it worked; no build request).
Old catch history was not recorded. Full inventory page-exchange motion and native
held withdrawal-bag choreography remain separate presentation work.

### Current partial work and exact resume point

Task: .claude-agents/backlog/inventory-mail-polish.md.
Spec/plan: docs/superpowers/specs/2026-11-01-inventory-mail-design.md and
docs/superpowers/plans/2026-11-01-inventory-mail.md.

- Parent changes: pockets_disc.rs; tags.rs; tags/layout.rs; tags/presentation.rs;
  NEW tags/label.rs. Native sen_qitem frame/shadow/six pointers, three-line
  Letter to / Delivery for labels, colored names,0.75 typography, native special
  parent action placement. Labels use existing letter/active delivery/first-job
  metadata; no save schema or item-condition changes. Ordinary tag APIs retained.
- One reused subagent changed ONLY mail_disc.rs: native menu Slide for paper and
  keyboard, notice_board::Roll for body cursor following, actual stationery for
  Write/Rewrite, retained close snapshot, ResetAudio cleanup. Static paper moves
  as a whole in native m_board_ovl; there is no deformation rig. Writing ink at209
  sits below editor210+, allowing all six lines instead of old five-line crop.
- Parent integration in mail.rs/input.rs: clear input events and hold actions
  while MailPresentation::busy(); retain Mail focus while closing().
- Last verification: cargo fmt --all then **cargo check --workspace -j1 PASSED**
  (10.45s). Initial Rust visibility errors were fixed with pub(in crate::pockets_disc).
  No clippy, capture/default/release build or runtime QA for this partial task.
  No tests added/run. The final requested subagent review failed from usage limit;
  do NOT treat it as a clean review. No preview or Cargo process remains running.

Resume by reviewing the diff and native source, then finishing gates and Temp QA.
Specific review concern: delivery_label currently requires first-job pockets_idx
to equal the selected slot; existing first_job_talk searches all slots, so moving
the quest item could lose its special label. Check ownership semantics before
changing it. Also verify special arrow variants/action bounds, rich name colors,
paper/editor timing/depth, rapid close/reopen and input release/no leaked typing.
Drive read, Write/cancel, Rewrite/cancel/commit, long legacy paging, presents and
refusals with complete numeric comparisons. F4 still uses the existing direct
commit; native edit-end confirmation is a separate gap, not implemented here.

Original C: m_tag_ovl.c1360-1450/1696-2000/2280-2420/8260-8830,
m_quest.c449+, m_board_ovl.c and existing notice_board Roll/Slide integration.
Older completed retail-tag-art/layout-motion and letters-retail-grid notes are
relevant; accent/voices already completed later and must not be rebuilt.

Task ownership is released at this handoff, not completed. No partial gameplay
commit/push. Finish review/QA/builds, update roadmaps/comms, complete claim and
publish explicit task files after resuming. Preserve unrelated .gitignore and
the existing older handoff edits. Real save/settings were never used for automation;
actual save SHA256 remains
0495927D27B005E9FD0A4AED424EAAD883F56BBD8A9C4F5910167C1F9BBB1BD9.

Current next steps:
1. Finish the existing uncommitted inventory/mail tags and paper-motion task.
2. Philip play-checks special tags, letter reading/writing/Rewrite and transitions.
3. Reconcile remaining M4 shop/audio/render/UI polish against completed notes;
   custom designs, title/rename and held-net appearance remain separate/deferred.

## Philip's handoff request - 2026-11-01 (read first)

Philip requested a fast handoff update. The next agent MUST END its first
catch-up reply with the numbered next steps at the bottom of this file, then
wait for Philip to select work. The previous three intro priorities are all
implemented; do not propose them again as missing. Historical entries below
can be stale: newer completed task notes take precedence. Philip explicitly
said this handoff-only edit does not need a task claim or coordination work.

Published main: **72adeb3c** (retail pause/save), following **9f7f5c63** (intro
reactions), **53c42320** (intro cameras and smoke hitch fix), **407d40a6** (train
engineer/smoke/steam/sounds). All implementation tasks are completed and pushed.
Final check/clippy/fmt and capture/default/release builds passed; no tests added
or run. Philip confirmed the prior corrected build "was smooth" before selecting
reactions. New reaction/pause visual/audio acceptance and broader M4 remain open.

### Latest pause/save work

P opens the native30CD save question and select1AE/1AF Save & Quit / Save &
Continue choices, rendered with runtime-disc message/choice/font/cursor assets.
P/Escape resumes; F1 opens PC Settings and confirmed Quit without saving.
Choices sit above the question for readability. Settings supports physical
Enter/NumpadEnter/Escape/Backspace, P as text and preservation of other keys.
The world stays paused through the actual synchronous save result; only success
can reach the saved/quit acknowledgement. Failed saves offer retry/back.
The writer uses a unique exclusive same-directory temporary, write/sync/close,
then rename-replace; failures preserve the old save and clean their temporary.
Simultaneous P+save/retry cannot accidentally resume the world.

Native and no-disc resume/cancel/settings/save/F9/quit paths passed with a
synthetic town: Bells543, loan17400, house2, quest item slot7/condition32768,
room and identity retained. Failed settings writes, destination-directory and
locked-original-save failures were driven; old bytes stayed intact, retry
succeeded,0 temporary files remained, and both explicit exit paths exited.
Release5m20s passed. Task: .claude-agents/completed/retail-pause-ui.md.
Plan: docs/superpowers/plans/2026-11-01-retail-pause-save.md.
This adapts the save conversation to PC pause; it does not implement the full
gyroid/house save scene or the deferred title screen.

### Intro and performance continuation

Train engineer, native smoke/steam and train audio are implemented. Cameras use
native arrival release/Nook framing with temporary foreground culling. The smoke
hitch was full-town material invalidation on every puff birth/death; environment
now initializes only Added/Loaded materials and refreshes existing ones on time/
scene changes. Preserve this fix; never key lighting refresh on material count.
Rover/Nook now use16 native reaction orders,27 disc clips, eye/mouth metadata,
7 feel effects plus bulb glow/sounds. Scripted sitting/phone/movement/handovers
retain ownership. Rover's final sit releases to seated idle for its last reactions.
Full train and Nook refusal/acceptance/repeat/payment-cancel/Give flow passed;
home2/loan17400/starting bag removed. All15 pooled effect meshes hide afterward;
F9 and missing-disc numeric retention passed. Bounded final smoke/reaction/walking
peaks18.24/16.61/15.81ms; launches vary, so do not claim exact FPS parity/speedup.
Task notes: completed/intro-train-effects.md, completed/intro-camera-framing.md,
completed/intro-dialogue-reactions.md under .claude-agents/.

All owned previews are closed; no automated input is pending. Evidence remains
in Temp/bevy-pause-save-20261101 and Temp/bevy-intro-reactions-20261101.
Actual save baseline/final SHA256:
0495927D27B005E9FD0A4AED424EAAD883F56BBD8A9C4F5910167C1F9BBB1BD9.
Preserve unrelated .gitignore and untracked savegame.json. Actual save/settings
were never used for automation. No tests unless Philip explicitly requests them;
use sequential Cargo gates/builds only when code changes require them.

## NPC speed correction - 2026-10-31 (newest)

Philip played the Nook sequence and said it works, but Nook and the other NPCs
move too fast. Task npc-retail-speed is complete; all gates/builds passed.
Three source files: npc.rs, intro_house.rs, character.rs. Source aNPC_spd_data is
walk1/run3; Actor_position_move advances
0.5 per60-Hz tick. With40 units/cell, native caps are0.75/2.25 world units/s.
The guide/follower share the resident run cap; generic NPC/keeper run playback
uses native0.5 frames/60-Hz tick and resident run threshold follows the new caps.
Existing steering/schedules/waits and player input speed are outside this fix.

check/clippy/fmt/capture/default passed; release passed (5m19s). Isolated Nook median
2.263; six villagers sampled45s, one-second walk/run medians0.74997/2.24929.
Nook reached the houses and run-pose captures were inspected. No tests. Temporary
QA folder: Temp/bevy-npc-retail-speed-20261031; latest PID/log files identify the
owned preview. A fresh interactive station preview is open for Philip; no more
automated input is pending. Real save unchanged. Completed task note is
.claude-agents/completed/npc-retail-speed.md; commit title is
Match Nook and villager movement to native NPC speeds.
Do not touch unrelated .gitignore/savegame.json. Speed acceptance,
remaining intro staging/emotions/engineer/smoke/sounds and M4 remain open.

## Codex continuation - 2026-10-31 (newest)

Philip resumed after the usage wrap-up. Nook welcome/house-choice implementation
and runtime verification are complete; all builds passed. Task note:
.claude-agents/completed/intro-nook-house-choice.md; plan:
docs/superpowers/plans/2026-10-30-intro-nook-house-choice.md.

Station exit now reaches Nook's native welcome/tour, inspection of any four homes,
original choices/explanation, exact quest Bell-bag Give/Quit, handover, nearer-end
departure and existing first job. Selected HouseSave.home_slot defaults to0 for
old saves and drives door, roof, mailbox/flag, gyroid, postman delivery and statue.
Played payment uses complete_intro, avoiding Skip's bag allocation. Completed
identities are guarded before missing-data fallback. Rover's mandatory disc
system is now gated, fixing the no-disc panic found during QA.

Verified: native normal arrival/stairs to Nook; home1 refusal (unchanged ownership,
loan and bag), home2 acceptance, explanation repeat, acre warning, Escape/Quit
payment cancellation, Give/handover and actual corrected south departure without
actor edits. Final save: home2, loan17400, bag removed, intro_done/first_job true.
F9 and process restart preserve selected door/mailbox/roof, numeric inventory,
loan and full room; entering the shop starts existing first-job greeting.
Completed-save INTRO=play with missing disc preserves house3, loan12345, wallet543,
pockets/conditions/room and no job reset. Fresh missing-disc intro falls back to
playable world/save. Source reviewer confirmed ownership/payment/departure fixes.

check/clippy(-D warnings)/fmt and capture/default passed after final Rover guard;
release passed (4m29s). No tests added/run. All previews used synthetic Temp saves;
all owned previews are closed. Temp/bevy-intro-houses-20261030 retains evidence.
Earlier blocked old-build departure was moved beyond the board only to inspect
integration; the final fresh latest-binary run needed no guide transform edits.

Keep M4, Philip's acceptance and overall intro fidelity open. Train engineer,
smoke/sounds, camera staging and remaining native emotion commands are follow-ups;
intro remains development-gated as before (INTRO=play, ARRIVE/HOUSES QA flags).
Do not silently start another gameplay slice. Next concrete step: Philip plays
from the station through house choice/payment and the first shop visit.

Preserve unrelated .gitignore and real savegame.json. Real save SHA256 remains
C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
Previous pushed gameplay9542eb3b fixed Porter's collision ("def better now").
The Nook slice commit is titled "Continue the arrival into Nook house selection".

## Claude continuation - 2026-10-30, fifth session: the new-game intro (newest, read this first)

Identity `claude`, `main`, pushed (last code commit `4edda5e0`). Only `.gitignore` and `savegame.json` stay uncommitted (never commit them). Philip picked the intro demo; it is built in slices, each with a note in `.claude-agents/completed/`:
- **Gate and train scene** (`intro-new-game-gate`, `intro-train-scene`, `intro-train-script`, `intro-train-look`, `intro-time-eyes`, `intro-slice2-finish`): new-game gate and intro state, `finish_intro`; Rover's full train script, the name window and
  questions, gender / face, window scenery, clock, the old woman, train music.
- **Slice 3, the arrival in town** (`intro-station`, `intro-train-arrival`, `intro-porter-walk`, `intro-train-orient-getoff`): the station building (`disc_town::station_type`), the train coming in (`logic/src/train_control.rs`,
  `app/src/intro_arrival.rs`), Porter's force talk 0x07DD, the player stepping off with `outtrain1` and walking out of the station; the latest commits fix the train's axis, platform alignment and the capsule-centre height.
- Not built (see each note's "Not done"): the engineer, smoke / steam, train sounds, camera mode 9 easing, Porter's idle clip, Nook's welcome, house choice (`finish_intro` still runs straight after the walk).
- Philip is still checking the arrival visually (several "fix" commits came from his feedback); re-drive it in a Temp capture copy before touching it.

Next: Nook's welcome / house choice after the walk, then the other intro leftovers above. Philip chooses.

## Claude continuation - 2026-10-29, fourth session

Identity `claude`, `main`, pushed (`067cbab1`; this note is the only later change). Philip picked three gaps from the third session's list:
- **Notice board cursor roll and keyboard slide-in: done** (`completed/notice-board-roll-keyboard.md`): `logic/src/notice_board.rs::Roll` (`mNT_roll_control` / `_control2`), the page group is lifted by the roll,
  every line is drawn, the keyboard is its own group sliding from the bottom (`mSM_MOVE_IN_BOTTOM`, the menu slide negated) and out when the edit ends. Driven live; the port's own help rows at the screen top
  still overlap the header once the page has rolled up.
- **House demolition animation: no source.** `ac_house.c` only has door / open requests (`request_type`); a leaving villager's house is cleared as item data in `m_all_grow_ovl.c`. Nothing to build.
- **Per-villager memory counts: nothing missing for what the port models.** `mNpc_ForceRemove` compares `GetAnimalMemoryNum` / `LetterNum` / `LandKindNum`, which count one memory per player and per town; with one player and
  no foreign towns they are 0 or 1, exactly what `roster::force_remove` passes (`met`, `met && letter`, `met`). Larger counts need several players or visits to other towns (not built). Friendship
  (`Anmmem_c.friendship`) is a different field and is not part of this gap.
Next candidates are unchanged: the intro demo (largest), `remove_animal_idx` transfers, "take me with you" talks. Philip chooses.

## Claude continuation - 2026-10-28/29, third session (newest, read this first)

Identity `claude`, branch `main`, everything below is committed and pushed (last commit `6fc299ae`). Only `.gitignore` and `savegame.json` are untracked/modified and must NOT be committed
(real save hash `C15154C0...`, unchanged all session). Workflow as in the second session below (repo skills, backlog note + `task-claimer.sh claim`, `cargo -j 1`, QA only in a Temp copy of the capture
exe, fmt / `clippy --workspace --all-targets -D warnings`, explicit-file commits with the `Co-Authored-By: Claude Sonnet 5.5` line). Philip chose each task from the candidate list; each has
`.claude-agents/completed/<slug>.md` with source references, evidence and what was not done. The whole `cargo test -p bevy-crossing` run passes (76, 21 ignored).

Done, in order:
- **Notice board animations** (`notice-board-animations`): `logic/src/menu_slide.rs` (`mSM_move_Move`, shared with the map) and `notice_board::Motion` (page turn, to-write, to-read); `notice_board/render.rs` builds the arriving and leaving
  pages and the key hints as moving groups. Not built: the cursor-following roll, the keyboard's own slide-in, the per-hop page sound.
- **Museum room doors fade** (`museum-room-door-fade`): `SceneWipe::hold` / `release` for changes that are no state change. Door sounds already start with the fade (same frame), so unchanged.
- **Villager house door** driven with the fade (`villager-house-door`); `house.rs` logs each villager's door position.
- **Job redirects** (`job-moved-away-redirects`): `Job::moved_away` etc. in `logic/src/first_job.rs`; `guide.rs` takes the undelivered item back (player -> Nook hand-over), restarts the job with a new pick, turns a missing axe
  recipient into the second letter, handles a letter given to the wrong person (progress 3, `Post.letters_given`) and the "more paper" question (0x0807 -> 0x080A). `GameText::talk_until` renders a redirect message alone.
- **Roster slots and ForceRemove** (`roster-slots-force-remove`): `TownRoster.slots` (saved `villager_slots`) so a villager can leave without moving the others, `remove_day` (`last_remove_day`), `logic/src/force_remove.rs`
  (the 15-resident / 10-day rule, the goodbye choice, the goodbye letter number), goodbye letter via `post::goodbye_letters`. Homes are placed per occupied slot (`disc_town::place_homes_on_reserved`).
- **"I'm moving" talk** (`moving-talk`, `moving-cancel-branch`): `TownRoster.leaver` (`remove_animal_idx`, saved `remove_slot`, chosen at start, five or fewer villagers keep everyone); `talk::choose` makes that villager say
  `REMOVE_YES[looks] + RANDOM(3)` in the event branch; the quest order (index 1, value 2) in the "no/won't help" answers is read from a new quest queue in `DemoOrders` (`take_quest`) and `roster::cancel_moving` picks
  someone else. Seen live: Anchovy 0x1cd5, Bluebear 0x1504 / 0x1505 / 0x1506, and the cancel (`talked out of moving`).
- **QA aid** `BEVY_CROSSING_VILLAGER_MOOD=normal|happy` (`villager-mood-debug`): fixes the mood roll (happy villagers use the other talk tree).
- Roster test `a_new_villager_moves_in_once_a_day` fixed (it asked for a starter only; `can_move_in` allows `GROW_MOVE_IN` too).

QA traps learned (add to the earlier lists):
- A Windows-checked-out file may be CRLF: patch scripts must match the file's line endings (a replace that silently misses was the old trap; assert every replace).
- A pockets / letter / quest-offer panel left open blocks every talk (Escape x4, N declines an offer). A villager inside a house cannot be talked to. Teleporting beside a villager and tapping KeyD can walk the player into their own
  house door: stand 0.9 south and tap KeyW instead.
- Museum doors: read the door position from the `museum door at` log line (it depends on the town seed); the museum is open 09-18, launch with `-Hour 14`.
- F5 saves only in world focus and not while a dialogue / quest panel is up.
- Run long QA loops in the background and poll the output file; do not stop a process whose PID you did not verify (a stray `python` may be left from the moving-talk runs).

Known gaps now: the intro demo (arrival, name entry, house choice) is the largest open item; the villagers' "take me with you" talks (0x11FE, 0x11FD) need other-town trips; `remove_animal_idx` transfers (card / island) are not
built; the demolition animation of a removed villager's house; per-villager memory counts beyond met / letter; the notice board's cursor roll and keyboard slide; the lighthouse and island, visitor / foreign-town paths, parent
retail/M4 and Philip's acceptance stay open. Philip chooses the next task.

## Claude continuation - 2026-10-28, second session (read this first)

Identity `claude`, branch `main`, everything below is committed and pushed (last code commit: the circle wipes; `docs/codex-handoff.md` is this update). Only `.gitignore` and `savegame.json` are
untracked/modified and must NOT be committed (real save hash `C15154C0...`, unchanged all session). Workflow unchanged (repo skills, `task-claimer.sh claim <slug> claude` after writing
`.claude-agents/backlog/<slug>.md`, `cargo -j 1`, QA only in a Temp copy of the capture exe, gates fmt / `clippy --workspace --all-targets -D warnings` / capture build, explicit-file commits with the
`Co-Authored-By: Claude Sonnet 5.5` line, push). Every task below has `.claude-agents/completed/<slug>.md` with source references, evidence and what was not done.

Done, in order:
- **Town map overlay** (`map-overlay-retail`): `app/src/town_map.rs` draws the retail `m_map_ovl.c` from disc display lists (window `kan_win_model`, acre pictures `kan_tizu_model` with `l_map_texture` / `l_map_pal`
  in segments 8/9, label bubble, house marks, cursor glide and pulse, slide in/out); kernel `logic/src/town_map.rs` (own tests). Opened by X (needs `map_flag`) or by the action button facing a map
  signboard (field items 0x000C/D, `map-from-pause`: `data0 == 0`, colour list 0, magenta town name, X does not close it; the source has no pause-menu map). The plain text grid is the no-disc fallback.
- **Notice board overlay** (`notice-overlay-retail`): `notice_board/render.rs` uses `kei_win_model`, "entry NN" and date header, key display, page arrows, stick marker.
- **Display-list reader** (`app/src/disc_gfx.rs`): `run_display_lists_with_segment_lists` follows `G_DL` into runtime segments that game code points at lists (segment B colour list, A waku list).
- **Job gaps**: the end-of-job cheer window is green (`DialogueState.window_color`); the fanfare (sequence 232) loops until the message is read; hand-over of every job item (`shop_nook/handover.rs`
  `Item::Goods` from the disc's `mode_DL_table` / `vtx_DL_table`; `guide::hand_over_job_item` at message order (1,1,4); `DialogueState.held` = `mMsg_SET_LOCKCONTINUE`; `DemoOrders::take_raised`);
  the town map hand-over at the third job's end (order (1,9)). Drove bag, furniture (a leaf), paper, carpet, axe. Message order kinds in this port: NPC0 is kind 1.
- **Scene wipe** (`app/src/scene_wipe.rs`): black fade (out 30 frames, 10 held, in 30) on every `ShopLocation` / `HouseState` change; ordinary doors are caught late in the frame (`doors` in `Last`):
  the player and the pending states are put back, `InputFocus::Transition` freezes input, the picture fades out, then the change is applied. F9 and changes made while a fade runs pass through.
  The job's send-away does its own fade out (`guide::send_away`), then the town opens as the CIRCLE_LEFT wipe (`ef_wipe2_modelT`, its own perspective camera on render layer 2, texture window scrolled by
  rewriting U each frame, black bars beside the 4:3 picture on wide windows). `circle_next(right)` / `circle_out(right)` also build circle right and a closing circle; nothing in the source calls them.
  Driven: shop, museum, police, post office, player house (in and out).

QA recipes and traps learned (add to the earlier lists):
- Several scripts live in `%TEMP%/bevy-crossing-room-critters/higher-tiers-20261025` (`wipeqa.py`, `guideqa.py`, `introqa.py`, `tiers_qa.py`, `qa.py` ...) and `map-overlay-20261028` (door, wipe, hand-over and map drivers: `doors_qa.py`,
  `wipe4.py`, `map3.py`, `items_one.py`). Copy `qa.py`, `tiers_qa.py`, `preview_qa.py`, `sales_qa.py` together; they import each other.
- A new game's shirt comes from the savegame present at launch: write `player_cloth: 16` BEFORE launching or the guide sees the wrong shirt (job start/end talks go wrong).
- After an F9 reload later door tests can silently fail; relaunch between door tests. A door E-press needs a short key tap and a 0.6 s wait before E (turning arc); walking-trigger doors (museum, police, shop) need
  the walk itself. The player house door faces east: stand 0.2-0.4 east of `Player house door at` (stdout log) facing west.
- Screenshots saved in a burst are written asynchronously: frames can land out of order in a contact sheet.
- A system with `Query<&mut BackgroundColor>` twice needs `With`/`Without` filters or Bevy panics at runtime (cargo check does not catch it). Resources and states do not exist before the game state starts:
  take them as `Option<Res<..>>` in always-running systems. A `RENDER_WORLD`-only mesh cannot be modified in the main world after its first frame (keep a main-world copy).
- Do not pipe `launch.ps1` (hangs); start it with `(powershell -File launch.ps1 > launch.out 2>&1 &)` and poll BRP.

Known gaps now: the player-to-guide give-back hand-over and the second letter / second axe jobs (they belong to the "recipient moved away" redirects, which need a villager leaving mid-job); the item models in the
hand-overs are the disc's (uniform shirt texture and the map's own look not inspected); a villager's house door fade-out not driven; the museum's inner room doors still cut; the door sound plays before the fade;
the fade length and hold are the PC port's reading (30 / 10 frames); the mark-index bias `HOUSE_UT_Z_BIAS` and `tier_height` of the town map were not compared with retail; the notice board has no slide in/out or page-turn
animation and its paper does not roll while editing; the bridge picture (`pluss_bridge`) is never swapped (no built-bridge state); intro demo, visitor / foreign-town paths, lighthouse and island, parent retail/M4 and
Philip's acceptance stay open. Candidate next tasks: the intro demo (arrival, name entry, house choice) now that its job and wipes exist; the notice/map animations; the remaining doors. Philip chooses the next task.

## Claude continuation - 2026-10-28 (earlier the same day)

Identity `claude`, branch `main`, everything below is committed and pushed (last commit: the map and wipe, then a one-line note fix);
only `.gitignore` and `savegame.json` are untracked/modified and must NOT be committed. The workflow, gates and QA rules of the
2026-10-27 section still hold (read the repo skills, claim with `task-claimer.sh`, `cargo -j 1`, QA only in a Temp copy of the capture exe with a
synthetic save, commit explicit files with the `Co-Authored-By: Claude Sonnet 5.5` line). Each task has `.claude-agents/completed/<slug>.md`.

Done this session, in order:
- Shop start audit follow-ups, driven with no code change: sale-day welcome on tier 0 (`shop-start-welcome-tier-0`), the present hand-over frames (`shop-handover-frames`).
- Randomness: `logic/src/qrand.rs` is the source's `qrand/fqrand/RANDOM(n)`; the app's `GameRand` resource wraps it (`game_rand.rs`). The bargain sale hour and stock, grab bag, raffle and face blinks draw from `GameRand`;
  `town_gen`, the villager roster, the turnip week and the raffle lineup use per-seed `Qrand` streams so a reload gives the same result (`shop-bargain-rng-kernel`, `town-gen-qrand`, `gamerand-adopters`).
- HRATALK: `logic/src/personal_events.rs` (FIRSTJOB / HRAWAIT / HRATALK flags), saved in `PlayerIdentity.events`, daily renewal in `save.rs`, Nook's day-after greeting 0x082A first in the arrival order
  (`shop-hratalk`, `shop-hratalk-talk-end`).
- The whole new-game part-time job, built piece by piece and then driven in one run (`first-job-full-run`). Kernel: `logic/src/first_job.rs` (the eleven jobs, errand record, tables, finish rules, villager picker;
  eight own tests). App: `shop_nook/guide.rs` (Nook is the guide while `identity.events.first_job`), `first_job_talk.rs` (villagers' greeting and the delivery), `tortimer.rs`, `notice_board.rs`, `town_map.rs`, `job_end.rs`.
  Saved in `PlayerIdentity`: `events`, `events_day`, `first_job`, `arbeit_trophy`, `map_flag`; `Post.sent_to` and `Post.notices`.
  Chain as played: change clothes -> plant -> introductions (until every resident is met and Tortimer talked to) -> furniture -> letter -> open quest -> carpet -> axe -> notice -> the job ends
  (HRAWAIT, next day HRATALK). Restrictions while it runs: villagers forced outside, shop always open, no closing music (`part-time-job-core`), and no app quest offers except in the open-quest job (`quest.rs::job_blocks_quests`).
  Prerequisites built on the way: wearing a shirt by carrying it onto the player in the pockets (`wear-shirt`), demo-order codes kept as page markers and raised as the text types (`DemoOrders` in `msg_window.rs`),
  the fanfare override (`sound::Fanfare`), `Action::Map` (X) and `InputFocus::Notice` / `Map`.
  Details and exact message numbers: `completed/first-job-*.md`, `notice-board.md`, `job-map-and-wipe.md`.

How to start the job in a test: write a savegame with `identity.events = [true, false, false]` (and `identity.first_job` absent for the start) BEFORE launching: a new game's shirt and look come from the
savegame present at launch, F9 does not reset them. For a mid-job state write the `first_job` record (`job` is the index in `Job` order, `progress` 1 chosen / 2 running / 0 done).

QA traps learned (add to the earlier list):
- A system may not read and write the same message (B0002): use `ResMut<Messages<T>>` with a `MessageCursor`.
- To intercept a talk, order the system `.after(interact)` as well as in `QuestInputSet`, or the ordinary talk wins.
- `update_focus` is at Bevy's sixteen-parameter limit: bundle new modal flags into a tuple parameter.
- F5 only saves in world focus; a dialogue, a quest panel or a villager standing in front keeps it from saving. Teleport the player away before pressing Space to finish a dialogue, or Space starts a new talk.
- Do not pipe a long QA script into `tail` (the output is lost on a timeout); run it in the background (`nohup python -u ... > out`) and poll the file.
- Villagers wander: retry the teleport-and-E several times; one of them stands on a cliff acre.
- `python -B` and `rm -rf skills/gc-iso-reader/__pycache__` when importing the disc reader.

Known gaps (job): no hand-over visuals for the job items (they are given directly); the notice board and map use plain panels (retail overlays not built); the second letter / second axe jobs and the "recipient moved away"
redirects; the axe delivery's reply-letter display; the intro demo itself (the job is started by a save edit, Philip deferred the introduction); the fanfare length (14 s) and the 0x17B6 window colour; the scene wipe.
Other open items: visitor / foreign-town paths, the lighthouse and island, the other mWR warnings with no trigger, Nook's arm poses (the shop camera hides them), the pockets portrait not updating its shirt.
Candidate next tasks: the intro demo (arrival, name entry, house choice) now that its job exists; the retail map and notice overlays; drive the remaining paths listed above. Parent retail/M4 and Philip's acceptance stay open; Philip chooses the next task.

## Claude continuation - 2026-10-27 (earlier the same day)

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

Known gaps (the grid cursor pulse is done as the text-caret fade, `completed/grid-caret-pulse.md`; the source has no key pulse or editor rumble): escort/twin audited (`completed/shop-twin-escort-audit.md`: tiers 0-3 follow fixed; twins' upper floor built in `completed/shop-twins-upper-floor.md`; stairs goodbye done in `completed/shop-twins-finish.md`; forced greeting done in `completed/shop-forced-greeting.md`; start actions and Nook goodbye in `completed/shop-twins-start-nook-goodbye.md`; the door-facing trigger in `completed/shop-exit-door-trigger.md`); grab-bag capacity warning panel (done: `completed/grab-bag-warning-panel.md`; the other mWR warnings in `completed/warning-panel-conversions.md`, all driven in `completed/warning-panel-runtime.md`; the Yes/No cursor arrow fixed in `completed/warning-cursor-arrow.md`; non-twin start behaviour audited in `completed/shop-start-audit.md` and driven on tiers 1-3 in `completed/shop-start-tiers-1-3.md`); Nook
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
then tell Philip the next concrete step. The next agent MUST END its first
catch-up reply by listing these next steps, in this order; wait for selection:

1. **Finish the existing inventory/mail task:** review and verify the uncommitted
   special tags and paper/editor motion; follow the CURRENT budget handoff above.
2. **Philip play-checks inventory/mail:** special quest/letter labels, paper
   transitions, reading, writing, Rewrite and input ownership.
3. **Remaining M4 polish and acceptance:** reconcile shop/audio/render/UI gaps
   against completed notes before coding. HUD, collections and wallet controls
   are implemented and Philip accepted them; do not restart them as missing.

Held-net appearance, title screen and rename remain deferred by Philip. M4 is
still open; implementing individual slices does not imply blanket acceptance.
