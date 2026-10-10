# Common Bevy Pitfalls Reference

> Targets **Bevy 0.19**. For a 0.18 → 0.19 upgrade, see the "Bevy 0.18 → 0.19
> migration gotchas" section in `references/bevy_specific_tips.md` (FontSize,
> `Assets::get_mut` guard, `shadow_maps_enabled`, `Hdr`/`Atmosphere` moves,
> `WorldAssetRoot`, the wgpu/`windows` build pin, and the prepass bind-group
> strip).

## 1. Using the Old Event API

**❌ Problem:**
```rust
// The Bevy 0.16/0.17 buffered-event API no longer exists (renamed in 0.18)
#[derive(Event)]
struct MyEvent { data: String }

app.add_event::<MyEvent>()
   .add_systems(Update, handle_event);

fn handle_event(mut events: EventReader<MyEvent>) { /* ... */ }
fn trigger(mut events: EventWriter<MyEvent>) { /* ... */ }
```

**Symptoms:**
- Compilation error: `MyEvent is not a Message`
- `method 'add_event' not found` / `method 'send' not found for MessageWriter`

**✅ Solution:**
Buffered events were renamed to **messages** in 0.18 (still current in 0.19) — same pattern, new names:
```rust
// Messages API
#[derive(Message)]                       // was #[derive(Event)]
struct MyMessage { data: String }

app.add_message::<MyMessage>()           // was add_event::<T>()
   .add_systems(Update, handle_message);

fn handle_message(mut msgs: MessageReader<MyMessage>) {  // was EventReader
    for msg in msgs.read() { /* ... */ }
}
fn trigger(mut msgs: MessageWriter<MyMessage>) {         // was EventWriter
    msgs.write(MyMessage { data: "test".into() });       // was .send()
}
```

(Observers — `add_observer`/`commands.trigger` — are a separate reactive tool
for immediate callbacks, not a replacement for buffered messages.)

See `references/bevy_specific_tips.md` for complete migration guide.

## 2. Querying Material Handles

**❌ Problem:**
```rust
// Bevy 0.16 pattern — long removed
Query<&Handle<StandardMaterial>>
```

**Symptoms:**
- `Handle<StandardMaterial> is not a Component`
- Query trait bounds not satisfied

**✅ Solution:**
Use the `MeshMaterial3d` wrapper:
```rust
Query<&MeshMaterial3d<StandardMaterial>>

// Access handle with .0
for material_3d in query.iter() {
    // 0.19: get_mut returns an AssetMut guard — bind it `mut`.
    if let Some(mut material) = materials.get_mut(&material_3d.0) {
        material.emissive = color;
    }
}
```

## 3. Forgetting to Register Systems

**❌ Problem:**
```rust
// Created system but forgot to add to app
pub fn my_new_system() { /* ... */ }
```

**✅ Solution:**
Always add to `main.rs`:
```rust
.add_systems(Update, my_new_system)
```

## 4. Borrowing Conflicts

**❌ Problem:**
```rust
// Can't have multiple mutable borrows
mut query1: Query<&mut Transform>,
mut query2: Query<&mut Transform>,  // Error!
```

**✅ Solution:**
```rust
// Use get_many_mut for specific entities
mut query: Query<&mut Transform>,

if let Ok([mut a, mut b]) = query.get_many_mut([entity_a, entity_b]) {
    // Can mutate both
}
```

For two queries over the **same component** that address *different* entity sets
(e.g. players vs enemies both `&mut Transform`), disjoin them with `With`/
`Without`; if they can't be proven disjoint, wrap them in a `ParamSet` so only one
is accessed at a time:
```rust
fn system(mut set: ParamSet<(
    Query<&mut Transform, With<Player>>,
    Query<&mut Transform, With<Enemy>>,
)>) {
    for mut t in set.p0().iter_mut() { /* players */ }
    for mut t in set.p1().iter_mut() { /* enemies */ }
}
```

## 5. Infinite Loops with Messages

**❌ Problem:**
```rust
// System reads and writes same message type
fn system(
    mut writer: MessageWriter<MyMessage>,
    mut reader: MessageReader<MyMessage>,
) {
    for msg in reader.read() {
        writer.write(MyMessage);  // Infinite loop!
    }
}
```

