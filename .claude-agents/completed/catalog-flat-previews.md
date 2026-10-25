# Catalog flat previews

Philip selected priority1 on2026-10-24. Bounded codex follow-up; parent
nook-tickets-catalog stays claude-owned/open. No tests without request.

Design: original mCL_item_draw uses flat overlay display lists for wallpaper,
carpet, stationery and music, separate from the furniture perspective camera.
Use original models/segment bindings and fixed source placement/scales, not
generic room objects or invented album art. Music uses shared note art and its
source bob/sway. Cache selected models to avoid reallocation on every redraw.
Keep order, price/refusal, collection and save semantics unchanged.

Plan (native implementation in current workspace, authorized selected slice):
- [x] Audit original flat model modes, paper pointer table, wall/floor segments.
- [x] Add catalog_screen/flat_preview.rs with lazy model cache and flat spawn/
  music animation systems; existing catalog root owns cleanup. Furniture stays
  on existing camera. Missing/empty/no-disc selections spawn nothing.
- [x] Run sequential workspace check, warnings-denied clippy, fmt, capture,
  default/release builds. No test suites.
- [x] Temp-only synthetic collected lists: inspect all4 families/multiple
  patterns, empty tab, navigation/reopen/reload and music motion. Assert preview
  navigation/cancel preserves wallet, pockets, orders, mail and decorations.
- [x] Independent source review, task/comms/both roadmaps, complete own claim,
  explicit source/docs commit and push. Parent acceptance remains open.

Review focus: floor palette is segmentD (room shell usesC); paper ruling alpha;
music identical note for all songs; cache keys and root cleanup across changes;
missing assets and empty tabs must not revive previous preview.

Implementation: dedicated flat overlay module, original modes/models, matching
source(-85,7) position at16 overlay units/pixel; paper y4. Cache canonical paper
IDs modulo64 (catalog uses20C0..20FF), one shared music-note model. Music timer
starts60 and decrements at60Hz; source 0x222/0x444 sine bob and cosine tilt.
Reviewer caught initial paper range mismatch before final build; corrected
canonical key and follow-up review clean. Early wall/carpet/music captures
look correct; final four-family runtime checks pass.

Evidence: %TEMP%/bevy-crossing-room-critters/
catalog-flat-71148d39de284614ab649aa9666bed22, explicit disc/copy EXE and synthetic
save only. Initial stderr-214754 and initial/ screenshots precede paper fix;
final binary SHA25667FF12DF77DD36A0F7923BB1C9BC8491B9F99A5D6089FA1C303F80AD1DEDFB9C,
final stderr-215052.log has no ERROR/panic/WARN. Captures inspected: first/second/
last wall/carpet, airmail/sparkly/essay/ink paper, two songs' note, empty clothes
tab and returned furniture. Music-motion.json records20 changing transforms.

before.json vs after.json: all serialized fields identical except playtime_secs.
lifecycle-after.json: only playtime_secs/player_pos differ, as expected after
reload/reentry setup. Exactly one flat root at each selected boundary:
wall2742, carpet2642, paper20FF; none on empty tab or after close. F9 is gated
while catalog owns focus (source save.rs); initial helper assertion expecting
an immediate reload was wrong. Dismissed complete cancel dialogue, verified
stdout Loaded save(seed1), then normal reentry/reopened airmail20C0 correctly
(paper-reopen.png), closed with no stale flat/catalog roots. A helper retry
before dialogue dismissal could not reopen; corrected focus/timing, not code.
Initial helper also called non-exported qa.catalog; corrected local helper.

Owned40716/22276/30780 previews closed. Windows briefly retained EXE lock after
first stop; waited, copied again and verified hashes before final launch.
Real save only hashed, before/after unchanged:
C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
Final source check4.34s, clippy10.18s, fmt, capture30.63s, release5m56s pass.
Initial release was deliberately interrupted when reviewer caught paper ID bug;
only final release counts. Default build1m08s passes. No test suites.
No-disc/missing-asset guard reviewed, not separately replayed. Bounded captures
are not exhaustive visual verification of every design or Philip acceptance.
