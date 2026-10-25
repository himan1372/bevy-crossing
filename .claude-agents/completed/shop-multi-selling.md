# Marked and duplicate-item selling

Philip selects multi-item selling after paint (HEADbb3243b8).
Original mTG sell/all, mSM inv_mark_model, aNSC check_buy_item/buy_sum_check/
buy_check define this slice. Existing supported normal positive-price items;
quest/present warnings and zero-value disposal remain separate source flows.
Turnip service stays separate. Never make unsupported items silently sellable.

Design/inline plan:
- [x] Add transient selling marks (M) and selected-mask handoff, native Sell All
  label when selecting a marked slot; clear marks on unmarked pick/cancel/reset.
- [x] Draw original inv_mark_model at marked slots with source40tick colour
  pulse; retain fallback selection visibility and other UI modes.
- [x] Dedicated sales.rs captures exact slots/items/conditions/replacements and
  checked total. Single duplicate selection asks native15 all/one, followed by
  offer27; batch uses native214. Paper duplicates aggregate sheets; one sells
  one sheet from the actual selected stack. Batch sells complete marked stacks.
- [x] Commit staged pockets/wallet/bags atomically only on final yes; prefer
  sold slots for overflow bags, refuse if remaining capacity cannot fit them;
  validate every offered slot. Preserve ledger policy and unsupported conditions.
- [x] Check/clippy/fmt/capture/default/release; no test suites.
- [x] Temp QA marking/Sell All, batch accept/refuse/cancel/reopen,
  duplicates one/all and paper, overflow/capacity, protected slots/unrelated state,
  reload/no-disc. Real save only hash before/after. Independent review.
- [x] Notes/parent/both roadmaps/comms, complete ownclaim, explicit commit/push.
  Parent/M4/Philip acceptance remain open.

Implementation and source review:
- Original m_tag_ovl sell_proc/sell_all_proc and select-item mark clearing;
  m_submenu_ovl mSM_set_dl_item inv_mark_model and40tick green prim/env pulse;
  ac_npc_shop_common aNSC_check_buy_item_single/plural, check_buy_paper,
  set_buy_sum_str, buy_sum_check and buy_check.
- Native15 asks all/one;27 prices the selected quantity;214 quotes a marked
  batch;28 pays wallet and38 explains overflow bags;200 refuses no capacity.
  The native unit category symbol is a relocated pointer table, not flat data.
- Sale snapshots exact item/condition/slot. All eligible same-design paper
  stacks contribute sheets; one-sheet removal affects the actual selected stack.
  No new save fields. UI marks/picks are transient and do not mutate inventory.
- Final independent read-only review found no actionable issue after correcting
  unit-pointer lookup and preserving insect0x2D eligibility (turnips are0x2F).
  Commit stages every replacement and bag before publishing pockets and wallet.
  Stale-offer validation is source-reviewed; not forced through BRP.
- This bounded service still excludes presents/quests, currency, exercise cards,
  knife, zero-value goods and turnips. Their source warning/disposal/turnip
  conversations remain roadmap work; parent retail-shop-ui and M4 stay open.

Disposable gameplay evidence (2026-10-25):
- Temp multi-sale-b5c5b6a11b0e43929385b29127355c1f only, copied capture EXE,
  explicit ISO, synthetic save. Final batch sale marks slots0/2, native green
  rings and Sell All inspected: payout625, wallet5000->5625, sales312 under
  existing ledger policy. Exact unmarked duplicate, wrapped/quest slots,
  conditions576 and all unrelated saved fields remain unchanged.
- Cancel compares all39 saved fields except playtime/position unchanged.
  Duplicate Just one removes selected slot1 only for100; All removes normal
  slots0/1 for200, preserving same-ID wrapped/quest slots3/4.
- Paper stacks20C0/2040 offer six sheets. Just one pays10 and leaves2080/2040;
  All pays60 and clears both. The initial QA expectation used the purchase
  price40 per sheet; original pricing/code confirms sell10, corrected and rerun.
- Wallet99950 plus marked625 yields70575 and one30000 bag2102 in freed slot0,
  slot2 empty. Full pockets with99990 and a partial paper sale refuse atomically:
  all39 fields except playtime/position unchanged, including the full stack.
- Native duplicate/paper count choices and marked Sell All screenshots inspected.
  F9 is used between scenarios through the normal reload path; stale offers and
  exact pulse values remain source-reviewed rather than forced runtime mutations.
- Final source gates: workspace check4.12s, all-target warnings-denied clippy8.65s,
  fmt check, capture build (final incremental confirmation0.90s), default28.95s,
  release5m33s. No test suites added or run. Final independent source review and
  diff/HTML checklist checks pass; documentation does not change tracked counts.
- Mark controls reject wrapped/quest/currency/ticket/exercise/knife slots;
  butterfly/fish mark successfully (screenshot inspected), selecting unmarked
  shovel clears previous marks, M toggles off, cancel/reopen clears marks.
  Both cancel/reopen snapshots preserve all39 fields except playtime/position.
- Final disc log has no ERROR/WARN/panic. Explicit missing ISO shows the plain
  pockets fallback with all numeric item IDs retained; screenshot inspected,
  only the expected missing-disc warning. No claim of missing-disc Nook flow.
- Owned previews19240/8184/33048 closed through verified EXE identity. Actual
  save only hashed, unchanged SHA256
  C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
  Unrelated .gitignore and actual save remain unstaged. Philip acceptance open.
