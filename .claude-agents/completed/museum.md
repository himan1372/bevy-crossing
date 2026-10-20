# Museum

Philip 2026-10-20: "lets get the museum up". Enter the museum from town,
its disc rooms (lobby and exhibit rooms), Blathers, donating (fish,
insects, fossils, paintings) as the original does, the saved collection,
and the exhibits showing what was donated. Reuse indoor_room / the post
office and police box patterns.

## Result (claude, 2026-10-20) - first stages

Source map via Haiku skim; code checked: ac_museum door, scene door data,
FG door items (FG 0x17F-0x183 dumped), ac_npc_curator talk/donate tables,
m_museum_display ranges/tables, ac_museum_picture.
- logic/src/museum.rs: Collection (donors), exhibit(), offer() answers,
  asleep(), greeting().
- app/src/museum.rs: door, rooms via indoor_room, FG door walking, Blathers
  (NpcModel 347, voice 2) talk -> pockets donate mode (PocketsUi.donating),
  fossil plinths/pieces via furniture::spawn_furniture + solid footprints,
  paintings / covered frames; MuseumSave in the save.
- ShopLocation::Museum, Scene::Museum (BGM 88).
Verified (Temp preview, real save unchanged): enter, lobby, sleepy greeting,
donate fish/fossil/painting with the right answers, gallery shows the art,
fossil room shows the skull and blocks walking, save round trip.
Still to do: fish tanks (ac_museum_fish), insect cases (ac_museum_insect),
mail appraisal of fossils, after-talk hints (0x2F50 choice 2), completion
mail, Blathers' sleep animation.
