# Higher shop tiers and renovation lifecycle

Philip selects priority3 after42020f3c. One subagent handles source tier layout/
render assets; root handles persisted renewal, hours, integration and QA.
Follow original C aSL_JudgeRenewShop/RenewShop, mSP hours, indoor/department actors.

- [x] Replace immediate upgrade with Town-only source scheduling, saved pending
  date/visitor gate, tomorrow closure and +2-day completion; calendar exclusions.
- [x] Original ordinary tier rooms, merchandise/service access and department
  stairs/floors; tier clocks, uniforms and exterior assets. No guessed layout.
- [x] Rebuild owned rooms on tier/floor/load changes, preserve live transactions,
  opening/closing and refusal; legacy saves default safely.
- [x] Source review, sequential check/clippy/fmt/capture/default/release, synthetic
  Temp scenarios and screenshots, real save hash only. No test suites.
- [x] Both roadmaps/parent/comms/task evidence, claim complete, commit/push.

Design: reuse existing native sales thresholds/date kernels. Persist only a
small optional renewal state in existing version1 JSON. Reuse original room/
field assets and existing movement/menu systems. Parent/M4 acceptance stays open.

Source/implementation:
- aSL_JudgeRenewShop schedules in town only, +2 calendar dates; tomorrow is
  closed, completion on/after due date when outside the shop. Reuses native
  ShopState kernels; removes every immediate upgrade call. Optional version1
  shop_progress stores pending/date/notice/visitor with legacy defaults. Loading
  a due renewal invalidates stock and redraws the tier, without live transactions.
- Native mSP hours9-22, Combini7-23, raffle10 opening. Calendar exclusions for
  month-end raffle and fourth-November-Thursday+1 across today/tomorrow/+2.
  Bargain-event dates/forced chore opening are not present in the current app.
- Native renewal leaflet18/19/1A uses closed date/month/year and paper55;
  original one-shot full-mailbox policy retained (source reviewed, not forced).
- Native FG22..26, authored player/NPC starts, EXIT_DOOR1 centres and stair
  destinations; Departlower uses static rom_shop4_1 BG, upper rom_shop4_2 f/w.
  Separate X/Z extents and narrow stair/door corridors, portal reentry lock,
  owned root/clock/HUD replacement, floor-sensitive stock cache and temporary
  previews. Background/attendant clearance uses existing player/Nook radii.
- Native exterior cKF/palette37+tier/season and measured footprint/door caches;
  original Cranny restores on load. Original uniform NPC246..249; upper276/293
  with Timmy's native message bases/173D duplicate question and voice2. Lower
  Conv12/Super27/Depart9 route tables ported from original source. Clock rigs,
  joints and positions per-tier; native department preview camera table.
- Raffle prizes/drum/ball/Nook offsets now follow tier layout; no upstairs
  raffle or ordinary wall clock on raffle day.

Runtime fixes/evidence:
- Initial preview exposed a schedule cycle (MovementSet is after Doors) and
  a resource formerly inserted at Startup. Removed the conflicting beforeDoors
  constraint and initialized ShopObstacles explicitly; clean previews then run.
- Native exit corridors initially permitted walking past the interaction plane.
  Stop the player radius before the source portal centre; all four tiers then
  pass entry/exit and compare all40 saved fields except position/playtime.
- Native stairs cross both ways with one room/primary attendant and no duplicate
  roots. QA uses centred native corridor setup after turning; the controller's
  wide turn otherwise misses the narrow portal. Staff clearance checked.
- Synthetic Temp higher-tiers-20261025 only: threshold25000 stays tier0 today,
  renewal+2 persists; tomorrow closure refuses entry, notice arrives; due date
  yields tier1/restock/entry. Tier2 sales240000 without visitor stays tier2;
  visitor flag permits pending department renovation. Legacy no-progress loads.
- Department downstairs paint980 charges/sells only slot5, preserves pockets,
  conditions/catalog/mail; upstairs blue bed2100 gives77900 Bells, pocket12,
  sold slot0, sales2100 and ticket2C48->2C49. Timmy's native wallpaper preview
  and cancellation preserve all40 fields except position/playtime; inspected.
  Early QA used a guessed model name and searched for price on the final choice
  page; corrected to native model/actual question and reran successfully.

Limits/acceptance:
- Twin68-zone escort/secondary actor echo choreography, renovation-board visual
  presentation and native neighboring-field rewrites remain fidelity work.
  Native echo text is already provided by the original message script.
- Foreign-town travel is not implemented here; the native visitor gate is
  persisted and tested with a synthetic flag, not silently waived.
- No tests added/run, no actual-save loads. One reused implementation/review
  agent only; final read-only review found no actionable issue. Parent/M4,
  tracked counts, parent ownership and Philip acceptance remain open.

Additional final checks:
- 07:00 and22:00: only Combini permits entry;23:00: all four tiers refuse.
  Raffle October31 at09:00 refuses, F6 to10:00 enters Departlower with original
  raffle layout/sign/drum and no ordinary clock/upper twins. Screenshot inspected.
  October29 threshold25000 does not schedule across the October31 raffle.
- Actual notice body inspected with the intended closure date/month/year.
  Final native echo text/Timmy nameplate and wallpaper preview/restoration seen.
- Native attendant body clearance prevents walking through the upper twins;
  repeated closed-door refusal sound removed for held walking input.
- Final source check6.61s, warnings-denied all-target clippy8.88s, fmt check,
  capture19.11s and default19.63s pass. Final release4m36s and final check3.05s
  pass; final fmt check passes. No test suites were added or run.
  Earlier release5m42s also passed before the final closed-door sound removal.
- Final native raffle/exclusion logs have no ERROR/WARN/panic. Missing-disc
  plain pockets inspected with numeric IDs retained and only expected warning;
  no claim of full higher-tier disc rendering without assets. Owned previews
  closed by verified EXE identity, including final31932. Initial20644/9692
  terminated on the recorded startup faults; all subsequent owned previews
  replaced/closed by the isolated launch helper, never unrelated processes.
- Real save only hashed throughout; final SHA256 unchanged
  C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
  Unrelated .gitignore and actual save remain untouched/unstaged.
