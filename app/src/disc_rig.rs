//! A plain cKF rig from the disc by name (skeleton `cKF_bs_r_*` and one
//! animation `cKF_ba_r_*`), skinned like the electric fan's, with its own
//! textures. Used for the museum's fish.
use std::sync::Arc;

use bevy::camera::visibility::{DynamicSkinnedMeshBounds, NoFrustumCulling};
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::prelude::*;

use crate::disc_gfx::{decode_texture, run_display_lists, Segments};
use crate::game_disc::GameDisc;
use crate::n64_material::{texture_image, CombinerUniform, N64Material};
use crate::skeleton::{Animation, Skeleton};

/// A spawned rig and the frame it shows.
#[derive(Component)]
pub(crate) struct DiscRig {
    skeleton: Arc<Skeleton>,
    animation: Arc<Animation>,
    joints: Vec<Entity>,
    pub(crate) frame: f32,
}

impl DiscRig {
    /// Advances the loop by `frames` (repeat between 1 and the last frame).
    pub(crate) fn step<F: bevy::ecs::query::QueryFilter>(
        &mut self,
        frames: f32,
        transforms: &mut Query<&mut Transform, F>,
    ) {
        let last = self.animation.frames.max(2) as f32;
        self.frame += frames;
        while self.frame >= last {
            self.frame -= last - 1.0;
        }
        let locals = self
            .animation
            .pose(self.frame)
            .local_transforms(&self.skeleton);
        for (&joint, local) in self.joints.iter().zip(locals) {
            if let Ok(mut t) = transforms.get_mut(joint) {
                *t = local;
            }
        }
    }
}

/// Builds the rig under `root`; None when either name is not on the disc.
pub(crate) fn spawn(
    commands: &mut Commands,
    disc: &GameDisc,
    root: Entity,
    (skeleton, animation): (&str, &str),
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> Option<DiscRig> {
    let skeleton = Skeleton::read(&disc.rom, disc.catalog.get(skeleton)?.offset)?;
    let animation = Animation::read(
        &disc.rom,
        disc.catalog.get(animation)?.offset,
        skeleton.joints.len(),
    )?;
    let mut joints = Vec::with_capacity(skeleton.joints.len());
    for (i, local) in animation
        .pose(1.0)
        .local_transforms(&skeleton)
        .into_iter()
        .enumerate()
    {
        let parent = skeleton.parents[i].map_or(root, |p| joints[p]);
        joints.push(
            commands
                .spawn((local, Visibility::default(), ChildOf(parent)))
                .id(),
        );
    }
    let matrix_joints = skeleton.matrix_joints();
    let segments = Segments::default();
    let mut skin_parts = Vec::new();
    for (joint, data) in skeleton.joints.iter().enumerate() {
        let Some(list) = data.model else { continue };
        let blend = data.flags & 1 != 0;
        for part in run_display_lists(&disc.rom, &[list]).parts {
            let texture = part.key.tex.as_ref().and_then(|key| {
                decode_texture(&disc.rom, key, None, &segments)
                    .map(|rgba| images.add(texture_image(rgba, key)))
            });
            let material = materials.add(N64Material {
                combiner: CombinerUniform::from_key(&part.key, false, blend),
                texture,
                texture1: None,
                env: default(),
                blend,
            });
            let mut mesh = crate::character::skinned_part_mesh(&part, joint, &matrix_joints);
            let bounds = mesh.generate_skinned_mesh_bounds();
            let mut entity = commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                ChildOf(root),
            ));
            if bounds.is_ok() {
                entity.insert(DynamicSkinnedMeshBounds);
            } else {
                entity.insert(NoFrustumCulling);
            }
            skin_parts.push(entity.id());
        }
    }
    let skin_joints = joints.clone();
    commands.queue(move |world: &mut World| {
        let inverse = world
            .resource_mut::<Assets<SkinnedMeshInverseBindposes>>()
            .add(SkinnedMeshInverseBindposes::from(vec![
                Mat4::IDENTITY;
                skin_joints.len()
            ]));
        for entity in skin_parts {
            if let Ok(mut entity) = world.get_entity_mut(entity) {
                entity.insert(SkinnedMesh {
                    inverse_bindposes: inverse.clone(),
                    joints: skin_joints.clone(),
                });
            }
        }
    });
    Some(DiscRig {
        skeleton: Arc::new(skeleton),
        animation: Arc::new(animation),
        joints,
        frame: 1.0,
    })
}
