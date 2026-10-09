//! Phase 3: Tom Nook's shop.
//!
//! A shop building sits on the town map (placed on a free 5x4 grass rect
//! near the town center). Pressing E at the door teleports the player into
//! a shop interior built at a distant world offset (same pattern as the
//! house plugin). Inside, E at Tom Nook opens the buy/sell menu.
//!
//! Logic-crate usage: `rustimal_logic::shop::{ShopState, ShopTier}` drives
//! sales-sum tracking and the tier/upgrade display (`mSP_PlusSales` clamp,
//! `mSP_GetRealShopLevel` thresholds, `mSP_RenewShopLevel`). Stock,
//! pricing, the UI, and the building itself are Bevy-side: the ported
//! modules carry no price table, so prices here are sensible originals.
//! Renovation is simplified — the tier upgrades immediately when the sales
//! threshold is crossed instead of after the retail 2-day remodel.

use bevy::prelude::*;
use rustimal_logic::shop::{self, ShopTier};
use serde::{Deserialize, Serialize};

use crate::dialogue::DialogueState;
use crate::ecology::EquippedTool;
use crate::interaction::{Facing, PlayerInventory};
use crate::player::Player;
use crate::town::Town;

/// Town map vs. shop interior.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ShopLocation {
    #[default]
    Town,
    Inside,
}

/// System set for the shop-door E handler. Runs before the town `interact`
/// system (same slot pattern as `HouseInputSet`/`MailInputSet`) so a
/// successful enter (which teleports the player far away) can't double-fire
/// talk/pickup that frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShopInputSet;

/// Exterior shop building: door position plus the blocked footprint the
/// player and villagers can't walk through.
#[derive(Resource, Default)]
pub struct ShopBuilding {
    pub door_pos: Vec3,
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,
    pub placed: bool,
}

/// True when (x, z) is inside the shop building's blocked footprint.
pub fn point_blocked(building: &ShopBuilding, x: f32, z: f32) -> bool {
    building.placed && x > building.min_x && x < building.max_x && z > building.min_z && z < building.max_z
}

/// One item on the shelf. Serializable so the save file can persist stock.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StockItem {
    pub name: String,
    pub price: u32,
}

/// Shop runtime data: today's stock plus the decomp-verified sales ledger.
#[derive(Resource)]
pub struct ShopData {
    pub stock: Vec<StockItem>,
    pub shop: shop::ShopState,
}

impl Default for ShopData {
    fn default() -> Self {
        Self {
            stock: default_stock(),
            shop: shop::ShopState::default(),
        }
    }
}

/// Starter stock — sensible original prices (no price table in the ported
/// decomp modules).
fn default_stock() -> Vec<StockItem> {
    vec![
        StockItem { name: "Wooden Chair".into(), price: 800 },
        StockItem { name: "Potted Plant".into(), price: 900 },
        StockItem { name: "Wall Clock".into(), price: 1100 },
        StockItem { name: "Modern Lamp".into(), price: 1400 },
        StockItem { name: "Bookshelf".into(), price: 1800 },
        StockItem { name: "Cozy Bed".into(), price: 2000 },
    ]
}

/// Display name for a shop tier.
fn tier_name(tier: ShopTier) -> &'static str {
    match tier {
        ShopTier::Zakka => "Nook's Cranny",
        ShopTier::Combini => "Nook 'n Go",
        ShopTier::Super => "Nookway",
        ShopTier::Dsuper => "Nookington's",
    }
}

/// Town-map position to return to when leaving the shop.
#[derive(Resource, Clone, Copy)]
pub struct ShopReturnPos(pub Vec3);

