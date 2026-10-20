# Post office interior and loan repay

Philip 2026-10-20: build the post office repay first (before house-upgrades),
as in the original (ac_npc_post_girl menu choice 1 -> mSM_OVL_REPAY,
m_repay_ovl.c: money = wallet + money-bag items, max = min(money, loan)).

Scope: enter the post office from town, its disc room, the Post Girl at the
counter (day/night clerk as in retail), her menu, and the repay submenu
writing the player's loan. Letters/savings beyond what exists stay out of
scope unless needed by the menu. Loan state is shared with house-upgrades.
Philip: existing saves start with loan 0 (paid off); new saves owe 17,400.

## Result (claude, 2026-10-20)

- logic/src/post_office.rs: clerk day/night, music, aPG status/greeting/
  after-repay/result/business mapping, loan strings (FREE_STR1/2), RepayMenu
  (mRP init/press/commit incl. bag order 100/1000/10000/30000 and change).
- app/src/post_office.rs: door (aPOFF_check_player), room grd_post_office(T)
  at POST_ORIGIN with surface-sampled walls/counter, clerk NpcModel 241/256
  at unit (4,2)-20, talk via DialogueState::begin_special (name + voice id 4),
  only Repay/Quit answers offered, repay window (fkm_win_mode/model located
  after fkm_win_sw10T_model, opcodes checked; text per mRP_set_character_dl),
  RepayOpen focus, leave by E at the entry facing south.
- ShopLocation::PostOffice; environment indoor light, music scene, save
  return position for any interior; PlayerInventory.loan + SaveData.loan
  (serde default 0; new game DEBT_BUY_HOUSE).
Verified (Temp preview, real save unchanged): enter (W+D at the mat), Phyllis
greeting 0x8b2 with "17,400 Bells", repay window cash 15,000, paid 14,000 ->
loan 3,400, wallet 1,000, bag spent; 0x8d6 then 0x8dc; cancel -> 0x8e6; quit
0x8b4; F5 inside saved loan 3400 at the door; leave works. Facing straight up
does not open the door (strict retail range). Gates pass; no tests.
