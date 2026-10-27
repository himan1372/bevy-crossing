//! Phase 3: Tom Nook's shop.
//!
//! A shop building sits on the town map (placed on a free 5x4 grass rect
//! near the town center). Pressing E at the door teleports the player into
//! a shop interior built at a distant world offset (same pattern as the
//! house plugin). Disc gameplay uses `shop_nook` for merchandise dialogue
//! and pockets selling; the no-disc placeholder keeps its counter menu.
//!
//! Logic-crate usage: `rustimal_logic::shop::{ShopState, ShopTier}` drives
//! sales-sum tracking and the tier/upgrade display (`mSP_PlusSales` clamp,
//! `mSP_GetRealShopLevel` thresholds, `mSP_RenewShopLevel`). Stock,
//! pricing, the UI, and the building itself are Bevy-side; original price
//! tables are installed at runtime by `items` from the user's disc.
//! Saved renovations and business hours follow the original calendar lifecycle.

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

pub(crate) mod bargain;
pub(crate) mod layout;
pub(crate) mod schedule;
pub(crate) mod turnips;

/// Roots owned by the current shop room, replaced on tier/floor changes.
#[derive(Component)]
pub(crate) struct ShopInterior;

/// Town map vs. a public building's interior (the shop, or the post
/// office). Lives only while in game.
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::InGame)]
pub enum ShopLocation {
    #[default]
    Town,
    Inside,
    PostOffice,
    PoliceBox,
    Museum,
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
    /// aSHOP_check_player's point (actor x - 38, z + 42), for the disc's shop.
    pub trigger: Option<Vec3>,
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
    /// The game's item number (0 in saves from before real stock).
    #[serde(default)]
    pub item: ItemId,
    pub name: String,
    pub price: u32,
    /// Bought today: its slot shows a sold-out sign.
    #[serde(default)]
    pub sold: bool,
}

impl StockItem {
    pub(crate) fn new(item: ItemId) -> Self {
        Self {
            item,
            name: items::name(item),
            price: items::price(item).unwrap_or(0),
            sold: false,
        }
    }
}

/// Shop runtime data: today's stock plus the decomp-verified sales ledger.
#[derive(Resource, Default)]
pub struct ShopData {
    pub(crate) turnips: turnips::Market,
    pub(crate) events: bargain::Calendar,
    pub(crate) sale_view: bool,
    pub stock: Vec<StockItem>,
    pub shop: shop::ShopState,
    /// Rarity of lists A, B, C per kind; decided with the first stock.
    pub priorities: Option<[[u8; 3]; 5]>,
    /// Game day of the current stock (`exchange_time`).
    pub stock_day: Option<i64>,
    /// Next paint colour (`shop_info.paint_color`).
    pub paint: u8,
    /// `Shop_c.rare_item`: today's spotlight furniture, 0 for none.
    pub rare_item: ItemId,
    /// The last play time the rare-furniture notice saved (`goki.time`).
    pub played: Option<(u16, u8, u8, u8)>,
}

impl ShopData {
    /// Restores stock, sales and tier from a save; stock from before real
    /// item numbers is dropped and laid out again.
    pub fn restore(&mut self, save: &crate::save::SaveData) {
        self.turnips = save.shop_turnips.clone().unwrap_or_default();
        self.events = save.shop_events.clone().unwrap_or_default();
        self.sale_view = false;
        self.stock = save.shop_stock.clone();
        if self.stock.iter().any(|s| s.item == 0) {
            self.stock.clear();
            self.stock_day = None;
        } else {
            self.stock_day = save.shop_stock_day;
        }
        self.priorities = save.shop_priorities;
        self.paint = save.shop_paint;
        self.rare_item = save.shop_rare;
        crate::goods::set_rare_item(self.rare_item);
        self.shop.sales_sum = save.shop_sales;
        self.shop.shop_level = ShopTier::from_u8(save.shop_tier).unwrap_or(ShopTier::Zakka);
        let progress = save.shop_progress.clone().unwrap_or_default();
        self.shop.visitor_flag = progress.visitor;
        self.shop.upgrading_today = progress.pending;
        self.shop.renewal_ymd = progress.renewal;
        self.shop.send_upgrade_notice = progress.notice;
        self.played = progress.played;
        if schedule::advance(
            &mut self.shop,
            crate::clock::now(),
            self.events.bargain_date(),
        ) {
            self.stock_day = None;
        }
        if let Some(p) = self.priorities {
            crate::goods::set_priorities(p);
        }
    }

