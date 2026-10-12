# gx-backend-bevy

Run the disc's display lists through logic::graphics (Emu64) and draw them in Bevy.

- [x] RelGfx: GfxMemory over foresta.rel with relocation-patched Gfx (app/src/disc_gfx.rs)
- [x] GxBackend::set_state hook (logic, additive) -> triangles grouped by tile-0 texture
- [x] GX texture decoder (disc/src/gx_texture.rs): I4/I8/IA4/IA8/RGB565/RGB5A3/RGBA8/C4/C8/CMPR
- [x] Bevy mesh + StandardMaterial per part; --view/--pal debug viewer (app/src/model_view.rs)
- Known gaps (later tasks): palettes set by game code (pass --pal), TEV/prim/env colours, DOL pointers + runtime segments, culling, model scale unconfirmed.
