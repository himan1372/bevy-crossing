# Turnip prices and selling

2026-10-26 codex. Philip authorizes the next retail service: original daily
turnip prices, price questions and turnip selling, including quantity/spoilage.
Qwen optional explanations/summaries only; no call planned. At most one source
subagent, read-only native dialogue/spoilage lookup; Codex owns implementation.

## Design and plan

Reuse existing shop disc dialogue/pockets/atomic Sale transactions. Port the
native m_kabu_manager weekly trend generator with saved dates/daily prices,
not later games' AM/PM patterns. Resolve original quantity, Sunday refusal,
spoilage and mixed sales from source before changing callers. Retain source
UI/scripts and explicit legacy/no-disc save behavior. Joan actor/buying and
password service remain separate unless required to make this slice correct.

- [x] Native source audit and bounded price/spoilage/transaction design.
- [x] Saved native price schedule and date adapters.
- [x] Original Nook/Timmy price inquiry and turnip sale flow.
- [x] Relevant spoilage/legacy behavior and native quantity checks.
- [x] Sequential Rust gates/builds, isolated synthetic runtime comparisons.
- [x] Final review, task/parent/bothroadmaps/comms, complete/commit/push.

No tests requested. Real save must only be hashed; preserve unrelated
.gitignore and untracked savegame.json. Actual save baseline from prior slice:
C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.

## Source decisions and implementation

Original m_kabu_manager.c is the authority: seven daily prices, prior-trend
transition odds, Sunday70..129, random/rising, Mon..Fri8x spike and falling
trend. Float order, truncation and the unusual minimum clamp are retained in
logic::turnips. Existing PC RNG supplies bounded draws; this does not recreate
native global RNG state. app/shop/turnips saves prices/trend/update Sunday and
turnip-specific growth clocks, defaulting absent legacy fields. Same-week
prices persist; native Sunday startup renews non-Sunday prices while retaining
Sunday price. Weekly midnight and date-reversal bounds match lb_rtc.c.

Original Other things menu now exposes Turnip Prices and return, using native
message309/25/36 and free1. Password choices remain hidden. Existing source
scripts and Timmy bank mapping are retained; normal Sell/catalog/cancel remain.
Sale staging prices IDs2F00/01/02 as10/50/100 times the current daily price;
2F03 is zero-value disposal. Exact-ID duplicate groups exclude quest/present
conditions; one means one bundle. Mixed marked sales sum each chosen bundle.
Fresh turnips refuse Sunday, after quest refusal; marking excludes the whole
turnip category on Sunday, as m_tag_ovl.c. Zero spoiled single selection still
uses disposal, including Sunday. Confirmation revalidates slots/conditions,
Sunday and captured turnip price before atomic existing payout/Bell-bag logic.

Native m_all_grow_ovl.c defines Sunday06:00 spoilage and backward-date spoilage.
Turnip-specific Town growth hook updates pockets, field cells, every house
floor/layer and active storage contents; retains conditions and other IDs.
Existing spoiled outdoor items clear on the next forward growth pass before
fresh ones spoil; spoiled possessions persist. Legacy saves lacking an age
clock adopt the first observed day (age cannot be recovered), preserving
numeric items. Other players, island/gyroid storage and Joan buying are absent
systems, not invented by this slice. No changes to generic world growth,
clock UI, overall save version or external assets.

One source subagent supplied exact dialogue/quantities/spoilage references and
reviewed the small pricing/date slice. Review found lbRTC equality includes
Sunday midnight and native old spoiled-ground clearing; root checked/fixed both
before capture builds. Final code/UI/transactions are reviewed by Codex.
Qwen not called; optional explanations/summaries would add no value here.

## Validation in progress

Final check1.82s, all-target warnings-denied clippy21.61s, fmt, capture54.04s
and default46.39s pass. Release running. No tests added/run.
Temp-only synthetic runtime: bevy-crossing-turnips-20261026.

- Original Other things menu and123Bells price inquiry visually inspected;
  inquiry preserves all42 fields except position/playtime.
