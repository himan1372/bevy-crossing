//! Walkable ground from the disc's acre templates: each `data_bgd` entry
//! carries a 16x16 grid of collision words (`mCoBG_Collision_u`), and each
//! acre a height tier from the town generator.
//!
//! World heights follow the game: tier * 120 + sample * 10 (game units,
//! 40 per cell). Scene heights are shifted so the lowest flat ground
//! (sample 4 on tier 0) sits at y = 0.

use bevy::prelude::*;

use crate::town::{ACRE_SIZE, CELL_SIZE};
use rustimal_logic::random_field::{BLOCK_X_NUM, BLOCK_Z_NUM};
use rustimal_logic::town_gen::{ACRE_DEPTH, ACRE_WIDTH};

/// Game units per cell.
const UNIT: f32 = 40.0;
/// Game units per height tier (`3 * mFI_UT_WORLDSIZE_X`).
const TIER_HEIGHT: f32 = 120.0;
/// Game units per collision height sample.
const SAMPLE_HEIGHT: f32 = 10.0;
/// Height of the lowest flat ground, in game units.
const GROUND_ZERO: f32 = 40.0;
/// Highest step the player can walk up or down, in game units.
const MAX_STEP: f32 = 20.0;

/// `mCoBG_ATTRIBUTE_*` values that can't be walked on.
const ATTR_HOLE: u8 = 10;
const ATTR_WAVE: u8 = 11;
const ATTR_WATER: u8 = 12;
const ATTR_WATERFALL: u8 = 13;
const ATTR_RIVER_FIRST: u8 = 14; // RIVER_N .. RIVER_NE
const ATTR_RIVER_LAST: u8 = 21;
const ATTR_SEA: u8 = 24;

/// One collision word, decoded from the GameCube layout (bitfields packed
/// from the most significant bit): slate:1 center:5 top_left:5 bot_left:5
/// bot_right:5 top_right:5 attribute:6.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Unit {
    pub center: u8,
    pub top_left: u8,
    pub bot_left: u8,
    pub bot_right: u8,
    pub top_right: u8,
    pub attribute: u8,
}

impl Unit {
    pub fn from_raw(raw: u32) -> Self {
        let f = |shift: u32| ((raw >> shift) & 0x1F) as u8;
        Self {
            center: f(26),
            top_left: f(21),
            bot_left: f(16),
            bot_right: f(11),
            top_right: f(6),
            attribute: (raw & 0x3F) as u8,
        }
    }

    pub fn walkable(self) -> bool {
        !matches!(
            self.attribute,
            ATTR_HOLE | ATTR_WAVE | ATTR_WATER | ATTR_WATERFALL | ATTR_SEA
        ) && !(ATTR_RIVER_FIRST..=ATTR_RIVER_LAST).contains(&self.attribute)
    }
}

/// The whole 7x10-acre town's collision units and tiers.
#[derive(Clone)]
pub struct GroundMap {
    /// 16x16 units per acre, acres row-major.
    units: Vec<Unit>,
    tiers: [u8; BLOCK_X_NUM * BLOCK_Z_NUM],
}

impl GroundMap {
    /// `units[acre][z * 16 + x]`, `tiers[acre]`, acres row-major 7x10.
    pub fn new(units: Vec<[Unit; 256]>, tiers: [u8; BLOCK_X_NUM * BLOCK_Z_NUM]) -> Self {
        Self {
            units: units.into_iter().flatten().collect(),
            tiers,
        }
    }

    /// Scene-space north-west corner of block (bx, bz). Blocks (1..=5,
    /// 1..=6) are the playable acres of the town plan.
    pub fn block_corner(bx: usize, bz: usize) -> Vec2 {
        Vec2::new(
            (bx as f32 - 1.0) * ACRE_SIZE - ACRE_WIDTH as f32 * ACRE_SIZE / 2.0,
            (bz as f32 - 1.0) * ACRE_SIZE - ACRE_DEPTH as f32 * ACRE_SIZE / 2.0,
        )
    }

    /// Scene y offset that lifts an acre model to its tier.
    pub fn tier_offset(tier: u8) -> f32 {
        tier as f32 * TIER_HEIGHT / UNIT * CELL_SIZE
    }