/// Shop menu state. Other plugins read `open` to yield the E key.
#[derive(Resource, Default)]
pub struct ShopUi {
    pub open: bool,
    mode: ShopMode,
    selected: usize,
    message: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum ShopMode {
    #[default]
    Buy,
    Sell,
}

/// UI markers.
#[derive(Component)]
struct ShopPanel;
#[derive(Component)]
struct ShopTitleText;
#[derive(Component)]
struct ShopListText;
#[derive(Component)]
struct ShopMsgText;
#[derive(Component)]
struct ShopHintText;
#[derive(Component)]
struct ShopHudText;

/// Far-away origin for the shop interior (house interior is at +z).
const SHOP_ORIGIN: Vec3 = Vec3::new(3000.0, 0.0, -3000.0);
/// Exterior building footprint in world units.
const SHOP_W: f32 = 5.0;
const SHOP_D: f32 = 4.0;
const SHOP_H: f32 = 3.0;
const DOOR_RANGE_OUT: f32 = 3.4;
const DOOR_RANGE_IN: f32 = 2.6;
const DOOR_DOT: f32 = 0.6;
const NOOK_RANGE: f32 = 3.5;
const NOOK_DOT: f32 = 0.5;
/// Interior walk bounds (half-extents from the origin).
const SHOP_WALK_X: f32 = 5.2;
const SHOP_WALK_Z: f32 = 4.2;
const PLAYER_RADIUS: f32 = 0.4;
const PLAYER_SPEED: f32 = 5.0;
/// What Nook pays per unit.
const FRUIT_SELL: u32 = 100;
const FISH_SELL: u32 = 250;
const BUG_SELL: u32 = 150;

/// Interior obstacles (counter + display pedestals), in world coordinates.
#[derive(Resource, Default)]
struct ShopObstacles(Vec<Obstacle>);

struct Obstacle {
    min_x: f32,
    max_x: f32,
    min_z: f32,
    max_z: f32,
}

/// Find a free 5x4 grass rect near the town center for the shop building,
/// scanning outward from the middle of the map.
fn find_shop_spot(town: Res<Town>, mut commands: Commands) {
    // Global cell dims: 5 acres * 16 by 6 acres * 16.
    const GW: i32 = 5 * 16;
    const GD: i32 = 6 * 16;
    const RW: i32 = 5; // rect width in cells
    const RD: i32 = 4; // rect depth in cells
    let cx = GW / 2;
    let cz = GD / 2;

    let cell_center = |gx: i32, gz: i32| -> Vec3 {
        let ax = (gx / 16) as usize;
        let az = (gz / 16) as usize;
        let lx = (gx % 16) as usize;
        let lz = (gz % 16) as usize;
        Town::cell_to_world(ax, az, lx, lz)
    };

    let mut best: Option<(i32, i32, i32)> = None; // (gx, gz, dist2)
    for gz in 0..=(GD - RD) {
        for gx in 0..=(GW - RW) {
            // Skip rects covering the player spawn at world (0, 0).
            let mut ok = true;
            for dz in 0..RD {
                for dx in 0..RW {
                    let p = cell_center(gx + dx, gz + dz);
                    if !town.is_walkable(p) {
                        ok = false;
                        break;
                    }
                    if p.x.abs() < 1.0 && p.z.abs() < 1.0 {
                        ok = false; // player spawn
                        break;
                    }
                }
                if !ok {
                    break;
                }
            }
            if !ok {
                continue;
            }
            // The door approach (one cell south of the rect) must be walkable.
            let door_cell = cell_center(gx + RW / 2, gz + RD);
            if gz + RD >= GD || !town.is_walkable(door_cell) {
                continue;
            }
            let d2 = (gx + RW / 2 - cx).pow(2) + (gz + RD / 2 - cz).pow(2);
            if best.map_or(true, |(_, _, b)| d2 < b) {
                best = Some((gx, gz, d2));
            }
        }
    }

    let Some((gx, gz, _)) = best else {
        println!("No free spot for the shop!");
        return;
    };
    let x0 = cell_center(gx, gz).x - 0.5;
    let z0 = cell_center(gx, gz).z - 0.5;
    let center = Vec3::new(x0 + SHOP_W / 2.0, 0.0, z0 + SHOP_D / 2.0);
    let door_pos = Vec3::new(center.x, 0.0, z0 + SHOP_D + 0.7);
    commands.insert_resource(ShopBuilding {
        door_pos,
        min_x: center.x - SHOP_W / 2.0,
        max_x: center.x + SHOP_W / 2.0,
        min_z: center.z - SHOP_D / 2.0,
        max_z: center.z + SHOP_D / 2.0,
        placed: true,
    });
    println!("Shop placed at {:?}, door at {:?}", center, door_pos);
}

/// Spawn the exterior shop building mesh at the chosen spot.
fn spawn_shop_exterior(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    building: Res<ShopBuilding>,
) {
    if !building.placed {
        return;
    }
    let cx = (building.min_x + building.max_x) / 2.0;
    let cz = (building.min_z + building.max_z) / 2.0;

    // Main box.
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(SHOP_W, SHOP_H, SHOP_D)),
        material: materials.add(Color::srgb(0.75, 0.6, 0.4)),
        transform: Transform::from_xyz(cx, SHOP_H / 2.0, cz),
        ..default()
    });
    // Roof slab.
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(SHOP_W + 0.6, 0.5, SHOP_D + 0.6)),
        material: materials.add(Color::srgb(0.6, 0.25, 0.2)),
        transform: Transform::from_xyz(cx, SHOP_H + 0.25, cz),
        ..default()
    });
    // Door panel on the south face.
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.2, 2.2, 0.15)),
        material: materials.add(Color::srgb(0.35, 0.22, 0.12)),
        transform: Transform::from_xyz(cx, 1.1, building.max_z + 0.05),
        ..default()
    });
    // Sign board above the door.
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(2.6, 0.8, 0.2)),
        material: materials.add(Color::srgb(0.95, 0.9, 0.75)),
        transform: Transform::from_xyz(cx, 2.7, building.max_z + 0.08),
        ..default()
    });
    println!("Shop exterior spawned");
}

