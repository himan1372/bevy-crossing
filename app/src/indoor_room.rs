//! A public building's room from its field BG model (grd_post_office,
//! police_indoor...): drawn at an origin away from the town, with where
//! the player may stand taken from the model's own surface.
//!
//! Floor, rugs and doormats are walkable (within 0.4 cells of the floor);
//! counters and furniture rise higher and block; outside the model (the
//! dark sides of a doorway) there is no floor at all. Sampled every 1/8
//! cell, in cells local to the origin.
use bevy::prelude::*;

use crate::disc_gfx::{run_ground_lists, Segments};
use crate::game_disc::GameDisc;
use crate::input::Actions;
use crate::interaction::Facing;
use crate::locomotion::{self, Locomotion};
use crate::n64_material::N64Material;
use crate::town::CELL_SIZE;

/// Game units per cell.
const UNIT: f32 = 40.0;
/// Field BG models are 16x game units; the floor tier sits at y = 640.
const GROUND_SCALE: f32 = CELL_SIZE / 16.0 / UNIT;
const GROUND_BASE_Y: f32 = 640.0;
pub(crate) const PLAYER_RADIUS: f32 = 0.35;

#[derive(Default)]
pub(crate) struct IndoorRoom {
    pub(crate) origin: Vec3,
    min: Vec2,
    max: Vec2,
    columns: usize,
    floor: Vec<bool>,
    /// Surface height above the floor (cells) per sample; NaN: no surface.
    heights: Vec<f32>,
    blocked: Vec<Rect>,
}

