//! The text editor's character grid (`m_editor_ovl.c`): the ten by four key
//! tables for each layout, the grid cursor with its hold-to-repeat, the
//! shift, layout and mode toggles, and the button-hold repeat the editor
//! uses for backspace and text-cursor moves. Pure state and data; nothing
//! here reads a disc or a device. Drawing, sound and key mapping belong to
//! the caller.
//!
//! Verified against `src/game/m_editor_ovl.c` and `include/m_editor_ovl.h`
//! (GAFE01_00), with the X button's accent exchange and the typed-key voices.
//! Not ported: the multi-line, diary and letter-header edit routines (the
//! diary kernel has them) and the warning windows, which stay with their
//! own screens.

/// `mED_COLUMNS`, `mED_ROWS`.
pub const COLUMNS: usize = 10;
pub const ROWS: usize = 4;
/// `CHAR_SPACE`, and the wide space (`CHAR_SPACE_3`) that marks an empty key.
pub const CHAR_SPACE: u8 = 0x20;
pub const CHAR_SPACE_WIDE: u8 = 211;

/// `mED_INPUT_MODE_*`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Letter,
    Sign,
    Mark,
}

/// `mED_SHIFT_*`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Shift {
    #[default]
    Lower,
    Upper,
}

/// `mED_ARRANGE_*`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Arrange {
    #[default]
    Qwerty,
    Alpha,
}

/// Where the stick (or the arrow keys) points (`mED_STICK_AREA_*`, reduced
/// to the four the grid acts on; diagonals keep the previous direction).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

type Table = [u8; COLUMNS * ROWS];

/// `letterS_table`: lower case, QWERTY.
const LOWER_QWERTY: Table = [
    0x21, 0x3f, 0x22, 0x2d, 0x2a, 0x90, 0x27, 0xd0, 0x3a, 0xd4, 0x71, 0x77, 0x65, 0x72, 0x74, 0x79,
    0x75, 0x69, 0x6f, 0x70, 0x61, 0x73, 0x64, 0x66, 0x67, 0x68, 0x6a, 0x6b, 0x6c, 0xcd, 0x7a, 0x78,
    0x63, 0x76, 0x62, 0x6e, 0x6d, 0x2c, 0x2e, 0x20,
];
/// `letterS_table2`: lower case, alphabetical.
const LOWER_ALPHA: Table = [
    0x21, 0x3f, 0x22, 0x2d, 0x2a, 0x90, 0x27, 0xd0, 0x3a, 0x85, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66,
    0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0xcd, 0x74, 0x75,
    0x76, 0x77, 0x78, 0x79, 0x7a, 0x2c, 0x2e, 0x20,
];
/// `letterL_table`: upper case, QWERTY.
const UPPER_QWERTY: Table = [
    0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x51, 0x57, 0x45, 0x52, 0x54, 0x59,
    0x55, 0x49, 0x4f, 0x50, 0x41, 0x53, 0x44, 0x46, 0x47, 0x48, 0x4a, 0x4b, 0x4c, 0xcd, 0x5a, 0x58,
    0x43, 0x56, 0x42, 0x4e, 0x4d, 0x2c, 0x2e, 0x20,
];
/// `letterL_table2`: upper case, alphabetical.
const UPPER_ALPHA: Table = [
    0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46,
    0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0xcd, 0x54, 0x55,
    0x56, 0x57, 0x58, 0x59, 0x5a, 0x2c, 0x2e, 0x20,
];
/// `sign_table`.
const SIGN: Table = [
    0xd1, 0x3f, 0x22, 0x2d, 0x2a, 0x90, 0x85, 0xd0, 0x3a, 0xa2, 0x25, 0x26, 0x40, 0x5f, 0xa0, 0xae,
    0x97, 0xc0, 0xc1, 0x3d, 0x28, 0x29, 0x3c, 0x3e, 0xa5, 0xa6, 0xac, 0xad, 0xb4, 0xcd, 0x1d, 0x1e,
    0x86, 0x98, 0x9b, 0x9c, 0xa1, 0x2c, 0x2e, 0x20,
];
/// `mark_table`.
const MARK: Table = [
    0x2b, 0xb9, 0x2f, 0x3b, 0x5c, 0xb8, 0xc6, 0xb6, 0xb7, 0xaf, 0xb0, 0xb1, 0xb2, 0xb3, 0xba, 0xbb,
    0xbc, 0xbd, 0xbe, 0xbf, 0xa7, 0xa8, 0xa9, 0xab, 0xaa, 0xb5, 0xc2, 0xc3, 0xc4, 0xcd, 0xc7, 0xc8,
    0xc9, 0xca, 0xcb, 0xcc, 0xc5, 0xce, 0xcf, 0x20,
];

