# Animations and collisions for the event characters

`app/src/event_body.rs`: `EventBody` (a collision pipe: the player cannot walk through them, `player.rs` asks `step_clear`; the
event walkers ask it too, with `town.can_step` for the ground), the stride from the real speed (`NpcGait` walk / run,
`aNPC_set_anime_speed`) and `Idle(clip)` held while they stand. Wired into every event module's actors (aerobics, ball toss,
fireworks, foot race, franklin, gatherings, groundhog, hatsumode, jack, jingle, kamakura, miko, soncho, spooky, tourney, tug).
Clips (`cKF_ba_r_npc_1_*`): hatsumode throw `saisen1`, prayer `omairi1`, greetings `aisatu1..4` by looks (the original's table);
fireworks guests `utiwa_wait1` / `clap1` / `banzai1`; cherry viewers `sitdown_wait1` / `sitdown_clap1`; moon / meteor gazers
`sanbasi1`; harvest clappers `clap1`; tug pullers `tunahiki_aiko1`, leader `hatafuri1`; aerobics `taisou1`.
Not: the exercise sequence (`taisou1..7` in order), foot race / ball toss poses, the countdown's cheering at midnight, rerouting
(held up too long a walker goes on), pushing between villagers, collision for the older actors (Redd, Gulliver, ...).

## Result
Built as above; run in game (1 Jan 2027, Temp save, disc): the procession ran with the clips and strides, no panic; walking
the player into Tad stopped it 1.4 cells short (it had walked 30.0 -> 29.4 in 1.5 s of W). clippy / fmt clean.
Unverified by eye: the individual clips (frames showed only small arm moves), sitting poses at the cherry viewing, the
fireworks guests' fan / clap, wander blocking against trees.
