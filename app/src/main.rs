// Bevy systems take many params and nested Query filters by design.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;

mod ambience;
mod assets;
mod catalog;
mod catalog_lists;
mod catalog_screen;
mod character;
mod clock;
mod copper;
mod critters;
mod dev_overlay;
mod dialogue;
mod disc_gfx;
mod disc_rig;
mod disc_town;
mod ecology;
mod environment;
mod face;
mod field_items;
mod field_rank;
mod first_job_talk;
mod footsteps;
mod furniture;
mod game_disc;
mod game_rand;
mod goods;
mod ground_map;
mod gyroid;
mod home;
mod house;
mod idle_clock;
mod indoor_room;
mod input;
mod interaction;
mod interior;
mod items;
mod joan;
mod locomotion;
mod mail;
mod mail_disc;
mod mailbox;
mod messages;
mod model_view;
mod msg_window;
mod museum;
mod museum_fish;
mod museum_insect;
mod n64_material;
mod notice_board;
mod npc;
mod pause_menu;
mod player;
mod player_room;
mod player_tools;
mod player_toys;
mod pockets;
mod pockets_disc;
mod police_box;
mod post;
mod post_office;
mod postman;
mod quest;
mod raffle;
mod remote;
mod roster;
mod save;
mod shop;
mod shop_display;
mod shop_nook;
mod skeleton;
mod sound;
mod sound_rom;
mod states;
mod statue;
mod talk;
mod town;
mod villager_memory;
mod warning_panel;

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
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // BEVY_CROSSING_NO_VSYNC=1 measures the real frame cost.
                present_mode: if std::env::var_os("BEVY_CROSSING_NO_VSYNC").is_some() {
                    bevy::window::PresentMode::AutoNoVsync
                } else {
                    bevy::window::PresentMode::AutoVsync
                },
                ..default()
            }),
            ..default()
        }))
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
        .init_resource::<game_rand::GameRand>()
        .add_plugins(SavePlugin)
        .add_plugins(dev_overlay::DevOverlayPlugin)
        .add_plugins(TownPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(NpcPlugin)
        .add_plugins(InteractionPlugin)
        .add_plugins(DialoguePlugin)
        .add_plugins(EcologyPlugin)
        .add_plugins(HousePlugin)
        .add_plugins(player_room::PlayerRoomPlugin)
        .add_plugins(MailPlugin)
        .add_plugins(ShopPlugin)
        .add_plugins(shop_display::ShopDisplayPlugin)
        .add_plugins(shop_nook::ShopNookPlugin)
        .add_plugins(first_job_talk::FirstJobTalkPlugin)
        .add_plugins(notice_board::NoticeBoardPlugin)
        .add_plugins(field_items::FieldItemsPlugin)
        .add_plugins(critters::CrittersPlugin)
        .add_plugins(sound::SoundPlugin)
        .add_plugins(footsteps::FootstepPlugin)
        .add_plugins(gyroid::GyroidPlugin)
        .add_plugins(mailbox::MailboxPlugin)
        .add_plugins(pause_menu::PauseMenuPlugin)
        .add_plugins(msg_window::MsgWindowPlugin)
        .add_plugins(QuestPlugin)
        .add_plugins(PocketsPlugin)
        .add_plugins(warning_panel::WarningPanelPlugin)
        .add_plugins(ClockPlugin)
        .add_plugins(idle_clock::IdleClockPlugin)
        .add_plugins(post_office::PostOfficePlugin)
        .add_plugins(police_box::PoliceBoxPlugin)
        .add_plugins(museum::MuseumPlugin)
        .add_plugins(museum_fish::MuseumFishPlugin)
        .add_plugins(museum_insect::MuseumInsectPlugin)
        .add_plugins(copper::CopperPlugin)
        .add_plugins(joan::JoanPlugin)
        .add_plugins(field_rank::FieldRankPlugin)
        .add_plugins(post::PostPlugin)
        .add_plugins(catalog::CatalogPlugin)
        .add_plugins(catalog_screen::CatalogScreenPlugin)
        .add_plugins(raffle::RafflePlugin)
        .add_plugins(postman::PostmanPlugin)
        .add_plugins(villager_memory::VillagerMemoryPlugin)
        .add_plugins(home::HomePlugin)
        .add_plugins(statue::StatuePlugin)
        .add_plugins(furniture::house_clock::HouseClockPlugin)
        .add_plugins(environment::EnvironmentPlugin)
        .add_plugins(character::CharacterPlugin)
        .add_plugins(player_tools::PlayerToolsPlugin)
        .add_plugins(player_toys::PlayerToysPlugin)
        .add_plugins(face::FacePlugin)
        .add_plugins(roster::RosterPlugin)
        .add_systems(Startup, setup_light)
        .run();
}

fn setup_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}
