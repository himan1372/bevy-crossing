//! Phase 2: E-key interaction — talk to villagers, pick up ground items.
//!
//! The spatial query (nearest interactable in range + facing cone) is
//! Bevy-side game code. The logic crate's `interaction` module models
//! villager mood/patience ("feels"), not spatial queries, so it isn't
//! called here; talk-throttle mechanics are a later-phase concern.

use bevy::prelude::*;

use crate::dialogue::DialogueState;
use crate::house::HouseState;
use crate::npc::Villager;
use crate::player::Player;
use crate::town::Town;

/// What a ground item is.
#[derive(Clone, Copy, Debug)]
pub enum ItemKind {
    Bells(u32),
    Fruit,
}

/// Marker for pickable ground items.
#[derive(Component)]
pub struct GroundItem {
    pub kind: ItemKind,
    /// Index into `ITEM_SPOTS` — lets the save system track pickups.
    pub index: usize,
}

/// Fixed ground-item spawn spots: (kind, x, z).
/// The save system records picked-up items as indices into this array.
pub const ITEM_SPOTS: [(ItemKind, f32, f32); 6] = [
    (ItemKind::Bells(100), 3.0, 2.0),
    (ItemKind::Bells(200), -4.0, 3.0),
    (ItemKind::Bells(150), 2.0, -4.0),
    (ItemKind::Fruit, -3.0, -2.0),
    (ItemKind::Fruit, 5.0, -3.0),
    (ItemKind::Fruit, -5.0, -4.0),
];

/// Player's collected items (placeholder inventory for Phase 2).
#[derive(Resource, Default)]
pub struct PlayerInventory {
    pub bells: u32,
    pub fruit: u32,
    /// Fish caught with the rod (Phase 3 ecology).
    pub fish: u32,
    /// Bugs caught with the net (Phase 3 ecology).
    pub bugs: u32,
    /// Furniture bought at Nook's (Phase 3 shop).
    pub furniture: Vec<String>,
}

/// Fired when the player talks to a villager.
#[derive(Event)]
pub struct TalkEvent {
    pub villager: Entity,
}

/// Player's facing direction, tracked from movement.
#[derive(Component)]
pub struct Facing {
    pub dir: Vec3,
    pub(crate) last_pos: Vec3,
    initialized: bool,
}

const INTERACT_RANGE_VILLAGER: f32 = 4.0;
const INTERACT_RANGE_ITEM: f32 = 3.0;
/// Cosine threshold for the facing cone (~78 degrees).
const FACING_DOT: f32 = 0.2;

/// Spawn a single ground item. Shared by the startup spawner and the
/// save system (which respawns un-picked items on quick-load).
pub fn spawn_ground_item(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    index: usize,
    kind: ItemKind,
    pos: Vec3,
) {
    let (mesh, mat) = match kind {
        ItemKind::Bells(_) => (
            meshes.add(Sphere::new(0.25)),
            materials.add(Color::srgb(1.0, 0.85, 0.2)),
        ),
        ItemKind::Fruit => (
            meshes.add(Sphere::new(0.28)),
            materials.add(Color::srgb(1.0, 0.45, 0.15)),
        ),
    };
    commands.spawn((
        GroundItem { kind, index },
        PbrBundle {
            mesh,
            material: mat,
            transform: Transform::from_translation(pos),
            ..default()
        },
    ));
}

/// Spawn a few test items on walkable ground near the town center.
/// Skips items the loaded save says were already picked up.
fn spawn_items(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    town: Res<Town>,
    loaded: Option<Res<crate::save::LoadedSave>>,
) {
    let picked: &[usize] = loaded
        .as_ref()
        .and_then(|l| l.0.as_ref())
        .map(|s| s.picked_items.as_slice())
        .unwrap_or(&[]);

    for (index, (kind, x, z)) in ITEM_SPOTS.iter().enumerate() {
        if picked.contains(&index) {
            continue;
        }
        let pos = Vec3::new(*x, 0.3, *z);
        if !town.is_walkable(Vec3::new(*x, 0.0, *z)) {
            continue;
        }
        spawn_ground_item(&mut commands, &mut meshes, &mut materials, index, *kind, pos);
    }
    println!("Ground items spawned");
}

