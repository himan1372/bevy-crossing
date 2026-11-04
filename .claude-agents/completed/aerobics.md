# Aerobics (radio calisthenics: morning aerobics and the sports fair's)

Scope: the line-up at the shrine (`l_rc_map` / `l_fdd_radio_map`): four villagers at units (4,10) (6,10) (8,10) (10,10),
Copper at (7,12), Tortimer (`SP_NPC_SONCHO_D078`) at (9,8). Lines: villagers `msg_base[event][looks] + RANDOM(2) +
type*3`, Copper `0x2665/0x2668 + RANDOM(3)`. Sports fair: Tortimer's commemorative flow of `soncho.rs` (he stands in the
line and talks during the aerobics hours). Morning aerobics (Jul 25 - Aug 31, 06:00): Tortimer stamps the player's
13-day radio card (`mSC_Radio_Set_Talk_Proc`): new card, stamp, lost card, finish and the radio, same day, too late,
delete old cards; the radio-test furniture at the end. Not: the radio music sync, the exercise animations, failures.

- [ ] aerobics.rs, soncho.rs line-up variant, identity radio card, main
- [ ] gates, Temp runs (sports fair 9:00, morning aerobics 6:00), result, comms, commit

## Result
`app/src/aerobics.rs` (line-up and talks, the radio card), `soncho.rs` (the sports fair's Tortimer stands in the line at unit
(9,8) with the `SP_NPC_SONCHO_D078` model and talks during the aerobics; still silent in the ball toss),
`PlayerIdentity.radio_card`. Run in game (Temp saves, disc):
- Sports fair, 21 Mar 2027 09:37: Tortimer (medal event) plus four villagers in their gym outfits at units (4..10,10),
  Copper at (7,12); Copper says 0x2668.
- Morning aerobics, 1 Aug 2027 06:xx: the line; Tortimer: first morning 0x3422, 0x3425, the card (0x2523) handed over,
  0x3426; the same morning again 0x3433 and 0x3435.
Unverified: the villagers' own lines, the later mornings (stamp 0x3427.., finish 0x342A and the radio, lost card 0x342E,
too late 0x3437), full pockets (0x3424), two cards (0x343C), the sports fair medal from the line, the model of Copper
(`EV_TAISOU_0`; a fox in white shows).
Not built: the radio music and animation sync (the line stands), routine failures and tiredness, the radio set, the
exercise card's days in the HUD.
