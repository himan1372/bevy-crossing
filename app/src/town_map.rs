//! The town map (`mSM_CHECK_OPEN_MAP`: the X button while `Common(map_flag)` is set).
//!
//! The retail map overlay (`m_map_ovl.c`) is not built; this is a plain panel of the acre
//! grid (rows A-F, columns 1-5, as the guide's messages name them) with the town's public
//! buildings, the player's house, and where the player stands. The flag and when it is
//! given are the source's: off from the new-game intro, handed over by the guide after the
//! third job, and by Officer Copper when it is missing.

use bevy::prelude::*;
use rustimal_logic::town_gen::{Feature, ACRE_DEPTH, ACRE_WIDTH};

use crate::input::{Action, Actions, InputFocus};
use crate::player::Player;
use crate::town::Town;

#[derive(Resource, Default)]
pub(crate) struct MapUi {
    pub open: bool,
}

#[derive(Component)]
struct MapPanel;
#[derive(Component)]
struct MapText;

fn setup(mut commands: Commands) {
    commands
        .spawn((
            MapPanel,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(30.0),
                right: Val::Percent(30.0),
                top: Val::Percent(18.0),
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.09, 0.04, 0.95)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                MapText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.95, 0.75)),
            ));
        });
}

fn symbol(feature: u8) -> char {
    match feature {
        f if f == Feature::Station as u8 => 'T',
        f if f == Feature::Shop as u8 => 'S',
        f if f == Feature::PostOffice as u8 => 'P',
        f if f == Feature::PlayerHouse as u8 => 'H',
        f if f == Feature::WishingWell as u8 => 'W',
        f if f == Feature::PoliceStation as u8 => 'C',
        f if f == Feature::Museum as u8 => 'M',
        f if f == Feature::Tailor as u8 => 'A',
        f if f == Feature::Dock as u8 => 'D',
        _ => '.',
    }
}

/// The map as text: one row per acre row, two characters per acre (the building, then `*`
/// where the player stands).
fn render(town: &Town, here: Option<(usize, usize)>) -> String {
    let mut out = String::from("Town map\n\n    ");
    for x in 0..ACRE_WIDTH {
        out.push_str(&format!("{}  ", x + 1));
    }
    out.push('\n');
    for z in 0..ACRE_DEPTH {
        out.push_str(&format!("{}   ", ['A', 'B', 'C', 'D', 'E', 'F'][z.min(5)]));
        for x in 0..ACRE_WIDTH {
            let feature = town.plan.acres[z * ACRE_WIDTH + x].feature;
            out.push(symbol(feature));
            out.push(if here == Some((x, z)) { '*' } else { ' ' });
            out.push(' ');
        }
        out.push('\n');
    }
    out.push_str(
        "\nT station  S shop  P post office  C police  M museum\nA tailor  W well  H your house  D dock  * you\n",
    );
    out
}

#[allow(clippy::too_many_arguments)]
fn input(
    mut ui: ResMut<MapUi>,
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    identity: Res<crate::save::PlayerIdentity>,
    town: Res<Town>,
    state: Res<State<crate::shop::ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    player: Query<&Transform, With<Player>>,
    mut panel: Query<&mut Visibility, With<MapPanel>>,
    mut text: Query<&mut Text, With<MapText>>,
) {
    if ui.open {
        if actions.consume(Action::Cancel)
            || actions.consume(Action::Back)
            || actions.consume(Action::Map)
            || actions.consume(Action::Confirm)
        {
            ui.open = false;
            crate::sound::se(crate::sound::se_id::MENU_EXIT);
        }
    } else if focus.is_world()
        && identity.map_flag
        && *state.get() == crate::shop::ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town
        && actions.consume(Action::Map)
    {
        ui.open = true;
        crate::sound::se(crate::sound::se_id::CHOICE_OPEN);
        let here = player
            .single()
            .ok()
            .and_then(|t| Town::cell_indices_of(t.translation))
            .map(|(ax, az, _, _)| (ax, az));
        if let Ok(mut text) = text.single_mut() {
            text.0 = render(&town, here);
        }
    }
    if let Ok(mut visibility) = panel.single_mut() {
        *visibility = if ui.open {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub struct TownMapPlugin;

impl Plugin for TownMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapUi>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                input
                    .in_set(crate::input::InteractSet::Ui)
                    .in_set(crate::states::GameplaySet),
            );
    }
}