/// `mED_get_code`: the character on a key for a layout.
pub fn code_at(arrange: Arrange, shift: Shift, mode: InputMode, column: usize, row: usize) -> u8 {
    let table = match (mode, shift, arrange) {
        (InputMode::Letter, Shift::Lower, Arrange::Qwerty) => &LOWER_QWERTY,
        (InputMode::Letter, Shift::Lower, Arrange::Alpha) => &LOWER_ALPHA,
        (InputMode::Letter, Shift::Upper, Arrange::Qwerty) => &UPPER_QWERTY,
        (InputMode::Letter, Shift::Upper, Arrange::Alpha) => &UPPER_ALPHA,
        (InputMode::Sign, ..) => &SIGN,
        (InputMode::Mark, ..) => &MARK,
    };
    table[column.min(COLUMNS - 1) + row.min(ROWS - 1) * COLUMNS]
}

/// Frames a held direction waits before repeating, and the frame count it
/// rewinds to after each repeat (`stick_area_held_frames`).
const STICK_REPEAT_AT: u8 = 16;
const STICK_REPEAT_REWIND: u8 = 12;

/// The grid cursor and layout (`mED_Ovl_c`'s keyboard fields).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Keyboard {
    pub column: usize,
    pub row: usize,
    pub mode: InputMode,
    pub shift: Shift,
    pub arrange: Arrange,
    held: Option<Direction>,
    held_frames: u8,
}

/// What a grid step did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moved {
    /// The cursor stayed (no direction, waiting to repeat, or at an edge).
    No,
    /// It reached an edge and was clamped (no click sound).
    Edge,
    /// It moved to another key (`sAdo_SysTrgStart(0x411)`).
    Yes,
}

impl Keyboard {
    /// `mED_move_keyboard_cursor`, one 60 Hz frame: a new direction moves at
    /// once; held, it repeats every four frames after sixteen.
    pub fn step(&mut self, direction: Option<Direction>) -> Moved {
        let Some(direction) = direction else {
            self.held = None;
            return Moved::No;
        };
        let fire = if self.held != Some(direction) {
            self.held = Some(direction);
            self.held_frames = 0;
            true
        } else {
            self.held_frames += 1;
            if self.held_frames >= STICK_REPEAT_AT {
                self.held_frames = STICK_REPEAT_REWIND;
                true
            } else {
                false
            }
        };
        if !fire {
            return Moved::No;
        }
        let (dc, dr) = match direction {
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
        };
        let column = (self.column as i32 + dc).clamp(0, COLUMNS as i32 - 1) as usize;
        let row = (self.row as i32 + dr).clamp(0, ROWS as i32 - 1) as usize;
        let moved = (column, row) != (self.column, self.row);
        self.column = column;
        self.row = row;
        if moved {
            Moved::Yes
        } else {
            Moved::Edge
        }
    }

    /// `mED_check_shift` (L): flips lower/upper case.
    pub fn toggle_shift(&mut self) {
        self.shift = match self.shift {
            Shift::Lower => Shift::Upper,
            Shift::Upper => Shift::Lower,
        };
    }

    /// `mED_check_arrange` (Z): flips QWERTY/alphabetical.
    pub fn toggle_arrange(&mut self) {
        self.arrange = match self.arrange {
            Arrange::Qwerty => Arrange::Alpha,
            Arrange::Alpha => Arrange::Qwerty,
        };
    }

