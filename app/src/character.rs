//! The player's model from the disc: the boy skeleton (`cKF_bs_r_boy_1`)
//! as a Bevy skinned mesh, animated by the game's keyframe animations
//! (`cKF_ba_r_ply_1_*`).
//!
//! Each joint's display list is interpreted on its own. A vertex is
//! placed by the matrix current when it was loaded: the joint's own (the
//! draw code loads it before calling the list), or another joint's from
//! segment D (`gsSPMatrix(0x0D000000 + slot * 0x40)`), which is how limbs
//! bend at the joints. Vertices stay in their joint's local space, so the
//! inverse bind poses are all identity: skinning multiplies by the joint's
//! world matrix, exactly as the game does.

use std::sync::Arc;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::mesh::{PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use rustimal_logic::graphics::vertex::DecodedVtx;
use rustimal_logic::player_move::LocomotionState;

use crate::disc_gfx::{decode_texture, run_display_lists, Part, Segments};
use crate::game_disc::GameDisc;
use crate::interaction::Facing;
use crate::locomotion::Locomotion;
use crate::n64_material::{texture_image, CombinerUniform, N64Material, ST_ONE};
use crate::player::Player;
use crate::skeleton::{Animation, Skeleton};

/// Model units to world units (the actor draws at scale 0.01; 40 game
/// units per cell).
const MODEL_SCALE: f32 = 0.01 / 40.0;

/// Segment D: the joint matrix array (`cKF_Si3_draw_R_SV`), 0x40 bytes each.
const MATRIX_SEGMENT: u32 = 0x0D;
const MATRIX_SIZE: u32 = 0x40;

/// `gSPSegment`s the player draw sets (`m_player_draw.c_inc`): eye and
/// mouth textures, clothing texture and palette, face palette.
const SEG_EYE: u8 = 0x08;
const SEG_MOUTH: u8 = 0x09;
const SEG_CLOTH: u8 = 0x0A;
const SEG_CLOTH_PALETTE: u8 = 0x0B;
const SEG_FACE_PALETTE: u8 = 0x0C;

/// One face in `face_boy.bin` (`mPlib_Get_UseFaceTexRom_p_common`): eye
/// textures, then mouth textures (0x100 bytes each), then the palette at
/// +0xE00.
const FACE_SIZE: usize = 0xE20;
const FACE_TEX_SIZE: usize = 0x100;
const FACE_PALETTE: usize = 0xE00;
/// Eye textures before the mouths (`mPlayer_EYE_TEX_NUM`).
const EYE_TEX_NUM: usize = 8;
/// `mNW_DESIGN_TEX_SIZE` / `mNW_PALETTE_SIZE`.
const CLOTH_TEX_SIZE: usize = 0x200;
const PALETTE_SIZE: usize = 0x20;

/// The player's look; from the save once it has one (face type, shirt).
#[derive(Clone, Copy, Debug, Default)]
pub struct PlayerLook {
    /// `mPr_FACE_TYPE*`, 0-7.
    pub face: usize,
    /// Shirt index into the clothing bank.
    pub cloth: usize,
}

/// The segments the player draw points at for `look` (boy, not swollen,
/// eyes and mouth in their first, neutral frames).
fn player_segments(disc: &GameDisc, look: PlayerLook) -> Segments {
    let slice = |bank: &std::sync::Arc<[u8]>, start: usize, len: usize| -> std::sync::Arc<[u8]> {
        bank.get(start..start + len).unwrap_or_default().into()
    };
    // `mPlib_Get_UseFaceRom_index` (texture): face + sex * 8 + swell * 16.
    let face = look.face.min(7) * FACE_SIZE;
    let mut segs = Segments::default();
    segs.set_segment(SEG_EYE, slice(&disc.player_faces, face, FACE_TEX_SIZE));
    segs.set_segment(
        SEG_MOUTH,
        slice(
            &disc.player_faces,
            face + EYE_TEX_NUM * FACE_TEX_SIZE,
            FACE_TEX_SIZE,
        ),
    );
    segs.set_segment(
        SEG_FACE_PALETTE,
        slice(&disc.player_faces, face + FACE_PALETTE, PALETTE_SIZE),
    );
    segs.set_segment(
        SEG_CLOTH,
        slice(
            &disc.player_cloth,
            look.cloth * CLOTH_TEX_SIZE,
            CLOTH_TEX_SIZE,
        ),
    );
    segs.set_segment(
        SEG_CLOTH_PALETTE,
        slice(
            &disc.player_cloth_palettes,
            look.cloth * PALETTE_SIZE,
            PALETTE_SIZE,
        ),
    );
    segs
}

/// Animation frames per 60 Hz tick while standing (`cKF` default 0.5).
const WAIT_SPEED: f32 = 0.5;

/// What the model is doing, from the player's movement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Motion {
    Wait = 0,
    Walk = 1,
    Run = 2,
}

