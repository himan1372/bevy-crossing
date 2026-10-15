//! The game's own text, from the disc: villager dialogue
//! (`message_data.bin`, `forest_2nd.arc`), strings such as catchphrases
//! (`string_data.bin`) and choice labels (`select_data.bin`, both in
//! `forest_1st.arc`).
//!
//! Each bank is a big-endian u32 table of end offsets plus the data:
//! entry 0 is `[0, t[0])`, entry `i` is `[t[i-1], t[i])`
//! (`mMsg_Get_BodyParam`). Text uses the game's character set (`m_font.h`);
//! 0x7F starts a control code whose length comes from the disc's
//! `mFont_cont_info_tbl`. Pages break at CONTINUE / CLEAR; a message can
//! name the next one to show (`SET_NEXT_MESSAGE_*`).

use std::sync::Arc;

use disc::{Rarc, RomSource};

/// One text bank: offset table and data.
#[derive(Clone, Default)]
pub struct TextBank {
    table: Arc<[u8]>,
    data: Arc<[u8]>,
}

impl TextBank {
    fn new(table: &[u8], data: &[u8]) -> Self {
        Self {
            table: table.into(),
            data: data.into(),
        }
    }

    fn end(&self, i: usize) -> Option<usize> {
        let b = self.table.get(i * 4..i * 4 + 4)?;
        Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize)
    }

    /// Entry `i`'s bytes.
    pub fn get(&self, i: usize) -> Option<&[u8]> {
        let start = if i == 0 { 0 } else { self.end(i - 1)? };
        let end = self.end(i)?;
        self.data.get(start..end.max(start))
    }
}

/// `CHAR_CONTROL_CODE` / `CHAR_MESSAGE_TAG` / `CHAR_NEW_LINE`.
const CONTROL: u8 = 0x7F;
const TAG: u8 = 0x80;
const NEW_LINE: u8 = 0xCD;

/// Page text keeps the display codes the talk window needs as private-use
/// characters, each followed by its arguments as `ARG + value` characters:
/// [`markup::COLOR_CHARS`] r g b n (the next n characters, from
/// `mFont_CONT_CODE_SET_COLOR_CHAR`), [`markup::LINE_COLOR`] r g b (the
/// line, `mFont_CONT_CODE_COLOR`) and [`markup::PAUSE`] t (t ticks,
/// `mFont_CONT_CODE_CURSOR_SET_TIME`), plus voice status and character/line
/// scale controls. [`markup::strip`] removes all control markers.
pub mod markup {
    pub const COLOR_CHARS: char = '\u{E000}';
    pub const LINE_COLOR: char = '\u{E001}';
    pub const PAUSE: char = '\u{E002}';
    pub const STATUS: char = '\u{E003}';
    pub const CHAR_SCALE: char = '\u{E004}';
    pub const LINE_SCALE: char = '\u{E005}';
    /// Argument characters: `ARG + byte`.
    const ARG: u32 = 0xE100;

    pub fn arg(v: u8) -> char {
        char::from_u32(ARG + v as u32).unwrap_or(' ')
    }

    fn value(c: char) -> u8 {
        (c as u32).wrapping_sub(ARG) as u8
    }

