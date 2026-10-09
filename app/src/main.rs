use bevy::prelude::*;

mod town;
mod player;

use town::TownPlugin;
use player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TownPlugin)
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, setup_light)
        .run();
}

fn setup_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}
