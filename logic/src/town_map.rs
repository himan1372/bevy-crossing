//! The town map overlay's rules (`m_map_ovl.c`), free of the disc and the renderer.
//!
//! Acre pictures by block type, the label shown for the selected acre, the cursor's pulse and
//! glide, and where a villager's house marker sits inside its acre.

use crate::column_sweep::height2get_layer;
use crate::random_field::{Blocks, BLOCK_X_NUM};
use crate::room_light::add_calc;
use crate::town_gen::{
    block_kind, BK_DOCK, BK_DUMP, BK_MUSEUM, BK_POLICE, BK_POSTOFFICE, BK_SHOP, BK_SHRINE,
    BK_STATION, BK_TAILORS,
};

/// `FG_BLOCK_X_NUM` / `FG_BLOCK_Z_NUM`: the acres the map shows.
pub const ACRES_X: usize = 5;
pub const ACRES_Z: usize = 6;
pub const ACRES: usize = ACRES_X * ACRES_Z;
/// `mMP_BLOCK_SIZE_F`: an acre's picture, in screen pixels.
pub const BLOCK_SIZE: f32 = 22.0;
/// `mMP_CURSOR_FRAMES`.
pub const CURSOR_FRAMES: usize = 18;
/// `mFM_BLOCK_TYPE_FLAT`.
pub const FLAT: u8 = 39;
/// `mMP_house_pos_list`'s terminator (`fgblock_name`).
const HOUSE_POS_END: u16 = 0x03B8;
/// `mMP_house_pos_list` entry: name, three (ut_x, ut_z, idx), one pad byte.
const HOUSE_POS_SIZE: usize = 12;

/// `mMP_check_bg_kind`: border cliffs and their transitions and tunnels have no picture.
fn has_picture(block_type: u8) -> bool {
    !(block_type <= 8 || matches!(block_type, 9 | 10 | 61 | 62))
}

/// `mMP_make_max_no_table`: the block type behind each of the thirty acre pictures, row by row.
/// Types without a picture are skipped, not blanked (the source packs the ones it keeps and
/// fills the rest with grass); `bridge` is the built bridge's acre (`bx`, `bz` from 0) and
/// `plus_bridge` the disc's `pluss_bridge` (0xFF: no bridge picture).
pub fn acre_block_types(
    blocks: &Blocks,
    bridge: Option<(usize, usize)>,
    plus_bridge: &[u8],
) -> [u8; ACRES] {
    let mut out = [FLAT; ACRES];
    let mut n = 0;
    for bz in 1..=ACRES_Z {
        for bx in 1..=ACRES_X {
            let t = blocks[bz * BLOCK_X_NUM + bx];
            if !has_picture(t) {
                continue;
            }
            out[n] = match plus_bridge.get(t as usize) {
                Some(&b) if bridge == Some((bx - 1, bz - 1)) && b != 0xFF => b,
                _ => t,
            };
            n += 1;
            if n >= ACRES {
                return out;
            }
        }
    }
    out
}

/// `mMP_LABEL_*`: what the label panel for an acre is about.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Label {
    Npc,
    Player,
    Shop,
    Police,
    Post,
    Shrine,
    Station,
    Junk,
    Museum,
    Needle,
    Port,
}

/// `mMP_set_field_data`: the building label of an acre by its block kind, in the source's order.
pub fn field_label(block_type: u8) -> Option<Label> {
    let kind = block_kind(block_type);
    [
        (BK_SHOP, Label::Shop),
        (BK_POLICE, Label::Police),
        (BK_POSTOFFICE, Label::Post),
        (BK_SHRINE, Label::Shrine),
        (BK_STATION, Label::Station),
        (BK_DUMP, Label::Junk),
        (BK_MUSEUM, Label::Museum),
        (BK_TAILORS, Label::Needle),
        (BK_DOCK, Label::Port),
    ]
    .into_iter()
    .find(|&(bit, _)| kind & bit != 0)
    .map(|(_, label)| label)
}

/// A line of a label: its text and where it starts (`mMP_LabelWord_c`, pixels from the screen
/// centre, y up). `None` text is the residents' names.
pub struct Word {
    pub text: Option<&'static str>,
    pub x: f32,
    pub y: f32,
}

