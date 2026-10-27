# Joan, the Sunday turnip peddler

2026-10-26 claude, Philip: "joans". Original: `ac_ev_kabuPeddler.c` (+ `_move.c_inc`),
schedule `m_event_schedule.c_inc` (every Sunday 06:00-11:59), placement
`ac_event_manager.c` (`turnipbuyer_*`, `search_free_unit`), gate
`mAGrw_CheckKabuPeddler`. She sells turnip bundles at Sunday's price, which the shop
turnip work already stores (`shop/turnips.rs`).

- [x] Source audit of her talk script (msgs 0x06FD..0x0718) and purchase flow.
- [x] Joan appears in town Sunday morning, one per player/day, talks, sells
      10/50/100 turnips for the Sunday price (atomic Bells -> bundle, full/short refusals).
- [x] Remembers who she has spoken to this Sunday (saved), disc voice/name/model.
- [x] Gates, Temp QA, docs/roadmaps/comms, release claim, commit/push.

## Implemented (2026-10-26, claude)

`app/src/joan.rs`. Schedule from `m_event_schedule.c_inc`: every Sunday 06:00-11:59 she
is out in town (`visiting`), one Joan, despawned outside the window or when the player
is not in the town scene. Spot: `spot()` follows `search_free_unit`'s seed over the
town's acres (day-driven) and takes the clear, empty unit nearest that acre's
middle (the original's block exclusions and unit rules are approximated: this town has
no pool/station/shrine/dock block table here). Disc model NPC 245 (`SP_NPC_KABUPEDDLER`),
name Joan, voice 8 (`l_sp_actor_name`). She turns to the player while talking.

Talk (`ac_ev_kabuPeddler.c`): 0x06FD the first time this Sunday (her lesson,
answers link through the disc script via `talk_asking`/`MsgChoice.next`), 0x0718 after;
`Market.joan_spoken` (saved with the turnip market, per player and day) replaces
`spoken_pids`. Price strings free0..3 = price x 1/10/50/100 (Sunday's `Market::price`).
A four-answer question is the sale menu; picking a bundle shows its price line (0x70B-D
or 0x712-4) and the sale runs when that line ends: a free pocket first, then the Bells,
else 0x70F/0x710 (first round) or 0x716/0x717 (after a sale); a sale gives item
0x2F00/01/02 and 0x0711 asks again; "no" lines 0x70E/0x715 end it.

Also fixed: `copper.rs` took every talker's answer in town; it now only takes its own.

Verified (Temp, disc, synthetic save at Sun 1 Nov 08:25): she appears and talks (name
plate and voice), intro lines, "I know already" -> 0x0709, prices "74 Bells per turnip",
"Just 10 turnips? That'll be 740 Bells.", sale -> "Here you go, kiddo / Much obliged" and
the menu again; 50 for 3700; Bells 5000 -> 560, pockets [0x2F00, 0x2F01]; a 100 attempt
without Bells gets the refusal; a second visit greets with 0x0718; `joan_spoken` saved.
No WARN/ERROR in the logs. Real save hash unchanged.

Not done: she does not walk away after the talk (`turnipbuyer_behind`), the money and
turnip hand-over demos (`MONEY_GET_DEMO`, `KABU_TRANS_DEMO`, the player's give pose),
the wait-for-first-load gate `mAGrw_CheckKabuPeddler` (she is simply out every Sunday
morning), the exact free-block exclusions, and the NPC's own wander/idle behaviour.

