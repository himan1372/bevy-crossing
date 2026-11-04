# Spooky's spirits (hitodama)

Scope: five spirit acres rolled per ghost night (`ghost_start`: bx 1+RANDOM(5), bz 2+RANDOM(4), unique); once the player
has found Spooky the acre the player enters with a spirit block gets one flying spirit (`ac_ins_hitodama.c`), caught with
the net into a stacking ITM_SPIRIT0..4 item (`m_hand_ovl`, `mPlib_Get_space_putin_item_forHITODAMA`); an uncaught
spirit re-registers to another free acre when it goes (`aINS_destruct`). Spirits flee when the ghost event is over.

- [ ] app/src/spirits.rs, net hook in ecology.rs
- [ ] gates, Temp run, result, comms, commit

## Result
`app/src/spirits.rs` + a spirit branch in the net catch (`ecology::tool_interact`). Run in game (Temp save, disc,
01:00 on the ghost day; Spooky's weed check bypassed with a temporary env hook, removed): five acres rolled
("Spirit acres tonight"), Spooky found, a glowing spirit floated in the entered acre, one that was left re-registered
to another free acre, and a net swing caught one ("Caught a spirit (1/15 pockets)").
Acre mapping: block x 1..=5 / z 2..=5 are acres 0..=4 / 1..=4 of the port's 5x6 grid.
Unverified: stacking into SPIRIT1..4 on later catches and the full-pocket refusal (code follows
`mPlib_Get_space_putin_item_forHITODAMA`), the whole chain to the five-spirit hand-in with caught spirits.
Not built: glow point light, avoid/escape animations, the catch message (the HUD says "Caught a spirit!"), the disc's
spirit sprite (a glowing sphere stands in; so do the other bugs).
