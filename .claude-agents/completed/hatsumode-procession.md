# New Year's procession proper (`ac_hatumode_clip_think.c_inc` think table, `ac_hatumode_npc0_schedule.c_inc`, `ac_hatumode_clip_move.c_inc`, `ac_hatumode_control_move.c_inc`)

Rewrites `app/src/hatsumode.rs` from the standing villagers to the original's choreography: the 46-entry think table
(`dt_tbl`) run per villager (five of them) with its procs (ready, turn, flag waits, anime waits, timers, move waits,
greeting waits, to-omairi, player move / wait, kasasimai, sanpai wait), the shared flags (`flags0` 0x1000 / 0x2000 /
0x4000 / 0x8000, `flags1` pair bits), `cur_npc_idx`, the two loops of roots around the shrine (`aHTMD_next_position`),
the talk requests by think index (invitation at think 3 and 6, chats at 21 and 28), the player being walked behind the queue,
then to the front (`aHTMD_clip_player_move`) while the inviter steps on, then the shrine's own question / pay / prayer,
the coin when a villager throws (frame 33). Animations are timed pauses (throwing 2.6 s, praying 3.2 s, greeting 1.6 s).
Not: the real animations, the shrine bell (`anime_play_proc`), the red window colour, collision rerouting (`MOVE_BF`), the
New Year table, the throw-money / pray player animations.

- [ ] hatsumode.rs rewrite
- [ ] gates, Temp run Jan 1 07h, result, comms, commit

## Result
`app/src/hatsumode.rs` rewritten: the 46-entry think table, the procs / inits / flags and the two root loops as in the
original, five villagers cycling round the shrine (positions changing over 30 s of watching), the talk windows by think
(a log line "Villager n can be talked to at think k" marks each), the player's escort and the shrine's prompt.
Run in game (Temp save, disc, 1 Jan 2027 07h): chat 7727 (base + 12); an invitation 7661 -> Yes 7662 -> the player walked
behind the queue and to the front -> shrine question 4396 -> Yes 4397 -> "The New Year offering: 50 Bells" -> prayer ->
control back (the player walked afterwards).
Fixed on the way: answering used `dialogue.active`, which is false once a choice is made (the talk restarted instead).
Unverified: No at the invitation (base + 2) or at the shrine, no Bells (4398), the second leg of the walk checked only by
the prompt arriving, the greeting handshake (flags 0x1000 / 0x2000) beyond the villagers' thinks advancing, think 43 / 44
(the villager going behind after the prayer), a long run (several loops) for deadlocks.
Not built: the real animations (timed pauses), the shrine bell, red window, collision rerouting, the New Year table, the
player's own pray / throw-money animations.
