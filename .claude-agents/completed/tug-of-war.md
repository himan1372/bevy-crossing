# Sports fair: the tug of war (`ac_tunahiki_control`, `ac_tunahiki_npc0/1`)

Scope: SPORTS_FAIR_TUG_OF_WAR (15:00-16:59 on the equinox days): two teams of two pullers (units (3,11) (4,11) facing
east, (11,11) (12,11) facing west, moved in by 85 / 95 units) on a rope, the flag-waving leader at (7,10) + (20,-30)
units, Tortimer at (10,8). A pull cycle of 1.05 s: the shake starts at 0.2 s (speed = next speed) and ends at 0.8 s (a
new next speed: 45% +3..23, 45% -3..23, 10% 0); while shaking the rope base moves `speed * 0.025` per tick, clamped to
+-10 units, frozen while someone talks; the pullers (and the rope) shift with it. At the end each puller picks its
pose from the rope (even / swaying / straining). Talk: `base(looks) + offset + RANDOM(3)` (even 3, swaying 6, straining
9; the leader 0), bases 6532 6544 6520 6556 6568 6580. Runs while the player is in the shrine acre.
Not: the animations, the rope model (a cylinder stands in), the sweat effect, sounds, the team win.

- [ ] tugofwar.rs, soncho.rs unit (10,8), main
- [ ] gates, Temp run on an equinox day at 15:00, result, comms, commit

## Result
`app/src/tugofwar.rs`; `soncho.rs` puts Tortimer at unit (10,8) during the tug of war (he talks, as before). Run in game
(Temp save, disc, 21 Mar 2027 15:xx, the player in the shrine acre): two teams of two pullers on the residents' own
models either side of a brown rope (a cylinder), the leader by the well; the rope base swings (a puller's x moved
between 29.375 and 29.875, i.e. +-0.25 cell) with the shake and the 45/45/10 speed draws; a talk with a puller gave
message 6579 ("Thuh-this isn't working... I can't lose!", the straining line).
Unverified: the swaying and even lines, the leader's, the pause while talking, Tortimer at (10,8).
Not built: all animations, the rope model, the sweat effect, the sounds and any result of the contest.
