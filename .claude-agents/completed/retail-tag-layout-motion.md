# Retail tag placement, arrows and motion

Philip selects priority1 after retail-tag-art,2026-10-25. Implement supported
pockets/mail/shop popup placement and opening/closing, preserving actions.

Design: translate mTG item/SELECT2 body/text offsets, slot anchoring, right/bottom
edge rules and after-item/after-select placement. Load original item arrows;
ordinary action/confirmation after parent have arrow_dir0 in source. Retain
port single-line From/To names and existing supported actions. Multi-line quest
and special mail tag families remain separate; no invented gameplay.
Presentation owns persistent popup roots below pockets root, independent of
changing icon dynamic root. Animate transforms, not rebuilt meshes per frame.
Use20-frame name wait,0.125/tick normal open/close,0.185/tick first action after
its parent finishes. Cursor changes must not restart popup animation. Pause
freezes animation; close/reload/pockets teardown leaves no stale entities.

Plan (native execution):
- [x] Audit C1195-1535,1889-2000,2180-2510,8000-8060,8400-8495.
- [x] Pure layout math in tags/layout.rs, runtime retained presentation and
  motion in tags module. Replace fixed name/action rendering only with assets
  available; keep existing missing-disc/missing-tag fallbacks.
- [x] Sequential check/clippy/fmt/capture/default/release, no test suites.
- [x] Synthetic Temp captures all pocket/mail edges, long name, confirmation,
  cursor preservation, open/close/motion samples, reload/no-disc, full numeric
  state comparison for navigation/cancel.
- [x] Independent review, task/parent/comms/both roadmaps, claim complete,
  explicit source/docs commit/push. Real save only hash before/after.

Implementation/evidence:
- Pure layout algebra follows supported ITEM/SELECT2 branches, including the
  middle pocket row/two-choice above-name rule. Frame/text/cursor use native
  source coordinates and transforms instead of the previous bounds-fit boxes.
- Source item pointer lists loaded once. Ordinary child action/confirmation
  has no pointer arrow, matching source arrow_dir0. Current single-line From/To
  labels use ordinary name layout; special multi-line mail/quest frames excluded.
- Retained popup roots own cached frame/arrow geometry and text children. Choice
  changes rebuild only content, preserving root/scale. Tick system animates
  transforms at60Hz; name wait20, open/close.125, action.185 after parent opens.
- Review caught return-to-NONE initially left name open and allowed overlapping
  reopen. Fixed closing all affected roots, actual zero-scale completion, dirty
  redraw/new20tick wait and input guard during opening/return (Cancel/Back kept).
  An intermediate fixed8tick return delay was removed to match partial returns.
  Empty-slot old name is despawned immediately. Final read-only reviews clean.

Runtime only in %TEMP%/bevy-crossing-room-critters/
tag-layout-bcf2a03576914d28a4b28b487963bc1a, synthetic saves/copied EXE, explicit disc:
- stderr-030506: pocket four corners, right/middle rows, letter top-left/top-right
  long sender and bottom-right; action menus and nested confirmation inspected.
  before.json vs after.json differs only playtime_secs. Wallet80000, all15 items,
  packed gift condition4 and all10 complete letters preserved. Cursor move
  retains exact action entity. Rapid partial-cancel/reopen has no duplicate
  roots; early Down/Enter on hidden gift actions does not unwrap the gift.
- name-open/action-open/action-return/rapid-reopen JSON hold sampled transforms.
  Name first visible at0.3522s, finalscale1, consistent with20tick wait plus
  request/frame timing. Popup rates/parent order also checked against source.
- Final capture D59F407E8369BF45419A5EDF8F160340975F9BE7F4CB95E328C1A1821B8FD805,
  stderr-031036: QuickLoad has no stale roots, empty slot has no name/arrow,
  final-return.json records shrink/name reset; empty-setup vs empty-after differs
  only playtime. Two-action-middle-row.png and Sell/Quit screenshot inspected;
  sell-after differs only player_pos/playtime from empty-after.
- stderr-031639 explicit missing ISO: native menu opens/actions/cancels;
  fallback-menu/closed.png inspected. Missing-tag-only disc was not constructed.
- Final disc log no ERROR/panic/WARN. Owned35932/22444/26736 previews closed.
  Real save only hashed, unchanged before/after:
  C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
- Final check2.10s, clippy5.35s, fmt, capture16.63s and release4m09s pass;
  default46.80s passes. No test suites. Pause freeze reviewed via
  GameplaySet gating; not forced through a synthetic pause state at runtime.

These bounded checks do not establish every special tag family or Philip's
acceptance. Parent retail tasks stay open and their ownership is unchanged.
