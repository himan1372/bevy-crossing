# Seasonal shop stock

2026-10-25, codex. Philip selected remaining seasonal shop services and allows
up to one subagent when it saves tokens. One read-only source audit agent.

## Design and implementation plan

Use the existing daily stock vector, exact display indices, runtime disc art,
original dialogue and atomic purchase flow. Halloween candy is ordinary
merchandise, not a new service menu. Preserve save version 1 and unrelated state.
Direct source audit is preferable to a new event framework for this bounded slice.

- [x] Audit Halloween, December, November grab bags and bargain dependencies.
- [x] Implement native October16..30 candy/flower quantities and reserve mapping
  in goods.rs/shop_display.rs, retaining exact sold-slot persistence.
- [x] Check existing December substitutions against original source.
- [x] Verify sequential check/clippy/fmt, capture/default/release builds and
  isolated synthetic runtime purchases/refusals/reload/date boundaries.
- [x] Review diff; update task, parent, both roadmaps and append root comms;
  complete/release claim, commit and push only task files.

No tests added or run. Actual save is hashed only; previews use Temp cwd and
synthetic saves. Preserve unrelated .gitignore/savegame.json. Bargain calendar,
event-specific rooms and November grab-bag contents must not be invented or
declared implemented merely because an item can be displayed.

## Source audit and representation

Native source root is `C:/Users/phili/game c/shmanimal crossing/ACGC-PC-Port`.
`src/game/m_shop.c::mSP_CheckHallowinDay` (2011) uses October16..30 inclusive.
`mSP_SelectPlant` (985) replaces flower_count with sapling_count and clears
saplings while saving the old flower_count as the candy pool. Tier candy
counts2/3/4/5, flower counts1/1/2/3 come from l_*_goods (790ff).
`src/actor/ac_shop_design.c::aSD_ItemName2ReservePointName` (109) moves flowers
to FE05; `aSD_MakePlantFg_hal` (149) fills FE08 with candy2806.
`src/data/item/food_price.c` prices candy at100; original generic offer30,
success34, short33 and full39 already map correctly through shop_nook.rs.
`aNSC_check_item_with_ticket` excludes candy. No new dialogue is invented.

The adapter stores each candy in the existing StockItem vector, preserving
exact stock indices/sold flags across F5/F9 and restart. Native uses a shared
remaining count and repacks its reserves on reconstruction; exact sold-slot
persistence is the existing Bevy convention. Presence of candy, including
sold units, selects Halloween flower mapping. Legacy same-day lineups without
candy retain ordinary mapping and are never replenished by this change.

`mSP_SetSeasonFTR` (149..173) matches existing December IDs/order:1EBC/1E88
through24, ordinary25,3298/327C from26. Raffle display takes precedence on31.
No December generation code changed.

Remaining services are deliberately recorded as unfinished:

- November sale is the day after the fourth Thursday. Grab bag2E00 costs the
  current calendar year (`mSP_ItemNo2ItemPrice`,662), pool6/9/13/18 fills
  FE06/FE0C/FE0B/FE00/FE05/FE08 in that order. Opening requires three free
  pockets, selects three rare goods with duplicate exclusions and possible
  pinwheel, then removes the bag (`m_tag_ovl.c`,5093ff). A stock-only bag would
  leave the service unusable, so its stock and opening must be implemented together.
- Bargains depend on saved event start/end/kind/items, one-hour pre/post
  closures (`mSP_ShopOpen`,1791ff) and category-specific sale rooms/stock
  (`aSD`,212ff). The event calendar is not yet connected in this port.
- December26 runtime emits existing missing-model warning for iam_yaz_candle
  (3298). Numeric candle/flag substitutions pass; candle rendering remains open.

## Verification

One reused source-audit/review agent found no blocking issue in the final diff.
check2.50s, all-target warnings-denied clippy5.88s, fmt, capture31.50s,
default25.00s and release4m17s passed. Initial all-target compilation caught
six pre-existing test callers needing the new bool argument; signatures were
updated, no tests added or run. No dependency/profile/toolchain edits.

Disposable evidence/helpers only in `%TEMP%/bevy-crossing-seasonal-20261025`:

- All four tiers regenerate candy2/3/4/5, flower1/1/2/3, no saplings, all100Bells.
  Original candy meshes counted; room/closeup screenshots inspected.
- Cancel,50Bells and full15 pockets compare all40 saved fields unchanged
  except position/playtime. Purchase80000->79900, one candy, exact one sold
  stock flag, ledger100, no ticket/mail/other state mutations.
- F9 retains the sold spot and one remaining candy; true restart retains all
  affected fields. Restart also reproduces the known unrelated empty
  field_items->20 startup population from the handoff, recorded not fixed.
- Second candy costs100, both exact stock slots become sold; F9 retains both
  signs, no candy reappears. Native question and sold-out screenshots inspected.
- October15/16/30/31,November1 and December24/25/26 numeric stock/room checks
  pass. October31 uses raffle drum rather than ordinary candy merchandise.
  December helper initially assumed obj_s_shop2; changed only the Temp helper
  to recognize obj_w_shop2 and reran December checks successfully.
- Legacy October15-style stock assigned today's stock_day preserves all
  numeric/sold slots and ordinary display mapping on Halloween; no free restock.
- Explicit missing-disc launch retains numeric stock/sold flags; plain pockets
  screenshot inspected. Only the expected missing-disc warning appears.

Halloween runtime logs have no WARN/ERROR/panic. December candle model warning
is recorded above. No claim of Philip visual/audio acceptance or M4 completion.

All owned previews closed; actual save SHA256 unchanged:
C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
Parent retail-shop-ui, M4/migration and Philip acceptance remain open.
