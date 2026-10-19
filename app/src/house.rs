//! Player house interior (Phase 3).
//!
//! The player's house door is located from the town plan (the acre with
//! `Feature::PlayerHouse`). Pressing E while facing the door teleports the
//! player into a simple 8x8 wooden interior built once at a distant world
//! offset; pressing E at the interior door returns to town. [`HouseState`]
//! gates the town interaction systems so E is unambiguous in each place.
//!
//! Logic-crate usage: [`rustimal_logic::furniture::FurnitureSize`] drives the
//! furniture footprints. Furniture display names are placeholders — the
//! decomp data only carries numeric furniture IDs, no string table.

use bevy::prelude::*;
use bevy::text::FontSize;
use rustimal_logic::furniture::FurnitureSize;
use rustimal_logic::town_gen::{CellKind, Feature, ACRE_DEPTH, ACRE_WIDTH};

use crate::ecology::EquippedTool;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{DoorPanel, Facing};
use crate::locomotion::{self, Locomotion};
use crate::player::Player;
use crate::states::{AppState, GameplaySet};
use crate::town::{Town, HOUSE_SIZE};

/// Town map vs. house interior. Lives only while in game.
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::InGame)]
pub enum HouseState {
    #[default]
    Town,
    Inside,
}

/// World position of the player's house door (exterior), in town coordinates.
#[derive(Resource)]
pub struct PlayerHouse {
    pub door_pos: Vec3,
    /// Which way the door faces, as a yaw from south (+Z): 0 = south,
    /// +90 deg = east. Retail player houses face the acre path.
    pub door_yaw: f32,
}

/// Door yaw for a player house at plan cell column `cx` of its acre: with
/// the retail town, houses face the path through the acre centre (east
/// from the left half, west from the right); otherwise south.
pub fn player_house_door_yaw(retail: bool, cx: usize) -> f32 {
    match (retail, cx < 8) {
        (false, _) => 0.0,
        (true, true) => std::f32::consts::FRAC_PI_2,
        (true, false) => -std::f32::consts::FRAC_PI_2,
    }
}

/// Town-map position to return to when leaving the house.
#[derive(Resource, Clone, Copy)]
pub struct ReturnPosition(pub Vec3);

/// Interior furniture collision boxes, in world coordinates.
#[derive(Resource, Default)]
pub(crate) struct InteriorObstacles(Vec<Obstacle>);

struct Obstacle {
    player_room: bool,
    min_x: f32,
    max_x: f32,
    min_z: f32,
    max_z: f32,
}

impl Obstacle {
    /// A room furniture box, in a room at `origin`.
    fn around(origin: Vec3, f: &crate::interior::RoomFurniture) -> Self {
        Self {
            player_room: origin == INTERIOR_ORIGIN,
            min_x: origin.x + f.centre.x - f.half.x,
            max_x: origin.x + f.centre.x + f.half.x,
            min_z: origin.z + f.centre.z - f.half.y,
            max_z: origin.z + f.centre.z + f.half.y,
        }
    }
}

impl InteriorObstacles {
    pub(crate) fn replace_player(&mut self, furniture: &[crate::interior::RoomFurniture]) {
        self.0.retain(|o| !o.player_room);
        self.0.extend(
            furniture
                .iter()
                .map(|f| Obstacle::around(INTERIOR_ORIGIN, f)),
        );
    }
}

/// Marker for the "inside house" HUD indicator.
#[derive(Component)]
struct HouseHudText;

/// Far-away origin where the interior room is built (well clear of the town).
pub(crate) const INTERIOR_ORIGIN: Vec3 = Vec3::new(3000.0, 0.0, 3000.0);
/// Villager rooms sit beside the player's, this far apart in x.
const VILLAGER_ROOM_SPACING: f32 = 100.0;

/// The room the player is in (or last entered): its origin and bounds.
#[derive(Resource, Clone, Copy)]
pub(crate) struct CurrentRoom(Vec3, RoomBounds);

pub(crate) fn is_player_room(room: &CurrentRoom, state: &State<HouseState>) -> bool {
    *state.get() == HouseState::Inside && room.0 == INTERIOR_ORIGIN
}