- Bundle10/50/100 payouts1230/6150/12300, exact pocket slots, half-payout
  sales ledger and untouched merchandise. Cancellation42fields unchanged;
  spoiled-turnip disposal free, no ledger credit.
- Duplicate all/one removes only exact10-turnip bundle slots, excludes wrapped
  and quest slots, retains other bundle variants/conditions and unrelated state.
- Quest refusal42fields unchanged. Legacy missing-market save creates native
  schedule; F9 and F6 next-hour preserve it, forward/backward week repair works.
- Monday growth clears old spoiled field item without spoiling Sunday's fresh
  purchases; backward-date change spoils all exposed pocket/field/house layers
  while retaining other goods and conditions.

## Remaining runtime evidence and corrected fallback

- Mixed10+50+fruit+spoiled sale pays7480, ledger3740, exact four slots;
  other fields unchanged. Wrapped single bundle pays1230, clears only its own
  condition. Normal overflow creates one30000Bell bag, wallet82250; capacity
  refusal with1000Bells/unit and15full pockets preserves all42 fields.
- Sunday05 keeps fresh pockets/field/all house-floor layers; Sunday06 spoils
  them and clears old spoiled ground first. Sunday14 repeats this and source
  price/sale refusal with42fields unchanged; M creates no turnip mark. Next
  Sunday price update is generated. Timmy inquiry/1230 sale pass, screenshot
  inspected with original echo text. No AM/PM price reset is introduced.
- True process restart retains prices, sale, pockets/conditions, wallet, stock,
  catalog and ledger; known unrelated startup field_items population remains.
- Initial missing-disc launch exposed a real required-FieldItems panic: the
  fallback mode does not create that resource. Changed tick to optional field
  access, retaining pocket/house/market updates without it. Final no-disc run
  preserves prices, fresh2F01/present condition, house2F00, wallet/ledger/stock.
  Disc Sunday06 all exposed storage/field checks rerun and pass after the fix.
- QA-only followups: comparing separate fresh fixtures' catalog structures
  incorrectly compared randomized lottery prizes; source purchases award no
  tickets and per-case full comparisons pass. No-disc RoomSave comparison
  initially omitted its default music_box field; numeric layers/switches and
  all affected gameplay fields match. Waiting helper from crashed preview was
  identified/closed before rerun; no competing automation remains.
- Final fixed runtime has no ERROR/panic. Expected missing-disc warning,
  unmapped gamepad warning and existing Tommy InheritedVisibility B0004 during
  department stairs are recorded, not changed by this task.

After optional-field fix: check2.62s, clippy5.23s, fmt, capture23.85s and
(default)23.29s pass. Final release rebuild is pending output. No tests.
Owned previews closed. Real save still only hashed; preserve .gitignore/save.
Native global RNG/other players/island/gyroid systems and Joan buying remain
separate; legacy age adoption and per-type Town growth adapter documented.

Final review also moved Kabu_manager date refresh ahead of the Town-only growth
guard: native m_time updates weekly prices indoors too. Isolated Saturday22
Nook'n Go entry plus two F6 hours crosses Sunday00 while staying inside:
week changesOct18->Oct25, prices renew, pocket/wallet and growth seen-date stay
unchanged. This passes on final capture. Final gates after this ordering fix:
check2.41s/clippy5.00s/fmt/capture10.42s/default9.78s; release pending.
No further source changes planned. Owned final midnight preview closed.

Publication scope: native daily pricing/selling is complete; this does not
complete Joan's visitor/buying service or Philip acceptance. Final source review
checked kernel arithmetic/RNG call order, Sunday/update-date comparisons,
original price messages/slots, duplicate bundle versus unit counts, protected
conditions, atomic wallet overflow, all exposed storage spoil paths and
no-disc resource availability. Source/UI behavior and remaining adapters are
recorded above; no blanket M4/migration completion. Keep counts unchanged.

Final release3m09s passes on final source. Check2.41s/clippy5.00s/fmt/capture10.42s/default9.78s pass; no tests added/run. Owned previews/helper closed; actual save SHA256 remains unchanged. Completed bounded service, source/final review and documentation; publishing explicit files only.
