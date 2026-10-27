# Password service leftovers: Famicom, magazine, mini card-e, letter grid

Unclaimed. Left from completed/shop-password-service.md and
completed/shop-password-send.md (codec and flow already in place).

- [x] Famicom (type 0), magazine (type 3) and mini card-e (type 5) codes redeem.
- Retail character-grid entry window and receiver window artwork.
- Hand-over animation and sound timing for presents (item arrives at message end).
- Foreign-visitor refusal (321) once visitors exist; Timmy's department counter
  follow-up links untested.

## Implemented (2026-10-26, claude)

`shop_nook/password.rs::check` now follows `aNSC_pc_check_password_*` for every
type. Famicom: item must be one of the fifteen `pswd_famicom_list` furniture
indices facing south (`ftr_idx_2_item_no`), hit rate 1, names match -> 315,
someone else's -> 324. Mini card-e: hit rate 1 -> 320, no name check. Magazine:
`PW_MAGAZINE_HIT_RATE` [80,60,30,0,100] against a 0-100 roll from a small
xorshift kept in `PasswordUi` (not the original's global RNG state): win -> 317 and
the item, loss -> 322 and no item, rate above 4 -> 313. Card-e keeps 318. All
presents go through the same session limit of three and the present condition.

Verified (Temp, disc, synthetic save; no tests) with codes from the scratch
generator for QA/Temp: other-name Famicom -> 324 then Never mind -> 314; magazine
rate 3 (0 percent) -> 322 pages "NOT a winner", no item; magazine rate 4 (100
percent) -> 317 + blue bed; Famicom 0x1DA8 -> 315 + item; mini card-e -> 320 +
blue bed. Pockets [0x1358, 0x1DA8, 0x1358], conds 21, bells 80000, no WARN/ERROR.
Real save hash unchanged. The magazine text prints the town then player name
together ("TempQA") as the source fields do.

