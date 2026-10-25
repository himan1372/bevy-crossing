//! The built room, kept stable until restart if the shop upgrades during play.
use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Default)]
pub(crate) struct Layout {
    pub nookway: bool,
}

impl Layout {
    pub fn centre(self) -> Vec3 {
        let half = if self.nookway { 6.0 } else { 4.0 };
        Vec3::new(half, 1.0, half)
    }
    pub fn half(self) -> f32 {
        self.centre().x - 0.875
    }
    pub fn door_x(self) -> f32 {
        if self.nookway {
            2.0
        } else {
            0.0
        }
    }
    pub fn fg(self) -> u16 {
        if self.nookway {
            0x24
        } else {
            0x22
        }
    }
    pub fn floor(self) -> &'static str {
        if self.nookway {
            "rom_shop3f_model"
        } else {
            "rom_shop1f_model"
        }
    }
    pub fn wall(self) -> &'static str {
        if self.nookway {
            "rom_shop3w_model"
        } else {
            "rom_shop1w_model"
        }
    }
}
