# Rare-furniture notice letter (mSP_SetShopRareFurnitureChirashi)

Claimed by claude 2026-10-27. Philip chose it after letters-retail-grid.

Source: `m_shop.c` mSP_SetShopRareFurnitureChirashi (1143) and mSP_ShopItsumoChirashi (1093):
the first play of a day (after 06:00, before the shop closes), not on the month's last day, not on
the renewal day, not on a bargain day, mails the "rare furniture in stock" handbill
(`leaflet::rare_chirashi_template(tier, type&1)`, free string 7 = item name, paper 55) when the
spotlight item or any furniture in stock is on the rare list. The check's day is saved (goki.time).

## Implemented (2026-10-27, claude)

Kernel `logic/src/leaflet.rs`: `rare_notice_gate` (month's last day, renewal day and bargain day
save the play time without a leaflet; same day, or the next day before 06:00 after an earlier
save, or hour < 6 / > close hour wait; otherwise check the stock), `rare_notice_close_hour`
(22, 23 for Nook 'n Go, 22, 22), `FIELD_RENEW_HOUR`. Template `rare_chirashi_template(tier, 1)`
was already ported.

App: `goods::is_rare_furniture` (mSP_SearchItemCategoryPriority, furniture/rare list under the
shop's priorities); `shop/schedule.rs::rare_notice` in the town tick: spotlight piece first, else
the first rare furniture in stock, mails Nook's handbill (free string 7 = item name, paper 55)
if the mailbox has room (the play time is saved either way, as in C). The play time is saved as
`shop_progress.played` (y, m, d, hour; serde default).

Verified (Temp, disc, synthetic tier-2 save, noon): the owl-clock spotlight piece produced one
Nook letter "Today's special in home decor is none other than the amazing owl clock!" with the
Nookway footer on paper 55, `played` saved; restarting the saved state sent no second letter;
no WARN/ERROR. Not driven: the full-mailbox, month-end, bargain-day and renewal-day skips (pure
gate, reviewed against the source), the in-game reading of the letter. Legacy saves with no
`played` receive one notice on the first eligible day, as a new game does in the original.
