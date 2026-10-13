# town-from-disc

Draw the town ground from the disc's acre templates instead of flat planes.

- [x] Acre templates `grd_s_*` picked from cliff/river edges (c1-7 / r1-7 = field_gen block-type order), variant per acre from seed (app/src/disc_town.rs)
- [x] Ground textures/palettes rebuilt in .bss buffers (`*_tex_dummy` <- `mFM_grd_s_*_tex`, `*_pal_dummy` <- monthly `mFM_*_pal`); winter `mFM_grd_w_*`
- [x] bss relocations tagged (BSS_TAG) and resolved via `Segments`
- [x] Translucent layer (`_modelT`) alpha-blended
- [x] logic fix: G_LOADTLUT TLUT name is 4 bits (was 3; data loads name 15)
- [x] Holes fixed: G_TRIN face-data words were read as commands (could look like G_ENDDL); all c/r/s/f templates now cover their acre
- [x] (superseded by Phase 2: retail block types from logic::random_field) Edge -> shape mapping is a guess: rivers/cliffs don't line up across acres. Proper fix: generate retail block types (logic::field_gen) instead of mapping the rewrite plan.
- [ ] Month -> palette table is judged from colours, not the game's calendar
- [x] Collision/heights from the templates (Phase 2, ground_map.rs)

## Phase 2 (Philip OK'd 2026-10-12): retail generator + gameplay migration
- [x] Port m_random_field_ovl.c (logic/src/random_field.rs + random_field_tables.rs; 3 tests) (+ needed parts of m_random_field.c) to logic::random_field; tables read from disc where they exist
- [x] Ground visuals from the retail combi table (all 70 acres incl. border, rail, beach, ocean, island) (bg via data_combi_table -> data_bgd)
- [x] Collision + ground height from data_bgd collision words (app/src/ground_map.rs); acres raised by tier
- [x] Trees/rocks/flowers from fgdata.bin layouts; player house at HOUSE0, shop at SHOP0, resident homes re-sited onto level ground
- [x] Draw disc models for trees/rocks/flowers/buildings (disc_town.rs, field_items.rs)
- [x] River mouth: grass bank no longer runs into the sea (Philip confirmed fixed, 2026-10-13)
- [ ] Retire town_gen as the town's source of truth

### Port notes (resume here)
- Source: shmanimal crossing/ACGC-PC-Port/src/game/m_random_field_ovl.c (entry mRF_MakeRandomField_ovl, ~55 fns) + m_random_field.c (combi lookup, island). Enum: include/m_field_make.h (108 block types, BLOCK 7x10).
- Plan: script-convert the static u8 tables (l_base_blocks, l_mRF_step3_blocks*, trace/next tables) to Rust consts; hand-port functions in file order into logic/src/random_field.rs.
- Selection: mRF_SelectBlock picks combi_info entries by block type; combi entry = mFM_combo_info_c (check struct in m_field_make.h); disc tables data_combi_table (368 x 6 B) / data_bgd (295 x 0x434, model ptr at +4).
