//! The game's one global `qrand` state (`libc64/qrand.c`), shared through
//! `rustimal_logic::qrand::Qrand`. The original seeds it at boot with the console
//! clock; this seeds it from the system clock.

use bevy::prelude::*;
use rustimal_logic::qrand::Qrand;

#[derive(Resource)]
pub(crate) struct GameRand(pub Qrand);

impl Default for GameRand {
    fn default() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(1, |d| d.subsec_nanos() ^ d.as_secs() as u32);
        Self(Qrand::new(nanos))
    }
}
