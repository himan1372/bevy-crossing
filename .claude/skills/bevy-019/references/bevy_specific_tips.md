# Bevy-Specific Development Tips

> Targets **Bevy 0.19** (0.19.0, released 2026-06-19) — the current version.
> See "Bevy 0.18 → 0.19 migration gotchas" below for the deltas if upgrading a
> 0.18 project.

## Bevy API Notes (0.19)

**Important:** These API shapes trip up code carried over from older versions. If you encounter compilation errors related to materials, events/messages, queries, or colors, refer to this section.

### Material Component Wrapper

Material handles are wrapped in `MeshMaterial3d<T>` (meshes in `Mesh3d`):

```rust
// ❌ Bevy 0.16 - long removed
Query<&Handle<StandardMaterial>>

// ✅ Current - use the wrapper component
Query<&MeshMaterial3d<StandardMaterial>>

// Access the inner handle with .0
fn update_materials(
    query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for material_3d in query.iter() {
        // 0.19: get_mut returns an AssetMut guard — bind it `mut`.
        if let Some(mut material) = materials.get_mut(&material_3d.0) {
            material.emissive = LinearRgba::RED;
        }
    }
}
```

**Error symptoms:**
- `Handle<StandardMaterial> is not a Component`
- Query trait bounds not satisfied

**Solution:** Always use `MeshMaterial3d<T>` wrapper when querying material components.

### Events Renamed to Messages

Bevy renamed the **buffered event** API to "messages" (in 0.18; still current
in 0.19). The pattern is otherwise the same — a writer system pushes, one or
more reader systems drain the buffer on a later run:

```rust
// ❌ Old buffered-event API (Bevy 0.16/0.17)
#[derive(Event)]
struct SpellCastEvent { spell_name: String }

app.add_event::<SpellCastEvent>()
   .add_systems(Update, handle_spell_cast);

fn handle_spell_cast(mut events: EventReader<SpellCastEvent>) {
    for event in events.read() {
        info!("Cast: {}", event.spell_name);
    }
}

fn cast_spell(mut events: EventWriter<SpellCastEvent>) {
    events.send(SpellCastEvent { spell_name: "Fireball".into() });
}

// ✅ Current messages API
#[derive(Message)]                         // was #[derive(Event)]
struct SpellCastMessage { spell_name: String }

app.add_message::<SpellCastMessage>()      // was add_event::<T>()
   .add_systems(Update, handle_spell_cast);

fn handle_spell_cast(mut msgs: MessageReader<SpellCastMessage>) {  // was EventReader
    for msg in msgs.read() {
        info!("Cast: {}", msg.spell_name);
    }
}

fn cast_spell(mut msgs: MessageWriter<SpellCastMessage>) {         // was EventWriter
    msgs.write(SpellCastMessage { spell_name: "Fireball".into() }); // was .send()
}
```

**Rename cheat sheet:**
- `#[derive(Event)]` → `#[derive(Message)]`
- `app.add_event::<T>()` → `app.add_message::<T>()`
- `EventWriter<T>` → `MessageWriter<T>`, and `.send(x)` → `.write(x)`
- `EventReader<T>` → `MessageReader<T>` (still `.read()`)

**Error symptoms:**
- `T is not a Message`
- `method 'send' not found for MessageWriter` (use `.write`)
- `method 'add_event' not found` (use `add_message`)

### Observers Are a Separate Reactive Tool (Not a Replacement)

Observers still exist and are **complementary** to messages — do not treat them
as a drop-in replacement. Choose by timing:

- **Messages** (`MessageWriter`/`MessageReader`, buffered) — the default for
  decoupled system-to-system comms; delivered when the reader system next runs.
- **Observers** (`add_observer`, `commands.trigger`, `.observe(...)`) — run an
  **immediate** callback the moment something is triggered; the handler takes a
  trigger/`On<T>` parameter. Reach for these for reactive, per-entity hooks
  (e.g. "when this component is added", "when this entity is clicked").

