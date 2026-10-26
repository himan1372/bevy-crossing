# Password check: user and popularity-contest codes (Nook/Timmy "Other things")

2026-10-26 claude. Philip: keep the cuckoo catalog fit open/unclaimed
(see nook-tickets-catalog) and move to the original password service:
password entry, validation, item delivery, as the retail shop flow does.
Joan's visitor/turnip buying stays separate. Handoff priority 2.

## Plan

- [x] Source audit: shop-side flow (ac_npc_shop_common), m_mail_password_check,
      pinned rust/src/password.rs (see below).
- [x] Bounded slice (Philip 2026-10-26): user and popular codes, check direction.
- [x] Port codec kernel into logic/password.rs (safe API, no C ABI, no tests).
- [x] Validation: present checks (user = shop lists + price, other = category/
      birth-type rules), name and NPC-code checks, in app/shop_nook/password.rs.
- [x] Typed entry window on the disc font; delivery as one pocket put with the
      present condition; refusals leave numeric state unchanged.
- [x] Gates, isolated Temp QA, notes/roadmaps/comms, release claim, commit/push.

No tests unless requested. Real save hash only; never load it.

## Audit (2026-10-26)

- Pinned `rust/src/password.rs` was a pure codec plus `extern "C"` wrappers.
  Ported as safe `logic/src/password.rs`: `make_password`, `decode_code`,
  `unpack` (`PwFields::checksum_ok`), editor line helpers (`line_insert`,
  `backspace`, `all_usable`, ...) and mail-body `extract_code`. Original
  tests and wrappers dropped. Workspace check/clippy/fmt pass; nothing in the
  app calls it yet.
- Still to port from `m_mail_password_check.c`: `mMpswd_check_present` (type
  FAMICOM = `pswd_famicom_list`; USER = purchasable-from-Nook check via
  `mSP_SearchItemCategoryPriority` price != 0; others = category allow/deny),
  `check_npc_code`, `check_name` (player+town name compare), special-NPC table.
  `app/src/goods.rs` already loads the disc's shop lists and `items::price`
  gives `mSP_ItemNo2ItemPrice`, so the USER check can be exact.
- Shop flow is in `ac_npc_shop_common.c` (~2063 menu, 2592-2780 make/check
  states, 3106-3133 overlay opens, delivery via handOverItem + free slot).
  Type results: wrong/bad/decline messages, GOOD_FAMICON/NPC/MAGAZINE_1+2
  (magazine rolls RANDOM_F(100) against hit-rate table 80/60/30/0/100),
  CARDE, GOOD_USER, GOOD_CARDEM. Make-password (item+recipient+town -> code)
  is the second direction.
- Entry UI is the m_passwordChk/Make overlays on the shared editor (64-letter
  keyboard, two 14-char lines); no equivalent exists in the app yet.

## Proposed slices

1. (done) codec kernel. 2. validation kernel in logic/. 3. app: shop menu
"Password" choice -> check-overlay entry UI -> validate -> USER/popular
delivery with exact refusals (full pockets, wrong name). 4. make-password
(send) direction. 5. magazine/famicom/card-e specifics.

## Implemented (2026-10-26, claude)

Nook/Timmy request menu -> Other things now offers Turnip Prices, **Say code**
(choice 2 of the disc's four answers; "send a code" stays hidden) and Umm, hang
on. `shop_nook/password.rs`: `PasswordUi` resource, typed two-line entry window
(PC adapter for the retail character grid: letters, digits 2-9, `%`, `&`, `@`;
0/1 read as O/l; Backspace across lines; Esc closes with what is typed) and the
validation. Flow follows `aNSC_request_Q_answer_wait2` / `aNSC_pc_*`: gift limit
(3 per session, `unk_nook_present_count`) -> msg 323, no free pocket -> 311, else
312 -> window -> `aNSC_pc_check_password`: unfinished -> 314; undecodable/cheated/
unlisted/unsupported -> 313 then retry (window keeps the text) or 314; real code
for another player/town -> 324 then retry/314; card-e -> 318 (no item); user
(type 4, hit rate 1) and popular (type 1, hit rate 1, NPC code in range) for this
player and town -> 319/316 then the item arrives with the present condition
when the message ends (rustle sound, counter +1). Message fields filled as the
source: code lines (items 3/4), town (free 6), name (free 7), item (items 2).
`InputFocus::Password` is held until the result is consumed, so a code ending
in P cannot pause (found in QA).

Deliberate limits: no retail character-grid keyboard (typed entry like the
diary); present arrives at message end, not at Nook's hand-over animation and
sound cue; Famicom (type 0), magazine (3, needs RNG), mini card-e (5) codes are
answered as unusable (313) until their slices; a reserved item (0xFFFF) is
refused; furniture-form tools/diaries/fans/pinwheels are validated as ordinary
furniture; foreign-visitor refusal (321) has no caller; the present counter is
per session, not saved (original common data); popular codes the game itself
makes carry hit rate 4 (the source quirk) and are not redeemable, as in C.

## Verification (Temp, disc image, synthetic save; no tests)

Codes made with a scratch generator reusing the ported codec (round trip
verified). Valid user (blue bed 0x1358), popular (blue bed), user furniture
0x1008, stationery stack 0x20C0 (ends in P): each message sequence read ("A blue
bed has arrived for you", contest prize text), pockets [0x20C0,0x1008,0x1358]
conds 21 (all present), bells 80000 unchanged. Fourth attempt: 323 and back to the
menu, no window. Full pockets: 311. Code for another town: 324 -> retry text.
28 A's: 313 -> Yes reopens with the text, Never mind -> 314 -> menu; pockets and
conds unchanged (all zero), bells 80000. Backspace across the line boundary and
Esc decline checked. No WARN/ERROR/panic in the logs. Real save hash
C15154C0...2FBA unchanged; evidence only under Temp/password-qa-*.
