---
name: asset-pipeline
description: Blender → .glb asset pipeline for bevy-crossing (Bevy 0.19). Use when exporting models from Blender, importing .glb files into the game, debugging wrong scale/orientation/materials, adding skeletal animations, or validating a new asset. For the C/Rust ROM-asset pipeline, see rustimal-crossing's asset-pipeline skill instead — different system, same skill name, different repo.
---

# Asset Pipeline — Blender → .glb → Bevy 0.19

End to end:

```
Blender (.blend, metres, +Y-up export) → glTF 2.0 Binary (.glb)
  → assets/models/<category>/<name>.glb
  → AssetServer + GltfAssetLabel → WorldAssetRoot spawn
  → post-spawn hookup (materials, colliders, markers)
```

Phase 1 uses primitives; no `.glb` assets exist yet. These conventions exist so
the first import lands cleanly. **Validate one asset end to end before
batch-exporting the rest.**

## 1. Bevy 0.19 import conventions (the receiving end)

### Loading a model

```rust
use bevy::prelude::*;

fn spawn_house(assets: Res<AssetServer>, mut commands: Commands) {
    // Scene 0 of the .glb, as a WorldAsset (Bevy 0.19 form)
    let scene: Handle<WorldAsset> =
        assets.load(GltfAssetLabel::Scene(0).from_asset("models/house.glb"));
    commands.spawn((
        Name::new("House"),
        WorldAssetRoot(scene),
        Transform::default(),
        HouseMarker, // dedicated marker component — never string-match names in game code
    ));
}
```

- `GltfAssetLabel` also addresses `Mesh`, `Primitive { mesh, primitive }`,
  `Material`, `Texture`, `Animation(i)`, `Skin`, `Node`. Prefer labels over
  hand-built `#Scene0` strings.
- Loading the bare `.glb` as `Handle<Gltf>` gives the index asset:
  `scenes` / `named_scenes`, `meshes` / `named_meshes`, `materials`,
  `nodes`, `skins`, `animations`.
- Scene indices and author names can shift when the source file is edited — use
  them to select authored data, but use explicit marker components for durable
  gameplay links.
- Per the bevy skill's version-pin rule: treat any of the above as a
  hypothesis until confirmed on `docs.rs/bevy/0.19.1`. Bevy breaks APIs
  quarterly; old tutorials lie.

### Post-spawn hookup

Imported scenes arrive as plain entities. Walk them with
`Added<MeshMaterial3d>` + `ChildOf` queries to attach material overrides and
colliders after spawn, and tag roots with marker components.

### Skeletal animation

```rust
let clip: Handle<AnimationClip> = assets.load(
    GltfAssetLabel::Animation(0).from_asset("models/villager.glb"));
```

Build an `AnimationGraph` from the clip, spawn the scene, find the
auto-spawned `AnimationPlayer` on the skeleton root via descendants, and insert
the `AnimationGraphHandle` there. For seamless loops, author the last keyframe
equal to the first.

### Materials

Bevy maps glTF PBR metal/rough to `StandardMaterial`. Principled BSDF exports
map reasonably. `KHR_materials_unlit` imports as unlit;
`KHR_materials_emissive_strength` survives. Anything beyond Principled BSDF
does not survive export — bake it (see §2).

## 2. Blender export settings (the authoring end)

Standardize on the current Blender LTS so files stay openable.

### Before export — clean the source

1. **Apply all transforms** — Object → Apply → All Transforms (Ctrl+A). A
   non-uniform scale on an armature or parented mesh is the single most common
   cause of "looks right in Blender, wrong in engine."
2. **Model at real-world scale in metres.** 1 Blender unit = 1 m = 1 Bevy unit.
   Don't scale at export; scale once, in the source.
3. **Place the origin deliberately** — base of the object for props/furniture
   (sits on the ground plane), pivot point for doors and anything that rotates.
4. **Shade deliberately** — set smooth/flat per surface; don't rely on defaults
   you haven't looked at.
5. **Materials: Principled BSDF only.** Bevy does not evaluate Blender shader
   nodes — bake anything procedural to image textures first. For unlit
   surfaces: Emission shader wired **directly** to Material Output (no BSDF in
   the chain) exports as unlit.
6. **Join by material.** One draw call per material per mesh primitive — joining
   single-material objects collapses dozens of draws into a handful.
7. **Triangulate deliberately** (modifier or exporter) so the engine's
   triangulation matches what you saw in Blender.
8. **UVs on everything textured.** Check for missing UV maps before export.
9. **Armatures:** apply armature scale; bake actions to keyframes (visual
   keying); export deform bones.

### Exporter settings — File → Export → glTF 2.0

