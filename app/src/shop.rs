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
use bevy::text::FontSize;
use rustimal_logic::shop::{self, ShopTier};
use serde::{Deserialize, Serialize};

use crate::ecology::EquippedTool;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{DoorPanel, Facing, PlayerInventory};
use crate::items::{self, ItemId};
use crate::locomotion::{self, Locomotion};
use crate::player::Player;
use crate::states::{AppState, GameplaySet};
use crate::town::Town;

/// Town map vs. shop interior. Lives only while in game.
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::InGame)]
pub enum ShopLocation {
    #[default]
    Town,
    Inside,
}

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
    /// Ground height under the building (disc terrain), 0 on flat ground.
    pub base_y: f32,
}

/// True when (x, z) is inside the shop building's blocked footprint.
pub fn point_blocked(building: &ShopBuilding, x: f32, z: f32) -> bool {
    building.placed
        && x > building.min_x
        && x < building.max_x
        && z > building.min_z
        && z < building.max_z
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
        StockItem {
            name: "Wooden Chair".into(),
            price: 800,
        },
        StockItem {
            name: "Potted Plant".into(),
            price: 900,
        },
        StockItem {
            name: "Wall Clock".into(),
            price: 1100,
        },
        StockItem {
            name: "Modern Lamp".into(),
            price: 1400,
        },
        StockItem {
            name: "Bookshelf".into(),
            price: 1800,
        },
        StockItem {
            name: "Cozy Bed".into(),
            price: 2000,
        },
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
/// Half the exterior door panel's width (panel is 1.2 wide).
const DOOR_HALF_W: f32 = 0.6;
/// Interior door z offset from the origin: inner face of the south wall.
const INTERIOR_DOOR_Z: f32 = 4.8;
/// Half the interior door panel's width (panel is 1.4 wide).
const INTERIOR_DOOR_HALF_W: f32 = 0.7;
const NOOK_RANGE: f32 = 3.5;
const NOOK_DOT: f32 = 0.5;
/// Interior walk bounds (half-extents from the origin).
const SHOP_WALK_X: f32 = 5.2;
const SHOP_WALK_Z: f32 = 4.2;
const PLAYER_RADIUS: f32 = 0.4;

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
    // The retail layout says where the shop is.
    if let Some(at) = town.retail.as_ref().and_then(|r| r.shop) {
        let center = Vec3::new(at.x, 0.0, at.z - SHOP_D / 2.0 + 0.5);
        let base_y = town.ground_y(at);
        commands.insert_resource(ShopBuilding {
            door_pos: Vec3::new(center.x, base_y, center.z + SHOP_D / 2.0 + 0.7),
            min_x: center.x - SHOP_W / 2.0,
            max_x: center.x + SHOP_W / 2.0,
            min_z: center.z - SHOP_D / 2.0,
            max_z: center.z + SHOP_D / 2.0,
            placed: true,
            base_y,
        });
        println!("Shop placed from the retail layout at {center:?}");
        return;
    }
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
                    if !town.is_clear_ground(p) {
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
            if gz + RD >= GD || !town.is_clear_ground(door_cell) {
                continue;
            }
            let d2 = (gx + RW / 2 - cx).pow(2) + (gz + RD / 2 - cz).pow(2);
            if best.is_none_or(|(_, _, b)| d2 < b) {
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
        base_y: 0.0,
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
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(SHOP_W, SHOP_H, SHOP_D))),
        MeshMaterial3d(materials.add(Color::srgb(0.75, 0.6, 0.4))),
        Transform::from_xyz(cx, building.base_y + SHOP_H / 2.0, cz),
    ));
    // Roof slab.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(SHOP_W + 0.6, 0.5, SHOP_D + 0.6))),
        MeshMaterial3d(materials.add(Color::srgb(0.6, 0.25, 0.2))),
        Transform::from_xyz(cx, building.base_y + SHOP_H + 0.25, cz),
    ));
    // Door panel on the south face.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.2, 2.2, 0.15))),
        MeshMaterial3d(materials.add(Color::srgb(0.35, 0.22, 0.12))),
        Transform::from_xyz(cx, building.base_y + 1.1, building.max_z + 0.05),
    ));
    // Sign board above the door.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.6, 0.8, 0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.95, 0.9, 0.75))),
        Transform::from_xyz(cx, building.base_y + 2.7, building.max_z + 0.08),
    ));
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
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(12.0, 10.0))),
        MeshMaterial3d(floor_mat),
        Transform::from_xyz(ox, 0.02, oz),
    ));

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
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(sx, wall_h, sz))),
            MeshMaterial3d(wall_mat.clone()),
            Transform::from_xyz(wx, wall_h / 2.0, wz),
        ));
    }

    // Door panel on the south wall (visual only; exit is via E).
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.4, 2.4, 0.15))),
        MeshMaterial3d(door_mat),
        Transform::from_xyz(ox, 1.2, oz + 4.85),
    ));

    // Counter at the back (north side).
    let counter = Obstacle {
        min_x: ox - 2.25,
        max_x: ox + 2.25,
        min_z: oz - 3.5,
        max_z: oz - 2.5,
    };
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(4.5, 1.1, 1.0))),
        MeshMaterial3d(counter_mat),
        Transform::from_xyz(ox, 0.55, oz - 3.0),
    ));

    // Tom Nook behind the counter.
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.4, 1.0))),
        MeshMaterial3d(nook_mat),
        Transform::from_xyz(ox, 1.0, oz - 4.1),
    ));

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
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.9, 0.8, 0.9))),
            MeshMaterial3d(materials.add(Color::srgb(0.5, 0.35, 0.2))),
            Transform::from_xyz(ox + lx, 0.4, oz + lz),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.55, 0.55, 0.55))),
            MeshMaterial3d(materials.add(Color::srgb(*r, *g, *b))),
            Transform::from_xyz(ox + lx, 1.1, oz + lz),
        ));
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
        Text::new("Nook's — E at Nook to shop, E at the door to exit"),
        TextFont {
            font_size: FontSize::Px(22.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(50.0),
            ..default()
        },
        Visibility::Hidden,
    ));

    println!("Shop interior built at {:?}", SHOP_ORIGIN);
}