impl Label {
    /// `mMP_Label_c`: the icon's offset and its words.
    pub fn layout(self) -> ((f32, f32), &'static [Word]) {
        const fn word(text: &'static str, x: f32, y: f32) -> Word {
            Word {
                text: Some(text),
                x,
                y,
            }
        }
        const NAMES: [Word; 1] = [Word {
            text: None,
            x: -92.0,
            y: -19.0,
        }];
        const PLAYER: [Word; 1] = [Word {
            text: None,
            x: -92.0,
            y: -21.5,
        }];
        const SHOP: [Word; 1] = [word("Shop", -83.0, -25.0)];
        const POLICE: [Word; 2] = [word("Police", -83.0, -19.0), word("Station", -83.0, -31.0)];
        const POST: [Word; 2] = [word("Post", -83.0, -19.0), word("Office", -83.0, -31.0)];
        const SHRINE: [Word; 2] = [word("Wishing", -83.0, -19.0), word("Well", -83.0, -31.0)];
        const STATION: [Word; 2] = [word("Train", -83.0, -19.0), word("Station", -83.0, -31.0)];
        const JUNK: [Word; 1] = [word("Dump", -83.0, -25.0)];
        const MUSEUM: [Word; 1] = [word("Museum", -83.0, -25.0)];
        const NEEDLE: [Word; 1] = [word("Tailor", -83.0, -25.0)];
        const PORT: [Word; 1] = [word("Dock", -83.0, -25.0)];
        match self {
            Label::Npc => ((-98.0, -24.0), &NAMES),
            Label::Player => ((-98.0, -26.5), &PLAYER),
            Label::Shop => ((-93.0, -30.0), &SHOP),
            Label::Police => ((-93.0, -30.0), &POLICE),
            Label::Post => ((-93.0, -30.0), &POST),
            Label::Shrine => ((-93.0, -30.0), &SHRINE),
            Label::Station => ((-93.0, -31.0), &STATION),
            Label::Junk => ((-93.0, -30.0), &JUNK),
            Label::Museum => ((-93.0, -30.0), &MUSEUM),
            Label::Needle => ((-93.0, -30.0), &NEEDLE),
            Label::Port => ((-93.0, -30.0), &PORT),
        }
    }
}

/// `mMP_set_house_dl`'s three columns and rows of house marks inside an acre.
pub const HOUSE_OFFSET_X: [f32; 3] = [5.0, 13.0, 17.0];
pub const HOUSE_OFFSET_Y: [f32; 3] = [-4.0, -11.0, -18.0];

/// `mMP_house_pos_list`: the three house spots of an acre layout (`fgblock_name`).
#[derive(Clone, Copy, Debug)]
pub struct HousePos {
    pub fgblock: u16,
    /// (ut_x, ut_z, idx).
    pub entries: [(u8, u8, u8); 3],
}

/// Reads `mMP_house_pos_list` as it lies on the disc, up to its terminator.
pub fn parse_house_pos(bytes: &[u8]) -> Vec<HousePos> {
    let mut list = Vec::new();
    for raw in bytes.as_chunks::<HOUSE_POS_SIZE>().0 {
        let fgblock = u16::from_be_bytes([raw[0], raw[1]]);
        if fgblock == HOUSE_POS_END {
            break;
        }
        let e = |i: usize| (raw[2 + i * 3], raw[3 + i * 3], raw[4 + i * 3]);
        list.push(HousePos {
            fgblock,
            entries: [e(0), e(1), e(2)],
        });
    }
    list
}

/// `mMP_set_house_data`: which of the nine marks a house gets, from the house unit's place in
/// its acre layout. Unknown layouts and places use the first entry of the layout (or mark 0).
pub fn house_index(list: &[HousePos], fgblock: u16, ut_x: u8, ut_z: u8) -> u8 {
    let Some(pos) = list.iter().find(|p| p.fgblock == fgblock) else {
        return list.first().map_or(0, |p| p.entries[0].2);
    };
    pos.entries
        .iter()
        .find(|e| e.0 == ut_x && e.1 == ut_z.wrapping_sub(1))
        .unwrap_or(&pos.entries[0])
        .2
}

/// The height of an acre's tier (`mRF_FIELD_STEP*`) in the units `mCoBG_Height2GetLayer` reads.
pub fn tier_height(tier: u8) -> f32 {
    [0.0, 160.0, 320.0][usize::from(tier).min(2)]
}

