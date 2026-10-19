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

/// mDI_strLineCheck: explicit newlines are consumed, while a width overflow
/// starts the next line at the overflowing character (strictly greater than192).
pub fn lines<'a>(text: &'a [u8], widths: &[u8; 256]) -> Vec<&'a [u8]> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut width = 0;
    for (i, &code) in text.iter().enumerate() {
        if code == NEWLINE {
            result.push(&text[start..i]);
            start = i + 1;
            width = 0;
        } else {
            let next = usize::from(widths[code as usize]);
            if width + next > WRAP_WIDTH && i > start {
                result.push(&text[start..i]);
                start = i;
                width = 0;
            }
            width += next;
        }
    }
    result.push(&text[start..]);
    result
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
