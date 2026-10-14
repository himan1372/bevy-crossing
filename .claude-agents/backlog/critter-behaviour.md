# critter-behaviour — fish and bugs move and bite like the game

The models are from the disc (critters-from-disc), but movement, the bite
and catching are still ecology.rs's simplified versions.

- [x] Fish: port ac_gyo_test.c (swim/wait/near/touch/bite/escape, aGTT_* tables) using logic:: where it exists.
- [x] Fish spawning: ac_set_ovl_gyoei placement (one per acre with water, season table filtered by the acre's water kinds, interior water unit; sea/offing fish not yet).
- [ ] Bugs: port the ac_ins_* behaviours per family. Done: insect placement (ac_set_ovl_insect as logic::ins_set: per acre on entry, habitat units, swarms, nothing-rolls) and butterflies (ac_ins_chou as logic::ins_ai: circle, wander, land on white pansies, flee, fade out). Also done: semi (cicadas), tonbo (dragonflies), kabuto (tree beetles), batta (grasshoppers/crickets), ka (mosquitoes; sting is a logged event, no player reaction yet), amenbo (pondskaters). Left: tentou, hotaru, mino, kera, dango, goki, hitodama. Gaps: no weather (rain/snow), buried items (deposit) and town rank (6 assumed); month blending of tables.
- [x] Hooked fish switch to their model (aGYO_actor_draw_fish): landed fish held at the hand, billboarded, swung and flapping, shrunk on putaway. Sea fish placed on deep sea units (sea table for coastal acres, pond table for still water).
- [ ] Gate passes; live check; Philip playtest.

Progress (claude): logic::gyo_ai (wait/swim/near/touch/bite/escape, flow from river attributes, splash and running-player scares); ecology drives it; bobber holds the hooked fish; E while biting catches. Live: SmallBass took the bobber and bit. Bugs, spawn placement and the hooked-fish model still open.
