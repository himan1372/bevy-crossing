//! Walkable ground from the disc's acre templates: each `data_bgd` entry
//! carries a 16x16 grid of collision words (`mCoBG_Collision_u`), and each
//! acre a height tier from the town generator.
//!
//! World heights follow the game: tier * 120 + sample * 10 (game units,
//! 40 per cell). Scene heights are shifted so the lowest flat ground
//! (sample 4 on tier 0) sits at y = 0.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::town::{ACRE_SIZE, CELL_SIZE};
use rustimal_logic::random_field::{BLOCK_X_NUM, BLOCK_Z_NUM};
use rustimal_logic::town_gen::{ACRE_DEPTH, ACRE_WIDTH};
use rustimal_logic::{
    decode_height, forbid_vectors, search_slate_detail, search_wall_flag, wall_name,
    MAKE_VECTOR_TABLE,
};

/// Game units per cell.
const UNIT: f32 = 40.0;
/// Game units per height tier (`3 * mFI_UT_WORLDSIZE_X`).
const TIER_HEIGHT: f32 = 120.0;
/// Largest corner rise (samples) of a walkable triangle: ramps rise 2-3
/// per unit, bank drops 4, cliff faces 12.
const MAX_TRIANGLE_RISE: u8 = 3;
/// Edge height difference (game units) still stepped over; banks are
/// 40, cliffs 120.
const STEP_TOLERANCE: f32 = 20.0;
/// Game units per collision height sample.
const SAMPLE_HEIGHT: f32 = 10.0;
/// Height of the lowest flat ground, in game units.
const GROUND_ZERO: f32 = 40.0;

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

    /// Synthetic attribute walls (`mCoBG_MakeForbidAttrVector`) as wall
    /// names: UP/LEFT/DOWN/RIGHT edges, SLATE_UP/SLATE_DOWN diagonals.
    fn attribute_walls(self) -> impl Iterator<Item = u8> {
        forbid_vectors(self.attribute)
            .into_iter()
            .map(|v| MAKE_VECTOR_TABLE[v].wall_name)
    }

    fn has_wall(self, name: u8) -> bool {
        self.attribute_walls().any(|w| w == name)
    }

    /// How the unit splits into triangles: along the top-right/bottom-left
    /// diagonal (`SLATE_UP`) or top-left/bottom-right (`SLATE_DOWN`), from
    /// an attribute diagonal wall or else the corner heights
    /// (`mCoBG_SearchSlateDetail`). `None` for a flat unit.
    fn split(self) -> Option<u8> {
        if let Some(w) = self
            .attribute_walls()
            .find(|&w| w == wall_name::SLATE_UP || w == wall_name::SLATE_DOWN)
        {
            return Some(w);
        }
        let c = [self.top_left, self.top_right, self.bot_left, self.bot_right];
        if c.iter().all(|&v| v == c[0]) {
            return None;
        }
        Some(search_slate_detail(
            self.bot_right as u32,
            self.top_left as u32,
            self.top_right as u32,
            self.bot_left as u32,
        ))
    }

    /// Which triangle (0 or 1) a point (fx, fz in 0..1) is in.
    fn triangle(self, fx: f32, fz: f32) -> u8 {
        match self.split() {
            Some(w) if w == wall_name::SLATE_UP => (fx + fz >= 1.0) as u8,
            Some(_) => (fx < fz) as u8,
            None => 0,
        }
    }

    /// Corner samples (TL, TR, BL, BR) that bound triangle `t`.
    fn triangle_samples(self, t: u8) -> Vec<u8> {
        let (tl, tr, bl, br) = (self.top_left, self.top_right, self.bot_left, self.bot_right);
        match (self.split(), t) {
            (None, _) => vec![tl, tr, bl, br],
            (Some(w), 0) if w == wall_name::SLATE_UP => vec![tl, tr, bl],
            (Some(w), _) if w == wall_name::SLATE_UP => vec![br, tr, bl],
            (Some(_), 0) => vec![tl, tr, br],
            (Some(_), _) => vec![tl, bl, br],
        }
    }

    /// Height sample at (fx, fz) on its triangle's plane.
    fn sample_at(self, fx: f32, fz: f32) -> f32 {
        let (tl, tr, bl, br) = (
            self.top_left as f32,
            self.top_right as f32,
            self.bot_left as f32,
            self.bot_right as f32,
        );
        match (self.split(), self.triangle(fx, fz)) {
            (None, _) => tl,
            (Some(w), 0) if w == wall_name::SLATE_UP => tl + (tr - tl) * fx + (bl - tl) * fz,
            (Some(w), _) if w == wall_name::SLATE_UP => {
                br + (bl - br) * (1.0 - fx) + (tr - br) * (1.0 - fz)
            }
            (Some(_), 0) => tl + (tr - tl) * fx + (br - tr) * fz,
            (Some(_), _) => tl + (bl - tl) * fz + (br - bl) * fx,
        }
    }

    /// Whether triangle `t` can be stood on: standable ground and not a
    /// cliff face or bank drop (ramps rise at most 3 samples per unit).
    fn triangle_walkable(self, t: u8) -> bool {
        let c = self.triangle_samples(t);
        let rise = c.iter().max().unwrap_or(&0) - c.iter().min().unwrap_or(&0);
        // Stone bridge edges (also used by the station stairs) have a four-sample
        // incline. Their attribute walls still block the sides of the stairwell.
        let max_rise = if (32..=35).contains(&self.attribute) {
            4
        } else {
            MAX_TRIANGLE_RISE
        };
        self.walkable() && rise <= max_rise
    }

    /// River or pond water (where fish swim).
    pub fn is_water(self) -> bool {
        self.attribute == ATTR_WATER
            || (ATTR_RIVER_FIRST..=ATTR_RIVER_LAST).contains(&self.attribute)
    }

    /// Ground you can stand on at all: not water, sea, waves or a hole.
    /// Cliffs and banks are handled as walls between units (`can_step`).
    pub fn walkable(self) -> bool {
        !self.is_water()
            && !matches!(
                self.attribute,
                ATTR_HOLE | ATTR_WAVE | ATTR_WATERFALL | ATTR_SEA
            )
    }
}

