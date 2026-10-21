# Mailbox and postman

Philip 2026-10-20 chose "Mailbox and postman": separate the pockets'
letters from the house mailbox (taking letters out at the mailbox), the
mailbox flag, and Pete the postman (SP_NPC_POST_MAN) delivering.

## Result (claude, 2026-10-20)
- mail.rs: HouseMailbox (SaveData.house_mail); deliveries (post.rs send /
  deliver / Mom / museum info) land there; mailbox_interact requests the
  box's open animation, mailbox_visit opens the pockets (at_mailbox) and
  moves the last letter in every 3 frames (mTG_trans_mail, SE 0x446),
  closing the box when the pockets close. Full pockets: error SE only
  (mWR_WARNING_MAILBOX window not drawn).
- mailbox.rs: flag from HouseMailbox (any letter), owner ring 0x43B per
  second, body track BoxAction Open/Close (open1, SE 0x146@18 / 0x147@4)
  and Delivery (delivery1 1-98, 0x146@14 / 0x147@80).
- post.rs: deliver() decides by where the player is (mPO_make_post_man):
  houses' acre -> Pete; post office acre + force (left with a full desk)
  -> Pete from the post office; else immediate. Post.force saved.
- postman.rs: Pete (NpcModel 255, voice 3) with the original actions,
  speeds, height easing, path/mailbox/exit coordinates mapped through
  mailbox 0's acre; talk 0x08BF+R12 / 0x08CB+R4. Verified run: enter ->
  break -> hover -> landing -> kyoro -> turn -> walk 6-4-0 -> delivery
  (mail into box, flag up) -> fly up -> exit (9 s) -> removed.
- Rustimal reuse: props.rs mailbox facts (open angle/flag/SEs) matched;
  Pete's movement was C-only there.
Left: mailbox panel (pos_win) drawing and slide, the pockets-full
warning window, wing gusts (aPMAN_eff_wings), Pete talk untested live,
Resetti hijack of the mailbox.
