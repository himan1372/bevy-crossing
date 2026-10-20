# Police box interior

Philip 2026-10-20 chose this after the statues. Enter the police box from
town, its disc room, the officers (Copper/Booker as in the original), their
talk, and the lost-and-found (mPB_keep_item list) as far as the source
supports. Follow the post office pattern (ShopLocation variant, disc room
surface walk mask, special NPC model, disc messages, original door check).

## Result (claude, 2026-10-20)

Source map via Haiku skim (verified in code): ac_police_box_move door,
police_box scene, mvactor Booker (4,6), ac_npc_police2 talk, m_police_box.
- app/src/indoor_room.rs: shared disc-room build + surface walk mask +
  height_at; post office moved onto it.
- app/src/police_box.rs: door (walk in or E), room, Booker (NpcModel 254,
  voice 3), greeting/talk ids, LostAndFound (saved police_box), init with
  goods::select furniture 1 + cloth 2, items on RSV_POLICE_ITEM units
  (FG_TYPE_POLICE_INDOOR 0xCE) at surface height, A-facing offer 0x77E.
- ShopLocation::PoliceBox, Scene::PoliceBox (BGM 45).
Verified (Temp preview, real save unchanged): enter by walking, greeting
0x785, items on the back table, claim -> pockets and slot cleared, leave,
save keeps the list. Not done: Copper outside (his menu: event hint, lost
and found count), other feeds into the lost and found.
