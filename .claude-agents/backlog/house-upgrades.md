# House upgrades: loan, Nook orders, sizes, basement, upstairs, statue

Philip 2026-10-20 chose this; depends on post-office-repay for paying.
Source map (decomp, via Haiku skim; verify before use): ac_npc_shop_common.c
aNSC_start_wait / set_talk_info_start_wait(1) (offers, orders, renewal loans),
m_home.c mHm_CheckRehouseOrder (runs at start/load only; order date != today),
ac_my_indoor.c aMI_size_dsp_table (S rom_myhome1, M 2B, L 3, LL1 4_1, LL2 4_2,
basement rom_myhome_ug; _new/_new2 variants), step models obj_myhome_step_*,
door items 0x4000.. for floor changes, m_huusui_room_ovl usable units
S6 M8 L10 UPPER8 B10, m_home_h.h mHm_hs_c floors[3] with per-floor
wall/floor, ac_my_house skeletons obj_{s,w}_myhome1..4, ac_douzou statue.
logic/src/house.rs already ports phases A/B/C and loans.

Stages: 1 save state + Nook offer/order/renewal + construction on load;
2 room model/area per size + exterior per size; 3 basement and upstairs
floors with stairs and per-floor saves; 4 statue.
Philip: existing saves start paid off (loan 0); new saves owe 17,400.

## Stage 1 (claude, 2026-10-20) - done

- app/src/home.rs: HouseSave (saved), PlayerHouse, construction at
  PreStartup/F9 (mHm_CheckRehouseOrder incl. roof palettes), nook_greeting
  (aNSC_start_wait/_wait/_wait1 incl. renewal loans, statue order/built,
  large-house auto basement), order_expansion/order_basement, look() for the
  town model.
- shop_nook: Nook speaks on entry when there is house business; Step::House
  follows the disc's choice branches; colour pages (3 colours + Another,
  page = count mod 4, palette = page*3 + choice), basement answer in
  REHOUSE_2_OFFER; messages: PUT_STRING_DETERMINATION (code 46) added.
- disc_town: house 0 drawn and measured at its size (myhome1..4) and roof
  palette (a..l).
Verified in Temp previews (real save unchanged): offer 0x107B -> colour
0x107C -> order; backdated order built at load (size 1, orange); Nook
0x107E billed 148,000; house entry works. Limitation: F9 to a save with a
different house keeps the town model until restart.
Next: stage 2 (room model/area per size: rom_myhome1/2B/3/4_1, usable
6/8/10/8 units, mHm_RehouseWallDoor).
