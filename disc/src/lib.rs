//! Reads the player's own Animal Crossing GameCube disc image.
//!
//! Pure Rust, no Bevy. Ported from rustimal-crossing `rust/src/lib.rs`
//! (`Disc`, `FstFile`, `decode_yaz0`) without the C ABI.
//!
//! No disc data ever lives in this repo: tests build tiny synthetic images
//! in memory. Tests that need a real disc are `#[ignore]` and read its path
//! from `BEVY_CROSSING_ISO`.

mod image;
mod rom;
mod yaz0;

pub use image::{find_image, Disc, DiscError, FstFile, GAME_ID_NTSC_U};
pub use rom::{RomImage, RomSource, Target};
pub use yaz0::decode as decode_yaz0;

pub(crate) fn be32(bytes: &[u8]) -> Option<u32> {
    Some(u32::from_be_bytes(bytes.get(..4)?.try_into().ok()?))
}
