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
    assets: (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> Vec<Entity> {
    spawn_skinned_colored(
        commands, disc, root, label, skeleton, animation, lists, palette, segments, None, assets,
    )
    .0
}

/// As [`spawn_skinned`], starting each list from `colors` (primitive and
/// environment) the way a draw callback's segment list would set them; also
/// returns the materials that kept those starting colours.
#[allow(clippy::too_many_arguments)] // Same disc/asset stores as spawn_furniture.
fn spawn_skinned_colored(
    commands: &mut Commands,
    disc: &GameDisc,
    root: Entity,
    label: &str,
    skeleton: &Skeleton,
    animation: &Animation,
    lists: Vec<(usize, u32, bool)>,
    palette: Option<u32>,
    segments: &Segments,
    colors: Option<([u8; 4], [u8; 4])>,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> (Vec<Entity>, Vec<Handle<N64Material>>) {
    let mut seeded = Vec::new();
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
        let sink = match colors {
            Some((prim, env)) => crate::disc_gfx::run_display_lists_with_colors(
                &disc.rom,
                &[list],
                prim,
                255,
                env,
                &[],
            ),
            None => run_display_lists(&disc.rom, &[list]),
        };
        for part in &sink.parts {
            let texture = part.key.tex.as_ref().and_then(|key| {
                decode_texture(&disc.rom, key, palette, segments)
                    .map(|rgba| images.add(texture_image(rgba, key)))
            });
            let mut material = N64Material {
                combiner: CombinerUniform::from_key(&part.key, false, blend),
                texture,
                texture1: None,
                env: default(),
                blend,
            };
            crate::n64_material::attach_texture1(
                &mut material,
                &disc.rom,
                &part.key,
                palette,
                segments,
                images,
            );
            let material = materials.add(material);
            if colors.is_some_and(|(prim, env)| part.key.prim == prim && part.key.env == env) {
                seeded.push(material.clone());
            }
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
    (joints, seeded)
}

/// aSumgenji_dw: primitive (255, 180, 0, alpha) and environment (0, 255, 0).
const FIREFLY_PRIM: [u8; 4] = [255, 180, 0, 255];
const FIREFLY_ENV: [u8; 4] = [0, 255, 0, 255];
/// aSumgenji_dw prim_a_table, indexed by the move counter.
const FIREFLY_ALPHA: [u8; 68] = [
    255, 255, 255, 255, 240, 225, 210, 195, 180, 165, 150, 135, 120, 105, 90, 75, 50, 35, 20, 5, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 20, 35, 50,
    75, 90, 105, 120, 135, 150, 165, 180, 195, 210, 225, 240, 255, 255, 255, 255,
];

/// How a rigged profile plays in the room.
#[derive(Clone, Copy)]
enum Play {
    /// aMR_SystemAnimeCt_UniqueCt / aMR move: repeat at speed 0.5.
    System,
    /// fNBC_ct/fNBC_mv: speed ramps 0 -> 0.5, a switch change kicks 1.25.
    Balloon(native::BalloonSpeed),
    /// Cicadas/crickets: standard stop, replayed once at 0.5 per chirp.
    Stopped { speed: f32 },
    /// aSumgenji: standard repeat at 0.5 with a pulsing glow alpha.
    Firefly { counter: i16 },
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

/// Profiles whose original play is reproduced here: fish and insect cages
/// (plain rigs, or the ac_sum_*/ac_nog_* callbacks) and the common balloons.
fn play(profile: &Profile) -> Option<Play> {
    if balloon(profile.index) {
        return Some(Play::Balloon(native::BalloonSpeed::default()));
    }
    let index = u16::try_from(profile.index).ok()?;
    if !(ftr::INSECT_FIRST..ftr::FISH_FIRST + 40).contains(&index) {
        return None;
    }
    match ftr::critter_callback(index) {
        Some(ftr::CritterCallback::Chirp { stopped: true, .. }) => {
            Some(Play::Stopped { speed: 0.0 })
        }
        Some(ftr::CritterCallback::Firefly) => Some(Play::Firefly { counter: 0 }),
        Some(ftr::CritterCallback::Chirp { stopped: false, .. }) => Some(Play::System),
        None => profile.rig.is_some().then_some(Play::System),
    }
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
    /// sAdo_RoomIncectPos argument of a chirping cage.
    pub(crate) chirp: Option<u8>,
    /// The scheduler reported a sound this tick (the callback's "valid").
    pub(crate) chirped: bool,
    /// Firefly materials still holding the segment-8 primitive colour.
    glow: Vec<Handle<N64Material>>,
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
    let colors = matches!(play, Play::Firefly { .. }).then_some((FIREFLY_PRIM, FIREFLY_ENV));
    let (joints, glow) = spawn_skinned_colored(
        commands,
        disc,
        root,
        "rig joint",
        &skeleton,
        &animation,
        lists,
        profile.palette,
        segments,
        colors,
        assets,
    );
    let chirp = u16::try_from(profile.index)
        .ok()
        .and_then(ftr::critter_callback)
        .and_then(|c| match c {
            ftr::CritterCallback::Chirp { argument, .. } => Some(argument),
            ftr::CritterCallback::Firefly => None,
        });
    commands.entity(root).insert(FurnitureRig {
        play,
        skeleton,
        animation,
        joints,
        frame: 1.0,
        ticks: 0.0,
        switch: None,
        kick: false,
        chirp,
        chirped: false,
        glow,
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
    mut materials: ResMut<Assets<N64Material>>,
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
            match rig.play {
                Play::System => {
                    rig.frame = native::repeat_frame(rig.frame, end, native::SYSTEM_RIG_SPEED);
                }
                Play::Balloon(mut speed) => {
                    let speed_now = speed.step(std::mem::take(&mut rig.kick));
                    rig.play = Play::Balloon(speed);
                    rig.frame = native::repeat_frame(rig.frame, end, speed_now);
                }
                Play::Stopped { speed } => {
                    // init_standard_stop restarts the clip; play holds its end.
                    if std::mem::take(&mut rig.chirped) {
                        rig.frame = 1.0;
                        rig.play = Play::Stopped {
                            speed: native::SYSTEM_RIG_SPEED,
                        };
                    }
                    rig.frame = native::stop_frame(rig.frame, end, speed);
                }
                Play::Firefly { counter } => {
                    rig.frame = native::repeat_frame(rig.frame, end, native::SYSTEM_RIG_SPEED);
                    let counter = counter.wrapping_add(1);
                    rig.play = Play::Firefly { counter };
                    let index = (i32::from(counter) % FIREFLY_ALPHA.len() as i32).unsigned_abs();
                    let alpha = f32::from(FIREFLY_ALPHA[index as usize]) / 255.0;
                    for handle in &rig.glow {
                        if let Some(mut m) = materials.get_mut(handle) {
                            m.combiner.prim.w = alpha;
                        }
                    }
                }
            }
        }
        let pose = rig.animation.pose(rig.frame);
        for (&joint, local) in rig.joints.iter().zip(pose.local_transforms(&rig.skeleton)) {
            if let Ok(mut transform) = transforms.get_mut(joint) {
                *transform = local;
            }
        }
    }
}