- **Format: glTF Binary (.glb)** — one file, textures embedded.
- **Include:** Selected Objects (keeps cameras, lights, stray collections out).
- **Transform:** +Y Up — leave the axis conversion at its default. Blender is
  Z-up; the exporter converts. Don't "fix" orientation manually or you'll fix
  it twice.
- **Apply Modifiers:** on.
- **Geometry:** UVs, Normals on; Tangents only with normal-mapped materials;
  Vertex Colors as needed; Materials → Export.
- **Animation:** Skinning and shape keys as needed; sampling on.
- **Compression: OFF.** No Draco mesh compression — Bevy can't decode it.
- **Save the settings as a preset** and reuse it for every export.

### File layout

```
assets/models/<category>/<name>.glb        # textures embedded in the .glb
assets/models/scale_reference.glb           # 1-unit cube + humanoid silhouette, for eyeballing scale
```

One asset per file for props; characters get their own file including animations.

## 3. Naming conventions

- **Files:** `snake_case` — `villager_bob.glb`, `house_shop.glb`.
- **Blender objects:** `<Asset>_<Part>` — `House_Door`, `Villager_Body`. Names
  come across as `Name` components: debugging handles, not gameplay IDs.
- **Materials:** `<asset>_<surface>` — `house_wood`, `villager_skin`.
- **Textures:** `<asset>_<map>` — `house_wood_albedo.png`,
  `house_wood_normal.png`.
- **Animations:** `<character>_<action>` — `villager_walk`, `villager_wave`.
- **Collision proxies:** `<asset>_collider` — simple invisible mesh authored
  alongside the visual; stripped from rendering on import, kept for physics.

## 4. Scale reference

1 unit = 1 metre. Animal Crossing proportions are stylized-chibi, not realistic:

| Thing | Approx. size (units) |
|---|---|
| Villager | ~1.0 tall (big head, small body) |
| Door opening | ~1.0 wide × 2.0 tall |
| House | ~5 × 4 × 4 |
| Tree | ~2.5–3.5 tall |
| Fence post | ~1.0 tall |

Keep `assets/models/scale_reference.glb` (1-unit cube + 1.7-unit humanoid
silhouette) and stand every new asset next to it on first import.

## 5. Verification checklist

Run for every new or changed `.glb`. In-engine checks can use the brp-loop
skill to inspect the live game.

**Blender-side (pre-export):**
- [ ] All transforms applied — no non-uniform scale anywhere in the hierarchy.
- [ ] Format is `.glb`, Draco compression off.
- [ ] Only intended objects exported (no cameras, lights, stray collections).
- [ ] Procedural materials baked; Principled BSDF (or direct Emission) only.

**In-engine:**
- [ ] File loads: load state is Loaded, not Failed; no glTF warnings in console.
- [ ] Scale: correct height next to the scale reference.
- [ ] Orientation: up is +Y, front faces the expected direction — no 90°/180°
      surprise.
- [ ] Origin: sits where authored (base on ground — no floating or sinking).
- [ ] Materials: colors/textures match Blender's Material Preview;
      metallic/roughness plausible; unlit surfaces actually unlit.
- [ ] No missing textures (pink/black surfaces = missing).
- [ ] Draw calls sane — no accidental dozens from un-joined meshes.
- [ ] Animations: every clip plays, loops cleanly, no bone explosions.
- [ ] Skinned meshes don't vanish at the screen edge (0.19 culls against
      animated bounds — vanishing is a real bug, report with repro).
- [ ] Shadows cast/receive as expected.
- [ ] Round-trip: re-import the `.glb` into a fresh Blender scene — matches
      the source.

## 6. Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| 100× too big/small | Unit mismatch or unapplied scale | Model in metres; Ctrl+A → All Transforms |
| On its side / backwards | Axis conversion overridden | Leave exporter at +Y Up default |
| Right in Blender, warped in Bevy | Non-uniform scale on armature/parent | Apply transforms through the hierarchy |
| Pink or black surfaces | Missing textures | Embed textures in the `.glb` |
| Procedural look vanished | Shader nodes don't export | Bake to image textures |
| Animation explodes | Unbaked constraints / unapplied armature scale | Bake action (visual keying), apply scale |
| Skinned mesh vanishes at frame edge | 0.19 animated-bounds culling | Genuine bug — report with repro |
| Dozens of draw calls, one prop | One primitive per material | Join by material before export |
| Console import warnings | Degenerate geometry | Mesh → Clean Up in Blender, re-export |

## Cross-references

- **bevy skill** (this repo): version-pin rule, post-spawn hookup pattern
  (`Added<MeshMaterial3d>` + `ChildOf`), `Mesh3d`/`MeshMaterial3d<T>` typed handles.
- **brp-loop skill** (rustimal-crossing): live inspection over BRP for the
  verification checklist.
- **asset-pipeline skill** (rustimal-crossing): the C/Rust ROM-asset pipeline —
  unrelated system, same skill name, different repo.
