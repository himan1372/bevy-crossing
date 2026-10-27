//! Safe native diary kernels reconciled from Rustimal 122867f:diary.rs and
//! m_diary.c / m_diary_ovl.c. No C ABI, card I/O or test harness is imported.

pub const ENTRY_SIZE: usize = 992;
pub const MONTHS: usize = 12;
pub const SPACE: u8 = 0x20;
pub const NEWLINE: u8 = 205;
pub const WRAP_WIDTH: usize = 192;
pub const VISIBLE_LINES: usize = 6;

/// mDi_strlen scans backward for the last byte other than the supplied padding.
pub fn text_len(text: &[u8]) -> usize {
    text.iter().rposition(|&c| c != SPACE).map_or(0, |i| i + 1)
}

/// mDi_init_diary fills all 992 bytes with spaces, not zeroes.
pub fn entry(text: &[u8]) -> Vec<u8> {
    let mut bytes = vec![SPACE; ENTRY_SIZE];
    let len = text.len().min(ENTRY_SIZE);
    bytes[..len].copy_from_slice(&text[..len]);
    bytes
}

/// mDI_strLineCheck as byte ranges: explicit newlines are consumed, while a width
/// overflow starts the next line at the overflowing character (strictly greater
/// than 192).
pub fn line_ranges(text: &[u8], widths: &[u8; 256]) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut width = 0;
    for (i, &code) in text.iter().enumerate() {
        if code == NEWLINE {
            result.push((start, i));
            start = i + 1;
            width = 0;
        } else {
            let next = usize::from(widths[code as usize]);
            if width + next > WRAP_WIDTH && i > start {
                result.push((start, i));
                start = i;
                width = 0;
            }
            width += next;
        }
    }
    result.push((start, text.len()));
    result
}

pub fn lines<'a>(text: &'a [u8], widths: &[u8; 256]) -> Vec<&'a [u8]> {
    line_ranges(text, widths)
        .into_iter()
        .map(|(a, b)| &text[a..b])
        .collect()
}

/// The line holding `cursor` and the pixel column in front of it. A cursor at a
/// wrap point sits at the start of the following line.
pub fn cursor_line(text: &[u8], cursor: usize, widths: &[u8; 256]) -> (usize, usize) {
    let ranges = line_ranges(text, widths);
    let line = ranges
        .iter()
        .rposition(|&(start, _)| start <= cursor)
        .unwrap_or(0);
    let (start, _) = ranges[line];
    let column = text[start..cursor.min(text.len())]
        .iter()
        .map(|&c| usize::from(widths[c as usize]))
        .sum();
    (line, column)
}

/// mDI_move_cursol_upper/lower and upper10/lower10, six visible lines.
pub fn scroll(current: usize, line_count: usize, delta: i32) -> usize {
    current
        .saturating_add_signed(delta as isize)
        .min(line_count.saturating_sub(VISIBLE_LINES))
}

/// The source reader/editor draws at most 31 body lines.
pub fn can_append(text: &[u8], code: u8, widths: &[u8; 256]) -> bool {
    if text.len() >= ENTRY_SIZE {
        return false;
    }
    let mut candidate = text.to_vec();
    candidate.push(code);
    lines(&candidate, widths).len() <= 31
}

/// mDI_move_Obey / pc_di_confirm_branch: Yes writes, Rewrite or B re-edits.
pub fn confirm_writes(choice: usize) -> bool {
    choice == 0
}

/// What a multi-line entry may hold: the diary (992 bytes, 31 lines) or a letter
/// body (`MAIL_BODY_LEN` 192 bytes, `mBD_BODY_LINE_NUM` 6 lines).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub bytes: usize,
    pub lines: usize,
}

pub const DIARY: Limits = Limits {
    bytes: ENTRY_SIZE,
    lines: 31,
};
pub const LETTER: Limits = Limits {
    bytes: 192,
    lines: 6,
};

/// mED_input_multi_line: a code goes in at the cursor while the entry still fits
/// (992 bytes, at most 31 lines).
pub fn insert(
    text: &mut Vec<u8>,
    cursor: &mut usize,
    code: u8,
    widths: &[u8; 256],
    limits: Limits,
) -> bool {
    let at = (*cursor).min(text.len());
    if text.len() >= limits.bytes {
        return false;
    }
    let mut candidate = text.clone();
    candidate.insert(at, code);
    if lines(&candidate, widths).len() > limits.lines {
        return false;
    }
    *text = candidate;
    *cursor = at + 1;
    true
}

/// mED_backspace_func: removes the code before the cursor.
pub fn backspace(text: &mut Vec<u8>, cursor: &mut usize) -> bool {
    let at = (*cursor).min(text.len());
    if at == 0 {
        return false;
    }
    text.remove(at - 1);
    *cursor = at - 1;
    true
}

/// mED_move_cursol_left.
pub fn left(cursor: &mut usize) -> bool {
    if *cursor == 0 {
        return false;
    }
    *cursor -= 1;
    true
}

/// mED_move_cursol_right: past the end it adds a space instead.
pub fn right(text: &mut Vec<u8>, cursor: &mut usize, widths: &[u8; 256], limits: Limits) -> bool {
    if *cursor < text.len() {
        *cursor += 1;
        true
    } else {
        insert(text, cursor, SPACE, widths, limits)
    }
}

/// The index on `line` whose column is nearest at or before `column`.
fn index_at(text: &[u8], range: (usize, usize), column: usize, widths: &[u8; 256]) -> usize {
    let (start, end) = range;
    let mut x = 0;
    for i in start..end {
        let next = x + usize::from(widths[text[i] as usize]);
        if next > column {
            return i;
        }
        x = next;
    }
    end
}

/// mED_move_cursol_upper: one line up, keeping the pixel column.
pub fn upper(text: &[u8], cursor: &mut usize, widths: &[u8; 256]) -> bool {
    let (line, column) = cursor_line(text, *cursor, widths);
    if line == 0 {
        return false;
    }
    let ranges = line_ranges(text, widths);
    *cursor = index_at(text, ranges[line - 1], column, widths);
    true
}

/// mED_move_cursol_lower: one line down; at the very end it adds a newline.
pub fn lower(text: &mut Vec<u8>, cursor: &mut usize, widths: &[u8; 256], limits: Limits) -> bool {
    let (line, column) = cursor_line(text, *cursor, widths);
    let ranges = line_ranges(text, widths);
    if line + 1 < ranges.len() {
        *cursor = index_at(text, ranges[line + 1], column, widths);
        true
    } else if *cursor == text.len() {
        insert(text, cursor, NEWLINE, widths, limits)
    } else {
        false
    }
}
