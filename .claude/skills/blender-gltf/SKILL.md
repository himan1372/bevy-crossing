---
name: blender-gltf
description: Export Blender models to glTF/.glb for Bevy 0.19. Use when authoring, exporting or debugging 3D assets, when a model imports wrong in Bevy, or when deciding what glTF can and cannot carry. Covers the exact bpy.ops.export_scene.gltf flags, the bake-or-rebuild rule, and verification steps.
---

# Blender → glTF for Bevy 0.19

Adapted from `ladvien/skillz` → `procedural-animation-assets`
(https://github.com/ladvien/skillz). Focused here on the export pipeline;
see the upstream repo for the full procedural-animation decision framework.

## The one rule

glTF carries only **three motion types** natively: node TRS (translation /
rotation / scale), skinning, and morph weights. Everything else — drivers,
constraints, IK, geometry nodes, physics sims — must be **baked to keyframes**
or **rebuilt in engine code**. Bake it or rebuild it; there is no third option.

## Export flags that bite

Real flag names and defaults for `bpy.ops.export_scene.gltf`:

- **Morph:** `export_morph=True`, `export_morph_normal=True`. ⚠️
  **`export_apply=False` — turning it on silently deletes every shape key**
  (it applies modifiers, which is incompatible with morph export). This is the
  single most damaging default to get wrong.
- **Skin:** `export_skins=True`, `export_all_influences=False` (clamps to 4
  influences/vertex, which is what most engines read).
- **Animation / drivers:** `export_animations=True`, and
  **`export_force_sampling=True`** — this is what **bakes drivers /
  constraints / IK into keyframes**. Without it, glTF (which has no drivers)
  drops that motion entirely.
- **Always:** `use_active_scene=True` — without it the exporter walks **every
  scene in the .blend** and drags in unrelated geometry. `export_yup=True`
  (Blender is Z-up; glTF/Bevy are +Y-up).
- **Watch the types:** `export_vertex_color` (default `'MATERIAL'`) and
  `export_animation_mode` (default `'ACTIONS'`) are **enums, not booleans** —
  passing `True`/`False` misbehaves.

## Before exporting

- **Apply transforms** (`Ctrl+A → All Transforms`). A non-uniform scale on an
  armature or a parented mesh is the most common cause of an asset that looks
  right in Blender and wrong in engine.
- **Name things deliberately.** Object, bone, and action names come across and
  become the handles the game code uses (`AnimationTargetId::from_name` in
  Bevy 0.19 resolves bones by name).

## Verify — numbers *and* eyes

1. Run `gltf-validator` on the exported `.glb`; fix errors, not just warnings.
2. Open it in a standalone viewer (e.g. donmccurdy's glTF viewer) before it
   ever touches Bevy — isolates exporter problems from importer problems.
3. Load in Bevy via `SceneRoot` and eyeball it: transforms, materials,
   animations playing, no disappearing meshes at frame edges (Bevy 0.19 culls
   skinned meshes against animated bounds).

## See also

- `asset-pipeline` — the pinned end-to-end pipeline for this project
  (Blender 5.2 LTS → .glb → Bevy 0.19), with the exact export script.
- `bevy-animation` — Bevy 0.19 animation import, graphs, and events.