    /// `mED_check_input_mode` (Y): letters, signs, marks; back to lower case.
    pub fn next_mode(&mut self) {
        self.mode = match self.mode {
            InputMode::Letter => InputMode::Sign,
            InputMode::Sign => InputMode::Mark,
            InputMode::Mark => InputMode::Letter,
        };
        self.shift = Shift::Lower;
    }

    /// The character on the key at (`column`, `row`) in the current layout.
    pub fn code(&self, column: usize, row: usize) -> u8 {
        code_at(self.arrange, self.shift, self.mode, column, row)
    }

    /// `mED_check_now_code` (A): the selected key's character, unless the
    /// key is empty (the wide space).
    pub fn selected(&self) -> Option<u8> {
        let code = self.code(self.column, self.row);
        (code != CHAR_SPACE_WIDE).then_some(code)
    }
}

/// A held editing button (`mED_check_move_cursol`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Backspace,
    CursorLeft,
    CursorDown,
    CursorUp,
    CursorRight,
}

/// Hold-to-repeat for the editing buttons: it fires on the first frame,
/// again after thirty, then every six frames, then every two once the
/// accelerator has run down (`_10`, `_0F`, `_0E` in the original).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButtonRepeat {
    held: Option<Button>,
    wait: u8,
    accel: u8,
    repeating: bool,
}

impl ButtonRepeat {
    /// One 60 Hz frame; true when the held button should act now.
    pub fn step(&mut self, button: Option<Button>) -> bool {
        let Some(button) = button else {
            *self = Self::default();
            return false;
        };
        if self.held != Some(button) {
            *self = Self {
                held: Some(button),
                wait: 30,
                accel: 26,
                repeating: false,
            };
            return true;
        }
        if self.repeating && self.accel != 0 {
            self.accel -= 1;
        }
        if self.wait != 0 {
            self.wait -= 1;
            return false;
        }
        if self.accel == 0 {
            self.wait = 2;
        } else {
            self.wait = 6;
            self.repeating = true;
        }
        true
    }
}

/// A single-line, space-padded field with a cursor (the town and player
/// name of `m_passwordMake_ovl`): `mED_output_code`, `mED_backspace_func` and
/// the cursor moves of `mED_move_cursol_left` / `_right`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field<const N: usize> {
    pub text: [u8; N],
    pub len: usize,
    pub cursor: usize,
}

impl<const N: usize> Default for Field<N> {
    fn default() -> Self {
        Self {
            text: [CHAR_SPACE; N],
            len: 0,
            cursor: 0,
        }
    }
}

impl<const N: usize> Field<N> {
    /// Inserts `code` at the cursor, shifting the rest right; false when full.
    pub fn insert(&mut self, code: u8) -> bool {
        if self.len >= N {
            return false;
        }
        self.text
            .copy_within(self.cursor..self.len, self.cursor + 1);
        self.text[self.cursor] = code;
        self.cursor += 1;
        self.len += 1;
        true
    }

    /// Deletes the character before the cursor; false at the start.
    pub fn backspace(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.cursor -= 1;
        self.len -= 1;
        self.text
            .copy_within(self.cursor + 1..self.len + 1, self.cursor);
        self.text[self.len] = CHAR_SPACE;
        true
    }

    pub fn left(&mut self) -> bool {
        if self.cursor > 0 {
            self.cursor -= 1;
            true
        } else {
            false
        }
    }

    /// Inside the text it steps right; at the end it types a space, if
    /// there is room (`mED_move_cursol_right`).
    pub fn right(&mut self) -> bool {
        if self.cursor < self.len {
            self.cursor += 1;
            true
        } else {
            self.insert(CHAR_SPACE)
        }
    }