/// Track which way the player is facing from their movement.
pub fn track_player_facing(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform), With<Player>>,
    mut facing_query: Query<&mut Facing>,
) {
    let Ok((entity, transform)) = player_query.get_single() else {
        return;
    };
    let pos = transform.translation;

    if let Ok(mut facing) = facing_query.get_single_mut() {
        if !facing.initialized {
            facing.last_pos = pos;
            facing.initialized = true;
            return;
        }
        let delta = pos - facing.last_pos;
        if delta.length() > 0.01 {
            facing.dir = Vec3::new(delta.x, 0.0, delta.z).normalize();
            facing.last_pos = pos;
        }
    } else {
        commands.entity(entity).insert(Facing {
            dir: Vec3::new(0.0, 0.0, -1.0),
            last_pos: pos,
            initialized: false,
        });
    }
}

/// E key: talk to the nearest faced villager, else pick up the nearest item.
///
/// When a fishing rod or bug net is equipped (see `ecology::EquippedTool`),
/// this system stands down and lets the ecology plugin handle the E press.
pub fn interact(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<HouseState>>,
    shop_loc: Res<State<crate::shop::ShopLocation>>,
    dialogue: Res<DialogueState>,
    tool: Res<crate::ecology::EquippedTool>,
    mail_ui: Res<crate::mail::MailUi>,
    shop_ui: Res<crate::shop::ShopUi>,
    quest_ui: Res<crate::quest::QuestUi>,
    mut talk_events: EventWriter<TalkEvent>,
    mut commands: Commands,
    player_query: Query<(&Transform, &Facing), With<Player>>,
    villagers: Query<(Entity, &Villager, &Transform), Without<Player>>,
    items: Query<(Entity, &GroundItem, &Transform), Without<Player>>,
    mut inventory: ResMut<PlayerInventory>,
) {
    // Inside the house or shop, E belongs to the doors.
    if *state != HouseState::Town || *shop_loc != crate::shop::ShopLocation::Town {
        return;
    }
    if !keyboard.just_pressed(KeyCode::KeyE) || dialogue.active {
        return;
    }
    // The mail, shop, or quest UI claims the E key while open.
    if mail_ui.open || shop_ui.open || quest_ui.open {
        return;
    }
    // A held tool claims the E press — the ecology plugin handles it.
    if *tool != crate::ecology::EquippedTool::None {
        return;
    }
    let Ok((player_transform, facing)) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation;

    // 1. Nearest villager in range inside the facing cone.
    let mut best: Option<(Entity, f32)> = None;
    for (entity, _villager, v_transform) in villagers.iter() {
        let to_v = v_transform.translation - player_pos;
        let dist = to_v.length();
        if dist > INTERACT_RANGE_VILLAGER || dist < 0.001 {
            continue;
        }
        if facing.dir.dot(to_v / dist) < FACING_DOT {
            continue;
        }
        if best.map_or(true, |(_, d)| dist < d) {
            best = Some((entity, dist));
        }
    }
    if let Some((entity, _)) = best {
        talk_events.send(TalkEvent { villager: entity });
        return;
    }

    // 2. Nearest ground item in range (no facing requirement).
    let mut best_item: Option<(Entity, ItemKind, f32)> = None;
    for (entity, item, i_transform) in items.iter() {
        let dist = (i_transform.translation - player_pos).length();
        if dist > INTERACT_RANGE_ITEM {
            continue;
        }
        if best_item.map_or(true, |(_, _, d)| dist < d) {
            best_item = Some((entity, item.kind, dist));
        }
    }
    if let Some((entity, kind, _)) = best_item {
        match kind {
            ItemKind::Bells(amount) => {
                inventory.bells += amount;
                println!("Picked up {amount} bells (total: {})", inventory.bells);
            }
            ItemKind::Fruit => {
                inventory.fruit += 1;
                println!("Picked up a fruit (total: {})", inventory.fruit);
            }
        }
        commands.entity(entity).despawn();
    }
}

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerInventory::default())
            .add_event::<TalkEvent>()
            .add_systems(Startup, spawn_items.after(crate::town::generate_town))
            .add_systems(
                Update,
                (track_player_facing, interact)
                    .chain()
                    .after(crate::house::HouseInputSet)
                    .after(crate::mail::MailInputSet)
                    .after(crate::shop::ShopInputSet),
            );
    }
}
