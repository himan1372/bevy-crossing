//! Skinned disc rigs: the shared joint/part builder, aMR_SystemAnimeCt's
//! half-speed repeat for audited fish/insect profiles and fNBC balloons.
use super::*;
use crate::disc_gfx::decode_texture;
use crate::house::{CurrentRoom, HouseState};
use crate::n64_material::{texture_image, CombinerUniform};
use bevy::camera::visibility::{DynamicSkinnedMeshBounds, NoFrustumCulling};
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use rustimal_logic::furniture::{self as ftr, special_furniture};
use rustimal_logic::furniture_rig as native;

/// Spawn one joint entity per skeleton joint (first-frame pose) and skinned
/// meshes for `(joint, display list, translucent)`; returns the joints.
#[allow(clippy::too_many_arguments)] // Same disc/asset stores as spawn_furniture.
pub(super) fn spawn_skinned(
    commands: &mut Commands,
    disc: &GameDisc,
    root: Entity,
    label: &str,
    skeleton: &Skeleton,
    animation: &Animation,
    lists: Vec<(usize, u32, bool)>,
    palette: Option<u32>,
    segments: &Segments,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> Vec<Entity> {
    let mut joints = Vec::with_capacity(skeleton.joints.len());
    for (i, local) in animation
        .pose(1.0)
        .local_transforms(skeleton)
        .into_iter()
        .enumerate()
    {
        let parent = skeleton.parents[i].map_or(root, |p| joints[p]);
        joints.push(
            commands
                .spawn((
                    Name::new(format!("{label} {i}")),
                    local,
                    Visibility::default(),
                    ChildOf(parent),
                ))
                .id(),
        );
    }
    let matrix_joints = skeleton.matrix_joints();
    let mut parts = Vec::new();
    for (joint, list, blend) in lists {
        for part in &run_display_lists(&disc.rom, &[list]).parts {
            let texture = part.key.tex.as_ref().and_then(|key| {
                decode_texture(&disc.rom, key, palette, segments)
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
    joints
}

/// How a rigged profile plays in the room.
#[derive(Clone, Copy)]
enum Play {
    /// aMR_SystemAnimeCt_UniqueCt / aMR move: repeat at speed 0.5.
    System,
    /// fNBC_ct/fNBC_mv: speed ramps 0 -> 0.5, a switch change kicks 1.25.
    Balloon(native::BalloonSpeed),
}

fn balloon(index: u32) -> bool {
    (u32::from(special_furniture::BALLOON_COMMON0)..=u32::from(special_furniture::BALLOON_COMMON7))
        .contains(&index)
}

/// fNBC_balloon_skel_table / anime_table: `(name - 0x3FC) & 7`.
pub(super) fn balloon_base(index: u32) -> Option<String> {
    balloon(index).then(|| {
        format!(
            "nog_balloon{}",
            ((index - u32::from(special_furniture::BALLOON_COMMON0)) & 7) + 1
        )
    })
}

/// Profiles whose original play is reproduced here: callback-free fish and
/// insect rigs (no aFTR_vtable_c) and the common balloons.
fn play(profile: &Profile) -> Option<Play> {
    if balloon(profile.index) {
        return Some(Play::Balloon(native::BalloonSpeed::default()));
    }
    let index = u16::try_from(profile.index).ok()?;
    (profile.rig.is_some()
        && (ftr::INSECT_FIRST..ftr::FISH_FIRST + 40).contains(&index)
        && !ftr::critter_has_callback(index))
    .then_some(Play::System)
}

pub(super) fn animated(profile: &Profile) -> bool {
    play(profile).is_some()
}

#[derive(Component)]
pub(crate) struct FurnitureRig {
    play: Play,
    skeleton: Skeleton,
    animation: Animation,
    joints: Vec<Entity>,
    frame: f32,
    ticks: f32,
    switch: Option<u8>,
    kick: bool,
}

#[allow(clippy::too_many_arguments)] // Same disc/asset stores as spawn_furniture.
pub(super) fn spawn(
    commands: &mut Commands,
    disc: &GameDisc,
    root: Entity,
    profile: &Profile,
    skeleton: Skeleton,
    animation: Animation,
    segments: &Segments,
    assets: (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) {
    let Some(play) = play(profile) else {
        return;
    };
    let lists = skeleton
        .joints
        .iter()
        .enumerate()
        .filter_map(|(i, j)| Some((i, j.model?, j.flags & 1 != 0)))
        .collect();
    let joints = spawn_skinned(
        commands,
        disc,
        root,
        "rig joint",
        &skeleton,
        &animation,
        lists,
        profile.palette,
        segments,
        assets,
    );
    commands.entity(root).insert(FurnitureRig {
        play,
        skeleton,
        animation,
        joints,
        frame: 1.0,
        ticks: 0.0,
        switch: None,
        kick: false,
    });
}

/// Advance placed rigs at the original 60Hz tick while the room is shown.
pub(crate) fn update(
    time: Res<Time>,
    house: Res<State<HouseState>>,
    current: Res<CurrentRoom>,
    player_room: Res<crate::player_room::PlayerRoom>,
    mut rigs: Query<(&mut FurnitureRig, Option<&crate::player_room::RoomPiece>)>,
    mut transforms: Query<&mut Transform>,
) {
    let inside = crate::house::is_player_room(&current, &house);
    for (mut rig, piece) in &mut rigs {
        let switch = piece.and_then(|piece| player_room.switch_bit(piece));
        rig.kick |= rig.switch.is_some() && switch != rig.switch;
        rig.switch = switch;
        if inside {
            rig.ticks += time.delta_secs() * 60.0;
        }
        let end = rig.animation.frames;
        while rig.ticks >= 1.0 {
            rig.ticks -= 1.0;
            let speed = match rig.play {
                Play::System => native::SYSTEM_RIG_SPEED,
                Play::Balloon(mut speed) => {
                    let speed_now = speed.step(std::mem::take(&mut rig.kick));
                    rig.play = Play::Balloon(speed);
                    speed_now
                }
            };
            rig.frame = native::repeat_frame(rig.frame, end, speed);
        }
        let pose = rig.animation.pose(rig.frame);
        for (&joint, local) in rig.joints.iter().zip(pose.local_transforms(&rig.skeleton)) {
            if let Ok(mut transform) = transforms.get_mut(joint) {
                *transform = local;
            }
        }
    }
}
