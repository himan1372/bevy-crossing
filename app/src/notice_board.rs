//! The town's notice board (`m_notice_ovl.c`): the bulletin board in the plaza shows the
//! posts, oldest first, one per page; A starts a new post in the letter editor
//! (`mED_TYPE_NOTICE`), and the end check ("post it?") writes it (`mNtc_notice_write`).
//! Posting finishes the part-time job's notice errand (`mNT_finish_notice_first_job`).
//!
//! The paper and keyboard are the diary's (the notice overlay's own window art is not
//! built); the editor, limits and page logic are the source's.

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use rustimal_logic::diary as native;
use rustimal_logic::editor_keyboard as kb;
use rustimal_logic::menu_slide::Slide;
use rustimal_logic::notice_board as board;
use rustimal_logic::notice_board::Motion;
use serde::{Deserialize, Serialize};

use crate::input::{Action, Actions, InputFocus};
use crate::interaction::Facing;
use crate::player::Player;

mod render;

/// One post (`mNtc_board_post_c`): the message codes and when it was written.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Notice {
    pub message: Vec<u8>,
    /// (year, month, day, hour, minute)
    pub time: (u16, u8, u8, u8, u8),
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Read,
    Edit,
    Confirm,
}

/// The overlay while it is up.
#[derive(Resource, Default)]
pub(crate) struct NoticeUi(Option<View>);

impl NoticeUi {
    pub(crate) fn open(&self) -> bool {
        self.0.is_some()
    }
}

struct View {
    page: usize,
    mode: Mode,
    draft: Vec<u8>,
    choice: usize,
    just_opened: bool,
    revision: u64,
    cursor: usize,
    scroll: usize,
    keyboard: kb::Keyboard,
    repeat: kb::ButtonRepeat,
    held: crate::shop_nook::editor::Held,
    ticks: f32,
    over: bool,
    /// `mSM_move_Move`: the board comes in from the top and leaves upward.
    slide: Slide,
    /// `mNT_Play_page_move` and friends: the paper's sideways motion.
    motion: Motion,
    anim_ticks: f32,
}

// `sAdo_SysTrgStart` ids from `m_editor_ovl.c`.
const GRID_MOVE: u16 = 0x411;
const CURSOR: u16 = 0x35;
const BACKSPACE: u16 = 0x1034;
const START: u16 = 0x5F;
const TOGGLE: u16 = 0x37;
const ERROR: u16 = 0x1003;
/// The paper shows five lines above the keyboard while editing.
const EDIT_LINES: usize = 5;
const LIMITS: native::Limits = native::LETTER;

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

fn type_code(view: &mut View, code: u8, widths: &[u8; 256]) {
    if native::insert(&mut view.draft, &mut view.cursor, code, widths, LIMITS) {
        crate::shop_nook::editor::speak(code, false);
    } else {
        view.over = true;
        crate::sound::se(ERROR);
    }
}

fn begin_edit(view: &mut View, widths: &[u8; 256]) {
    view.mode = Mode::Edit;
    view.cursor = view.draft.len();
    view.repeat = kb::ButtonRepeat::default();
    follow_cursor(view, widths);
}

