//! Audited ac_*clock callbacks, with original skinned joint/display-list data.
use super::*;
use crate::disc_gfx::decode_texture;
use crate::house::{CurrentRoom, HouseState};
use crate::n64_material::{texture_image, CombinerUniform};
use bevy::camera::visibility::{DynamicSkinnedMeshBounds, NoFrustumCulling};
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use rustimal_logic::furniture_clock as native;

#[derive(Clone, Copy)]
pub(super) struct Spec {
    hour: usize,
    minute: Option<usize>,
    extra: bool,
    tick: bool,
}

pub(super) fn spec(profile: &str) -> Option<Spec> {
    let (hour, minute, extra, tick) = match profile.strip_prefix("iam_")? {
        "kon_blueclock" | "kon_grclock" | "kon_ameclock" => (3, Some(5), false, false),
        "kon_redclock" | "nog_tri_clock01" => (3, Some(4), false, false),
        "kon_mimiclock" => (3, Some(6), false, false),
        "kon_waclock" => (2, None, false, false),
        "kon_atqclock" => (5, Some(7), false, true),
        "kon_snowclock" => (6, Some(7), true, false),
        "sum_mezaclock" => (3, Some(4), true, false),
        _ => return None,
    };
    Some(Spec {
        hour,
        minute,
        extra,
        tick,
    })
}

fn extra_lists(disc: &GameDisc, profile: &Profile, spec: Spec) -> Option<Vec<(usize, u32)>> {
    if !spec.extra {
        return Some(Vec::new());
    }
    let base = profile.name.strip_prefix("iam_")?;
    Some(vec![
        (
            spec.hour,
            disc.catalog.get(&format!("int_{base}_short_model"))?.offset,
        ),
        (
            spec.minute?,
            disc.catalog.get(&format!("int_{base}_long_model"))?.offset,
        ),
    ])
}

pub(super) fn available(disc: &GameDisc, profile: &Profile) -> bool {
    let Some(spec) = spec(&profile.name) else {
        return false;
    };
    let Some((skeleton, Some(_))) = profile_skeleton(disc, profile) else {
        return false;
    };
    skeleton.joints.len() > spec.hour.max(spec.minute.unwrap_or(0))
        && extra_lists(disc, profile, spec).is_some()
}

#[derive(Component)]
pub(crate) struct ClockRig {
    spec: Spec,
    skeleton: Skeleton,
    animation: Animation,
    joints: Vec<Entity>,
    frame: f32,
    owned: bool,
}

#[allow(clippy::too_many_arguments)] // Same disc/asset stores as spawn_furniture.
pub(super) fn spawn(
    commands: &mut Commands,
    disc: &GameDisc,
    root: Entity,
    profile: &Profile,
    spec: Spec,
    skeleton: Skeleton,
    animation: Animation,
    segments: &Segments,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) {
    let Some(extras) = extra_lists(disc, profile, spec) else {
        return;
    };
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
                    Name::new(format!("clock joint {i}")),
                    local,
                    Visibility::default(),
                    ChildOf(parent),
                ))
                .id(),
        );
    }
    let matrix_joints = skeleton.matrix_joints();
    let lists = skeleton
        .joints
        .iter()
        .enumerate()
        .filter_map(|(i, j)| Some((i, j.model?, j.flags & 1 != 0)))
        .chain(extras.into_iter().map(|(i, list)| (i, list, false)));
    let mut parts = Vec::new();
    for (joint, list, blend) in lists {
        for part in &run_display_lists(&disc.rom, &[list]).parts {
            let texture = part.key.tex.as_ref().and_then(|key| {
                decode_texture(&disc.rom, key, profile.palette, segments)
                    .map(|rgba| images.add(texture_image(rgba, key)))
            });
            let material = materials.add(N64Material {
                combiner: CombinerUniform::from_key(&part.key, false, blend),
                texture,
                env: default(),
                blend,
            });
            let mut mesh = crate::character::skinned_part_mesh(part, joint, &matrix_joints);
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
            parts.push(entity.id());
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
        for entity in parts {
            world.entity_mut(entity).insert(SkinnedMesh {
                inverse_bindposes: inverse.clone(),
                joints: skin_joints.clone(),
            });
        }
    });
    commands.entity(root).insert(ClockRig {
        spec,
        skeleton,
        animation,
        joints,
        frame: 1.0,
        owned: false,
    });
}

#[derive(Default)]
pub(crate) struct RoomBeat {
    room: Option<Vec3>,
    phase: f32,
}

pub(crate) fn update(
    time: Res<Time>,
    house: Res<State<HouseState>>,
    current: Res<CurrentRoom>,
    player_room: Res<crate::player_room::PlayerRoom>,
    mut loads: MessageReader<crate::sound::ResetAudio>,
    mut beat: Local<RoomBeat>,
    mut clocks: Query<(
        &mut ClockRig,
        &GlobalTransform,
        Option<&crate::player_room::RoomPiece>,
    )>,
    mut transforms: Query<&mut Transform>,
) {
    let room = (*house.get() == HouseState::Inside).then_some(current.origin());
    let changed = loads.read().count() != 0 || beat.room != room;
    let pulse = if changed {
        beat.room = room;
        beat.phase = (time.elapsed_secs() * 60.0).rem_euclid(120.0);
        native::ClockPulse::default()
    } else if room.is_some() {
        native::advance_beat(&mut beat.phase, time.delta_secs())
    } else {
        native::ClockPulse::default()
    };
    let now = crate::clock::now();
    let (hour, minute) = native::hand_angles(now.hour, now.minute, (now.day_secs % 60) as u8);
    let radians = std::f32::consts::PI / 32768.0;
    for (mut rig, at, piece) in &mut clocks {
        rig.owned |= piece.is_some();
        let active = room.is_some_and(|room| at.translation().distance(room) < 8.0);
        if changed || (rig.spec.tick && active && pulse.reset) {
            rig.frame = 1.0;
        } else {
            rig.frame = native::advance_frame(rig.frame, rig.animation.frames, time.delta_secs());
        }
        let mut pose = rig.animation.pose(rig.frame);
        if let Some(rotation) = pose.rotations.get_mut(rig.spec.hour) {
            rotation.z -= f32::from(hour) * radians;
        }
        if let Some(rotation) = rig.spec.minute.and_then(|i| pose.rotations.get_mut(i)) {
            rotation.z -= f32::from(minute) * radians;
        }
        for (&joint, local) in rig.joints.iter().zip(pose.local_transforms(&rig.skeleton)) {
            if let Ok(mut transform) = transforms.get_mut(joint) {
                *transform = local;
            }
        }
        let sound_allowed =
            !rig.owned || piece.is_some_and(|piece| player_room.clock_sound_allowed(piece));
        if rig.spec.tick && active && pulse.tick && sound_allowed {
            crate::sound::se_at(0x12D, at.translation());
        }
    }
}