impl Default for CurrentRoom {
    fn default() -> Self {
        Self(INTERIOR_ORIGIN, RoomBounds::default())
    }
}

/// Villager front doors and the rooms behind them.
#[derive(Resource, Default)]
struct VillagerDoors(Vec<(Vec3, Vec3)>);

/// Startup, after the villagers: builds each villager's room and records
/// their door.
#[allow(clippy::too_many_arguments)]
fn build_villager_rooms(
    mut commands: Commands,
    disc: Option<Res<crate::game_disc::GameDisc>>,
    villagers: Query<&crate::npc::Villager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut n64: ResMut<Assets<crate::n64_material::N64Material>>,
    mut images: ResMut<Assets<Image>>,
    mut obstacles: ResMut<InteriorObstacles>,
) {
    let Some(disc) = disc else { return };
    let mut doors = Vec::new();
    for (i, v) in villagers.iter().enumerate() {
        let Some(idx) = v.npc_index else { continue };
        let origin = INTERIOR_ORIGIN + Vec3::X * VILLAGER_ROOM_SPACING * (i + 1) as f32;
        let Some(furniture) = crate::interior::spawn_villager_room(
            &mut commands,
            &disc,
            idx,
            origin,
            &mut meshes,
            &mut n64,
            &mut images,
        ) else {
            continue;
        };
        obstacles
            .0
            .extend(furniture.iter().map(|f| Obstacle::around(origin, f)));
        doors.push((v.door(), origin));
    }
    info!("{} villager rooms from the disc", doors.len());
    commands.insert_resource(VillagerDoors(doors));
}
/// How far the player may walk from the room center.
const WALK_HALF: f32 = 3.3;
const PLAYER_RADIUS: f32 = 0.4;
const DOOR_RANGE_OUT: f32 = 3.2;
/// Player-home entry shares the doorstep with a mailbox and gyroid.
/// Reach only the nearby panel we are actually aiming at.
const PLAYER_DOOR_REACH: f32 = 1.25;
const DOOR_RANGE_IN: f32 = 2.6;
/// Interior door z offset from the origin: inner face of the south wall
/// (wall centre +4.0, thickness 0.4).
const INTERIOR_DOOR_Z: f32 = 3.8;
/// Half the interior door panel's width (panel is 1.4 wide).
const INTERIOR_DOOR_HALF_W: f32 = 0.7;
const DOOR_DOT: f32 = 0.6;
/// `PlayerHouse::door_pos` sits this far in front of the house's front wall.
const DOOR_OUT: f32 = 0.8;
/// Half the exterior door's width.
const DOOR_HALF_W: f32 = 0.6;

/// Find the player's house from the town plan and record its door position.
pub(crate) fn find_player_house(town: Res<Town>, mut commands: Commands) {
    let mut door = None;
    // Retail town: the player's house is HOUSE0, placed as the game does.
    if let Some((cell, body)) = town
        .retail
        .as_ref()
        .and_then(|r| Some((r.player_homes[0]?, r.player_home_bodies[0].as_ref()?)))
    {
        let (_, _, yaw) = crate::disc_town::player_home_placement(0, cell);
        let out = Quat::from_rotation_y(yaw) * Vec3::Z * DOOR_OUT;
        door = Some((body.door + out, yaw));
    }
    'acres: for az in 0..ACRE_DEPTH {
        if door.is_some() {
            break;
        }
        for ax in 0..ACRE_WIDTH {
            let acre = &town.plan.acres[az * ACRE_WIDTH + ax];
            if acre.feature != Feature::PlayerHouse as u8 {
                continue;
            }
            for cz in 0..16 {
                for cx in 0..16 {
                    if acre.cells[cz * 16 + cx] == CellKind::House as u8 {
                        let p = Town::cell_to_world(ax, az, cx, cz);
                        // Door on the house's front face, which way it faces.
                        let yaw = player_house_door_yaw(town.retail.is_some(), cx);
                        let out = Quat::from_rotation_y(yaw)
                            * Vec3::new(0.0, 0.0, HOUSE_SIZE / 2.0 + DOOR_OUT);
                        door = Some((p + out, yaw));
                        break 'acres;
                    }
                }
            }
        }
    }
    // Fallback: first generic house cell, so the door always exists.
    if door.is_none() {
        'acres: for az in 0..ACRE_DEPTH {
            for ax in 0..ACRE_WIDTH {
                let acre = &town.plan.acres[az * ACRE_WIDTH + ax];
                for cz in 0..16 {
                    for cx in 0..16 {
                        if acre.cells[cz * 16 + cx] == CellKind::House as u8 {
                            let p = Town::cell_to_world(ax, az, cx, cz);
                            door =
                                Some((p + Vec3::new(0.0, 0.0, HOUSE_SIZE / 2.0 + DOOR_OUT), 0.0));
                            break 'acres;
                        }
                    }
                }
            }
        }
    }
    let (door_pos, door_yaw) = door.unwrap_or((Vec3::new(0.0, 0.0, 5.0), 0.0));
    commands.insert_resource(PlayerHouse { door_pos, door_yaw });
    println!("Player house door at {:?}", door_pos);
}

