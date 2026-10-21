# Nook's lottery tickets and catalog

Philip 2026-10-20 chose "Nook's tickets & catalog": lottery tickets from
purchases (and their mail overflow), playing the raffle, and catalog
ordering at Nook's with the orders delivered by mail
(special_delivery.rs; rustimal catalog.rs).

## Stage 1 (committed 2026-10-21, claude)

- logic `collection.rs` (mPr_SetItemCollectBit bitfields); `special_delivery` pub.
- app `catalog.rs`: `Catalog` (collection, TicketOverflow, CatalogOrders, raffle lineup) in `SaveData.catalog`; collect bits from normal pocket items; start-of-play ticket/order letters (0x057 / 0x049+level, paper 55) into the house mailbox.
- `shop_nook.rs`: qualifying purchases give a ticket (0x10AE) or mail it (0x10AF).
- app `raffle.rs`: last day of the month, monthly lineup (an unheld item first), 0x10D1 ask -> 0x10D4 take five -> 3 s drum -> win 0x10D9-DB + 0x10E3 (prize wrapped, slot sold) + 0x10DD/0x10DC, or loss 0x10D5 + 0x10D7/0x10D6.
- Verified in a temp preview (Oct 31 at noon, two stacks of five tickets): ask, a win (prize 0x3274 given), another go, and a loss.
- Verified 2026-10-21: seven waiting October tickets arrive at start-up as two letters 0x057 (stacks 0x2C4C and 0x2C49).
- Not verified: the ticket coming with a purchase (my test script never reached a display), order mail at start-up.
- Not done: the prize display and drum/ball/Nook walk on the raffle day (normal goods still show); the catalog screen and ordering (Nook choice 1, 0x10A6-0x10AC, 9 tabs, clg_* models, 5 slots).