impl IndoorRoom {
    /// Draw `opaque` (and `translucent`) at `origin` and measure the floor.
    pub(crate) fn build(
        commands: &mut Commands,
        disc: &GameDisc,
        name: &str,
        origin: Vec3,
        (opaque, translucent): (&str, Option<&str>),
        (meshes, materials, images): (
            &mut Assets<Mesh>,
            &mut Assets<N64Material>,
            &mut Assets<Image>,
        ),
    ) -> Option<Self> {
        let list = |name: &str| disc.catalog.get(name).map(|a| a.offset);
        let opaque = list(opaque)?;
        let translucent = translucent.and_then(list);
        let root = commands
            .spawn((
                Name::new(name.to_string()),
                Transform::from_translation(origin - Vec3::Y * GROUND_BASE_Y * GROUND_SCALE)
                    .with_scale(Vec3::splat(GROUND_SCALE)),
                Visibility::default(),
            ))
            .id();
        let segs = Segments::default();
        for (l, blend) in [(Some(opaque), false), (translucent, true)] {
            let Some(l) = l else { continue };
            let sink = run_ground_lists(&disc.rom, &[l]);
            for (mesh, material) in crate::disc_town::sink_meshes_scaled(
                disc,
                &sink,
                None,
                blend,
                &segs,
                1.0,
                (&mut *meshes, &mut *materials, &mut *images),
            ) {
                commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), ChildOf(root)));
            }
        }
        let surface = run_ground_lists(&disc.rom, &[opaque]);
        let points: Vec<[f32; 3]> = surface
            .parts
            .iter()
            .flat_map(|p| p.vertices.iter().map(|v| v.pos))
            .collect();
        let floor: Vec<&[f32; 3]> = points
            .iter()
            .filter(|p| (p[1] - GROUND_BASE_Y).abs() < 1.0)
            .collect();
        if floor.is_empty() {
            return None;
        }
        let lo = floor
            .iter()
            .fold(Vec2::splat(f32::MAX), |m, p| m.min(Vec2::new(p[0], p[2])));
        let hi = floor
            .iter()
            .fold(Vec2::splat(f32::MIN), |m, p| m.max(Vec2::new(p[0], p[2])));
        let cell = 16.0 * UNIT;
        let step = cell / 8.0;
        let (nx, nz) = (((hi.x - lo.x) / step) as i32, ((hi.y - lo.y) / step) as i32);
        let mut room = IndoorRoom {
            origin,
            min: lo / cell,
            max: hi / cell,
            columns: nx.max(0) as usize,
            floor: vec![false; (nx.max(0) * nz.max(0)) as usize],
            heights: vec![f32::NAN; (nx.max(0) * nz.max(0)) as usize],
            blocked: Vec::new(),
        };
        for z in 0..nz {
            let mut start = None;
            for x in 0..=nx {
                let sx = lo.x + (x as f32 + 0.5) * step;
                let sz = lo.y + (z as f32 + 0.5) * step;
                let height = (x < nx).then(|| surface.height_at(sx, sz)).flatten();
                if let Some(y) = height {
                    room.heights[(z * nx + x) as usize] = (y - GROUND_BASE_Y) / cell;
                }
                if height.is_some_and(|y| ((y - GROUND_BASE_Y) / cell).abs() <= 0.4) {
                    room.floor[(z * nx + x) as usize] = true;
                }
                let raised = height.is_some_and(|y| (y - GROUND_BASE_Y) / cell > 0.4);
                if raised {
                    start.get_or_insert(x);
                } else if let Some(first) = start.take() {
                    room.blocked.push(Rect::new(
                        (lo.x + first as f32 * step) / cell,
                        (lo.y + z as f32 * step) / cell,
                        (lo.x + x as f32 * step) / cell,
                        (lo.y + (z + 1) as f32 * step) / cell,
                    ));
                }
            }
        }
        info!(
            "{name}: floor {:?}..{:?}, {} raised rows",
            room.min,
            room.max,
            room.blocked.len()
        );
        Some(room)
    }

    /// mCoBG_GetBgY_OnlyCenter: the surface height (cells above the floor)
    /// under a room-local point, e.g. a table top.
    pub(crate) fn height_at(&self, p: Vec2) -> Option<f32> {
        let cell = (p - self.min) * 8.0;
        if cell.x < 0.0 || cell.y < 0.0 || cell.x as usize >= self.columns {
            return None;
        }
        let h = *self
            .heights
            .get(cell.y as usize * self.columns + cell.x as usize)?;
        (!h.is_nan()).then_some(h)
    }

    fn floor_at(&self, p: Vec2) -> bool {
        let cell = (p - self.min) * 8.0;
        if cell.x < 0.0 || cell.y < 0.0 {
            return false;
        }
        let (x, z) = (cell.x as usize, cell.y as usize);
        x < self.columns
            && self
                .floor
                .get(z * self.columns + x)
                .copied()
                .unwrap_or(false)
    }

    /// The player's circle stands on floor all round, clear of furniture.
    pub(crate) fn free(&self, p: Vec2) -> bool {
        [Vec2::ZERO, Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y]
            .iter()
            .all(|d| self.floor_at(p + *d * PLAYER_RADIUS))
            && !self.blocked.iter().any(|r| {
                p.x + PLAYER_RADIUS > r.min.x
                    && p.x - PLAYER_RADIUS < r.max.x
                    && p.y + PLAYER_RADIUS > r.min.y
                    && p.y - PLAYER_RADIUS < r.max.y
            })
    }

    /// One frame of walking, each axis tried on its own.
    pub(crate) fn walk(
        &self,
        actions: &Actions,
        seconds: f32,
        (transform, facing, loco): (&mut Transform, &mut Facing, &mut Locomotion),
    ) {
        let delta = locomotion::step(actions, seconds, facing, loco);
        if delta == Vec3::ZERO {
            return;
        }
        let at = |t: &Transform| {
            Vec2::new(
                t.translation.x - self.origin.x,
                t.translation.z - self.origin.z,
            )
        };
        if self.free(at(transform) + Vec2::new(delta.x, 0.0)) {
            transform.translation.x += delta.x;
        }
        if self.free(at(transform) + Vec2::new(0.0, delta.z)) {
            transform.translation.z += delta.z;
        }
    }
}