fn follow_cursor(view: &mut View, widths: &[u8; 256]) {
    let (line, _) = native::cursor_line(&view.draft, view.cursor, widths);
    if line < view.scroll {
        view.scroll = line;
    } else if line >= view.scroll + EDIT_LINES {
        view.scroll = line + 1 - EDIT_LINES;
    }
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

/// Facing a bulletin board (field items 0x0007 / 0x000B) in the plaza: the action button opens it.
#[allow(clippy::too_many_arguments)]
fn open_board(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    player: Query<(&Transform, &Facing), With<Player>>,
    items: Option<Res<crate::field_items::FieldItems>>,
    house: Res<State<crate::house::HouseState>>,
    shop: Res<State<crate::shop::ShopLocation>>,
    tool: Res<crate::ecology::EquippedTool>,
    post: Res<crate::post::Post>,
    mut ui: ResMut<NoticeUi>,
) {
    if ui.0.is_some()
        || !focus.is_world()
        || *house.get() != crate::house::HouseState::Town
        || *shop.get() != crate::shop::ShopLocation::Town
        || *tool != crate::ecology::EquippedTool::None
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let (Some(items), Ok((at, facing))) = (items, player.single()) else {
        return;
    };
    let ahead = at.translation + facing.dir * 0.6;
    let Some((ax, az, cx, cz)) = crate::town::Town::cell_indices_of(ahead) else {
        return;
    };
    let item = items
        .acres
        .get(az * rustimal_logic::town_gen::ACRE_WIDTH + ax)
        .map(|acre| acre[cz * 16 + cx]);
    if !matches!(item, Some(0x0007 | 0x000B)) {
        return;
    }
    actions.consume(Action::Interact);
    ui.0 = Some(View {
        page: board::first_page(post.notices.len()),
        mode: Mode::Read,
        draft: Vec::new(),
        choice: 0,
        just_opened: true,
        revision: 0,
        cursor: 0,
        scroll: 0,
        keyboard: kb::Keyboard::default(),
        repeat: kb::ButtonRepeat::default(),
        held: Default::default(),
        ticks: 0.0,
        over: false,
        slide: Slide::new(),
        motion: Motion::new(board::first_page(post.notices.len())),
        anim_ticks: 0.0,
    });
    crate::sound::se(0x5F);
}

#[allow(clippy::too_many_arguments)]
fn input(
    mut ui: ResMut<NoticeUi>,
    mut actions: ResMut<Actions>,
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut focus: ResMut<InputFocus>,
    font: Option<Res<crate::msg_window::MsgAssets>>,
    time: Res<Time>,
    mut warning: ResMut<crate::warning_panel::WarningPanel>,
    mut post: ResMut<crate::post::Post>,
    mut identity: ResMut<crate::save::PlayerIdentity>,
) {
    let Some(mut view) = ui.0.take() else {
        events.clear();
        return;
    };
    let frames = time.delta_secs() * 60.0;
    if view.slide.advance(frames) {
        // The closing slide has finished: the world has the input again.
        events.clear();
        *focus = InputFocus::World;
        return;
    }
    view.anim_ticks += frames;
    while view.anim_ticks >= 1.0 {
        view.anim_ticks -= 1.0;
        view.motion.tick(post.notices.len());
    }
    if std::mem::take(&mut view.over) {
        warning.open(&rustimal_logic::warning::WORD_OVER);
    }
    if warning.is_open() || *focus != InputFocus::Notice || view.just_opened {
        events.clear();
        view.just_opened = false;
        ui.0 = Some(view);
        return;
    }
    let Some(font) = font else {
        events.clear();
        ui.0 = Some(view);
        return;
    };
    // Nothing is read while the board slides or a page turns (`mNT_Play_*` take no buttons).
    if !view.slide.settled() || !view.motion.idle() {
        events.clear();
        ui.0 = Some(view);
        return;
    }
    let before = (
        view.page,
        view.mode,
        view.scroll,
        view.choice,
        view.draft.clone(),
        view.cursor,
        view.keyboard,
        view.held,
    );
    let count = post.notices.len();
    let mut close = false;
    match view.mode {
        Mode::Read => {
            events.clear();
            let step = if actions.consume(Action::MoveLeft) {
                Some(board::Move::Older)
            } else if actions.consume(Action::MoveRight) {
                Some(board::Move::Newer)
            } else if actions.consume(Action::MoveDown) {
                Some(board::Move::First)
            } else if actions.consume(Action::MoveUp) {
                Some(board::Move::Last)
            } else {
                None
            };
            if let Some(page) = step.and_then(|s| board::move_page(view.page, count, s)) {
                view.motion.start_move(page as isize - view.page as isize);
                view.page = page;
            } else if actions.consume(Action::Cancel) || actions.consume(Action::Back) {
                close = true;
            } else if actions.consume(Action::Confirm) {
                // `mNT_Play_page_read`: A starts a new post on an empty page.
                view.draft.clear();
                view.scroll = 0;
                crate::sound::se(START);
                view.motion.start_write();
                begin_edit(&mut view, &font.widths);
            }
        }
        Mode::Edit => {
            let widths = &font.widths;
            let ctrl = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
            for event in events.read() {
                if event.state != ButtonState::Pressed {
                    continue;
                }
                match event.key_code {
                    KeyCode::Escape => {
                        view.mode = Mode::Read;
                        view.motion.start_read(view.page);
                        break;
                    }
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        if let Some(code) = view.keyboard.selected() {
                            type_code(&mut view, code, widths);
                        }
                        continue;
                    }
                    KeyCode::Space => {
                        type_code(&mut view, kb::CHAR_SPACE, widths);
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
                        if let Some(code) = kb::exchange_code(&view.draft, view.cursor) {
                            kb::exchange(&mut view.draft, view.cursor);
                            crate::shop_nook::editor::speak(code, true);
                        } else {
                            crate::sound::se(ERROR);
                        }
                        continue;
                    }
                    KeyCode::F4 => {
                        // Start ends the edit (`mED_COMMAND_END_EDIT`): the end check.
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
                            type_code(&mut view, code, widths);
                        }
                    }
                }
            }
            if view.mode == Mode::Edit {
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
                                kb::Button::CursorRight => {
                                    native::right(&mut v.draft, &mut v.cursor, widths, LIMITS)
                                }
                                kb::Button::CursorUp => {
                                    native::upper(&v.draft, &mut v.cursor, widths)
                                }
                                kb::Button::CursorDown => {
                                    native::lower(&mut v.draft, &mut v.cursor, widths, LIMITS)
                                }
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
            } else if actions.consume(Action::MoveUp) {
                view.choice = (view.choice + 2) % 3;
            } else if actions.consume(Action::MoveDown) {
                view.choice = (view.choice + 1) % 3;
            } else if actions.consume(Action::Confirm) {
                match board::end_choice(view.choice) {
                    board::EndChoice::Post => {
                        let now = crate::clock::now();
                        board::write(
                            &mut post.notices,
                            Notice {
                                message: native::entry(&view.draft),
                                time: (now.year, now.month, now.day, now.hour, now.minute),
                            },
                        );
                        // `mNT_finish_notice_first_job`.
                        if identity.events.first_job {
                            if let Some(job) = identity.first_job.as_mut() {
                                if job.job == Some(rustimal_logic::first_job::Job::PostNotice)
                                    && job.progress != 0
                                {
                                    job.finish();
                                    info!("first job progress 2 -> 0 (notice posted)");
                                }
                            }
                        }
                        close = true;
                    }
                    board::EndChoice::KeepWriting => begin_edit(&mut view, &font.widths),
                    board::EndChoice::Cancel => {
                        view.mode = Mode::Read;
                        view.page = board::first_page(count + 1).min(count.saturating_sub(1));
                        view.motion.start_read(view.page);
                    }
                }
            }
        }
    }
    if close {
        // `mSM_MOVE_OUT_TOP`: the board leaves upward; the world waits for it.
        view.slide.close();
        crate::sound::se(crate::sound::se_id::MENU_EXIT);
        ui.0 = Some(view);
    } else {
        if before
            != (
                view.page,
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
            if view.mode != Mode::Edit && (view.mode != before.1 || view.page != before.0) {
                crate::sound::se(crate::sound::se_id::CURSOR);
            }
        }
        ui.0 = Some(view);
    }
}

pub struct NoticeBoardPlugin;

impl Plugin for NoticeBoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NoticeUi>()
            .init_resource::<render::NoticeAssets>()
            .add_systems(
                Update,
                (
                    open_board.in_set(crate::input::InteractSet::Ui),
                    input.in_set(crate::input::InteractSet::Ui),
                )
                    .in_set(crate::states::GameplaySet),
            )
            .add_systems(
                Update,
                (render::draw, render::animate)
                    .chain()
                    .after(crate::input::InteractSet::Ui)
                    .in_set(crate::states::GameplaySet)
                    .run_if(resource_exists::<crate::msg_window::MsgAssets>),
            );
    }
}
