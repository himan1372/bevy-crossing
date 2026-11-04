# Sports fairs, slice 1: Tortimer's part

Scope: SONCHO_SPRING/FALL_SPORTS_FAIR (equinox days, 09:00-16:59): the commemorative speeches apply (the golden statues,
mSC_EVENT 8 / 20); his think states per field event (`aES2_wait`, `dt_tbl`): no talk request during the aerobics (09-10) and
the ball toss (13-14), talks during the foot race (11-12) and the tug of war (15-16).
Not in this slice (separate, large): the villagers' foot race (`ac_tokyoso_*`), ball toss baskets, tug of war and the
aerobics (`ac_taisou_npc0`), spectators and their lines.

- [ ] soncho.rs: sports fair events allowed, talk gated by the field event
- [ ] gates, Temp run on an equinox day at 11:00, result, comms, commit

## Result
`soncho.rs`: the sports fairs' events (8 spring, 20 fall) are no longer excluded; during the aerobics (09-10) and the
ball toss (13-14) he takes no talk. Run in game (Temp save, disc, 21 Mar 2027 11:28, the foot race hour): Tortimer at
the shrine with the spring medal; talk 0x32D0, 0x32D3 and the medal given (event 8).
Unverified: the gate during the aerobics / ball toss hours, the fall fair's medal.
NOT built (the real bulk of the sports fairs, to be done in slices): the foot race of the villagers (`ac_tokyoso_control`,
`_npc0`, `_npc1`: warm-up, ready-set-go, spectators' lines), the ball toss (baskets, throwers), the tug of war, the
aerobics (`ac_taisou_npc0`), their positions at the shrine, and the villagers' talk lines (`base_msg + RANDOM(2)`).
