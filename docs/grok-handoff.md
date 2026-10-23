# Grok handoff - 2026-10-21

Continue in `C:/Users/phili/game c/bevy-crossing`, Windows (PowerShell, Git Bash
also available), branch `main`, identity `grok`. Philip authorizes committing and
pushing finished work to `origin main`; never force push. His latest message
controls the task. This file preserves continuity; it does not authorize starting
every item below.

Raffle-day shop room is `18d957c5`: three prizes on the fuku furniture spots,
Nook at the drum, the drum model, and the ball at tick 46. The catalog screen
and paid ordering are the commit after that. Short on bells (0x10AB) and five
full slots (0x10AC) were not played. A ticket with a purchase, order mail at
the next start-up, and the raffle ball during a live draw are still unverified.
The claim on nook-tickets-catalog is still claude's; do not edit claims by hand.

## Read before continuing

1. The general rules in [codex-handoff.md](codex-handoff.md) ("Read before
   continuing", "Skills to use" and everything on runtime isolation) apply to you
   too. Read it first, then the relevant `.claude/skills/*/SKILL.md` before you
   change code. A skill catalog is not a substitute for reading the skill.
2. Both roadmaps: [docs/roadmap.html](roadmap.html) (Philip's readable page) and
   [.claude-agents/ROADMAP.md](../.claude-agents/ROADMAP.md) (appended engineering
   log). Also the latest entries of [comms.txt](../comms.txt).
3. The task note for the open work:
   [.claude-agents/backlog/nook-tickets-catalog.md](../.claude-agents/backlog/nook-tickets-catalog.md).
   The task is currently claimed by `claude`. Before you work on it, post in
   `comms.txt` that you are taking it over, then claim it with
   `bash task-claimer.sh claim nook-tickets-catalog grok`.

## Hard rules

- **Never load, stage or overwrite the real `savegame.json`** in the repo root
  (SHA256 starts `C15154C01FE105A3`). Hash it before and after runtime work.
  Test only with copies under `%TEMP%\bevy-crossing-room-critters` and always
  pass an explicit `--disc`. Stop only processes you launched (record the PID,
  check the exe path).
- Leave the `.gitignore` modification in the working tree alone (never stage it).
- No Nintendo assets in the repo. Content comes from Philip's disc at runtime.
- Faithful to the GameCube original. Look up behaviour in the decomp and disc
  (`gc-iso-reader` skill: sym/hex/arc lookups), not memory or later games.
- Reuse the `logic/` crate first, then the rustimal-crossing modules pinned at
  `122867f762122eb266f04e319af0f0c31d54f476:rust/src/...` (see the module-port
  skill). Port only what is missing.
- Write tests only when Philip asks.
- Gates, one Cargo process at a time:
  `cargo fmt --check`; `cargo clippy --workspace --all-targets -j 1 -- -D warnings`;
  `cargo build -p bevy-crossing --features capture -j 1`; `cargo build -p bevy-crossing --release -j 1`.
- After every patch script, assert that each replacement matched. Silent misses
  have happened. rustfmt rewraps lines, so match the formatted text.
- When you finish: update the task note, `docs/roadmap.html`,
  `.claude-agents/ROADMAP.md` and `comms.txt` (append only). Then
  `task-claimer.sh complete`, commit with a plain sentence-style message, and push.
- Philip's usage budget is tight. Be economical: targeted greps over whole-file
  reads, and few rebuilds.

## Where things stand (nook-tickets-catalog)

Done and pushed (stage 1):
- `logic/src/collection.rs`: `ItemCollection`, mPr_SetItemCollectBit.
- `app/src/catalog.rs`:
  - The `Catalog` resource (collection, TicketOverflow, CatalogOrders, raffle lineup), saved as `SaveData.catalog`.
  - Collect bits are set from normal-condition pocket items.
  - At the start of play, ticket and order letters are delivered (0x057 and 0x049+level, paper 55) through `special_delivery::deliver_special_mail`.
- `app/src/shop_nook.rs`, Purchase arm: qualifying items give a ticket (0x10AE), or mail it when the pockets are full (0x10AF).
- `app/src/raffle.rs`, on the last day of the month:
  - Ask 0x10D1, then take five tickets (0x10D4), then a 3 s drum.
  - A win: 0x10D9-DB plus 0x10E3; the prize is given wrapped, then 0x10DD or 0x10DC.
  - A loss: 0x10D5, then 0x10D7 or 0x10D6.
  - Verified in a preview (env `BEVY_CROSSING_DAY_OFFSET` / `BEVY_CROSSING_HOUR_OFFSET` set to Oct 31 at noon).
  - Raffle-day room: the three prizes replace the normal goods, Nook stands at the drum, and the drum model is in the room. The ball rolls out at drum tick 46; that moment was not played (the preview save had no tickets).

Not verified yet:
- A ticket coming with a purchase. The test script never reached a shop display; displays sit around (2997-3001, -3002.5) inside the shop, and you buy by facing one and pressing E.
- The order mail at start-up. (Ticket mail at start-up is verified: 7 tickets
  arrive as two letters, stacks of 5 and 2.)

Catalog screen (this commit):
- Nook choice 1. Played: 0x10A6, 0x10A7, 0x10A8 (tricera skull), 0x10A9 (Escape), 0x10AA (blue bench, 1900 bells, slot 0 item 0x135c). Not played: 0x10AB, 0x10AC.
- 9 tabs, 7 rows, collected items only. `clg_*` models on the talk-window camera. Each page uses its own window texture. The left circle shows the selected furniture from the side and spins it. Wallpaper, carpet, stationery and music stay empty there.
- Payment is on confirm. Price 0 does not order. Escape does not spend bells or a slot. Raffle day still returns before this talk.

Next: ask Philip which roadmap item to do. The open checks above can wait.

Shop door note for previews: enter by walking north-east (W+D) from about
(-15.9, -35.9) during opening hours. On a closed hour the door does nothing.
