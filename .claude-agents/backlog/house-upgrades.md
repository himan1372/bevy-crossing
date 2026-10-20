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