```rust
// Observer: immediate callback, handler takes the trigger parameter.
app.add_observer(|trigger: On<SpellCastMessage>| {
    info!("Cast: {}", trigger.spell_name);
});

fn cast_spell(mut commands: Commands) {
    commands.trigger(SpellCastMessage { spell_name: "Fireball".into() });
}
```

### Queries: `get_single` Removed

`get_single()`/`get_single_mut()` are gone. Use `.single()`/`.single_mut()`,
which now return `Result`:

```rust
// ❌ Bevy 0.16/0.17
let player = query.get_single().unwrap();

// ✅ Current — idiomatic early-return
let Ok(player) = query.single() else { return };
```

### Color Operations

Direct color arithmetic operations aren't supported:

```rust
// ❌ Doesn't compile
let emissive = color * 0.5;
let darker = color - 0.2;

// ✅ Extract components manually
let emissive = Color::srgb(
    color.to_srgba().red * 0.5,
    color.to_srgba().green * 0.5,
    color.to_srgba().blue * 0.5,
);

// Or use LinearRgba for math operations
let linear = color.to_linear();
let dimmed = LinearRgba::rgb(
    linear.red * 0.5,
    linear.green * 0.5,
    linear.blue * 0.5,
);
```

**Error symptoms:**
- `cannot multiply Color by {float}`
- `no implementation for Color * f32`

**Solution:** Convert to component form or use `LinearRgba` for mathematical operations.

---

## Bevy 0.18 → 0.19 migration gotchas

Verified deltas from a real 0.18.1 → 0.19.0 upgrade of a large project. If a
project is on 0.18, these are what break.

### Ecosystem crate cadence

Every Bevy minor needs matching ecosystem-crate bumps. For 0.19:
`bevy-inspector-egui 0.37`, `bevy_kira_audio 0.26` (0.36 / 0.25 tracked 0.18).
Bump them together with `bevy`, or the graph won't resolve.

### wgpu / `windows`-crate build failure (Windows/D3D12)

Bevy 0.19 uses wgpu 29, whose `wgpu-hal` needs the `windows` crate at **0.62**.
If another dependency drags in `windows` 0.61 (e.g. `cpal` via `kira`/
`bevy_kira_audio`), `gpu-allocator` compiles against the wrong D3D12 types and
`wgpu-hal` fails to build (`ID3D12Heap: Param`, `ResourceCategory: From<&D3D12_RESOURCE_DESC>`).
Fix — unify onto 0.62:

```bash
cargo update -p windows@0.61.3 --precise 0.62.2
```

### `TextFont.font_size` is now the `FontSize` enum

```rust
TextFont { font_size: 35.0, .. }            // ❌ 0.18
TextFont { font_size: FontSize::Px(35.0), .. } // ✅ 0.19  (FontSize is in the prelude)
```

### `Assets::get_mut` returns an `AssetMut` guard

It now returns `Option<AssetMut<'_, A>>` (change-detection) instead of
`Option<&mut A>`, so the binding must be `mut`. Symptom: E0596 "cannot borrow as
mutable". `get_mut_untracked` still returns `&mut A`.

```rust
if let Some(mut mat) = materials.get_mut(&h) { mat.base_color = ...; } // add `mut`
```

Watch for the follow-on E0499 when a `get_mut` guard is the scrutinee of an
`if let … else` that also touches the same `Assets` in the `else` — gate on
`assets.get(&h).is_some()` first so each branch takes a fresh borrow.

### Renames / moves

- `DirectionalLight.shadows_enabled` → `shadow_maps_enabled` (new sibling `contact_shadows_enabled`).
- `Hdr`: `bevy::render::view` → `bevy::camera`.
- `Atmosphere`, `ScatteringMedium`: `bevy::pbr` → `bevy::light::atmosphere`; `::earthlike(..)` → `::earth(..)`.
- `ComputePipelineDescriptor.push_constant_ranges` → `immediate_size: u32`.

