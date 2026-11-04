# halloween-treat-trick — Halloween slice 2: the treat and the trick

Source: `ac_halloween_npc_talk.c_inc` (menu_open/close_wait, get_other_item, receive_tool_item, trick_timing,
trick_chg_cloth, get_ame, `aHWN_get_trick_type`), kernel `logic/src/halloween.rs`.
Treat: pockets in the give mode (any item: Give / Quit). Nothing picked: 0x98C + trick. Candy: taken, 0x98F, no trick.
Tool: handed over, 0x990, handed back, trick. Other item: taken, 0x98E, trick. Trick: if a swappable item exists and
RANDOM_F < 0.5 one becomes a Halloween2 furniture piece, else the costume shirt (ITM_CLOTH017); then 0x98D + looks*6.
Port: hand-over before the line; quest-condition items cannot be offered; no change animation / sparkle / 0xFD anim.
Not in this slice: the ordinary talk 2479 + looks (needs the villager talk intercepted).

- [ ] pockets.rs: `halloween` mode; halloween.rs: Step machine.
- [ ] check / clippy / fmt; Temp run with the disc: candy, an ordinary item, a tool, nothing, Trick; both tricks.

## Result
- `pockets.rs`: `PocketsUi.halloween` (give mode: "Give" / "Quit" for any item without the quest condition; the slot
  comes back in `sell_slot`), `pockets_disc.rs` compact menu. `halloween.rs`: `Step` machine (Greeting, Asked, Picking,
  Said, Hand), `trick`, `pause_villagers` (+ `npc::VillagerPause`: villagers stand still during the steps with no
  window up). `character.rs`: villagers now play `NpcClip` (hand-over arm clips; the item waited forever on the
  taker's clip before), with `Without<Player>, Without<NpcModel>` on the query (a B0001 conflict `cargo check` does not
  catch showed up at runtime and was fixed).
- Temp runs with the disc (synthetic save: candy, net, an item, a furniture piece; clock faked to Oct 31 evening; BRP keys
  and screenshots; F5 saves to read the pockets):
  - Candy: 0x2806 offered, hand-over completes (modes 2, 4, 7), 0x9A1 "Awesome, sluuuurp! Thanks!", pocket 0 empty,
    shirt unchanged, no trick.
  - Tool: net 0x2200 offered, 0x9A2, handed back (modes 2, 3, 7), then the item trick: pocket 2 became 0x14C4 (a
    Halloween piece), 0x99F; the net stayed in pocket 1.
  - Trick answer (Pinky): her own line "What?! You're not going to give me candy?!", the cloth trick (player in the
    stitched costume shirt), 0x993 "Ha-ha! See what happens when you don't share, cubbie?".
  - Treat, nothing picked (Escape in the pockets): 0x99E, then the cloth trick, 0x99F.
  Message numbers match 0x98x + looks * 6 for looks 3 (Tad) and 1 (Pinky).
Not driven: an ordinary (non-tool, non-candy) item (same code as candy plus the trick), a full-pocket case, the
other three villagers. Not built: change animation / sparkle for the shirt, the item trick's player animation (0xFD),
the ordinary talk 2479 + looks, quest-condition items offered (kept out on purpose).
check / clippy -D warnings / fmt pass; no tests; real save untouched.
