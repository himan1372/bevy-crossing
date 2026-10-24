# Remaining catalog and raffle verification

Philip authorized this follow-up on 2026-10-24 after catalog-preview-review:
play purchase tickets (including overflow mail), insufficient bells, five
catalog orders full, restart order delivery, and the raffle ball animation.
Use only synthetic Temp saves and an explicit disc. No automated tests.
Parent nook-tickets-catalog remains claude-owned; do not steal its claim.
Record numerical state, inspected screenshots, failures and scope limits.

## Completed verification, 2026-10-24

Checked commit372e29bf with capture build12.62s and a copied standalone EXE.
No gameplay source changed and no test suites were added/run. Scenario helpers,
numeric assertions, screenshots and synthetic saves remain only under
%TEMP%/bevy-crossing-room-critters/catalog-checks-191b6221bb294ac48586df52c3f72862.

Passed:

- Purchase: blue bed0x1358 at2100 Bells. Wallet80000->77900, exact bed in
  pocket0, October ticket0x2C48 in pocket1, conditions0, stock0 sold only.
  Response0x10AE and ticket explanation inspected.
- Overflow:14 apples0x2800 plus one empty slot. Purchase fills final slot
  with bed, leaves apples/conditions intact, wallet77900, tickets(10,1).
  Response0x10AF says ticket will be mailed; screenshot inspected.
- Short on Bells:100 Bells selecting2100 bed, confirming yes ->0x10AB.
  Before/after equality for bells, all pockets/conditions, stock/sales, full
  catalog, letters/house mail and room furniture. No order/purchase mutations.
- Five orders full: retained exact slots[(1358,0),(1008,0),(1138,0),(1268,0),
  (135C,0)] in hex, response0x10AC before opening catalog. Same full-field
  refusal equality as above, wallet80000 unchanged.
- Actual paid order from overflow scenario: bed2100, wallet77900->75800,
  slot0(0x1358,0), other slots empty, full pockets intact, pending ticket(10,1).
- Real process restart:0x057 attaches0x2C48, then0x049 attaches0x1358,
  both stationery55. Ticket count clears to0 and all five orders empty;
  wallet, pockets/conditions, stock/sales and room data unchanged. Opening
  mailbox transfers the exact three letters (Museum plus two Nook letters)
  to pockets; inspected delivered-present icons, original bed-order letter
  and October-ticket letter. House mailbox empty afterward, attachments intact.
- Raffle-day room has prizes/drum/decorations. Three stacks of five October
  tickets; actual draw lost (result3), consumed first stack only, leaving10
  tickets, wallet80000 and prize lineup unchanged. Declined retry explicitly.
  Sampled ball entity from Hidden through Inherited at multiple moving
  positions, then settled at(3000.5,1.55,-2999.2036). No exact-frame claim.

Failed / follow-ups:

- Ball is not discernible in inspected screenshots at0.65/1.2/2.0/3.0s.
  load_ball_frames calls build_model_full -> sink_meshes -> OBJECT_SCALE
  (0.00025), then present also sets parent scaleGOODS_SCALE(0.00025).
  Live child Aabb half width0.0625 times parent scale gives diameter0.00003125
  world units:4000 times smaller than a single scale. Original aGRBL_draw
  applies scale0.01 once in game units. See backlog/raffle-ball-scale.md.
- Two bevy_render::slab_allocator errors at2026-10-24T19:14:22Z during mail
  viewing: "Use-after-free: attempted to copy element data for an unallocated
  key". Game continued, letters/numeric snapshots correct. Not diagnosed or
  dismissed as harmless; see backlog/mail-render-slab-errors.md.

Setup corrections excluded from passing evidence: Python calendar-day seed
did not match game day9794; used the game's saved stock day to prevent restock.
Early dialogue choices / a30ms facing nudge sometimes failed, and initial F5
was ignored while modal. Corrected to settled choices,120ms facing, checked
F5 file timestamp and repeated baselines; both refusal comparisons passed.
Displayed normal date was Oct25; day offset+6 reached Oct31 (verified scene
and native clock text). Hour offset targeted noon. These are scenario setup
facts, not a new clock-fidelity claim.

Owned processes37148 ->7392 ->2608 closed in sequence after checking EXE paths.
Real save before/after SHA256 unchanged:
C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
Verification task complete with failures recorded; parent task, non-furniture
previews, clock fitting and Philip acceptance stay open. No renderer/ball fix
was included in this verification-only follow-up.
