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
}