    /// Characters typed so far.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// `mED_ornament_table`: the character the X button swaps the one before the
/// cursor for (accented and marked forms trade places with their plain letter).
pub const ORNAMENT: [u8; 256] = [
    0x21, 0x3F, 0x03, 0x04, 0x05, 0x06, 0x07, 0x41, 0x43, 0x0A, 0x0B, 0x0C, 0x45, 0x0E, 0x0F, 0x10,
    0x49, 0x44, 0x4E, 0x14, 0x15, 0x16, 0x17, 0x4F, 0x8D, 0x1A, 0x1B, 0x1C, 0x55, 0x1D, 0x95, 0x23,
    0x20, 0x00, 0xA4, 0x24, 0x5B, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F,
    0x18, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x01,
    0x40, 0x02, 0x42, 0x08, 0x11, 0x09, 0x46, 0x47, 0x48, 0x0D, 0x4A, 0x4B, 0x4C, 0x4D, 0x12, 0x13,
    0x50, 0x51, 0x52, 0x53, 0x54, 0x19, 0x56, 0x57, 0x58, 0x96, 0x5A, 0x5D, 0x5C, 0x5E, 0x61, 0x5F,
    0x63, 0x1F, 0x62, 0x60, 0x64, 0x7B, 0x66, 0x67, 0x68, 0x81, 0x6A, 0x6B, 0x6C, 0x6D, 0x87, 0x88,
    0x70, 0x71, 0x72, 0x73, 0x74, 0x8E, 0x76, 0x77, 0x78, 0x93, 0x7A, 0x7C, 0x7D, 0x7E, 0x65, 0x7F,
    0x80, 0x82, 0x83, 0x84, 0x69, 0x85, 0x86, 0x6E, 0x89, 0x8A, 0x8B, 0x8C, 0x6F, 0x30, 0x8F, 0x91,
    0x90, 0x92, 0x75, 0x94, 0x79, 0x1E, 0x59, 0x97, 0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F,
    0xA0, 0xA1, 0xA3, 0xA2, 0x22, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF,
    0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF,
    0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF,
    0xD0, 0xD1, 0xFF, 0xD2, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF,
    0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE, 0xEF,
    0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xD3,
];

/// `mED_exchange_se_list`: whether swapping in this character plays its voice
/// rather than the plain refusal sound.
const EXCHANGE_VOICE: [bool; 256] = [
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    false, false, false, true, true, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, true, false, true, true, false, true, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, true, true, true,
    true, false, false, false, false, false, false, false, true, true, true, true, true, true,
    true, true, true, true, false, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, false, true, true, true, true, true, true, true, true, true, true,
    true, true, true, false, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true,
];

/// `mED_get_exchange_code`: what the X button would turn the character before
/// the cursor into, if it differs.
pub fn exchange_code(text: &[u8], cursor: usize) -> Option<u8> {
    let prev = *text.get(cursor.checked_sub(1)?)?;
    let ornament = ORNAMENT[usize::from(prev)];
    (ornament != prev).then_some(ornament)
}

/// `mED_exchange_code_func`: swaps the character before the cursor in place.
pub fn exchange(text: &mut [u8], cursor: usize) -> bool {
    match exchange_code(text, cursor) {
        Some(code) => {
            text[cursor - 1] = code;
            true
        }
        None => false,
    }
}

/// `mMsg_sound_voice_get_for_editor`: the voice id a character speaks with in
/// the editor (the quiet ones, 0x80/0x82/0x83/0x85, speak 0x86).
pub fn editor_voice(phoneme: u8) -> u8 {
    match phoneme {
        0x80 | 0x82 | 0x83 | 0x85 => 0x86,
        other => other,
    }
}

/// `mED_set_se` for a typed character (`OUTPUT_CODE`): its voice, and 0x86
/// for the silent ones (also 0x81).
pub fn output_voice(phoneme: u8) -> u8 {
    match editor_voice(phoneme) {
        0x81 => 0x86,
        other => other,
    }
}

/// `mED_set_se` for an exchange: the new character's voice, or none when the
/// list says it stays quiet (the caller plays the plain 0x36 refusal sound).
pub fn exchange_voice(code: u8, phoneme: u8) -> Option<u8> {
    EXCHANGE_VOICE[usize::from(code)].then(|| editor_voice(phoneme))
}
