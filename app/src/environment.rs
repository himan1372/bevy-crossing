//! Time-of-day light, fog and background colour (`m_kankyo.c`), and the
//! building window lights that follow the clock.
//!
//! The light table is the disc's `l_mEnv_kcolor_fine_data` (sunny
//! weather): eight periods starting at 0, 4, 6, 8, 12, 16, 18 and 20
//! o'clock (`klight_chg_tim`), blended linearly toward the next one
//! (`mEnv_SetBaseLight` / `mEnv_CalcSetLight`). Every N64 material gets the
//! blended ambient, sun and moon lights and fog when it changes.

use bevy::prelude::*;
use disc::RomSource;

use crate::clock;
use crate::game_disc::GameDisc;
use crate::n64_material::{EnvUniform, N64Material};

/// Start of each light period, in seconds since midnight, plus the end.
const PERIOD_STARTS: [u32; 9] = [
    0,
    4 * 3600,
    6 * 3600,
    8 * 3600,
    12 * 3600,
    16 * 3600,
    18 * 3600,
    20 * 3600,
    24 * 3600,
];

/// Size of one `BaseLight` entry on the disc.
const BASE_LIGHT_SIZE: usize = 38;

/// One `BaseLight` (`m_kankyo.h`), as floats so periods can be blended.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BaseLight {
    pub ambient: [f32; 3],
    pub sun_dir: [f32; 3],
    pub sun_color: [f32; 3],
    pub moon_dir: [f32; 3],
    pub moon_color: [f32; 3],
    pub fog_color: [f32; 3],
    /// Fog start and end in the N64 depth range 0..1000.
    pub fog_near: f32,
    pub fog_far: f32,
    pub background: [f32; 3],
}

impl BaseLight {
    /// Parses one big-endian `BaseLight`.
    pub fn parse(b: &[u8]) -> Option<Self> {
        if b.len() < BASE_LIGHT_SIZE {
            return None;
        }
        let rgb = |o: usize| [b[o] as f32, b[o + 1] as f32, b[o + 2] as f32];
        let dir = |o: usize| {
            [
                b[o] as i8 as f32,
                b[o + 1] as i8 as f32,
                b[o + 2] as i8 as f32,
            ]
        };
        let s16 = |o: usize| i16::from_be_bytes([b[o], b[o + 1]]);
        Some(Self {
            ambient: rgb(0),
            sun_dir: dir(3),
            sun_color: rgb(6),
            moon_dir: dir(9),
            moon_color: rgb(12),
            fog_color: rgb(15),
            // `mEnv_ChangeFogLight` keeps the low 10 bits of the near value.
            fog_near: (s16(18) & 0x3FF) as f32,
            fog_far: s16(20) as f32,
            background: rgb(34),
        })
    }

    fn lerp(&self, next: &Self, t: f32) -> Self {
        let l3 = |a: [f32; 3], b: [f32; 3]| [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t);
        Self {
            ambient: l3(self.ambient, next.ambient),
            // Directions aren't blended by `mEnv_ChangeDiffuseLight` in the
            // field; they're the same in every period.
            sun_dir: self.sun_dir,
            sun_color: l3(self.sun_color, next.sun_color),
            moon_dir: self.moon_dir,
            moon_color: l3(self.moon_color, next.moon_color),
            fog_color: l3(self.fog_color, next.fog_color),
            fog_near: self.fog_near + (next.fog_near - self.fog_near) * t,
            fog_far: self.fog_far + (next.fog_far - self.fog_far) * t,
            background: l3(self.background, next.background),
        }
    }

    /// The shader's lighting uniform (`mEnv_SetFog` caps near/far).
    pub fn uniform(&self) -> EnvUniform {
        let c = |v: [f32; 3]| Vec4::new(v[0], v[1], v[2], 0.0) / 255.0;
        let d = |v: [f32; 3]| Vec3::from(v).normalize_or_zero().extend(0.0);
        EnvUniform {
            ambient: c(self.ambient),
            sun_dir: d(self.sun_dir),
            sun_color: c(self.sun_color),
            moon_dir: d(self.moon_dir),
            moon_color: c(self.moon_color),
            fog_color: c(self.fog_color),
            fog: Vec4::new(self.fog_near.min(996.0), self.fog_far.min(1600.0), 1.0, 0.0),
        }
    }
}

/// The eight light periods of the sunny-weather table.
#[derive(Resource, Clone, Debug)]
pub struct LightTable(pub [BaseLight; 8]);

impl LightTable {
    /// Reads `l_mEnv_kcolor_fine_data` from the disc.
    pub fn from_disc(disc: &GameDisc) -> Option<Self> {
        let asset = disc.catalog.get("l_mEnv_kcolor_fine_data")?;
        let bytes = disc
            .rom
            .slice(RomSource::Rel, asset.offset, (BASE_LIGHT_SIZE * 8) as u32)?;
        let mut out = [BaseLight::default(); 8];
        for (i, entry) in out.iter_mut().enumerate() {
            *entry = BaseLight::parse(&bytes[i * BASE_LIGHT_SIZE..])?;
        }
        Some(Self(out))
    }

    /// The light at `day_secs` seconds after midnight.
    pub fn at(&self, day_secs: u32) -> BaseLight {
        let s = day_secs % 86_400;
        let period = (0..8).find(|&i| PERIOD_STARTS[i + 1] > s).unwrap_or(7);
        let (start, end) = (PERIOD_STARTS[period], PERIOD_STARTS[period + 1]);
        let t = (s - start) as f32 / (end - start) as f32;
        // The last period runs into the first one of the next day.
        self.0[period].lerp(&self.0[(period + 1) % 8], t)
    }
}