    /// One shown piece of a page.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Item {
        Char(char, Option<[u8; 3]>),
        NewLine,
        /// The cursor waits this many 60 Hz ticks.
        Pause(u32),
        Status(u8),
        CharScale(u8),
        LineScale(u8),
    }

    /// The page as characters with their colours, line breaks and pauses.
    pub fn parse(s: &str) -> Vec<Item> {
        let mut out = Vec::new();
        let mut chars = s.chars();
        let (mut run, mut run_left, mut line): (Option<[u8; 3]>, u32, Option<[u8; 3]>) =
            (None, 0, None);
        while let Some(c) = chars.next() {
            let mut rgb = || [chars.next(), chars.next(), chars.next()].map(|c| c.map_or(0, value));
            match c {
                COLOR_CHARS => {
                    run = Some(rgb());
                    run_left = chars.next().map_or(0, value) as u32;
                }
                LINE_COLOR => line = Some(rgb()),
                PAUSE => out.push(Item::Pause(chars.next().map_or(0, value) as u32 * 2)),
                STATUS => out.push(Item::Status(chars.next().map_or(0, value))),
                CHAR_SCALE => out.push(Item::CharScale(chars.next().map_or(32, value))),
                LINE_SCALE => out.push(Item::LineScale(chars.next().map_or(32, value))),
                '\n' => {
                    line = None;
                    out.push(Item::NewLine);
                }
                _ => {
                    let color = if run_left > 0 {
                        run_left -= 1;
                        run
                    } else {
                        line
                    };
                    out.push(Item::Char(c, color));
                }
            }
        }
        out
    }

    /// The page as plain text.
    pub fn strip(s: &str) -> String {
        parse(s)
            .into_iter()
            .filter_map(|i| match i {
                Item::Char(c, _) => Some(c),
                Item::NewLine => Some('\n'),
                Item::Pause(_) | Item::Status(_) | Item::CharScale(_) | Item::LineScale(_) => None,
            })
            .collect()
    }
}

/// `mMsg_Set_PlayerNameColor`: the player's name in (75, 95, 155).
const PLAYER_NAME_COLOR: [u8; 3] = [75, 95, 155];

/// `mFont_CONT_CODE_*` values used here.
mod code {
    pub const LAST: u8 = 0;
    pub const CONTINUE: u8 = 1;
    pub const CLEAR: u8 = 2;
    pub const CURSOR_SET_TIME: u8 = 3;
    pub const COLOR: u8 = 5;
    pub const SET_COLOR_CHAR: u8 = 80;
    pub const CONTENTS_NORMAL: u8 = 75;
    pub const CONTENTS_SLEEPY: u8 = 79;
    pub const CONTENTS_GLOOMY: u8 = 97;
    pub const SET_CHAR_SCALE: u8 = 84;
    pub const SET_LINE_SCALE: u8 = 90;
    pub const NEXT_F: u8 = 14;
    pub const NEXT_0: u8 = 15;
    pub const NEXT_3: u8 = 18;
    pub const NEXT_RANDOM_2: u8 = 19;
    pub const NEXT_RANDOM_4: u8 = 21;
    pub const SELECT_2: u8 = 22;
    pub const SELECT_4: u8 = 24;
    pub const PLAYER_NAME: u8 = 26;
    pub const TALK_NAME: u8 = 27;
    pub const TAIL: u8 = 28;
    pub const YEAR: u8 = 29;
    pub const MONTH: u8 = 30;
    pub const WEEK: u8 = 31;
    pub const DAY: u8 = 32;
    pub const HOUR: u8 = 33;
    pub const MIN: u8 = 34;
    pub const FREE0: u8 = 36;
    pub const FREE9: u8 = 45;
    pub const ITEM0: u8 = 49;
    pub const ITEM4: u8 = 53;
    pub const FREE10: u8 = 54;
    pub const FREE19: u8 = 63;
    pub const SPACE: u8 = 103;
    pub const CAPITAL_LETTER: u8 = 117;
    pub const AM_PM: u8 = 118;
    pub const NEXT_4: u8 = 119;
    pub const NEXT_5: u8 = 120;
    pub const SELECT_5: u8 = 121;
    pub const SELECT_6: u8 = 122;
}