/// Box footprint (x, z) in world units for a furniture size class.
fn furniture_footprint(size: FurnitureSize) -> (f32, f32) {
    match size {
        FurnitureSize::Size1x1 => (0.9, 0.9),
        FurnitureSize::Size1x2 => (0.9, 1.8),
        FurnitureSize::Size2x2 => (1.8, 1.8),
    }
}

/// Build the interior room once at the distant origin.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_interior(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    disc: Option<Res<crate::game_disc::GameDisc>>,
    mut n64: ResMut<Assets<crate::n64_material::N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    // The disc's room: its walls, carpet, wallpaper and starting furniture.
    if let Some(furniture) = disc.as_deref().and_then(|d| {
        crate::interior::spawn_player_room(
            &mut commands,
            d,
            INTERIOR_ORIGIN,
            &mut meshes,
            &mut n64,
            &mut images,
        )
    }) {
        let obstacles = furniture
            .iter()
            .map(|f| Obstacle::around(INTERIOR_ORIGIN, f))
            .collect();
        commands.insert_resource(InteriorObstacles(obstacles));
        commands.insert_resource(RoomBounds::disc_room());
        spawn_house_hud(&mut commands);
        return;
    }
    let ox = INTERIOR_ORIGIN.x;
    let oz = INTERIOR_ORIGIN.z;

    let floor_mat = materials.add(Color::srgb(0.55, 0.38, 0.22));
    let wall_mat = materials.add(Color::srgb(0.85, 0.78, 0.65));
    let door_mat = materials.add(Color::srgb(0.4, 0.25, 0.12));
    let rug_mat = materials.add(Color::srgb(0.7, 0.25, 0.25));

    // Floor.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(8.8, 8.8))),
        MeshMaterial3d(floor_mat),
        Transform::from_xyz(ox, 0.02, oz),
    ));

    // Rug.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(2.6, 2.6))),
        MeshMaterial3d(rug_mat),
        Transform::from_xyz(ox, 0.04, oz - 0.5),
    ));

    // Four walls (no ceiling; the camera looks in from above at an angle).
    let wall_h = 3.0;
    let wall_t = 0.4;
    let walls = [
        // (center_x, center_z, size_x, size_z)
        (ox, oz - 4.0, 8.8, wall_t), // north
        (ox, oz + 4.0, 8.8, wall_t), // south
        (ox - 4.0, oz, wall_t, 8.8), // west
        (ox + 4.0, oz, wall_t, 8.8), // east
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
        Transform::from_xyz(ox, 1.2, oz + 3.85),
    ));

    // Furniture: (placeholder name, size class, local x, local z, height, color).
    // Kept clear of the door path (x in [-1.2, 1.2], z > 1.0).
    let furniture: [(&str, FurnitureSize, f32, f32, f32, (f32, f32, f32)); 4] = [
        (
            "Oak Table",
            FurnitureSize::Size2x2,
            -1.9,
            -1.4,
            0.75,
            (0.5, 0.33, 0.18),
        ),
        (
            "Red Chair",
            FurnitureSize::Size1x1,
            -1.9,
            0.3,
            0.7,
            (0.75, 0.2, 0.2),
        ),
        (
            "Blue Bed",
            FurnitureSize::Size1x2,
            2.3,
            -2.2,
            0.5,
            (0.25, 0.4, 0.8),
        ),
        (
            "Floor Lamp",
            FurnitureSize::Size1x1,
            2.6,
            1.6,
            1.5,
            (0.9, 0.8, 0.4),
        ),
    ];
    let mut obstacles = Vec::new();
    for (name, size, lx, lz, h, (r, g, b)) in furniture {
        let (fw, fd) = furniture_footprint(size);
        let mat = materials.add(Color::srgb(r, g, b));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(fw, h, fd))),
            MeshMaterial3d(mat),
            Transform::from_xyz(ox + lx, h / 2.0, oz + lz),
        ));
        obstacles.push(Obstacle {
            player_room: true,
            min_x: ox + lx - fw / 2.0,
            max_x: ox + lx + fw / 2.0,
            min_z: oz + lz - fd / 2.0,
            max_z: oz + lz + fd / 2.0,
        });
        println!("Placed furniture: {name}");
    }
    commands.insert_resource(InteriorObstacles(obstacles));

    spawn_house_hud(&mut commands);

    println!("House interior built at {:?}", INTERIOR_ORIGIN);
}