impl Motion {
    fn of(state: LocomotionState) -> Self {
        match state {
            LocomotionState::Walk => Motion::Walk,
            LocomotionState::Run | LocomotionState::Dash => Motion::Run,
            _ => Motion::Wait,
        }
    }
}

/// Player animations, indexed by [`Motion`].
const ANIMATIONS: [&str; 3] = [
    "cKF_ba_r_ply_1_wait1",
    "cKF_ba_r_ply_1_walk1",
    "cKF_ba_r_ply_1_run1",
];

/// A drawn skeleton: its joints' entities and animations.
#[derive(Component)]
pub struct CharacterRig {
    skeleton: Arc<Skeleton>,
    joints: Vec<Entity>,
    animations: [Option<Arc<Animation>>; 3],
    playing: Motion,
    frame: f32,
}

/// Which joint places a vertex: its load-time matrix (see module docs).
fn joint_of(v: &DecodedVtx, own: usize, matrix_joints: &[usize]) -> usize {
    if v.mtx >> 24 != MATRIX_SEGMENT {
        return own;
    }
    let slot = ((v.mtx & 0x00FF_FFFF) / MATRIX_SIZE) as usize;
    matrix_joints.get(slot).copied().unwrap_or(own)
}

/// A part's mesh in joint-local model units, skinned to `joint_of`.
fn skinned_part_mesh(part: &Part, own: usize, matrix_joints: &[usize]) -> Mesh {
    let (w, h) = part
        .key
        .tex
        .map_or((1.0, 1.0), |k| (k.width as f32, k.height as f32));
    let v = &part.vertices;
    let positions: Vec<[f32; 3]> = v.iter().map(|v| v.pos).collect();
    let uvs: Vec<[f32; 2]> = v
        .iter()
        .map(|v| [v.st[0] / ST_ONE / w, v.st[1] / ST_ONE / h])
        .collect();
    let joints: Vec<[u16; 4]> = v
        .iter()
        .map(|v| [joint_of(v, own, matrix_joints) as u16, 0, 0, 0])
        .collect();
    let weights = vec![[1.0f32, 0.0, 0.0, 0.0]; v.len()];
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(joints),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, weights);
    if part.key.lit {
        let normals: Vec<[f32; 3]> = v.iter().map(|v| v.normal).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    } else {
        let colors: Vec<[f32; 4]> = v
            .iter()
            .map(|v| v.color.map(|c| c as f32 / 255.0))
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.compute_flat_normals();
    }
    mesh
}