/// Latin letters and symbols from CHAR_* in m_font.h, including accented
/// letters occupying positions otherwise associated with ASCII punctuation.
fn char_of(c: u8) -> Option<char> {
    const LOW: [char; 31] = [
        '¡', '¿', 'Ä', 'À', 'Á', 'Â', 'Ã', 'Å', 'Ç', 'È', 'É', 'Ê', 'Ë', 'Ì', 'Í', 'Î', 'Ï', 'Ð',
        'Ñ', 'Ò', 'Ó', 'Ô', 'Õ', 'Ö', 'Œ', 'Ù', 'Ú', 'Û', 'Ü', 'ß', 'Þ',
    ];
    Some(match c {
        0..=30 => LOW[c as usize],
        0x1F => 'à',
        0x23 => 'á',
        0x24 => 'â',
        0x5B => 'ã',
        0x5D => 'ä',
        0x5E => 'å',
        0x7B => 'è',
        0x7C => 'é',
        0x7D => 'ê',
        0x7E => 'ë',
        0x81 => 'ì',
        0x82 => 'í',
        0x83 => 'î',
        0x84 => 'ï',
        0x87 => 'ñ',
        0x88 => 'ò',
        0x89 => 'ó',
        0x8A => 'ô',
        0x8B => 'õ',
        0x8C => 'ö',
        0x8D => 'œ',
        0x8E => 'ù',
        0x8F => 'ú',
        0x91 => 'û',
        0x92 => 'ü',
        0x93 => 'ý',
        0x94 => 'ÿ',
        0x97 => '¦',
        0x98 => '§',
        0x99 => 'ª',
        0x9A => 'º',
        0x9B => '‖',
        0x9C => 'µ',
        0x9D => '³',
        0x9E => '²',
        0x9F => '¹',
        0xA0 => '¯',
        0xA1 => '¬',
        0xA2 => 'Æ',
        0xA3 => 'æ',
        0x2A => '~',
        0x2B => '♥',
        0x2F => '♪',
        0x3B => '💧',
        0x5C => '💢',
        0x60 => 'ç',
        0x85 => '·',
        0x86 => 'ð',
        0x90 => '-',
        0x95 => 'þ',
        0x96 => 'Ý',
        0xA4 => '“',
        0xA5 => '«',
        0xA6 => '»',
        0xA7 => '☀',
        0xA8 => '☁',
        0xA9 => '☂',
        0xAE => '/',
        0xAF => '∞',
        0xB0 => '○',
        0xB1 => '×',
        0xB2 => '□',
        0xB3 => '△',
        0xB4 => '+',
        0xB9 => '★',
        0xC1 => '÷',
        0xD0 => ';',
        0xD1 => '#',
        0xD2 | 0xD3 => ' ',
        0xD5 => '“',
        0xD6 => '”',
        0xD7 => '‘',
        0xD8 => '’',
        0xDE => '\\',
        0x20..=0x7E => c as char,
        _ => return None,
    })
}

/// The AC charset code that draws `ch` (`char_of` backwards; `?` for
/// characters the font doesn't have).
pub fn ac_code(ch: char) -> u8 {
    ac_code_opt(ch).unwrap_or(b'?')
}

/// Voice lookup keeps unsupported Unicode silent instead of voicing '?'.
pub fn ac_code_opt(ch: char) -> Option<u8> {
    static TABLE: std::sync::OnceLock<std::collections::HashMap<char, u8>> =
        std::sync::OnceLock::new();
    let table = TABLE.get_or_init(|| {
        let mut t = std::collections::HashMap::new();
        for c in (0..=255u8).rev() {
            if let Some(ch) = char_of(c) {
                t.insert(ch, c);
            }
        }
        t
    });
    table.get(&ch).copied()
}

#[cfg(test)]
mod markup_tests {
    use super::markup::{self, Item};

    #[test]
    fn colours_runs_lines_and_pauses() {
        let mut s = String::from("Hi ");
        s.push(markup::COLOR_CHARS);
        s.extend([75, 95, 155, 2].map(markup::arg));
        s.push_str("Bob!");
        s.push(markup::PAUSE);
        s.push(markup::arg(5));
        s.push('\n');
        s.push(markup::LINE_COLOR);
        s.extend([1, 2, 3].map(markup::arg));
        s.push('x');
        let items = markup::parse(&s);
        assert_eq!(items[3], Item::Char('B', Some([75, 95, 155])));
        assert_eq!(items[4], Item::Char('o', Some([75, 95, 155])));
        assert_eq!(items[5], Item::Char('b', None));
        assert_eq!(items[7], Item::Pause(10));
        assert_eq!(items[9], Item::Char('x', Some([1, 2, 3])));
        assert_eq!(markup::strip(&s), "Hi Bob!\nx");
    }
}

