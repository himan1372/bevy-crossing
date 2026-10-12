# asset-table

**Problem:** the renderer needs to find assets by name (and name what a pointer points at) inside `main.dol` / `foresta.rel`.

**Solution:** generate a compact symbol table (`disc/data/*.sym`) from the ac-decomp symbol maps (`tools/gen_symbols.py`; metadata only, no game bytes), and resolve it against `RomImage` into a catalog: name → (source, file offset, size), plus reverse lookup by file offset.

- [x] tools/gen_symbols.py + disc/data/{foresta,dol}.sym
- [x] disc/src/assets.rs: parse, resolve via REL sections / DOL section map, lookups
- [x] Synthetic tests; real-disc test checks known offsets from rustimal-crossing pc_assets.c (e.g. FONT_nes_tex_font1 = REL 0x4E1A40)
- [x] Gate passes