/// Build the shop interior once at the distant origin.
fn build_shop_interior(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ox = SHOP_ORIGIN.x;
    let oz = SHOP_ORIGIN.z;

    let floor_mat = materials.add(Color::srgb(0.6, 0.44, 0.26));
    let wall_mat = materials.add(Color::srgb(0.88, 0.8, 0.66));
    let counter_mat = materials.add(Color::srgb(0.45, 0.3, 0.16));
    let door_mat = materials.add(Color::srgb(0.4, 0.25, 0.12));
    let nook_mat = materials.add(Color::srgb(0.55, 0.38, 0.22));

    // Floor.
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(12.0, 10.0)),
        material: floor_mat,
        transform: Transform::from_xyz(ox, 0.02, oz),
        ..default()
    });

    // Four walls (no ceiling; camera looks in from above).
    let wall_h = 3.0;
    let wall_t = 0.4;
    let walls = [
        (ox, oz - 5.0, 12.0, wall_t), // north
        (ox, oz + 5.0, 12.0, wall_t), // south
        (ox - 6.0, oz, wall_t, 10.0), // west
        (ox + 6.0, oz, wall_t, 10.0), // east
    ];
    for (wx, wz, sx, sz) in walls {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(sx, wall_h, sz)),
            material: wall_mat.clone(),
            transform: Transform::from_xyz(wx, wall_h / 2.0, wz),
            ..default()
        });
    }

    // Door panel on the south wall (visual only; exit is via E).
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.4, 2.4, 0.15)),
        material: door_mat,
        transform: Transform::from_xyz(ox, 1.2, oz + 4.85),
        ..default()
    });

    // Counter at the back (north side).
    let counter = Obstacle {
        min_x: ox - 2.25,
        max_x: ox + 2.25,
        min_z: oz - 3.5,
        max_z: oz - 2.5,
    };
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(4.5, 1.1, 1.0)),
        material: counter_mat,
        transform: Transform::from_xyz(ox, 0.55, oz - 3.0),
        ..default()
    });

    // Tom Nook behind the counter.
    commands.spawn(PbrBundle {
        mesh: meshes.add(Capsule3d::new(0.4, 1.0)),
        material: nook_mat,
        transform: Transform::from_xyz(ox, 1.0, oz - 4.1),
        ..default()
    });

    // Display pedestals with sample goods (colors only; names/prices live
    // in the shop menu).
    let mut obstacles = vec![counter];
    let pedestal_cols = [
        (0.75, 0.2, 0.2),
        (0.25, 0.4, 0.8),
        (0.9, 0.8, 0.4),
        (0.3, 0.7, 0.3),
    ];
    for (i, (r, g, b)) in pedestal_cols.iter().enumerate() {
        let lx = -3.75 + i as f32 * 2.5;
        let lz = 0.8;
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(0.9, 0.8, 0.9)),
            material: materials.add(Color::srgb(0.5, 0.35, 0.2)),
            transform: Transform::from_xyz(ox + lx, 0.4, oz + lz),
            ..default()
        });
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(0.55, 0.55, 0.55)),
            material: materials.add(Color::srgb(*r, *g, *b)),
            transform: Transform::from_xyz(ox + lx, 1.1, oz + lz),
            ..default()
        });
        obstacles.push(Obstacle {
            min_x: ox + lx - 0.45,
            max_x: ox + lx + 0.45,
            min_z: oz + lz - 0.45,
            max_z: oz + lz + 0.45,
        });
    }
    commands.insert_resource(ShopObstacles(obstacles));

    // Persistent "inside shop" indicator (hidden until the player enters).
    commands.spawn((
        ShopHudText,
        TextBundle {
            text: Text::from_section(
                "Nook's — E at Nook to shop, E at the door to exit",
                TextStyle {
                    font_size: 22.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(50.0),
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        },
    ));

    println!("Shop interior built at {:?}", SHOP_ORIGIN);
}

/// Build the shop menu UI (hidden until opened).
fn setup_shop_ui(mut commands: Commands) {
    commands
        .spawn((
            ShopPanel,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(24.0),
                    right: Val::Percent(24.0),
                    top: Val::Percent(12.0),
                    bottom: Val::Percent(12.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.10, 0.08, 0.05, 0.96)),
                visibility: Visibility::Hidden,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                ShopTitleText,
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 28.0,
                        color: Color::srgb(1.0, 0.9, 0.5),
                        ..default()
                    },
                ),
            ));
            parent.spawn((
                ShopListText,
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
            ));
            parent.spawn((
                ShopMsgText,
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 20.0,
                        color: Color::srgb(1.0, 0.85, 0.4),
                        ..default()
                    },
                ),
            ));
            parent.spawn((
                ShopHintText,
                TextBundle::from_section(
                    "1: Buy   2: Sell   Up/Down: select   Enter: confirm   Esc: close",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.7, 0.7, 0.75),
                        ..default()
                    },
                ),
            ));
        });
    println!("Shop UI ready");
}

