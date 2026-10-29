//! The name window of `m_ledit_ovl.c` (`mLE_TYPE_PLAYER_NAME`, `mLE_TYPE_COUNTRY_NAME`): a titled
//! window that slides in from the top with the editor's keyboard coming up from the bottom, and a
//! single-line field of eight characters. Ending the edit is refused for an empty name and for the
//! disc's list of banned words (`mED_all_space_check`, `mED_ng_word_check`).
use bevy::camera::visibility::RenderLayers;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use rustimal_logic::editor_keyboard as kb;
use rustimal_logic::menu_slide::Slide;

use crate::game_disc::GameDisc;
use crate::msg_window::MsgAssets;
use crate::n64_material::N64Material;
use crate::pockets_disc::Paint;

/// Which name the window asks for (`mLE_TYPE_*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Player,
    Country,
}

/// `mLE_win_data`: the window's art, title, where the field sits and its ink.
struct Window {
    mode: &'static str,
    model: &'static str,
    title: &'static str,
    title_at: Vec2,
    edit_at: Vec2,
    ink: [u8; 4],
}

impl Kind {
    fn window(self) -> Window {
        match self {
            Kind::Player => Window {
                mode: "nam_win_mode",
                model: "nam_win_model",
                title: "Enter your name.",
                title_at: Vec2::new(120.0, 72.0),
                edit_at: Vec2::new(132.0, 45.0),
                ink: [0, 0, 255, 255],
            },
            Kind::Country => Window {
                mode: "mra_win_mode",
                model: "mra_win_model",
                title: "Enter Destination.",
                title_at: Vec2::new(118.0, 74.0),
                edit_at: Vec2::new(134.0, 43.0),
                ink: [215, 15, 0, 255],
            },
        }
    }
}

/// `mED_NG_WORD_START` and the lengths of its `mED_NG_WORD_NUM` words.
const NG_WORD_START: usize = 0x74A;
const NG_WORD_LENGTHS: [usize; 14] = [4, 4, 4, 4, 4, 5, 5, 7, 7, 7, 7, 7, 9, 10];
/// `edit_scale` of both windows (28/32 of the font).
const EDIT_SCALE: f32 = 0.875;

// `sAdo_SysTrgStart` ids from `m_editor_ovl.c`.
const GRID_MOVE: u16 = 0x411;
const CURSOR: u16 = 0x35;
const BACKSPACE: u16 = 0x1034;
const START: u16 = 0x5F;
const TOGGLE: u16 = 0x37;
const ERROR: u16 = 0x1003;

struct View {
    kind: Kind,
    field: kb::Field<8>,
    keyboard: kb::Keyboard,
    repeat: kb::ButtonRepeat,
    held: crate::shop_nook::editor::Held,
    ticks: f32,
    /// `mSM_MOVE_IN_TOP` / `OUT_TOP`: the window.
    slide: Slide,
    /// `mSM_MOVE_IN_BOTTOM` / `OUT_BOTTOM`: the keyboard.
    keys: Slide,
    /// The name, once it has been accepted; the window is leaving.
    accepted: Option<[u8; 8]>,
    just_opened: bool,
    revision: u64,
}

/// The window while it is up.
#[derive(Resource, Default)]
pub(crate) struct NameEntry {
    view: Option<View>,
    finished: Option<[u8; 8]>,
}

impl NameEntry {
    pub(crate) fn open(&self) -> bool {
        self.view.is_some()
    }

    /// Opens the window for `kind` (`mSM_open_submenu(.., mSM_OVL_LEDIT, ..)`): the field starts
    /// empty.
    pub(crate) fn begin(&mut self, kind: Kind) {
        self.finished = None;
        self.view = Some(View {
            kind,
            field: kb::Field::default(),
            keyboard: kb::Keyboard::default(),
            repeat: kb::ButtonRepeat::default(),
            held: Default::default(),
            ticks: 0.0,
            slide: Slide::new(),
            keys: Slide::new(),
            accepted: None,
            just_opened: true,
            revision: 0,
        });
    }

    /// The name, once the window has left.
    pub(crate) fn take(&mut self) -> Option<[u8; 8]> {
        self.finished.take()
    }
}

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

/// `mED_all_space_check` / `mED_ng_word_check`: whether the name may be kept.
fn acceptable(field: &kb::Field<8>, disc: &GameDisc) -> bool {
    if field.text[..field.len].iter().all(|&c| c == kb::CHAR_SPACE) {
        return false;
    }
    let Some(text) = disc.text.as_ref() else {
        return true;
    };
    for (i, len) in NG_WORD_LENGTHS.iter().enumerate() {
        let Some(word) = text.strings.get(NG_WORD_START + i) else {
            continue;
        };
        let word = &word[..(*len).min(word.len())];
        if !word.is_empty() && field.text.windows(word.len()).any(|w| w == word) {
            return false;
        }
    }
    true
}

fn type_code(view: &mut View, code: u8) {
    if view.field.insert(code) {
        crate::shop_nook::editor::speak(code, false);
    } else {
        crate::sound::se(ERROR);
    }
}

