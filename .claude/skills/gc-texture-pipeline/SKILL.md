---
name: gc-texture-pipeline
description: "Decode and diagnose runtime disc textures, palettes, UVs and combiner materials in bevy-crossing; use for original art rendering faults."
---

# Runtime disc textures

Consult [project workflow](../references/project-workflow.md) once per task for claims, source authority, save isolation and validation boundaries.

This project reads N64-derived Gfx display lists and texture data from the disc at runtime. Begin with the existing decoder/material path, rather than an offline PNG or GLB import pipeline.

## Trace one asset

- Locate the symbol, segment mapping and active display list in `app/src/disc_gfx.rs`, backed by `disc/src/rom.rs` and the asset catalog.
- Trace `TexKey`: image address, format, dimensions, wrap modes and TLUT address. Check the palette bank and source tile commands, not just the final image bytes.
- Follow decoding through `disc/src/gx_texture.rs` and `logic/src/graphics/texture.rs`, then material construction in `app/src/n64_material.rs` and `app/src/n64_combiner.wgsl`.

Most assets use GX-tiled data, but some legacy lists use row-major N64 IA data and old tile commands. Confirm the active path before choosing a decoder. N64 and GX intensity/alpha format names are not interchangeable. A plausible decode with wrong dimensions or palette can still look almost correct.

## Diagnose the layer that differs

Check address/segment bounds, tile layout, palette, alpha, UV scale, wrapping, primitive/environment colors and combiner selection separately. `texture_image` currently uses `Rgba8Unorm`; the combiner handles color conversion. A blanket sRGB change or image/UV flip can break other assets.

For pocket/menu masking, inspect the scoped inventory display-list runner and pattern material behavior before changing global decoding. Keep caches keyed by all relevant texture state and preserve no-disc fallback.

Compare the source commands and the rendered asset through [visual QA](../brp-visual-qa/SKILL.md). Any diagnostic PNG belongs in Temp. Do not install conversion tools, export whole banks, normalize palettes or commit derived assets just to inspect a texture fault.