/// Restore shop stock/sales/tier and owned furniture from the loaded save.
fn apply_loaded_shop(
    loaded: Option<Res<crate::save::LoadedSave>>,
    mut shop: ResMut<ShopData>,
    mut inventory: ResMut<PlayerInventory>,
) {
    let Some(save) = loaded.as_ref().and_then(|l| l.0.as_ref()) else {
        return;
    };
    if !save.shop_stock.is_empty() {
        shop.stock = save.shop_stock.clone();
    }
    shop.shop.sales_sum = save.shop_sales;
    shop.shop.shop_level =
        ShopTier::from_u8(save.shop_tier).unwrap_or(ShopTier::Zakka);
    inventory.furniture = save.furniture.clone();
    println!(
        "Shop restored: {} items in stock, {} sales, tier {:?}, {} owned furniture",
        shop.stock.len(),
        shop.shop.sales_sum,
        shop.shop.shop_level,
        inventory.furniture.len()
    );
}

/// E at the exterior shop door: teleport into the interior.
///
/// Runs in [`ShopInputSet`], before the town `interact` system, so a
/// successful enter (which moves the player far away) means the talk/pickup
/// query finds nothing that same frame.
fn shop_enter(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<ShopLocation>>,
    dialogue: Res<DialogueState>,
    tool: Res<EquippedTool>,
    mail_ui: Res<crate::mail::MailUi>,
    quest_ui: Res<crate::quest::QuestUi>,
    building: Option<Res<ShopBuilding>>,
    mut next_state: ResMut<NextState<ShopLocation>>,
    mut commands: Commands,
    mut player_query: Query<(&mut Transform, &mut Facing), With<Player>>,
) {
    if *state != ShopLocation::Town {
        return;
    }
    if !keyboard.just_pressed(KeyCode::KeyE) || dialogue.active {
        return;
    }
    // The mail or quest UI claims the E key while open.
    if mail_ui.open || quest_ui.open {
        return;
    }
    // A held tool claims E for the ecology plugin; the door needs empty hands.
    if *tool != EquippedTool::None {
        return;
    }
    let Some(building) = building else { return };
    if !building.placed {
        return;
    }
    let Ok((mut transform, mut facing)) = player_query.get_single_mut() else {
        return;
    };
    let to_door = building.door_pos - transform.translation;
    let dist = to_door.length();
    if dist > DOOR_RANGE_OUT || dist < 0.001 {
        return;
    }
    if facing.dir.dot(to_door / dist) < DOOR_DOT {
        return;
    }
    commands.insert_resource(ShopReturnPos(transform.translation));
    transform.translation = SHOP_ORIGIN + Vec3::new(0.0, 1.0, 3.4);
    facing.dir = Vec3::new(0.0, 0.0, -1.0);
    facing.last_pos = transform.translation;
    next_state.set(ShopLocation::Inside);
    println!("Entered the shop");
}

