use crate::house::HouseState;
use crate::input::{Actions, InputFocus};
use crate::interaction::Facing;
use crate::locomotion::{self, Locomotion};
use crate::states::GameplaySet;
use crate::town::Town;
use bevy::prelude::*;

/// Player marker component.
#[derive(Component)]
pub struct Player;

/// The camera following gameplay, separate from menu render-to-texture views.
#[derive(Component)]
pub(crate) struct WorldCamera;

/// Spawn the player and camera.
pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_mesh = meshes.add(Capsule3d::new(0.4, 1.0));
    let player_mat = materials.add(Color::srgb(0.2, 0.4, 0.9));

    commands.spawn((
        Player,
        Facing::default(),
        Locomotion::default(),
        // The hand-over arm clips play on the player too.
        crate::character::NpcClip::default(),
        Mesh3d(player_mesh),
        MeshMaterial3d(player_mat),
        Transform::from_xyz(0.0, 1.0, 0.0),
    ));

    // Spawn camera
    commands.spawn((
        WorldCamera,
        Name::new("world camera"),
        Camera3d::default(),
        // The GameCube shows its colours as they are: no tonemapping.
        bevy::core_pipeline::tonemapping::Tonemapping::None,
        Projection::Perspective(PerspectiveProjection {
            fov: CAMERA_FOV_Y_DEG.to_radians(),
            near: CAMERA_NEAR,
            far: CAMERA_FAR,
            ..default()
        }),
        Transform::from_translation(camera_eye_offset()).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    println!("Player and camera spawned");
}

/// Handle WASD movement with collision. Stands down inside the house
/// or shop (see `house::interior_movement` / `shop::shop_interior_movement`),
/// and whenever a UI has input focus.
pub fn player_movement(
    actions: Res<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<HouseState>>,
    shop_loc: Res<State<crate::shop::ShopLocation>>,
    shop_building: Option<Res<crate::shop::ShopBuilding>>,
    mut player_query: Query<(&mut Transform, &mut Facing, &mut Locomotion), With<Player>>,
    town: Res<Town>,
    net: Res<crate::player_tools::NetAction>,
    toys: Res<crate::player_toys::ToyAction>,
    time: Res<Time>,
    (statue, copper): (
        Option<Res<crate::statue::StatueUnit>>,
        Option<Res<crate::copper::CopperUnit>>,
    ),
) {
    if *state != HouseState::Town || *shop_loc != crate::shop::ShopLocation::Town {
        return;
    }
    if !focus.is_world() {
        return;
    }
    let Ok((mut transform, mut facing, mut loco)) = player_query.single_mut() else {
        return;
    };
    if net.busy() || toys.blocks_movement() {
        *loco = Locomotion::default();
        return;
    }

    let movement = locomotion::step(&actions, time.delta().as_secs_f32(), &mut facing, &mut loco);
    if movement != Vec3::ZERO {
        let blocked = |x: f32, z: f32| {
            shop_building
                .as_ref()
                .is_some_and(|s| crate::shop::point_blocked(s, x, z))
                || statue.as_ref().is_some_and(|s| s.blocks(x, z))
                || copper.as_ref().is_some_and(|c| c.blocks(x, z))
        };

        // Try X movement
        let new_x = transform.translation.x + movement.x;
        let test_pos_x = Vec3::new(new_x, 0.0, transform.translation.z);
        if town.can_step(transform.translation, test_pos_x)
            && !blocked(new_x, transform.translation.z)
        {
            transform.translation.x = new_x;
        }

        // Try Z movement
        let new_z = transform.translation.z + movement.z;
        let test_pos_z = Vec3::new(transform.translation.x, 0.0, new_z);
        if town.can_step(transform.translation, test_pos_z)
            && !blocked(transform.translation.x, new_z)
        {
            transform.translation.z = new_z;
        }
    }
}

/// Game units (as in the decomp) per world unit: one cell is 40.
const GAME_UNIT: f32 = 1.0 / 40.0;
/// Town camera (`Init_Camera2`, `Camera2_Get_GoalDistanceAndDirection`):
/// 20 deg vertical field of view, 620 units from the look-at point,
/// pitched 45 deg down, facing north.
const CAMERA_FOV_Y_DEG: f32 = 20.0;
const CAMERA_DISTANCE: f32 = 620.0 * GAME_UNIT;
const CAMERA_PITCH_DEG: f32 = 45.0;
const CAMERA_NEAR: f32 = 200.0 * GAME_UNIT;
const CAMERA_FAR: f32 = 1600.0 * GAME_UNIT;
/// The look-at point is the player's eye position, 33 units above the
/// feet (`Actor_world_to_eye(actorx, 33.0f)`).
const PLAYER_EYE_HEIGHT: f32 = 33.0 * GAME_UNIT;
/// `Camera2_MoveCenterPosAndSpeed_fromEndCenterPos`: the look-at point
/// closes this fraction of the gap per 60 Hz frame, by at most 8.75 units.
const CAMERA_CHASE_FRACTION: f32 = 0.133_974_61;
const CAMERA_CHASE_MAX_STEP: f32 = 8.75 * GAME_UNIT;

/// From the look-at point to the camera.
fn camera_eye_offset() -> Vec3 {
    let pitch = CAMERA_PITCH_DEG.to_radians();
    Vec3::new(0.0, pitch.sin(), pitch.cos()) * CAMERA_DISTANCE
}

/// Camera follows the player like the game's normal camera: the look-at
/// point eases toward the player's eye, and the camera sits at a fixed
/// distance and angle from it.
pub fn camera_follow(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<
        &mut Transform,
        (
            With<WorldCamera>,
            Without<Player>,
            Without<crate::msg_window::OverlayCamera>,
        ),
    >,
    mut centre: Local<Option<Vec3>>,
    preview: Res<crate::shop_nook::preview::Preview>,
    location: Res<State<crate::shop::ShopLocation>>,
    mut preview_camera: Local<crate::shop_nook::preview::CameraBlend>,
    layout: Res<crate::shop::layout::Layout>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let feet = player_transform.translation - Vec3::Y * crate::disc_town::PLAYER_HALF_HEIGHT;
    let target = feet + Vec3::Y * PLAYER_EYE_HEIGHT;
    let c = centre.get_or_insert(target);
    // add_calc per 60 Hz frame, scaled to this frame's length.
    let frames = time.delta_secs() * 60.0;
    let fraction = 1.0 - (1.0 - CAMERA_CHASE_FRACTION).powf(frames);
    let gap = target - *c;
    let step = (gap * fraction).clamp_length_max(CAMERA_CHASE_MAX_STEP * frames.max(1.0));
    // Snap across big jumps (warps, entering buildings).
    *c = if gap.length() > CAMERA_DISTANCE {
        target
    } else {
        *c + step
    };

    let (look, eye) = preview_camera.view(
        preview.camera(*layout),
        (*c, *c + camera_eye_offset()),
        frames,
        *location == crate::shop::ShopLocation::Inside,
    );
    camera_transform.translation = eye;
    camera_transform.look_at(look, Vec3::Y);
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player).add_systems(
            Update,
            (
                player_movement.in_set(locomotion::MovementSet),
                camera_follow
                    .after(locomotion::MovementSet)
                    .after(crate::shop_nook::interact),
            )
                .in_set(GameplaySet),
        );
    }
}
