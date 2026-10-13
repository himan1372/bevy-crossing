//! Pockets screen: the 15 pocket slots from `rustimal_logic::inventory`.
//!
//! I (or Tab) opens it while the world has focus; I, Tab or Esc closes it.
//! While open it owns input (see `InputFocus::Pockets`), so the player
//! stands still.

use bevy::prelude::*;
use bevy::text::FontSize;
use rustimal_logic::inventory::{EMPTY_NO, POCKETS_SLOT_COUNT};

use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::PlayerInventory;
use crate::items;
use crate::states::GameplaySet;

/// Whether the pockets screen is open. Read by `input::update_focus`.
#[derive(Resource, Default)]
pub struct PocketsUi {
    pub open: bool,
}

#[derive(Component)]
struct PocketsPanel;
#[derive(Component)]
struct PocketsText;

fn setup_pockets_ui(mut commands: Commands) {
    commands
        .spawn((
            PocketsPanel,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(30.0),
                right: Val::Percent(30.0),
                top: Val::Percent(15.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(18.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.10, 0.07, 0.95)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Pockets"),
                TextFont {
                    font_size: FontSize::Px(26.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.5)),
            ));
            parent.spawn((
                PocketsText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("I / Esc: close"),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.75)),
            ));
        });
}

/// Open from the world; close with the same key or Cancel.
fn toggle_pockets(
    actions: Res<Actions>,
    focus: Res<InputFocus>,
    mut ui: ResMut<PocketsUi>,
    mut panel: Query<&mut Visibility, With<PocketsPanel>>,
) {
    let next = if ui.open {
        !(actions.just_pressed(Action::Pockets) || actions.just_pressed(Action::Cancel))
    } else {
        focus.is_world() && actions.just_pressed(Action::Pockets)
    };
    if next == ui.open {
        return;
    }
    ui.open = next;
    if let Ok(mut vis) = panel.single_mut() {
        *vis = if next {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// One line per slot, numbered 1-15.
fn pockets_listing(inventory: &PlayerInventory) -> String {
    let mut s = String::new();
    for (i, &item) in inventory.pockets.pockets.iter().enumerate() {
        let label = if item == EMPTY_NO {
            "-".to_string()
        } else {
            items::name(item)
        };
        s.push_str(&format!("{:>2}. {label}\n", i + 1));
    }
    s.push_str(&format!(
        "\n{}/{} used    Bells: {}",
        inventory.used(),
        POCKETS_SLOT_COUNT,
        inventory.bells
    ));
    s
}

fn refresh_pockets(
    ui: Res<PocketsUi>,
    inventory: Res<PlayerInventory>,
    mut text: Query<&mut Text, With<PocketsText>>,
) {
    if !ui.open || !(ui.is_changed() || inventory.is_changed()) {
        return;
    }
    if let Ok(mut t) = text.single_mut() {
        t.0 = pockets_listing(&inventory);
    }
}

pub struct PocketsPlugin;

impl Plugin for PocketsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PocketsUi>()
            .add_systems(Startup, setup_pockets_ui)
            .add_systems(
                Update,
                (toggle_pockets, refresh_pockets)
                    .chain()
                    .in_set(InteractSet::Ui)
                    .in_set(GameplaySet),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listing_shows_15_numbered_slots() {
        let mut inv = PlayerInventory::default();
        assert!(inv.add(items::FRUIT));
        let text = pockets_listing(&inv);
        assert!(text.starts_with(" 1. apple\n 2. -\n"));
        assert!(text.contains("15. -"));
        assert!(text.contains("1/15 used"));
    }
}
