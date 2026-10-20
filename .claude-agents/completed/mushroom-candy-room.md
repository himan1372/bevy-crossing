# Mushroom and candy room goods

2026-10-18, codex. Philip authorizes the next bounded slice ("nxt").
Audit original indoor models and flags for food0x2805/0x2806; extend the existing
supported loose-item path to mushrooms/candy only. Preserve item IDs, placement,
pickup, carrying/turning, source presentation, conditions, storage and save/load.
No eating/seasonal acquisition or other item families. Use existing kernels and
runtime disc assets. No tests without request; check/clippy/format/builds and
bounded disposable preview. Preserve actual save/settings/.gitignore/handoff.

- [x] Source audit and implementation.
- [x] Validation, roadmap/comms evidence and release claim.

Philip acceptance is pending; M4/migration remain open.

## Implementation and bounded evidence

Original m_name_table.h defines food0x2805 mushroom and0x2806 candy, followed by
coconut0x2807 and exclusive FOOD_END0x2808. ac_shop_goods_data.c_inc uses identical
shop/indoor display lists item_matutake_modelT / obj_shop_candy_modelT, both with
flags0. Existing model lookup, source0x128 non-fruit landing sound and normal ITEM1
room/storage routing already support these IDs. Pinned furniture kernel adds no
special rule to port. Local Ollama was unavailable earlier in this session; no
repeat retry, direct source/shared-callsite review used.

Centralized loose support now includes all eight food IDs0x2800..2807, preserving
the existing lower/upper bounds. No rendering, animation, storage whitelist,
inventory, mail, selling or food-use rule changed. Placement/pickup/push/pull and
all supported turns inherit the existing atomic transactions and presentation.
No new assets, item grants, eating or seasonal acquisition added.

Workspace check, all-target clippy -D warnings, format and capture build pass.
No tests added/run. Final default/release build results recorded below.

Disposable Temp/bevy-crossing-mushroom-candy-review, own preview2668:

- Both original models inspected on long country table4512. Right turn around
  far endpoint moves base34->19 (4515), mushroom10245->19 and candy10246 stays35,
  switch512->4. Both goods retain identity yaw (source flags0) while positions
  follow the turn. F5/F9/reentry/F5 preserves all room/pocket/condition data.
- Surface pickups add exactly one10245 and one10246, clearing their secondary
  cells while retaining table. Placement from normal pocket0 puts mushroom at
  secondary35; pocket1 candy falls back to floor51 because that surface is full.
  Source drop/pickup presentation completes. Full IDs and condition32 retained;
  numeric room/pockets match exactly across another F9/reentry/save.
- Push carries both goods34/35->18/19 and switch512->2; pull returns all numeric
  room layers/switches/pockets/conditions exactly to the original fixture.
- Full15 pockets refuse pickup, and fully blocked room refuses candy placement;
  complete numeric room/pocket/condition equality confirmed after each refusal.
- Stored mushroom/candy/diary in three owner layers of bureau4196 stay hidden
  (zero loose roots) with exact numeric save preservation.
- Screenshots inspected; log has no WARN/ERROR. Synthetic saves/helper scripts
  and evidence remain Temp-only. Own2668 closed before final builds.

Actual-save SHA256 unchanged:
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.
Settings/.gitignore/handoff preserved. No fresh no-disc GUI cohort; shared numeric
fallback unchanged. Other loose families and detailed existing movement/animation
limits remain open. This does not certify retail food behavior or close M4 or
migration. Philip playtest acceptance remains pending.

Final default workspace build passed (22.01s), then release (2m57s). Claim
released; no own preview/Cargo remains. Actual-save hash still matches baseline.
