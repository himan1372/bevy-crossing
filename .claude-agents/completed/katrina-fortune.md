# katrina-fortune — Katrina the fortune teller

Source: `ac_ev_gypsy_move.c_inc`, `ac_event_manager.c` (`gypsy_start`), kernel `logic/src/event_visitors.rs`.
Appears while `ev::GYPSY` is active (21:00 to 05:59 next day, see `special-event-schedule`).
Talk: fortune already given this visit -> 0x982; destiny normal -> 0x970 (question Yes/No); otherwise 0x97B.
Yes with >= 50 Bells -> 0x973, then pay 50, four random strings (0x1A4 + r32 adjective, 0x1C4 noun, 0x184 verb,
0x164 place -> message item strings 0..3), roll RANDOM(10) -> destiny and 0x974 + index; Yes without Bells -> 0x972;
No -> 0x971. A saved non-normal destiny: 0x97B then 0x97C + destiny - 1. The destiny resets to normal on a new day
(`Game_play_Reset_destiny`).
Port: destiny and its day saved in `PlayerIdentity` (`destiny`, `destiny_day`); `fortune_given` is per visit.
Not built: the fortune tent and its lot (she stands on free ground, like Joan), the destiny's effects on prices and
goods (`mPr_GetMoneyPower` / `GetGoodsPower` have no port equivalent), the demo-order timing of the continue messages.

- [ ] app/src/katrina.rs: spawn, talk, destiny save and daily reset.
- [ ] check / clippy / fmt; Temp run with the disc: pay, fortune, destiny saved, saved-destiny path, no-Bells path.

## Result
`app/src/katrina.rs` (`KatrinaPlugin`): spawn while `ev::GYPSY` is active (the day before counts after midnight),
on `joan::spot` (now `pub(crate)`), talk (`Talk`, `Step`: CallIn, Deciding, Remind, Close), destiny
(`PlayerIdentity.destiny` / `destiny_day`, reset to normal on a new day by `reset_destiny`).
Temp runs with the disc (synthetic saves, calendar GYPSY today, cap build, BRP keys + screenshots):
- Yes with 200 Bells: 0x970 (question), 0x973, fee paid (F5 save: bells 150), roll 6 -> destiny 2, message 0x976,
  four words drawn ("stressed-out", "bathroom sink", "coughing", "On the beach"), F5 saved destiny 2 + day;
  talking again the same visit: 0x982.
- Relaunch with that save: 0x97B then 0x97D (saved destiny 2 -> 0x97C + 1).
- 10 Bells, normal destiny: 0x970 then Yes -> 0x972.
Not driven: the No answer (0x971, same code as the others), the word pages of the other fortunes (screenshot only
showed the unpopular one), a talk after midnight. Not built: the tent, destiny effects on prices/goods.
check / clippy -D warnings / fmt pass; no tests; real save untouched.
