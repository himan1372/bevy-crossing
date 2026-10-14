# critter-behaviour — fish and bugs move and bite like the game

The models are from the disc (critters-from-disc), but movement, the bite
and catching are still ecology.rs's simplified versions.

- [x] Fish: port ac_gyo_test.c (swim/wait/near/touch/bite/escape, aGTT_* tables) using logic:: where it exists.
- [x] Fish spawning: ac_set_ovl_gyoei placement (one per acre with water, season table filtered by the acre's water kinds, interior water unit; sea/offing fish not yet).
- [x] Bugs: port the ac_ins_* behaviours per family. Done: insect placement (ac_set_ovl_insect as logic::ins_set: per acre on entry, habitat units, swarms, nothing-rolls) and butterflies (ac_ins_chou as logic::ins_ai: circle, wander, land on white pansies, flee, fade out). Also done: semi (cicadas), tonbo (dragonflies), kabuto (tree beetles), batta (grasshoppers/crickets), ka (mosquitoes; sting is a logged event, no player reaction yet), amenbo (pondskaters), tentou (ladybugs, mantis, snail), hotaru (fireflies; glow() exposed, no point light drawn yet), dango (pill bugs; appear only when a rock is struck, and the app has no axe/shovel yet). goki (cockroaches), mino (bagworms, spiders; need tree shaking, which the app lacks). kera (mole crickets), hitodama (Wisp spirits). All 14 families done. Gaps: no weather (rain/snow), buried items (deposit) and town rank (6 assumed); month blending of tables.
- [x] Hooked fish switch to their model (aGYO_actor_draw_fish): landed fish held at the hand, billboarded, swung and flapping, shrunk on putaway. Sea fish placed on deep sea units (sea table for coastal acres, pond table for still water).
- [ ] Gate passes; live check; Philip playtest.

Progress (claude): logic::gyo_ai (wait/swim/near/touch/bite/escape, flow from river attributes, splash and running-player scares); ecology drives it; bobber holds the hooked fish; E while biting catches. Live: SmallBass took the bobber and bit. Bugs, spawn placement and the hooked-fish model still open.

Follow-ups (not this task): firefly/spirit point lights (glow() exposed); tree shaking, axe and shovel (cicadas/beetles/bagworms/spiders/pill bugs/mole crickets react to them via World fields the app leaves empty); weather (rain/snow); mosquito sting reaction; the Wisp event (World::wisp_night); ants via make_ant_proc.
