# Clothing room furniture conversion

Philip's nxt selects the roadmap's remaining room item families, starting with
clothing. Original mRmTp_Item1ItemNo2FtrItemNo_AtPlayerRoom converts shirts
0x2400..0x24FE to mannequin furniture indices0x1EB..0x2E9;0x24FF is the mannequin
sentinel and excluded. Pickup reverses all four furniture rotations to the exact
shirt. Reuse existing mannequin renderer, native furniture placement/movement,
save/transaction and source icon presentation. Audit before changes.

No wearing/custom design/umbrella/tool conversion in this slice. No tests or
actual-save automation; preserve unrelated .gitignore, settings and handoff.
Actual save starting SHA256:
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.
Use Temp synthetic saves/evidence only. Check/clippy/fmt/capture/default/release,
update roadmap/comms, release claim and publish authorized completed work.

## Implementation and source audit

- Original m_room_type conversion range is exclusive ITM_CLOTH_END24FF;
  FTR_CLOTH_START/END cover mannequin000..254 including four directions.
  Added narrow native clothing_to_furniture/clothing_from_furniture helpers in
  logic::furniture; no umbrella/tool/custom-design migration in this slice.
- App normal-condition pockets offer Place in room. Conversion happens before
  the existing runtime model/profile guard and native placement search; consume
  original shirt only after success. Existing actor renderer binds each shirt's
  runtime tex_boy/pallet_boy banks to mannequin segments8/9.
- Pickup maps the standard mannequin back to its shirt before the existing
  cloned-pockets capacity/owner-layer transaction. Orientation never changes the
  shirt identity. Storage contents, native room numeric representation and
  movement kernels remain unchanged.
- Birth passes the converted furniture ID to existing presentation, so native
  furniture_icon selects aMR_ICON_CLOTH2, as original aMR_ItemNo2IconNo. Pickup
  already passes its furniture ID and retains that source clothing icon.

## Bounded evidence

Temp bevy-crossing-clothing-room preview35448: first2400/middle2480/last24FE
Place -> original mannequin/texture -> exact shirt pickup; first F5/F9 complete
home_furniture/pockets/conditions/wallet equality. Gallery source textures
visually inspected. Rotations West/North/East recover exact IDs; live push/pull
roundtrip preserves complete room, live turn/F9/pickup recovers2480 once. All four
orientations covered collectively with normal placement.

Three-layer stored shirts24E2/24E3/24E4 under blue wardrobe4144 stay hidden/intact;
unrelated conditioned shirt24A0/condition32 preserved. Full-pocket pickup and
fully blocked placement preserve complete numeric state. Reserved24FF and
conditioned shirts show Grab-only menus, screenshots inspected. Initial blocked
helper reloaded before dismissing prior refusal focus; excluded/repeated after
explicit dismissal. Disc log contains no WARN/ERROR. No tests added/run.

Fresh missing-disc preview30124 uses synthetic gallery, F5 preserves all numeric
mannequin orientations, stored shirts, pockets/conditions/wallet. Placeholder
art inspected; log contains only expected missing-disc warning. Both own previews
closed before final builds. Actual save hash remains the starting baseline;
settings/.gitignore/handoff preserved, evidence/helpers/saves remain Temp-only.

Check, all-target clippy -D warnings, fmt and capture build pass (35.03s).
Default build passes (28.43s); release build passes (2m38s).
Implementation and required checks for this slice complete; no own previews,
browser or Cargo left running. Release claim and publish authorized source/docs.
Browser roadmap73 items40 implemented6 progress
27 planned; remaining33 excludes implemented, search/status work,1280/390 widths
have no overflow. Desktop screenshot inspected and browser closed. Roadmap now
puts clothing first in latest work, umbrellas/tools next; new acceptance pending.
