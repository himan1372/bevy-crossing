# Music discs, fans and pinwheels as room goods

2026-10-18, codex. Philip authorizes next bounded work ("nxt").
Audit original 55 minidisc IDs and eight fan/eight pinwheel IDs, indoor models,
rotation flags and common ITEM1 room placement. Extend the existing loose-goods
path, preserving exact item/condition data, storage hiding and numeric save/load.
Music registration/playback and holding/using/acquiring toys remain separate.
Use existing runtime disc models and transaction/presentation kernels. No tests
without request; check/clippy/fmt/builds and bounded disposable GUI evidence.
Preserve actual save/settings/.gitignore/handoff. M4/migration/acceptance open.

- [x] Source audit and implementation.
- [x] Validation, task/roadmap/comms evidence and release claim.

## Source and implementation

Original m_name_table.h defines55 minidiscs2A00..2A36 (exclusive END2A37), eight
pinwheels224C..2253 and eight fans2254..225B. ac_shop_goods_data.c_inc supplies
identical shop/indoor obj_mdT_mat_model+obj_mdT_gfx_model for discs with flags0,
individual obj_shop_kaza1..8 models for pinwheels, and individual utiwa1..8 mat/gfx
pairs for fans, all toys flags1. Source musical items use the original gold-note
model, not a new physical-disc image. All models remain runtime disc content.

mTG_room_put_proc calls mRmTp_Item1ItemNo2FtrItemNo_AtPlayerRoom with
no_convert_tools=TRUE. These families stay ITEM1 and use ordinary room drop;
the minidisc registration/music overlay uses a separate mTG_putin_proc branch.
Reviewed that distinction and the original name boundaries/model flag entries.
Existing/pinned furniture logic needs no additional family-specific kernel.

Shared loose support now adds exactly two bounded ranges,224C..225B and
2A00..2A36. Existing model selection, normal-condition pocket action, floor/
surface placement, atomic pickup/carry/turn/save and transient presentation are
reused. Source yaw helper already turns secondary toys and keeps disc facing
fixed while their positions orbit. Storage whitelist/music registration/audio
engine and toy holding/use/acquisition are unchanged. No new assets/save fields.

## Bounded evidence

Workspace check, all-target clippy -D warnings, format and capture build pass;
no tests added/run. Final default/release results recorded below. Local Ollama
unavailable earlier this session; direct source/shared-callsite review used.

Disposable Temp/bevy-crossing-music-toys-review, own preview27580:

- Mixed square table4532 carries first disc10752, last disc10806, bluebell fan8788
  and yellow pinwheel8780; floor K.K. Love Song10787 also renders. Left turn saves
  base4533 and permutes the tops to34fan/35last-disc/50first-disc/51pinwheel.
  Observed toys have90-degree yaw, discs identity yaw. Original models inspected.
- Push moves the rotated table/tops34/35/50/51->18/19/34/35 and switch512->2;
  pull restores exact room/pocket/condition/wallet state. F9/reentry/F5 preserves
  the complete numeric state. Runtime secondary-facing reset on reload remains
  the existing source SG_angle_y behavior; no angle save fields added.
- All five mixed/floor items pick up exactly once, retaining exact track/toy IDs,
  condition512 and wallet12345. Normal pockets placement puts boundary discs
  10752/10806 at secondary50/51; pinwheel8780 and leaf fan8795 use floor67/52
  fallback when surfaces are occupied. Source drop/pickup presentation completes.
- Separate galleries render all eight pinwheel variants and all eight fan
  variants. Two views of each inspected; eight original roots each confirmed.
  Both gallery saves preserve complete numeric room/pockets/conditions/wallet.
- Full15 pockets refuse disc pickup, fully occupied room refuses fan placement,
  with exact room/pocket/condition/wallet preservation. Stored last-disc10806,
  leaf-fan8795 and fancy-pinwheel8787 in bureau4196's three owner layers remain
  hidden (zero loose roots) and numerically intact on save.
- All55 track IDs share one audited range/model; the GUI used first/last and
  track35, not55 individual playtests. No music-player registration/playback or
  toy-use playtest is claimed. No WARN/ERROR in preview log. Screenshots inspected;
  all synthetic saves/helper scripts/evidence remain Temp-only.

Own27580 closed before final builds. Actual-save SHA256 unchanged:
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.
Settings/.gitignore/handoff preserved. No fresh no-disc GUI cohort; shared numeric
fallback unchanged. Other item/furniture behavior and detailed movement/contact
limits remain open. Philip playtest acceptance pending; M4/migration open.

Final default workspace build passed (25.34s), then release (2m56s). Claim
released; no own preview/Cargo remains. Actual-save hash still matches baseline.
