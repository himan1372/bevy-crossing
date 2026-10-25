# Selling warnings and disposal

Philip selects priority1 after895ca05b; conserve tokens, at most one subagent.
Design: original quest/present warning and junk disposal conversations extend
existing marked/slot-bound sales. Preserve cancellation, exact conditions,
atomic payments, protected items and separately scoped turnips.

- [x] Audit original message/condition ordering; implement in sales.rs,
  shop_nook.rs and pockets.rs without new save fields or renderer framework.
- [x] Root review; sequential check/clippy/fmt/capture/default/release.
- [x] Synthetic Temp QA warnings accept/cancel, junk disposal and mixed marks,
  unchanged real-save hash, no tests. Use existing isolated helpers.
- [x] Task/parent/both roadmaps/comms, release claim, commit/push.

One implementation agent owns the three source files; root owns QA/docs/builds.
Native behavior defines the design; no new product decisions are needed.

Source/design findings:
- Original ac_npc_shop_common check_buy single maps quest ->57 refusal and
  zero-value ->56 free disposal; all-zero plural ->216. Disc message bodies
  confirm56/216 ask Shall I take it,58 accepts and59 cancels. Root corrected
  the agent's initial follow-up mapping after reading the actual disc bytes.
- Wrapped presents have no separate warning in the audited selling route.
  They sell as captured slots, excluded from normal duplicate expansion.
- Quest bit2 in any captured offer refuses the whole batch. This deliberately
  prevents the original plural path bypassing single-item quest protection.
- Known zero-price goods become selectable; unknown IDs, currency, cards,
  knife and separately scoped turnips stay blocked. Mixed priced/junk batches
  quote only paid value. All-junk disposal has no ledger/register/bag side effect.
- No new save fields. Every commit validates exact items/conditions; discard
  stages pockets before mutation. One implementation agent hit its usage limit
  after writing the core diff; root completed and reviewed it, no second agent.
- Original pockets pre-transfer quest-warning window presentation remains in
  the existing special-tag/fidelity follow-up; Nook's refusal/protection is here.

Disposable QA: selling-warnings-20261025, explicit ISO/copied capture EXE and
synthetic15-slot save only. Quest single/batch refusal and wrapped cancellation
compare all39 fields except playtime/position unchanged. Wrapped slot3 sale
pays100, clears only slot3/its condition, preserves normal/quest duplicates.
Junk250E disposal clears only slot0, wallet5000/ledger0 unchanged; cancellation
preserves all39 fields. Marked250E/250F removes both with no payment; mixed
junk250E/apple2800 removes exact slots0/2, pays100/ledger50, preserves others.
Quest refusal and actual final Shall I take it choice screenshots inspected.
Initial disposal-cancel QA selected stale hidden choice text; tightened driver
to require the actual question and reran disposal/cancel/batches successfully.
No production change was needed for this driver error. F9 fixture reloads pass.

Check31.68s, warnings-denied all-target clippy7.26s, fmt, capture46.84s and
default1m01s pass. No test suites. Root source review completed; only one agent
used. Roadmap HTML/five-priority checklist and diff checks pass. Parent/M4 and
Philip acceptance remain open; tracked counts/parent ownership unchanged.

Release6m26s passes. Normal duplicate All regression pays200/removes only normal slots0/1; wrapped/quest duplicates unchanged. Missing-disc plain pockets inspected, numeric IDs retained. Disc log no ERROR/WARN/panic; missing-disc expected warning only. Owned27196/12116 previews closed. Real save SHA256 unchanged C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA; unrelated .gitignore/save unstaged.
