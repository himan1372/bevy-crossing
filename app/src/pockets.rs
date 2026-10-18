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
    pub selected: usize,
    pub menu: Option<usize>,
    pub grabbed: Option<usize>,
    pub mail_slot: Option<usize>,
    /// Inventory mode 0x5: return a slot to Nook, never transact here.
    pub(crate) selling: bool,
    pub(crate) sell_slot: Option<usize>,
    pub(crate) home: bool,
    pub(crate) storing: Option<usize>,
    pub(crate) room_request: Option<crate::player_room::PocketRequest>,
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
    mut actions: ResMut<Actions>,
    mut focus: ResMut<InputFocus>,
    mut ui: ResMut<PocketsUi>,
    mut mail: ResMut<crate::mail::MailUi>,
    mut panel: Query<&mut Visibility, With<PocketsPanel>>,
    home: Res<crate::player_room::PlayerRoom>,
) {
    if ui.open && *focus != InputFocus::Pockets {
        return;
    }
    if ui.open
        && (actions.just_pressed(Action::Cancel) || actions.just_pressed(Action::Back))
        && (mail.menu.is_some() || mail.grabbed.is_some())
    {
        mail.cancel_action();
        actions.consume(Action::Cancel);
        actions.consume(Action::Back);
        return;
    }
    if ui.open
        && (actions.just_pressed(Action::Cancel) || actions.just_pressed(Action::Back))
        && (ui.menu.is_some() || ui.grabbed.is_some())
    {
        ui.menu = None;
        ui.grabbed = None;
        crate::sound::se(crate::sound::se_id::MENU_EXIT);
        return;
    }
    let next = if ui.open {
        !(actions.just_pressed(Action::Pockets) || actions.just_pressed(Action::Cancel))
    } else {
        focus.is_world() && actions.just_pressed(Action::Pockets)
    };
    if next == ui.open {
        return;
    }
    ui.open = next;
    if next {
        ui.home = home.available;
    }
    ui.menu = None;
    ui.grabbed = None;
    mail.menu = None;
    mail.grabbed = None;
    mail.discard = false;
    // Acquire/release focus before Update world handlers. A combined
    // open/confirm or close/confirm press must never also use a tool/door.
    *focus = if next {
        InputFocus::Pockets
    } else {
        InputFocus::World
    };
    actions.consume(Action::Interact);
    actions.consume(Action::Confirm);
    crate::sound::se(if next {
        crate::sound::se_id::MENU_PAUSE
    } else {
        crate::sound::se_id::MENU_EXIT
    });
    if let Ok(mut vis) = panel.single_mut() {
        *vis = if next {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Existing tool support only; other retail actions require their own gameplay.
pub(crate) fn held_tool(item: u16) -> Option<crate::ecology::EquippedTool> {
    match item {
        0x2200 => Some(crate::ecology::EquippedTool::Net),
        0x2203 => Some(crate::ecology::EquippedTool::Rod),
        _ => None,
    }
}

pub(crate) fn item_actions(item: u16, condition: u32) -> Vec<&'static str> {
    if condition & 1 == 0 && held_tool(item).is_some() {
        vec!["Hold", "Grab"]
    } else {
        vec!["Grab"]
    }
}

impl PocketsUi {
    pub(crate) fn actions(&self, item: u16, condition: u32) -> Vec<&'static str> {
        if self.selling {
            if condition == 0 && crate::items::sell_price(item).is_some() {
                vec!["Sell", "Quit"]
            } else {
                // Present/quest handling and zero-value disposal have their
                // own source flows; don't offer an unsupported transaction.
                vec!["Quit"]
            }
        } else if self.storing.is_some() {
            if crate::player_room::can_store(item, condition) {
                vec!["Put in", "Quit"]
            } else {
                vec!["Quit"]
            }
        } else if self.home && condition == 0 && crate::player_room::can_place_in_room(item) {
            vec!["Place in room", "Grab"]
        } else {
            item_actions(item, condition)
        }
    }
}

fn pockets_input(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    mut ui: ResMut<PocketsUi>,
    mut inventory: ResMut<PlayerInventory>,
    mut tool: ResMut<crate::ecology::EquippedTool>,
    renderer: Option<Res<crate::pockets_disc::PocketAssets>>,
    mut mail: ResMut<crate::mail::MailUi>,
    mut mailbox: ResMut<crate::mail::Mailbox>,
) {
    if !ui.open || *focus != InputFocus::Pockets {
        return;
    }
    if renderer.as_ref().is_some_and(|r| r.hand_busy()) {
        actions.consume(Action::Confirm);
        actions.consume(Action::Interact);
        return;
    }
    let dx = i32::from(actions.just_pressed(Action::MoveRight))
        - i32::from(actions.just_pressed(Action::MoveLeft));
    let dy = i32::from(actions.just_pressed(Action::MoveDown))
        - i32::from(actions.just_pressed(Action::MoveUp));
    if let Some(slot) = ui.mail_slot {
        if mail.menu.is_some() {
            mail.letter_actions(&mut actions, &mut mailbox, slot, true);
            return;
        }
        if dx < 0 && slot % 2 == 0 && mail.grabbed.is_none() {
            ui.mail_slot = None;
            ui.selected = (slot / 2).saturating_sub(2) * 5 + 4;
        } else if dx != 0 || dy != 0 {
            ui.mail_slot = Some(
                ((slot as i32 / 2 + dy).rem_euclid(5) * 2 + (slot as i32 % 2 + dx).rem_euclid(2))
                    as usize,
            );
        }
        if dx != 0 || dy != 0 {
            crate::sound::se(crate::sound::se_id::CURSOR);
            return;
        }
        mail.letter_actions(&mut actions, &mut mailbox, slot, true);
        return;
    }
    if dx != 0 || dy != 0 {
        if let Some(choice) = ui.menu {
            let n = ui
                .actions(
                    inventory.pockets.pockets[ui.selected],
                    inventory.pockets.item_cond(ui.selected),
                )
                .len();
            ui.menu = Some((choice as i32 + dy).rem_euclid(n as i32) as usize);
        } else {
            if dx > 0
                && ui.selected % 5 == 4
                && ui.grabbed.is_none()
                && renderer.is_some()
                && !ui.selling
                && ui.storing.is_none()
            {
                // mTG_move_cursol_between_table_inventory_right: same Y.
                ui.mail_slot = Some((ui.selected / 5 + 2) * 2);
                crate::sound::se(crate::sound::se_id::CURSOR);
                return;
            }
            let col = (ui.selected as i32 % 5 + dx).rem_euclid(5);
            let row = (ui.selected as i32 / 5 + dy).rem_euclid(3);
            ui.selected = (row * 5 + col) as usize;
        }
        crate::sound::se(crate::sound::se_id::CURSOR);
    }
    if !(actions.consume(Action::Confirm) || actions.consume(Action::Interact)) {
        return;
    }
    let slot = ui.selected;
    if let Some(from) = ui.grabbed.take() {
        // mHD_swap catches/drops a slot; conditions travel with the item.
        let a = inventory.pockets.item_cond(from);
        let b = inventory.pockets.item_cond(slot);
        inventory.pockets.pockets.swap(from, slot);
        inventory.pockets.set_item_cond(from, b);
        inventory.pockets.set_item_cond(slot, a);
    } else if let Some(choice) = ui.menu.take() {
        let item = inventory.pockets.pockets[slot];
        if item == EMPTY_NO {
            return;
        }
        if ui.selling {
            if choice == 0
                && inventory.pockets.item_cond(slot) == 0
                && items::sell_price(item).is_some()
            {
                ui.sell_slot = Some(slot);
                ui.open = false;
            }
            crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
            return;
        }
        if let Some(owner) = ui.storing {
            if choice == 0 && crate::player_room::can_store(item, inventory.pockets.item_cond(slot))
            {
                ui.room_request =
                    Some(crate::player_room::PocketRequest::Store { owner, slot, item });
                ui.open = false;
            }
            return;
        }
        if ui.home
            && choice == 0
            && inventory.pockets.item_cond(slot) == 0
            && crate::player_room::can_place_in_room(item)
        {
            ui.room_request = Some(crate::player_room::PocketRequest::Place { slot, item });
            ui.open = false;
            return;
        }
        if inventory.pockets.item_cond(slot) & 1 == 0 && held_tool(item).is_some() && choice == 0 {
            if let Some(selected_tool) = held_tool(item) {
                *tool = selected_tool;
            }
            ui.open = false;
        } else {
            ui.grabbed = Some(slot);
        }
    } else if inventory.pockets.pockets[slot] != EMPTY_NO {
        ui.menu = Some(0);
    }
    crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
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
    mut panel: Query<&mut Visibility, With<PocketsPanel>>,
    disc_ui: Option<Res<crate::pockets_disc::PocketAssets>>,
) {
    if let Ok(mut visibility) = panel.single_mut() {
        *visibility = if ui.open && disc_ui.is_none() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if !ui.open || !(ui.is_changed() || inventory.is_changed()) {
        return;
    }
    if let Ok(mut t) = text.single_mut() {
        t.0 = format!(
            "{}\nSelected: {}  (arrows / Enter)\n{}",
            pockets_listing(&inventory),
            ui.selected + 1,
            if let Some(choice) = ui.menu {
                ui.actions(
                    inventory.pockets.pockets[ui.selected],
                    inventory.pockets.item_cond(ui.selected),
                )[choice]
            } else if ui.grabbed.is_some() {
                "Choose a slot; Enter to swap, Esc to cancel"
            } else {
                ""
            }
        );
    }
}

pub struct PocketsPlugin;

impl Plugin for PocketsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PocketsUi>()
            .add_systems(Startup, setup_pockets_ui)
            .add_plugins(crate::pockets_disc::PocketDiscPlugin)
            .add_systems(
                PreUpdate,
                toggle_pockets
                    .after(crate::input::update_focus)
                    .run_if(in_state(crate::states::PauseState::Running)),
            )
            .add_systems(
                Update,
                (pockets_input, refresh_pockets)
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
