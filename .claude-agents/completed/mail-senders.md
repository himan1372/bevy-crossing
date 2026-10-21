# Missing letter senders

Philip 2026-10-20: "lew go" (after: fill the missing letters from the
unused logic modules, starting with Mom's letters and catalog/lottery).
Use logic/src/mother_mail.rs, special_delivery.rs (and later
npc_event_mail.rs, leaflet.rs) with post.rs delivery.

## Result (claude, 2026-10-20)
- Mom: logic mother_mail (now pub; MotherMailData bytes()/from_bytes())
  driven from post.rs daily (Post.mother_date / mother_sent saved);
  presents resolved (goods, umbrella range, mFI_GetOtherFruit retail
  rule with FieldItems.fruit now kept, cake 0x11FC, doll 0x10E8,
  dracaena 0x13B0); Mother's/Father's Day = May 2nd / June 3rd Sunday
  (m_event_schedule); no player birthday stored yet -> (0,0).
  Verified: Nov 11 (day offset) -> mail 0x178 on autumn paper.
- Blocked, recorded for later:
  - special_delivery (catalog orders, lottery tickets): the shop has no
    catalog ordering or ticket handout yet.
  - npc_event_mail: Valentine needs player gender + valentines_day_date
    event; birthday cards need the player's birthday; Christmas card is
    sent by Jingle on Toy Day (and logic XMAS_PRESENT is a placeholder,
    real FTR_START(FTR_FAMICOM_COMMON01) to resolve); goodbye mail needs
    villagers moving out.
  - leaflet: needs special events (shop sale / broker sale).