/// E at the interior door: return to the town map. Only when the menu is
/// closed — an open menu claims the E key.
fn shop_exit(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<ShopLocation>>,
    ui: Res<ShopUi>,
    mut next_state: ResMut<NextState<ShopLocation>>,
    return_pos: Option<Res<ShopReturnPos>>,
    mut player_query: Query<(&mut Transform, &mut Facing), With<Player>>,
) {
    if *state != ShopLocation::Inside || ui.open {
        return;
    }
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok((mut transform, mut facing)) = player_query.get_single_mut() else {
        return;
    };
    let door = SHOP_ORIGIN + Vec3::new(0.0, 0.0, 4.0);
    let to_door = door - transform.translation;
    let dist = to_door.length();
    if dist > DOOR_RANGE_IN || dist < 0.001 {
        return;
    }
    if facing.dir.dot(to_door / dist) < 0.5 {
        return;
    }
    transform.translation = return_pos.map(|r| r.0).unwrap_or(Vec3::new(0.0, 1.0, 5.0));
    facing.dir = Vec3::new(0.0, 0.0, 1.0);
    facing.last_pos = transform.translation;
    next_state.set(ShopLocation::Town);
    println!("Left the shop");
}

/// E facing Tom Nook (behind the counter): open the shop menu.
fn nook_interact(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<ShopLocation>>,
    dialogue: Res<DialogueState>,
    mut ui: ResMut<ShopUi>,
    player_query: Query<(&Transform, &Facing), With<Player>>,
    mut panel_query: Query<&mut Visibility, With<ShopPanel>>,
) {
    if *state != ShopLocation::Inside || ui.open || dialogue.active {
        return;
    }
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok((transform, facing)) = player_query.get_single() else {
        return;
    };
    let nook = SHOP_ORIGIN + Vec3::new(0.0, 0.0, -4.1);
    let to_nook = nook - transform.translation;
    let dist = to_nook.length();
    if dist > NOOK_RANGE || dist < 0.001 {
        return;
    }
    if facing.dir.dot(to_nook / dist) < NOOK_DOT {
        return;
    }
    ui.open = true;
    ui.mode = ShopMode::Buy;
    ui.selected = 0;
    ui.message.clear();
    if let Ok(mut vis) = panel_query.get_single_mut() {
        *vis = Visibility::Visible;
    }
    println!("Shop menu opened");
}

/// Sellable player goods: (good id, label, count, unit price).
/// Good ids: 0 = fruit, 1 = fish, 2 = bugs.
fn sell_list(inventory: &PlayerInventory) -> Vec<(u8, String, u32, u32)> {
    let mut list = Vec::new();
    if inventory.fruit > 0 {
        list.push((0, format!("Fruit x{}", inventory.fruit), inventory.fruit, FRUIT_SELL));
    }
    if inventory.fish > 0 {
        list.push((1, format!("Fish x{}", inventory.fish), inventory.fish, FISH_SELL));
    }
    if inventory.bugs > 0 {
        list.push((2, format!("Bugs x{}", inventory.bugs), inventory.bugs, BUG_SELL));
    }
    list
}

/// After a transaction, upgrade the tier if the sales sum earned it
/// (simplified: immediate instead of the retail 2-day remodel).
fn maybe_upgrade(shop: &mut shop::ShopState) -> Option<ShopTier> {
    if shop.real_level() as u8 > shop.shop_level as u8 {
        shop.renew_level();
        Some(shop.shop_level)
    } else {
        None
    }
}

