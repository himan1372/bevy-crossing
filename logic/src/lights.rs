//! Per-object scene lights (`m_lights.c`), from pinned Rustimal
//! `122867f7:rust/src/lights.rs`, verified against ac-decomp and
//! ACGC-PC-Port (identical).
//!
//! - `LightsN` collects at most 7 lights plus the ambient
//!   (`LightsN_new_diffuse` drops the 8th onwards silently).
//! - `LightsN_list_check(pos != NULL)` turns a point light into a
//!   directional one relative to the object: inside `radius`, colour is
//!   scaled by `1 - (d/r)^2` and the direction is the object->light vector
//!   scaled to length 120 (or x120 when closer than 1 unit).
//!
//! Only the relative path has a caller (furniture via `aMR_FurnitureLightSet`,
//! actors via `Actor_draw`). The source's `Light_buffer` pool, absolute N64
//! point light and glow-sprite scale are left in the pinned module.

pub const MAX_DIFFUSE: usize = 7;

/// `LightPoint` (without the glow flag, which only affects sprites).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LightPoint {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub color: [u8; 3],
    pub radius: i16,
}

/// One computed per-object directional light (`Light_new`): colour and the
/// direction toward the light, in the source's s8-ish range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightOut {
    pub color: [u8; 3],
    pub dir: [i32; 3],
}

/// `LightsN`: ambient + up to 7 lights.
#[derive(Clone, Debug, PartialEq)]
pub struct LightsN {
    pub ambient: [u8; 3],
    pub lights: Vec<LightOut>,
}

impl LightsN {
    /// `new_LightsN` / `Global_light_read`.
    pub fn new(ambient: [u8; 3]) -> Self {
        Self {
            ambient,
            lights: Vec::with_capacity(MAX_DIFFUSE),
        }
    }

    /// `LightsN__point_proc`: point light seen from `obj` (game units).
    pub fn add_point_relative(&mut self, p: &LightPoint, obj: [f32; 3]) {
        if p.radius <= 0 {
            return;
        }
        let dx = p.x as f32 - obj[0];
        let dy = p.y as f32 - obj[1];
        let dz = p.z as f32 - obj[2];
        let d2 = dx * dx + dy * dy + dz * dz;
        let r = p.radius as f32;
        if d2 >= r * r || self.lights.len() >= MAX_DIFFUSE {
            return;
        }
        let d = d2.sqrt();
        let k = d / r;
        let k = 1.0 - k * k;
        let color = [
            (p.color[0] as f32 * k) as u8,
            (p.color[1] as f32 * k) as u8,
            (p.color[2] as f32 * k) as u8,
        ];
        let s = if d < 1.0 { 120.0 } else { 120.0 / d };
        self.lights.push(LightOut {
            color,
            dir: [(dx * s) as i32, (dy * s) as i32, (dz * s) as i32],
        });
    }
}