**✅ Solution:**
Use different message types or add a termination condition.

## 6. Not Using Changed<T>

**❌ Problem:**
```rust
// Runs every frame for every entity
fn system(query: Query<&BigFive>) {
    for traits in query.iter() {
        // Expensive calculation every frame
    }
}
```

**✅ Solution:**
```rust
// Only runs when BigFive changes
fn system(query: Query<&BigFive, Changed<BigFive>>) {
    for traits in query.iter() {
        // Only when needed
    }
}
```

## 7. Entity Queries After Despawn

**❌ Problem:**
```rust
commands.entity(entity).despawn();
// Later in same system
let component = query.get(entity).unwrap();  // Crash!
```

**✅ Solution:**
`Commands` are deferred — they apply at the next sync point, not immediately.
The same timing bites the **spawn** direction: an entity spawned via `Commands`
is not visible to a `Query` **in the same system that spawned it** (nor reliably
earlier in the same frame's schedule). Read it in a later system. For existing
entities that may have been despawned, use the `Ok()` pattern:
```rust
if let Ok(component) = query.get(entity) {
    // Safe
}
```

## 8. Material/Asset Handle Confusion

**❌ Problem:**
```rust
// Created material but didn't store handle
materials.add(StandardMaterial { .. });  // Handle dropped!
```

**✅ Solution:**
```rust
let material_handle = materials.add(StandardMaterial { .. });
commands.spawn((
    MeshMaterial3d(material_handle),
    // ...
));
```

## 9. System Ordering Issues

**❌ Problem:**
```rust
// UI updates before state changes
.add_systems(Update, (
    update_ui,
    process_input,  // Wrong order!
))
```

**✅ Solution:**
Order systems by dependencies:
```rust
.add_systems(Update, (
    // Input processing
    process_input,

    // State changes
    update_state,

    // UI updates (reads state)
    update_ui,
))
```

## 10. Not Filtering Queries Early

**❌ Problem:**
```rust
// Filter in loop (inefficient)
Query<(&A, Option<&B>, Option<&C>)>
// Then check in loop
```

**✅ Solution:**
```rust
// Filter in query (efficient)
Query<&A, (With<B>, Without<C>)>
```

## 11. Frame-Rate-Dependent Movement

**❌ Problem:**
```rust
// Speed is tied to frame rate — faster PCs move faster.
fn move_player(mut q: Query<&mut Transform, With<Player>>) {
    for mut t in &mut q { t.translation.x += 5.0; } // per FRAME, not per second
}
```

**Symptoms:** movement/animation runs faster on high-refresh monitors and slower
under load; physics feels inconsistent.

**✅ Solution:**
Scale per-frame changes by delta time so motion is measured per second:
```rust
fn move_player(time: Res<Time>, mut q: Query<&mut Transform, With<Player>>) {
    for mut t in &mut q {
        t.translation.x += 300.0 * time.delta_secs(); // 300 units/second
    }
}
```
`delta_secs()`/`elapsed_secs()` are the current names (they were `delta_seconds()`/
`elapsed_seconds()` before 0.16). For physics that must be perfectly stable, run it
in the `FixedUpdate` schedule and read `Time<Fixed>` instead.

## 12. Deprecated Bundles (`*Bundle` Types)

**❌ Problem:**
```rust
// Bundle types were deprecated in 0.15 and removed in 0.16 — long gone by 0.19.
commands.spawn(Camera2dBundle::default());          // not found
commands.spawn(SpriteBundle { texture, .. });       // not found
commands.spawn(PbrBundle { mesh, material, .. });    // not found
```

**Symptoms:** `cannot find type 'Camera2dBundle'` / `SpriteBundle' in this scope`.

**✅ Solution:**
Spawn the components directly as a tuple; **required components** pull in the rest
(`Camera2d` brings `Camera`/`Transform`, `Mesh3d` brings what it needs, etc.):
```rust
commands.spawn(Camera2d);
commands.spawn((Sprite::from_image(texture), Transform::from_xyz(x, y, 0.0)));
commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::default()));
```
