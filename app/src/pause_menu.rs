//! PC pause with retail save dialogue (`ac_npc_restart_talk.c_inc`).
mod render;
use crate::input::{Action, Actions};
use crate::states::PauseState;
use bevy::app::AppExit;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
const SETTINGS_FILE: &str = "settings.json";

/// Taken by the save collector even while the world is paused.
#[derive(Resource, Default)]
pub struct SaveRequest(pub bool);
/// A menu may only quit after an actual successful write.
#[derive(Resource, Default)]
pub(crate) struct SaveOutcome(pub Option<bool>);
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Page {
    #[default]
    Main,
    Pc,
    Settings,
    QuitConfirm,
    Saving {
        quit: bool,
    },
    Saved {
        quit: bool,
    },
    Error {
        quit: bool,
    },
}
#[derive(Resource, Default)]
struct Menu {
    page: Page,
    selected: usize,
    path: String,
    notice: Option<String>,
}
#[derive(Clone, PartialEq, Eq)]
struct View {
    body: String,
    answers: Vec<String>,
    selected: usize,
    hint: &'static str,
}

fn read_disc_path() -> String {
    std::fs::read_to_string(SETTINGS_FILE)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("disc_path")?.as_str().map(String::from))
        .unwrap_or_default()
}
fn write_disc_path(path: &str) -> std::io::Result<()> {
    let mut v = std::fs::read_to_string(SETTINGS_FILE)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .filter(|v| v.is_object())
        .unwrap_or_else(|| serde_json::json!({}));
    v["disc_path"] = serde_json::Value::String(path.into());
    std::fs::write(SETTINGS_FILE, serde_json::to_string_pretty(&v)?)
}
impl Menu {
    fn page(&mut self, page: Page) {
        self.page = page;
        self.selected = 0;
        self.notice = None;
    }
    fn save(&mut self, quit: bool, request: &mut SaveRequest, outcome: &mut SaveOutcome) {
        self.page(Page::Saving { quit });
        outcome.0 = None;
        request.0 = true;
    }
    fn view(
        &self,
        disc: Option<&crate::game_disc::GameDisc>,
        identity: &crate::save::PlayerIdentity,
    ) -> View {
        let text = disc.and_then(|d| d.text.as_ref());
        let message = |id, fallback: &str| {
            text.map(|text| {
                let ctx = crate::messages::MsgContext {
                    player_name: identity.player.clone(),
                    talk_name: identity.player.clone(),
                    // The actorless PC overlay addresses the player instead of using an NPC catchphrase.
                    tail: identity.player.clone(),
                    town_name: identity.town.clone(),
                    ..default()
                };
                let (pages, _) = text.talk(id, &ctx, &mut |_| 0);
                crate::messages::markup::strip(&pages.join("\n"))
                    .trim()
                    .to_owned()
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| fallback.into())
        };
        let choice = |id, fallback: &str| {
            text.and_then(|t| t.choice(id))
                .unwrap_or_else(|| fallback.into())
        };
        let (body, answers, hint) = match self.page {
            Page::Main => (
                message(0x30CD, "Are you done playing?\nWhat would you like to do?"),
                vec![
                    choice(0x1AE, "Save & Quit"),
                    choice(0x1AF, "Save & Continue"),
                ],
                "P/Esc: resume    F1: PC options",
            ),
            Page::Pc => (
                "PC options".into(),
                vec![
                    "Settings".into(),
                    "Quit without saving".into(),
                    "Back".into(),
                ],
                "Esc: save menu    P: resume",
            ),
            Page::Settings => {
                let chars: Vec<_> = self.path.chars().collect();
                let path = chars[chars.len().saturating_sub(64)..]
                    .chunks(32)
                    .map(|s| s.iter().collect::<String>())
                    .collect::<Vec<_>>()
                    .join("\n");
                (
                    format!(
                        "PC disc path (next start):\n{path}_\n{}",
                        self.notice.as_deref().unwrap_or("")
                    ),
                    vec![],
                    "Enter: apply    Esc: back",
                )
            }
            Page::QuitConfirm => (
                "Quit without saving?\nUnsaved progress will be lost.".into(),
                vec!["Keep playing".into(), "Quit without saving".into()],
                "Esc: cancel",
            ),
            Page::Saving { .. } => ("Saving your town...".into(), vec![], "Please wait"),
            Page::Saved { quit: false } => (
                message(
                    0x30D3,
                    "Your game has been saved.\nEnjoy the rest of your day!",
                ),
                vec![],
                "Enter/Esc/P: resume",
            ),
            Page::Saved { quit: true } => (
                message(0x30E0, "Your game has been saved.\nSee you again soon!"),
                vec![],
                "Saved - Enter: quit    Esc: resume",
            ),
            Page::Error { .. } => (
                "Your game could not be saved.\nPlease try again.".into(),
                vec!["Try again".into(), "Back".into()],
                "Esc: back - the game is still open",
            ),
        };
        View {
            body,
            answers,
            selected: self.selected,
            hint,
        }
    }
}
fn open(mut menu: ResMut<Menu>, mut outcome: ResMut<SaveOutcome>) {
    *menu = Menu::default();
    outcome.0 = None;
}
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PauseInputSet;
#[allow(clippy::too_many_arguments)] // One owner for paused input, saving and process exit.
fn menu_input(
    mut actions: ResMut<Actions>,
    mut menu: ResMut<Menu>,
    mut keys: MessageReader<KeyboardInput>,
    buttons: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<PauseState>>,
    mut request: ResMut<SaveRequest>,
    mut outcome: ResMut<SaveOutcome>,
    mut exit: MessageWriter<AppExit>,
) {
    if menu.page == Page::Settings {
        actions.consume(Action::Pause); // P is text while editing.
        for key in keys.read().filter(|k| k.state.is_pressed()) {
            match key.key_code {
                KeyCode::Backspace => {
                    menu.path.pop();
                }
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    match write_disc_path(&menu.path) {
                        Ok(()) => {
                            menu.page(Page::Pc);
                            crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
                        }
                        Err(_) => {
                            menu.notice = Some("Could not write settings.".into());
                            crate::sound::se(crate::sound::se_id::MENU_EXIT);
                        }
                    }
                    break;
                }
                KeyCode::Escape => {
                    menu.page(Page::Pc);
                    break;
                }
                _ => match &key.logical_key {
                    Key::Character(c) => {
                        menu.path.push_str(c);
                        menu.notice = None;
                    }
                    Key::Space => menu.path.push(' '),
                    _ => {}
                },
            }
        }
        for a in [
            Action::Cancel,
            Action::Confirm,
            Action::Interact,
            Action::Advance,
            Action::Back,
        ] {
            actions.consume(a);
        }
        return;
    }
    keys.clear();
    if let Page::Saving { quit } = menu.page {
        for a in [
            Action::Pause,
            Action::Cancel,
            Action::Back,
            Action::Confirm,
            Action::Interact,
            Action::Advance,
        ] {
            actions.consume(a);
        }
        if let Some(saved) = outcome.0.take() {
            menu.page(if saved {
                Page::Saved { quit }
            } else {
                Page::Error { quit }
            });
            crate::sound::se(if saved {
                crate::sound::se_id::CHOICE_CONFIRM
            } else {
                crate::sound::se_id::MENU_EXIT
            });
        }
        return;
    }
    let back = actions.consume(Action::Cancel) | actions.consume(Action::Back);
    let confirm = actions.consume(Action::Confirm)
        | actions.consume(Action::Interact)
        | actions.consume(Action::Advance);
    if back {
        match menu.page {
            Page::Main | Page::Saved { .. } => next.set(PauseState::Running),
            Page::QuitConfirm => menu.page(Page::Pc),
            _ => menu.page(Page::Main),
        }
        crate::sound::se(crate::sound::se_id::MENU_EXIT);
        return;
    }
    if buttons.just_pressed(KeyCode::F1) && matches!(menu.page, Page::Main | Page::Pc) {
        let page = if menu.page == Page::Main {
            Page::Pc
        } else {
            Page::Main
        };
        menu.page(page);
        return;
    }
    let count = match menu.page {
        Page::Main | Page::QuitConfirm | Page::Error { .. } => 2,
        Page::Pc => 3,
        _ => 0,
    };
    if count > 0 {
        let before = menu.selected;
        if actions.consume(Action::MenuUp) | actions.consume(Action::MoveUp) {
            menu.selected = (menu.selected + count - 1) % count;
        }
        if actions.consume(Action::MenuDown) | actions.consume(Action::MoveDown) {
            menu.selected = (menu.selected + 1) % count;
        }
        crate::sound::cursor_changed(before, menu.selected);
    }
    if !confirm {
        return;
    }
    crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
    match menu.page {
        Page::Main => {
            actions.consume(Action::Pause);
            let quit = menu.selected == 0;
            menu.save(quit, &mut request, &mut outcome);
        }
        Page::Pc => match menu.selected {
            0 => {
                menu.page(Page::Settings);
                menu.path = read_disc_path();
            }
            1 => menu.page(Page::QuitConfirm),
            _ => menu.page(Page::Main),
        },
        Page::QuitConfirm if menu.selected == 1 => {
            exit.write(AppExit::Success);
        }
        Page::QuitConfirm => next.set(PauseState::Running),
        Page::Saved { quit: true } => {
            exit.write(AppExit::Success);
        }
        Page::Saved { quit: false } => next.set(PauseState::Running),
        Page::Error { quit } if menu.selected == 0 => {
            actions.consume(Action::Pause);
            menu.save(quit, &mut request, &mut outcome);
        }
        Page::Error { .. } => menu.page(Page::Main),
        _ => {}
    }
}
pub struct PauseMenuPlugin;
impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Menu>()
            .init_resource::<SaveRequest>()
            .init_resource::<SaveOutcome>()
            .add_systems(Startup, render::setup)
            .add_systems(OnEnter(PauseState::Paused), open)
            .add_systems(
                Update,
                menu_input
                    .in_set(PauseInputSet)
                    .before(crate::states::toggle_pause)
                    .run_if(in_state(PauseState::Paused)),
            )
            .add_systems(
                Update,
                render::draw
                    .after(PauseInputSet)
                    .run_if(in_state(crate::states::AppState::InGame)),
            );
    }
}
