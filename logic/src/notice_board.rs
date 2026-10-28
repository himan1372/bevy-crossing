//! The town's notice board (`m_notice.c`, `m_notice_ovl.c`): fifteen posts, oldest first.
//!
//! A post is a letter-sized message (`MAIL_BODY_LEN` 192 bytes, six lines of at most 192
//! pixels, `mNT_MAX_LINES` / `mNT_MAX_WIDTH`) and the time it was written. The editor
//! limits are `diary::LETTER`.

/// `mNtc_BOARD_POST_COUNT`.
pub const POSTS: usize = 15;

/// `mNtc_notice_write`: with room the post goes after the last one; on a full board the
/// first (oldest) post is dropped and the rest move down.
pub fn write<T>(posts: &mut Vec<T>, post: T) {
    if posts.len() >= POSTS {
        posts.remove(0);
    }
    posts.push(post);
}

/// The page shown when the board opens (`mNT_set_init_data`): the newest post, if any.
pub fn first_page(count: usize) -> usize {
    count.saturating_sub(1)
}

/// `mNT_Play_page_read`: how the page moves for a button (Left/Right one page, Down to the
/// first, Up to the newest); `None` when it cannot.
pub fn move_page(now: usize, count: usize, button: Move) -> Option<usize> {
    let last = count.checked_sub(1)?;
    match button {
        Move::Older if now > 0 => Some(now - 1),
        Move::Newer if now < last => Some(now + 1),
        Move::First if now > 0 => Some(0),
        Move::Last if now < last => Some(last),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    Older,
    Newer,
    First,
    Last,
}

/// `mNT_Play_move_Obey` after the end check: 0 posts it, 1 goes back to editing, anything
/// else reads the board again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndChoice {
    Post,
    KeepWriting,
    Cancel,
}

pub fn end_choice(data1: usize) -> EndChoice {
    match data1 {
        0 => EndChoice::Post,
        1 => EndChoice::KeepWriting,
        _ => EndChoice::Cancel,
    }
}

/// The page number of the post being written (`mNtc_BOARD_POST_COUNT` as a page).
pub const WRITE_PAGE: usize = POSTS;

/// `mNT_PLAY_PAGE_*`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Play {
    Read,
    Move,
    ToWrite,
    ToRead,
}

/// The paper's sideways motion and the key hints' drop (`mNT_Play_page_move`, `_to_write`,
/// `_to_read`). `now` is the page that arrives, `disp` the one that leaves; while they differ
/// both are drawn, `disp` 320 pixels to the side of `x`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    pub mode: Play,
    pub now: usize,
    pub disp: usize,
    pub move_time: u32,
    /// `menu_info->position[0]`.
    pub x: f32,
    /// `control_position`: the key hints' height (0 at rest, -100 hidden while writing).
    pub control: f32,
}

impl Motion {
    /// `mNT_set_init_data`.
    pub fn new(page: usize) -> Self {
        Self {
            mode: Play::Read,
            now: page,
            disp: page,
            move_time: 0,
            x: 0.0,
            control: 0.0,
        }
    }

    pub fn idle(&self) -> bool {
        self.mode == Play::Read
    }

    /// `mNT_Play_page_read`, a page button: `step` pages towards newer (positive) or older.
    pub fn start_move(&mut self, step: isize) {
        if step == 0 {
            return;
        }
        self.mode = Play::Move;
        self.move_time = step.unsigned_abs().min(3) as u32;
        self.now = self
            .now
            .saturating_add_signed(step / self.move_time as isize);
        self.x = if step > 0 { 320.0 } else { -320.0 };
    }

    /// A pressed: the blank post slides in from the right.
    pub fn start_write(&mut self) {
        self.mode = Play::ToWrite;
        self.now = WRITE_PAGE;
        self.x = 320.0;
    }

    /// The end check was cancelled: the board slides back in from the left.
    pub fn start_read(&mut self, page: usize) {
        self.mode = Play::ToRead;
        self.now = page;
        self.x = -320.0;
    }

