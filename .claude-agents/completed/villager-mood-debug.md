# Debug: force a villager's mood

Philip: "non happy" (2026-10-29), after `completed/moving-talk.md` could not be seen live (every reachable villager rolled happy).
`BEVY_CROSSING_VILLAGER_MOOD=normal|happy` fixes the mood roll in `dialogue.rs` (like `BEVY_CROSSING_HOUR_OFFSET`), so the
non-happy talk tree (and the moving talk) can be driven. Unset: the ordinary 25% roll.

## Done (2026-10-29)
- `dialogue.rs::roll_happy`: `BEVY_CROSSING_VILLAGER_MOOD=normal` or `happy` fixes both mood rolls.
- Driven (Temp capture copy, `BEVY_CROSSING_VILLAGER_MOOD=normal`, saves with `remove_slot` 4 and 1): Anchovy (lazy) reached the event branch and said the moving talk (msg 0x1cd5, an announcement with no question); Bluebear (peppy) said 0x1504 (announcement) and 0x1505 ("do you want some moving secrets?", answers "Any tips?" / "I know already."). The log tags each talk `thinking of moving`.
- Personalities differ: some variants end in a question whose "no" branch carries the cancel order (peppy 0x1506 -> 0x150A has quest order index 1 value 2, `<0c:010002>`); normal 0x11FC -> 0x1200. That branch (and `roster::cancel_moving` in the running game) was not reached in the harness: the 1/3 chance of 0x1506 plus the answer keys did not line up in the time spent. Unit-tested instead.
- Harness traps: a pockets or letter modal left open blocks every talk (Escape x4 first); the same villager stops answering after one talk when ArrowDown is sent with every Space.
