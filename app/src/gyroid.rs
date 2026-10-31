//! The houses' gyroids (`ac_haniwa.c`, `ac_haniwa_move.c_inc`).
//!
//! `cKF_bs_r_hnw`, drawn with `hnw_tmem_txt` in segment B
//! (`ANIME_4_TXT_SEG`) and the `hnw_face` palette in TLUT 15, prim white
//! at LOD 128, on the layout's `HANIWA0..3` units. It loops frames 1-9 of
//! `cKF_ba_r_hnw_move` (`aHNW_setupAction`): 0.3 frames a tick by the
//! player's own house (speed eased by `chase_f` 0.05 up / 0.015 down),
//! 0.1 by another player's, and stopped where nobody lives. An owned
//! gyroid turns to the player (`aHNW_search_player`, 0x600 a tick), an
//! empty house's to +-8000 (`aHNW_search_front`).

use bevy::prelude::*;

use crate::disc_gfx::Segments;
use crate::game_disc::GameDisc;
use crate::n64_material::N64Material;
use crate::player::Player;
use crate::skeleton::{place_on_joints, Animation, Skeleton};
use crate::town::Town;

type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;

/// `cKF_ba_r_hnw_move` loops frames 1..=9.
const FIRST_FRAME: f32 = 1.0;
const LAST_FRAME: f32 = 9.0;
/// Baked poses per animation frame.
const STEPS_PER_FRAME: f32 = 2.0;
/// `anim_frame_speed` by the player's house, while waiting.
const OWNER_SPEED: f32 = 0.3;
/// `chase_angle` step per tick, s16 units.
const TURN_STEP: f32 = 0x0600 as f32;
/// `aHNW_search_front` targets for empty houses 0-3.
const FRONT_ANGLE: [i16; 4] = [8000, -8000, 8000, -8000];
/// s16 angle units per radian.
const S16_PER_RAD: f32 = 32768.0 / std::f32::consts::PI;

/// A gyroid's animation state.
#[derive(Component)]
pub struct Gyroid {
    house: usize,
    /// The player owns this house (house 0 is the player's).
    owned: bool,
    frame: f32,
    speed: f32,
    /// Facing, s16 units.
    angle: f32,
    parts: Vec<Entity>,
}

/// The baked poses, frame 1 first.
#[derive(Resource)]
struct GyroidPoses(Vec<Model>);

/// Builds the poses and spawns a gyroid on each `HANIWA` unit.
pub fn spawn_gyroids(
    commands: &mut Commands,
    disc: &GameDisc,
    town: &Town,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) {
    let Some(retail) = &town.retail else { return };
    let cat = &disc.catalog;
    let Some(skel) = cat
        .get("cKF_bs_r_hnw")
        .and_then(|a| Skeleton::read(&disc.rom, a.offset))
    else {
        warn!("gyroid model not found on the disc");
        return;
    };
    let anim = cat
        .get("cKF_ba_r_hnw_move")
        .and_then(|a| Animation::read(&disc.rom, a.offset, skel.joints.len()));
    let (Some(tex), Some(face)) = (cat.get("hnw_tmem_txt"), cat.get("hnw_face")) else {
        return;
    };
    let Some(bytes) = disc.rom.slice(disc::RomSource::Rel, tex.offset, tex.size) else {
        return;
    };
    let mut segs = Segments::default();
    segs.set_segment(0x0B, bytes.into());
    // Its limbs share one vertex buffer through segment-D matrices.
    let steps = ((LAST_FRAME - FIRST_FRAME) * STEPS_PER_FRAME) as usize;
    let poses: Vec<Model> = (0..steps)
        .map(|i| {
            let frame = FIRST_FRAME + i as f32 / STEPS_PER_FRAME;
            let world = skel.world_matrices(anim.as_ref(), frame);
            let mut sink = crate::disc_gfx::run_display_lists_with_colors(
                &disc.rom,
                &skel.lists(),
                [255; 4],
                128,
                [0; 4],
                &[(15, face.offset)],
            );
            place_on_joints(&mut sink, &skel, &world);
            crate::disc_town::sink_meshes(
                disc,
                &sink,
                None,
                false,
                &segs,
                (&mut *meshes, &mut *materials, &mut *images),
            )
        })
        .collect();
    for (house, cell) in retail.gyroids.iter().enumerate() {
        let Some(&cell) = cell.as_ref() else { continue };
        let at = crate::disc_town::mailbox_position(cell);
        let angle = FRONT_ANGLE[house % 4] as f32;
        let root = commands
            .spawn((
                crate::disc_town::DiscObject,
                Name::new(format!("gyroid {house}")),
                Transform::from_xyz(at.x, town.ground_y(at), at.z)
                    .with_rotation(Quat::from_rotation_y(angle / S16_PER_RAD)),
                Visibility::default(),
            ))
            .id();
        let parts = poses[0]
            .iter()
            .map(|(mesh, material)| {
                commands
                    .spawn((
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(material.clone()),
                        ChildOf(root),
                    ))
                    .id()
            })
            .collect();
        commands.entity(root).insert(Gyroid {
            house,
            owned: house == crate::home::index(),
            frame: FIRST_FRAME,
            speed: 0.0,
            angle,
            parts,
        });
    }
    commands.insert_resource(GyroidPoses(poses));
}

