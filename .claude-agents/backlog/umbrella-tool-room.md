# Umbrellas and tools in rooms

Philip authorizes the next room-item slice. Audit original m_room_type and
m_tag_ovl: standard umbrellas2204..2223 convert to furniture indices342..361;
custom umbrellas2224..222B require player patterns and remain excluded. Room
placement passes no_convert_tools TRUE: tools remain loose goods, preserving
normal/golden and worn-axe identity rather than using shop furniture conversions.

Use existing room placement/pickup, disc models, movement/presentation and saves.
Preserve Hold for supported net/rod, storage rules and atomic refusals. No new
held-tool actions, custom designs, assets or save fields. No tests requested.
Use Temp synthetic previews only; actual save starting SHA256
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.
Check/clippy/fmt/capture/default/release, record runtime evidence and limitations,
update both roadmaps/comms, release claim and publish finished work.

## Implementation and source audit

- Original m_tag_ovl.c:mTG_room_put_proc passes TRUE to the room conversion
  helper; normal/golden tools and worn axes must stay ITEM1 goods. The FALSE
  conversion to tool furniture is for other callers (including shops), and
  would erase worn-axe identity. Native pinned furniture.rs contains no matching
  item-conversion helper; retain its existing Bevy movement/storage adaptations.
- Added native umbrella forward/reverse conversion, indices342..361 and all
  four directions, alongside existing clothing conversions. Room placement,
  eligibility, birth icon3 and pickup use the shared supported conversion entry
  points. Existing runtime aFU_data/obj_shop_umb01..32 rendering is reused.
- Original ac_myfumbrella uses private_data[player].my_org textures/palettes;
  custom2224..222B are excluded until pattern support exists. Sickle222C and
  paint IDs remain outside the new ranges. No invented custom appearance.
- Loose support adds2200..2203 and2239..2243. Existing goods table provides
  normal/golden tools (including rod line lists); worn axes share axe geometry
  but retain their item IDs. All these entries carry the source surface yaw flag.
- Home net/rod menus retain Hold alongside Place in room/Grab. Hold dispatch
  now resolves its index from the displayed choices, also preserving conditioned
  tool menus and existing fans/pinwheels. No new held-tool action or appearance.
- Existing cloned-pocket transactions, hidden storage, per-layer occupancy,
  animation, movement/save kernels and missing-disc numeric fallback remain
  unchanged. No save fields, extracted assets or tests added.

## Bounded runtime evidence

Disposable Temp bevy-crossing-umbrella-tools, owned preview12676:

- First/middle/last standard umbrellas place from pockets, preserve unrelated
  conditioned inventory, save/F9 and pick up as the exact original IDs. Original
  textured models inspected. Live push/pull exact room roundtrip, turn/reload and
  last-umbrella pickup pass.
- Four normal/four golden tools plus worn-axe IDs223D/2240/2243 place/pick up
  exactly as loose items. Worn axe surface placement/pickup passes; mixed normal,
  golden and worn tools ride a table turn and save/F9 without item loss.
- Full-pocket umbrella/tool pickup and completely blocked placement preserve
  complete home_furniture/pockets/pocket_conds/bells equality. Three hidden
  owner-storage layers remain intact. Custom first/last, sickle and conditioned
  umbrella/tool menus preserve numeric state; inspected Grab-only custom menu.
- Net/rod Hold in the second home-menu row equips the expected HUD tool and
  preserves complete numeric room/inventory state. Menu screenshots inspected.

These are bounded development checks, not Philip playtest acceptance or full
retail fidelity. Custom patterns, new held-tool actions and broader room item
families remain separate. Net appearance/title/rename remain deferred.

All four saved umbrella orientations recover exact first/middle/last identities.
Normal/golden table galleries inspected, including both rod line lists. Disc
preview log contains no WARN/ERROR. Fresh no-disc preview34332 preserves complete
room (including rotated stands and hidden storage), pockets/conditions/wallet on
F5 with zero asset profiles; placeholder scene inspected. Screenshot remains in
the main Temp evidence folder. Actual save SHA256 matches the starting baseline.
Both verified own previews closed before final builds; evidence/helpers and
synthetic saves stay in Temp. No tests added/run.

Workspace check, warnings-denied all-target clippy, fmt check and capture build
pass (final capture17.73s). Final default/release build results follow.
Default workspace build passes (24.91s). Roadmap static unique-ID/anchor checks and diff check pass;74 items41 implemented6 in progress27 planned, not a game-completion percentage. No new browser-layout claim for these text-only roadmap edits.
Final release workspace build passes (3m27s). All required gates complete, no tests run. Runtime acceptance remains for Philip. Release claim and publish authorized task source/docs.
