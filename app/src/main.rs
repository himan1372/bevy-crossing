// Bevy systems take many params and nested Query filters by design.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;

mod assets;
mod character;
mod clock;
mod dialogue;
mod disc_gfx;
mod disc_town;
mod ecology;
mod environment;
mod face;
mod game_disc;
mod ground_map;
mod house;
mod input;
mod interaction;
mod items;
mod locomotion;
mod mail;
mod model_view;
mod n64_material;
mod npc;
mod player;
mod pockets;
mod quest;
mod remote;
mod save;
mod shop;
mod skeleton;
mod states;
mod talk;
mod town;

use assets::GameAssetsPlugin;
use clock::ClockPlugin;
use dialogue::DialoguePlugin;
use disc_town::DiscTownPlugin;
use ecology::EcologyPlugin;
use game_disc::GameDiscPlugin;
use house::HousePlugin;
use input::InputPlugin;
use interaction::InteractionPlugin;
use mail::MailPlugin;
use model_view::ModelViewPlugin;
use n64_material::N64MaterialPlugin;
use npc::NpcPlugin;
use player::PlayerPlugin;
use pockets::PocketsPlugin;
use quest::QuestPlugin;
use remote::RemotePluginGate;
use save::SavePlugin;
use shop::ShopPlugin;
use states::GameStatePlugin;
use town::TownPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RemotePluginGate)
        // Foundations first: states, input, and the asset gate.
        .add_plugins((
            GameStatePlugin,
            InputPlugin,
            GameAssetsPlugin,
            GameDiscPlugin,
            ModelViewPlugin,
            N64MaterialPlugin,
            DiscTownPlugin,
        ))
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
        .add_plugins(PocketsPlugin)
        .add_plugins(ClockPlugin)
        .add_plugins(environment::EnvironmentPlugin)
        .add_plugins(character::CharacterPlugin)
        .add_plugins(face::FacePlugin)
        .add_systems(Startup, setup_light)
        .run();
}

fn setup_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}