/// `chase_angle`: step `angle` toward `target` by at most `step` (s16).
fn chase_angle(angle: f32, target: f32, step: f32) -> f32 {
    let diff = ((target - angle + 32768.0).rem_euclid(65536.0)) - 32768.0;
    angle + diff.clamp(-step, step)
}

fn animate_gyroids(
    time: Res<Time>,
    poses: Res<GyroidPoses>,
    player: Query<&Transform, (With<Player>, Without<Gyroid>)>,
    mut gyroids: Query<(&mut Gyroid, &mut Transform)>,
    mut parts: Query<(&mut Mesh3d, &mut MeshMaterial3d<N64Material>)>,
) {
    let ticks = time.delta_secs() * 60.0;
    let player = player.single().ok().map(|t| t.translation);
    for (mut g, mut t) in &mut gyroids {
        g.owned = g.house == crate::home::index();
        // aHNW_common_process: ease the speed, turn to the player or front.
        let target_speed = if g.owned { OWNER_SPEED } else { 0.0 };
        let rate = if target_speed > g.speed { 0.05 } else { 0.015 };
        g.speed += (target_speed - g.speed).clamp(-rate * ticks, rate * ticks);
        let target = match (g.owned, player) {
            (true, Some(p)) => {
                let d = p - t.translation;
                d.x.atan2(d.z) * S16_PER_RAD
            }
            _ => FRONT_ANGLE[g.house % 4] as f32,
        };
        g.angle = chase_angle(g.angle, target, TURN_STEP * ticks);
        t.rotation = Quat::from_rotation_y(g.angle / S16_PER_RAD);
        if g.speed <= 0.0 {
            continue;
        }
        // cKF_FRAMECONTROL_REPEAT over frames 1..9.
        let span = LAST_FRAME - FIRST_FRAME;
        g.frame = FIRST_FRAME + (g.frame - FIRST_FRAME + g.speed * ticks).rem_euclid(span);
        let i = (((g.frame - FIRST_FRAME) * STEPS_PER_FRAME) as usize).min(poses.0.len() - 1);
        for (e, (mesh, material)) in g.parts.iter().zip(&poses.0[i]) {
            if let Ok((mut m, mut mat)) = parts.get_mut(*e) {
                if m.0 != *mesh {
                    m.0 = mesh.clone();
                    mat.0 = material.clone();
                }
            }
        }
    }
}

pub struct GyroidPlugin;

impl Plugin for GyroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            animate_gyroids.run_if(
                in_state(crate::states::AppState::InGame).and_then(resource_exists::<GyroidPoses>),
            ),
        );
    }
}