    /// The unit under a scene position, its acre tier and the position
    /// inside the unit (0..1, 0..1).
    fn locate(&self, x: f32, z: f32) -> Option<(Unit, u8, f32, f32)> {
        let origin = Self::block_corner(0, 0);
        let (ux, uz) = ((x - origin.x) / CELL_SIZE, (z - origin.y) / CELL_SIZE);
        if ux < 0.0 || uz < 0.0 {
            return None;
        }
        let (ix, iz) = (ux as usize, uz as usize);
        let (bx, bz) = (ix / 16, iz / 16);
        if bx >= BLOCK_X_NUM || bz >= BLOCK_Z_NUM {
            return None;
        }
        let acre = bz * BLOCK_X_NUM + bx;
        let unit = self.units.get(acre * 256 + (iz % 16) * 16 + ix % 16)?;
        Some((*unit, self.tiers[acre], ux.fract(), uz.fract()))
    }

    /// Ground height (scene units) at a scene position.
    pub fn height(&self, x: f32, z: f32) -> Option<f32> {
        let (u, tier, fx, fz) = self.locate(x, z)?;
        let s = |v: u8| v as f32;
        // Bilinear over the four corner samples.
        let north = s(u.top_left) * (1.0 - fx) + s(u.top_right) * fx;
        let south = s(u.bot_left) * (1.0 - fx) + s(u.bot_right) * fx;
        let sample = north * (1.0 - fz) + south * fz;
        let game_y = tier as f32 * TIER_HEIGHT + sample * SAMPLE_HEIGHT;
        Some((game_y - GROUND_ZERO) / UNIT * CELL_SIZE)
    }

    /// Whether the ground at a scene position can be stood on.
    pub fn walkable_at(&self, x: f32, z: f32) -> bool {
        self.locate(x, z).is_some_and(|(u, ..)| u.walkable())
    }

    /// Centre of the first walkable unit in block (bx, bz), scanning
    /// outward from the acre's middle.
    pub fn walkable_spot_in(&self, bx: usize, bz: usize) -> Option<Vec2> {
        let corner = Self::block_corner(bx, bz);
        let mut spots: Vec<(i32, i32)> =
            (0..16).flat_map(|z| (0..16).map(move |x| (x, z))).collect();
        spots.sort_by_key(|&(x, z)| (x - 8).pow(2) + (z - 8).pow(2));
        spots
            .into_iter()
            .map(|(x, z)| corner + Vec2::new(x as f32 + 0.5, z as f32 + 0.5) * CELL_SIZE)
            .find(|p| self.walkable_at(p.x, p.y))
    }

    /// Whether a step from `from` to `to` stays on walkable ground without
    /// climbing or dropping a cliff.
    pub fn can_step(&self, from: Vec3, to: Vec3) -> bool {
        let (Some((unit, ..)), Some(h0), Some(h1)) = (
            self.locate(to.x, to.z),
            self.height(from.x, from.z),
            self.height(to.x, to.z),
        ) else {
            return false;
        };
        unit.walkable() && (h1 - h0).abs() * UNIT / CELL_SIZE <= MAX_STEP
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(center: u32, corners: [u32; 4], attr: u32) -> u32 {
        center << 26
            | corners[0] << 21
            | corners[1] << 16
            | corners[2] << 11
            | corners[3] << 6
            | attr
    }

    #[test]
    fn decodes_msb_first_layout() {
        let u = Unit::from_raw(raw(4, [1, 2, 3, 5], ATTR_WATER as u32));
        assert_eq!(
            (u.center, u.top_left, u.bot_left, u.bot_right, u.top_right),
            (4, 1, 2, 3, 5)
        );
        assert!(!u.walkable());
        assert!(Unit::from_raw(raw(4, [4; 4], 0)).walkable());
    }

    #[test]
    fn heights_and_cliffs() {
        let flat = Unit::from_raw(raw(4, [4; 4], 0));
        let mut tiers = [0u8; BLOCK_X_NUM * BLOCK_Z_NUM];
        tiers[BLOCK_X_NUM + 2] = 1; // block (2, 1) is one tier up
        let map = GroundMap::new(vec![[flat; 256]; BLOCK_X_NUM * BLOCK_Z_NUM], tiers);
        let c = GroundMap::block_corner(1, 1) + Vec2::splat(0.5);
        assert!(map.height(c.x, c.y).unwrap().abs() < 1e-4);
        let up = GroundMap::block_corner(2, 1) + Vec2::splat(0.5);
        assert!((map.height(up.x, up.y).unwrap() - 3.0 * CELL_SIZE).abs() < 1e-4);
        let edge = GroundMap::block_corner(2, 1) - Vec2::new(0.1, -0.5);
        let across = GroundMap::block_corner(2, 1) + Vec2::new(0.1, 0.5);
        let (a, b) = (
            Vec3::new(edge.x, 0.0, edge.y),
            Vec3::new(across.x, 0.0, across.y),
        );
        assert!(!map.can_step(a, b), "can't walk up a cliff");
        assert!(map.can_step(a, a + Vec3::new(0.0, 0.0, 0.5)));
    }
}
