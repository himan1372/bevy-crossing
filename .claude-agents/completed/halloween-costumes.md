# halloween-costumes — Halloween trick-or-treat, slice 1: costumes, the chase, the greeting

Source: `ac_halloween_npc.c`, `ac_halloween_npc_talk.c_inc`, `ac_halloween_npc_schedule.c_inc`,
`m_event_map_npc.c` (`mEvMN_GetHalloweenNpcName`), `ac_set_npc_manager.c`; kernel `logic/src/halloween.rs`.

The trick-or-treaters are the town's own villagers: while `mEv_EVENT_HALLOWEEN` runs (Oct 31 18:00 to midnight)
each villager the set-NPC manager makes is registered as `SP_NPC_EV_HALLOWEEN_0 + free slot` (five slots): the
costume's draw entry is the texture, the villager stays the identity and animation. They run at the player while
in the player's block and force a talk within 70 units: 2442, then 0x98B + looks*6 (the question).

- [ ] character.rs: `Costume` (draw entry replaces the villager's own), rig rebuilt when it changes.
- [ ] npc.rs: `Villager::run_at`.
- [ ] halloween.rs: costumes on the first five villagers while the event is active; chase; forced greeting + question.
- [ ] check / clippy / fmt; Temp run with the disc at Oct 31 evening (BEVY_CROSSING_DAY_OFFSET / HOUR_OFFSET).
Later slices: the treat (pockets picker, candy / tool / other item, hand-over), the trick (item swap, costume shirt),
the ordinary talk 2479 + looks. Not in this slice: villagers made outside the player's block keep their costume
registration only by order here (the original registers as they are made).

## Result
- `character.rs`: `Costume(u32)` (draw entry replaces the villager's own), `WornModel`, `refit_costumes` (despawns the
  rig child and lets `spawn_villager_models` rebuild it); `npc.rs`: `Villager::run_at`, and a costumed villager counts
  as out in `villager_schedule` (the original's LEAVE_HOUSE step); `halloween.rs`: `assign_costumes`, `approach`,
  `talk` (forced greeting 2442, then 0x98B + looks*6 and its Treat / Trick question).
- Temp run with the disc (synthetic save, BEVY_CROSSING_DAY_OFFSET=-4 / HOUR_OFFSET so the clock read Sat 10/31 8:25 pm):
  the log lists HALLOWEEN active, five "puts on costume 0..4"; teleporting the player beside villager doors, Tad (a
  pumpkin costume seen beside the player) ran up and the dialogue "Trick or treat!" (0x98A) opened; advancing gave
  0x99D (= 0x98B + looks 3 * 6) and the answer was logged.
Not done: the treat and trick branches (the talk ends after the answer), the normal talk afterwards (2479 + looks),
the other four costumes were not looked at one by one, the question window itself was not screenshotted, costumes go
to the first five villagers by table order (the original registers them as they are made), approach has no block
limit beyond the acre (the original compares map blocks; here an acre is a block), no trick animation.
check / clippy -D warnings / fmt pass; no tests; real save untouched.