/// Plain text of a string-bank entry (names, catchphrases): no codes.
pub fn plain(bytes: &[u8]) -> String {
    bytes
        .iter()
        .filter_map(|&c| char_of(c))
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// Who and what a message talks about (`mMsg_Set_*_str`).
#[derive(Clone, Debug, Default)]
pub struct MsgContext {
    pub player_name: String,
    pub talk_name: String,
    /// The villager's catchphrase (`Animal_c::catchphrase`).
    pub tail: String,
    /// `PUT_STRING_FREE0..19` and `PUT_STRING_ITEM0..4`: what game code
    /// fills in for the topic (an item's name, a count...).
    pub free: Vec<String>,
    pub items: Vec<String>,
    pub year: u16,
    pub month: u8,
    pub weekday: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

/// The disc's text banks and control-code lengths.
#[derive(Clone, Default)]
pub struct GameText {
    pub messages: TextBank,
    pub strings: TextBank,
    pub choices: TextBank,
    /// `mFont_cont_info_tbl[code].size`: bytes including 0x7F and the code.
    code_size: Vec<u8>,
}

/// Messages followed through `SET_NEXT_MESSAGE` before giving up.
const MAX_CHAIN: usize = 8;

impl GameText {
    /// Reads the banks from the disc's archives and the code table from
    /// the REL.
    pub fn read<R: std::io::Read + std::io::Seek>(
        disc: &mut disc::Disc<R>,
        rom: &disc::RomImage,
        catalog: &disc::AssetCatalog,
    ) -> Option<Self> {
        let first = Rarc::parse(disc.read_file("forest_1st.arc").ok()?)?;
        let second = Rarc::parse(disc.read_file("forest_2nd.arc").ok()?)?;
        let bank = |arc: &Rarc, name: &str| {
            Some(TextBank::new(
                arc.file(&format!("{name}_table.bin"))?,
                arc.file(&format!("{name}.bin"))?,
            ))
        };
        let info = catalog.get("mFont_cont_info_tbl")?;
        let raw = rom.slice(RomSource::Rel, info.offset, info.size)?;
        Some(Self {
            messages: bank(&second, "message_data")?,
            strings: bank(&first, "string_data")?,
            choices: bank(&first, "select_data")?,
            code_size: raw.chunks(2).map(|e| e[0]).collect(),
        })
    }

    fn size_of(&self, code: u8) -> usize {
        self.code_size
            .get(code as usize)
            .map_or(2, |&s| (s as usize).max(2))
    }

    /// String-bank entry as plain text.
    pub fn string(&self, i: usize) -> Option<String> {
        self.strings.get(i).map(plain)
    }

    /// Message `msg_no` and the ones it chains to, as pages of text.
    /// `pick(n)` is a random 0..n.
    #[cfg(test)]
    pub fn pages(
        &self,
        msg_no: usize,
        ctx: &MsgContext,
        pick: &mut dyn FnMut(u32) -> u32,
    ) -> Vec<String> {
        self.talk(msg_no, ctx, pick).0
    }

    /// As [`GameText::pages`], stopping at a question: its answers
    /// (`SELECT_n`) and the message each leads to (`NEXT_0`..`NEXT_5`).
    pub fn talk(
        &self,
        msg_no: usize,
        ctx: &MsgContext,
        pick: &mut dyn FnMut(u32) -> u32,
    ) -> (Vec<String>, Option<MsgChoice>) {
        let mut pages = vec![String::new()];
        let mut next = Some(msg_no);
        let mut followed = 0;
        let mut choice = None;
        while let Some(no) = next.take() {
            followed += 1;
            if followed > MAX_CHAIN {
                break;
            }
            let Some(text) = self.messages.get(no) else {
                break;
            };
            if !pages.last().is_some_and(String::is_empty) {
                pages.push(String::new());
            }
            let (then, asks) = self.render(text, ctx, &mut pages, pick);
            if asks.is_some() {
                choice = asks;
                break;
            }
            next = then;
        }
        pages.retain(|p| !p.trim().is_empty());
        let pages = pages.iter().map(|p| p.trim_end().to_string()).collect();
        (pages, choice)
    }

    /// Appends one message to `pages`; returns the message it continues to,
    /// or the question it ends on.
    fn render(
        &self,
        text: &[u8],
        ctx: &MsgContext,
        pages: &mut Vec<String>,
        pick: &mut dyn FnMut(u32) -> u32,
    ) -> (Option<usize>, Option<MsgChoice>) {
        let mut next = None;
        let mut labels: Option<Vec<String>> = None;
        let mut branches: [Option<usize>; 6] = [None; 6];
        let mut capital = false;
        let mut status = 0;
        let mut i = 0;
        let u16_at = |i: usize| -> u16 {
            u16::from_be_bytes([
                text.get(i).copied().unwrap_or(0xFF),
                text.get(i + 1).copied().unwrap_or(0xFF),
            ])
        };
        while i < text.len() {
            let c = text[i];
            let page = pages.last_mut().expect("at least one page");
            match c {
                CONTROL => {
                    let k = text.get(i + 1).copied().unwrap_or(code::LAST);
                    let mut put = |s: &str, page: &mut String| {
                        let mut chars = s.chars();
                        if capital {
                            if let Some(f) = chars.next() {
                                page.extend(f.to_uppercase());
                            }
                            capital = false;
                        }
                        page.push_str(chars.as_str());
                    };
                    match k {
                        code::LAST => break,
                        code::CONTINUE | code::CLEAR => {
                            let mut page = String::new();
                            if status != 0 {
                                page.extend([markup::STATUS, markup::arg(status)]);
                            }
                            pages.push(page);
                        }
                        code::CONTENTS_NORMAL..=code::CONTENTS_SLEEPY | code::CONTENTS_GLOOMY => {
                            status = if k == code::CONTENTS_GLOOMY {
                                5
                            } else {
                                k - code::CONTENTS_NORMAL
                            };
                            page.extend([markup::STATUS, markup::arg(status)]);
                        }
                        code::SET_CHAR_SCALE | code::SET_LINE_SCALE => {
                            let marker = if k == code::SET_CHAR_SCALE {
                                markup::CHAR_SCALE
                            } else {
                                markup::LINE_SCALE
                            };
                            page.extend([
                                marker,
                                markup::arg(text.get(i + 2).copied().unwrap_or(32)),
                            ]);
                        }
                        code::NEXT_F => {
                            next = Some(u16_at(i + 2))
                                .filter(|&n| n != 0xFFFF)
                                .map(usize::from);
                        }
                        // Where each answer leads (NEXT_0..3, NEXT_4/5).
                        code::NEXT_0..=code::NEXT_3 | code::NEXT_4 | code::NEXT_5 => {
                            let slot = match k {
                                code::NEXT_4 => 4,
                                code::NEXT_5 => 5,
                                _ => (k - code::NEXT_0) as usize,
                            };
                            branches[slot] = Some(u16_at(i + 2))
                                .filter(|&n| n != 0xFFFF)
                                .map(usize::from);
                        }
                        code::NEXT_RANDOM_2..=code::NEXT_RANDOM_4 => {
                            let n = (k - code::NEXT_RANDOM_2 + 2) as u32;
                            let which = pick(n) as usize;
                            next = Some(u16_at(i + 2 + which * 2))
                                .filter(|&n| n != 0xFFFF)
                                .map(usize::from);
                        }
                        code::SELECT_2..=code::SELECT_4 | code::SELECT_5 | code::SELECT_6 => {
                            let n = match k {
                                code::SELECT_5 => 5,
                                code::SELECT_6 => 6,
                                _ => (k - code::SELECT_2 + 2) as usize,
                            };
                            labels = Some(
                                (0..n)
                                    .filter_map(|j| {
                                        self.choices.get(u16_at(i + 2 + j * 2) as usize).map(plain)
                                    })
                                    .collect(),
                            );
                        }
                        code::PLAYER_NAME => {
                            let n = ctx.player_name.chars().count().min(255) as u8;
                            page.push(markup::COLOR_CHARS);
                            page.extend(PLAYER_NAME_COLOR.map(markup::arg));
                            page.push(markup::arg(n));
                            put(&ctx.player_name, page);
                        }
                        code::CURSOR_SET_TIME => {
                            page.push(markup::PAUSE);
                            page.push(markup::arg(text.get(i + 2).copied().unwrap_or(0)));
                        }
                        code::COLOR => {
                            page.push(markup::LINE_COLOR);
                            page.extend(
                                (2..5).map(|o| markup::arg(text.get(i + o).copied().unwrap_or(0))),
                            );
                        }
                        code::SET_COLOR_CHAR => {
                            page.push(markup::COLOR_CHARS);
                            page.extend(
                                (2..6).map(|o| markup::arg(text.get(i + o).copied().unwrap_or(0))),
                            );
                        }
                        code::TALK_NAME => put(&ctx.talk_name, page),
                        code::TAIL => put(&ctx.tail, page),
                        code::YEAR => put(&ctx.year.to_string(), page),
                        code::MONTH => put(MONTHS[(ctx.month.max(1) as usize - 1) % 12], page),
                        code::WEEK => put(WEEKDAYS[ctx.weekday as usize % 7], page),
                        code::DAY => put(&ctx.day.to_string(), page),
                        code::HOUR => put(&(((ctx.hour as u32 + 11) % 12) + 1).to_string(), page),
                        code::MIN => put(&format!("{:02}", ctx.minute), page),
                        code::AM_PM => put(if ctx.hour < 12 { "AM" } else { "PM" }, page),
                        code::FREE0..=code::FREE9 | code::FREE10..=code::FREE19 => {
                            let slot = if k >= code::FREE10 {
                                k - code::FREE10 + 10
                            } else {
                                k - code::FREE0
                            };
                            put(ctx.free.get(slot as usize).map_or("", String::as_str), page);
                        }
                        code::ITEM0..=code::ITEM4 => {
                            put(
                                ctx.items
                                    .get((k - code::ITEM0) as usize)
                                    .map_or("", String::as_str),
                                page,
                            );
                        }
                        code::SPACE => page.push(' '),
                        code::CAPITAL_LETTER => capital = true,
                        _ => {}
                    }
                    i += self.size_of(k);
                    continue;
                }
                TAG => {
                    i += 2;
                    continue;
                }
                NEW_LINE => page.push('\n'),
                _ => {
                    if let Some(ch) = char_of(c) {
                        if capital {
                            page.extend(ch.to_uppercase());
                            capital = false;
                        } else {
                            page.push(ch);
                        }
                    }
                }
            }
            i += 1;
        }
        let choice = labels.map(|labels| MsgChoice {
            next: branches[..labels.len().min(6)].to_vec(),
            labels,
        });
        // Without a question, NEXT_0 is where the talk goes on.
        (
            next.or(if choice.is_none() { branches[0] } else { None }),
            choice,
        )
    }
}

/// A question a message ends on: its answers, and the message each one
/// leads to.
#[derive(Clone, Debug, PartialEq)]
pub struct MsgChoice {
    pub labels: Vec<String>,
    pub next: Vec<Option<usize>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bank(entries: &[&[u8]]) -> TextBank {
        let mut table = Vec::new();
        let mut data = Vec::new();
        for e in entries {
            data.extend_from_slice(e);
            table.extend_from_slice(&(data.len() as u32).to_be_bytes());
        }
        TextBank::new(&table, &data)
    }

    fn text(messages: &[&[u8]]) -> GameText {
        let mut sizes = vec![2u8; 123];
        sizes[code::NEXT_F as usize] = 4;
        sizes[code::NEXT_RANDOM_2 as usize] = 6;
        GameText {
            messages: bank(messages),
            strings: bank(&[b"hmm"]),
            choices: bank(&[b"Yes", b"No"]),
            code_size: sizes,
        }
    }

    #[test]
    fn bank_entries_follow_end_offsets() {
        let b = bank(&[b"ab", b"cde", b""]);
        assert_eq!(b.get(0), Some(&b"ab"[..]));
        assert_eq!(b.get(1), Some(&b"cde"[..]));
        assert_eq!(b.get(2), Some(&b""[..]));
        assert_eq!(b.get(3), None);
    }

    #[test]
    fn names_pages_and_chains() {
        // "Hi, <player>!" / page break / "<tail>!" then continue to msg 1.
        let m0: &[u8] = &[
            b'H',
            b'i',
            b',',
            b' ',
            CONTROL,
            code::PLAYER_NAME,
            b'!',
            CONTROL,
            code::CONTINUE,
            CONTROL,
            code::TAIL,
            b'!',
            CONTROL,
            code::NEXT_F,
            0,
            1,
            CONTROL,
            code::LAST,
        ];
        let m1: &[u8] = &[b'B', b'y', b'e', NEW_LINE, b'x'];
        let t = text(&[m0, m1]);
        let ctx = MsgContext {
            player_name: "Phil".into(),
            tail: "hmm".into(),
            ..default_ctx()
        };
        let pages: Vec<String> = t
            .pages(0, &ctx, &mut |_| 0)
            .iter()
            .map(|p| markup::strip(p))
            .collect();
        assert_eq!(pages, vec!["Hi, Phil!", "hmm!", "Bye\nx"]);
    }

    #[test]
    fn random_next_and_choices() {
        let m0: &[u8] = &[
            b'Q',
            CONTROL,
            code::SELECT_2,
            0,
            0,
            0,
            1,
            CONTROL,
            code::NEXT_RANDOM_2,
            0,
            1,
            0,
            2,
        ];
        let mut t = text(&[m0, b"one", b"two"]);
        t.code_size[code::SELECT_2 as usize] = 6;
        let pages = t.pages(0, &default_ctx(), &mut |_| 1);
        // The question stops the page; its answers come back separately.
        assert_eq!(pages, vec!["Q"]);
    }

    #[test]
    fn questions_stop_with_their_branches() {
        let m0: &[u8] = &[
            b'Q',
            CONTROL,
            code::SELECT_2,
            0,
            0,
            0,
            1,
            CONTROL,
            code::NEXT_0,
            0,
            1,
            CONTROL,
            16, // NEXT_1
            0,
            2,
        ];
        let mut t = text(&[m0, b"one", b"two"]);
        t.code_size[code::SELECT_2 as usize] = 6;
        t.code_size[code::NEXT_0 as usize] = 4;
        t.code_size[16] = 4;
        let (pages, choice) = t.talk(0, &default_ctx(), &mut |_| 0);
        assert_eq!(pages, vec!["Q"]);
        let choice = choice.expect("asks");
        assert_eq!(choice.labels, vec!["Yes", "No"]);
        assert_eq!(choice.next, vec![Some(1), Some(2)]);
    }

    /// Probe (needs the disc): a few real messages.
    #[test]
    #[ignore]
    fn probe_disc_messages() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut d = disc::Disc::open_path(std::path::Path::new(&path)).unwrap();
        let rom = disc::RomImage::from_disc(&mut d).unwrap();
        let cat = disc::AssetCatalog::builtin(&rom);
        let t = GameText::read(&mut d, &rom, &cat).unwrap();
        println!("code sizes {:?}", &t.code_size[..30]);
        let ctx = MsgContext {
            player_name: "Phil".into(),
            talk_name: "Bob".into(),
            tail: "pthhpth".into(),
            free: vec!["apple".into(); 20],
            items: vec!["apple".into(); 5],
            month: 10,
            day: 12,
            hour: 19,
            ..MsgContext::default()
        };
        for no in [0x0761usize, 0x0762, 0x0800, 0x1000, 0x2000] {
            println!("--- {no:#06x}");
            for p in t.pages(no, &ctx, &mut |_| 0) {
                println!("| {}", p.replace('\n', "\n| "));
            }
        }
        for i in [0usize, 1, 2, 3] {
            println!("string {i}: {:?}", t.string(i));
        }
    }

    fn default_ctx() -> MsgContext {
        MsgContext {
            month: 1,
            ..MsgContext::default()
        }
    }
}
