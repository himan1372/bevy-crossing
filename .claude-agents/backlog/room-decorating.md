# Wallpaper and carpet decorating

Philip authorizes wallpaper/carpet decorating after umbrella/tool room support.
Audit source mTG_cover_proc/mTG_carpet_proc and aMI_ChangeWall/ChangeCarpet:
replace selected pocket item with prior installed standard pattern, including
full pockets, and change runtime disc textures after menu closes. Preserve exact
room/inventory/storage/music state; carpet also selects original footstep data.

Use the existing small player room and JSON persistence with optional compatible
pattern fields. Legacy/missing data resolve disc defaults; missing-disc saves
retain numeric selections. Custom patterns and basement/room expansions remain
separate. Include loose wallpaper/carpet rolls using existing goods path.

No tests requested. No actual-save automation. Actual save baseline SHA256:
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.
Use disposable Temp saves/previews, preserve .gitignore/settings/handoff. Read
project workflow and relevant previously loaded Rust/Bevy/disc/UI/save/QA/build
skills; validate check/clippy/fmt/capture/default/release, update both roadmaps
and root comms, release ownership and publish finished task work.

## Implementation and source audit

- Source m_tag_ovl.c:mTG_cover_proc/carpet_proc replaces the chosen pocket slot
  with aMI_ChangeWall/Carpet's returned previous pattern. No free-slot search is
  needed. Both native bounds are 0..70: FLOOR_PLAYER_ROOM_END/WALL_ETC_END71,
  including four shop patterns. Disc forest_2nd.arc banks are583904/293088 bytes,
  exactly71 blocks of0x2020/0x1020. Original custom pattern banks use private
  designs and separate flags; this slice does not enable them.
- Added decorating module using the existing room/inventory transaction. Checks
  slot/condition, valid standard prior/new patterns and disc banks/models precede
  any mutation. Pocket receives the previous pattern; wallet, other slots,
  conditions, room layers/storage/switches/music collection stay independent.
- Numeric transaction commits immediately. Transient pending state owns Furniture
  focus and blocks movement until PocketAssets::world_ready. Original0x11B sound
  and new shell wait for submenu closure, matching aMI_Change2ReservedWall/Floor.
  Same-pattern operations retain the shell and still finish cleanly. Pending
  state is not serialized and resets on load.
- PlayerRoomShell marks only the player shell. Replacement reuses existing disc
  geometry/segment texture decoding and removes the old root/children, without
  respawning furniture or touching collision/other interiors. New IndoorFloor
  uses the selected carpet index, so existing SE_FLOOR_DATA lookup follows it.
  No new audio/render/save framework or extracted assets.
- Optional home_furniture.decoration={floor,wall} retains JSON version1. Missing
  data resolve original disc defaults; explicit empty furniture remains empty.
  Missing-disc and unsupported numeric decoration values remain saved, rather
  than discarding the whole save. Unsupported prior values refuse replacement.
- Standard roll IDs route through existing loose goods/models/yaw/presentation;
  goods table supplies rolls0..66. Shop patterns67..70 decorate normally, but
  their loose placement is refused when no goods-table model exists.
- No matching indoor/room-type kernel exists in the pinned Rustimal module tree;
  this integrates original C behavior into the existing Bevy room adapter.

## Bounded runtime evidence

Temp bevy-crossing-decorating, owned preview3572: first/middle/last floor and
wall patterns0/35/70 swap exact old item into the selected slot and survive F5/F9.
Full pockets accept floor35 followed by wall66 without changing other pockets;
same-floor35 operation is complete numeric identity. Three hidden storage layers
and unrelated conditioned worn axe remain intact. Original floor/wall textures
and menu labels inspected in screenshots. Loose floor/wall rolls place and pick
up as the exact IDs without changing installed patterns; roll image inspected.
Further edge/fresh-process evidence and final checks follow.

Source footstep index wiring is verified; no audible retail acceptance claimed.
Custom designs, expanded rooms/basement, new seasonal acquisition and per-type
furniture/diary writing remain separate. Philip playtest/M4/migration stay open.

Cancel, conditioned and reserved71 menus preserve complete numeric state; town
menu has no decorating action. Unsupported floor255/wall72 remain serializable
and refuse replacement without item mutation. Legacy missing room restores
source furniture/default patterns; explicit empty room retains empty layers.
Repeated changes/reloads retain exactly one player shell. First unsupported
warning overflowed; shortened with explicit line breaks before final verification.
No gameplay transaction failed; no tests added/run.

Fresh disc process29292 restores floor66/wall35 plus exact furniture/inventory
from startup (no F9 first). Subsequent change preserves nonzero music_box[1,1]
and confirms a replaced shell with pockets fully closed. The first subsecond
sampling assertion arrived after the menu had already closed; excluded as a
sampling assumption, rechecked settled shell/menu transform and numeric state.
Source world_ready gating establishes the timing rule, not an exact frame claim.
Corrected two-line unsupported warning inspected and refusal equality repeated.
Both disc logs contain no WARN/ERROR.

Fresh no-disc process32040 F5 preserves installed floor66/wall35, nonzero music
collection, rotated umbrella, three hidden storage layers and complete pocket/
condition/wallet state. Placeholder screenshot inspected. All three owned
previews closed before final builds. Actual save SHA256 matches the starting
baseline; unrelated .gitignore/settings/handoff preserved. All helpers/screenshots
and synthetic saves remain in Temp. No tests added/run.

Final check, warnings-denied all-target clippy, fmt and capture build pass
(final capture12.22s); default/release results follow. Static roadmap unique IDs/
anchors and diff checks pass:75 items42 implemented6 progress27 planned. This is
not a game-completion percentage; no new browser-layout verification claimed.
Final default build passes43.54s; release build passes2m38s. All required checks complete; no tests added/run. No own preview or Cargo process remains. Release claim and publish authorized task source/docs; Philip acceptance pending.
