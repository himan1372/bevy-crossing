//! Window sunshine on the player room's floor (`ef_room_sunshine`).
//!
//! `SCENE_MY_ROOM_S` places two `ROOM_SUNSHINE` actors at its side windows.
//! Each draws the disc's `light_floor01_mode` + `light_floor{L,R}01_vtx`
//! (a window-pane patch on the floor, `light_window` I4 texture) and
//! `light_shine01_mode` + `light_shine{L,R}01_vtx` (the slanted shaft),
//! translucent, in game units. The kernels are `rustimal_logic::
//! room_sunshine`; the alpha multiplies by the window light alpha that
//! `lighting.rs` eases from the clock and the switch level.

use bevy::prelude::*;
use rustimal_logic::room_sunshine::{self, Side, MY_ROOM_S_ACTORS};

use super::PlayerRoom;
use crate::disc_gfx::{run_display_lists, run_display_lists_with_prim, TexKey};
use crate::environment::LightTable;
use crate::game_disc::GameDisc;
use crate::house::{CurrentRoom, HouseState, INTERIOR_ORIGIN};
use crate::n64_material::{part_mesh, texture_image, CombinerUniform, N64Material, FLAG_CULL_BACK};
use rustimal_logic::graphics::texture::GxTexFormat;

/// The room shell's draw scale (`aMI`): model units to game units.
const ROOM_MODEL_SCALE: f32 = 0.0625;
/// Floor height used without the floor list (model cell height 1).
const FLOOR_Y: f32 = 40.0;

#[derive(Component)]
pub(super) struct Sunshine {
    side: Side,
    materials: Vec<Handle<N64Material>>,
    /// Last logged (visible, primitive alpha), once the window alpha settled.
    logged: Option<(bool, u8)>,
}

/// Game units to the room-local Bevy position (inverse of
/// `lighting::game_units`).
fn world(p: [f32; 3]) -> Vec3 {
    Vec3::from(p) / 40.0 - Vec3::new(3.0, 1.0, 3.0) + INTERIOR_ORIGIN
}

/// Legacy load-block I4 (`gsDPLoadBlock`, not Dolphin-converted): row-major
/// nibbles, high first; GX intensity formats use the intensity as alpha.
fn decode_i4_rows(bytes: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .flat_map(|v| [v >> 4, v & 15])
        .flat_map(|n| [n * 17; 4])
        .collect()
}

