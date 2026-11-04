# Fireworks show (`ac_hanabi_npc0/1`, `ac_ev_yomise`, the stall, the hanabi effect)

Scope: FIREWORKS_SHOW (July 4, 19:00-20:59) in the acre with the river (the port has no pond acre): one of the seven
layouts (`l_fws_pule1..7`, by day) puts the five villagers, the night-stall seller (`SP_NPC_EV_YOMISE` / `_2`) and his stall
on their units. The villagers: the first wanders within 100 units, the others stand fanning / clapping, facing by the
`def_angle` table; their talk is `base(looks) + 3 * k + RANDOM(3)` (k = 0..4), bases 5711 5726 5696 5741 5756 5771. The
seller: three kinds of goods by the day's draw (bluebell fan 780 Bells, yellow pinwheel 680, red balloon 480), eight of
each kind in colours; the offer 0x1758 / 0x1759 / 0x175A (Yes / No), the list of three goods and a fourth refusal
("I'm not buying!" / "I don't want it!", paging), full pockets 0x175D, too poor 0x175E, bought 0x1764 then 0x1766 (more?)
or 0x1765, declined 0x1763 (0x1767 for the next page), sold out 0x1757. Saved (the goods left are the event's area).
Fireworks: simple coloured bursts over the acre while the event runs (the original's effect is not built).
Not: the villagers' animations (fan, clap), the stall model (a box), the seller's paper fan, sounds, the hanabi sync.

- [ ] fireworks.rs, save area, main
- [ ] gates, Temp run Jul 4 19:xx, result, comms, commit

## Result
`app/src/fireworks.rs` (+ `stall_event` in the save). Run in game (Temp save, disc, 4 Jul 2027 19:xx): in the river acre
(the port has no pond acre) five villagers on their own models at the layout's units (the first wanders within 100
units), the night-stall seller on his model with a box stall; coloured spark bursts over the acre. The seller: offer
0x1759 (Yes / No), Yes gave 0x1762 and the list of goods (yellow / red / tiger pinwheel + "I don't want it!"), picking
one gave "The seller sells 0x224c for 680 Bells", 0x1764 (several pages), the item crossing, then 0x1766 ("anything
else?", Oh yeah / No thanks). The goods left are saved with the game.
Unverified: villager chat (base + 3k + RANDOM(3)), second page of goods, 0x1763 / 0x1767, full pockets 0x175D, too
poor 0x175E, the sold-out line 0x1757, the day-by-day layouts other than 2, the save round trip.
Not built: fan and clap animations, the stall's model (a box), the seller's fan, the sounds, the original's hanabi
effect (the sparks are a stand-in), the pond-acre choice (the acre with most river tiles stands in).
