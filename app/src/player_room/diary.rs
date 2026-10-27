//! Placed-diary access to the player's shared monthly notebook.
use super::*;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use rustimal_logic::diary as native;
use rustimal_logic::editor_keyboard as kb;

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
    /// Text cursor in `draft` while editing.
    cursor: usize,
    keyboard: kb::Keyboard,
    repeat: kb::ButtonRepeat,
    held: crate::shop_nook::editor::Held,
    /// 60 Hz frames owed to the retail repeat counters.
    ticks: f32,
    /// A character did not fit: the next frame opens `mWR_WARNING_WORD_OVER`.
    over: bool,
}

// `sAdo_SysTrgStart` ids from `m_editor_ovl.c`.
const GRID_MOVE: u16 = 0x411;
const CURSOR: u16 = 0x35;
const BACKSPACE: u16 = 0x1034;
const START: u16 = 0x5F;
const TOGGLE: u16 = 0x37;
const ERROR: u16 = 0x1003;

fn direction(keys: &ButtonInput<KeyCode>) -> Option<kb::Direction> {
    [
        (KeyCode::ArrowLeft, kb::Direction::Left),
        (KeyCode::ArrowRight, kb::Direction::Right),
        (KeyCode::ArrowUp, kb::Direction::Up),
        (KeyCode::ArrowDown, kb::Direction::Down),
    ]
    .into_iter()
    .find(|(key, _)| keys.pressed(*key))
    .map(|(_, direction)| direction)
}

/// A key typed into the draft: it speaks, or the entry refuses it.
fn type_code(view: &mut View, code: u8, widths: &[u8; 256]) {
    if native::insert(
        &mut view.draft,
        &mut view.cursor,
        code,
        widths,
        native::DIARY,
    ) {
        crate::shop_nook::editor::speak(code, false);
    } else {
        view.over = true;
        crate::sound::se(ERROR);
    }
}

/// X: swaps the character before the cursor for its accent twin.
fn exchange(view: &mut View) {
    match kb::exchange_code(&view.draft, view.cursor) {
        Some(code) => {
            kb::exchange(&mut view.draft, view.cursor);
            crate::shop_nook::editor::speak(code, true);
        }
        None => crate::sound::se(ERROR),
    }
}

/// Starts editing: the cursor goes to the end of the entry.
fn begin_edit(view: &mut View, widths: &[u8; 256]) {
    view.mode = Mode::Edit;
    view.cursor = view.draft.len();
    view.repeat = kb::ButtonRepeat::default();
    follow_cursor(view, widths);
}

/// The paper shows five lines above the keyboard while editing.
pub(super) const EDIT_LINES: usize = 5;