/// Replaces the player's placeholder capsule with the disc model, once.
#[allow(clippy::too_many_arguments)]
fn spawn_player_model(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    players: Query<Entity, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
    mut bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    let (Some(disc), Ok(player)) = (disc, players.single()) else {
        return;
    };
    *done = true;
    let rom = &disc.rom;
    let Some(skeleton) = disc
        .catalog
        .get("cKF_bs_r_boy_1")
        .and_then(|a| Skeleton::read(rom, a.offset))
    else {
        warn!("player skeleton not found on the disc");
        return;
    };
    let animations = ANIMATIONS.map(|name| {
        let a = disc.catalog.get(name)?;
        Some(Arc::new(Animation::read(
            rom,
            a.offset,
            skeleton.joints.len(),
        )?))
    });
    let Some(wait) = &animations[Motion::Wait as usize] else {
        warn!("player animations not found on the disc");
        return;
    };

    // The model hangs under the player, at the feet.
    let feet = Vec3::Y * -crate::disc_town::PLAYER_HALF_HEIGHT;
    let root = commands
        .spawn((
            Name::new("player model"),
            Transform::from_translation(feet).with_scale(Vec3::splat(MODEL_SCALE)),
            Visibility::default(),
            ChildOf(player),
        ))
        .id();
    let locals = wait.pose(1.0).local_transforms(&skeleton);
    let mut joints: Vec<Entity> = Vec::with_capacity(skeleton.joints.len());
    for (j, local) in locals.iter().enumerate() {
        let parent = skeleton.parents[j].map_or(root, |p| joints[p]);
        joints.push(
            commands
                .spawn((*local, Visibility::default(), ChildOf(parent)))
                .id(),
        );
    }
    let inverse = bindposes.add(SkinnedMeshInverseBindposes::from(vec![
        Mat4::IDENTITY;
        joints.len()
    ]));
    let matrix_joints = skeleton.matrix_joints();
    let segs = player_segments(&disc, PlayerLook::default());
    let mut parts = 0;
    for &j in &matrix_joints {
        let Some(list) = skeleton.joints[j].model else {
            continue;
        };
        for part in &run_display_lists(rom, &[list]).parts {
            let texture = part.key.tex.as_ref().and_then(|k| {
                decode_texture(rom, k, None, &segs).map(|rgba| images.add(texture_image(rgba, k)))
            });
            let material = materials.add(N64Material {
                combiner: CombinerUniform::from_key(&part.key, false, false),
                texture,
                env: default(),
                blend: false,
            });
            commands.spawn((
                Mesh3d(meshes.add(skinned_part_mesh(part, j, &matrix_joints))),
                MeshMaterial3d(material),
                SkinnedMesh {
                    inverse_bindposes: inverse.clone(),
                    joints: joints.clone(),
                },
                ChildOf(root),
            ));
            parts += 1;
        }
    }
    commands.entity(root).insert(CharacterRig {
        skeleton: Arc::new(skeleton),
        joints,
        animations,
        playing: Motion::Wait,
        frame: 1.0,
    });
    commands
        .entity(player)
        .remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>)>();
    info!("player model from the disc: {parts} parts");
}

/// Plays the animation for the player's movement and turns the model to
/// face where the player faces.
fn animate_player(
    time: Res<Time>,
    players: Query<(&Locomotion, &Facing), With<Player>>,
    mut rigs: Query<(Entity, &mut CharacterRig)>,
    mut transforms: Query<&mut Transform>,
) {
    let Ok((loco, facing)) = players.single() else {
        return;
    };
    // Facing angle as the actor's shape rotation: 0 = +z, 90 deg = +x.
    let yaw = Quat::from_rotation_y(facing.dir.x.atan2(facing.dir.z));
    let ticks = time.delta_secs() * 60.0;
    for (root, mut rig) in &mut rigs {
        if let Ok(mut t) = transforms.get_mut(root) {
            t.rotation = yaw;
        }
        let state = Motion::of(loco.state());
        if state != rig.playing && rig.animations[state as usize].is_some() {
            rig.playing = state;
            rig.frame = 1.0;
        }
        let Some(anim) = rig.animations[rig.playing as usize].clone() else {
            continue;
        };
        let speed = if rig.playing == Motion::Wait {
            WAIT_SPEED
        } else {
            loco.anim_speed()
        };
        // Repeat between frame 1 and the last (`cKF_FrameControl_repeat_proc`).
        let last = anim.frames.max(2) as f32;
        rig.frame += speed * ticks;
        while rig.frame >= last {
            rig.frame -= last - 1.0;
        }
        let locals = anim.pose(rig.frame).local_transforms(&rig.skeleton);
        for (&joint, local) in rig.joints.iter().zip(locals) {
            if let Ok(mut t) = transforms.get_mut(joint) {
                *t = local;
            }
        }
    }
}

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_player_model, animate_player).chain());
    }
}
