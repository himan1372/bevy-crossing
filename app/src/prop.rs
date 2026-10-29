//! Animated disc props: a `cKF_Skeleton_R_c` drawn as one entity per drawing joint and posed from a
//! `cKF_Animation_R_c`, for things that are not characters (the train door, Rover's phone).
use std::sync::Arc;

use bevy::prelude::*;

use crate::game_disc::GameDisc;
use crate::n64_material::N64Material;
use crate::skeleton::{Animation, Skeleton};

/// A spawned prop: its skeleton and the entity holding each drawing joint's parts.
#[derive(Component, Clone)]
pub(crate) struct Prop {
    skeleton: Arc<Skeleton>,
    joints: Vec<Option<Entity>>,
}

impl Prop {
    /// The skeleton `skeleton_name` (`cKF_bs_r_*`) under `parent` at `local`, in the disc's model units
    /// (the caller's transform scales them). Returns the root entity and a copy of the [`Prop`] on it.
    pub(crate) fn spawn(
        commands: &mut Commands,
        disc: &GameDisc,
        skeleton_name: &str,
        parent: Entity,
        local: Transform,
        palette: Option<u32>,
        (meshes, materials, images): (
            &mut Assets<Mesh>,
            &mut Assets<N64Material>,
            &mut Assets<Image>,
        ),
    ) -> Option<(Entity, Prop)> {
        let skeleton = disc
            .catalog
            .get(skeleton_name)
            .and_then(|a| Skeleton::read(&disc.rom, a.offset))?;
        let root = commands
            .spawn((
                Name::new(skeleton_name.to_string()),
                local,
                Visibility::default(),
                ChildOf(parent),
            ))
            .id();
        let mut joints = vec![None; skeleton.joints.len()];
        for (j, joint) in skeleton.joints.iter().enumerate() {
            let Some(list) = joint.model else { continue };
            let sink = crate::disc_gfx::run_display_lists(&disc.rom, &[list]);
            let holder = commands
                .spawn((Transform::default(), Visibility::default(), ChildOf(root)))
                .id();
            for (mesh, material) in crate::disc_town::sink_meshes_scaled(
                disc,
                &sink,
                palette,
                joint.flags & 1 != 0,
                &crate::disc_gfx::Segments::default(),
                1.0,
                (&mut *meshes, &mut *materials, &mut *images),
            ) {
                commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), ChildOf(holder)));
            }
            joints[j] = Some(holder);
        }
        let prop = Prop {
            skeleton: Arc::new(skeleton),
            joints,
        };
        commands.entity(root).insert(prop.clone());
        Some((root, prop))
    }

    /// The animation `name` (`cKF_ba_r_*`) for this skeleton.
    pub(crate) fn animation(&self, disc: &GameDisc, name: &str) -> Option<Arc<Animation>> {
        disc.catalog
            .get(name)
            .and_then(|a| Animation::read(&disc.rom, a.offset, self.skeleton.joints.len()))
            .map(Arc::new)
    }

    /// Poses every joint at `frame` of `anim`.
    pub(crate) fn pose<F: bevy::ecs::query::QueryFilter>(
        &self,
        anim: Option<&Animation>,
        frame: f32,
        transforms: &mut Query<&mut Transform, F>,
    ) {
        let world = self.skeleton.world_matrices(anim, frame);
        for (holder, matrix) in self.joints.iter().zip(world) {
            let Some(holder) = holder else { continue };
            if let Ok(mut transform) = transforms.get_mut(*holder) {
                *transform = Transform::from_matrix(matrix);
            }
        }
    }
}