/// Build the shop menu UI (hidden until opened).
fn setup_shop_ui(mut commands: Commands) {
    commands
        .spawn((
            ShopPanel,
            Node {
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
            BackgroundColor(Color::srgba(0.10, 0.08, 0.05, 0.96)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                ShopTitleText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(28.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.5)),
            ));
            parent.spawn((
                ShopListText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                ShopMsgText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.4)),
            ));
            parent.spawn((
                ShopHintText,
                Text::new("1: Buy   2: Sell   Up/Down: select   Enter: confirm   Esc: close"),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.75)),
            ));
        });
    println!("Shop UI ready");
}

/// Restore shop stock/sales/tier and owned furniture from the loaded save.
fn apply_loaded_shop(loaded: Option<Res<crate::save::LoadedSave>>, mut shop: ResMut<ShopData>) {
    let Some(save) = loaded.as_ref().and_then(|l| l.0.as_ref()) else {
        return;
    };
    if !save.shop_stock.is_empty() {
        shop.stock = save.shop_stock.clone();
    }
    shop.shop.sales_sum = save.shop_sales;
    shop.shop.shop_level = ShopTier::from_u8(save.shop_tier).unwrap_or(ShopTier::Zakka);
    println!(
        "Shop restored: {} items in stock, {} sales, tier {:?}",
        shop.stock.len(),
        shop.shop.sales_sum,
        shop.shop.shop_level
    );
}

/// True when a player at `pos` facing `facing_dir` is close to, and facing,
/// the exterior door panel.
///
/// Measured on the ground (XZ) plane against the nearest point of the door
/// panel on the south wall. `door_pos` itself sits 0.7 out from the wall at
/// y = 0, so measuring to it rejected a player pressed against the door: the
/// point was beside/behind them and the 1-unit height gap skewed the dot.
fn can_enter_door(building: &ShopBuilding, pos: Vec3, facing_dir: Vec3) -> bool {
    DoorPanel {
        center_x: building.door_pos.x,
        half_w: DOOR_HALF_W,
        face_z: building.max_z,
        into_z: -1.0,
    }
    .faced_by(pos, facing_dir, DOOR_RANGE_OUT, DOOR_DOT)
}

