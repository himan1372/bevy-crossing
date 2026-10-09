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
use rustimal_logic::furniture::FurnitureSize;
use rustimal_logic::town_gen::{CellKind, Feature, ACRE_DEPTH, ACRE_WIDTH};

use crate::dialogue::DialogueState;
use crate::ecology::EquippedTool;
use crate::interaction::Facing;
use crate::player::Player;
use crate::town::Town;

/// Town map vs. house interior.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HouseState {
    #[default]
    Town,
    Inside,
}

/// System set for the house-door E handler. The town `interact` system runs
/// after this set so that on a successful enter (which teleports the player
/// far away) the talk/pickup query finds nothing that same frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HouseInputSet;

/// World position of the player's house door (exterior), in town coordinates.
#[derive(Resource)]
pub struct PlayerHouse {
    pub door_pos: Vec3,
}

/// Town-map position to return to when leaving the house.
#[derive(Resource, Clone, Copy)]
pub struct ReturnPosition(pub Vec3);

/// Interior furniture collision boxes, in world coordinates.
#[derive(Resource, Default)]
struct InteriorObstacles(Vec<Obstacle>);

struct Obstacle {
    min_x: f32,
    max_x: f32,
    min_z: f32,
    max_z: f32,
}

/// Marker for the "inside house" HUD indicator.
#[derive(Component)]
struct HouseHudText;

/// Far-away origin where the interior room is built (well clear of the town).
const INTERIOR_ORIGIN: Vec3 = Vec3::new(3000.0, 0.0, 3000.0);
/// How far the player may walk from the room center.
const WALK_HALF: f32 = 3.3;
const PLAYER_RADIUS: f32 = 0.4;
const PLAYER_SPEED: f32 = 5.0;
const DOOR_RANGE_OUT: f32 = 3.2;
const DOOR_RANGE_IN: f32 = 2.6;
const DOOR_DOT: f32 = 0.6;

