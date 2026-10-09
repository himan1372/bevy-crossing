use bevy::prelude::*;
use crate::town::Town;

/// Player marker component.
#[derive(Component)]
pub struct Player;

/// Movement speed in units per second.
const PLAYER_SPEED: f32 = 5.0;

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
        PbrBundle {
            mesh: player_mesh,
            material: player_mat,
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },
    ));

    // Spawn camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 12.0, 10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    println!("Player and camera spawned");
}

/// Handle WASD movement with collision.
pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    town: Res<Town>,
    time: Res<Time>,
) {
    let Ok(mut transform) = player_query.get_single_mut() else {
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

        // Try X movement
        let new_x = transform.translation.x + movement.x;
        let test_pos_x = Vec3::new(new_x, 0.0, transform.translation.z);
        if town.is_walkable(test_pos_x) {
            transform.translation.x = new_x;
        }

        // Try Z movement
        let new_z = transform.translation.z + movement.z;
        let test_pos_z = Vec3::new(transform.translation.x, 0.0, new_z);
        if town.is_walkable(test_pos_z) {
            transform.translation.z = new_z;
        }
    }
}

/// Camera follows the player.
pub fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    let target = player_transform.translation + Vec3::new(0.0, 12.0, 10.0);
    camera_transform.translation = camera_transform.translation.lerp(target, 0.1);
    camera_transform.look_at(player_transform.translation, Vec3::Y);
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (player_movement, camera_follow));
    }
}