/// E at the exterior shop door: teleport into the interior.
///
/// Runs in [`InteractSet::Doors`]: a successful enter consumes the press,
/// so talk/pickup can't also fire on it.
fn shop_enter(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    tool: Res<EquippedTool>,
    building: Option<Res<ShopBuilding>>,
    mut next_state: ResMut<NextState<ShopLocation>>,
    mut commands: Commands,
    mut player_query: Query<(&mut Transform, &mut Facing), With<Player>>,
) {
    if *state != ShopLocation::Town {
        return;
    }
    if !actions.just_pressed(Action::Interact) || !focus.is_world() {
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
    let Ok((mut transform, mut facing)) = player_query.single_mut() else {
        return;
    };
    if !can_enter_door(&building, transform.translation, facing.dir)
        || !actions.consume(Action::Interact)
    {
        return;
    }
    commands.insert_resource(ShopReturnPos(transform.translation));
    transform.translation = SHOP_ORIGIN + Vec3::new(0.0, 1.0, 3.4);
    facing.dir = Vec3::new(0.0, 0.0, -1.0);
    next_state.set(ShopLocation::Inside);
    println!("Entered the shop");
}

/// E at the interior door: return to the town map. Only when the menu is
/// closed — an open menu claims the E key.
fn shop_exit(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    mut next_state: ResMut<NextState<ShopLocation>>,
    return_pos: Option<Res<ShopReturnPos>>,
    mut player_query: Query<(&mut Transform, &mut Facing), With<Player>>,
) {
    if *state != ShopLocation::Inside || !focus.is_world() {
        return;
    }
    if !actions.just_pressed(Action::Interact) {
        return;
    }
    let Ok((mut transform, mut facing)) = player_query.single_mut() else {
        return;
    };
    // The door panel on the inner face of the south wall.
    let door = DoorPanel {
        center_x: SHOP_ORIGIN.x,
        half_w: INTERIOR_DOOR_HALF_W,
        face_z: SHOP_ORIGIN.z + INTERIOR_DOOR_Z,
        into_z: 1.0,
    };
    if !door.faced_by(transform.translation, facing.dir, DOOR_RANGE_IN, 0.5)
        || !actions.consume(Action::Interact)
    {
        return;
    }
    transform.translation = return_pos.map(|r| r.0).unwrap_or(Vec3::new(0.0, 1.0, 5.0));
    facing.dir = Vec3::new(0.0, 0.0, 1.0);
    next_state.set(ShopLocation::Town);
    println!("Left the shop");
}

/// E facing Tom Nook (behind the counter): open the shop menu.
fn nook_interact(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    mut ui: ResMut<ShopUi>,
    player_query: Query<(&Transform, &Facing), With<Player>>,
    mut panel_query: Query<&mut Visibility, With<ShopPanel>>,
) {
    if *state != ShopLocation::Inside || ui.open || !focus.is_world() {
        return;
    }
    if !actions.just_pressed(Action::Interact) {
        return;
    }
    let Ok((transform, facing)) = player_query.single() else {
        return;
    };
    let nook = SHOP_ORIGIN + Vec3::new(0.0, 0.0, -4.1);
    let to_nook = nook - transform.translation;
    let dist = to_nook.length();
    if dist > NOOK_RANGE || dist < 0.001 {
        return;
    }
    if facing.dir.dot(to_nook / dist) < NOOK_DOT || !actions.consume(Action::Interact) {
        return;
    }
    ui.open = true;
    ui.mode = ShopMode::Buy;
    ui.selected = 0;
    ui.message.clear();
    if let Ok(mut vis) = panel_query.single_mut() {
        *vis = Visibility::Visible;
    }
    println!("Shop menu opened");
}

/// Sellable goods in the pockets, grouped by item in pocket order:
/// (item, label, count, unit price).
fn sell_list(inventory: &PlayerInventory) -> Vec<(ItemId, String, u32, u32)> {
    let mut list: Vec<(ItemId, String, u32, u32)> = Vec::new();
    for &item in inventory.pockets.pockets.iter() {
        let Some(price) = items::sell_price(item) else {
            continue;
        };
        if list.iter().any(|(it, ..)| *it == item) {
            continue;
        }
        let count = inventory.count(item);
        list.push((
            item,
            format!("{} x{count}", items::name(item)),
            count,
            price,
        ));
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
    actions: Res<Actions>,
    mut ui: ResMut<ShopUi>,
    mut shop: ResMut<ShopData>,
    mut inventory: ResMut<PlayerInventory>,
    mut panel_query: Query<&mut Visibility, With<ShopPanel>>,
) {
    if !ui.open {
        return;
    }
    if actions.just_pressed(Action::Cancel) {
        ui.open = false;
        if let Ok(mut vis) = panel_query.single_mut() {
            *vis = Visibility::Hidden;
        }
        println!("Shop menu closed");
        return;
    }
    if actions.just_pressed(Action::Tab1) {
        ui.mode = ShopMode::Buy;
        ui.selected = 0;
        ui.message.clear();
        return;
    }
    if actions.just_pressed(Action::Tab2) {
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
    if actions.just_pressed(Action::MenuUp) {
        ui.selected = ui.selected.checked_sub(1).unwrap_or(list_len - 1);
        return;
    }
    if actions.just_pressed(Action::MenuDown) {
        ui.selected = (ui.selected + 1) % list_len;
        return;
    }
    if !actions.just_pressed(Action::Confirm) {
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
            let Some(id) = items::furniture(&item.name) else {
                ui.message = format!("{} isn't for sale.", item.name);
                return;
            };
            if !inventory.add(id) {
                ui.message = "Your pockets are full!".to_string();
                return;
            }
            inventory.bells -= item.price;
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
            // Sell one of the selected item.
            if !inventory.remove(good) {
                return;
            }
            let name = items::name(good);
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
    mut msg_q: Query<
        &mut Text,
        (
            With<ShopMsgText>,
            Without<ShopTitleText>,
            Without<ShopListText>,
        ),
    >,
) {
    if !ui.open {
        return;
    }
    if let Ok(mut t) = title_q.single_mut() {
        t.0 = format!(
            "{}   |   Bells: {}",
            tier_name(shop.shop.shop_level),
            inventory.bells
        );
    }
    if let Ok(mut t) = list_q.single_mut() {
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
        t.0 = s;
    }
    if let Ok(mut t) = msg_q.single_mut() {
        t.0 = ui.message.clone();
    }
}

/// WASD movement inside the shop: room bounds plus counter/pedestal collision.
fn shop_interior_movement(
    actions: Res<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    mut player_query: Query<(&mut Transform, &mut Facing, &mut Locomotion), With<Player>>,
    obstacles: Res<ShopObstacles>,
    time: Res<Time>,
) {
    if *state != ShopLocation::Inside || !focus.is_world() {
        return;
    }
    let Ok((mut transform, mut facing, mut loco)) = player_query.single_mut() else {
        return;
    };

    let movement = locomotion::step(&actions, time.delta().as_secs_f32(), &mut facing, &mut loco);
    if movement != Vec3::ZERO {
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
        app.add_sub_state::<ShopLocation>()
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
            .add_systems(
                Update,
                (shop_enter, shop_exit)
                    .in_set(InteractSet::Doors)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                nook_interact
                    .in_set(InteractSet::Objects)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                shop_menu_input.in_set(InteractSet::Ui).in_set(GameplaySet),
            )
            .add_systems(
                Update,
                (refresh_shop_ui, shop_interior_movement, update_shop_hud).in_set(GameplaySet),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The live layout: footprint x -6..-1, z -2..2, door at (-3.5, 0, 2.7).
    fn building() -> ShopBuilding {
        ShopBuilding {
            door_pos: Vec3::new(-3.5, 0.0, 2.7),
            min_x: -6.0,
            max_x: -1.0,
            min_z: -2.0,
            max_z: 2.0,
            placed: true,
            base_y: 0.0,
        }
    }

    const NORTH: Vec3 = Vec3::new(0.0, 0.0, -1.0);

    #[test]
    fn enters_when_pressed_against_door_facing_it() {
        // The reported bug: player walked into the door (y = 1) and faced it.
        assert!(can_enter_door(
            &building(),
            Vec3::new(-3.5, 1.0, 2.04),
            NORTH
        ));
        // Flush against the face (seen live once movement followed the
        // smoothed facing): must still count.
        assert!(can_enter_door(
            &building(),
            Vec3::new(-3.5, 1.0, 2.0007),
            NORTH
        ));
    }

    #[test]
    fn enters_from_door_edge_and_from_a_few_steps_back() {
        assert!(can_enter_door(
            &building(),
            Vec3::new(-3.0, 1.0, 2.04),
            NORTH
        ));
        assert!(can_enter_door(
            &building(),
            Vec3::new(-3.5, 1.0, 4.6),
            NORTH
        ));
    }

    #[test]
    fn rejects_facing_away_or_too_far() {
        let south = Vec3::new(0.0, 0.0, 1.0);
        assert!(!can_enter_door(
            &building(),
            Vec3::new(-3.5, 1.0, 2.5),
            south
        ));
        assert!(!can_enter_door(
            &building(),
            Vec3::new(-3.5, 1.0, 6.0),
            NORTH
        ));
    }

    #[test]
    fn rejects_beside_the_door_along_the_wall() {
        // Against the wall 2 units east of the panel, facing the wall.
        assert!(!can_enter_door(
            &building(),
            Vec3::new(-1.5, 1.0, 2.04),
            NORTH
        ));
    }
}
