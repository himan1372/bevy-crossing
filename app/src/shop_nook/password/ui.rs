//! The counter's two entry windows over the retail editor grid: Say code
//! (`m_passwordChk_ovl`, two fourteen-letter lines) and the receiver's town and
//! player name (`m_passwordMake_ovl`). The grid, its cursor and the button
//! repeat are `rustimal_logic::editor_keyboard`; the panels and keys are the
//! disc's models (`super::super::editor`). Typed letters still work, as the
//! PC port's keyboard typing does.
use bevy::camera::visibility::RenderLayers;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use rustimal_logic::editor_keyboard as kb;
use rustimal_logic::password as pw;

use super::super::editor::{self, Held, Skin, Stores};
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus};
use crate::msg_window::{quad_mesh, text_width, MsgAssets};
use crate::n64_material::N64Material;
use crate::pockets_disc::Paint;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Mode {
    /// The 28-letter code to redeem.
    #[default]
    Code,
    /// Who a new code is for: town, then player.
    Receiver,
}

/// A receiver's town and player name, space padded to eight bytes each.
pub(in crate::shop_nook) type Receiver = ([u8; 8], [u8; 8]);

/// The entry window and what the counter remembers between questions.
#[derive(Resource)]
pub(crate) struct PasswordUi {
    pub open: bool,
    mode: Mode,
    /// Set when the receiver window closes: the names, or `None` when
    /// the player backed out.
    pub receiver: Option<Option<Receiver>>,
    town: kb::Field<8>,
    name: kb::Field<8>,
    field: usize,
    lines: [[u8; 14]; 2],
    selected: usize,
    cursor: usize,
    /// Set when the window closes: all 28 characters, spaces when unfinished.
    pub done: Option<[u8; 28]>,
    /// `shop_common->password_str`, kept so a retry starts from it.
    text: [u8; 28],
    /// Presents handed over since the game started.
    pub presents: u32,
    keyboard: kb::Keyboard,
    repeat: kb::ButtonRepeat,
    held: Held,
    ticks: f32,
    rng: u32,
    just_opened: bool,
    revision: u64,
}

impl Default for PasswordUi {
    fn default() -> Self {
        Self {
            open: false,
            mode: Mode::Code,
            receiver: None,
            town: kb::Field::default(),
            name: kb::Field::default(),
            field: 0,
            lines: [[pw::PW_CHAR_SPACE; 14]; 2],
            selected: 0,
            cursor: 0,
            done: None,
            text: [pw::PW_CHAR_SPACE; 28],
            presents: 0,
            keyboard: kb::Keyboard::default(),
            repeat: kb::ButtonRepeat::default(),
            held: Held::default(),
            ticks: 0.0,
            rng: 0,
            just_opened: false,
            revision: 0,
        }
    }
}

