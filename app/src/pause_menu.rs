//! The pause menu: Save / Settings / Quit (roadmap `pause-menu`).
//!
//! P opens it (and closes it); Esc closes it too. Up/down pick, Enter or
//! E chooses. Save queues the usual quick save and resumes. Settings
//! shows the disc path from `settings.json` and lets you type a new one
//! (Enter writes it; it's used from the next start). Quit exits.

use bevy::app::AppExit;
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::text::FontSize;

use crate::input::{Action, Actions};
use crate::states::PauseState;

const ITEMS: [&str; 3] = ["Save", "Settings", "Quit"];
const SETTINGS_FILE: &str = "settings.json";

/// A save the menu asked for; quick save takes it when play resumes.
#[derive(Resource, Default)]
pub struct SaveRequest(pub bool);

#[derive(Resource, Default)]
struct Menu {
    selected: usize,
    /// Editing the disc path (the Settings page).
    editing: Option<String>,
}

#[derive(Component)]
struct MenuText;

fn read_disc_path() -> String {
    std::fs::read_to_string(SETTINGS_FILE)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("disc_path")?.as_str().map(String::from))
        .unwrap_or_default()
}

/// Writes `disc_path` into settings.json, keeping its other keys.
fn write_disc_path(path: &str) -> std::io::Result<()> {
    let mut v = std::fs::read_to_string(SETTINGS_FILE)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .filter(|v| v.is_object())
        .unwrap_or_else(|| serde_json::json!({}));
    v["disc_path"] = serde_json::Value::String(path.to_string());
    std::fs::write(SETTINGS_FILE, serde_json::to_string_pretty(&v)?)
}

fn setup(mut commands: Commands) {
    commands.spawn((
        MenuText,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(30.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(35.0),
            left: Val::Percent(38.0),
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
        Visibility::Hidden,
    ));
}

fn open(mut menu: ResMut<Menu>, mut text: Query<&mut Visibility, With<MenuText>>) {
    *menu = Menu::default();
    for mut v in &mut text {
        *v = Visibility::Visible;
    }
}

fn close(mut text: Query<&mut Visibility, With<MenuText>>) {
    for mut v in &mut text {
        *v = Visibility::Hidden;
    }
}

#[allow(clippy::too_many_arguments)]
fn menu_input(
    mut actions: ResMut<Actions>,
    mut menu: ResMut<Menu>,
    mut keys: MessageReader<KeyboardInput>,
    mut next: ResMut<NextState<PauseState>>,
    mut save: ResMut<SaveRequest>,
    mut exit: MessageWriter<AppExit>,
    mut text: Query<&mut Text, With<MenuText>>,
) {
    if let Some(path) = menu.editing.as_mut() {
        // Typing: P and other letters are text here, not commands.
        actions.consume(Action::Pause);
        for k in keys.read().filter(|k| k.state.is_pressed()) {
            match &k.logical_key {
                Key::Character(c) => path.push_str(c),
                Key::Space => path.push(' '),
                Key::Backspace => {
                    path.pop();
                }
                Key::Enter => {
                    let saved = write_disc_path(path);
                    crate::sound::se(if saved.is_ok() {
                        crate::sound::se_id::CHOICE_CONFIRM
                    } else {
                        crate::sound::se_id::MENU_EXIT
                    });
                    println!("Disc path saved ({saved:?}); used from the next start");
                    menu.editing = None;
                    break;
                }
                Key::Escape => {
                    crate::sound::se(crate::sound::se_id::MENU_EXIT);
                    menu.editing = None;
                    break;
                }
                _ => {}
            }
        }
        actions.consume(Action::Cancel);
        actions.consume(Action::Confirm);
    } else {
        keys.clear();
        let n = ITEMS.len();
        let before = menu.selected;
        if actions.just_pressed(Action::MenuUp) || actions.just_pressed(Action::MoveUp) {
            menu.selected = (menu.selected + n - 1) % n;
        }
        if actions.just_pressed(Action::MenuDown) || actions.just_pressed(Action::MoveDown) {
            menu.selected = (menu.selected + 1) % n;
        }
        crate::sound::cursor_changed(before, menu.selected);
        if actions.consume(Action::Cancel) {
            crate::sound::se(crate::sound::se_id::MENU_EXIT);
            next.set(PauseState::Running);
        } else if actions.consume(Action::Confirm) || actions.consume(Action::Interact) {
            crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
            match menu.selected {
                0 => {
                    save.0 = true;
                    next.set(PauseState::Running);
                }
                1 => menu.editing = Some(read_disc_path()),
                _ => {
                    exit.write(AppExit::Success);
                }
            }
        }
    }
    let shown = match &menu.editing {
        Some(path) => format!(
            "Settings\n\nDisc path:\n{path}_\n\nEnter: save   Esc: back\n(used from the next start)"
        ),
        None => {
            let mut s = String::from("Paused\n\n");
            for (i, item) in ITEMS.iter().enumerate() {
                s.push_str(if i == menu.selected { "> " } else { "  " });
                s.push_str(item);
                s.push('\n');
            }
            s.push_str("\nP / Esc: resume");
            s
        }
    };
    for mut t in &mut text {
        if t.0 != shown {
            t.0 = shown.clone();
        }
    }
}

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Menu>()
            .init_resource::<SaveRequest>()
            .add_systems(Startup, setup)
            .add_systems(OnEnter(PauseState::Paused), open)
            .add_systems(OnExit(PauseState::Paused), close)
            .add_systems(
                Update,
                menu_input
                    .before(crate::states::toggle_pause)
                    .run_if(in_state(PauseState::Paused)),
            );
    }
}
