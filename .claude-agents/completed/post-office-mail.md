# Real post office mail

Philip 2026-10-20: "post office" (asked after the museum; unblocks fossil
appraisal by mail, museum completion and first-donation letters).
Retail mail: letters on stationery with an attached present, written from
the pockets, sent at the post office, delivered on the post office
schedule; system letters from the disc's mail text (mail_no); mail to the
museum (fossil appraisal); the post office holding mail when the mailbox
is full.

## Result (claude, 2026-10-20)
- messages.rs: MailBanks (super/mail/ps + superz/maila-c/psz from
  forest_1st.arc), GameText::letter / letter_z (header name at
  header_back_start).
- mail.rs: Letter {header, footer, present, present_mine, to: Recipient};
  invented starter/villager/reply letters removed; writing from a paper
  item (one sheet used), Rewrite, Present take-out; retail letter menus.
- pockets.rs: "Write" on stationery, item carried onto a written letter,
  send mode (mailing/mail_pick), "Open" for wrapped presents.
- post_office.rs: Send a letter flow (0x8B9/0x8E1/0x8E3/0x8BD/0x8BB/0x8B3).
- post.rs: Post (store 5, 09/17 delivery, replies, museum record) saved
  as SaveData.post; daily: replies (good mHandbillz / wrong 0xC5+), museum
  results (remail 0x22B-D, fossils 0x10E.. via mRmTp_birth_type 12/13),
  info letter 189, completion 0x22F + 0x3030.
- logic museum.rs: Mailed, mailed(), fossil_mail_no, constants;
  logic mail module made pub.
Verified (Temp preview, real save unchanged): info letter on stationery,
writing to Museum, fossil attached, posted (0x8BA Phyllis), next day
apatosaurus torso letter + present (wrapped), letter menus, villager
letter scored Bad -> next-day wrong reply, good reply pieced (Tybalt,
muffins). Typing capitals/punctuation not possible through BRP keys.
Left: separate pocket/mailbox stores, postman + mailbox flag, villager
initiated letters, Mom/Nook/catalog mail, on-screen keyboard, present
cloth wearing, foreign-town letters.
