use bevy::prelude::*;

mod dialogue;
mod ecology;
mod house;
mod interaction;
mod mail;
mod npc;
mod player;
mod quest;
mod remote;
mod save;
mod shop;
mod town;

use dialogue::DialoguePlugin;
use ecology::EcologyPlugin;
use house::HousePlugin;
use interaction::InteractionPlugin;
use mail::MailPlugin;
use npc::NpcPlugin;
use player::PlayerPlugin;
use quest::QuestPlugin;
use remote::RemotePluginGate;
use save::SavePlugin;
use shop::ShopPlugin;
use town::TownPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RemotePluginGate)
        .add_plugins(SavePlugin)
        .add_plugins(TownPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(NpcPlugin)
        .add_plugins(InteractionPlugin)
        .add_plugins(DialoguePlugin)
        .add_plugins(EcologyPlugin)
        .add_plugins(HousePlugin)
        .add_plugins(MailPlugin)
        .add_plugins(ShopPlugin)
        .add_plugins(QuestPlugin)
        .add_systems(Startup, setup_light)
        .run();
}

fn setup_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}
