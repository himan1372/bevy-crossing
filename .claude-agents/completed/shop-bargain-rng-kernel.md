# Bargain RNG kernel

Philip: "do the bargain RNG kernel" (2026-10-28). `completed/shop-bargain-sales.md` left the sale-hour roll and the offer stock choice on local adapters
(`seed % 8`, a xorshift in `shop/bargain.rs`). Source: `libc64/qrand.c` (global LCG: idum = idum * 0x19660D + 0x3C6EF35F; fqrand = bits(idum >> 9 | 0x3F800000)
- 1.0), `RANDOM(n) = (int)(fqrand() * (f32)n)` (`sys_math.h`), `m_event.c` init_special_event (`dates[SPECIAL3] = 12 + RANDOM(8)`),
`ac_event_manager.c` init_sp_bargain -> `mSP_SelectRandomItem_New`. rustimal-crossing has no qrand module (only a private copy in `town_gen.rs`).

- [x] `logic/src/qrand.rs`: Qrand (qrand, sqrand, fqrand, fqrand2, random) with its own tests.
- [x] App `GameRand` resource (seeded from the clock like the boot `sqrand`), used for the sale start hour and the offer stock.
- [x] Gates, Temp QA of a scheduled sale, docs/comms, release claim, commit/push.

## Implemented (2026-10-28, claude)
- `logic/src/qrand.rs` (new, exported in `logic/src/lib.rs`): `Qrand` = `libc64/qrand.c` (`qrand`, `sqrand` as `seed`, `fqrand`, `fqrand2`) plus `random(n)` =
  `RANDOM(n)`. Its three own tests pass (recurrence from seed 0 = 0x3C6EF35F, the mantissa trick, `random` stays below n and hits every value of 8).
- `app/src/game_rand.rs` (new): `GameRand(Qrand)` resource, registered in `ShopPlugin`, seeded from the system clock (the original seeds at boot).
- `app/src/shop/bargain.rs`: the sale start hour is now `12 + RANDOM(8)` (was `12 + seed % 8`, same seed as the event-type pick) and the offer stock choices
  draw `RANDOM(n)` from `GameRand` (was a private xorshift keyed by player id and day). The event-type pick, gap and sale-day rule stay as they were: the source
  derives those from the player id and the clock, not from the global RNG.
- Not touched: `town_gen.rs` keeps its private copy of the same LCG (its own test reads the state field), `roster.rs`/`face.rs`/`turnips.rs`/`grab_bag.rs`/`raffle.rs`
  keep their local generators; they can move onto `GameRand` one at a time.
Gates: fmt, clippy -D warnings, check; `cargo test -p rustimal-logic qrand` (kernel's own tests only); capture build.
Verified (Temp, disc, synthetic tier-0 save without `shop_events`, fresh process per hour, `-Hour 9..13`): events were scheduled with the expected hours
(shop sale 12/18 on the two draws of the hour-13 run, both inside 12..19; others 6, 18, 21 as before); no WARN/ERROR. The offer stock draw was not read back at
runtime (no log of its items); it runs through the same `below` closure. The hour-14 run hung in the launcher and was stopped; hours 14-16 are unrun.