/// The "inside the house" HUD line.
fn spawn_house_hud(commands: &mut Commands) {
    // Persistent "inside" indicator (hidden until the player enters).
    commands.spawn((
        HouseHudText,
        Text::new("House — press E at the door to exit"),
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
}

/// Where the player may walk inside and where the door is, relative to
/// the interior origin: the placeholder room, or the disc's smaller room.
#[derive(Resource, Clone, Copy)]
struct RoomBounds {
    walk_half: f32,
    door_z: f32,
    spawn_z: f32,
    entrance: Option<Entrance>,
}

#[derive(Clone, Copy)]
struct Entrance {
    half_width: f32,
    start_z: f32,
    end_z: f32,
}

impl Default for RoomBounds {
    fn default() -> Self {
        Self {
            walk_half: WALK_HALF,
            door_z: INTERIOR_DOOR_Z,
            spawn_z: 2.6,
            // The no-disc room has a wooden floor up to its south door.
            entrance: Some(Entrance {
                half_width: INTERIOR_DOOR_HALF_W,
                start_z: 2.6,
                end_z: INTERIOR_DOOR_Z,
            }),
        }
    }
}

impl RoomBounds {
    /// The disc's 4x4-cell room and its wooden south entrance landing.
    fn disc_room() -> Self {
        Self {
            entrance: Some(Entrance {
                half_width: crate::interior::ENTRANCE_HALF_WIDTH,
                start_z: crate::interior::ROOM_HALF,
                end_z: crate::interior::ENTRANCE_END_Z,
            }),
            door_z: crate::interior::ENTRANCE_END_Z,
            ..Self::square(crate::interior::ROOM_HALF)
        }
    }

    /// A square room `half` cells across each way, door mid-south.
    fn square(half: f32) -> Self {
        Self {
            walk_half: half - PLAYER_RADIUS,
            door_z: half,
            spawn_z: half - 0.6,
            entrance: None,
        }
    }

    fn walkable(&self, local: Vec3, radius: f32) -> bool {
        let half = self.walk_half + PLAYER_RADIUS - radius;
        (local.x.abs() <= half && local.z.abs() <= half)
            || self.entrance.is_some_and(|e| {
                local.x.abs() <= e.half_width - radius
                    && local.z >= half
                    && local.z <= e.end_z - radius
            })
    }

    fn on_entrance(&self, local: Vec3) -> bool {
        self.entrance.is_some_and(|e| {
            local.x.abs() <= e.half_width - PLAYER_RADIUS
                && local.z >= e.start_z
                && local.z <= e.end_z - PLAYER_RADIUS
        })
    }
}

/// Furniture push/pull uses the same player-room floor/landing boundary, with
/// its existing small contact tolerance supplied as the radius by its caller.
pub(crate) fn player_room_walkable(pos: Vec3, radius: f32) -> bool {
    RoomBounds::disc_room().walkable(pos - INTERIOR_ORIGIN, radius)
}

/// E at the exterior house door: teleport into the interior.
///
/// Runs in [`InteractSet::Doors`]: a successful enter consumes the press,
/// so talk/pickup can't also fire on it.
fn house_enter(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<HouseState>>,
    shop_loc: Res<State<crate::shop::ShopLocation>>,
    tool: Res<EquippedTool>,
    house: Option<Res<PlayerHouse>>,
    mut next_state: ResMut<NextState<HouseState>>,
    mut commands: Commands,
    mut player_query: Query<(&mut Transform, &mut Facing), With<Player>>,
    bounds: Res<RoomBounds>,
    villager_doors: Res<VillagerDoors>,
) {
    if *state != HouseState::Town || *shop_loc != crate::shop::ShopLocation::Town {
        return;
    }
    if !actions.just_pressed(Action::Interact) || !focus.is_world() {
        return;
    }
    // A held tool claims E for the ecology plugin; the door needs empty hands.
    if *tool != EquippedTool::None {
        return;
    }
    let Ok((mut transform, mut facing)) = player_query.single_mut() else {
        return;
    };
    // A villager's front door (they face south, `aHUS` rotation 0).
    let villager_room = villager_doors.0.iter().find_map(|&(door_pos, origin)| {
        let door = DoorPanel {
            center_x: door_pos.x,
            half_w: DOOR_HALF_W,
            face_z: door_pos.z - 0.5,
            into_z: -1.0,
        };
        door.faced_by(transform.translation, facing.dir, DOOR_RANGE_OUT, DOOR_DOT)
            .then_some(origin)
    });
    if let Some(origin) = villager_room {
        if !actions.consume(Action::Interact) {
            return;
        }
        commands.insert_resource(ReturnPosition(transform.translation));
        let room = RoomBounds::square(crate::interior::VILLAGER_ROOM_HALF);
        commands.insert_resource(CurrentRoom(origin, room));
        transform.translation = origin + Vec3::new(0.0, 1.0, room.spawn_z);
        facing.dir = Vec3::new(0.0, 0.0, -1.0);
        crate::sound::door_sound(None, true);
        next_state.set(HouseState::Inside);
        println!("Entered a villager's house");
        return;
    }
    let Some(house) = house else { return };
    // The door panel on the house's front wall. The panel is south-facing,
    // so check in the door's frame (turned so the door faces south).
    let door = DoorPanel {
        center_x: house.door_pos.x,
        half_w: DOOR_HALF_W,
        face_z: house.door_pos.z - DOOR_OUT,
        into_z: -1.0,
    };
    let to_local = Quat::from_rotation_y(-house.door_yaw);
    let local_pos = house.door_pos + to_local * (transform.translation - house.door_pos);
    if !door.aimed_at(local_pos, to_local * facing.dir, PLAYER_DOOR_REACH)
        || !actions.consume(Action::Interact)
    {
        return;
    }
    commands.insert_resource(ReturnPosition(transform.translation));
    commands.insert_resource(CurrentRoom(INTERIOR_ORIGIN, *bounds));
    transform.translation = INTERIOR_ORIGIN + Vec3::new(0.0, 1.0, bounds.spawn_z);
    facing.dir = Vec3::new(0.0, 0.0, -1.0);
    crate::sound::door_sound(None, true);
    next_state.set(HouseState::Inside);
    println!("Entered house");
}

/// E at the interior door: return to the town map.
fn house_exit(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<HouseState>>,
    mut next_state: ResMut<NextState<HouseState>>,
    return_pos: Option<Res<ReturnPosition>>,
    mut player_query: Query<(&mut Transform, &mut Facing), With<Player>>,
    room: Res<CurrentRoom>,
) {
    let bounds = room.1;
    if *state != HouseState::Inside {
        return;
    }
    if !actions.just_pressed(Action::Interact) || !focus.is_world() {
        return;
    }
    let Ok((mut transform, mut facing)) = player_query.single_mut() else {
        return;
    };
    // The door panel on the inner face of the south wall. (The old target,
    // +3.0 at floor level, sat inside the walk bound and below the player.)
    let door = DoorPanel {
        center_x: room.0.x,
        half_w: INTERIOR_DOOR_HALF_W,
        face_z: room.0.z + bounds.door_z,
        into_z: 1.0,
    };
    let can_exit = if bounds.entrance.is_some() {
        bounds.on_entrance(transform.translation - room.0)
            && facing.dir.normalize_or_zero().z >= 0.9
            && door.aimed_at(transform.translation, facing.dir, 2.25)
    } else {
        door.faced_by(transform.translation, facing.dir, DOOR_RANGE_IN, 0.5)
    };
    if !can_exit || !actions.consume(Action::Interact) {
        return;
    }
    transform.translation = return_pos.map(|r| r.0).unwrap_or(Vec3::new(0.0, 1.0, 5.0));
    facing.dir = Vec3::new(0.0, 0.0, 1.0);
    crate::sound::door_sound(None, false);
    next_state.set(HouseState::Town);
    println!("Left house");
}

/// WASD movement inside the room: room bounds plus furniture collision.
fn interior_movement(
    actions: Res<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<HouseState>>,
    mut player_query: Query<(&mut Transform, &mut Facing, &mut Locomotion), With<Player>>,
    obstacles: Res<InteriorObstacles>,
    time: Res<Time>,
    room: Res<CurrentRoom>,
    toys: Res<crate::player_toys::ToyAction>,
) {
    let bounds = room.1;
    if *state != HouseState::Inside || !focus.is_world() {
        return;
    }
    let Ok((mut transform, mut facing, mut loco)) = player_query.single_mut() else {
        return;
    };

    if toys.blocks_movement() {
        *loco = Locomotion::default();
        return;
    }
    let movement = locomotion::step(&actions, time.delta().as_secs_f32(), &mut facing, &mut loco);
    if movement != Vec3::ZERO {
        let nx = transform.translation.x + movement.x;
        if bounds.walkable(
            Vec3::new(nx - room.0.x, 0.0, transform.translation.z - room.0.z),
            PLAYER_RADIUS,
        ) && !hits_obstacle(nx, transform.translation.z, &obstacles)
        {
            transform.translation.x = nx;
        }
        let nz = transform.translation.z + movement.z;
        if bounds.walkable(
            Vec3::new(transform.translation.x - room.0.x, 0.0, nz - room.0.z),
            PLAYER_RADIUS,
        ) && !hits_obstacle(transform.translation.x, nz, &obstacles)
        {
            transform.translation.z = nz;
        }
    }
}

fn hits_obstacle(x: f32, z: f32, obstacles: &InteriorObstacles) -> bool {
    obstacles.0.iter().any(|o| {
        x + PLAYER_RADIUS > o.min_x
            && x - PLAYER_RADIUS < o.max_x
            && z + PLAYER_RADIUS > o.min_z
            && z - PLAYER_RADIUS < o.max_z
    })
}

/// Show/hide the "inside house" indicator with the state.
fn update_house_hud(
    state: Res<State<HouseState>>,
    room: Res<crate::player_room::PlayerRoom>,
    mut query: Query<(&mut Visibility, &mut Text), With<HouseHudText>>,
) {
    for (mut vis, mut text) in query.iter_mut() {
        *vis = match *state.get() {
            HouseState::Inside => Visibility::Visible,
            HouseState::Town => Visibility::Hidden,
        };
        let hint = if room.available {
            "House - I: place | Backspace: pick up | E: furniture | G + directions: move / turn\nPlaytest - F1: toys | F2: song | F3: drawers | F4: cabinet"
        } else {
            "House — press E at the door to exit"
        };
        if text.0 != hint {
            text.0 = hint.into();
        }
    }
}

pub struct HousePlugin;

impl Plugin for HousePlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<HouseState>()
            .add_systems(Startup, find_player_house.after(crate::town::generate_town))
            .init_resource::<RoomBounds>()
            .init_resource::<CurrentRoom>()
            .init_resource::<InteriorObstacles>()
            .init_resource::<VillagerDoors>()
            .add_systems(
                Startup,
                build_villager_rooms
                    .after(crate::npc::spawn_villagers)
                    .after(build_interior),
            )
            .add_systems(
                Startup,
                build_interior.after(crate::game_disc::start_disc_load),
            )
            .add_systems(
                Update,
                (house_enter, house_exit)
                    .in_set(InteractSet::Doors)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                (
                    interior_movement.in_set(locomotion::MovementSet),
                    update_house_hud,
                )
                    .in_set(GameplaySet),
            );
    }
}
