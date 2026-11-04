# Sports fair: the foot race (`ac_tokyoso_control`, `ac_tokyoso_npc0/1`)

Scope: SPORTS_FAIR_FOOT_RACE (11:00-12:59 on the equinox days): two runners (one per team) run four laps (eight half
laps) round the wishing well in lanes of ~165 / ~130 units at `4 + lane/90 - 0.5*team` units per tick (the leader slows by
1), the first over the line wins and the other loses, a pause, then the stretch, ready and set again; the starter, two
spectators of the teams and Tortimer (unit (5,8)) stand by. The player talks to them: `base_msg(looks) + offset +
RANDOM(2)` with the offset by what they are doing (stretch 2, ready 4, run 6, goal 10, spectators 12, idle 14); the
starter `base_msg + RANDOM(2)`. Runs while the player is in the shrine acre.
Not: the animations (stretch, ready crouch, banzai, tired, stumbles), the team swap relay details, sounds.

- [ ] footrace.rs, soncho.rs position (5,8), main
- [ ] gates, Temp run on an equinox day at 11:00, result, comms, commit

## Result
`app/src/footrace.rs`; `soncho.rs` puts Tortimer at unit (5,8) during the foot race. Run in game (Temp save, disc, 21 Mar
2027 11:xx, the player in the shrine acre): five actors (two runners, two spectators, the starter) on the villagers'
own models; the race cycles: stretch (10 s), ready, go (four laps round the well, lanes of 4-4.25 / 3.25-3.4 cells,
leader slowed), a winner and a loser logged, a 10 s pause, the next stretch; a talk with a runner after the race gave
message 6664 (`base 6653 + 10 + 1`: "Pant, pant, pant...").
Unverified: the lines in the other phases, the winner's / loser's difference, the starter's lines, Tortimer at (5,8).
Not built: all animations (they glide upright), stumbles, the team relay, sounds, the camera; the runners are plain
orbits rather than the original steering, and the event-slot NPC models (white blobs in the port) are replaced by the
residents' own models.