/// Shop menu key handling: 1/2 switch tabs, Up/Down select, Enter confirms.
fn shop_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui: ResMut<ShopUi>,
    mut shop: ResMut<ShopData>,
    mut inventory: ResMut<PlayerInventory>,
    mut panel_query: Query<&mut Visibility, With<ShopPanel>>,
) {
    if !ui.open {
        return;
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        ui.open = false;
        if let Ok(mut vis) = panel_query.get_single_mut() {
            *vis = Visibility::Hidden;
        }
        println!("Shop menu closed");
        return;
    }
    if keyboard.just_pressed(KeyCode::Digit1) {
        ui.mode = ShopMode::Buy;
        ui.selected = 0;
        ui.message.clear();
        return;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        ui.mode = ShopMode::Sell;
        ui.selected = 0;
        ui.message.clear();
        return;
    }

    let list_len = match ui.mode {
        ShopMode::Buy => shop.stock.len(),
        ShopMode::Sell => sell_list(&inventory).len(),
    };
    if list_len == 0 {
        return;
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        ui.selected = ui.selected.checked_sub(1).unwrap_or(list_len - 1);
        return;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        ui.selected = (ui.selected + 1) % list_len;
        return;
    }
    if !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }

    match ui.mode {
        ShopMode::Buy => {
            let Some(item) = shop.stock.get(ui.selected).cloned() else {
                return;
            };
            if inventory.bells < item.price {
                ui.message = "Not enough bells!".to_string();
                return;
            }
            inventory.bells -= item.price;
            inventory.furniture.push(item.name.clone());
            // Full price counts toward the shop upgrade (mSP_PlusSales).
            shop.shop.record_purchase(item.price);
            let mut msg = format!("Bought {}!", item.name);
            if let Some(tier) = maybe_upgrade(&mut shop.shop) {
                msg.push_str(&format!(" Nook's upgraded to {}!", tier_name(tier)));
            }
            ui.message = msg;
            println!("Bought {} for {} bells", item.name, item.price);
        }
        ShopMode::Sell => {
            let list = sell_list(&inventory);
            let Some((good, _, _, unit)) = list.get(ui.selected).cloned() else {
                return;
            };
            // Sell one unit of the selected good.
            let name = match good {
                0 => {
                    inventory.fruit -= 1;
                    "Fruit"
                }
                1 => {
                    inventory.fish -= 1;
                    "Fish"
                }
                _ => {
                    inventory.bugs -= 1;
                    "Bug"
                }
            };
            inventory.bells += unit;
            // Half of Nook's payout counts toward the upgrade (mSP_PlusSales).
            shop.shop.record_sale(unit);
            let mut msg = format!("Sold {name} for {unit} bells!");
            if let Some(tier) = maybe_upgrade(&mut shop.shop) {
                msg.push_str(&format!(" Nook's upgraded to {}!", tier_name(tier)));
            }
            ui.message = msg;
            println!("Sold {name} for {unit} bells");
            // Keep the cursor valid if the row emptied out.
            let new_len = sell_list(&inventory).len();
            if new_len == 0 {
                ui.selected = 0;
            } else {
                ui.selected = ui.selected.min(new_len - 1);
            }
        }
    }
}
/// Refresh the shop menu text every frame while open.
fn refresh_shop_ui(
    ui: Res<ShopUi>,
    shop: Res<ShopData>,
    inventory: Res<PlayerInventory>,
    mut title_q: Query<&mut Text, With<ShopTitleText>>,
    mut list_q: Query<&mut Text, (With<ShopListText>, Without<ShopTitleText>)>,
    mut msg_q: Query<&mut Text, (With<ShopMsgText>, Without<ShopTitleText>, Without<ShopListText>)>,
) {
    if !ui.open {
        return;
    }
    if let Ok(mut t) = title_q.get_single_mut() {
        t.sections[0].value = format!(
            "{}   |   Bells: {}",
            tier_name(shop.shop.shop_level),
            inventory.bells
        );
    }
    if let Ok(mut t) = list_q.get_single_mut() {
        let mut s = String::new();
        match ui.mode {
            ShopMode::Buy => {
                s.push_str("--- For Sale ---\n");
                for (i, item) in shop.stock.iter().enumerate() {
                    let cursor = if i == ui.selected { ">" } else { " " };
                    s.push_str(&format!("{cursor} {} — {} bells\n", item.name, item.price));
                }
            }
            ShopMode::Sell => {
                s.push_str("--- Sell to Nook ---\n");
                let list = sell_list(&inventory);
                if list.is_empty() {
                    s.push_str("Nothing to sell.\n");
                }
                for (i, (_, label, _, price)) in list.iter().enumerate() {
                    let cursor = if i == ui.selected { ">" } else { " " };
                    s.push_str(&format!("{cursor} {label} — {price} bells each\n"));
                }
            }
        }
        // Progress toward the next tier (decomp-verified thresholds).
        let next = match shop.shop.shop_level {
            ShopTier::Zakka => Some(("Nook 'n Go", shop::COMBINI_SUM)),
            ShopTier::Combini => Some(("Nookway", shop::SUPER_SUM)),
            ShopTier::Super => Some(("Nookington's", shop::DSUPER_SUM)),
            ShopTier::Dsuper => None,
        };
        if let Some((name, target)) = next {
            s.push_str(&format!(
                "\nSales toward {name}: {} / {}",
                shop.shop.sales_sum, target
            ));
        } else {
            s.push_str("\nNookington's — fully upgraded!");
        }
        t.sections[0].value = s;
    }
    if let Ok(mut t) = msg_q.get_single_mut() {
        t.sections[0].value = ui.message.clone();
    }
}

