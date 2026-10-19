//! ac_nog_fan01: motor-driven cKF rig and three cached blade textures.
use super::*;
use crate::disc_gfx::{decode_texture, Part};
use crate::house::{CurrentRoom, HouseState};
use crate::n64_material::{texture_image, CombinerUniform};
use crate::player_room::{PlayerRoom, RoomPiece};
use bevy::camera::visibility::{DynamicSkinnedMeshBounds, NoFrustumCulling};
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use rustimal_logic::furniture_fan::Fan;

pub(crate) fn is_fan(index: u16) -> bool {
    index == 0x22
}

struct Loaded {
    skeleton: Skeleton,
    animation: Animation,
    parts: Vec<FanPart>,
}

struct FanPart {
    joint: usize,
    part: Part,
    blend: bool,
    frames: Vec<Vec<u8>>,
}

/// Preflight and spawn share the same fully decoded assets. The source's
/// skeleton name omits the profile's trailing01, and segment8 is three frames.
fn load(disc: &GameDisc, profile: &Profile) -> Option<Loaded> {
    let skeleton = Skeleton::read(&disc.rom, disc.catalog.get("cKF_bs_r_int_nog_fan")?.offset)?;
    let animation = Animation::read(
        &disc.rom,
        disc.catalog.get("cKF_ba_r_int_nog_fan")?.offset,
        skeleton.joints.len(),
    )?;
    let segments: Option<Vec<_>> = (1..=3)
        .map(|frame| {
            let asset = disc
                .catalog
                .get(&format!("int_nog_fan_fan{frame}_TA_tex_txt"))?;
            let mut segments = Segments::default();
            segments.set_segment(
                8,
                Arc::from(disc.rom.slice(RomSource::Rel, asset.offset, asset.size)?),
            );
            Some(segments)
        })
        .collect();
    let segments = segments?;
    let mut parts = Vec::new();
    for (joint, data) in skeleton.joints.iter().enumerate() {
        let Some(list) = data.model else { continue };
        for part in run_display_lists(&disc.rom, &[list]).parts {
            let frames = if let Some(key) = part.key.tex.as_ref() {
                let count = if key.img_addr >> 24 == 8 { 3 } else { 1 };
                segments
                    .iter()
                    .take(count)
                    .map(|s| decode_texture(&disc.rom, key, profile.palette, s))
                    .collect::<Option<Vec<_>>>()?
            } else {
                Vec::new()
            };
            parts.push(FanPart {
                joint,
                part,
                blend: data.flags & 1 != 0,
                frames,
            });
        }
    }
    (!parts.is_empty() && parts.iter().any(|p| p.frames.len() == 3)).then_some(Loaded {
        skeleton,
        animation,
        parts,
    })
}

pub(super) fn available(disc: &GameDisc, profile: &Profile) -> bool {
    load(disc, profile).is_some()
}

#[derive(Component)]
pub(crate) struct FanRig {
    skeleton: Skeleton,
    animation: Animation,
    joints: Vec<Entity>,
    motor: Fan,
    switch: Option<bool>,
    ticks: f32,
    texture: usize,
    textures: Vec<(Handle<N64Material>, Vec<Handle<Image>>)>,
    pub(crate) sound_on: bool,
}

pub(super) fn spawn(
    commands: &mut Commands,
    disc: &GameDisc,
    root: Entity,
    profile: &Profile,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) {
    let Some(loaded) = load(disc, profile) else {
        warn!("incomplete electric fan assets");
        return;
    };
    let Loaded {
        skeleton,
        animation,
        parts,
    } = loaded;
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
                .spawn((
                    Name::new(format!("fan joint {i}")),
                    local,
                    Visibility::default(),
                    ChildOf(parent),
                ))
                .id(),
        );
    }
    let matrix_joints = skeleton.matrix_joints();
    let mut textures = Vec::new();
    let mut skin_parts = Vec::new();
    for FanPart {
        joint,
        part,
        blend,
        frames,
    } in parts
    {
        let frames: Vec<_> = part.key.tex.as_ref().map_or_else(Vec::new, |key| {
            frames
                .into_iter()
                .map(|rgba| images.add(texture_image(rgba, key)))
                .collect()
        });
        let material = materials.add(N64Material {
            combiner: CombinerUniform::from_key(&part.key, false, blend),
            texture: frames.first().cloned(),
            env: default(),
            blend,
        });
        if frames.len() == 3 {
            textures.push((material.clone(), frames));
        }
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
    commands.entity(root).insert(FanRig {
        skeleton,
        animation,
        joints,
        motor: Fan::new(false),
        switch: None,
        ticks: 0.0,
        texture: 0,
        textures,
        sound_on: false,
    });
}

pub(crate) fn update(
    time: Res<Time>,
    house: Res<State<HouseState>>,
    current: Res<CurrentRoom>,
    room: Res<PlayerRoom>,
    mut fans: Query<(&mut FanRig, Option<&RoomPiece>)>,
    mut transforms: Query<&mut Transform>,
    mut materials: ResMut<Assets<N64Material>>,
) {
    let inside = crate::house::is_player_room(&current, &house);
    for (mut rig, piece) in &mut fans {
        let on = piece
            .and_then(|p| room.switch_bit(p))
            .is_some_and(|s| s != 0);
        if rig.switch.is_none() {
            rig.motor = Fan::new(on);
        }
        rig.switch = Some(on);
        rig.sound_on = inside && on && piece.is_some_and(|p| room.clock_sound_allowed(p));
        if inside {
            // Match the other 60Hz adapters: bound catch-up after a stall.
            rig.ticks = (rig.ticks + time.delta_secs() * 60.0).min(4.0);
        }
        while rig.ticks >= 1.0 {
            rig.ticks -= 1.0;
            let end = rig.animation.frames;
            rig.motor.step(on, end);
        }
        for (&joint, local) in rig.joints.iter().zip(
            rig.animation
                .pose(rig.motor.frame)
                .local_transforms(&rig.skeleton),
        ) {
            if let Ok(mut transform) = transforms.get_mut(joint) {
                *transform = local;
            }
        }
        let texture = rig.motor.texture();
        if rig.texture != texture {
            rig.texture = texture;
            for (handle, frames) in &rig.textures {
                if let Some(mut material) = materials.get_mut(handle) {
                    material.texture = frames.get(texture).cloned();
                }
            }
        }
    }
}
