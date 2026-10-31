# Nook's welcome, house choice and initial payment

Philip selected this slice with "go" on 2026-10-30, then resumed after the usage
wrap-up. Implementation, bounded runtime verification and all builds complete.
No tests added or run.

## Implementation

Original source: actor/ac_intro_demo_move.c_inc and npc/ac_npc_rcn_guide*.c_inc
in ACGC-PC-Port. Rustimal migration authority remains pinned object
122867f762122eb266f04e319af0f0c31d54f476. Plan:
docs/superpowers/plans/2026-10-30-intro-nook-house-choice.md.

- Station walk now leaves free control for the stairs; native Z970 triggers Nook.
- Dedicated intro_house controller drives native greeting/tour, all four door
  targets, before/after inspection messages, refusal/confirmation, explanation
  repeat, leaving-acre reminder, payment and nearer-end departure.
- Original messages/choices and NPC0 order9=1 drive payment. Pockets Give/Quit
  selects only exact 2100/quest-condition2; cancellation preserves payment state.
  Reuse the money handover, then native07EC. complete_intro avoids refunding bag.
- Defaulted HouseSave.home_slot preserves old saves' home0 and selects door,
  exterior/roof, mailbox/flag, gyroid, postman delivery and statue placement.
- Commit home/loan17400/job only after departure. Block decorating and ordinary
  quests during preview. Existing first-job guide takes over at shop entry.
- Missing assets fall back without rewriting completed identities. Runtime also
  found Rover's unguarded Res<GameDisc>; match train builder's resource guard.

## Verification evidence

All automation used Temp/bevy-intro-houses-20261030 with synthetic saves and
explicit disc paths. No real save was loaded for automation. Captures inspected.

- Native normal train/station arrival, free stairs walk, Nook call/greeting/tour.
- Houses1 and2 inspected; refusal kept home0, loan0 and quest bag. Original
  confirmation and repeat-explanation choices exercised. Acre warning released.
- Escape and Give-menu Quit cancellations returned07EB and reopened selection;
  valid Give removed exactly the bag and used handover. Final latest-route run
  completed with no guide transform edits: house2, loan17400, empty pockets,
  intro_done and first_job true, guide despawned, controls/save returned.
- An earlier run exposed always-north departure blocked at the board. That old
  build alone was moved beyond it to inspect downstream state. Final fresh run
  uses source Z1540 split and north1300/1220 or south1900/1980 routes correctly.
- F9 and process restart restored selected house2 door, mailbox and palette5
  roof while preserving full house, pockets/conditions, wallet/loan and room.
  Shop entry displayed existing first-job greeting.
- Completed-save missing-disc + INTRO=play kept house3, loan12345, wallet543,
  pocket IDs/conditions, numeric room and job flag unchanged. Fresh missing-disc
  fallback returned usable world/save with loan17400, one quest bag and job flag.
- check/clippy (-D warnings)/fmt, capture/default passed after final Rover guard.
  Final release rebuild passed (4m29s). Focused reviewer confirmed ownership/payment
  fixes, completed-save guard and native departure coordinates.

## Scope and remaining acceptance

The full intro remains development-gated as before (INTRO=play / ARRIVE QA).
HOUSES QA starts at Nook for isolated checks; it never edits the real save.
Philip's visual/gameplay acceptance, M4 and overall migration remain open.
Existing train engineer/smoke/sounds/camera-fidelity and remaining native emotion
command gaps are outside this slice.
Room-size changes retain the existing restart requirement. No asset exports,
dependencies, new tests, implicit saves or sibling-repository edits.

Preserve unrelated .gitignore and savegame.json. Real save SHA256:
C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
One reused reviewer; inline main checkout under standing authorization.