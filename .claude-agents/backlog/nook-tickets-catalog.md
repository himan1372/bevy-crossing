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
- Raffle-day room (2026-10-23, grok): the fuku layout's three furniture spots show this month's prizes (sold spots show the sold-out sign). Nook runs to the drum (160, 140) and, after a win, to that prize. The drum (`cKF_bs_r_obj_gara`) sits at (160, 60, 190); at tick 46 the ball (`act_fukutama`) rolls out. Preview on 31 Oct: prizes dracaena, hamster cage and queen piece, Nook at the drum, drum model in place. The ball was not played out (fresh save, no tickets).
- Catalog screen (2026-10-23, grok): Nook choice 1 opens 0x10A6, then the `clg_*` window (9 tabs, 7 rows, collected items only). A priced pick is 0x10A7; confirm pays, files the order (shop level at that moment) and says 0x10AA. Price 0 (fossils, gyroids) is 0x10A8 and does not order. Escape is 0x10A9 and leaves bells and slots alone. Preview: blue bench 0x135c for 1900 (bells 80000 to 78100, slot 0), then a tricera skull refused, then Escape. Not played: 0x10AB (short on bells) and 0x10AC (five slots full). The left circle does not draw the item model. Raffle day still skips this talk.
