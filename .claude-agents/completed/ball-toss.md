# Sports fair: the ball toss (`ac_tamaire_npc0/1`, `ac_tama`)

Scope: SPORTS_FAIR_BALL_TOSS (13:00-14:59 on the equinox days): two baskets (red at unit (3,8), white at (12,8)), four
tossers (two per team) who look about, run to a spot 1.75-2.25 cells from their basket, pick up three balls, turn and
throw them into it one by one (only one tosser of a team throws at a time), stand 2 s, look about again; the leader
(`TAMAIRE_0`) walks between four stations round the well and cheers; Tortimer stands at unit (9,4) and takes no talk.
Talk: `base(looks) + offset + RANDOM(3)` (tossers 3 looking / walking, 6 picking up / throwing, 9 after a throw; the
leader `base0(looks) + RANDOM(3)`; bases 7781 7793 7769 7805 7817 7829). Runs while the player is in the shrine acre.
Not: animations, the basket and ball models (plain shapes stand in), the effect's scatter and sounds, the team score.

- [ ] balltoss.rs, soncho.rs unit (9,4), main
- [ ] gates, Temp run on an equinox day at 13:00, result, comms, commit

## Result
`app/src/balltoss.rs`; `soncho.rs` puts Tortimer at unit (9,4) during the ball toss (he still takes no talk). Run in game
(Temp save, disc, 21 Mar 2027 13:xx, the player in the shrine acre): a red and a white basket (plain cylinders), the
leader and four tossers on the residents' models; tossers look about, run to a spot by their basket, pick up, turn and
throw: balls fly an arc into the basket (0-2 in the air at a time); a talk with a tosser gave message 7773
("I can't talk right now! Can't you see I'm busy looking for more balls?").
Unverified: the leader's lines and walk between the stations, the other offsets (6 / 9), the white team, the basket
rejection rule (a team's tosser waits while another throws).
Not built: all animations (they slide upright), the real basket/ball models, the scatter of the ball effect (every
ball lands in or beside the basket), the sounds and any score.