impl PasswordUi {
    /// `RANDOM_F(100.0)` for a magazine code. The port keeps its own
    /// generator, not the original's global state.
    pub(in crate::shop_nook) fn next_roll(&mut self) -> f32 {
        if self.rng == 0 {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(1, |d| d.subsec_nanos());
            self.rng = nanos | 1;
        }
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng >> 8) as f32 / (1u32 << 24) as f32 * 100.0
    }

    /// `aNSC_pc_input_pw_start_wait_init`: a fresh question clears the text.
    pub(in crate::shop_nook) fn clear(&mut self) {
        self.text = [pw::PW_CHAR_SPACE; 28];
    }

    fn reopen(&mut self, mode: Mode) {
        self.mode = mode;
        self.keyboard = kb::Keyboard::default();
        self.repeat = kb::ButtonRepeat::default();
        self.held = Held::default();
        self.open = true;
        self.just_opened = true;
        self.revision += 1;
    }

    /// `mSM_OVL_PASSWORDMAKE`: empty town and player name fields.
    pub(in crate::shop_nook) fn begin_receiver(&mut self) {
        self.town = kb::Field::default();
        self.name = kb::Field::default();
        self.field = 0;
        self.receiver = None;
        self.reopen(Mode::Receiver);
    }

    fn finish_receiver(&mut self, accepted: bool) {
        self.receiver = Some(accepted.then_some((self.town.text, self.name.text)));
        self.open = false;
        self.revision += 1;
    }

    /// `mSM_OVL_PASSWORDCHK`: both lines start from the remembered text.
    pub(in crate::shop_nook) fn begin(&mut self) {
        self.lines[0].copy_from_slice(&self.text[..14]);
        self.lines[1].copy_from_slice(&self.text[14..]);
        let filled = self
            .text
            .iter()
            .position(|&c| c == pw::PW_CHAR_SPACE)
            .unwrap_or(28);
        self.selected = usize::from(filled >= 14);
        self.cursor = if filled == 28 { 14 } else { filled % 14 };
        self.done = None;
        self.reopen(Mode::Code);
    }

    fn finish(&mut self) {
        self.text[..14].copy_from_slice(&self.lines[0]);
        self.text[14..].copy_from_slice(&self.lines[1]);
        self.done = Some(self.text);
        self.open = false;
        self.revision += 1;
    }

    /// Closes the window without a result, as leaving the scene does.
    pub(in crate::shop_nook) fn close(&mut self) {
        if self.open || self.done.is_some() || self.receiver.is_some() {
            self.open = false;
            self.done = None;
            self.receiver = None;
            self.revision += 1;
        }
    }

    /// A key chosen on the grid, or typed (`mED_input_pw_chk_line` /
    /// `mED_input_pw_make_line`). False plays the error sound.
    fn put(&mut self, code: u8) -> bool {
        match self.mode {
            Mode::Code => {
                if code == pw::PW_CHAR_NEW_LINE {
                    return false;
                }
                let selected = self.selected;
                let mut cursor = self.cursor;
                let result = pw::line_insert(&mut self.lines[selected], &mut cursor, code);
                self.cursor = cursor;
                if result == pw::LineInsert::LineFull && selected == 0 {
                    self.selected = 1;
                    self.cursor = 0;
                }
                result != pw::LineInsert::Rejected
            }
            Mode::Receiver => self.field_mut().insert(code),
        }
    }

    fn field_mut(&mut self) -> &mut kb::Field<8> {
        if self.field == 0 {
            &mut self.town
        } else {
            &mut self.name
        }
    }

    /// B: erase (`mED_backspace_func*`). In the receiver window, at the start
    /// of the player name it steps back to the town.
    fn erase(&mut self) -> bool {
        match self.mode {
            Mode::Code => {
                let before = (self.lines, self.selected, self.cursor);
                let (first, second) = self.lines.split_at_mut(1);
                pw::backspace(
                    &mut first[0],
                    &mut second[0],
                    &mut self.selected,
                    &mut self.cursor,
                );
                before != (self.lines, self.selected, self.cursor)
            }
            Mode::Receiver => {
                if self.field_mut().backspace() {
                    true
                } else if self.field == 1 {
                    self.field = 0;
                    let len = self.town.len;
                    self.town.cursor = len;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// D-pad moves of the text cursor and line.
    fn pad(&mut self, button: kb::Button) -> bool {
        match (self.mode, button) {
            (Mode::Code, kb::Button::CursorLeft) => pw::cursor_left(&mut self.cursor),
            (Mode::Code, kb::Button::CursorRight) => pw::cursor_right(&mut self.cursor),
            (Mode::Code, kb::Button::CursorUp) => pw::change_line(&mut self.selected, false),
            (Mode::Code, kb::Button::CursorDown) => pw::change_line(&mut self.selected, true),
            (Mode::Receiver, kb::Button::CursorLeft) => self.field_mut().left(),
            (Mode::Receiver, kb::Button::CursorRight) => self.field_mut().right(),
            (Mode::Receiver, kb::Button::CursorUp) => {
                let moved = self.field == 1;
                self.field = 0;
                if moved {
                    let len = self.town.len;
                    self.town.cursor = self.town.cursor.min(len);
                }
                moved
            }
            (Mode::Receiver, kb::Button::CursorDown) => {
                // `mED_move_pw_make_table`: the town must not be empty.
                if self.field == 0 && !self.town.is_empty() {
                    self.field = 1;
                    true
                } else {
                    false
                }
            }
            (_, kb::Button::Backspace) => self.erase(),
        }
    }

    /// Start (`mED_COMMAND_END_EDIT`). False plays the error sound.
    fn start(&mut self) -> bool {
        match self.mode {
            Mode::Code => {
                // The charset gate runs once the last character is filled.
                if pw::needs_charset_check(&self.lines[1])
                    && !pw::all_usable(&self.lines[0], &self.lines[1])
                {
                    return false;
                }
                self.finish();
                true
            }
            Mode::Receiver => {
                if self.field == 0 {
                    self.pad(kb::Button::CursorDown)
                } else if self.name.is_empty() {
                    false
                } else {
                    self.finish_receiver(true);
                    true
                }
            }
        }
    }
}

/// The button a held key stands for, and the grid direction.
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

/// Drives the grid and the text from the keyboard.
#[allow(clippy::too_many_arguments)]
pub(crate) fn input(
    mut ui: ResMut<PasswordUi>,
    mut actions: ResMut<Actions>,
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    focus: Res<InputFocus>,
) {
    if !ui.open || *focus != InputFocus::Password {
        events.clear();
        return;
    }
    if ui.just_opened {
        events.clear();
        ui.just_opened = false;
        return;
    }
    let before = (
        ui.keyboard,
        ui.held,
        ui.lines,
        ui.selected,
        ui.cursor,
        ui.field,
    );
    let before_names = (ui.town, ui.name);

    // Key presses: A (Enter), the layout keys, Start (F4) and typed letters.
    for event in events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match event.key_code {
            KeyCode::Enter | KeyCode::NumpadEnter => {
                if let Some(code) = ui.keyboard.selected() {
                    if ui.put(code) {
                        editor::speak(code, false);
                    } else {
                        crate::sound::se(ERROR);
                    }
                }
                continue;
            }
            KeyCode::Space => {
                if ui.put(kb::CHAR_SPACE) {
                    editor::speak(kb::CHAR_SPACE, false);
                } else {
                    crate::sound::se(ERROR);
                }
                continue;
            }
            KeyCode::F1 => {
                ui.keyboard.toggle_shift();
                crate::sound::se(TOGGLE);
                continue;
            }
            KeyCode::F2 => {
                ui.keyboard.toggle_arrange();
                crate::sound::se(TOGGLE);
                continue;
            }
            KeyCode::F3 => {
                ui.keyboard.next_mode();
                crate::sound::se(TOGGLE);
                continue;
            }
            KeyCode::F4 => {
                if ui.start() {
                    crate::sound::se(START);
                    break;
                }
                crate::sound::se(ERROR);
                continue;
            }
            KeyCode::Escape => {
                // Backing out: the code window closes with what is typed
                // (a declined code), the receiver window goes back.
                match ui.mode {
                    Mode::Code => ui.finish(),
                    Mode::Receiver => ui.finish_receiver(false),
                }
                break;
            }
            _ => {}
        }
        if let Key::Character(text) = &event.logical_key {
            for ch in text.chars() {
                let code = match ui.mode {
                    Mode::Code => typed_code(ch),
                    Mode::Receiver => (ch.is_ascii_graphic())
                        .then(|| crate::messages::ac_code_opt(ch))
                        .flatten(),
                };
                if let Some(code) = code {
                    if ui.put(code) {
                        editor::speak(code, false);
                    } else {
                        crate::sound::se(ERROR);
                    }
                }
            }
        }
    }
    if !ui.open {
        consume(&mut actions);
        return;
    }

    // Held keys: the grid cursor (arrows) and the editing buttons
    // (Backspace, Ctrl+arrows), counted in 60 Hz frames.
    let ctrl = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
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
    ui.ticks += time.delta_secs() * 60.0;
    let mut steps = ui.ticks.floor() as u32;
    ui.ticks -= steps as f32;
    // A press never waits for the next whole frame.
    if steps == 0 && (stick.is_some() || button.is_some()) && ui.held.stick != stick {
        steps = 1;
    }
    for _ in 0..steps {
        match ui.keyboard.step(stick) {
            kb::Moved::Yes => crate::sound::se(GRID_MOVE),
            kb::Moved::Edge | kb::Moved::No => {}
        }
        if ui.repeat.step(button) {
            if let Some(button) = button {
                if ui.pad(button) {
                    crate::sound::se(if button == kb::Button::Backspace {
                        BACKSPACE
                    } else {
                        CURSOR
                    });
                } else {
                    crate::sound::se(ERROR);
                }
            }
        }
    }
    ui.held = Held {
        a: keys.pressed(KeyCode::Enter),
        b: keys.pressed(KeyCode::Backspace),
        l: keys.pressed(KeyCode::F1),
        y: keys.pressed(KeyCode::F3),
        r: keys.pressed(KeyCode::Space),
        x: false,
        exchange: None,
        pad,
        stick,
    };
    if before
        != (
            ui.keyboard,
            ui.held,
            ui.lines,
            ui.selected,
            ui.cursor,
            ui.field,
        )
        || before_names != (ui.town, ui.name)
    {
        ui.revision += 1;
    }
    consume(&mut actions);
}

// `sAdo_SysTrgStart` ids from `m_editor_ovl.c`.
const GRID_MOVE: u16 = 0x411;
const CURSOR: u16 = 0x35;
const BACKSPACE: u16 = 0x1034;
const START: u16 = 0x5F;
const TOGGLE: u16 = 0x37;
const ERROR: u16 = 0x1003;

/// Typed keys also map to gameplay actions; none may leak through.
fn consume(actions: &mut Actions) {
    for action in [
        Action::Confirm,
        Action::Cancel,
        Action::Back,
        Action::Interact,
        Action::Pause,
    ] {
        actions.consume(action);
    }
}

/// A typed character as a password character: letters, `%`, `&`, `@` and
/// digits (0 and 1 read as `O` and `l`, as the check does) and `#` for the
/// retail grid's 0xD1 symbol.
fn typed_code(ch: char) -> Option<u8> {
    let code = match ch {
        '0' => b'O',
        '1' => b'l',
        // The retail grid's 0xD1 symbol prints as a hash.
        '#' => 0xD1,
        c if c.is_ascii_alphanumeric() || matches!(c, '%' | '&' | '@') => c as u8,
        _ => return None,
    };
    pw::usable_letter(code).then_some(code)
}

#[derive(Resource, Default)]
pub(crate) struct PasswordAssets {
    root: Option<Entity>,
    shown: Option<u64>,
    white: Option<Handle<Image>>,
    skin: Skin,
    /// The window's background materials, whose texture slides.
    frame: Vec<Handle<N64Material>>,
    /// `menu_control.texture_pos`: the background's slide, 0 to 1024.
    slide: f32,
}

fn scaled(text: &str, scale: u8) -> String {
    format!(
        "{}{}{text}",
        crate::messages::markup::LINE_SCALE,
        crate::messages::markup::arg(scale)
    )
}

fn line_text(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| crate::messages::char_of(b).unwrap_or('?'))
        .collect()
}

/// The entry window: the disc's panel and editor, the text on top.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw(
    mut commands: Commands,
    ui: Res<PasswordUi>,
    disc: Option<Res<GameDisc>>,
    font: Option<Res<MsgAssets>>,
    mut assets: ResMut<PasswordAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut n64: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !ui.open {
        if let Some(root) = assets.root.take() {
            commands.entity(root).despawn();
        }
        assets.shown = None;
        assets.frame.clear();
        return;
    }
    let (Some(disc), Some(font)) = (disc, font) else {
        return;
    };
    if assets.shown == Some(ui.revision) && assets.root.is_some() {
        return;
    }
    if let Some(root) = assets.root.take() {
        commands.entity(root).despawn();
    }
    let white = assets
        .white
        .get_or_insert_with(|| {
            images.add(Image::new_fill(
                bevy::render::render_resource::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                bevy::render::render_resource::TextureDimension::D2,
                &[255, 255, 255, 255],
                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                bevy::asset::RenderAssetUsages::RENDER_WORLD,
            ))
        })
        .clone();
    let root = commands
        .spawn((
            Name::new("password entry"),
            Transform::default(),
            Visibility::default(),
            RenderLayers::layer(1),
        ))
        .id();
    assets.root = Some(root);
    assets.shown = Some(ui.revision);
    let skin = &mut assets.skin;
    let mut stores = Stores {
        meshes: &mut meshes,
        n64: &mut n64,
        images: &mut images,
        materials: &mut materials,
    };
    editor::spawn(
        &mut commands,
        root,
        skin,
        &disc,
        &font,
        &ui.keyboard,
        ui.held,
        &mut stores,
    );
    match ui.mode {
        Mode::Code => window_code(
            &mut commands,
            root,
            skin,
            &disc,
            &font,
            &ui,
            &white,
            &mut stores,
        ),
        Mode::Receiver => window_receiver(
            &mut commands,
            root,
            skin,
            &disc,
            &font,
            &ui,
            &white,
            &mut stores,
        ),
    }
    // The background model, again from the cache, is the one that slides.
    let (key, mode) = match ui.mode {
        Mode::Code => ("pwd", "pwd_win_mode"),
        Mode::Receiver => ("pas", "pas_win_mode"),
    };
    let background = if ui.mode == Mode::Code {
        "pwd_win_model"
    } else {
        "pas_win_model"
    };
    let frame = skin.window(&disc, key, &[mode, background], [255; 4], &mut stores);
    assets.frame = frame.into_iter().map(|(_, material)| material).collect();
}

