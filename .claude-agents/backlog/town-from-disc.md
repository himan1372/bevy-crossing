# town-from-disc

Draw the town ground from the disc's acre templates instead of flat planes.

- [x] Acre templates `grd_s_*` picked from cliff/river edges (c1-7 / r1-7 = field_gen block-type order), variant per acre from seed (app/src/disc_town.rs)
- [x] Ground textures/palettes rebuilt in .bss buffers (`*_tex_dummy` <- `mFM_grd_s_*_tex`, `*_pal_dummy` <- monthly `mFM_*_pal`); winter `mFM_grd_w_*`
- [x] bss relocations tagged (BSS_TAG) and resolved via `Segments`
- [x] Translucent layer (`_modelT`) alpha-blended
- [x] logic fix: G_LOADTLUT TLUT name is 4 bits (was 3; data loads name 15)
- [ ] Holes: some acres have missing ground triangles (dark patches in the overhead view)
- [ ] Edge -> shape mapping is a guess: rivers/cliffs don't line up across acres. Proper fix: generate retail block types (logic::field_gen) instead of mapping the rewrite plan.
- [ ] Month -> palette table is judged from colours, not the game's calendar
- [ ] Collision/heights still from the town plan, not the templates
