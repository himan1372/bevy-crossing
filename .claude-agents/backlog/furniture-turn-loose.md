# Turn loaded single-cell and square surfaces

Philip authorizes loose items rotating with single-cell/square furniture. Audit
source relative offsets/angle flags, preserve atomic destination validation,
save data and existing push/pull. Keep loaded long-piece turns refused. No new
contact pivots or other item families. Source persistence semantics must be read.

- [x] Claim/source audit and implementation.
- [x] Check/lint/fmt/build and bounded disposable GUI; no tests added/run.
- [x] Evidence separate from Philip acceptance; release claim.

Preserve save/settings/.gitignore/codex-handoff.md; M4/migration remain open.
Actual-save baseline26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.

Implementation: original GetItemPosOnMovingFurniture rotates fitted offsets by
the base angle delta; UnFit restores cells and SG_angle_y. Goods rendering applies
flag1 (aSG_DATA_ROTATE_Y_FLAG) on layer1 only; fruit flags0 stay unrotated while
their positions orbit. Shared goods-entry lookup exposes this source flag without
changing shop model selection. Supported TypeA/TypeC offsets quarter-turn around
the same centers as native top actors; native clearance/destination/owner/player
validation stays in place. Copied layers and angle table publish together.

Transient quarter-turn angle state follows source runtime SG_angle_y (not saved
foreground data), persists through in-process movement/refresh and clears on app
restore/F9 or fresh process. Numeric item positions and base angle remain saved;
there is no save-format addition. Placement/pickup reset secondary-cell angles.
This does not promise loose-facing persistence after reload or original F9
behavior (F9 is this adapter's reload control). Loaded TypeB turns remain refused.

Evidence: workspace check, all-target clippy -D warnings, format and capture build
pass; no tests added/run. Temp/bevy-crossing-turn-review with synthetic saves:
TypeC country table4532 left turn saves4533; cells34/35/50/51 become
paper8387/coral9499/diary11008/coconut10247. Paused mid-turn screenshot and root
transforms show orbital positions, matching table/diary/paper/coral quaternions
and identity coconut quaternion. Push/pull preserves90-degree loose angles; right
turn returns exact original room/pockets/condition/switch data. F9 retains saved
permuted cells/base angle while loose-facing resets. Mixed tape+three goods turn
together: tape7768->7769 at cell50, switch512->131072; base switch512 retained.
Single-cell box turns its diary in place; pickup/replacement resets its model yaw
to identity. Long country table4512 and unsupported mushroom top refuse, preserving
complete numeric room/pockets/conditions. Screenshots inspected; all evidence,
synthetic saves and helper script remain Temp. Own preview25604 closed for builds.

Ollama unavailable; original source and existing/pinned helpers reviewed directly.
No separate no-disc/fresh-process GUI repeated. Numeric no-disc save path unchanged;
actual save hash matches baseline. Settings/.gitignore/handoff preserved. Exact
TypeB pivots, sweep/root/contact fidelity and other item behavior remain open.
Philip acceptance pending; M4/migration remain open.

Final default workspace build and release build (-j1, 2m38s) pass. Preview log
has no WARN/ERROR entries. No tests added/run; actual-save hash unchanged.