/// Keeps the cursor's line among the visible ones.
fn follow_cursor(view: &mut View, widths: &[u8; 256]) {
    let (line, _) = native::cursor_line(&view.draft, view.cursor, widths);
    if line < view.scroll {
        view.scroll = line;
    } else if line >= view.scroll + EDIT_LINES {
        view.scroll = line + 1 - EDIT_LINES;
    }
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
        cursor: 0,
        keyboard: kb::Keyboard::default(),
        repeat: kb::ButtonRepeat::default(),
        held: Default::default(),
        ticks: 0.0,
        over: false,
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
    time: Res<Time>,
    mut warning: ResMut<crate::warning_panel::WarningPanel>,
) {
    if !crate::house::is_player_room(&current, &state) {
        room.diary_ui = None;
    }
    let Some(mut view) = room.diary_ui.take() else {
        events.clear();
        return;
    };
    if std::mem::take(&mut view.over) {
        warning.open(&rustimal_logic::warning::WORD_OVER);
    }
    if warning.is_open() {
        events.clear();
        room.diary_ui = Some(view);
        return;
    }
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
        view.cursor,
        view.keyboard,
        view.held,
    );
    let mut close = false;
    match view.mode {
        Mode::Read => {
            events.clear();
            if actions.consume(Action::Cancel) || actions.consume(Action::Back) {
                close = true;
            } else if actions.consume(Action::Confirm) {
                begin_edit(&mut view, &font.widths);
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
            let widths = &font.widths;
            let ctrl = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
            // Key presses: A (Enter), the layout keys, Start (F4) and typed letters.
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
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        if let Some(code) = view.keyboard.selected() {
                            let v = &mut view;
                            type_code(v, code, widths);
                        }
                        continue;
                    }
                    KeyCode::Space => {
                        let v = &mut view;
                        type_code(v, kb::CHAR_SPACE, widths);
                        continue;
                    }
                    KeyCode::F1 => {
                        view.keyboard.toggle_shift();
                        crate::sound::se(TOGGLE);
                        continue;
                    }
                    KeyCode::F2 => {
                        view.keyboard.toggle_arrange();
                        crate::sound::se(TOGGLE);
                        continue;
                    }
                    KeyCode::F3 => {
                        view.keyboard.next_mode();
                        crate::sound::se(TOGGLE);
                        continue;
                    }
                    KeyCode::KeyX if ctrl => {
                        exchange(&mut view);
                        continue;
                    }
                    KeyCode::F4 => {
                        // Start ends the edit (`mED_COMMAND_END_EDIT`).
                        view.mode = Mode::Confirm;
                        view.choice = 0;
                        crate::sound::se(START);
                        break;
                    }
                    _ => {}
                }
                if ctrl {
                    continue;
                }
                if let Key::Character(chars) = &event.logical_key {
                    for ch in chars.chars().filter(|c| !c.is_control()) {
                        if let Some(code) = crate::messages::ac_code_opt(ch) {
                            let v = &mut view;
                            type_code(v, code, widths);
                        }
                    }
                }
            }
            if view.mode == Mode::Edit {
                // Held keys: the grid (arrows), the text cursor (Ctrl+arrows)
                // and Backspace, counted in 60 Hz frames.
                let arrow = direction(&keys);
                let (stick, pad) = if ctrl { (None, arrow) } else { (arrow, None) };
                let button = if keys.pressed(KeyCode::Backspace) {
                    Some(kb::Button::Backspace)
                } else {
                    pad.map(|d| match d {
                        kb::Direction::Left => kb::Button::CursorLeft,
                        kb::Direction::Right => kb::Button::CursorRight,
                        kb::Direction::Up => kb::Button::CursorUp,
                        kb::Direction::Down => kb::Button::CursorDown,
                    })
                };
                view.ticks += time.delta_secs() * 60.0;
                let mut steps = view.ticks.floor() as u32;
                view.ticks -= steps as f32;
                // A press never waits for the next whole frame.
                if steps == 0 && (stick.is_some() || button.is_some()) && view.held.stick != stick {
                    steps = 1;
                }
                for _ in 0..steps {
                    if view.keyboard.step(stick) == kb::Moved::Yes {
                        crate::sound::se(GRID_MOVE);
                    }
                    if view.repeat.step(button) {
                        if let Some(button) = button {
                            let v = &mut view;
                            let done = match button {
                                kb::Button::Backspace => {
                                    native::backspace(&mut v.draft, &mut v.cursor)
                                }
                                kb::Button::CursorLeft => native::left(&mut v.cursor),
                                kb::Button::CursorRight => native::right(
                                    &mut v.draft,
                                    &mut v.cursor,
                                    widths,
                                    native::DIARY,
                                ),
                                kb::Button::CursorUp => {
                                    native::upper(&v.draft, &mut v.cursor, widths)
                                }
                                kb::Button::CursorDown => native::lower(
                                    &mut v.draft,
                                    &mut v.cursor,
                                    widths,
                                    native::DIARY,
                                ),
                            };
                            crate::sound::se(if !done {
                                ERROR
                            } else if button == kb::Button::Backspace {
                                BACKSPACE
                            } else {
                                CURSOR
                            });
                        }
                    }
                }
                view.held = crate::shop_nook::editor::Held {
                    a: keys.pressed(KeyCode::Enter),
                    b: keys.pressed(KeyCode::Backspace),
                    l: keys.pressed(KeyCode::F1),
                    y: keys.pressed(KeyCode::F3),
                    r: keys.pressed(KeyCode::Space),
                    x: ctrl && keys.pressed(KeyCode::KeyX),
                    exchange: kb::exchange_code(&view.draft, view.cursor)
                        .map(|code| (view.draft[view.cursor - 1], code)),
                    pad,
                    stick,
                };
                follow_cursor(&mut view, widths);
            }
        }
        Mode::Confirm => {
            events.clear();
            if actions.consume(Action::Cancel) {
                begin_edit(&mut view, &font.widths);
            } else if actions.consume(Action::MoveUp) || actions.consume(Action::MoveDown) {
                view.choice ^= 1;
            } else if actions.consume(Action::Confirm) {
                if native::confirm_writes(view.choice) {
                    room.diary.get_or_insert_with(Notebook::default).months[view.month] =
                        native::entry(&view.draft);
                    close = true;
                } else {
                    begin_edit(&mut view, &font.widths);
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
                view.cursor,
                view.keyboard,
                view.held,
            )
        {
            view.revision = view.revision.wrapping_add(1);
            if view.mode != Mode::Edit && (view.mode != before.1 || view.month != before.0) {
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