    /// Indices of the goods still for sale.
    fn for_sale(&self) -> Vec<usize> {
        (0..self.goods().len())
            .filter(|&i| !self.goods()[i].sold)
            .collect()
    }

    pub(crate) fn goods(&self) -> &[StockItem] {
        if self.sale_view {
            if let Some(offer) = self.events.offer.as_ref() {
                return &offer.items;
            }
        }
        &self.stock
    }

    pub(crate) fn goods_mut(&mut self) -> &mut [StockItem] {
        if self.sale_view {
            if let Some(offer) = self.events.offer.as_mut() {
                return &mut offer.items;
            }
        }
        &mut self.stock
    }

    /// Lays out the day's goods (`mSP_ExchangeLineUp_InGame`).
    fn restock(&mut self, day: i64) {
        let mut state = (day as u64) ^ 0x9E37_79B9_7F4A_7C15 ^ u64::from(self.shop.sales_sum);
        let mut below = |n: u32| -> u32 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state % u64::from(n.max(1))) as u32
        };
        let priorities = *self
            .priorities
            .get_or_insert_with(|| crate::goods::decide_priorities(&mut below));
        crate::goods::set_priorities(priorities);
        let now = crate::clock::now();
        let roll = crate::goods::Roll {
            year: now.year,
            priorities,
            goods_power: 0,
            month: now.month,
            day: now.day,
        };
        let goods = crate::goods::make_goods_list(
            &roll,
            self.shop.shop_level as usize,
            self.shop.sales_sum,
            &mut self.paint,
            &mut self.rare_item,
            &mut below,
        );
        self.stock = goods.into_iter().map(StockItem::new).collect();
        self.stock_day = Some(day);
        info!(
            "Nook's goods today: {}",
            self.stock
                .iter()
                .map(|s| format!("{} ({})", s.name, s.price))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

/// New goods each day, once the disc's lists are loaded.
fn restock_daily(mut shop: ResMut<ShopData>, disc: Option<Res<crate::game_disc::GameDisc>>) {
    if disc.is_none() {
        return;
    }
    let today = crate::clock::now().day_number;
    if shop.stock_day != Some(today) {
        shop.restock(today);
    }
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
pub const SHOP_ORIGIN: Vec3 = Vec3::new(3000.0, 0.0, -3000.0);
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
pub(crate) struct ShopObstacles(Vec<Obstacle>);

struct Obstacle {
    min_x: f32,
    max_x: f32,
    min_z: f32,
    max_z: f32,
}

/// Find a free 5x4 grass rect near the town center for the shop building,
/// scanning outward from the middle of the map.
fn find_shop_spot(town: Res<Town>, mut commands: Commands) {
    // The retail layout says where the shop is (drawn by disc_town).
    if let Some(b) = town.retail.as_ref().and_then(|r| {
        r.buildings
            .iter()
            .find(|b| b.item == crate::disc_town::SHOP0)
    }) {
        let base_y = town.ground_y(b.centre);
        // A step out from the door, away from the building's centre.
        let door = b.door.unwrap_or(b.centre + Vec3::Z * b.half.y);
        let out = (door - b.centre).with_y(0.0).normalize_or(Vec3::Z);
        let door_pos = (door + out * 0.7).with_y(base_y);
        let inner = b.half.min_element() * std::f32::consts::FRAC_1_SQRT_2;
        commands.insert_resource(ShopBuilding {
            door_pos,
            // The town blocks the real (possibly 45-deg) footprint; this
            // box only needs to stay inside it.
            min_x: b.centre.x - inner,
            max_x: b.centre.x + inner,
            min_z: b.centre.z - inner,
            max_z: b.centre.z + inner,
            placed: true,
            base_y,
            // ac_shop.c: the actor stands at the layout point shifted (-20, +20).
            trigger: Some(b.origin + Vec3::new(-38.0, 0.0, 42.0) / 40.0),
        });
        println!(
            "Shop placed from the retail layout at {:?}, door {door_pos:?}, trigger {:?}",
            b.centre,
            b.origin + Vec3::new(-38.0, 0.0, 42.0) / 40.0
        );
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
        trigger: None,
    });
    println!("Shop placed at {:?}, door at {:?}", center, door_pos);
}

/// Spawn the exterior shop building mesh at the chosen spot.
fn spawn_shop_exterior(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    building: Res<ShopBuilding>,
    town: Res<Town>,
) {
    // With the disc, the shop model is drawn by disc_town.
    if !building.placed || town.retail.is_some() {
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

/// Replace only owned room roots when tier, floor or seasonal layout changes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_shop_interior(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    disc: Option<Res<crate::game_disc::GameDisc>>,
    mut n64: ResMut<Assets<crate::n64_material::N64Material>>,
    mut images: ResMut<Assets<Image>>,
    shop: Res<ShopData>,
    active: Res<layout::Layout>,
    owned: Query<Entity, With<ShopInterior>>,
    mut built: Local<Option<(layout::Layout, bool, bool)>>,
) {
    let ox = SHOP_ORIGIN.x;
    let oz = SHOP_ORIGIN.z;
    // Raffle changes the shell; bargain sales change only native FG placement.
    let today = crate::clock::now();
    let fukubiki =
        rustimal_logic::shop::tanuki_shop_status(today.year, today.month, today.day, false)
            == rustimal_logic::shop::TanukiShopStatus::Fukubiki;
    let layout = layout::Layout {
        tier: shop.shop.shop_level as u8,
        upper: shop.shop.shop_level == ShopTier::Dsuper && active.upper && !fukubiki,
        sale: shop
            .events
            .offer
            .as_ref()
            .filter(|_| shop.sale_view)
            .map(|o| o.kind),
    };
    let key = (layout, fukubiki, disc.is_some());
    if *built == Some(key) {
        return;
    }
    for entity in &owned {
        commands.entity(entity).despawn();
    }
    *built = Some(key);
    commands.insert_resource(layout);
    // The disc's room, or the placeholder; the counter, Nook and the
    // stands keep their arrangement scaled into it.
    let real = disc.as_deref().is_some_and(|d| {
        crate::interior::spawn_shop_room(
            &mut commands,
            d,
            SHOP_ORIGIN,
            fukubiki,
            layout,
            &mut meshes,
            &mut n64,
            &mut images,
        )
    });
    // House_Clock_Actor_ct: shops 0-4 have no wall clock on raffle day.
    if real && !fukubiki {
        if let Some(d) = disc.as_deref() {
            crate::furniture::house_clock::spawn_shop_clock(
                &mut commands,
                d,
                SHOP_ORIGIN,
                layout,
                (&mut meshes, &mut n64, &mut images),
            );
        }
    }
    let k = if real { ShopRoom::disc(layout).k } else { 1.0 };
    commands.insert_resource(if real {
        ShopRoom::disc(layout)
    } else {
        ShopRoom::default()
    });

    let floor_mat = materials.add(Color::srgb(0.6, 0.44, 0.26));
    let wall_mat = materials.add(Color::srgb(0.88, 0.8, 0.66));
    let counter_mat = materials.add(Color::srgb(0.45, 0.3, 0.16));
    let door_mat = materials.add(Color::srgb(0.4, 0.25, 0.12));
    let nook_mat = materials.add(Color::srgb(0.55, 0.38, 0.22));

    // Placeholder floor, walls and door (the disc room has its own).
    if !real {
        commands.spawn((
            ShopInterior,
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
                ShopInterior,
                Mesh3d(meshes.add(Cuboid::new(sx, wall_h, sz))),
                MeshMaterial3d(wall_mat.clone()),
                Transform::from_xyz(wx, wall_h / 2.0, wz),
            ));
        }

        // Door panel on the south wall (visual only; exit is via E).
        commands.spawn((
            ShopInterior,
            Mesh3d(meshes.add(Cuboid::new(1.4, 2.4, 0.15))),
            MeshMaterial3d(door_mat),
            Transform::from_xyz(ox, 1.2, oz + 4.85),
        ));
    }

    // Counter at the back (north side).
    let counter = Obstacle {
        min_x: ox - 2.25 * k,
        max_x: ox + 2.25 * k,
        min_z: oz - 3.5 * k,
        max_z: oz - 2.5 * k,
    };
    // The disc room has its own counter and tables.
    if !real {
        commands.spawn((
            ShopInterior,
            Mesh3d(meshes.add(Cuboid::new(4.5 * k, 1.1 * k, 1.0 * k))),
            MeshMaterial3d(counter_mat),
            Transform::from_xyz(ox, 0.55 * k, oz - 3.0 * k),
        ));
    }

    // Tom Nook behind the counter (his disc model replaces the capsule).
    commands.spawn((
        Name::new(if layout.upper { "Timmy" } else { "Tom Nook" }),
        crate::shop_nook::Shopkeeper {
            twin: layout.upper.then_some(0),
            ..default()
        },
        ShopInterior,
        crate::character::NpcModel(layout.npc()),
        crate::character::NpcClip::default(),
        Mesh3d(meshes.add(Capsule3d::new(0.4, 1.0))),
        MeshMaterial3d(nook_mat),
        Transform::from_translation(if real {
            SHOP_ORIGIN + layout.npc_position()
        } else {
            Vec3::new(ox, 1.0, oz - 4.1 * k)
        })
        .with_rotation(if layout.upper {
            // `aNMD_actor_ct`: both twins start facing 180 degrees.
            Quat::from_rotation_y(std::f32::consts::PI)
        } else {
            Quat::IDENTITY
        }),
    ));
    if real && layout.upper {
        commands.spawn((
            ShopInterior,
            Name::new("Tommy"),
            crate::shop_nook::Shopkeeper {
                twin: Some(1),
                ..default()
            },
            crate::character::NpcModel(layout.sibling_npc()),
            crate::character::NpcClip::default(),
            Visibility::default(),
            // `aNMD_actor_ct`: both twins start facing 180 degrees.
            Transform::from_translation(SHOP_ORIGIN + layout.sibling_position())
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        ));
    }

    // Display pedestals with sample goods (colors only; names/prices live
    // in the shop menu).
    let mut obstacles = if real {
        disc.as_deref()
            .map(|d| room_obstacles(d, layout))
            .unwrap_or_default()
    } else {
        vec![counter]
    };
    let pedestal_cols = [
        (0.75, 0.2, 0.2),
        (0.25, 0.4, 0.8),
        (0.9, 0.8, 0.4),
        (0.3, 0.7, 0.3),
    ];
    for (i, (r, g, b)) in pedestal_cols.iter().enumerate().filter(|_| !real) {
        let lx = (-3.75 + i as f32 * 2.5) * k;
        let lz = 0.8 * k;
        commands.spawn((
            ShopInterior,
            Mesh3d(meshes.add(Cuboid::new(0.9, 0.8, 0.9))),
            MeshMaterial3d(materials.add(Color::srgb(0.5, 0.35, 0.2))),
            Transform::from_xyz(ox + lx, 0.4, oz + lz),
        ));
        commands.spawn((
            ShopInterior,
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
        ShopInterior,
        ShopHudText,
        crate::dev_overlay::DevOverlay,
        Text::new(if real {
            "Nook's — E facing goods to browse, E at Nook to sell; E at the door to exit"
        } else {
            "Nook's — E at Nook to shop, E at the door to exit"
        }),
        TextFont {
            font_size: FontSize::Px(22.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(70.0),
            left: Val::Px(50.0),
            ..default()
        },
        Visibility::Hidden,
    ));

    println!("Shop interior built at {:?}", SHOP_ORIGIN);
}

/// The shop room's size: the placeholder, or the disc's room with the
/// layout scaled by `k` to fit inside its walls.
#[derive(Resource, Clone, Copy)]
struct ShopRoom {
    k: f32,
    walk_x: f32,
    walk_z: f32,
    door_z: f32,
    door_x: f32,
}

impl Default for ShopRoom {
    fn default() -> Self {
        Self {
            k: 1.0,
            walk_x: SHOP_WALK_X,
            walk_z: SHOP_WALK_Z,
            door_z: INTERIOR_DOOR_Z,
            door_x: 0.0,
        }
    }
}

impl ShopRoom {
    /// Native room bounds and the chosen layout's south-wall doorway.
    fn disc(layout: layout::Layout) -> Self {
        let half = layout.half_z();
        Self {
            k: half / 5.0,
            walk_x: layout.half_x() - 0.4,
            walk_z: half - 0.4,
            door_z: layout.door_z(),
            door_x: layout.door_x(),
        }
    }
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
    shop.restore(save);
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
    mut player_query: Query<(&mut Transform, &mut Facing, &Locomotion), With<Player>>,
    room: Res<ShopRoom>,
    shop: Res<ShopData>,
    layout: Res<layout::Layout>,
) {
    if *state != ShopLocation::Town || !focus.is_world() {
        return;
    }
    let Some(building) = building else { return };
    if !building.placed {
        return;
    }
    let Ok((mut transform, mut facing, loco)) = player_query.single_mut() else {
        return;
    };
    // A held tool claims E for the ecology plugin; the door needs empty hands.
    let pressed = actions.just_pressed(Action::Interact) && *tool == EquippedTool::None;
    if let Some(trigger) = building.trigger {
        // aSHOP_check_player / _check_player2: within sqrt(550) game units of
        // the door point facing between east and north, walking into it (E
        // there works too).
        let d = (transform.translation - trigger).with_y(0.0) * 40.0;
        let yaw = facing.dir.x.atan2(facing.dir.z).to_degrees();
        let at_door = d.length_squared() < 550.0 && yaw > 90.0 && yaw < 180.0;
        let walking = loco.state() != rustimal_logic::player_move::LocomotionState::Wait;
        if !at_door || !(walking || pressed) {
            return;
        }
        if pressed {
            actions.consume(Action::Interact);
        }
    } else if !pressed
        || !can_enter_door(&building, transform.translation, facing.dir)
        || !actions.consume(Action::Interact)
    {
        return;
    }
    if !schedule::is_open(&shop.shop, crate::clock::now())
        || shop.events.closed(crate::clock::now())
    {
        return;
    }
    commands.insert_resource(ShopReturnPos(transform.translation));
    transform.translation = SHOP_ORIGIN
        + if building.trigger.is_some() {
            layout.entry()
        } else {
            Vec3::new(room.door_x, 1.0, room.door_z - 0.8)
        };
    facing.dir = Vec3::new(0.0, 0.0, -1.0);
    crate::sound::door_sound(Some(shop.shop.shop_level), true);
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
    room: Res<ShopRoom>,
    shop: Res<ShopData>,
    layout: Res<layout::Layout>,
    disc: Option<Res<crate::game_disc::GameDisc>>,
    mut twins: ResMut<crate::shop_nook::Twins>,
) {
    use crate::shop_nook::Goodbye;
    if *state != ShopLocation::Inside || !focus.is_world() || layout.upper {
        return;
    }
    // The keeper's goodbye (`aNSC_goodbye_wait`, message 35) has ended: go.
    let said_goodbye = twins.goodbye == Goodbye::Done;
    if !said_goodbye && !actions.just_pressed(Action::Interact) {
        return;
    }
    let Ok((mut transform, mut facing)) = player_query.single_mut() else {
        return;
    };
    // The door panel on the inner face of the south wall.
    let door = DoorPanel {
        center_x: SHOP_ORIGIN.x + room.door_x,
        half_w: INTERIOR_DOOR_HALF_W,
        face_z: SHOP_ORIGIN.z + room.door_z,
        into_z: 1.0,
    };
    if !said_goodbye {
        if !door.faced_by(transform.translation, facing.dir, DOOR_RANGE_IN, 0.5)
            || !actions.consume(Action::Interact)
        {
            return;
        }
        // Facing the exit, the keeper turns and says goodbye first (not on the raffle day, when
        // the raffle script runs the conversations).
        if disc.as_ref().is_some_and(|d| d.text.is_some()) && !crate::raffle::raffle_day() {
            if twins.goodbye == Goodbye::Idle {
                twins.goodbye = Goodbye::Wanted;
            }
            return;
        }
    }
    twins.goodbye = Goodbye::Idle;
    transform.translation = return_pos.map(|r| r.0).unwrap_or(Vec3::new(0.0, 1.0, 5.0));
    facing.dir = Vec3::new(0.0, 0.0, 1.0);
    crate::sound::door_sound(Some(shop.shop.shop_level), false);
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
    room: Res<ShopRoom>,
    disc: Option<Res<crate::game_disc::GameDisc>>,
) {
    if disc.as_ref().is_some_and(|d| d.text.is_some())
        || *state != ShopLocation::Inside
        || ui.open
        || !focus.is_world()
    {
        return;
    }
    if !actions.just_pressed(Action::Interact) {
        return;
    }
    let Ok((transform, facing)) = player_query.single() else {
        return;
    };
    let nook = SHOP_ORIGIN + Vec3::new(0.0, 0.0, -4.1 * room.k);
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
    crate::sound::se(crate::sound::se_id::CHOICE_OPEN);
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
        crate::sound::se(crate::sound::se_id::PANEL_CLOSE);
        println!("Shop menu closed");
        return;
    }
    if actions.just_pressed(Action::Tab1) {
        let before = (ui.mode, ui.selected);
        ui.mode = ShopMode::Buy;
        ui.selected = 0;
        crate::sound::cursor_changed(before, (ui.mode, ui.selected));
        ui.message.clear();
        return;
    }
    if actions.just_pressed(Action::Tab2) {
        let before = (ui.mode, ui.selected);
        ui.mode = ShopMode::Sell;
        ui.selected = 0;
        crate::sound::cursor_changed(before, (ui.mode, ui.selected));
        ui.message.clear();
        return;
    }

    let list_len = match ui.mode {
        ShopMode::Buy => shop.for_sale().len(),
        ShopMode::Sell => sell_list(&inventory).len(),
    };
    if list_len == 0 {
        return;
    }
    if actions.just_pressed(Action::MenuUp) {
        let before = ui.selected;
        ui.selected = ui.selected.checked_sub(1).unwrap_or(list_len - 1);
        crate::sound::cursor_changed(before, ui.selected);
        return;
    }
    if actions.just_pressed(Action::MenuDown) {
        let before = ui.selected;
        ui.selected = (ui.selected + 1) % list_len;
        crate::sound::cursor_changed(before, ui.selected);
        return;
    }
    if !actions.just_pressed(Action::Confirm) {
        return;
    }

    match ui.mode {
        ShopMode::Buy => {
            let Some(index) = shop.for_sale().get(ui.selected).copied() else {
                return;
            };
            let item = shop.goods()[index].clone();
            if inventory.bells < item.price {
                ui.message = "Not enough bells!".to_string();
                crate::sound::se(crate::sound::se_id::MENU_EXIT);
                return;
            }
            let id = item.item;
            if id == 0 {
                ui.message = format!("{} isn't for sale.", item.name);
                crate::sound::se(crate::sound::se_id::MENU_EXIT);
                return;
            }
            if !inventory.add(id) {
                ui.message = "Your pockets are full!".to_string();
                crate::sound::se(crate::sound::se_id::MENU_EXIT);
                return;
            }
            inventory.bells -= item.price;
            crate::sound::se(crate::sound::se_id::REGISTER);
            // Each piece on display sells once (`aSD_ReportGoodsSales`).
            shop.goods_mut()[index].sold = true;
            ui.selected = ui.selected.min(shop.for_sale().len().saturating_sub(1));
            // Full price counts toward the shop upgrade (mSP_PlusSales).
            shop.shop.record_purchase(item.price);
            ui.message = format!("Bought {}!", item.name);
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
            crate::sound::se(crate::sound::se_id::REGISTER);
            // Half of Nook's payout counts toward the upgrade (mSP_PlusSales).
            shop.shop.record_sale(unit);
            ui.message = format!("Sold {name} for {unit} bells!");
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
                for (i, item) in shop
                    .for_sale()
                    .into_iter()
                    .map(|i| &shop.goods()[i])
                    .enumerate()
                {
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
    room: Res<ShopRoom>,
    layout: Res<layout::Layout>,
    staff: Query<&Transform, (With<crate::character::NpcModel>, Without<Player>)>,
) {
    if *state != ShopLocation::Inside || !focus.is_world() {
        return;
    }
    let Ok((mut transform, mut facing, mut loco)) = player_query.single_mut() else {
        return;
    };

    let movement = locomotion::step(&actions, time.delta().as_secs_f32(), &mut facing, &mut loco);
    // Match the existing Nook BG-check radius (eight game units), keeping
    // the player clear of the department-store attendants as well.
    let staff_blocks = |x: f32, z: f32, before: Vec3| {
        staff.iter().any(|npc| {
            let next = (npc.translation - Vec3::new(x, npc.translation.y, z)).length_squared();
            let old = (npc.translation - before).with_y(0.0).length_squared();
            next < (PLAYER_RADIUS + 0.2).powi(2) && next < old
        })
    };
    if movement != Vec3::ZERO {
        let nx = transform.translation.x + movement.x;
        if walkable(Vec3::new(nx, 0.0, transform.translation.z), &room, *layout)
            && !staff_blocks(nx, transform.translation.z, transform.translation)
            && !hits_obstacle(nx, transform.translation.z, &obstacles)
        {
            transform.translation.x = nx;
        }
        let nz = transform.translation.z + movement.z;
        if walkable(Vec3::new(transform.translation.x, 0.0, nz), &room, *layout)
            && !staff_blocks(transform.translation.x, nz, transform.translation)
            && !hits_obstacle(transform.translation.x, nz, &obstacles)
        {
            transform.translation.z = nz;
        }
    }
}

fn walkable(pos: Vec3, room: &ShopRoom, layout: layout::Layout) -> bool {
    let p = pos - SHOP_ORIGIN;
    if p.x.abs() <= room.walk_x && p.z.abs() <= room.walk_z {
        return true;
    }
    let native = p + layout.centre().with_y(0.0);
    // Department stairs and the native south door are narrower than the room.
    let stair = layout.tier == 3
        && (6.5 + PLAYER_RADIUS..=9.0 - PLAYER_RADIUS).contains(&native.x)
        && (1.0 + PLAYER_RADIUS..=3.5).contains(&native.z);
    let door_x = layout.centre().x + layout.door_x();
    let door_end = layout.centre().z + layout.door_z();
    stair
        || (!layout.upper
            && (door_x - 1.0 + PLAYER_RADIUS..=door_x + 1.0 - PLAYER_RADIUS).contains(&native.x)
            && (layout.centre().z + layout.half_z() - 0.625..=door_end - PLAYER_RADIUS)
                .contains(&native.z))
}

fn department_stairs(
    mut layout: ResMut<layout::Layout>,
    state: Res<State<ShopLocation>>,
    focus: Res<InputFocus>,
    disc: Option<Res<crate::game_disc::GameDisc>>,
    mut player: Query<(&mut Transform, &mut Facing, &mut Locomotion), With<Player>>,
    mut locked: Local<bool>,
    mut twins: ResMut<crate::shop_nook::Twins>,
) {
    use crate::shop_nook::Goodbye;
    if layout.tier != 3
        || crate::raffle::raffle_day()
        || *state != ShopLocation::Inside
        || !focus.is_world()
        || disc.is_none()
    {
        return;
    }
    let Ok((mut transform, mut facing, mut loco)) = player.single_mut() else {
        return;
    };
    let p = transform.translation - SHOP_ORIGIN + layout.centre().with_y(0.0);
    // The arrival is inside the opposite portal. Leave its landing before
    // rearming, so a held direction cannot immediately send the player back.
    if p.z > 2.5 {
        *locked = false;
    }
    if *locked {
        return;
    }
    let at_stairs = p.x >= 7.0
        && p.x <= 9.0
        && p.z < 2.0
        && facing.dir.z < -0.5
        && loco.state() != rustimal_logic::player_move::LocomotionState::Wait;
    // Leaving the upper floor, the selected twin says goodbye first.
    if layout.upper && disc.as_ref().is_some_and(|d| d.text.is_some()) {
        match twins.goodbye {
            Goodbye::Idle if at_stairs => {
                twins.goodbye = Goodbye::Wanted;
                *loco = Locomotion::default();
                return;
            }
            Goodbye::Wanted | Goodbye::Speaking => return,
            _ => {}
        }
    }
    // Only the upper floor's goodbye ends in the stairs; downstairs it ends in the door.
    let goodbye_done = layout.upper && twins.goodbye == Goodbye::Done;
    if at_stairs || goodbye_done {
        twins.goodbye = Goodbye::Idle;
        layout.upper = !layout.upper;
        *locked = true;
        transform.translation = SHOP_ORIGIN + layout.stair_arrival();
        facing.dir = Vec3::Z;
        *loco = Locomotion::default();
        crate::sound::door_sound(Some(ShopTier::Dsuper), true);
    }
}

fn hits_obstacle(x: f32, z: f32, obstacles: &ShopObstacles) -> bool {
    blocked_at(x, z, PLAYER_RADIUS, obstacles)
}

pub(crate) fn blocked_at(x: f32, z: f32, radius: f32, obstacles: &ShopObstacles) -> bool {
    obstacles.0.iter().any(|o| {
        x + radius > o.min_x && x - radius < o.max_x && z + radius > o.min_z && z - radius < o.max_z
    })
}

/// The disc shell contains its own tables, rack and pots. Use the existing
/// surface reader at 1/8-cell spacing, merging contiguous samples into row
/// bounds. Both actors collide with the visible furniture, even empty tables.
fn room_obstacles(disc: &crate::game_disc::GameDisc, layout: layout::Layout) -> Vec<Obstacle> {
    let mut names = vec![layout.floor(), layout.wall()];
    if crate::raffle::raffle_day() {
        names.push(layout.raffle_model());
    }
    let lists: Vec<u32> = names
        .iter()
        .filter_map(|n| disc.catalog.get(n).map(|a| a.offset))
        .collect();
    let surface = crate::disc_gfx::run_display_lists(&disc.rom, &lists);
    let mut out = Vec::new();
    let corner = SHOP_ORIGIN - layout.centre().with_y(0.0);
    let samples = (layout.centre().x * 16.0) as i32;
    for z in 0..(layout.centre().z * 16.0) as i32 {
        let mut start = None;
        for x in 0..=samples {
            let raised = x < samples
                && !(layout.tier == 3
                    && (7.0..9.0).contains(&((x as f32 + 0.5) * 0.125))
                    && (z as f32 + 0.5) * 0.125 < 3.5)
                && surface
                    .height_at((x as f32 + 0.5) * 80.0, (z as f32 + 0.5) * 80.0)
                    .is_some_and(|y| y / 640.0 > 1.2);
            if raised {
                start.get_or_insert(x);
            } else if let Some(first) = start.take() {
                out.push(Obstacle {
                    min_x: corner.x + first as f32 * 0.125,
                    max_x: corner.x + x as f32 * 0.125,
                    min_z: corner.z + z as f32 * 0.125,
                    max_z: corner.z + (z + 1) as f32 * 0.125,
                });
            }
        }
    }
    if crate::raffle::raffle_day() {
        if let Some((min, max)) = crate::raffle::obstacle_bounds(disc, layout) {
            info!("raffle drum collision: {min:?}..{max:?}");
            out.push(Obstacle {
                min_x: min.x,
                max_x: max.x,
                min_z: min.y,
                max_z: max.y,
            });
        }
    }
    out
}

/// Show/hide the "inside shop" indicator with the state.
fn update_shop_hud(
    state: Res<State<ShopLocation>>,
    mut query: Query<&mut Visibility, With<ShopHudText>>,
) {
    for mut vis in query.iter_mut() {
        *vis = match *state.get() {
            ShopLocation::Inside => Visibility::Visible,
            ShopLocation::Town
            | ShopLocation::PostOffice
            | ShopLocation::PoliceBox
            | ShopLocation::Museum => Visibility::Hidden,
        };
    }
}

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShopRoom>()
            .init_resource::<ShopObstacles>()
            .init_resource::<layout::Layout>()
            .add_sub_state::<ShopLocation>()
            .insert_resource(ShopData::default())
            .insert_resource(ShopUi::default())
            .add_systems(Startup, find_shop_spot.after(crate::town::generate_town))
            .add_systems(
                Startup,
                (
                    spawn_shop_exterior.after(find_shop_spot),
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
                (
                    schedule::tick,
                    restock_daily,
                    bargain::tick,
                    turnips::tick,
                    department_stairs,
                    build_shop_interior,
                )
                    .chain()
                    .after(crate::save::LoadSet)
                    .after(locomotion::MovementSet)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                (
                    refresh_shop_ui,
                    shop_interior_movement.in_set(locomotion::MovementSet),
                    update_shop_hud,
                )
                    .in_set(GameplaySet),
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
            trigger: None,
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