/// `mMP_check_layer`: the house mark's elevation row (0 low) from the house's height.
pub fn house_layer(height: f32, three_step: bool) -> usize {
    let mut layer = match height2get_layer(height, three_step) {
        0 => 2,
        1 => 1,
        _ => 0,
    };
    if !three_step {
        layer -= 1;
    }
    layer.max(0) as usize
}

/// `mMP_set_cursol_dl`'s green channel of the pulsing prim colour.
pub const CURSOR_G: [u8; CURSOR_FRAMES] = [
    0, 1, 2, 5, 10, 20, 50, 75, 90, 100, 90, 75, 50, 20, 10, 5, 2, 1,
];
/// And its scale.
pub const CURSOR_SCALE: [f32; CURSOR_FRAMES] = [
    1.0, 1.015, 1.03, 1.04, 1.05, 1.06, 1.07, 1.08, 1.09, 1.1, 1.09, 1.08, 1.07, 1.06, 1.05, 1.04,
    1.03, 1.015,
];

/// The map's cursor: the selected acre, where its frame is drawn and its pulse.
#[derive(Clone, Copy, Debug)]
pub struct Cursor {
    pub bx: usize,
    pub bz: usize,
    /// Where the player stands (`player_bx`, `player_bz`).
    pub player: (usize, usize),
    x: f32,
    z: f32,
    target_x: f32,
    target_z: f32,
    frame: f32,
}

impl Cursor {
    /// `mMP_set_init_data`: starts on the player's acre.
    pub fn new(player_bx: usize, player_bz: usize) -> Self {
        let x = player_bx as f32 * BLOCK_SIZE;
        let z = -(player_bz as f32) * BLOCK_SIZE;
        Self {
            bx: player_bx,
            bz: player_bz,
            player: (player_bx, player_bz),
            x,
            z,
            target_x: x,
            target_z: z,
            frame: 0.0,
        }
    }

    /// `mMP_move_Play`: one step of the C-stick; true when it moved (the glide then runs).
    pub fn step(&mut self, dx: i32, dz: i32) -> bool {
        let bx = self.bx as i32 + dx;
        let bz = self.bz as i32 + dz;
        if !(0..ACRES_X as i32).contains(&bx) || !(0..ACRES_Z as i32).contains(&bz) {
            return false;
        }
        self.bx = bx as usize;
        self.bz = bz as usize;
        self.target_x += dx as f32 * BLOCK_SIZE;
        self.target_z -= dz as f32 * BLOCK_SIZE;
        true
    }

    /// `mMP_move_Play`'s pulse: `ticks` 60 Hz frames.
    pub fn pulse(&mut self, ticks: f32) {
        self.frame = (self.frame + ticks).rem_euclid(CURSOR_FRAMES as f32);
    }

    /// `mMP_move_Wait`: one 60 Hz frame of the glide; true when it has settled.
    pub fn glide(&mut self) -> bool {
        let rx = add_calc(&mut self.x, self.target_x, 0.7, 19.0, 1.8);
        let rz = add_calc(&mut self.z, self.target_z, 0.7, 19.0, 1.8);
        rx.abs() < 0.1 && rz.abs() < 0.1
    }

    /// Where the frame is drawn, relative to the map's origin (pixels, y up).
    pub fn position(&self) -> (f32, f32) {
        (self.x, self.z)
    }

    /// Pulse frame, for [`CURSOR_G`] and [`CURSOR_SCALE`].
    pub fn pulse_frame(&self) -> usize {
        (self.frame as usize).min(CURSOR_FRAMES - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn border_types_are_skipped_not_blanked() {
        let mut blocks: Blocks = [FLAT; crate::random_field::BLOCK_TOTAL_NUM];
        blocks[BLOCK_X_NUM + 1] = 0;
        blocks[BLOCK_X_NUM + 2] = 40;
        let out = acre_block_types(&blocks, None, &[]);
        assert_eq!(out[0], 40);
        assert_eq!(out[1], FLAT);
    }

    #[test]
    fn cursor_stays_inside_the_acres() {
        let mut c = Cursor::new(0, 0);
        assert!(!c.step(-1, 0));
        assert!(c.step(1, 0));
        assert_eq!(c.bx, 1);
        while !c.glide() {}
        assert_eq!(c.position(), (BLOCK_SIZE, 0.0));
    }
}
