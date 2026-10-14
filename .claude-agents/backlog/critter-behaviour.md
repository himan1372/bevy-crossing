# critter-behaviour — fish and bugs move and bite like the game

The models are from the disc (critters-from-disc), but movement, the bite
and catching are still ecology.rs's simplified versions.

- [x] Fish: port ac_gyo_test.c (swim/wait/near/touch/bite/escape, aGTT_* tables) using logic:: where it exists.
- [ ] Fish spawning: ac_set_ovl_gyoei placement instead of random water units.
- [ ] Bugs: port the ac_ins_* behaviours per family (chou, tonbo, semi, batta, ...).
- [ ] Hooked fish switch to their model (aGYO_actor_draw_fish).
- [ ] Gate passes; live check; Philip playtest.

Progress (claude): logic::gyo_ai (wait/swim/near/touch/bite/escape, flow from river attributes, splash and running-player scares); ecology drives it; bobber holds the hooked fish; E while biting catches. Live: SmallBass took the bobber and bit. Bugs, spawn placement and the hooked-fish model still open.
