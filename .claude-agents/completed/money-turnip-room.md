# Money bags and turnip bundles in rooms

2026-10-18, codex. Philip authorizes next bounded work ("nxt").
Audit original money/turnip IDs, item routing, indoor display lists and flags.
Connect the existing loose item placement/pickup/carry/turn/save path, preserving
each denomination/quantity/spoiled ID, pocket conditions and wallet balance.
Keep original turnip storage exclusion and money landing sound. No wallet
redemption, turnip trade/aging, acquisition or unrelated item families.
Use runtime assets/existing transactions, no tests without request; check,
clippy, format/builds and disposable preview evidence. Preserve actual save,
settings/.gitignore/handoff; M4/migration/Philip acceptance remain open.

- [x] Source audit and implementation.
- [x] Validation, notes/roadmap/comms and release claim.

## Implementation

Original m_name_table.h defines money0x2100..2103 as1000/10000/30000/100 Bells,
and turnips0x2F00..2F03 as10/50/100/spoiled. No extra per-item quantity field is
needed for these retail IDs. mRmTp_Item1ItemNo2FtrItemNo_AtPlayerRoom leaves both
families as ITEM1, and retail Player_actor_putin_item/putin_item_layer2 transfers
the exact item into a pocket. The local PC_ENHANCEMENTS money auto-wallet path
is a separate extension, not the retail behavior adopted here. PC-only expanded
money IDs beyond2103 remain unsupported.

ac_shop_goods_data.c_inc uses identical indoor/shop bag_DL_mode+bag_DL_vtx for
all four bags and item_kabu_modelT for all four turnips, both flags0. Original
shop_goods drop sound is0x444 for these money IDs,0x128 for turnips. Existing
normal ITEM1 storage whitelist permits money but excludes the turnip category,
matching mSM_check_item_for_furniture; this change does not alter it. Existing
and pinned furniture kernels provide no additional family-specific rule to port.

Central loose supports adds exactly these two four-ID ranges. Shared model,
normal-condition pockets action, floor/surface placement, atomic pickup, native
carry/turn and transient presentation do the rest. Money landing sound branch
now matches source. Wallet balance, amounts, bundle/spoiled IDs and condition bits
remain separate and intact. No acquisition, redemption, trading, daily/weekly
turnip spoilage, ant spawning or other item families added.

## Bounded evidence

Workspace check, all-target clippy -D warnings, format and capture build pass;
no tests added/run. Final default/release results recorded below. Local Ollama
unavailable earlier in this session; source/shared-callsite review used directly.

Disposable Temp/bevy-crossing-money-turnip-review, own preview7604:

- Original bag/turnip models inspected in gallery and mixed table views. Four
  denominations on square table and four floor turnip IDs recover into pockets
  as8448/8449/8450/8451 and12032/12033/12034/12035, respectively. Numeric room,
  pockets, condition32 and wallet12345 match across F9/reentry/F5.
- Initial gallery10-turnip stand was at the room edge; excluded as contact proof
  and repeated from a normal interior position with floor item12032 at34.
  Pickup leaves floor empty, adds exact12032 and retains wallet/conditions.
- Pocket0 30000-Bell bag8450 places at secondary50 of the square table; pocket1
  100-turnip bundle12034 falls back to floor66 when that surface is occupied.
  Exact IDs persist; both source drop/pickup presentations complete, wallet stays
  12345. Sound event0x444 implemented; no claim of Philip listening acceptance.
- Long table mixed bag8450/turnip12034 push34/35->18/19 changes switch512->2;
  pull restores complete room/pocket/condition/wallet data exactly. Far-end right
  turn moves base34->19, bag->19, turnip remains35, switch512->4. Both goods
  preserve identity yaw (source flags0) while their positions follow the table.
- Bureau storage menu inspected: normal100-turnip bundle offers Quit only;
  30000-Bell bag offers Put in/Quit. Storing it retains100-Bell bag8451 in layer1,
  adds8450 in layer2 at owner34, leaves turnip12034 in pocket1 and wallet12345.
- Full15 pockets refuse floor money pickup without redeeming it or mutating any
  numeric room/pocket/condition/wallet state. Fully blocked turnip placement also
  preserves all four. Stored bags remain hidden rather than rendering as tops.
- Screenshots inspected; log has no WARN/ERROR. Synthetic saves/helpers/evidence
  remain Temp-only. Own7604 closed before final builds.

Actual-save SHA256 unchanged:
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.
Settings/.gitignore/handoff preserved. No fresh no-disc GUI cohort; shared
numeric fallback unchanged. Philip acceptance remains pending, M4/migration open.

Final default workspace build passed (23.28s), then release (2m59s). Claim
released; no own preview/Cargo remains. Actual-save hash still matches baseline.
