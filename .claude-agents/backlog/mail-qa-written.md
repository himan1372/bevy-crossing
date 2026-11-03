# Mail QA: written-letter Rewrite, arrow variants, Museum colours, Write refusal

2026-11-02, claude. Remaining from mail-qa-remaining: drive in a Temp copy with a 10-letter synthetic mailbox (written letters to a villager and to the Museum,
rows 0/1/2 for the arrow variants) and a full-mailbox Write attempt. Fix what the drive shows.

## Result 2026-11-02 (claude)
Driven in `Temp/bevy-mailqa-20261102` with a synthetic 10-letter mailbox (written letters to a villager and the Museum, received letters from Mom / Nook / Redd / HRA / Jingle). All passed, no code changed:
- Tag arrows: mail row 0 pointer to the right with the tag left or right of the cursor depending on room; row 1 lower pointer (tag above); row 2 upper pointer (menu above, tag below); the pockets' own
  delivery label ("Delivery for Kabuki from Tom Nook") sits below the cursor with the pocket pointer. Museum recipient reads "the Museum" in green, villager names red, senders in their colours.
- Written letters offer Rewrite / Grab / Throw Away / Quit (received: Read ...). Rewrite cancel (Escape) returns to the grid, letters unchanged. Rewrite commit (F4, answers, Yes) saved the new body
  ("Hi there.!!") and kept `to: Museum`; the other nine letters were byte-identical.
- Write on stationery with ten letters shows "You can't carry any more letters, so you can't write a new one!".
Observation, not changed: Rewrite keeps the addressee only if they are still in the address book; otherwise `write_recipient` falls back to the first entry (the Museum), so a letter to a villager the
player no longer "remembers" is re-addressed on commit. Check `mAD` / `mMl` source before touching it. Philip acceptance open.