#[allow(clippy::too_many_arguments)]
fn input(
    mut names: ResMut<NameEntry>,
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut focus: ResMut<crate::input::InputFocus>,
    time: Res<Time>,
    disc: Option<Res<GameDisc>>,
) {
    let Some(mut view) = names.view.take() else {
        events.clear();
        return;
    };
    let frames = time.delta_secs() * 60.0;
    if let Some(name) = view.accepted {
        // The window and keyboard are leaving; the world hears about it when both are gone.
        let window_gone = view.slide.advance(frames);
        let keys_gone = view.keys.advance(frames);
        events.clear();
        if window_gone && keys_gone {
            names.finished = Some(name);
            *focus = crate::input::InputFocus::Dialogue;
        } else {
            names.view = Some(view);
        }
        return;
    }
    view.slide.advance(frames);
    view.keys.advance(frames);
    let (Some(disc), true) = (disc, *focus == crate::input::InputFocus::NameEntry) else {
        events.clear();
        names.view = Some(view);
        return;
    };
    if std::mem::take(&mut view.just_opened) || !view.slide.settled() || !view.keys.settled() {
        events.clear();
        names.view = Some(view);
        return;
    }
    let ctrl = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let before = (view.field, view.keyboard, view.held);
    for event in events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match event.key_code {
            KeyCode::Enter | KeyCode::NumpadEnter => {
                if let Some(code) = view.keyboard.selected() {
                    type_code(&mut view, code);
                }
                continue;
            }
            KeyCode::Space => {
                type_code(&mut view, kb::CHAR_SPACE);
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
                let cursor = view.field.cursor;
                if let Some(code) = kb::exchange_code(&view.field.text, cursor) {
                    kb::exchange(&mut view.field.text, cursor);
                    crate::shop_nook::editor::speak(code, true);
                } else {
                    crate::sound::se(ERROR);
                }
                continue;
            }
            KeyCode::F4 => {
                // `mED_COMMAND_END_EDIT`.
                if acceptable(&view.field, &disc) {
                    crate::sound::se(START);
                    view.accepted = Some(view.field.text);
                    view.slide.close();
                    view.keys.close();
                } else {
                    // The refused name is wiped (`mED_clear_input_data`).
                    view.field = kb::Field::default();
                    crate::sound::se(ERROR);
                }
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
                    type_code(&mut view, code);
                }
            }
        }
    }
    if view.accepted.is_none() {
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
        view.ticks += frames;
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
                    let done = match button {
                        kb::Button::Backspace => view.field.backspace(),
                        kb::Button::CursorLeft => view.field.left(),
                        kb::Button::CursorRight => view.field.right(),
                        // A single line: up and down do nothing.
                        kb::Button::CursorUp | kb::Button::CursorDown => false,
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
        let cursor = view.field.cursor;
        view.held = crate::shop_nook::editor::Held {
            a: keys.pressed(KeyCode::Enter),
            b: keys.pressed(KeyCode::Backspace),
            l: keys.pressed(KeyCode::F1),
            y: keys.pressed(KeyCode::F3),
            r: keys.pressed(KeyCode::Space),
            x: ctrl && keys.pressed(KeyCode::KeyX),
            exchange: if cursor > 0 {
                kb::exchange_code(&view.field.text, cursor)
                    .map(|code| (view.field.text[cursor - 1], code))
            } else {
                None
            },
            pad,
            stick,
        };
    }
    if before != (view.field, view.keyboard, view.held) {
        view.revision = view.revision.wrapping_add(1);
    }
    names.view = Some(view);
}

type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;

/// The window's cloth creeps half a texel for every 0.3535 of `texture_pos` (`mLE_set_dl`: -pos * 2
/// quarter texels).
fn creep(
    time: Res<Time>,
    mut assets: ResMut<NameAssets>,
    mut n64: ResMut<Assets<N64Material>>,
    images: Res<Assets<Image>>,
) {
    assets.slide = (assets.slide + 0.3535 * 0.5 * time.delta_secs() * 60.0).rem_euclid(1024.0);
    for model in assets.windows.iter().flatten() {
        crate::time_window::scroll_cloth(model, assets.slide, &mut n64, &images);
    }
}

#[derive(Resource, Default)]
pub(crate) struct NameAssets {
    slide: f32,
    windows: [Option<Model>; 2],
    root: Option<Entity>,
    shown: Option<(Kind, u64, bool)>,
    skin: crate::shop_nook::editor::Skin,
}

#[derive(Component)]
struct WindowRoot;

#[derive(Component)]
struct KeyboardGroup;

/// The width in pixels of the field's first `chars` characters.
fn width_before(field: &kb::Field<8>, chars: usize, widths: &[u8; 256]) -> f32 {
    field.text[..chars.min(8)]
        .iter()
        .map(|&c| f32::from(widths[usize::from(c)]))
        .sum()
}

