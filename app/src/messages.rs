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

/// `mFont_CONT_CODE_*` values used here.
mod code {
    pub const LAST: u8 = 0;
    pub const CONTINUE: u8 = 1;
    pub const CLEAR: u8 = 2;
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

/// Characters below 0x20 and the symbols placed among ASCII
/// (`CHAR_*` in `m_font.h`); everything else not listed is plain ASCII.
fn char_of(c: u8) -> Option<char> {
    const LOW: [char; 31] = [
        '¡', '¿', 'Ä', 'À', 'Á', 'Â', 'Ã', 'Å', 'Ç', 'È', 'É', 'Ê', 'Ë', 'Ì', 'Í', 'Î', 'Ï', 'Ð',
        'Ñ', 'Ò', 'Ó', 'Ô', 'Õ', 'Ö', 'Œ', 'Ù', 'Ú', 'Û', 'Ü', 'ß', 'Þ',
    ];
    Some(match c {
        0..=30 => LOW[c as usize],
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
    table.get(&ch).copied().unwrap_or(b'?')
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
    /// Choices are listed on the page that asks; the first choice's
    /// branch is followed. `pick(n)` is a random 0..n.
    pub fn pages(
        &self,
        msg_no: usize,
        ctx: &MsgContext,
        pick: &mut dyn FnMut(u32) -> u32,
    ) -> Vec<String> {
        let mut pages = vec![String::new()];
        let mut next = Some(msg_no);
        let mut followed = 0;
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
            next = self.render(text, ctx, &mut pages, pick);
        }
        pages.retain(|p| !p.trim().is_empty());
        pages.iter().map(|p| p.trim_end().to_string()).collect()
    }

    /// Appends one message to `pages`; returns the message it continues to.
    fn render(
        &self,
        text: &[u8],
        ctx: &MsgContext,
        pages: &mut Vec<String>,
        pick: &mut dyn FnMut(u32) -> u32,
    ) -> Option<usize> {
        let mut next = None;
        let mut capital = false;
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
                        code::CONTINUE | code::CLEAR => pages.push(String::new()),
                        code::NEXT_F | code::NEXT_0 => {
                            next = Some(u16_at(i + 2))
                                .filter(|&n| n != 0xFFFF)
                                .map(usize::from);
                        }
                        // Branches for later choices: we follow choice 0.
                        16..=code::NEXT_3 | code::NEXT_4 | code::NEXT_5 => {}
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
                            let labels: Vec<String> = (0..n)
                                .filter_map(|j| {
                                    self.choices.get(u16_at(i + 2 + j * 2) as usize).map(plain)
                                })
                                .collect();
                            page.push_str(&format!("\n[{}]", labels.join(" / ")));
                        }
                        code::PLAYER_NAME => put(&ctx.player_name, page),
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
        next
    }
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
        let pages = t.pages(0, &ctx, &mut |_| 0);
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
        assert_eq!(pages, vec!["Q\n[Yes / No]", "two"]);
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
