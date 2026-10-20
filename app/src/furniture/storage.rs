//! Audited aMR_OpenCloseCommonMove door/drawer constructors. Assets stay on disc.
use bevy::camera::visibility::{DynamicSkinnedMeshBounds, NoFrustumCulling};
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::prelude::*;

use crate::disc_gfx::{decode_texture, run_display_lists, Segments};
use crate::game_disc::GameDisc;
use crate::n64_material::{texture_image, CombinerUniform, N64Material};
use crate::skeleton::{Animation, Skeleton};

/// End frames passed by the corresponding original src/furniture/ac_*.c move
/// callbacks. Non-common/special constructors are deliberately absent.
pub(super) fn open_end(profile: &str) -> Option<f32> {
    Some(match profile.strip_prefix("iam_")? {
        "sum_blue_cab01"
        | "kon_snowtansu"
        | "nog_tri_chest01"
        | "sum_asi_chest02"
        | "sum_bla_chest01"
        | "sum_blue_lowchest01"
        | "sum_classicchest01"
        | "sum_cont_chest03"
        | "sum_fruitchest01"
        | "sum_gre_chest03"
        | "sum_hal_chest03"
        | "sum_liccalowchest"
        | "sum_log_chest01"
        | "sum_ratan_chest01"
        | "sum_whi_chest01"
        | "sum_x_chest02"
        | "yaz_turkey_chest" => 10.0,
        "nog_tri_chest02" | "sum_asi_chest03" | "sum_bla_chest03" | "sum_cont_chest02"
        | "sum_fruitchest03" | "sum_gre_chest02" | "sum_liccachest" | "sum_log_chest02"
        | "sum_ratan_chest02" | "sum_whi_chest02" | "sum_x_chest01" | "yaz_turkey_closet" => 12.0,
        "kob_locker1" | "kon_snowbox" | "nog_tri_chest03" | "sum_blue_bureau01"
        | "sum_clchest03" | "sum_hal_chest02" => 16.0,
        _ => return None,
    })
}

#[derive(Component)]
pub(crate) struct StorageRig {
    pub(crate) end: f32,
    skeleton: Skeleton,
    animation: Animation,
    joints: Vec<Entity>,
    frame: f32,
}

impl StorageRig {
    pub(crate) fn pose(&mut self, frame: f32, transforms: &mut Query<&mut Transform>) {
        if self.frame == frame {
            return;
        }
        self.frame = frame;
        for (&entity, local) in self
            .joints
            .iter()
            .zip(self.animation.pose(frame).local_transforms(&self.skeleton))
        {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                *transform = local;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)] // Same runtime mesh/material stores as spawn_furniture.
pub(super) fn spawn(
    commands: &mut Commands,
    disc: &GameDisc,
    root: Entity,
    skeleton: Skeleton,
    animation: Animation,
    end: f32,
    palette: Option<u32>,
    segments: &Segments,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) {
    let locals = animation.pose(1.0).local_transforms(&skeleton);
    let mut joints = Vec::with_capacity(locals.len());
    for (i, local) in locals.into_iter().enumerate() {
        let parent = skeleton.parents[i].map_or(root, |p| joints[p]);
        joints.push(
            commands
                .spawn((local, Visibility::default(), ChildOf(parent)))
                .id(),
        );
    }
    let matrix_joints = skeleton.matrix_joints();
    let mut parts = Vec::new();
    for &joint in &matrix_joints {
        let Some(list) = skeleton.joints[joint].model else {
            continue;
        };
        let blend = skeleton.joints[joint].flags & 1 != 0;
        for part in &run_display_lists(&disc.rom, &[list]).parts {
            let texture = part.key.tex.as_ref().and_then(|k| {
                decode_texture(&disc.rom, k, palette, segments)
                    .map(|rgba| images.add(texture_image(rgba, k)))
            });
            let material = materials.add(N64Material {
                combiner: CombinerUniform::from_key(&part.key, false, blend),
                texture,
                texture1: None,
                env: default(),
                blend,
            });
            let mut mesh = crate::character::skinned_part_mesh(part, joint, &matrix_joints);
            // Some original models (blue bureau) have a large joint translation
            // cancelled by their vertices. Bind-space AABBs cull the visible
            // posed model; use Bevy's joint-aware bounds, including drawer travel.
            let bounds = mesh.generate_skinned_mesh_bounds();
            let mut entity = commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                ChildOf(root),
            ));
            if let Err(error) = bounds {
                warn!("storage mesh bounds unavailable ({error}); keeping part visible");
                entity.insert(NoFrustumCulling);
            } else {
                entity.insert(DynamicSkinnedMeshBounds);
            }
            parts.push(entity.id());
        }
    }
    // The existing spawn_furniture callers need no new asset-store parameter:
    // create the shared identity bind poses when these deferred entities exist.
    let skin_joints = joints.clone();
    commands.queue(move |world: &mut World| {
        let inverse = world
            .resource_mut::<Assets<SkinnedMeshInverseBindposes>>()
            .add(SkinnedMeshInverseBindposes::from(vec![
                Mat4::IDENTITY;
                skin_joints.len()
            ]));
        for entity in parts {
            world.entity_mut(entity).insert(SkinnedMesh {
                inverse_bindposes: inverse.clone(),
                joints: skin_joints.clone(),
            });
        }
    });
    commands.entity(root).insert(StorageRig {
        end,
        skeleton,
        animation,
        joints,
        frame: 1.0,
    });
}