    /// One 60 Hz frame; `count` is the number of posts on the board.
    pub fn tick(&mut self, count: usize) {
        match self.mode {
            Play::Read => {}
            Play::Move => self.tick_move(count),
            Play::ToWrite | Play::ToRead => {
                let hidden = if self.mode == Play::ToWrite {
                    -100.0
                } else {
                    0.0
                };
                let mut x = self.x;
                let mut control = self.control;
                let page_stopped = add_calc(&mut x, 0.0, 0.4, 74.0, 2.5).abs() < 0.1;
                let control_stopped = add_calc(&mut control, hidden, 0.4, 74.0, 2.5).abs() < 0.1;
                self.x = x;
                self.control = control;
                if page_stopped {
                    self.x = 0.0;
                    self.disp = self.now;
                }
                if control_stopped {
                    self.control = hidden;
                }
                if page_stopped && control_stopped {
                    self.mode = Play::Read;
                }
            }
        }
    }

    fn tick_move(&mut self, count: usize) {
        let mut direction = false;
        let done = if self.move_time == 1 {
            let mut x = self.x;
            let left = add_calc(&mut x, 0.0, 0.4, 74.0, 2.5);
            self.x = x;
            left.abs() < 0.1
        } else if self.x > 0.0 {
            direction = true;
            self.x -= 74.0;
            self.x <= 0.0
        } else {
            self.x += 74.0;
            self.x >= 0.0
        };
        if !done {
            return;
        }
        self.move_time -= 1;
        self.disp = self.now;
        if self.move_time == 0 {
            self.x = 0.0;
            self.mode = Play::Read;
        } else if direction {
            self.x += 320.0;
            self.now += (count.saturating_sub(self.now + 1)) / self.move_time as usize;
        } else {
            self.x -= 320.0;
            self.now -= self.now / self.move_time as usize;
        }
    }
}

use crate::room_light::add_calc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_oldest_post_drops_when_the_board_is_full() {
        let mut posts: Vec<u32> = (0..POSTS as u32).collect();
        write(&mut posts, 100);
        assert_eq!(posts.len(), POSTS);
        assert_eq!((posts[0], posts[POSTS - 1]), (1, 100));
        let mut few = vec![1];
        write(&mut few, 2);
        assert_eq!(few, [1, 2]);
    }

    #[test]
    fn pages_move_like_the_overlay() {
        assert_eq!(first_page(0), 0);
        assert_eq!(first_page(4), 3);
        assert_eq!(move_page(3, 4, Move::Older), Some(2));
        assert_eq!(move_page(3, 4, Move::Newer), None);
        assert_eq!(move_page(1, 4, Move::First), Some(0));
        assert_eq!(move_page(0, 4, Move::First), None);
        assert_eq!(move_page(1, 4, Move::Last), Some(3));
        assert_eq!(move_page(0, 0, Move::Last), None);
    }

    #[test]
    fn the_end_check_answers() {
        assert_eq!(end_choice(0), EndChoice::Post);
        assert_eq!(end_choice(1), EndChoice::KeepWriting);
        assert_eq!(end_choice(2), EndChoice::Cancel);
    }

    #[test]
    fn a_single_page_turn_eases_in_and_settles() {
        let mut m = Motion::new(3);
        m.start_move(-1);
        assert_eq!((m.now, m.disp, m.x), (2, 3, -320.0));
        let mut frames = 0;
        while !m.idle() {
            m.tick(4);
            frames += 1;
            assert!(frames < 100);
        }
        assert_eq!((m.now, m.disp, m.x), (2, 2, 0.0));
    }

    #[test]
    fn a_long_jump_takes_three_hops_and_lands_on_the_target() {
        let mut m = Motion::new(0);
        m.start_move(9);
        while !m.idle() {
            m.tick(10);
        }
        assert_eq!(m.now, 9);
        let mut m = Motion::new(9);
        m.start_move(-9);
        while !m.idle() {
            m.tick(10);
        }
        assert_eq!(m.now, 0);
    }

    #[test]
    fn writing_hides_the_hints_and_reading_brings_them_back() {
        let mut m = Motion::new(2);
        m.start_write();
        while !m.idle() {
            m.tick(3);
        }
        assert_eq!((m.now, m.disp, m.control), (WRITE_PAGE, WRITE_PAGE, -100.0));
        m.start_read(2);
        while !m.idle() {
            m.tick(3);
        }
        assert_eq!((m.now, m.disp, m.control), (2, 2, 0.0));
    }
}