/// `gDPSetTileSize_Dolphin(RENDERTILE, -texture_pos * 4, ..)` before the
/// window's background (`mPC_set_frame_dl`, `mPM_set_frame_dl`): the pattern
/// creeps with `mSM_menu_control_move`'s 0.3535 per tick, as the pockets'.
pub(crate) fn scroll(
    time: Res<Time>,
    mut assets: ResMut<PasswordAssets>,
    mut n64: ResMut<Assets<N64Material>>,
    images: Res<Assets<Image>>,
) {
    if assets.frame.is_empty() {
        return;
    }
    assets.slide = (assets.slide + 0.3535 * time.delta_secs() * 60.0).rem_euclid(1024.0);
    for handle in &assets.frame {
        let Some(mut material) = n64.get_mut(handle) else {
            continue;
        };
        // Only the cloth pieces (tile 0 the cloth, tile 1 their shape mask)
        // keep the slid tile; the other parts load their own tile 0.
        if material.texture1.is_none() {
            continue;
        }
        let Some(size) = material
            .texture
            .as_ref()
            .and_then(|h| images.get(h))
            .map(|image| image.size_f32())
        else {
            continue;
        };
        // The tile origin is -s/4 texels with s = -pos * 4: the pattern is
        // sampled `pos` texels further on.
        material.combiner.extra.y = (assets.slide / size.x.max(1.0)).fract();
        material.combiner.extra.z = (assets.slide / size.y.max(1.0)).fract();
    }
}

