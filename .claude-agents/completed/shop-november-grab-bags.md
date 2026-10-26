# November grab bags

2026-10-26, codex. Philip authorized the next seasonal task: native November
grab-bag sale, purchase and opening. Use local Qwen when useful; report its
performance and request English in every prompt.

## Design and plan

Extend existing stock, exact purchase flow and pockets UI. Sale is the day
after November's fourth Thursday; bag2E00 costs the calendar year. Ordinary
furniture/shirts/carpets/walls remain; paper/diary/tools/paint/sign/plants are
replaced with bags, total6/9/13/18 by tier. Use original reserve order and
explicit saved stock indices, including across the two department floors.
Preserve legacy same-day stock and save version1.

Opening requires three currently empty slots in addition to the bag. Select
three rare-priority ABC goods (seasonal clothes), equal category chances and
one possible pinwheel with50% probability. Exclude earlier picks. Stage all
changes before committing; keep transient shrink/staggered-grow presentation
and modal input ownership separate from committed save state. Wrapped bags
unwrap before their contents can be opened. Unsupported other bag type stays
unsupported. Refusals leave numeric state intact; no-disc opening refuses safely.

- [x] Source audit/claim and focused Qwen assessment.
- [x] Stock/date/year-price/reserve mapping and purchases.
- [x] Atomic contents, capacity warning and transient pocket presentation.
- [x] Sequential check/clippy/fmt, capture/default/release builds and isolated
  runtime purchase/refusal/opening/reload/legacy/date/floor checks. No tests.
- [x] Review, task/parent/roadmaps/comms, claim release, explicit commit/push.

Qwen first call:393seconds for an8-bullet C summary. Capacity/categories were
correct; duplicate handling was wrong (ignored the exclusion arguments to
mSP_SelectRandomItem_New). Its output is advisory, not source authority. Root
continues directly because this call did not save time/tokens overall.

## Source and final behavior

Source root: `C:/Users/phili/game c/shmanimal crossing/ACGC-PC-Port`.
`m_shop.c::mSP_Chk_HukubukuroSail` selects the day after the fourth November
Thursday. `mSP_MakeRandomGoodsList` and `mSP_MakeGoodsList` retain ordinary
furniture, clothing, carpet and wallpaper; paper/diary/tool/paint/sign/plants
become bag pool6/9/13/18. No umbrella is added. `mSP_ItemNo2ItemPrice` prices
2E00 at the full calendar year. Existing same-day lineups stay intact until
normal restock; no new save fields/version.

`ac_shop_design.c::aSD_MakeHukubukuroFg` orders reserves FE06,FE0C,FE0B,FE00,
FE05,FE08, then FG scan order per kind. Runtime-disc fgdata inspection confirms
capacities6/9/13/18/0 for the five layouts. Nookington's eighteen are all
downstairs (3tools,1sign,1paint,5paper,3sapling,5plant reserves). Per-unit stock
and sold flags preserve exact purchased cells on floor switches/reloads; source
uses a shared remaining count and reconstructs reserves. A sold bag always
draws SOLD_PLANT even when its reserve originally held a tool or paper.

`ac_npc_shop_common.c` has no bag-specific branch: offer30/success34,
short33/full39 and no lottery ticket use the existing exact stock transaction.
`m_tag_ovl.c::mTG_hukubukuro_open_proc` scans three EMPTY slots before bag
removal, chooses one candidate position for a50% pinwheel, and otherwise rolls
equal clothing/carpet/wall/furniture categories with rare ABC priority and
earlier-item exclusions. Short lists retain the native duplicate fallback.
The RARE and LIST_C enum values both equal2, so the native condition DOES
apply the seasonal clothing subset; a review inference to the contrary was
checked against header values and withdrawn. Runtime pools verify this path.