/// WASD movement inside the shop: room bounds plus counter/pedestal collision.
fn shop_interior_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<ShopLocation>>,
    ui: Res<ShopUi>,
    mut player_query: Query<(&mut Transform, &mut Facing), With<Player>>,
    obstacles: Res<ShopObstacles>,
    time: Res<Time>,
) {
    if *state != ShopLocation::Inside || ui.open {
        return;
    }
    let Ok((mut transform, mut facing)) = player_query.get_single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction.length() > 0.0 {
        direction = direction.normalize();
        let movement = direction * PLAYER_SPEED * time.delta_seconds();

        let nx = transform.translation.x + movement.x;
        if (nx - SHOP_ORIGIN.x).abs() <= SHOP_WALK_X
            && !hits_obstacle(nx, transform.translation.z, &obstacles)
        {
            transform.translation.x = nx;
        }
        let nz = transform.translation.z + movement.z;
        if (nz - SHOP_ORIGIN.z).abs() <= SHOP_WALK_Z
            && !hits_obstacle(transform.translation.x, nz, &obstacles)
        {
            transform.translation.z = nz;
        }
        facing.dir = Vec3::new(direction.x, 0.0, direction.z);
        facing.last_pos = transform.translation;
    }
}

fn hits_obstacle(x: f32, z: f32, obstacles: &ShopObstacles) -> bool {
    obstacles.0.iter().any(|o| {
        x + PLAYER_RADIUS > o.min_x
            && x - PLAYER_RADIUS < o.max_x
            && z + PLAYER_RADIUS > o.min_z
            && z - PLAYER_RADIUS < o.max_z
    })
}

/// Show/hide the "inside shop" indicator with the state.
fn update_shop_hud(
    state: Res<State<ShopLocation>>,
    mut query: Query<&mut Visibility, With<ShopHudText>>,
) {
    for mut vis in query.iter_mut() {
        *vis = match *state.get() {
            ShopLocation::Inside => Visibility::Visible,
            ShopLocation::Town => Visibility::Hidden,
        };
    }
}

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ShopLocation>()
            .insert_resource(ShopData::default())
            .insert_resource(ShopUi::default())
            .add_systems(Startup, find_shop_spot.after(crate::town::generate_town))
            .add_systems(
                Startup,
                (
                    spawn_shop_exterior.after(find_shop_spot),
                    build_shop_interior,
                    setup_shop_ui,
                    apply_loaded_shop,
                ),
            )
            .add_systems(Update, shop_enter.in_set(ShopInputSet))
            .add_systems(Update, (shop_exit, nook_interact, shop_menu_input))
            .add_systems(Update, (refresh_shop_ui, shop_interior_movement, update_shop_hud));
    }
}
