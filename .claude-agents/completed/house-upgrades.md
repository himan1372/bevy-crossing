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

## Stage 2 (claude, 2026-10-20) - done

home.rs room helpers (room_model/room_centre/room_half/room_last_unit by
size: rom_myhome1 6, 2B 8, 3 10, 4_1 10 units); interior.rs, house.rs
bounds/entrance, player_room corner and unit ranges (placement, loose
items), lighting/sunshine coordinates and floor list, decorating checks use
them. Verified Medium room (8x8) renders with saved patterns; furniture at
(1,1) and (6,6); walking reaches the new walls. Not checked: window
sunshine spots in bigger rooms, _new/_new2 (stair) variants (stage 3).

## Stage 3 (claude, 2026-10-20) - done

home.rs: floor() / set_floor (0 main, 1 LL2, 2 basement), room_lists per
floor (basement single model), stairs()/Stair::span/stair_pos (aMI tables).
player_room/floors.rs: HomeFloors (other floors' RoomSave), stair models,
climb (south into the stair span, re-armed after releasing down), snapshot/
restore swap carrying diary + music box, empty floors start with default
decoration. PlayerRoomShell keys (decoration, floor) so decorating::render
rebuilds the shell. house.rs refresh_player_room_bounds; non-main floors
have no front door. Save: home_floors [upper, basement]; F9 shows main.
Verified (Temp previews, real save unchanged): LL1 room with both stairs,
basement and LL2 render, transitions both ways, saves. Not checked: room
light / sunshine / insects per floor beyond not crashing (sunshine is main
floor only).

## Stage 4 (claude, 2026-10-20) - done; task complete

app/src/statue.rs (ac_douzou): RetailLayout.statue = DOUZOU (0x5843) unit;
house 0's statue at +(0, 200) game units when next_size == STATUE; skeleton
cKF_bs_r_obj_{s,w}_douzou posed at frame 1, boy joints 2-4 (no player sex in
the port), girl hidden, rank scale on joint 2 and prim/env per rank on the
figure and plate, segs 8/9 eye/mouth by face, A = structure palette 89;
StatueUnit blocks town movement. Verified gold and silver renders and the
block (Temp preview, real save unchanged). Not done: sparkle effect
(ef_douzou_light), idle clip animation, other houses' statues.
