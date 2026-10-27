# Rare piece: buy it and check the sold sign

Claimed by claude 2026-10-27. Philip chose it after fish-spawn-field-rank.

Goal: drive a purchase of the spotlight rare furniture in a tier-2 shop (Temp, synthetic save),
check the stock flag, the FE09 display swapping to the 0xFE15 sold sign, persistence across
save/load and the next day's restock; fix whatever is missing. Source: `m_shop.c`
mSP_ShopSaleReport, `ac_shop_design.c` aSD_MakeReaGoodsFg.

## Result (2026-10-27, claude): verified, no code change

Temp, disc, synthetic tier-2 save (noon, loan 0): entered Nookway, stood on the unit in front of the
displayed owl clock (the FE09 rare spot, stock index 3, 1300 Bells), pressed E, Space, Enter (Yes).
Log `Nook purchase stock=3 item=0x1e70 price=1300`; Bells 80000 -> 78700, the owl clock landed in
the pockets, the stock entry was saved `sold: true` with `shop_rare` still 0x1e70 (as in C), and the
clock model was replaced by the SOLD OUT sign (0xFE15, `sold_sign(RSV_RARE)`) on that spot. After a
restart from that save the sign was still there and no clock was drawn. Restarting a day later
(DayOffset 1) restocked: a new spotlight piece (vibraphone), nothing sold, and a fresh rare notice
letter. No WARN/ERROR in the logs. Not driven: buying it with a full pocket or short Bells (the
ordinary stock paths).
Evidence/helpers: %TEMP%/bevy-crossing-room-critters/diary-grid-20261027 (b0..r0 screenshots).
Tip for QA: a first shop visit with a loan starts Nook's house talk automatically, and Space on its
Yes/No question orders a house expansion; stand on the unit in front of the piece and use a short
KeyW (20 ms) to face it before E.