/// Runtime terrain sound metadata; no audio assets are stored here.
#[derive(Clone, Copy)]
pub(crate) struct TerrainSound {
    pub key: u64,
    pub id: u8,
    pub block: [usize; 2],
    pub position: Vec3,
}

/// The whole 7x10-acre town's collision units and tiers.
#[derive(Clone)]
pub struct GroundMap {
    /// 16x16 units per acre, acres row-major.
    units: Vec<Unit>,
    tiers: [u8; BLOCK_X_NUM * BLOCK_Z_NUM],
    pub(crate) sound_sources: Vec<TerrainSound>,
}

impl GroundMap {
    /// `units[acre][z * 16 + x]`, `tiers[acre]`, acres row-major 7x10.
    pub fn new(units: Vec<[Unit; 256]>, tiers: [u8; BLOCK_X_NUM * BLOCK_Z_NUM]) -> Self {
        Self {
            units: units.into_iter().flatten().collect(),
            tiers,
            sound_sources: Vec::new(),
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

    /// Height in game units at a scene position (the triangle plane).
    fn game_y(&self, x: f32, z: f32) -> Option<f32> {
        let (u, tier, fx, fz) = self.locate(x, z)?;
        Some(tier as f32 * TIER_HEIGHT + u.sample_at(fx, fz) * SAMPLE_HEIGHT)
    }

    /// Water a fish can live in: still water, falls, river, sea.
    pub fn is_fish_water(&self, x: f32, z: f32) -> bool {
        matches!(
            self.attribute_at(x, z),
            Some(ATTR_WATER..=ATTR_RIVER_LAST | ATTR_SEA)
        )
    }

    /// Where a fish swims (scene y): 8 below the water surface
    /// (`aGTT_Get_water_surface_position_y`), the surface being the sea's
    /// 20 or 20 over the bed elsewhere (`mCoBG_GetWaterHeight_File`).
    pub fn fish_y(&self, x: f32, z: f32) -> Option<f32> {
        let surface = match self.attribute_at(x, z)? {
            ATTR_SEA | 37 | 38 => 20.0,
            _ => self.game_y(x, z)? + 20.0,
        };
        Some((surface - 8.0 - GROUND_ZERO) / UNIT * CELL_SIZE)
    }

    /// Sea at least 20 deep (`aSOG_gyoei_set_gyoei_data`'s sea fish rule).
    pub fn deep_sea(&self, x: f32, z: f32) -> bool {
        self.attribute_at(x, z) == Some(ATTR_SEA)
            && self.game_y(x, z).is_some_and(|y| 20.0 - y >= 20.0)
    }

    /// Collision attribute of the unit under a scene position.
    pub fn attribute_at(&self, x: f32, z: f32) -> Option<u8> {
        self.locate(x, z).map(|(u, ..)| u.attribute)
    }

    /// Effective gameplay surface, using the translated retail resolver.
    /// The shoreline uses a neutral wave phase until water animation exists.
    pub fn sound_attribute_at(&self, x: f32, z: f32) -> Option<u8> {
        use rustimal_logic::{wpos2attribute_step, Wpos2AttributeInputs};
        let mut at = Vec2::new(x, z);
        for _ in 0..8 {
            let (unit, _, fx, fz) = self.locate(at.x, at.y)?;
            let local = ((fx - 0.5) * UNIT, (fz - 0.5) * UNIT);
            let area = rustimal_logic::get_unit_area(local.0, local.1);
            let out = wpos2attribute_step(&Wpos2AttributeInputs {
                raw_attr: unit.attribute,
                area,
                local_xz: local,
                local_valid: true,
                is_fg_field: true,
                wave_cos: 0.0,
            });
            let Some(direction) = out.redirect else {
                return Some(out.attr);
            };
            at += match direction {
                0 => Vec2::new(0.0, -CELL_SIZE),
                1 => Vec2::new(-CELL_SIZE, 0.0),
                2 => Vec2::new(0.0, CELL_SIZE),
                _ => Vec2::new(CELL_SIZE, 0.0),
            };
        }
        None
    }

    /// Collision attribute of unit `ut` (row by row) of block (bx, bz).
    pub fn unit_attribute(&self, bx: usize, bz: usize, ut: usize) -> u8 {
        self.units
            .get((bz * BLOCK_X_NUM + bx) * 256 + ut)
            .map_or(0, |u| u.attribute)
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

    /// Global unit coordinates (ux, uz) under a scene position.
    fn unit_coords(&self, x: f32, z: f32) -> Option<(i32, i32)> {
        let origin = Self::block_corner(0, 0);
        let (ux, uz) = ((x - origin.x) / CELL_SIZE, (z - origin.y) / CELL_SIZE);
        let in_town = ux >= 0.0
            && uz >= 0.0
            && ux < (BLOCK_X_NUM * 16) as f32
            && uz < (BLOCK_Z_NUM * 16) as f32;
        in_town.then_some((ux as i32, uz as i32))
    }

    /// The unit at global unit coordinates and its acre's base height
    /// (game units).
    fn unit_at(&self, ux: i32, uz: i32) -> (Unit, f32) {
        let (bx, bz) = (ux as usize / 16, uz as usize / 16);
        let acre = bz * BLOCK_X_NUM + bx;
        let unit = self.units[acre * 256 + (uz as usize % 16) * 16 + ux as usize % 16];
        (unit, self.tiers[acre] as f32 * TIER_HEIGHT)
    }

    /// Ground height (scene units) at a scene position, on the plane of
    /// the triangle it stands in.
    pub fn height(&self, x: f32, z: f32) -> Option<f32> {
        let (u, tier, fx, fz) = self.locate(x, z)?;
        let game_y = tier as f32 * TIER_HEIGHT + u.sample_at(fx, fz) * SAMPLE_HEIGHT;
        Some((game_y - GROUND_ZERO) / UNIT * CELL_SIZE)
    }

    /// River and pond cells of the playable acres as (ax, az, cx, cz),
    /// the town plan's cell indexing (playable acres are blocks 1..=5, 1..=6).
    pub fn water_cells(&self) -> HashSet<(usize, usize, usize, usize)> {
        let mut cells = HashSet::new();
        for az in 0..ACRE_DEPTH {
            for ax in 0..ACRE_WIDTH {
                let acre = (az + 1) * BLOCK_X_NUM + ax + 1;
                for cz in 0..16 {
                    for cx in 0..16 {
                        if self.units[acre * 256 + cz * 16 + cx].is_water() {
                            cells.insert((ax, az, cx, cz));
                        }
                    }
                }
            }
        }
        cells
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

    /// Whether a step from `from` to `to` is allowed: the target must be
    /// standable ground, and crossing into a neighbouring unit must not
    /// hit a terrain wall. As in the game (`mCoBG_SearchWallFlag`), a wall
    /// stands on a unit edge wherever the two units' edge heights differ:
    /// cliffs and river banks; slopes and bridges line up and pass.
    pub fn can_step(&self, from: Vec3, to: Vec3) -> bool {
        let (Some((ua, ..)), Some((ub, _, bfx, bfz))) =
            (self.locate(from.x, from.z), self.locate(to.x, to.z))
        else {
            return false;
        };
        let (Some(a), Some(b)) = (
            self.unit_coords(from.x, from.z),
            self.unit_coords(to.x, to.z),
        ) else {
            return false;
        };
        let target_tri = ub.triangle(bfx, bfz);
        if !ub.triangle_walkable(target_tri) {
            return false;
        }
        if a == b {
            // Crossing this unit's own diagonal: blocked by an attribute
            // diagonal wall.
            let (_, _, afx, afz) = self.locate(from.x, from.z).unwrap_or((ua, 0, 0.0, 0.0));
            let from_tri = ua.triangle(afx, afz);
            return from_tri == target_tri || ua.split().is_none_or(|w| !ua.has_wall(w));
        }
        // Walk the unit grid from `a` to `b`, x first then z, checking
        // every edge crossed (moves are usually within one unit).
        let mut at = a;
        while at != b {
            let step = if at.0 != b.0 {
                ((b.0 - at.0).signum(), 0)
            } else {
                (0, (b.1 - at.1).signum())
            };
            if self.edge_wall(at, step.0, step.1) {
                return false;
            }
            at = (at.0 + step.0, at.1 + step.1);
            if at != b && !self.unit_at(at.0, at.1).0.walkable() {
                return false;
            }
        }
        true
    }

    /// Whether a terrain wall stands between unit `a` and its neighbour in
    /// direction (dx, dz).
    fn edge_wall(&self, a: (i32, i32), dx: i32, dz: i32) -> bool {
        let (ua, base_a) = self.unit_at(a.0, a.1);
        let (ub, base_b) = self.unit_at(a.0 + dx, a.1 + dz);
        let h = |v: u8, base: f32| decode_height(v as u32, base);
        // Shared edge as seen from each side: (start, end) corner heights.
        let (wall, mine, theirs) = match (dx, dz) {
            (1, _) => (
                wall_name::RIGHT,
                [ua.top_right, ua.bot_right],
                [ub.top_left, ub.bot_left],
            ),
            (-1, _) => (
                wall_name::LEFT,
                [ua.top_left, ua.bot_left],
                [ub.top_right, ub.bot_right],
            ),
            (_, 1) => (
                wall_name::DOWN,
                [ua.bot_left, ua.bot_right],
                [ub.top_left, ub.top_right],
            ),
            _ => (
                wall_name::UP,
                [ua.top_left, ua.top_right],
                [ub.bot_left, ub.bot_right],
            ),
        };
        let (h0, h1) = (
            [h(mine[0], base_a), h(mine[1], base_a)],
            [h(theirs[0], base_b), h(theirs[1], base_b)],
        );
        // The game only stops an actor whose body overlaps the wall's
        // height range; small bumps between units are stepped over.
        let rise = (h1[0] - h0[0]).abs().max((h1[1] - h0[1]).abs());
        // Attribute walls on this edge, from either side (bridge rails,
        // pier ends, bank corners).
        let opposite = match wall {
            w if w == wall_name::RIGHT => wall_name::LEFT,
            w if w == wall_name::LEFT => wall_name::RIGHT,
            w if w == wall_name::DOWN => wall_name::UP,
            _ => wall_name::DOWN,
        };
        ua.has_wall(wall)
            || ub.has_wall(opposite)
            || (search_wall_flag(wall, h0, h1).is_some() && rise > STEP_TOLERANCE)
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
        // Multi-unit steps (NPCs) walk the grid instead of recursing.
        assert!(map.can_step(a, a + Vec3::new(-5.0, 0.0, 3.0)));
        assert!(!map.can_step(a, a + Vec3::new(5.0, 0.0, 0.0)));
    }

    /// Unit from (attribute, [TL, TR, BL, BR]) as in the disc probe dump.
    fn unit(attr: u32, [tl, tr, bl, br]: [u32; 4]) -> Unit {
        Unit::from_raw(tl << 26 | tl << 21 | bl << 16 | br << 11 | tr << 6 | attr)
    }

    #[test]
    fn cliff_edge_unit_splits_into_floor_and_face() {
        let u = unit(56, [16, 16, 4, 16]); // from a cliff template
        assert_eq!(u.split(), Some(wall_name::SLATE_DOWN));
        assert!(u.triangle_walkable(u.triangle(0.9, 0.1)), "flat top half");
        assert!(
            !u.triangle_walkable(u.triangle(0.1, 0.9)),
            "cliff face half"
        );
    }

    #[test]
    fn bank_lip_blocks_the_drop() {
        let u = unit(41, [4, 4, 4, 0]); // river-bank lip
        assert!(u.triangle_walkable(u.triangle(0.1, 0.1)));
        assert!(!u.triangle_walkable(u.triangle(0.9, 0.9)));
    }

    #[test]
    fn attribute_edge_wall_blocks_crossing() {
        let flat = unit(0, [4; 4]);
        let railed = unit(32, [4; 4]); // attribute 32: wall on the north edge
        let mut units = vec![[flat; 256]; BLOCK_X_NUM * BLOCK_Z_NUM];
        let acre = BLOCK_X_NUM + 1; // block (1, 1)
        units[acre][5 * 16 + 5] = railed;
        let map = GroundMap::new(units, [0; BLOCK_X_NUM * BLOCK_Z_NUM]);
        let c = GroundMap::block_corner(1, 1);
        let inside = Vec3::new(c.x + 5.5, 0.0, c.y + 5.5);
        let north = Vec3::new(c.x + 5.5, 0.0, c.y + 4.9);
        let east = Vec3::new(c.x + 6.1, 0.0, c.y + 5.5);
        assert!(!map.can_step(inside, north), "rail on the north edge");
        assert!(!map.can_step(north, inside), "from either side");
        assert!(map.can_step(inside, east));
    }
}
