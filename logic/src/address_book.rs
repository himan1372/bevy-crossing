//! The letter address book (`m_address_ovl.c`), adapted from rustimal's
//! `address_book.rs` (122867f7: `pc_ad_player_list`, `pc_ad_npc_pages`,
//! `pc_ad_page_model`) as safe Rust.
//!
//! It is rebuilt each time it opens, not saved: the first page holds the
//! town's other players and then the museum once its address is known
//! (`mPr_CheckMuseumAddress`); the next two hold, in house order, the
//! villagers who remember the player (`mNpc_GetAnimalMemoryIdx`), eight
//! to a page. With nobody to write to it refuses (`mAD_PROC_REFUSE`).

/// `mAD_PAGE_MAX_ENTRIES`.
pub const PAGE_MAX_ENTRIES: usize = 8;

/// The three pages: players and the museum, then villagers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddressBook<T> {
    pub pages: [Vec<T>; 3],
}

impl<T> AddressBook<T> {
    /// `mAD_make_player_address` + `mAD_make_npc_address`.
    pub fn new(players: Vec<T>, museum: Option<T>, villagers: Vec<T>) -> Self {
        let mut first = players;
        first.extend(museum);
        let mut villagers = villagers;
        let second = villagers
            .drain(..villagers.len().min(PAGE_MAX_ENTRIES))
            .collect();
        villagers.truncate(PAGE_MAX_ENTRIES);
        Self {
            pages: [first, second, villagers],
        }
    }

    /// `mAD_PROC_REFUSE`: no page has anyone.
    pub fn is_empty(&self) -> bool {
        self.pages.iter().all(Vec::is_empty)
    }

    /// `pc_ad_page_model`'s first page: the players' page when it has
    /// anyone, else the first villager page.
    pub fn first_page(&self) -> usize {
        if self.pages[0].is_empty() {
            1
        } else {
            0
        }
    }

    /// Every entry, page by page.
    pub fn entries(&self) -> impl Iterator<Item = &T> {
        self.pages.iter().flatten()
    }
}