/// Find the player's house from the town plan and record its door position.
fn find_player_house(town: Res<Town>, mut commands: Commands) {
    let mut door = None;
    'acres: for az in 0..ACRE_DEPTH {
        for ax in 0..ACRE_WIDTH {
            let acre = &town.plan.acres[az * ACRE_WIDTH + ax];
            if acre.feature != Feature::PlayerHouse as u8 {
                continue;
            }
            for cz in 0..16 {
                for cx in 0..16 {
                    if acre.cells[cz * 16 + cx] == CellKind::House as u8 {
                        let p = Town::cell_to_world(ax, az, cx, cz);
                        // Door on the south face of the 3-wide house box.
                        door = Some(p + Vec3::new(0.0, 0.0, 2.3));
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
                            door = Some(p + Vec3::new(0.0, 0.0, 2.3));
                            break 'acres;
                        }
                    }
                }
            }
        }
    }
    let door_pos = door.unwrap_or(Vec3::new(0.0, 0.0, 5.0));
    commands.insert_resource(PlayerHouse { door_pos });
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
fn build_interior(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ox = INTERIOR_ORIGIN.x;
    let oz = INTERIOR_ORIGIN.z;

    let floor_mat = materials.add(Color::srgb(0.55, 0.38, 0.22));
    let wall_mat = materials.add(Color::srgb(0.85, 0.78, 0.65));
    let door_mat = materials.add(Color::srgb(0.4, 0.25, 0.12));
    let rug_mat = materials.add(Color::srgb(0.7, 0.25, 0.25));

    // Floor.
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(8.8, 8.8)),
        material: floor_mat,
        transform: Transform::from_xyz(ox, 0.02, oz),
        ..default()
    });

    // Rug.
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(2.6, 2.6)),
        material: rug_mat,
        transform: Transform::from_xyz(ox, 0.04, oz - 0.5),
        ..default()
    });

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
        transform: Transform::from_xyz(ox, 1.2, oz + 3.85),
        ..default()
    });

    // Furniture: (placeholder name, size class, local x, local z, height, color).
    // Kept clear of the door path (x in [-1.2, 1.2], z > 1.0).
    let furniture: [(&str, FurnitureSize, f32, f32, f32, (f32, f32, f32)); 4] = [
        ("Oak Table", FurnitureSize::Size2x2, -1.9, -1.4, 0.75, (0.5, 0.33, 0.18)),
        ("Red Chair", FurnitureSize::Size1x1, -1.9, 0.3, 0.7, (0.75, 0.2, 0.2)),
        ("Blue Bed", FurnitureSize::Size1x2, 2.3, -2.2, 0.5, (0.25, 0.4, 0.8)),
        ("Floor Lamp", FurnitureSize::Size1x1, 2.6, 1.6, 1.5, (0.9, 0.8, 0.4)),
    ];
    let mut obstacles = Vec::new();
    for (name, size, lx, lz, h, (r, g, b)) in furniture {
        let (fw, fd) = furniture_footprint(size);
        let mat = materials.add(Color::srgb(r, g, b));
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(fw, h, fd)),
            material: mat,
            transform: Transform::from_xyz(ox + lx, h / 2.0, oz + lz),
            ..default()
        });
        obstacles.push(Obstacle {
            min_x: ox + lx - fw / 2.0,
            max_x: ox + lx + fw / 2.0,
            min_z: oz + lz - fd / 2.0,
            max_z: oz + lz + fd / 2.0,
        });
        println!("Placed furniture: {name}");
    }
    commands.insert_resource(InteriorObstacles(obstacles));

    // Persistent "inside" indicator (hidden until the player enters).
    commands.spawn((
        HouseHudText,
        TextBundle {
            text: Text::from_section(
                "House — press E at the door to exit",
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

    println!("House interior built at {:?}", INTERIOR_ORIGIN);
}

/// E at the exterior house door: teleport into the interior.
///
/// Runs in [`HouseInputSet`], before the town `interact` system, so a
/// successful enter (which moves the player far away) means the talk/pickup
/// query finds nothing that same frame.
fn house_enter(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<HouseState>>,
    dialogue: Res<DialogueState>,
    tool: Res<EquippedTool>,
    house: Option<Res<PlayerHouse>>,
    mut next_state: ResMut<NextState<HouseState>>,
    mut commands: Commands,
    mut player_query: Query<(&mut Transform, &mut Facing), With<Player>>,
) {
    if *state != HouseState::Town {
        return;
    }
    if !keyboard.just_pressed(KeyCode::KeyE) || dialogue.active {
        return;
    }
    // A held tool claims E for the ecology plugin; the door needs empty hands.
    if *tool != EquippedTool::None {
        return;
    }
    let Some(house) = house else { return };
    let Ok((mut transform, mut facing)) = player_query.get_single_mut() else {
        return;
    };
    let to_door = house.door_pos - transform.translation;
    let dist = to_door.length();
    if dist > DOOR_RANGE_OUT || dist < 0.001 {
        return;
    }
    if facing.dir.dot(to_door / dist) < DOOR_DOT {
        return;
    }
    commands.insert_resource(ReturnPosition(transform.translation));
    transform.translation = INTERIOR_ORIGIN + Vec3::new(0.0, 1.0, 2.6);
    facing.dir = Vec3::new(0.0, 0.0, -1.0);
    facing.last_pos = transform.translation;
    next_state.set(HouseState::Inside);
    println!("Entered house");
}

/// E at the interior door: return to the town map.
fn house_exit(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<HouseState>>,
    mut next_state: ResMut<NextState<HouseState>>,
    return_pos: Option<Res<ReturnPosition>>,
    mut player_query: Query<(&mut Transform, &mut Facing), With<Player>>,
) {
    if *state != HouseState::Inside {
        return;
    }
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok((mut transform, mut facing)) = player_query.get_single_mut() else {
        return;
    };
    let door = INTERIOR_ORIGIN + Vec3::new(0.0, 0.0, 3.0);
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
    next_state.set(HouseState::Town);
    println!("Left house");
}

/// WASD movement inside the room: room bounds plus furniture collision.
fn interior_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<HouseState>>,
    mut player_query: Query<(&mut Transform, &mut Facing), With<Player>>,
    obstacles: Res<InteriorObstacles>,
    time: Res<Time>,
) {
    if *state != HouseState::Inside {
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
        if (nx - INTERIOR_ORIGIN.x).abs() <= WALK_HALF
            && !hits_obstacle(nx, transform.translation.z, &obstacles)
        {
            transform.translation.x = nx;
        }
        let nz = transform.translation.z + movement.z;
        if (nz - INTERIOR_ORIGIN.z).abs() <= WALK_HALF
            && !hits_obstacle(transform.translation.x, nz, &obstacles)
        {
            transform.translation.z = nz;
        }
        facing.dir = Vec3::new(direction.x, 0.0, direction.z);
        facing.last_pos = transform.translation;
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
    mut query: Query<&mut Visibility, With<HouseHudText>>,
) {
    for mut vis in query.iter_mut() {
        *vis = match *state.get() {
            HouseState::Inside => Visibility::Visible,
            HouseState::Town => Visibility::Hidden,
        };
    }
}

pub struct HousePlugin;

impl Plugin for HousePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<HouseState>()
            .add_systems(Startup, find_player_house.after(crate::town::generate_town))
            .add_systems(Startup, build_interior)
            .add_systems(Update, house_enter.in_set(HouseInputSet))
            .add_systems(Update, house_exit)
            .add_systems(Update, interior_movement)
            .add_systems(Update, update_house_hud);
    }
}
