# Retail pockets and mail tag artwork

Philip selected remaining retail UI (priority2), 2026-10-24. Start with shared
pockets/mail/shop popup presentation, following existing UI order. Parent retail
pockets/mail/shop tasks and acceptance stay open; pause/save remains a follow-up.

Design: load original sen_itemw name-tag and sen_win action-tag frames/shadows
and sen_win_cursor from the disc via sen_item2_DL_mode. Preserve current known
popup actions/14px spacing and basic anchors; lift tall menus clear of the name
tag and use source SELECT2 body sizing. This is an art
slice, not a full port of mTG edge/arrow/morph animation or gameplay actions.
Use original ordinary tag colors and0.875 font scale consistently, cache models
and bounds before render extraction. Missing assets retain existing UI.

Plan, native implementation:
- [x] Audit m_tag_ovl.c frame/shadow/cursor and color/font setup.
- [x] Dedicated pockets_disc/tags.rs runtime model loader/bounds/draw helpers;
  PocketAssets owns models; replace generic name rectangle and dialogue action
  body/cursor while retaining no-disc and missing-asset fallbacks.
- [x] Sequential check/clippy/fmt/capture/default/release. No test suites.
- [x] Temp-only synthetic QA: ordinary tool/gift tags, long names, mail action
  and discard, shop Sell/Quit; cancel, selection, reopen and numeric state.
- [x] Independent review; update task/comms/both roadmaps; complete own claim,
  explicit source/docs commit and push. Real save only hash before/after.

Review focus: pre-extraction bounds, I4 shadow intensity coverage,
frame/shadow common transform, source colors, all existing actions unchanged.

Implementation and source:
- m_tag_ovl.c8170+: on/off ink(120,35,255)/(140,150,190), cursor(215,30,215),
  item ink(90,60,50), ordinary name/action/confirmation prim/env colors.
- Frame/shadow/cursor display lists loaded once by PocketAssets, bounds cached
  from TriangleSink before render extraction. Shadow uses the same transform
  as the frame; I4 intensity supplies alpha locally, no global decoder change.
- SELECT2 sizing constants from mTG_set_tag_win_scale_p1360-1447/call2242.
  Initial generic bounds fit clipped long menu rows; corrected source sizing.
  Then tall frames obscured sender name; final shared offset caps body at y190,
  keeping name at193 and translating heading/text/cursor/frame equally.
  Current discard prompt gap22px is retained (source gap14px), a port adaptation.
- Source present_str1889/init1947 names condition1 items Present. The carried
  slot's condition is used while grabbing. No transaction/input/save code edits.
- Read-only review and both sizing/shift/Present follow-ups found no remaining
  concrete issues. No test suites added/run.

Evidence under %TEMP%/bevy-crossing-room-critters/
retail-tags-557c8cac24ff45508a279e93bf012de1 only, synthetic saves/copied EXE and
explicit disc. initial/ and sizing-only/ captures show superseded layouts;
stderr-225503 and232103 are intermediate. Final layout stderr-232510,
tool/gift/paper/mail-actions/mail-discard/mail-name/reopened/empty-slot.png
inspected; before.json vs after.json all serialized fields equal except
playtime_secs. Wrapped name follow-up/final shop stderr-232721,
final-present.png and sell-actions.png inspected, sell-after.json differs only
in player_pos/playtime_secs. Wallet80000, net/wrappedbed/paper/apple and packed
condition4, both letters and complete other fields preserved.
Final capture EXE SHA256C6294C0614DD800360D8C488609281A319400F70E42C839258C2B56207F98A3D.
No ERROR/panic/WARN in final disc log. Initial sell helper forgot that Quit only
closes the action menu; added pockets Escape before snapshot, then passed.

Explicit nonexistent-ISO fallback: stderr-233007, fallback-pockets/closed.png
inspected; native pockets/menu opens and cancels, no tag assets required. The
expected missing-disc notice stays usable. No missing-tag-only disc was built.
Owned39192/39656/38544/36076/31668 previews all closed. Real save only hashed,
before/after unchangedC15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
Final check2.73s/clippy6.89s/fmt/capture10.94s pass. One expected fmt correction
applied after Present branch. Earlier release builds stopped for visual fixes;
final release4m06s and default31.41s pass.

Full tag geometry/arrow/morph fidelity, context-special color variants,
pause/save and remaining shop/HUD features and Philip acceptance remain open.