The new opening helper stages all four affected slots/conditions before one
inventory assignment. The bag's old slot becomes empty; three gifts occupy the
original empty slots with normal conditions. Existing catalog possession hooks
mark exactly the new collectible items. Wrapped2E00 unwraps first; quest state
cannot open contents. Other bag types are not silently treated as November bags.
Missing disc/lists refuses safely without consuming the bag.

`m_inventory_ovl.c` gives a70tick timer: bag shrink remaining/12, grow scales
1-(remaining-14-i*14)/12, clamped0..1. `m_submenu_ovl.c` category14 points to
six hukuro texture/palette pairs indexed by min(elapsed,5). UI presentation
uses these runtime assets at30Hz and blocks input/closing until settled;
pause gates freeze it, load/audio-reset and exit clear transient state.
Capacity warning uses original three-line words in the existing dialogue
window, preserving pockets until dismissed; full mWR warning-panel geometry
is a remaining presentation limit. Source rare_item exclusion awaits the
existing unimplemented spotlight stock field. RNG uses a local adapter stream,
not a claim of bit-exact retail random state.

## Verification

No tests added or run. One reused source/review agent found no blocking final
issue, supplied read-only disc list/FG evidence in Temp. check3.85s,
warnings-denied all-target clippy9.32s, fmt, capture48.89s, default38.14s and
release4m26s passed. Initial check caught new field/method visibility and an
unused wrapper, corrected before final gates. No profile/dependency edits.

Evidence/helpers only in `%TEMP%/bevy-crossing-grab-bags-20261026`:

- All four tiers: exact6/9/13/18 stock units, price2026, no replaced ordinary
  categories, original bag displays counted/screenshots inspected. Department
  upstairs0, return downstairs18, original ordinary upstairs service retained.
- Cancel,100Bells and full15 pockets preserve all40 saved fields except player
  position/playtime. Purchase80000->77974, exact one bag/stock sold, ledger2026,
  no ticket/mail/other mutation. F9 and true process restart retain the purchase
  and five remaining Cranny displays. True restart reproduces the known
  unrelated empty field_items->20 startup population, recorded not fixed.
- Capacity0/1/2 refuses without mutation; capacity3 fills exactly slots1/7/13
  and clears bag4, preserving other pockets/conditions/Bells/shop/mail/room.
  Expected catalog bits from gifts change through the existing collection hook.
- Six further openings: every output belongs to the runtime rare list selected
  by saved shop priorities; clothing uses the winter subset. All three items
  distinct, maximum one pinwheel; four of six draws had a pinwheel. This is
  bounded evidence, not a statistical proof. Wrapped bag unwrap preserves its
  item and clears only its wrapping condition before contents can be opened.
- Native icon growth/shrink captures inspected; Escape and movement during
  opening are blocked, player position unchanged, completed contents persist
  across F9. Capacity wording screenshot inspected. A QA helper initially
  assumed the cursor reset on F9; corrected it to read selection and navigate,
  then reran all capacity cases. The first successful numeric check also
  exposed expected new catalog bits; assertion tightened to exact collection
  changes rather than incorrectly requiring an unchanged collection.
- November26/27/28/30 in2026 checks ordinary/sale/ordinary/raffle precedence;
  November26 in2027 checks date rollover and price2027. Legacy ordinary stock
  on the new sale date preserves its exact saved lineup and sold flags.
- Explicit missing-disc launch preserves bag/pockets/conditions/Bells/stock/
  ledger/catalog when Open is attempted; fallback screenshot inspected. Only
  the expected missing-disc warning occurs in that session.

Runtime logs: no ERROR/panic; one department stair visit emitted existing B0004
for npc293/Tommy's parent missing InheritedVisibility. That NPC hierarchy was
not changed here; its warning/fidelity remains open. Full warning panel,
spotlight rare-item support, bargain calendar/stock/layouts and festive candle
rendering remain unfinished. Parent/M4/migration/Philip acceptance stay open.

All owned previews closed. Actual save SHA256 remained unchanged:
C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
Roadmaps/parent/comms updated without gameplay item-count or acceptance changes.
