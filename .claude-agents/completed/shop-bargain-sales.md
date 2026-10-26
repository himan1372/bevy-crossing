# Scheduled bargain sales

2026-10-26, codex. Philip authorizes next seasonal shop task and requests
efficient narrow local-Qwen use with English, source verification and timing.

## Design and plan

Implement the native saved special-event scheduling slice needed for shop
sales, reserving other visitor event turns without inventing their actors.
Use native nonrepeat six-event selection, minimum-two-day gap, shop-sale date
exclusions, noon..19 start, one-hour sale and one-hour pre/post closure. Current
port lacks retail personal IDs and field assessment: stable town/resident seed
and an explicit default rank input are bounded adapters, not full calendar
fidelity. Persist dates/type/stock/announcement separately from normal stock.

Keep ShopData ordinary stock intact; expose selected stock through methods so
existing exact purchases/previews/selling operate on the active native scene.
Latch sale layout at town entry/update to avoid rebuilding under transactions.
Original sale FG, rare/uncollected-first stock, message scripts, leaflet and
first-talk balloon reuse disc/source paths. No guessed discounts or new UI.

- [x] Audit/claim and write bounded implementation.
- [x] Native saved calendar/offer/stock selection and leaflet.
- [x] Active stock/layout/doors/renovation/intro adapters.
- [x] Sequential check/clippy/fmt and capture/default/release; isolated runtime
  dates/hours/stock/purchase/refusal/persistence/floors/fallback. No tests.
- [x] Source review; task/parent/both roadmaps/comms, release, commit/push.

Qwen small check:6.06s, English-only, supplied only a five-branch Rust function
and requested one sentence about start14. It wrongly returned sale hours15/16;
direct comparison proves only14. Rejected. No further calls this task because
verification costs outweigh savings. Architecture/final review/validation stay
with Codex; one reused source agent supplies bounded native asset/UI evidence.

## Implementation and source review

Native references: m_event.c init_special_event; ac_event_manager.c
init_sp_bargain; m_shop.c mSP_ShopOpen; ac_shop_design.c event reserve mapping;
m_fg_type.h sale FG IDs. Calendar uses native six-event order, nonrepeat
selection, 2..7-day rank-dependent gap, date exclusions and event expiry.
Other visitor turns are reserved only. Rank0 and stable town/resident seed
substitute for unavailable assessment/personal IDs; sale-hour randomness uses
the same bounded seed adapter rather than native global RNG.

Offer persists independently of ordinary stock. Native category/day, count,
rare/uncollected-first selection and seasonal clothing reuse loaded disc lists.
Shared goods getters feed display, preview, fallback menu, quest names and
atomic purchases. Sale view changes in Town only, preserving an active scene.
Native FG163..173 and FE07 event reserves retain ordinary shell geometry.
Source prices and normal purchase tickets remain unchanged. Original leaflet
text/paper55, native welcome scripts and wrapped random balloon2244..224B
reuse existing systems; saved gift latch prevents reload duplication. Native
handover/escort choreography remains parent fidelity work. Existing spotlight
rare_item exclusion is still absent pending that separate system.

Runtime found and corrected two implementation issues: event stock initially
used ordinary reserve IDs (fixed to FE07), and restore scheduled renovations
without bargain lookahead (now passed centrally to schedule::advance both
on restore and in Town). QA-only corrections: distinguish shop/museum beds,
include required Letter fields in full-mailbox fixture, and copy the capture
binary before a default build replaces target/debug. No product workaround.

## Verification so far

Sequential final check3.11s, all-target warnings-denied clippy6.75s, fmt check,
capture30.58s and default29.27s pass. Final release4m18s passes. No tests
added/run. Temp-only synthetic runtime in bevy-crossing-bargains-20261026:

- Full16 tier/category entry/layout matrix, both department floors and original
  merchandise closeups inspected; single room root, Timmy present upstairs.
- Cancellation/100Bells/full15pockets preserve all41 save fields except position
  and playtime. Cranny bed2100:80000->77900, exact offer sold, ledger2100,
  bed1358 and ticket2C48; ordinary wardrobe stock untouched; F9 exact retention.
- Wrapped first-talk balloon, no charge/ledger change, F9 no duplicate.
- Generated offer and original dated leaflet persist. Full10-letter mailbox
  defers notice; freeing space retries exactly once; F9 no duplicate.
- Missing calendar in legacy save generates/persists; expired special event
  selects another type; controlled resident seed naturally schedules Oct28
  sale, valid hour/category/notice. Sale tomorrow blocks threshold renovation.
- Hours12/13/14/15/16 around start14: open/closed/open/closed/open.
- Uncollected rare wallpaper chosen first; all-collected fallback remains rare
  and distinct. Ordinary source stock stays separate.
- No ERROR/panic. Existing Tommy parent InheritedVisibility B0004 warning on
  stairs remains, as in earlier higher-tier work.

Real save only hashed, unchanged:
C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
Unrelated .gitignore and actual save stay unstaged. Parent/M4/counts/Philip
acceptance remain open. Latest user narrows Qwen to optional summaries or
explanations only; English required each prompt, no code review/check delegation.

Final runtime: Timmy upstairs buys the same2100 bed with exact offer/ledger,
ordinary stock untouched and normal ticket. Sale wallpaper preview/cancel
preserves all41 fields except position/playtime. True process restart retains
all sale/stock/pocket/ledger/wallet state; only known startup field_items delta
occurs. Missing-disc run retains all sale numeric fields (other existing
no-disc town/police/villager adapters differ). Hour16 returns ordinary bureau
1008 upstairs while retaining the sold sale offer. QA assertion initially used
an invented model filename from fixture label; native entity is sum_blue_bureau01,
confirmed and corrected. Owned preview closed. No further source edits after
final gates; release4m18s. Actual save hash remains unchanged. Completed bounded
implementation; original identity/rank/RNG inputs, spotlight exclusion, exact
handover/twin choreography and Philip acceptance remain explicit limits.
