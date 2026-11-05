# The island's cottages and its resident

- `disc_town.rs`: the island's two cottages (`COTTAGE_MY` 0x5850, `COTTAGE_NPC` 0x5851: the disc's `obj_s_myhome_i` / `obj_s_house_i`
  models, palettes 96 / 97, +20 / +20 game units) are read from the island blocks' FG records (blocks (4,8) and (5,8), acres (3,7) /
  (4,7)) and drawn like the other structures; they have no winter model, so the summer one takes the winter palette.
- `roster.rs`: `VillagerTables::islander(seed)` is `mNpc_DecideIslandNpc` (the first 18 villagers whose grow permission is
  islander, picked by the town seed). `npc.rs`: that villager (here Faith, #228 in the test town) is spawned with the town's own
  villagers, living at the NPC cottage (`IslandNpc` names them); wandering stays on the island's ground.
- `dialogue.rs`: the island resident's chat is the island table (`aQMgr_decide_normal_msg_no_island`: 0x3631 0x3654 0x36BC 0x36DB
  0x367A 0x369D by personality + RANDOM(10), never the same twice running), not the town's talk table.
- `house.rs`: the resident's room is built like every villager's (`spawn_villager_room`), and the player's cottage has a room of the
  player's kind (bare, default decoration) behind its door; doors carry their room bounds now.
Run in game (Temp save, disc, 1 Jan 2027): both cottages stand on the island; Faith talks ("Hey, Player! I was won ...", msg 0x3639);
E at her door shows her toy-themed room; E at the player's cottage door shows the bare cottage room and E at its doorway leaves.
Unverified: the player's cottage' real room (`PLAYER_ROOM_ISLAND` is the cottage-sized room; the port shows the player's own room
size), the island furniture save (`island.cottage`), the resident's schedule / sleeping, her furniture-trade and mail talk
(`aQMgr_talk_island_*`: trade, "best furniture", letters), hello lines by feeling, her greeting at the boat, the doors' exact
positions against the originals (south-facing here), the lights of the cottages' windows at night.
Not built: those talks (furniture trade, player and other-player lines, mail), the island's own item layout from the island save
(trees, flowers, the flag), naming the island, the island's save (so nothing is kept between visits).
