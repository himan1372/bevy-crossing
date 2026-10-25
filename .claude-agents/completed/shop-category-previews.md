# Shop clothing, wallpaper and carpet previews

Philip selects priority1 after popup placement,2026-10-25. Parent retail-shop-ui
remains open/ownership unchanged; finish this bounded preview/transaction slice.

Design/source: aNSC SHOW_CLOTH18 -> preview CONFIRM19 or skip CANCEL20;
SHOW_ITEM40 -> preview OFFER41 or skip DECLINE42; third choice cancels31.
Only final yes uses existing exact stock-slot transaction. Cloth restored before
decision commits, wall/floor reset on completion/refusal, as source. Temporary
textures never write PlayerLook/room/save data. Native room camera endpoints,
28tick Hermite(.6,.3) and reverse; shirt uses MENU_CHANGE1/frame36 texture switch.
Original naked mannequin uses shirt255/obj_shop_manekin while trying on.

Plan, native execution:
- [x] Audit source state/message/restore paths and existing renderer/camera.
- [x] Dedicated shop_nook/preview.rs request/cache/texture lifecycle + camera
  blend. Tag only relevant shop surfaces/player cloth materials. Keep fallback.
- [x] Wire three-choice preview question into existing purchase controller;
  use original clothing clip, restore before commit/cancel/scene reset, keep
  stock/Bells/pockets/ticket validation and input ownership.
- [x] Sequential check/clippy/fmt/capture/default/release, no test suites.
- [x] Temp synthetic screenshots clothing/wall/floor/restore/skip, purchase,
  refusal/full/funds cases, cleanup/reload and saved-field comparisons. Hash
  real save only; no real settings/save access.
- [x] Independent source review; task/parent/comms/both roadmaps; complete claim,
  explicit source/docs commit/push. Philip acceptance remains separate.

Review focus: all exits restore materials/camera/mannequin; unchanged saved
look/decorations; exact duplicate stock identity; source try-on frame timing;
scene/stock invalidation; missing assets preserve normal purchase flow.

Implementation and bounded verification, 2026-10-25:

- Source baseline HEAD b4b896ba; local original C: aNSC_show_item_check,
  sell_check and sell_answer0, Player_actor_*Change_cloth, aSM draw and
  Camera2 main_inter. New shop_nook/preview.rs holds temporary texture/clip
  state, per-part decoded-image caches and original image handles. Surface
  parts are restricted to ShopShell; shirt parts to the player's segmentA.
  PlayerLook, HomeFloors, room geometry, obstacles and saved decoration never
  receive the temporary pattern. No new dependency or extracted asset.
- Three choices route to the native second question or finish. Final decision
  uses the existing exact index/item/price check and transaction. Shirt gesture
  runs before both try-on and return; texture switches at frame36; bare model
  uses cloth255 with obj_shop_manekin_model, segments8/9. Return to idle after
  the gesture. Native angle easing faces camera while trying on. Wall/floor
  camera uses the source endpoints, 28tick Hermite(.6,.3), reverse on clear.
- Review caught input becoming World while dialogue was cleared for gestures;
  update_focus now reserves Dialogue for active previews. Runtime reload exposed
  stale owned talk; leaving the shop now clears all Nook steps. Shared Resource
  mannequin cache makes OnExit and Update cleanup use the same entity/visibility
  record (separate Local instances would not). Independent final review clean.
- Temp only: shop-preview-9b97ee4ad9344cb393e71c7971661ec9 beneath
  %TEMP%/bevy-crossing-room-critters. final-{wall,floor,shirt}-preview/restored
  images inspected. Saved final-before versus each cancel, skip-cancel and
  initial-cancel: all39 fields equal except position/playtime. Native camera
  eyes asserted within .001: wall(3000,4.875,-2981.5), floor(3000,10.875,-2983).
  Restored original patterns/player shirt/mannequin visually inspected.
- Shirt preview purchase350: Bells80000->79650, item2408 in slot0, ticket2C48,
  only stock0 sold. Wall skip-preview purchase2000: Bells77650, item2701,
  ticket stack2C49, only stock0/2 sold. Carpet preview purchase1800:
  Bells75850, item2601, ticket stack2C4A, only stock0/1/2 sold; shop_sales4150.
  Saved player_face/player_cloth remain0; only expected money/pocket/stock/
  sales/collection fields differ beyond position/playtime. A helper initially
  expected the wrong ticket-stack encoding; corrected its numeric assertion.
- Latest final capture build: short100Bells and full15pockets each refuse all
  three categories; all39 saved fields preserved except position/playtime.
  final-short-* and final-full-* responses inspected, original patterns and
  shirt restored. Initial full-pocket driver faced Nook instead of the shirt;
  fixed synthetic approach to turn fully before E and reran the scenarios.
- During try-on, combined movement/pockets/pause/F5/F9 inputs preserve exact
  player transform and synthetic-save mtime. Temporary mannequin exists during
  preview and disappears on cancellation. final-input-after equals full-before
  in every field except position/playtime. Reload/reentry used in refusal setup.
  Explicit missing ISO fallback opens/closes plain pockets; image inspected.
  No ERROR/WARN/panic in final disc log (missing-disc warning expected).
- Final check11.70s, clippy23.86s, fmt, capture1m02s, default55.73s and
  release4m47s pass, sequential. No test suites added or run. Owned previews
  3024/11160/28072/12588/3956/9004 closed. Real save only hashed unchanged:
  C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.

Limits: clothing puff/wear sound not ported in this slice. Individual missing
texture banks and forced AppState/stock invalidation were source-reviewed,
not fabricated at runtime. Saved appearance was held constant; no new wardrobe
equip feature. Parent retail-shop-ui/M4 and Philip's playtest acceptance remain
open and their ownership unchanged. Next priority: pause/save presentation,
special quest/mail tags, then remaining shop effects/services.
