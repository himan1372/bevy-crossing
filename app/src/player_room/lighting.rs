//! The player-room light switch (Z) and the room's switch-driven lighting.
//!
//! Source: `mEnv_ManagePointLight`, `mEnv_GetRoomPrimColor`,
//! `mEnv_ChangeDiffuseLight`, `mRmTp_*LightSwitch*`, `aMR_FurnitureLightSet`
//! and `Actor_draw`. The switch is session state like the source's RAM bits:
//! it is not saved, and loading resets it to the game-start default.
//!
//! Each frame the room's point light (`SCENE_MY_ROOM_S`) is turned into a
//! directional light per object, as `LightsN_list_check` does: from each
//! furniture/loose item's shape centre, the player's position, and the
//! `ac_my_indoor` control actor at the origin for the floor and walls. The
//! floor and wall lists keep the inherited primitive colour, which
//! `Global_kankyo_set_room_prim` sets to the switch-dependent room colour.
//! The window light alpha (`mEnv_MakeWindowLightAlpha`) follows the level
//! and the clock; the sunshine patches (`sunshine.rs`) draw with it.

use bevy::prelude::*;
use rustimal_logic::kankyo_env;
use rustimal_logic::lights::LightsN;
use rustimal_logic::room_light::{self, RoomLight, MY_ROOM_S_POINT_LIGHT};

use super::{loose, presentation, PlayerRoom, RoomPiece};
use crate::environment::LightTable;
use crate::house::{CurrentRoom, HouseState, INTERIOR_ORIGIN};
use crate::input::{Action, Actions, InputFocus};
use crate::interior::PlayerRoomShell;
use crate::n64_material::{EnvUniform, N64Material, FLAG_MENU_LIGHTING};
use crate::player::Player;

/// `graph_dt_60hz_ticks` caps catch-up at four frames.
const MAX_TICKS: u32 = 4;

pub(super) struct Lighting {
    light: RoomLight,
    /// `l_mEnv_electric_light.windowlight_alpha`.
    window_alpha: f32,
    /// The room is entered (its level set) once per visit.
    inside: bool,
    accum: f32,
    pending_z: bool,
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            light: RoomLight::new(false),
            window_alpha: 0.0,
            inside: false,
            accum: 0.0,
            pending_z: false,
        }
    }
}

impl Lighting {
    /// Game start / load: `mRmTp_SetDefaultLightSwitchData` for the owned house.
    pub(super) fn reset(&mut self) {
        let now = crate::clock::now();
        *self = Self {
            light: RoomLight::new(room_light::default_on(now.day_secs, true)),
            ..Self::default()
        };
    }

    /// `mKK_windowlight_alpha_get`.
    pub(super) fn window_alpha(&self) -> f32 {
        self.window_alpha
    }

    /// Whether the window alpha has reached its current target.
    pub(super) fn window_settled(&self) -> bool {
        let now = crate::clock::now().day_secs as i32;
        self.window_alpha == kankyo_env::window_light_target(now, Some(self.light.percent))
    }

    /// `mEnv_MakeWindowLightAlpha` in this light-switch room.
    fn step_window(&mut self, enabled: bool) {
        let now = crate::clock::now().day_secs as i32;
        let target = kankyo_env::window_light_target(now, Some(self.light.percent));
        kankyo_env::window_light_step(&mut self.window_alpha, target, enabled);
    }
}

/// Room-local Bevy position (cells) to the source's game units: the small
/// room's unit (0, 0) corner is 3 cells from the interior origin and its
/// floor stands at game height 40.
fn game_units(world: Vec3) -> [f32; 3] {
    let p = (world - INTERIOR_ORIGIN + Vec3::new(3.0, 1.0, 3.0)) * 40.0;
    p.to_array()
}

