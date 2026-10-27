# Shop spotlight rare item

2026-10-27 claude, Philip: "do what you think we should". Chosen because three completed notes
(shop-bargain-sales, shop-november-grab-bags, shop-seasonal-stock) record "rare_item exclusion awaits
the unimplemented spotlight stock field". Source: m_shop.c `Save_Get(shop).rare_item` (lines ~551, 598, 857, 1146ff).

- [x] Audit: how rare_item is chosen, shown (spotlight slot), sold/replaced, saved; what it excludes.
- [x] Implement as a disc-independent kernel where possible; save field with serde default.
- [x] Wire exclusions in bargain/grab-bag selection.
- [x] Temp QA; gates; docs/comms; release; commit/push.

## Implemented (2026-10-27, claude)
Source: `m_shop.c` mSP_MakeRandomGoodsList (rare rolled first from the rare list when the tier's RARE_FTR count is non-zero: Nookway and Nookington's), mSP_SelectRandomItem_New line 551 (no draw may pick `Save_Get(shop).rare_item`), mSP_ShopSaleReport (a sold rare becomes RSV_SHOP_SOLD_RARE), `ac_shop_design.c` aSD_MakeReaGoodsFg (RSV_SHOP_RARE 0xFE09, sold sign 0xFE15, not shown on sale-event days).
- `goods.rs`: `make_goods_list` takes `rare: &mut ItemId`, rolls it first (rare list via `select_items(.., Some(2))`), publishes it with `set_rare_item` and appends it to the day's stock after the ordinary furniture; `select_items` and the uncollected-first bargain pick skip it.
- `shop.rs`/`save.rs`: `ShopData.rare_item`, saved as `shop_rare` (serde default, omitted when 0), restored with the stock.
- `shop_display.rs`: the rare piece goes to its FE09 reserve and its sold sign is 0xFE15.
Purchase is the ordinary stock path (it is a stock item with a sold flag).
Verified (Temp, disc, tier-2 save): goods list shows violin, chalk board, bug zapper then the exotic chest (rare), the chest is drawn in the shop, F5 save has shop_rare 0x1324; no WARN/ERROR. Not re-driven: buying it and seeing the 0xFE15 sign; same-day stock saved before this change has no rare until the next restock; tiers 0 and 1 have none, as in the original.
Not done: `mSP_SetShopRareFurnitureChirashi` (the notice letter when the rare piece is a rare-list item); the leaflet text/paper path exists in the port but is not wired to this.
