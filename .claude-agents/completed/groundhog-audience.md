# Groundhog Day's audience (`ac_groundhog_npc0`)

Scope: four villagers at units (5,12) (6,12) (8,12) (10,12) of the shrine block (nudged (0,-20) (20,0) (20,0) (0,-20)
units, facing 150.5 / 180 / 180 / 209.5 degrees) while GROUNDHOG_DAY runs; talk `base(looks) + term * 3 + RANDOM(3)`
with term 0 (before 07:30), 1 (07:30), 2 (07:45), 3 (07:55) and 4 (from 08:00), none in the minute before 08:00; bases
15698 15729 15605 15636 15667 15760 by looks. Not: the heads turning, animations, the other shrine events' audiences (New
Year's hatsumode procession is a long scripted walk with the player and is not part of this).

- [ ] groundhog.rs (spawn and talk), gates, Temp run Feb 2 07:xx and 08:xx, result, comms, commit

## Result
`groundhog.rs`: four residents at the audience units with the original's nudges and facings, `Audience` talk lines.
Also a small fix across the sports-fair villagers (`footrace`, `balltoss`, `tugofwar`, `aerobics`) and the audience: the
villager's own catchphrase (`dialogue::catchphrase`, `npc_def_list`) now fills the message's tail (it was blank, e.g.
"...to end, ."). Run in game (Temp save, disc, 2 Feb 2027 07:xx): the four villagers stand in the shrine acre; a talk gave
message 15672 ("What kind of predictions do you think that idiot'll make this year, grrrolf?", term 0).
Unverified: the later terms (07:30, 07:45, 07:55, 08:00+), the silent minute before 08:00, the fixed tails in the sports
fair talks (same helper, not re-run).
Not built: the heads turning (`head.angle_add_y`), animations, the other shrine events' audiences. The New Year's
hatsumode procession (five visitors walking to the shrine with the player) is a long scripted sequence and not started.
