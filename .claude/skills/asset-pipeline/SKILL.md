---
name: asset-pipeline
description: End-to-end 3D asset pipeline for bevy-crossing: Blender 5.2 LTS → .glb → Bevy 0.19. Use when creating a new asset, exporting from Blender, importing into Bevy, or debugging an asset that looks wrong in-game. Pinned export settings, naming contracts, and verification steps.
---

# Asset pipeline: Blender → Bevy 0.19

The one path a model takes from Blender to the game. For deeper dives, see
`blender-gltf` (export flags) and `bevy-animation` (import + playback).

## Toolchain

- **Blender 5.2 LTS** (released 2026-07-14, fixes until July 2028). Pin this
  version — non-LTS releases routinely change node and operator names, which
  breaks scripted exports.
- **Bevy 0.19.** glTF loads via `SceneRoot` or
  `asset_server.load("models/foo.glb")`. The `gltf_animation` feature is on by
  default; no opt-in needed.

## Step 1 — Author

- Model in **meters**. Scene Properties → Unit Scale must be **1.0**.
- **Name things deliberately.** Object, bone, and action names survive export
  and become the handles game code uses — Bevy 0.19's
  `AnimationTargetId::from_name` resolves bones by `Name`, so sloppy Blender
  naming is a load-bearing bug.
- **Materials:** Principled BSDF maps to Bevy's `StandardMaterial`
  reasonably; anything else does not. Bake procedural materials to textures —
  Bevy does not evaluate Blender shader nodes.
- **Decide generate vs import:** reach for Blender when the thing genuinely
  has *form* (a modelled object, a rigged character, an animation). Reach for
  generated meshes / materials when it has only pattern (no asset to load, no
  cache to invalidate).

## Step 2 — Export (.glb)

Never hand-export from the GUI for shipped assets — script it so the settings
can't drift. Pinned export call:

```python
bpy.ops.export_scene.gltf(
    filepath="assets/models/foo.glb",
    export_format='GLB',
    use_active_scene=True,      # never export the whole .blend
    export_yup=True,            # Blender is Z-up; Bevy is +Y-up
    export_apply=False,         # True SILENTLY DELETES shape keys
    export_morph=True,
    export_morph_normal=True,
    export_skins=True,
    export_all_influences=False,  # clamp to 4 influences/vertex
    export_animations=True,
    export_force_sampling=True,   # bake drivers / constraints / IK
    export_animation_mode='ACTIONS',  # enum, not bool
)
```

Pre-export checklist:

- [ ] Transforms applied (`Ctrl+A → All Transforms`).
- [ ] Axis conversion left at default (+Y up) — don't "fix" it twice.
- [ ] Drivers/constraints/IK baked (`export_force_sampling=True`).
- [ ] Shape keys intact (`export_apply=False`).

## Step 3 — Import (Bevy 0.19)

- Single `.glb` is easiest — the loader resolves the buffer inline.
- Animations: `asset_server.load("models/character.glb#Animation0")`,
  `#Animation1`, … — one index per Blender action, zero-indexed.
- The glTF loader puts `Name`, `AnimationTargetId`, and `AnimatedBy` on bone
  entities automatically. Hand-built rigs must add `Name` themselves.
- **Bevy 0.19 culls skinned meshes against animated bounds**
  (`DynamicSkinnedMeshBounds`). An animated mesh disappearing at the frame
  edge is a real bug to report, not rest-pose behaviour.
- `KHR_animation_pointer` (animating material properties via glTF) is **not
  implemented** — bone transforms only.

## Step 4 — Verify

1. `gltf-validator` on the `.glb` — zero errors.
2. Standalone viewer check (isolates exporter vs importer issues).
3. Bevy `SceneRoot` smoke test: correct transforms, materials, animations
   playing, no culling pop at frame edges.

## See also

- `blender-gltf` — full export-flag reference and the bake-or-rebuild rule.
- `bevy-animation` — animation graphs, blending, `AnimationEvent`s, the 12
  principles.
- `bevy-019` — Bevy 0.19 API patterns and the cargo gate.
