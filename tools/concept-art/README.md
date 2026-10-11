# Concept-art generator

Renders the GameCube-era style concept images (see `docs/art-direction.md`)
with Blender. Every texture is painted procedurally by `tex.py` (numpy);
no outside images or models are used.

```
blender -b --factory-startup --python tools/concept-art/build.py -- <scene> <out.png> [width height]
```

Scenes: `square`, `river`, `lineup`, `night`, `wide`, `chat`.

- `build2.py` + `softtex.py`: **current style** (v2: smooth, rounded,
  pastel; matches the Pots anchor in `docs/art-direction.md`).
- `build.py`: first blocky pass (v1), kept for comparison.
Tested with Blender 5.2 LTS (EEVEE). Characters and props here are
placeholders for the art pass, not final assets.