/// Whether building lights are on: villagers switch theirs on from 18:00
/// to 05:00 (`aHUS_ctrl_light`); the other buildings use the same hours here.
pub fn lights_on(hour: u8) -> bool {
    !(5..18).contains(&hour)
}

/// Window materials whose colour follows the lights.
#[derive(Resource, Default)]
pub struct BuildingLights {
    /// `light` parts: window glass, tinted (0,0,0) off to (255,255,150) on
    /// (primitive colour for houses, environment colour for the shop).
    pub glass: Vec<Handle<N64Material>>,
    /// `window*` parts: the light the windows cast on the ground at night
    /// (drawn in the shadow list), alpha = texel x LOD fraction (0 off,
    /// 120 on).
    pub glow: Vec<Handle<N64Material>>,
    /// 0 = off, 1 = on, eased like `chase_f(..., 0.019532442)` per frame.
    level: f32,
}

/// `chase_f` step per 60 Hz frame for the light level.
const LIGHT_CHASE_PER_FRAME: f32 = 0.019_532_442;
/// Window glow LOD fraction with the lights fully on.
const GLOW_LOD_ON: f32 = 120.0 / 255.0;

fn load_light_table(mut commands: Commands, disc: Option<Res<GameDisc>>, mut done: Local<bool>) {
    if *done {
        return;
    }
    let Some(disc) = disc else { return };
    *done = true;
    match LightTable::from_disc(&disc) {
        Some(table) => {
            info!(
                "time-of-day lights from the disc; midnight ambient {:?}, fog {}..{}",
                table.0[0].ambient, table.0[0].fog_near, table.0[0].fog_far
            );
            commands.insert_resource(table);
        }
        None => warn!("time-of-day light table not found on the disc"),
    }
}

/// Updates every N64 material's light and fog, and the background colour,
/// once per game minute.
fn update_environment(
    table: Option<Res<LightTable>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut clear: ResMut<ClearColor>,
    mut last: Local<Option<(u32, usize)>>,
) {
    let Some(table) = table else { return };
    let now = clock::now();
    let minute = now.day_secs / 60;
    // Also refresh when new materials appear (town built after the table).
    let count = materials.len();
    if *last == Some((minute, count)) {
        return;
    }
    *last = Some((minute, count));
    let light = table.at(now.day_secs);
    let env = light.uniform();
    for (_, material) in materials.iter_mut() {
        material.env = env;
    }
    let [r, g, b] = light.background.map(|v| v / 255.0);
    clear.0 = Color::srgb(r, g, b);
}

/// Eases the building lights toward on/off and recolours their windows.
fn update_building_lights(
    time: Res<Time>,
    lights: Option<ResMut<BuildingLights>>,
    mut materials: ResMut<Assets<N64Material>>,
) {
    let Some(mut lights) = lights else { return };
    let target = if lights_on(clock::now().hour) {
        1.0
    } else {
        0.0
    };
    if lights.level == target && !lights.is_changed() {
        return;
    }
    let step = LIGHT_CHASE_PER_FRAME * time.delta_secs() * 60.0;
    let level = lights.level;
    lights.level = if target > level {
        (level + step).min(target)
    } else {
        (level - step).max(target)
    };
    let p = lights.level;
    let glass = Vec4::new(p, p, p * 150.0 / 255.0, 1.0);
    for handle in &lights.glass {
        if let Some(mut m) = materials.get_mut(handle) {
            // Houses tint the glass with the primitive colour, the shop and
            // other buildings with the environment colour (`aSHOP` draw):
            // each list reads only its own, so set both.
            m.combiner.prim = glass;
            m.combiner.env = glass.with_w(0.0);
        }
    }
    for handle in &lights.glow {
        if let Some(mut m) = materials.get_mut(handle) {
            m.combiner.extra.x = p * GLOW_LOD_ON;
        }
    }
}

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (load_light_table, update_environment, update_building_lights).chain(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn light(v: f32) -> BaseLight {
        BaseLight {
            ambient: [v; 3],
            ..default()
        }
    }

    #[test]
    fn blends_toward_the_next_period() {
        let table = LightTable(std::array::from_fn(|i| light(i as f32 * 10.0)));
        // 05:00 is halfway through 04:00-06:00 (period 1 -> 2).
        assert_eq!(table.at(5 * 3600).ambient, [15.0; 3]);
        // 22:00 is halfway from period 7 back to period 0.
        assert_eq!(table.at(22 * 3600).ambient, [35.0; 3]);
        assert_eq!(table.at(0).ambient, [0.0; 3]);
    }

    #[test]
    fn parses_base_light() {
        let mut b = [0u8; BASE_LIGHT_SIZE];
        b[..3].copy_from_slice(&[20, 10, 120]);
        b[3] = 73;
        b[18..20].copy_from_slice(&1799i16.to_be_bytes());
        b[20..22].copy_from_slice(&1000i16.to_be_bytes());
        b[34..37].copy_from_slice(&[28, 32, 92]);
        let l = BaseLight::parse(&b).unwrap();
        assert_eq!(l.ambient, [20.0, 10.0, 120.0]);
        assert_eq!(l.sun_dir[0], 73.0);
        assert_eq!(l.fog_near, (1799 & 0x3FF) as f32);
        assert_eq!(l.fog_far, 1000.0);
        assert_eq!(l.background, [28.0, 32.0, 92.0]);
    }

    #[test]
    fn lights_follow_villager_hours() {
        assert!(lights_on(18) && lights_on(23) && lights_on(4));
        assert!(!lights_on(5) && !lights_on(12) && !lights_on(17));
    }
}
