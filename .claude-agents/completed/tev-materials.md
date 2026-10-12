# tev-materials

Draw disc models with the game's own colour combiner instead of a plain texture.

- [x] PartKey groups triangles by texture + combiner + prim/env + lighting + cycle type (app/src/disc_gfx.rs)
- [x] N64Material + n64_combiner.wgsl: (a-b)*c+d per cycle, RDP mux tables, 2-cycle (app/src/n64_material.rs)
- [x] Shade = vertex colour, or simple directional+ambient light when G_LIGHTING (vertex normals)
- [x] Interpreter command budget (logic, additive) so looping lists can't hang the game
- [x] Prim colour starts white (game code normally sets it)
- Gaps: TEXEL1/second tile, noise, LOD, fog, blender modes (alpha is a 0.5 cutoff), runtime segments 8-15 (wallpaper/carpet etc.), palettes from game code.
