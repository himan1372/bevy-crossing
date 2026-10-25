# Empty text mesh renderer allocation errors

Completed 2026-10-24 by codex, selected by Philip after raffle-ball-scale.
Parent nook-tickets-catalog remains claude-owned/open; acceptance is separate.

## Cause and fix

Bevy0.19.1 skips allocation for zero vertex bytes but its upload loop still
copies vertex/index keys. Blank letter lines and markup-only dialogue prefixes
produced indexed meshes without vertices, causing two unallocated-key errors.
Shared msg_window::text_mesh now returns Option<Mesh>, None for no glyph
geometry. All six calls in dialogue, pockets/mail, catalog and music grid skip
allocation/spawning for None. Layout, markup, reveal/choice timing unchanged.
No registry/dependency changes; temporary producer logs removed.

## Evidence

Synthetic saves, copied capture EXE, explicit disc, all runtime evidence in
%TEMP%/bevy-crossing-room-critters/render-empty-17b173340bb6414fa02a9fc2f80cb3fc:

- Baseline stderr-172922.log: empty Paint text at(64,112), then two allocator
  errors about6ms later while opening the original delivered blue-bed letter.
- Baseline stderr-173210.log: third-prize0x1138, 0x10DB/0x10E3/0x10DD chain;
  empty dialogue typed=1 then two allocator errors.
- Fixed stderr-212908.log: same letter opened, fixed-letter.png inspected,
  blank spacing preserved, no ERROR/panic. baseline-mail-after.json matches
  fixed-mail-after.json in bells/pockets/conditions/catalog/stock/sales/letters/
  house_mail/home_furniture. Wallet75800, fourteen apples+bed.
- Fixed stderr-213012.log: actual third-prize win, same message chain, no
  ERROR/panic. fixed-raffle-win/prize.png inspected; next/choice flow works.
  fixed-raffle-after.json asserts wallet80000, exactly five tickets consumed,
  wrapped0x1138 in slot0(condition1), two stacks0x2C4C remain, lineup
  [0x1358,0x1008,0xFE10]. Diagnostic first expected0 for sold slot; corrected
  assertion to source RSV_SHOP_SOLD_FTR=0xFE10 and passed.
- Independent read-only review clean, all six text_mesh callers handled.
- Workspace check39.82s/clippy all-targets -D warnings9.26s/fmt pass.
  Capture used for before/after runtime QA; final cached capture1.63s,
  default26.96s and release6m00s all pass.
  No test suites added/run. Catalog/music-grid callers compiled/inspected;
  their complete UI flows were not replayed in this bounded renderer fix.
- Owned mail30448 and raffle7184 closed with EXE identity checks; baseline26848
  had exited before resume. Real save only hashed, unchanged SHA256:
  C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.

Non-furniture previews, exact clock fit and remaining retail UI stay open.

## Original report (superseded by diagnosis above)

Observed2026-10-24 at372e29bf during catalog-raffle-verification. Untriaged.

Fresh startup delivered ticket0x2C48 and catalog bed0x1358 into house mailbox.
Opened mailbox0, transferred its three letters, then read bed and ticket letters
from pockets. At2026-10-24T19:14:22.431065Z and.431578Z, the capture build logged
bevy_render::slab_allocator: "Use-after-free: attempted to copy element data
for an unallocated key". Process continued; inspected letter screenshots and
saved inventories/attachments were correct. No claim this is harmless.

Evidence: %TEMP%/bevy-crossing-room-critters/
catalog-checks-191b6221bb294ac48586df52c3f72862/stderr-151316.log,
order-letter.png, ticket-letter.png, restart-after.json and mail-collected.json.
Read bevy-debug/systematic-debugging/QA skills; reproduce with synthetic saves
before assigning cause to game code or Bevy. No source changes made here.

Additional2026-10-24 evidence during raffle-ball-scale: a height-corrected
preview logged the same two allocator errors at20:35:07.128564Z/.128727Z,
immediately after a second-prize win (0x10DA/0x10E3/0x10DD). Prize display
refresh is another reproduction lead; cause is not established. Evidence:
%TEMP%/bevy-crossing-room-critters/raffle-ball-fix-fe8bf84ca33c4e5c9798ec43a0a61068/
stderr-163004.log. This extends the issue beyond mail viewing.