### Scene system rename → `bevy_world_serialization`

The old runtime-scene system was renamed; `bevy_scene` is now the unrelated new
BSN scene system. glTF scenes load as `Handle<WorldAsset>` and spawn via
`WorldAssetRoot`:

```rust
// ❌ 0.18
SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("arms.glb")))
// ✅ 0.19  (use bevy::world_serialization::WorldAssetRoot;)
WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("arms.glb")))
```

### Prepass strips the material bind group (custom vertex-displacing materials)

0.19 binds an **empty** material bind group for the depth-only opaque prepass
*and* shadow passes (`bevy_pbr` prepass `is_depth_only_opaque_prepass`). A custom
prepass vertex shader that reads a material binding —
`@group(#{MATERIAL_BIND_GROUP}) @binding(100)` — then fails pipeline validation
at runtime: *"binding is missing from the pipeline layout"*. Fix: don't read
material bindings in a depth-only-opaque prepass shader — inline the values as
consts (fine if they're global) or read from the always-bound `globals` uniform.

---

## Using Bevy Registry Examples

**The registry examples are your bible.** Bevy ships with extensive examples that demonstrate best practices and patterns.

**Location:**
```bash
~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy-0.19.0/examples
```

**When to consult registry examples:**
- Before implementing a new feature type
- When unsure about API usage
- To see working patterns for complex systems
- To understand how plugins should be structured
- For reference implementations of common game mechanics

**How to use them:**
1. Browse the examples directory for relevant use cases
2. Study the complete implementation (not just snippets)
3. Note how they structure components, systems, and plugins
4. Adapt patterns to your specific needs

There are MANY examples covering:
- 2D/3D rendering
- Animation
- Audio
- Input handling
- UI systems
- Physics
- Scenes and assets
- And much more

**Always refer to examples before diving into implementation.**

## Plugin Structure

Break your app into discrete modules using plugins whenever possible.

**Why use plugins:**
- Organizes code by feature/domain
- Makes systems reusable
- Improves code discoverability
- Enables modular development
- Follows Bevy best practices

**Plugin pattern:**
```rust
use bevy::prelude::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<DamageEvent>()
            .add_systems(Startup, setup_combat)
            .add_systems(Update, (
                process_damage,
                check_death,
                update_health_bars,
            ));
    }
}

// In main.rs
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CombatPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(UIPlugin)
        .run();
}
```

**References:**
- Plugin guide: https://bevy.org/learn/quick-start/getting-started/plugins/
- System sets: https://bevy-cheatbook.github.io/programming/system-sets.html

## Build Performance and Optimization

### Dynamic Linking

**Always use dynamic linking during development:**
```bash
cargo build --features bevy/dynamic_linking
```

**Why:**
- 2-3x faster compile times
- Critical for iteration speed
- Only affects development builds

> **0.19.0 caveat:** as of the 0.19.0 release, `bevy_dylib` was not published at
> `0.19.0` (only `0.19.0-rc.x`), so `--features bevy/dynamic_linking` fails to
> resolve (`failed to select a version for the requirement bevy_dylib = "^0.19.0"`).
> Until a matching `bevy_dylib` lands, build without the feature on 0.19.0, or pin
> `bevy_dylib` to the rc explicitly. Check `cargo tree -i bevy_dylib` if unsure.

**Setup in `.cargo/config.toml`:**
```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=/usr/local/opt/llvm/bin/ld64.lld"]
```

**Optimization levels** - See: https://bevy.org/learn/quick-start/getting-started/setup/

For faster dev builds, add to `Cargo.toml`:
```toml
[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3
```

### Build Management

**CRITICAL: Do not delete target binaries freely!**

Bevy takes **minutes** to rebuild from scratch. Be mindful of:

1. **Target directory management:**
   - Avoid `cargo clean` unless absolutely necessary
   - Incremental builds are your friend
   - Each clean rebuild costs valuable development time

2. **Version and dependency management:**
   - Bevy is under active development
   - Be mindful of the version you are using
   - Dependencies can get tangled easily
   - Version mismatches can force complete rebuilds
   - Stick to one Bevy version per project when possible

3. **Crate dependencies:**
   - Adding/removing dependencies triggers rebuilds
   - Changing feature flags triggers rebuilds
   - Plan dependency changes carefully
   - Batch dependency updates when possible

**Best practices:**
- Use `cargo check` for quick validation (no binary)
- Use `cargo build --features bevy/dynamic_linking` for testing
- Only use `cargo clean` when dealing with corrupted build artifacts
- Keep a stable `Cargo.lock` for consistent builds

## Domain-Driven Design for ECS

**Pure ECS structure demands careful data modeling.**

### Think Before You Code

Because it's hard to search a massive list of systems in one file, you must:

1. **Design the data model first:**
   - What entities exist in your domain?
   - What components do they need?
   - What behaviors (systems) operate on them?
   - How do components relate?

2. **Refer to docs and existing code:**
   - Check Bevy examples for similar patterns
   - Review the official docs for component design
   - Look at existing project code for consistency
   - Understand the domain before implementing

3. **Use bounded contexts:**
   - Group related components together
   - Create plugins per domain area
   - Keep systems focused on single responsibilities
   - Avoid cross-domain coupling

### Example Domain Modeling Process

**Bad approach:**
```
❌ Start coding immediately
❌ Add systems to one giant file
❌ Discover missing components mid-implementation
❌ Hard to navigate, hard to maintain
```

**Good approach:**
```
✅ Define the domain (e.g., "Combat System")
✅ List entities (Player, Enemy, Projectile)
✅ List components (Health, Damage, Armor)
✅ List events (DamageEvent, DeathEvent)
✅ List systems (process_damage, check_death, spawn_projectile)
✅ Check examples for similar implementations
✅ Create CombatPlugin
✅ Implement incrementally
✅ Test at each step
```

### File Organization for Discoverability

```
src/
├── main.rs                      # App setup only
├── plugins/
│   ├── mod.rs
│   ├── combat.rs                # CombatPlugin
│   ├── movement.rs              # MovementPlugin
│   └── inventory.rs             # InventoryPlugin
├── components/
│   ├── mod.rs
│   ├── combat.rs                # Health, Armor, Damage
│   ├── movement.rs              # Velocity, Speed
│   └── inventory.rs             # Inventory, Item
└── events.rs                    # All game events
```

**Benefits:**
- Easy to find related code
- Clear domain boundaries
- Plugin-based modularity
- Searchable by feature/domain

## Version Management

**Bevy is under active development.**

1. **Check your Bevy version:**
   ```bash
   cargo tree | grep bevy
   ```

2. **Stay on one version per project:**
   - Avoid mixing Bevy versions
   - Update all Bevy crates together
   - Test thoroughly after version updates

3. **API changes between versions:**
   - Read the migration guide when updating
   - Bevy's API evolves rapidly
   - Code from older versions may not work
   - Examples are version-specific

4. **When seeking help:**
   - Always mention your Bevy version
   - Check if examples match your version
   - Look for version-specific documentation

## Summary Checklist

**Before implementing:**
- [ ] Check registry examples for similar features
- [ ] Design the data model (entities, components, events, systems)
- [ ] Create a plugin for the feature domain
- [ ] Review existing code for patterns

**During development:**
- [ ] Use `cargo build --features bevy/dynamic_linking`
- [ ] Avoid `cargo clean` unless necessary
- [ ] Test incrementally
- [ ] Keep systems focused and organized

**After implementation:**
- [ ] Verify the feature works
- [ ] Check for code organization issues
- [ ] Document domain-specific patterns
- [ ] Update plugin structure if needed
