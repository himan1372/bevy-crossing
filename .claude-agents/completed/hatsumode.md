# New Year's procession (hatsumode: `ac_hatumode_control`, `ac_hatumode_npc0`, clip move/think)

Scope (a reduced port of the 45-step script): Jan 1 06:00-09:59 (NEW_YEARS_DAY): five villagers stand at the shrine front
(units (7,9) (6,10) (9,10) (6,12) (9,12), nudged by the original's tables, facing north); the first asks the player to
pray with them (`base + 0`, Yes / No; yes `base + 1`, no `base + 2`); on yes the player walks to (20,120) and then (20,85)
units from the well (2 units a tick), the shrine's own prompt follows: with 50 Bells `MSG_4396` (0x112C) then yes `0x112D`
(50 Bells are paid, the coin is thrown, a pause for the prayer) or no `0x112E`; without the money `MSG_4398` (0x112E). The
others chat: `base + 3/6/9/12 + RANDOM(3)` by their place in the queue (the first villager's turn). Bases by looks: 7679
7697 7661 7715 7733 7751.
Not built: the villagers' own queue walking and bows (45 think states), the pray and coin-throw animations, the shrine's
bell, the second and third praying positions of the original, the red message window.

- [ ] hatsumode.rs, tortimer.rs well helper, main
- [ ] gates, Temp run Jan 1 07:xx, result, comms, commit

## Result
`app/src/hatsumode.rs` (a reduced port). Run in game (Temp save, disc, 1 Jan 2027 07:xx): five villagers (on their own
models) stand north-facing before the shrine; the first one's talk is the invitation (7715, Yes / No "You here to make a
New Year's wish, too?"); Yes gave 7716, then the player was walked to the shrine front (the leg to (20,120) then (20,85)
units from the well), the shrine's own question 4396 ("Will you throw money into the wishing well?"), Yes gave 4397
(0x112D), 50 Bells were paid ("The New Year offering: 50 Bells"), a 3 s prayer with the player held, then control back.
Unverified: No at either question (7717 / 0x112E), the no-money line 4398, the other villagers' chat, the held player at
other frame orders (the walk set `InputFocus::Transition` each frame and it held).
Not built: the villagers' own queue walks and prayers (45 think states), the prayer and coin animations, the shrine
bell and the red message window, the New Year table (a structure), the original's second and third walking legs.