#[allow(clippy::too_many_arguments)]
fn draw(
    mut commands: Commands,
    names: Res<NameEntry>,
    disc: Option<Res<GameDisc>>,
    font: Option<Res<MsgAssets>>,
    mut assets: ResMut<NameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut n64: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(view) = names.view.as_ref() else {
        if let Some(root) = assets.root.take() {
            commands.entity(root).despawn();
        }
        assets.shown = None;
        return;
    };
    let (Some(disc), Some(font)) = (disc.as_deref(), font.as_deref()) else {
        return;
    };
    let signature = (view.kind, view.revision, view.accepted.is_some());
    if assets.shown == Some(signature) && assets.root.is_some() {
        return;
    }
    let slot = usize::from(view.kind == Kind::Country);
    if assets.windows[slot].is_none() {
        let window = view.kind.window();
        let lists: Vec<_> = ["ledit_common_mode", window.mode, window.model]
            .iter()
            .filter_map(|n| disc.catalog.get(n).map(|s| s.offset))
            .collect();
        let sink = crate::disc_gfx::run_display_lists(&disc.rom, &lists);
        assets.windows[slot] = Some(crate::disc_town::sink_meshes_scaled(
            disc,
            &sink,
            None,
            true,
            &crate::disc_gfx::Segments::default(),
            16.0,
            (&mut meshes, &mut n64, &mut images),
        ));
    }
    if let Some(root) = assets.root.take() {
        commands.entity(root).despawn();
    }
    let window = view.kind.window();
    let root = commands
        .spawn((
            Name::new("name entry"),
            WindowRoot,
            Transform::from_xyz(0.0, view.slide.pos * 16.0, 0.0),
            Visibility::default(),
        ))
        .id();
    assets.root = Some(root);
    assets.shown = Some(signature);
    if let Some(model) = &assets.windows[slot] {
        for (i, (mesh, material)) in model.iter().enumerate() {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.0, 0.0, 200.0 + i as f32 * 0.1),
                RenderLayers::layer(1),
                ChildOf(root),
            ));
        }
    }
    let scaled = |text: String| {
        format!(
            "{}{}{}",
            crate::messages::markup::LINE_SCALE,
            crate::messages::markup::arg((EDIT_SCALE * 32.0) as u8),
            text
        )
    };
    let shown: String = view.field.text[..view.field.len]
        .iter()
        .map(|&c| crate::messages::char_of(c).unwrap_or('?'))
        .collect();
    let at = Vec2::new(
        window.edit_at.x + width_before(&view.field, view.field.cursor, &font.widths) * EDIT_SCALE,
        window.edit_at.y,
    );
    {
        let mut paint = Paint {
            commands: &mut commands,
            meshes: &mut meshes,
            materials: &mut materials,
            root,
        };
        paint.text(
            font,
            window.title,
            window.title_at,
            [255, 255, 255, 255],
            230.0,
        );
        paint.text(font, &scaled(shown), window.edit_at, window.ink, 230.0);
        if let Some(caret) = paint.text(font, "_", at, crate::shop_nook::editor::CARET_INK, 230.0) {
            paint
                .commands
                .entity(caret)
                .insert(crate::shop_nook::editor::Caret(at));
        }
    }
    let keyboard = commands
        .spawn((
            KeyboardGroup,
            Transform::from_xyz(0.0, -(view.slide.pos + view.keys.pos) * 16.0, 0.0),
            Visibility::default(),
            ChildOf(root),
        ))
        .id();
    let mut stores = crate::shop_nook::editor::Stores {
        meshes: &mut meshes,
        n64: &mut n64,
        images: &mut images,
        materials: &mut materials,
    };
    crate::shop_nook::editor::spawn(
        &mut commands,
        keyboard,
        &mut assets.skin,
        disc,
        font,
        &view.keyboard,
        view.held,
        &mut stores,
    );
}

/// Slides the window and the keyboard (`menu_info->position[1]`; one pixel is 16 units).
fn animate(
    names: Res<NameEntry>,
    mut roots: Query<&mut Transform, (With<WindowRoot>, Without<KeyboardGroup>)>,
    mut groups: Query<&mut Transform, (With<KeyboardGroup>, Without<WindowRoot>)>,
) {
    let Some(view) = names.view.as_ref() else {
        return;
    };
    for mut transform in &mut roots {
        transform.translation.y = view.slide.pos * 16.0;
    }
    for mut transform in &mut groups {
        // The keyboard is a child of the window: undo the window's slide, then add its own.
        transform.translation.y = -(view.slide.pos + view.keys.pos) * 16.0;
    }
}

pub struct NameEntryPlugin;

impl Plugin for NameEntryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NameEntry>()
            .init_resource::<NameAssets>()
            .add_systems(
                Update,
                input
                    .in_set(crate::input::InteractSet::Ui)
                    .in_set(crate::states::GameplaySet),
            )
            .add_systems(
                Update,
                (draw, creep, animate)
                    .chain()
                    .after(crate::input::InteractSet::Ui)
                    .in_set(crate::states::GameplaySet)
                    .run_if(resource_exists::<MsgAssets>),
            );
    }
}
