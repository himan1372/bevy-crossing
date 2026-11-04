# migrate-audit-1 — audit of the 52 modules ported in 0b1602ad..8dfd724a

Read-only. Source: pinned Rustimal 122867f7, original C in ACGC-PC-Port/src.

## Hygiene scan (all 52 files in `logic/src`)
- `no_mangle` / `extern "C"`: none. The ABI exports were removed.
- `unsafe`: 22 files keep `pc_*` out-pointer adapters over pure functions (aprilfool, balloon, birth_control,
  countdown, design, event_map_npc, famicom, fishing_tourney, hatsumode, kamakura, lost_found, melody,
  moon_viewing, mushroom, snowman, taisou, tamaire, tokyoso, tunahiki, villager_ai, wisp, yomise). aprilfool.rs
  documents them ("replaced when the module is wired in"). Retire each adapter when its module gets an app caller.
- Non-test `unwrap` / `expect` / `panic!` / `todo!`: none.
- App callers: only `totakeke` (kk_slider.rs) and `game_math` (other logic modules) are used. The other ~48 modules
  have no caller yet, so none is exercised outside its own tests.

## Fidelity spot check
- `gulliver.rs` against `ac_ev_dozaemon_move.c_inc`: talk message numbers (0x23F8 under RANDOM_F < 0.35, else
  0x23F9+RANDOM(5); forced 0x23FE+RANDOM(6); to_ageru 0x2404/0x2408; to_ageru2 0x2405/0x2409; give 0x2406/0x2407;
  wander 0x240A+RANDOM(6)) all match. The module holds only pure kernels; actor lifecycle, placement and the
  furniture selector are not ported and are the work for wiring Gulliver.
- Not checked: the other 50 modules. They stay unaudited until each is wired; check a module against its C when
  it gets a caller.

## Decision
Do not bulk-retire the adapters now (no callers to check). Audit per module at wiring time.
