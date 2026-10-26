# Password service (Nook/Timmy "Other things")

2026-10-26 claude. Philip: keep the cuckoo catalog fit open/unclaimed
(see nook-tickets-catalog) and move to the original password service:
password entry, validation, item delivery, as the retail shop flow does.
Joan's visitor/turnip buying stays separate. Handoff priority 2.

## Plan

- [x] Source audit: shop-side flow (ac_npc_shop_common), m_mail_password_check,
      pinned rust/src/password.rs (see below).
- [ ] Decide bounded slice (which password types) and record it here.
- [x] Port codec kernel into logic/password.rs (safe API, no C ABI, no tests).
- [ ] Port validation: present checks (famicom list, user/other), name and
      NPC-code checks, special-NPC table.
- [ ] Entry UI on the existing disc renderer; delivery as an atomic pocket/
      wallet transaction; refusal paths leave numeric state unchanged.
- [ ] Gates, isolated Temp QA, notes/roadmaps/comms, release claim, commit/push.

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