/// `m_passwordChk_ovl`: the secret-code window.
#[allow(clippy::too_many_arguments)]
fn window_code(
    commands: &mut Commands,
    root: Entity,
    skin: &mut Skin,
    disc: &GameDisc,
    font: &MsgAssets,
    ui: &PasswordUi,
    white: &Handle<Image>,
    stores: &mut Stores,
) {
    let frame = skin.window(
        disc,
        "pwd",
        &["pwd_win_mode", "pwd_win_model"],
        [255; 4],
        stores,
    );
    editor::place_frame(commands, root, &frame);
    let mut paint = Paint {
        commands,
        meshes: &mut *stores.meshes,
        materials: &mut *stores.materials,
        root,
    };
    paint.text(
        font,
        &scaled("Enter a secret code.", 28),
        Vec2::new(131.0, 34.0),
        [255, 255, 255, 255],
        editor::Z + 0.5,
    );
    let ink = [235, 235, 50, 255];
    for (row, line) in ui.lines.iter().enumerate() {
        let y = 81.0 + row as f32 * 18.0;
        paint.text(
            font,
            &line_text(line),
            Vec2::new(77.0, y),
            ink,
            editor::Z + 0.5,
        );
    }
    // The cursor: a red bar where the next character goes.
    let line = line_text(&ui.lines[ui.selected][..ui.cursor.min(14)]);
    let x = 77.0 + text_width(&line, &font.widths);
    let y = 81.0 + ui.selected as f32 * 18.0;
    paint.put(
        quad_mesh(Vec2::new(x - 0.5, y), Vec2::new(1.5, 14.0)),
        white.clone(),
        Color::srgb_u8(195, 80, 80),
        editor::Z + 0.6,
    );
    editor::hint(
        &mut paint,
        font,
        "Arrows: key  Enter: type  Bksp: erase  Ctrl+Arrows: cursor  F1 case  F2 layout  F3 mode  F4 done  Esc cancel",
    );
}

