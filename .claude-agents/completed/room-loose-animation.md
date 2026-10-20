# Supported loose room item presentation

2026-10-18, codex. Philip authorizes the next bounded slice ("nxt") after
long-piece turning; this does not certify earlier playtest acceptance.

Audit original placement/pickup player clips and item presentation for supported
diary/paper/fruit/shell room goods. Reuse runtime disc models/clips and existing
atomic inventory/layer transfers. Preserve quantity, conditions, storage hiding,
surface height, save/reload and exclusive input. No tests without request.
Use check/clippy/format/build gates and disposable preview evidence.

- [x] Source audit and bounded implementation.
- [x] Validation, task/roadmap/comms evidence and release claim.

Preserve actual save/settings/.gitignore/handoff. M4/migration remain open;
Philip playtest acceptance is separate.

## Implementation

Reviewed mTG_room_put_proc/common_throw_put_room and Shop_Goods_Actor's
player_drop_entry/drop_entry/drop_move/drop_draw/destruct; player_common pickup
routing, m_player_main_pickup and pickup_jump, player_draw's single_draw path,
sound IDs and the original left-hand callback. Current/pinned logic has no goods
drop or loose pickup presentation kernel to reuse. Existing gameplay transfers
remain authoritative; app presentation adds no alternate item rules.

New loose_presentation state has source pickup1 for floor goods and clear_table1
for surface goods (0.5 clip frame per60Hz tick). Floor goods stay at source until
timer20 then shrink/approach the actual rendered left hand through timer40.
Surface goods stay through frame11, approach the hand through16, then shrink
through30. Secondary goods yaw retains the source flag behavior; floor yaw is0.
Floor frame10/20 ITEM_GET/rustle and surface-start0x430 use existing sound engine.
Each temporary model uses the original goods display lists, not a furniture leaf.

Placement uses the source idle player plus item flight: feet+50 units, initial
velocity from the source28-tick formula, -1.2 acceleration, counter/30 horizontal
interpolation, source growing scale, quarter-speed bounce capped3 and second
contact completion. Fixed30Hz (dt2) integration keeps this discrete source
presentation stable across display frame rates. Source0x127 landing sound applies
to five ordinary fruit; coconut and other supported goods use0x128.

Loose placement now returns the chosen Piece so presentation knows its real
floor/surface target and height. Inventory/layers still commit atomically before
presentation. F5/pause Save always snapshots the committed result; F9 or scene
departure discards presentation. New state participates in existing exclusive
room focus and manipulation checks; no second transfer can compete with it.
Pockets closing gates placement playback, using the accepted renderer's existing
world_ready gate. Normal meshes regain exact scale/height when the flight ends.
Pickup models are separate temporary roots, cleaned after completion/reload.
No save fields, extracted content or new supported item categories added.

## Bounded evidence

Workspace check, all-target clippy -D warnings, format and capture build pass.
No tests added/run. Final default/release builds also pass (42.86s / 2m57s).
Local Ollama executable unavailable in this session; source reviewed directly.

Disposable Temp/bevy-crossing-loose-animation-review, own preview33144:

- Starter-style diary11008 on box12536: original clear-table pose, full model at
  surface(2999.5,0.5,2999.5), then hand-bound shrink. Observed hand-stage scale
  0.0001233757 vs full0.00025; item position follows rendered hand. Numeric
  transfer adds exactly one diary, leaves supporting box and condition32 intact.
- Paper8387 (four-sheet bundle) placed on that surface: observed flight scale
  0.0000706314 at y1.5414, then pause Save/F9 restored exact item at secondary34
  with pocket0 empty. Backspace returns complete8387 to pocket0, retaining box.
- Coconut10247 floor placement:40 sampled poses show growing arc, peak y1.6571,
  bounce and exact final(3000.5,0,3000.5), identity yaw/full0.00025 scale. Floor
  pickup1 then shrinks into hand; mid-stage scale0.0001983524 observed. Paused
  mid-pickup Save/F9 restores exact coconut to its pocket and clears floor51.
- Floor coral9499 pickup and diary11009 pickup both transfer exactly once.
  Combined I/Backspace/G during an active clip did not open another transaction;
  all temporary goods roots gone after completion. Mid-diary F5/F9 preserves
  empty numeric room and exact pocket11009; condition32 retained.
- Box+diary left turn still works; pickup preserves surface quaternion
  (0,0.70710677,0,0.70710677) before shrinking, rather than resetting its yaw.
- Full15 pockets refuse pickup; fully occupied floor/surface refuses placement.
  Both preserve every numeric layer/switch/pocket/condition. Initial reload was
  attempted before a refusal notice released focus, causing an unconfirmed
  entry; excluded and repeated after settled notice dismissal/confirmed entry.
- Blue-bureau three-layer stored diary/paper/coral stay hidden (zero loose roots),
  with the complete numeric room unchanged on save. Screenshots inspected;
  helpers/synthetic saves/logs remain Temp-only. No WARN/ERROR in preview log.

Preview33144 closed with escalation before final builds. Actual-save SHA256 still
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.
Settings/.gitignore/handoff preserved. No fresh no-disc GUI cohort; numeric save
and fallback path unchanged. This is a bounded empty-hands/one-item adapter:
marked multi-drop, retail movement during flight, detailed root/contact/part-table
fidelity and other loose families remain open. Source sound events implemented;
Philip listening/playtest acceptance pending, M4/migration remain open.

Final default workspace build passed (42.86s), followed by release (2m57s).
Claim released; actual-save hash unchanged and own preview/Cargo closed.
Philip acceptance remains pending; M4/migration remain open.
