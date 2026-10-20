//! Room-insect chirp scheduler: `Na_RoomIncectPos` and `Sou_Insect_Lev_Cont`
//! (jaudio_NES game64.c), verified against ACGC-PC-Port.
//!
//! Each frame a cage furniture reports itself with its index (call argument,
//! 0x36-based). A new reporter takes a free slot and a countdown of
//! `(rand & 15) + SE_ROOM_INS_RANDOM_OFFSET[index]`, bumped past any equal
//! countdown (the rescan restarts at `j = 0`, then the loop increments, so
//! slot 0 is skipped on rescans). Its sound starts at once only when no
//! trigger slot holds a room-insect sound; the table's trailing 0 matches
//! empty slots, so in practice a new insect registers silently. On every
//! change of the beat to an even value the countdown drops; at 1 the insect
//! sounds and draws a new countdown. Slots not reported since the previous
//! frame are freed.

/// `SE_ROOM_INS_DATA`: sound per index (call argument - 0x36).
pub const SE_ROOM_INS_DATA: [u16; 17] = [
    0x505, 0x509, 0x50D, 0x511, 0x515, 0x519, 0x51D, 0x521, 0x525, 0x529, 0x501, 0x52D, 0x531,
    0x535, 0x539, 0x53D, 0,
];

/// `SE_ROOM_INS_RANDOM_OFFSET`.
pub const SE_ROOM_INS_RANDOM_OFFSET: [u16; 17] = [
    11, 11, 11, 10, 10, 10, 12, 100, 20, 100, 45, 10, 4, 9, 2, 1, 0,
];

/// `sou_room_ins` size.
pub const SLOTS: usize = 50;

/// `SOU_ROOM_INS`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Slot {
    id: u32,
    calls: u8,
    seen: u8,
    beat: u8,
    countdown: u8,
}

/// `sou_room_ins` plus the two routines that touch it.
#[derive(Clone, Debug)]
pub struct RoomInsects {
    slots: [Slot; SLOTS],
}

impl Default for RoomInsects {
    fn default() -> Self {
        Self {
            slots: [Slot::default(); SLOTS],
        }
    }
}

impl RoomInsects {
    /// `sou_room_ins_init` for every slot.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// The bump loop: `r29` past every equal countdown, `j = 0` then `j++`.
    fn unique(&self, mut value: u8) -> u8 {
        let mut retries = 0u8;
        let mut j = 0usize;
        while j < SLOTS {
            if value == self.slots[j].countdown {
                value = value.wrapping_add(1);
                j = 0;
                retries = retries.wrapping_add(1);
                if usize::from(retries) > SLOTS {
                    retries = 0;
                    j = SLOTS;
                }
            }
            j += 1;
        }
        value
    }

    fn draw(&self, index: usize, rand: u16) -> u8 {
        let value = ((rand & 0xF) as u8).wrapping_add(SE_ROOM_INS_RANDOM_OFFSET[index] as u8);
        self.unique(value)
    }

    /// `Na_RoomIncectPos`. `argument` is the furniture's call value (0x36..),
    /// `beat` the furniture sequence's port 2 (values above 15 read as 0),
    /// `trigger_ids` the trigger slots' sound ids (0 = empty). Returns the
    /// sound to start at the insect's position, which is also the callback's
    /// "valid" result that restarts its clip.
    pub fn report(
        &mut self,
        id: u32,
        argument: u8,
        beat: u8,
        rand: &mut dyn FnMut() -> u16,
        trigger_ids: &[u16],
    ) -> Option<u16> {
        let index = usize::from(argument.wrapping_sub(0x36));
        if id == 0 || index >= SE_ROOM_INS_DATA.len() {
            return None;
        }
        let beat = if beat > 0xF { 0 } else { beat };
        if let Some(found) = self.slots.iter().position(|s| s.id == id) {
            let mut sound = None;
            let slot = self.slots[found];
            if slot.beat != beat && slot.countdown != 0 && beat % 2 == 0 {
                if slot.countdown == 1 {
                    sound = Some(SE_ROOM_INS_DATA[index]);
                    self.slots[found].countdown = self.draw(index, rand());
                }
                self.slots[found].countdown = self.slots[found].countdown.wrapping_sub(1);
            }
            let slot = &mut self.slots[found];
            slot.calls = slot.calls.wrapping_add(1);
            slot.beat = beat;
            return sound;
        }
        let free = self.slots.iter().position(|s| s.id == 0)?;
        let countdown = self.draw(index, rand());
        self.slots[free] = Slot {
            id,
            calls: 1,
            seen: 0,
            beat,
            countdown,
        };
        let sounding = trigger_ids.iter().any(|t| SE_ROOM_INS_DATA.contains(t));
        (!sounding).then_some(SE_ROOM_INS_DATA[index])
    }

    /// `Sou_Insect_Lev_Cont`, once per game frame after the reports.
    pub fn retire(&mut self) {
        for slot in &mut self.slots {
            if slot.calls == slot.seen {
                *slot = Slot::default();
            } else {
                slot.seen = slot.calls;
            }
        }
    }
}
