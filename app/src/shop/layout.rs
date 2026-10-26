//! Native scene selection (aSI_disp_data_table and FG_TYPE_ROM_SHOP*).
use bevy::prelude::*;
pub(crate) mod follow;

#[derive(Resource, Clone, Copy, Default, PartialEq, Eq, Debug)]
pub(crate) struct Layout {
    /// Zakka, Combini, Super, Depart.
    pub tier: u8,
    pub upper: bool,
}

impl Layout {
    pub fn index(self) -> usize {
        if self.tier == 3 && self.upper {
            4
        } else {
            self.tier.min(3) as usize
        }
    }
    pub fn centre(self) -> Vec3 {
        match self.index() {
            0 => Vec3::new(4.0, 1.0, 4.0),
            1 => Vec3::new(6.0, 1.0, 4.0),
            3 | 4 => Vec3::new(6.0, 1.0, 8.0),
            _ => Vec3::new(6.0, 1.0, 6.0),
        }
    }
    pub fn half_x(self) -> f32 {
        self.centre().x - 0.875
    }
    pub fn half_z(self) -> f32 {
        if self.tier == 3 {
            5.125
        } else {
            self.centre().z - 0.875
        }
    }
    pub fn door_x(self) -> f32 {
        if self.tier == 0 {
            0.0
        } else {
            2.0
        }
    }
    /// EXIT_DOOR1 marker centres in the scene's original FG template.
    pub fn door_z(self) -> f32 {
        [8.5, 8.5, 12.5, 14.5, 1.5][self.index()] - self.centre().z
    }
    /// Authored scene player arrival: one native cell before the south exit.
    pub fn entry(self) -> Vec3 {
        if self.upper {
            return self.stair_arrival();
        }
        let (x, z) = [(4.0, 7.5), (8.0, 7.5), (8.0, 11.5), (8.0, 13.5)][self.tier.min(3) as usize];
        Vec3::new(x, 1.0, z) - self.centre().with_y(0.0)
    }
    pub fn fg(self) -> u16 {
        0x22 + self.index() as u16
    }
    pub fn raffle_fg(self) -> u16 {
        0x14D + self.tier.min(3) as u16
    }
    pub fn floor(self) -> &'static str {
        [
            "rom_shop1f_model",
            "rom_shop2f_model",
            "rom_shop3f_model",
            "rom_shop4_1_model",
            "rom_shop4_2f_model",
        ][self.index()]
    }
    pub fn wall(self) -> &'static str {
        [
            "rom_shop1w_model",
            "rom_shop2w_model",
            "rom_shop3w_model",
            "",
            "rom_shop4_2w_model",
        ][self.index()]
    }
    pub fn floor_transparent(self) -> &'static str {
        [
            "rom_shop1f_modelT",
            "rom_shop2f_modelT",
            "rom_shop3f_modelT",
            "rom_shop4_1_modelT",
            "rom_shop4_2f_modelT",
        ][self.index()]
    }
    pub fn wall_transparent(self) -> &'static str {
        [
            "rom_shop1w_modelT",
            "rom_shop2w_modelT",
            "rom_shop3w_modelT",
            "",
            "rom_shop4_2w_modelT",
        ][self.index()]
    }
    pub fn raffle_model(self) -> &'static str {
        [
            "rom_shop1_fuku_model",
            "rom_shop2_fuku_model",
            "rom_shop3_fuku_model",
            "rom_shop4_fuku_model",
        ][self.tier.min(3) as usize]
    }
    pub fn raffle_transparent(self) -> &'static str {
        [
            "rom_shop1_fuku_modelT",
            "rom_shop2_fuku_modelT",
            "rom_shop3_fuku_modelT",
            "rom_shop4_fuku_modelT",
        ][self.tier.min(3) as usize]
    }
    pub fn patterned(self) -> bool {
        self.index() != 3
    }
    /// Native door destinations in shop04_1f/2f.c, in game units.
    pub fn stair_arrival(self) -> Vec3 {
        let native = if self.upper {
            Vec3::new(300.0, 40.0, 65.0)
        } else {
            Vec3::new(280.0, 40.0, 65.0)
        };
        native / 40.0 - self.centre().with_y(0.0)
    }
    pub fn npc(self) -> u32 {
        if self.tier == 3 && self.upper {
            276
        } else {
            238 + 8 + u32::from(self.tier.min(3))
        }
    }
    /// mFM move-actor tables: each coordinate is the centre of an authored UT.
    pub fn npc_position(self) -> Vec3 {
        let (x, z) = [(3.5, 5.5), (7.5, 5.5), (8.5, 9.5), (7.5, 11.5), (7.5, 4.5)][self.index()];
        Vec3::new(x, 1.0, z) - self.centre().with_y(0.0)
    }
    pub fn sibling_position(self) -> Vec3 {
        Vec3::new(8.5, 1.0, 4.5) - self.centre().with_y(0.0)
    }
    pub fn sibling_npc(self) -> u32 {
        293
    }
    /// ac_shop_design garagara_set_table / aSHM_set_base_position.
    pub fn raffle_shift(self) -> Vec3 {
        Vec3::new(
            if self.tier == 0 { 0.0 } else { 80.0 },
            0.0,
            if self.tier >= 2 { 160.0 } else { 0.0 },
        )
    }
    pub fn raffle_nook(self) -> Vec2 {
        crate::raffle::NOOK_AT_DRUM + self.raffle_shift().xz()
    }
}
