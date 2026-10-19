//! Placed-diary access to the player's shared monthly notebook.
use super::*;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use rustimal_logic::diary as native;

mod render;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Notebook {
    months: [Vec<u8>; native::MONTHS],
}

impl Default for Notebook {
    fn default() -> Self {
        Self {
            months: std::array::from_fn(|_| native::entry(&[])),
        }
    }
}

impl Notebook {
    fn text(&self, month: usize) -> Vec<u8> {
        let bytes = &self.months[month];
        bytes[..native::text_len(bytes).min(native::ENTRY_SIZE)].to_vec()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Read,
    Edit,
    Confirm,
}

pub(super) struct View {
    month: usize,
    mode: Mode,
    draft: Vec<u8>,
    scroll: usize,
    choice: usize,
    just_opened: bool,
    revision: u64,
}

/// aMR_CheckDiaryOnMe: only a visible diary on a contacted surface opens it.
/// Floor diaries and owner-cell hidden storage do not act as reading desks.
pub(super) fn can_open(room: &PlayerRoom, slot: usize, pos: Vec3) -> bool {
    let Some(actor) = room.runtime.actors.get(slot).and_then(Option::as_ref) else {
        return false;
    };
    if actor.layer != 0
        || !room
            .profiles
            .get(&actor.name)
            .is_some_and(|p| p.set_type == SetType::Surface && !ftr::is_storage(p.interaction_type))
    {
        return false;
    }
    actor
        .shape_type
        .unit_offsets()
        .iter()
        .take(actor.shape_type.unit_count())
        .any(|&(dx, dz)| {
            let (x, z) = (actor.cell_x + dx, actor.cell_z + dz);
            let center = corner() + Vec3::new(x as f32 + 0.5, 0.0, z as f32 + 0.5);
            room.layers[1]
                .get(x, z)
                .is_some_and(|item| (0x2B00..=0x2B0F).contains(&item))
                && ((center.x - pos.x).abs() < 0.3 || (center.z - pos.z).abs() < 0.3)
        })
}

pub(super) fn open(room: &mut PlayerRoom) {
    let month = usize::from(crate::clock::now().month.saturating_sub(1)).min(11);
    room.diary_ui = Some(View {
        month,
        mode: Mode::Read,
        draft: room.diary.as_ref().map_or_else(Vec::new, |n| n.text(month)),
        scroll: 0,
        choice: 0,
        just_opened: true,
        revision: 0,
    });
    crate::sound::se(0x5F); // mDI_diary_ovl_init.
}

fn display(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&code| {
            if code == native::NEWLINE {
                '\n'
            } else {
                crate::messages::char_of(code).unwrap_or('?')
            }
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn input(
    mut room: ResMut<PlayerRoom>,
    mut actions: ResMut<Actions>,
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut focus: ResMut<InputFocus>,
    current: Res<CurrentRoom>,
    state: Res<State<HouseState>>,
    font: Option<Res<crate::msg_window::MsgAssets>>,
) {
    if !crate::house::is_player_room(&current, &state) {
        room.diary_ui = None;
    }
    let Some(mut view) = room.diary_ui.take() else {
        events.clear();
        return;
    };
    if *focus != InputFocus::Diary || view.just_opened {
        events.clear();
        view.just_opened = false;
        room.diary_ui = Some(view);
        return;
    }
    let Some(font) = font else {
        room.diary_ui = Some(view);
        events.clear();
        return;
    };
    let before = (
        view.month,
        view.mode,
        view.scroll,
        view.choice,
        view.draft.clone(),
    );
    let mut close = false;
    match view.mode {
        Mode::Read => {
            events.clear();
            if actions.consume(Action::Cancel) || actions.consume(Action::Back) {
                close = true;
            } else if actions.consume(Action::Confirm) {
                view.mode = Mode::Edit;
                view.scroll = native::lines(&view.draft, &font.widths)
                    .len()
                    .saturating_sub(native::VISIBLE_LINES);
            } else if actions.just_pressed(Action::MoveLeft)
                || actions.just_pressed(Action::MoveRight)
            {
                let delta = if actions.consume(Action::MoveLeft) {
                    11
                } else {
                    1
                };
                actions.consume(Action::MoveRight);
                view.month = (view.month + delta) % native::MONTHS;
                view.draft = room
                    .diary
                    .as_ref()
                    .map_or_else(Vec::new, |n| n.text(view.month));
                view.scroll = 0;
            } else {
                let delta = if actions.consume(Action::MoveUp) {
                    -1
                } else if actions.consume(Action::MoveDown) {
                    1
                } else if keys.just_pressed(KeyCode::PageUp) {
                    -10
                } else if keys.just_pressed(KeyCode::PageDown) {
                    10
                } else {
                    0
                };
                view.scroll = native::scroll(
                    view.scroll,
                    native::lines(&view.draft, &font.widths).len(),
                    delta,
                );
            }
        }
        Mode::Edit => {
            for event in events.read() {
                if event.state != ButtonState::Pressed {
                    continue;
                }
                match event.key_code {
                    KeyCode::Escape => {
                        // PC editor cancellation discards only this transient draft.
                        view.draft = room
                            .diary
                            .as_ref()
                            .map_or_else(Vec::new, |n| n.text(view.month));
                        view.mode = Mode::Read;
                        view.scroll = 0;
                        break;
                    }
                    KeyCode::Enter
                        if keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) =>
                    {
                        view.mode = Mode::Confirm;
                        view.choice = 0;
                        break;
                    }
                    KeyCode::Backspace => {
                        view.draft.pop();
                    }
                    KeyCode::Enter => {
                        if native::can_append(&view.draft, native::NEWLINE, &font.widths) {
                            view.draft.push(native::NEWLINE);
                        }
                    }
                    _ => {
                        if let Key::Character(chars) = &event.logical_key {
                            for ch in chars.chars().filter(|c| !c.is_control()) {
                                if let Some(code) = crate::messages::ac_code_opt(ch) {
                                    if native::can_append(&view.draft, code, &font.widths) {
                                        view.draft.push(code);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if view.mode == Mode::Edit {
                view.scroll = native::lines(&view.draft, &font.widths)
                    .len()
                    .saturating_sub(native::VISIBLE_LINES);
            }
        }
        Mode::Confirm => {
            events.clear();
            if actions.consume(Action::Cancel) {
                view.mode = Mode::Edit;
            } else if actions.consume(Action::MoveUp) || actions.consume(Action::MoveDown) {
                view.choice ^= 1;
            } else if actions.consume(Action::Confirm) {
                if native::confirm_writes(view.choice) {
                    room.diary.get_or_insert_with(Notebook::default).months[view.month] =
                        native::entry(&view.draft);
                    close = true;
                } else {
                    view.mode = Mode::Edit;
                }
            }
        }
    }
    if close {
        *focus = InputFocus::World;
        crate::sound::se(crate::sound::se_id::MENU_EXIT);
    } else {
        if before
            != (
                view.month,
                view.mode,
                view.scroll,
                view.choice,
                view.draft.clone(),
            )
        {
            view.revision = view.revision.wrapping_add(1);
            if view.mode != Mode::Edit {
                crate::sound::se(crate::sound::se_id::CURSOR);
            }
        }
        room.diary_ui = Some(view);
    }
}

pub(super) struct DiaryPlugin;
impl Plugin for DiaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<render::DiaryAssets>()
            .add_systems(
                Update,
                input
                    .in_set(InteractSet::Ui)
                    .after(crate::save::LoadSet)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                render::draw
                    .after(InteractSet::Ui)
                    .after(crate::save::LoadSet)
                    .in_set(GameplaySet)
                    .run_if(resource_exists::<crate::msg_window::MsgAssets>),
            );
    }
}