/// `m_passwordMake_ovl`: the receiver window.
#[allow(clippy::too_many_arguments)]
fn window_receiver(
    commands: &mut Commands,
    root: Entity,
    skin: &mut Skin,
    disc: &GameDisc,
    font: &MsgAssets,
    ui: &PasswordUi,
    white: &Handle<Image>,
    stores: &mut Stores,
) {
    let town_selected = ui.field == 0;
    let frame = skin.window(
        disc,
        "pas",
        &["pas_win_mode", "pas_win_model"],
        [255; 4],
        stores,
    );
    editor::place_frame(commands, root, &frame);
    for (n, (list, selected)) in [
        ("pas_win_waku1_model", town_selected),
        ("pas_win_waku2_model", !town_selected),
    ]
    .into_iter()
    .enumerate()
    {
        let prim = if selected {
            [150, 50, 30, 255]
        } else {
            [255, 255, 255, 255]
        };
        let model = skin.window(
            disc,
            &format!("{list}|{selected}"),
            &["pas_win_mode", list],
            prim,
            stores,
        );
        editor::place_frame_at(commands, root, &model, 0.1 + n as f32 * 0.1);
    }
    if town_selected && !ui.town.is_empty() {
        let model = skin.window(
            disc,
            "pas_start",
            &["pas_win_mode", "pas_start_model"],
            [255; 4],
            stores,
        );
        editor::place_frame_at(commands, root, &model, 0.3);
    }
    let mut paint = Paint {
        commands,
        meshes: &mut *stores.meshes,
        materials: &mut *stores.materials,
        root,
    };
    paint.text(
        font,
        &scaled("Enter a receiver.", 28),
        Vec2::new(141.0, 34.0),
        [255, 255, 255, 255],
        editor::Z + 0.5,
    );
    for (row, (field, selected)) in [(&ui.town, town_selected), (&ui.name, !town_selected)]
        .into_iter()
        .enumerate()
    {
        let ink = if selected {
            [20, 0, 0, 255]
        } else {
            [120, 100, 80, 255]
        };
        let y = 68.0 + row as f32 * 28.0;
        paint.text(
            font,
            &line_text(&field.text[..field.len]),
            Vec2::new(136.0, y),
            ink,
            editor::Z + 0.5,
        );
        if selected {
            let before = line_text(&field.text[..field.cursor.min(field.len)]);
            let x = 136.0 + text_width(&before, &font.widths);
            paint.put(
                quad_mesh(Vec2::new(x - 0.5, y), Vec2::new(1.5, 14.0)),
                white.clone(),
                Color::srgb_u8(195, 80, 80),
                editor::Z + 0.6,
            );
        }
    }
    editor::hint(
        &mut paint,
        font,
        "Arrows: key  Enter: type  Bksp: erase  F1 case  F2 layout  F3 mode  F4 next/done  Esc back",
    );
}