pub(super) fn spawn(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(disc) = disc else { return };
    let list = |name: &str| disc.catalog.get(name).map(|a| a.offset);
    let (Some(floor_mode), Some(shine_mode), Some(window)) = (
        list("light_floor01_mode"),
        list("light_shine01_mode"),
        disc.catalog.get("light_window"),
    ) else {
        warn!("room sunshine lists not found on the disc");
        return;
    };
    // mCoBG_GetBgY_OnlyCenter_FromWpos: the room floor list's surface.
    let floor = list("rom_myhome1_floor_model").map(|l| run_display_lists(&disc.rom, &[l]));
    let bg_y = |x: f32, z: f32| {
        floor
            .as_ref()
            .and_then(|f| f.height_at(x / ROOM_MODEL_SCALE, z / ROOM_MODEL_SCALE))
            .map_or(FLOOR_Y, |y| y * ROOM_MODEL_SCALE)
    };
    let texture = disc
        .rom
        .slice(window.source, window.offset, window.size)
        .map(decode_i4_rows);
    for (arg, x, z) in MY_ROOM_S_ACTORS {
        let (side, pos) = room_sunshine::actor_ct(arg, x, z, bg_y);
        let (floor_vtx, shine_vtx) = match side {
            Side::Left => ("light_floorL01_vtx", "light_shineL01_vtx"),
            Side::Right => ("light_floorR01_vtx", "light_shineR01_vtx"),
        };
        let (Some(floor_vtx), Some(shine_vtx)) = (list(floor_vtx), list(shine_vtx)) else {
            continue;
        };
        let sink = run_display_lists_with_prim(
            &disc.rom,
            &[floor_mode, floor_vtx, shine_mode, shine_vtx],
            [255; 4],
            255,
        );
        let triangles = sink.triangle_count();
        let root = commands
            .spawn((
                Name::new(format!("room sunshine {side:?}")),
                Transform::from_translation(world([pos[0], pos[1] + 0.1, pos[2]]))
                    .with_scale(Vec3::splat(1.0 / 40.0)),
                Visibility::Hidden,
            ))
            .id();
        let mut handles = Vec::new();
        for mut part in sink.parts {
            // The interpreter records only Dolphin tiles' format and size;
            // this list's plain tiles load `light_window` as I 4-bit 16x16,
            // mirrored on both axes (`light_floor01_mode`).
            let image = part.key.tex.as_mut().and_then(|key| {
                *key = TexKey {
                    img_addr: window.offset,
                    format: GxTexFormat::I4,
                    width: 16,
                    height: 16,
                    wrap_s: 2,
                    wrap_t: 2,
                    tlut_addr: 0,
                };
                Some(images.add(texture_image(texture.clone()?, key)))
            });
            let mut combiner = CombinerUniform::from_key(&part.key, false, true);
            combiner.flags |= FLAG_CULL_BACK;
            let material = materials.add(N64Material {
                combiner,
                env: default(),
                texture: image,
                texture1: None,
                blend: true,
            });
            handles.push(material.clone());
            commands.spawn((
                Mesh3d(meshes.add(part_mesh(&part, 1.0))),
                MeshMaterial3d(material),
                ChildOf(root),
            ));
        }
        info!(
            "room sunshine {side:?} at {:?}: {} parts, {} triangles",
            pos,
            handles.len(),
            triangles
        );
        commands.entity(root).insert(Sunshine {
            side,
            materials: handles,
            logged: None,
        });
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn render(
    room: Res<PlayerRoom>,
    current: Res<CurrentRoom>,
    house: Res<State<HouseState>>,
    table: Option<Res<LightTable>>,
    mut suns: Query<(&mut Sunshine, &mut Transform, &mut Visibility)>,
    mut materials: ResMut<Assets<N64Material>>,
) {
    let inside = crate::house::is_player_room(&current, &house);
    let Some(table) = table.filter(|_| inside) else {
        for (mut sun, _, mut visibility) in &mut suns {
            visibility.set_if_neq(Visibility::Hidden);
            sun.logged = None;
        }
        return;
    };
    let lighting = &room.lighting;
    let window_alpha = lighting.window_alpha();
    let now = crate::clock::now().day_secs as i32;
    let settled = lighting.window_settled();
    let period = table.at(now as u32);
    let colour = if room_sunshine::sun_colour(now) {
        period.sun_window
    } else {
        period.moon_window
    };
    // No weather simulation yet: Common(weather) stays clear (as sound.rs).
    let alpha = room_sunshine::prim_alpha(now, false, window_alpha);
    let prim = Vec4::new(
        colour[0].floor(),
        colour[1].floor(),
        colour[2].floor(),
        alpha as f32,
    ) / 255.0;
    for (mut sun, mut transform, mut visibility) in &mut suns {
        let scale_x = room_sunshine::scale_x(sun.side, now);
        let drawn = room_sunshine::drawn(window_alpha, scale_x);
        if settled && sun.logged != Some((drawn, alpha)) {
            info!(
                "room sunshine {:?}: drawn {drawn}, alpha {alpha}, scale {scale_x:.3}, \
                 window {window_alpha:.3}, colour {:?}",
                sun.side,
                colour.map(|c| c as u8)
            );
            sun.logged = Some((drawn, alpha));
        }
        if !drawn {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        }
        visibility.set_if_neq(Visibility::Inherited);
        let scale = Vec3::new(scale_x, 1.0, 1.0) / 40.0;
        if transform.scale != scale {
            transform.scale = scale;
        }
        for handle in &sun.materials {
            if materials
                .get(handle.id())
                .is_some_and(|m| m.combiner.prim != prim)
            {
                if let Some(mut material) = materials.get_mut(handle.id()) {
                    material.combiner.prim = prim;
                }
            }
        }
    }
}
