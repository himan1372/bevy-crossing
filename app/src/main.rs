use bevy::prelude::*;

mod town;
mod player;
mod npc;
mod interaction;
mod dialogue;

use town::TownPlugin;
use player::PlayerPlugin;
use npc::NpcPlugin;
use interaction::InteractionPlugin;
use dialogue::DialoguePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TownPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(NpcPlugin)
        .add_plugins(InteractionPlugin)
        .add_plugins(DialoguePlugin)
        .add_systems(Startup, setup_light)
        .run();
}

fn setup_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}
