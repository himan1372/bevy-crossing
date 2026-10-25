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
- Catalog screen (2026-10-23, grok): Nook choice 1 opens 0x10A6, then the `clg_*` window (9 tabs, 7 rows, collected items only). A priced pick is 0x10A7; confirm pays, files the order (shop level at that moment) and says 0x10AA. Price 0 (fossils, gyroids) is 0x10A8 and does not order. Escape is 0x10A9 and leaves bells and slots alone. Preview: blue bench 0x135c for 1900 (bells 80000 to 78100, slot 0), then a tricera skull refused, then Escape. Not played: 0x10AB (short on bells) and 0x10AC (five slots full). Raffle day still skips this talk.
- Catalog look (2026-10-23, grok): each page loads its own `clg_win_*` texture, so the window has the leaf pattern, the scalloped edge, navy name bars and a red selected bar. The left circle draws the selected furniture from the side (`mCL_item_draw`: a 0x900 look down, then a spin around the vertical). Preview: the blue bed lies across the ring and turns. Wallpaper, carpet, stationery and music stay empty in the circle.

## Bounded review fixes (2026-10-24, codex)

Philip authorized reviewing/fixing the pending art slice, then asking again.
Page scroll remains 0.3535 texels per 60 Hz frame and rotation 1.25 degrees
per frame. Removed the unsupported green-leaf pixel rewrite: clg_mwin_mode
copies TEXEL0 RGB with TEXEL1 coverage. The separate perspective camera uses
mSM_change_view's width256 branch (20 degrees, near100/far800).
Preview fitting now uses rendered hierarchy Aabbs and GlobalTransforms,
including joint-aware skinned bounds, so early-return specialised renderers
no longer stay hidden. Removed the incomplete LocalBounds addition to shared
furniture spawning. Inspected blue bed, blue wardrobe, retro TV and alarm
clock screenshots in a synthetic Temp preview; all visible and rotating.
Clock geometry has conservative bounds beyond its visible body, leaving it
smaller/higher in the circle; exact visible-coverage fitting remains open.
The parent task remains open/claude-owned. No remaining purchase/refusal/mail/
raffle scenarios were taken over by this follow-up.

## Remaining-path verification (2026-10-24, codex)

Purchase ticket0x10AE, full-pocket overflow0x10AF, insufficient Bells0x10AB,
full five orders0x10AC, actual paid order and restart ticket/order mail now
played with complete affected numeric snapshots. Both refusals preserved all
compared fields. Read both delivered Nook letters with correct attachments.
Raffle loss consumed exactly five tickets, and ball entity timing/motion ran,
but its rendered ball was4000x too small due to double model scaling. The later
fix is recorded in completed/raffle-ball-scale.md. Two renderer slab-allocation errors while
viewing delivered letters were later fixed in completed/mail-render-slab-errors.md.
See completed/catalog-raffle-verification.md for exact evidence/setup limits.
Parent remains open; non-furniture previews, exact clock fitting and Philip
acceptance are not closed by these checks.

## Raffle presentation follow-up (2026-10-24, codex)

Ball scale fixed by retaining the existing world-scaled mesh under a unit-scale
parent. Philip additionally reported floating drum and hat clipping: raffle
objects now use the shell's vertical origin, seating drum at tabley0.5, and
the shop blocks the original posed drum footprint including its tray overhang.
Player front stop moves fromz-2998.59985 to-2998.38110; held-forward recheck
stays there. Final inspected screenshots show seated drum, visible rolling
ball and front clearance. Actual loss consumes exactly5 tickets and preserves
wallet/conditions/catalog/prizes/stock/room/mail; scene cleanup/reentry checked.
See completed/raffle-ball-scale.md for gates/evidence. The previously recorded
allocator error also occurred during a winning-prize display refresh; the later
fix is in completed/mail-render-slab-errors.md. Parent ownership/acceptance unchanged.

## Empty text renderer follow-up (2026-10-24, codex)

Blank letter lines and markup-only raffle reveal prefixes submitted zero-vertex
indexed meshes. Shared text builder now returns None for no glyph geometry;
dialogue, pockets/mail, catalog and music-grid callers skip allocation/spawning.
Both baseline error paths reproduced with producer logs, then same letter and
actual third-prize win replayed without allocator errors. Letter saved-state
comparison and five-ticket/wrapped-prize/sold-slot assertions pass. Source review
and compiler/lint/format gates pass; no tests. Parent remains open for missing
non-furniture previews, exact clock fitting and Philip's acceptance.

## Flat catalog previews (2026-10-24, codex)

Selected follow-up catalog-flat-previews adds original mCL flat overlay models
for wallpaper, carpet, stationery and music. Source scales0.54/0.54/0.28/0.55,
wall paletteA and floor paletteD with textures8-B; paper design modulo64 matches
catalog20C0..20FF IDs. Music shares original note art with 60Hz bob/sway/tilt.
Lazy model cache retains assets across selection changes, existing catalog root
owns cleanup. Original furniture perspective renderer and order logic unchanged.
Inspected multiple patterns/ruling/no-ruling papers and music screenshots;
navigation/cancel preserved complete compared numeric fields and music motion
was sampled. See completed/catalog-flat-previews.md for final verification.
This supersedes earlier empty-preview notes. Parent remains open for exact
clock fitting and Philip's acceptance; ownership unchanged.