pub(super) fn advance(
    mut room: ResMut<PlayerRoom>,
    current: Res<CurrentRoom>,
    house: Res<State<HouseState>>,
    focus: Res<InputFocus>,
    mut actions: ResMut<Actions>,
    time: Res<Time>,
) {
    let lighting = &mut room.lighting;
    if !crate::house::is_player_room(&current, &house) {
        lighting.inside = false;
        return;
    }
    if !lighting.inside {
        lighting.light.enter();
        // Global_kankyo_ct: the level is set first, then the window alpha
        // restarts from 0.
        lighting.step_window(false);
        info!(
            "player room light {} on entry",
            if lighting.light.on { "on" } else { "off" }
        );
        lighting.inside = true;
        lighting.accum = 0.0;
        lighting.pending_z = false;
    }
    if focus.is_world() && actions.consume(Action::LightSwitch) {
        lighting.pending_z = true;
    }
    lighting.accum += time.delta_secs() * 60.0;
    let ticks = (lighting.accum as u32).min(MAX_TICKS);
    lighting.accum = (lighting.accum - ticks as f32).min(1.0);
    for _ in 0..ticks {
        let z = std::mem::take(&mut lighting.pending_z);
        if let Some(se) = lighting.light.tick(z) {
            crate::sound::se(se);
            info!(
                "player room light switched {}",
                if lighting.light.on { "on" } else { "off" }
            );
        }
        // Global_kankyo_set runs after mEnv_ManagePointLight.
        lighting.step_window(true);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render(
    room: Res<PlayerRoom>,
    effects: Res<crate::furniture::gong::Effects>,
    current: Res<CurrentRoom>,
    house: Res<State<HouseState>>,
    table: Option<Res<LightTable>>,
    shells: Query<Entity, With<PlayerRoomShell>>,
    pieces: Query<
        (Entity, &GlobalTransform),
        Or<(
            With<RoomPiece>,
            With<loose::Piece>,
            With<presentation::RetiringPiece>,
        )>,
    >,
    players: Query<(Entity, &GlobalTransform), With<Player>>,
    children: Query<&Children>,
    handles: Query<&MeshMaterial3d<N64Material>>,
    mut materials: ResMut<Assets<N64Material>>,
) {
    let Some(table) = table else { return };
    if !crate::house::is_player_room(&current, &house) {
        return;
    }
    let light = &room.lighting.light;
    let now = crate::clock::now();
    let period = table.at(now.day_secs);
    let mut base = period.switch_room_uniform(light.celestial_adjust());
    base.effect_color = Vec3::splat(effects.light()).extend(0.0);
    let point = light.point_light(MY_ROOM_S_POINT_LIGHT);
    let room_color = period.room.map(|c| c as u8);
    let prim = light.room_prim(room_color, MY_ROOM_S_POINT_LIGHT.color);
    let prim = Vec4::new(prim[0] as f32, prim[1] as f32, prim[2] as f32, 255.0) / 255.0;

    let env_at = |anchor: [f32; 3]| {
        let mut lights = LightsN::new([0; 3]);
        lights.add_point_relative(&point, anchor);
        let mut env = base;
        if let Some(l) = lights.lights.first() {
            env.point_dir = Vec3::from(l.dir.map(|v| v as f32))
                .normalize_or_zero()
                .extend(0.0);
            env.point_color =
                Vec4::new(l.color[0] as f32, l.color[1] as f32, l.color[2] as f32, 0.0) / 255.0;
        }
        env
    };

    // A handle may be reachable from more than one root; the first wins so
    // a shared material never alternates between two lights.
    let mut done = bevy::platform::collections::HashSet::new();
    let mut apply = |root: Entity, env: EnvUniform, prim: Option<Vec4>| {
        for entity in std::iter::once(root).chain(children.iter_descendants(root)) {
            let Ok(handle) = handles.get(entity) else {
                continue;
            };
            if !done.insert(handle.id()) {
                continue;
            }
            let Some(current) = materials.get(handle.id()) else {
                continue;
            };
            if current.combiner.flags & FLAG_MENU_LIGHTING != 0 {
                continue;
            }
            let prim_ok = prim.is_none_or(|p| current.combiner.prim == p);
            if current.env == env && prim_ok {
                continue;
            }
            if let Some(mut material) = materials.get_mut(handle.id()) {
                material.env = env;
                if let Some(p) = prim {
                    material.combiner.prim = p;
                }
            }
        }
    };

    // ac_my_indoor is a control actor at the origin.
    let shell_env = env_at([0.0; 3]);
    for shell in &shells {
        apply(shell, shell_env, Some(prim));
    }
    for (entity, transform) in &pieces {
        apply(entity, env_at(game_units(transform.translation())), None);
    }
    for (entity, transform) in &players {
        apply(entity, env_at(game_units(transform.translation())), None);
    }
}
