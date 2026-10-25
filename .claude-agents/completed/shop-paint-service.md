# Nook paint / roof-colour service

Philip says lets go after prioritising remaining services; paint is first.
Base6a3693c2. Original aNSC_set_talk_info_sell_item202 and sell_check203:
check resident/funds, set next_outlook_pal, mark actual displayed stock sold,
charge once/record purchase, no pocket item/capacity check/ticket. Existing
mHm_CheckRehouseOrder equivalent applies pending palette on startup/load.
Only local resident exists in this port; visitor/travel service stays separate.

Design/inline plan:
- [x] Audit original transaction/palette timing and existing save/renderer.
- [x] Route paint to native offer202 and existing exact-slot Purchase; valid
  accepted resident purchase queues palette before charging/selling. Preserve
  no-home, insufficient Bells, cancellation, stock invalidation and full pockets.
- [x] Add a validated HouseSave paint helper, without new save fields. On F9,
  refresh only own house palette through bounded native model cache; keep other
  houses/materials unchanged. Size-changing reload still requires restart.
- [x] Sequential check/clippy/fmt/capture/default/release; no test suites.
- [x] Temp synthetic QA: offer/accept/cancel/short/full, exact slot and wallet,
  no item/ticket, pending/current palette, reload/restart roof art, repeat sold
  interaction, other roofs unchanged, missing-disc smoke; real save hash only.
- [x] Independent review; task/parent/comms/both roadmaps, complete ownclaim,
  explicit source/docs commit and push. Parent/M4/Philip acceptance stays open.

Required dependency discovered: native FG22 Cranny has no paint reserve;
FG24 SUPER/Nookway has RSV_PAINT at3,9. Added only the ordinary tier2 room
rom_shop3/default patterns69, centred6 with12cell extent; native FG door spans
7/8 at12, so door centre is originX+2. Build after loaded shop tier, then keep
Layout stable for the session. Shell, merchandise/height/collision, movement/
door, Nook coordinates and room preview camera share this Layout. Other tiers
and raffle layouts retain prior behavior. Nookway clock/uniform/exterior and
full tier fidelity remain separate follow-ups; do not imply those are complete.

Implementation/development evidence, 2026-10-25:

- Purchase uses original202/203 (0x2EB0/0x2EB1), existing exact index/item/price
  and funds checks, a validated twelve-colour HouseSave helper, existing
  register/ledger/sold display. No capacity check, item or ticket for paint.
  Current outlook and expansion's ordered palette remain unchanged at purchase;
  next_outlook_pal applies through existing startup/load check_rehouse_order.
  Missing-house/stock-invalid branches remain refusals, source-reviewed only.
  No save schema change. Visitor/travel state is not implemented in this port.
- Own-house Appearance marker and native-model cache replace only house0 mesh
  children on a same-size palette change after LoadSet. Cache retains at most
  twelve variants and their window materials; new windows invalidate the light
  level so current illumination applies. Other houses/materials aren't mutated.
  Size-changing exterior reload still requires restart, as before this slice.
- Native Nookway room uses FG24, rom_shop3 floor/wall and default69, centre6,
  bounds5.125 and source doorX+2. Build follows apply_loaded_shop; Layout stays
  fixed for the session rather than changing when sales upgrade the tier.
  Shelf placement/heights, collisions, player entry/exit, Nook coordinate
  conversion and preview camera share that layout. Cranny and other/raffle
  paths keep prior layouts; Cranny's wall clock is not misplaced in Nookway.
- Independent review found no important transaction/cache/layout/scheduling
  issues. Early compile caught a misplaced track insertion and FG variable
  shadow; corrected. Clippy map-entry corrected. No tests added or run.
- Temp only: paint-be27d56d38a54be4a20bd40783d406c3 under
  %TEMP%/bevy-crossing-room-critters. Initial QA approach was outside walkable
  bounds/insufficiently turned; moved it inside and used the real facing/E flow.
- normal purchase: orange paint222E price980, Bells5000->4020, sales+980;
  exactly stock0 sold, duplicate stock1 untouched. Pockets/conditions/catalog/
  tickets/mail unchanged; house changes only next_outlook_pal0->1. Sold-display
  E and re-save preserve all39 fields except position/playtime. Roof children
  remain unchanged until F9; afterward current palette1, original root retained,
  children replaced, all three other houses/mesh-material handles unchanged.
- cancel and100Bells refusal preserve all39 fields except position/playtime.
  full15pockets purchase succeeds for980 with every pocket/condition preserved,
  no item or ticket, same palette/stock/ledger assertions. Original pages/sold
  sign/roof colours inspected in normal/short/full screenshots.
- All12 palettes loaded through F9, with rendered roof colours inspected
  at0/1/5/11. Revisiting0/1 reuses cached material handles; child counts and
  all other house handles remain stable. Cycle finishes with all saved fields
  equal to full-loaded except position/playtime.
- Nookway carpet preview camera eye asserted(2998,10.875,-2985); original
  selected carpet and normal room restored on refusal, all39 fields unchanged
  except position/playtime. Real E exit through offset doorway returns to town.
- Fresh restart keeps roof1, next palette1, Bells4020, full pockets and sold
  stock. Unrelated field_items changed empty->20 on startup; affected paint
  state and every other saved field except position/playtime stayed unchanged.
  This existing startup population behavior was not changed in this task.
- Cranny restart regression: original centre(4,1,4)/shell transform, wall clock
  and absence of paint shelf verified; shovel offer/cancel preserves all39
  saved fields except position/playtime. Both disc logs have no ERROR/panic.
- Final check10.43s/clippy22.41s/fmt/capture1m06s/default1m19s/release5m40s
  pass, sequential. Later Rust edits are comments only. No test suites.

Parent retail-shop-ui/M4/Philip acceptance remain open. Nookington's paint-floor
access belongs to remaining tier layouts; ordinary Nookway service is complete.

Explicit missing ISO fallback opens/closes plain pockets; image inspected, no
panic/error (expected missing-disc warning). Owned40276/31396/6632/40176 previews
closed. Real save only hashed before/after, unchanged:
C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
Roadmap now starts with multi-item selling, then turnips/counter conversations/
remaining tier and seasonal fidelity; parent ownership and item counts unchanged.
