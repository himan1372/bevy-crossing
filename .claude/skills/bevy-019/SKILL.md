---
name: bevy
description: Bevy 0.19 game development — ECS patterns, system ordering, UI, assets, and build workflow for this Animal Crossing PC port. Use when writing or changing any Bevy code, designing components/systems, debugging borrow or scheduling issues, working with Bevy UI, or importing glTF assets. Do not rely on pre-0.19 Bevy knowledge without verifying.
---

# Bevy 0.19 — Port Skill

This project is an Animal Crossing (GameCube) PC port built on **Bevy 0.19**.
Bevy ships breaking changes roughly quarterly. The number one failure mode
when writing Bevy code with AI help is confidently using a pre-0.19 API that
no longer exists.

## Version pin — read this first

- This project pins **`bevy = "0.19.1"`** in `Cargo.toml`. That pin is the
  source of truth.
- **Treat any Bevy knowledge older than the pinned version as a hypothesis.**
  Before asserting a type, method, or plugin path exists, confirm it on
  `docs.rs/bevy/0.19.1`.
- Bevy publishes a migration guide for every release:
  `bevy.org/learn/migration-guides/`. When porting code or following a
  tutorial, check which Bevy version it targets first.

## What changed in 0.19 (invalidates older tutorials)

- **Resources are components.** A resource is stored as a component on a
  singleton entity. Lifecycle hooks and observers work on resource types now,
  and resources can participate in relationships.
- **Buffered events are "messages".** `EventWriter` → `MessageWriter`
  (`.write()`), `EventReader` → `MessageReader` (`.read()`),
  `add_event::<T>()` → `add_message::<T>()`, `#[derive(Event)]` →
  `#[derive(Message)]`. Observers (`add_observer`, `commands.trigger`) are a
  *separate* reactive tool, not a replacement for messages.
- **Material/mesh handles are typed.** Meshes use `Mesh3d`, materials use
  `MeshMaterial3d<T>` — not bare `Handle<T>`.
- **Render graph is ECS schedules now.** Render passes are ordinary systems in
  the `Core3d` / `Core2d` schedules. Old `RenderGraph` / `Node` trait
  boilerplate from tutorials no longer applies.
- **`FontSize` is an enum**, not `f32`: `FontSize::Px(35.0)`. `TextFont`
  gained `weight`, `width`, `style`. Porting old UI code without this change
  produces silent type errors.
- **Queries:** `.single()` / `.single_mut()` return `Result`; the old
  `get_single*` methods are gone.
- **BSN scenes.** The `bsn!` macro is the scene syntax (composable patches,
  `#[derive(SceneComponent)]`, `.bsn` assets).
- `Assets::get_mut` returns an `AssetMut` guard — the binding must be `mut`.

## Core ECS patterns

Think in **data** (components) and **transformations** (systems), never
objects and methods.

- **Components** = pure data, no logic. Keep them small and focused.
- **Systems** = pure logic over components.
- **Messages** = buffered communication between systems.
- **Resources** = global state. Use sparingly; prefer components.

```rust
// ✅ Small, focused components
#[derive(Component)]
pub struct Health { pub current: f32, pub max: f32 }

// ❌ Monolithic component — wastes memory, couples systems
#[derive(Component)]
pub struct Stats { pub health: f32, pub armor: f32, pub strength: f32 }
```

Put game logic behind `impl` blocks on components, and organize features as
**plugins** — one plugin per domain (player, villagers, ui, audio):

```rust
pub struct VillagerPlugin;

impl Plugin for VillagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TalkEvent>()
           .add_systems(Update, (villager_wander, villager_talk));
    }
}
```

### Borrow rules that bite

- Two systems (or two queries in one system) cannot mutably borrow the same
  component type. Use `get_many_mut([e1, e2])` when you need mutable access
  to several entities of the same query.
- Prefer `Changed<T>` / `Added<T>` filters over every-frame polling.
- Never mutate the `World` directly from inside an observer; use `commands`.

### System ordering

Default order within a schedule is unspecified. When order matters, be
explicit:

```rust
app.add_systems(Update, (move_player, camera_follow.after(move_player)));
```

Use schedule labels (`Startup`, `Update`, `FixedUpdate`) deliberately.
Gameplay logic that must be deterministic goes in `FixedUpdate`.

## UI

- Build UI with Bevy's ECS UI (`Node`, `Text`, `Button` components), not
  immediate-mode calls.
- `TextFont.font_size` takes the `FontSize` enum (`FontSize::Px(..)`).
- Keep UI state in components/resources and let systems react to it; do not
  hand-position nodes from gameplay code.

## Assets (glTF pipeline for the port)

- GameCube models come in through the Blender → `.glb` pipeline (see the
  asset-pipeline skill when it lands). In Bevy they arrive as scenes.
- Walk imported scenes with `Added<MeshMaterial3d>` + `ChildOf` queries to
  hook up materials and colliders after spawn.
- Mark asset handles with dedicated marker components rather than
  string-matching names.

## Build & run (Windows)

```bash
cargo check          # fast correctness pass
cargo run            # builds and runs the game
```

- First builds are slow (Bevy is a large dependency graph). Later builds are
  incremental and much faster.
- For faster iteration, enable Bevy's `dynamic_linking` cargo feature in a
  dev profile — it trades a slower first link for much quicker rebuilds.
- If the game window doesn't appear or BRP can't connect, check the firewall
  prompt for the new binary.

## Verifying with BRP

This project runs `bevy_remote` (`RemotePlugin` + `RemoteHttpPlugin`) so
agents can observe the live game over BRP at `localhost:15702`:

1. `cargo run` the game.
2. Query entities/components over BRP to confirm the scene matches intent.
3. Screenshot via the BRP tooling when available; never claim a visual result
   you haven't observed.

## When stuck

1. Check the Bevy 0.19 examples in the local cargo registry
   (`~/.cargo/registry/src/*/bevy-0.19.1/examples`) — they are the most
   reliable reference after docs.rs.
2. Re-read the version pin at the top of this file. If the API you're
   fighting came from a tutorial, check the tutorial's Bevy version.
