---
name: gc-texture-pipeline
description: Converts GameCube disc textures (TPL/BTI/TEX) into engine-ready PNGs with wimgt and lands them in assets/ correctly for Bevy. Extends asset-pipeline — read that skill first for repo asset conventions. Use after gc-iso-reader or iso-reader has extracted texture files from the disc, before wiring them into materials or UI.
---

# gc-texture-pipeline

> **This skill extends `asset-pipeline`.** It does not replace it: `asset-pipeline` owns the repo's asset conventions (where files live, naming, LFS), `gc-iso-reader`/`iso-reader` own *extracting* files from the GameCube disc. This skill owns the missing middle step: turning extracted GameCube texture blobs into usable images.

## When to use / when not to

- **Use:** you have extracted `.tpl`, `.bti`, or `.tex` files from the disc (via the ISO-reader skills) and need PNGs in `assets/` for Bevy materials, sprites, or UI.
- **Don't use:** for Blender-authored models/textures — that's `blender-gltf` + `asset-pipeline`. For disc *audio*, see `gc-audio-pipeline`.

## Step 1 — Know what you're holding (format primer)

GameCube textures come in a few container and pixel formats. Identify both before converting:

**Containers:** `TPL` (texture palette library, the common one), `TEX`, `BTI` (often embedded in BMD/BDL model files).

**Pixel formats:** `I4`, `I8`, `IA4`, `IA8` (intensity / intensity+alpha), `RGB565`, `RGB5A3`, `RGBA32`, `C4`/`C8`/`C14X2` (paletted — need an external palette), `CMPR` (S3TC-style block compression, the most common for large textures).

**Palette formats** (for paletted textures): `P-IA8`, `P-RGB565`, `P-RGB5A3`.

**Attributes** you'll see in tool output: `PALETTE`/`NO-PALETTE`, `GRAY`/`COLOR`, `ALPHA`/`NO-ALPHA`.

If a texture renders as garbage after conversion, the usual cause is a wrong pixel-format or palette assumption — go back to this identification step before anything else.

## Step 2 — Convert with wimgt

The converter is **wimgt** (Wiimms Image Tool), a batch CLI converter for GameCube/Wii image formats. Full command reference: https://szs.wiimm.de/wimgt/

Recipe shape:

1. Point wimgt at the extracted files and convert to PNG in batch.
2. Use `--transform` rules to control per-format handling. Rule examples from the wimgt docs:
   - `TPL.CMPR=PNG` — decode CMPR-compressed textures inside TPL containers to PNG.
   - `TPL.I4+I8=IA8` — normalize I4/I8 intensity formats up to IA8.
   - Palette handling: match the palette format to what the texture expects (`P-IA8`, `P-RGB565`, `P-RGB5A3`); a paletted texture converted with the wrong palette format comes out with wrong colors, not an error — verify visually.
   - `GRAY`/`COLOR`/`ALPHA` attributes select which channels to keep when you need e.g. a separate alpha or grayscale map.
3. Preserve mipmaps where the texture has them (check the wimgt docs for the mipmap options); UI textures usually don't need them, world textures do.

Checklist before moving on:

- [ ] Every source file produced a PNG (count in = count out; wimgt skips files it can't parse — find out which and why).
- [ ] Spot-check visually: open 3–5 converted PNGs, including at least one paletted and one CMPR texture. Wrong colors = wrong palette/format assumption, not a Bevy problem.
- [ ] File sizes are sane (a 256×256 RGBA32 texture is ~256 KB uncompressed; wildly smaller or larger means the format was misread).

## Step 3 — Land them in `assets/`

Follow `asset-pipeline`'s naming and directory conventions, plus:

- Keep the disc's logical grouping: `assets/textures/<area-or-system>/` (e.g. UI, town, characters). Don't dump everything in one folder — the next agent needs to find things.
- Name files after their in-disc identity where known (`<original-name>.png`), so they can be traced back to the ISO entry via `gc-iso-reader`.
- Commit a short conversion log next to the batch (what source files, which transform rules, date, agent): future reconversions need the exact recipe.

## Step 4 — Bevy-side gotchas

1. **UV origin flip.** Bevy UVs are top-left origin; GameCube textures may assume the opposite. If a texture appears vertically flipped on a mesh, flip the image (or the UVs) — don't "fix" it by editing the model. Reference: [bevy-cheatbook texture config](https://Bevy-cheatbook.github.io/setup/bevy-config.html) and the [UV troubleshooting notes](https://deepwiki.com/bevy-cheatbook/bevy-cheatbook/8-troubleshooting-and-common-issues).
2. **GPU compression decision.** Bevy's `basis-universal` feature enables Basis Universal / KTX2 GPU-compressed textures. Decide per texture class: compress large world textures, keep UI textures uncompressed for crispness. Document the choice in the conversion log.
3. **sRGB vs linear.** Color (albedo) textures are sRGB; data textures (masks, roughness) are linear. Setting this wrong washes out or darkens the texture — set it deliberately in the material, not by trial and error.

## Step 5 — Verify in the running game

Don't trust the PNG on disk — verify on screen via the BRP agent-loop runbook (`brp-visual-qa` covers the screenshot recipe):

1. Load the texture onto its target (material, sprite, or UI element).
2. Screenshot via BRP and compare against the source PNG: correct image, correct orientation, correct colors.
3. If it's wrong, work backwards: Bevy material settings → UV flip → conversion recipe → format identification. Most failures are in the first two.

## Models note (BMD/BDL)

Disc *models* (BMD, and BDL = BMD + MDL3 display-list section, per the SuperBMD wiki) are a separate conversion path: SuperBMD → DAE → Blender/glTF → the existing `blender-gltf` pipeline. This skill doesn't cover that path; it only notes that BTI textures embedded in model files go through Steps 1–2 the same way. Reference: [SuperBMD binary-model wiki](https://github-wiki-see.page/m/Sage-of-Mirrors/SuperBMD/wiki/Binary-Model).

## Sources

- [wimgt — Wiimms Image Tool](https://szs.wiimm.de/wimgt/)
- [bevy-cheatbook — texture/image configuration](https://Bevy-cheatbook.github.io/setup/bevy-config.html)
- [SuperBMD — binary model wiki](https://github-wiki-see.page/m/Sage-of-Mirrors/SuperBMD/wiki/Binary-Model)
