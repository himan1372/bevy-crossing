//! Character models from the disc: the player (`cKF_bs_r_boy_1`) and the
//! villagers (their `npc_draw_data_tbl` entries) as Bevy skinned meshes,
//! animated by the game's keyframe animations (`cKF_ba_r_ply_1_*`,
//! `cKF_ba_r_npc_1_*`).
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

use crate::disc_gfx::{decode_texture, run_display_lists_with_tluts, Part, Segments};
use crate::face::{Face, FaceKind, FacePart, FaceSlot};
use crate::game_disc::GameDisc;
use crate::interaction::Facing;
use crate::locomotion::Locomotion;
use crate::n64_material::{texture_image, CombinerUniform, N64Material, ST_ONE};
use crate::npc::Villager;
use crate::player::Player;
use crate::skeleton::{Animation, Skeleton};
use disc::{RomImage, RomSource, Target};

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

/// Mouth textures after the eyes (`mPlayer_MOUTH_TEX_NUM`).
const MOUTH_TEX_NUM: usize = 6;

/// `count` face textures from `first` in the player's face block.
fn player_face_frames(
    disc: &GameDisc,
    look: PlayerLook,
    first: usize,
    count: usize,
) -> Vec<Option<Arc<[u8]>>> {
    let face = look.face.min(7) * FACE_SIZE;
    (first..first + count)
        .map(|i| {
            let start = face + i * FACE_TEX_SIZE;
            disc.player_faces
                .get(start..start + FACE_TEX_SIZE)
                .map(Arc::from)
        })
        .collect()
}

/// Animation frames per 60 Hz tick while standing (`cKF` default 0.5).
const WAIT_SPEED: f32 = 0.5;

/// What the model is doing, from the player's movement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Motion {
    Wait = 0,
    Walk = 1,
    Run = 2,
    /// Played once, then held (`aNPC_ANIM_AISATU1`).
    Greet = 3,
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

/// Player animations, indexed by [`Motion`] (no greeting).
const PLAYER_ANIMATIONS: [&str; 4] = [
    "cKF_ba_r_ply_1_wait1",
    "cKF_ba_r_ply_1_walk1",
    "cKF_ba_r_ply_1_run1",
    "",
];

/// Villager animations (shared by every species' 26-joint skeleton).
/// Talking (`aNPC_ANIM_TALK1`) uses `wait1` too, with the mouth moving.
const NPC_ANIMATIONS: [&str; 4] = [
    "cKF_ba_r_npc_1_wait1",
    "cKF_ba_r_npc_1_walk1",
    "cKF_ba_r_npc_1_run1",
    "cKF_ba_r_npc_1_aisatu1",
];

/// A drawn skeleton: its joints' entities and animations.
#[derive(Component)]
pub struct CharacterRig {
    skeleton: Arc<Skeleton>,
    joints: Vec<Entity>,
    animations: [Option<Arc<Animation>>; 4],
    playing: Motion,
    frame: f32,
    /// Owner's last world position (villagers: motion from movement).
    last_pos: Option<Vec3>,
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

/// What a rig is drawn from: skeleton, animations, the segments and TLUT
/// slots its draw routine sets, and its scale (model units to world).
struct RigSource {
    skeleton: Skeleton,
    animations: [Option<Arc<Animation>>; 4],
    segments: Segments,
    tluts: Vec<(usize, u32)>,
    scale: f32,
    /// Every eye / mouth texture, by pattern (segments 8 and 9).
    eye_frames: Vec<Option<Arc<[u8]>>>,
    mouth_frames: Vec<Option<Arc<[u8]>>>,
    face: FaceKind,
}

/// Bevy asset stores a rig is built into.
type RigAssets<'a> = (
    &'a mut Assets<Mesh>,
    &'a mut Assets<N64Material>,
    &'a mut Assets<Image>,
    &'a mut Assets<SkinnedMeshInverseBindposes>,
);

/// Spawns a skinned model under `parent` (at its feet), returning the
/// number of mesh parts.
fn spawn_rig(
    commands: &mut Commands,
    rom: &RomImage,
    src: RigSource,
    parent: Entity,
    name: String,
    (meshes, materials, images, bindposes): RigAssets,
) -> usize {
    let Some(wait) = src.animations[Motion::Wait as usize].clone() else {
        return 0;
    };
    let skeleton = src.skeleton;
    let feet = Vec3::Y * -crate::disc_town::PLAYER_HALF_HEIGHT;
    let root = commands
        .spawn((
            Name::new(name),
            Transform::from_translation(feet).with_scale(Vec3::splat(src.scale)),
            Visibility::default(),
            ChildOf(parent),
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
    let mut parts = 0;
    let mut face_parts = Vec::new();
    for &j in &matrix_joints {
        let Some(list) = skeleton.joints[j].model else {
            continue;
        };
        for part in &run_display_lists_with_tluts(rom, &[list], &src.tluts).parts {
            let texture = part.key.tex.as_ref().and_then(|k| {
                decode_texture(rom, k, None, &src.segments)
                    .map(|rgba| images.add(texture_image(rgba, k)))
            });
            let material = materials.add(N64Material {
                combiner: CombinerUniform::from_key(&part.key, false, false),
                texture,
                env: default(),
                blend: false,
            });
            // Face parts: decode every frame of their segment.
            let face_slot = part
                .key
                .tex
                .as_ref()
                .and_then(|k| match (k.img_addr >> 24) as u8 {
                    SEG_EYE => Some((FaceSlot::Eye, SEG_EYE, &src.eye_frames, k)),
                    SEG_MOUTH => Some((FaceSlot::Mouth, SEG_MOUTH, &src.mouth_frames, k)),
                    _ => None,
                });
            if let Some((slot, seg, frames, k)) = face_slot {
                let frames = frames
                    .iter()
                    .map(|bytes| {
                        let mut segs = src.segments.clone();
                        segs.set_segment(seg, bytes.clone()?);
                        decode_texture(rom, k, None, &segs)
                            .map(|rgba| images.add(texture_image(rgba, k)))
                    })
                    .collect();
                face_parts.push(FacePart {
                    slot,
                    material: material.clone(),
                    frames,
                });
            }
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
        animations: src.animations,
        playing: Motion::Wait,
        frame: 1.0,
        last_pos: None,
    });
    debug!("{} face parts", face_parts.len());
    if !face_parts.is_empty() {
        commands
            .entity(root)
            .insert(Face::new(src.face, face_parts, parent));
    }
    parts
}

fn read_animations(
    disc: &GameDisc,
    names: [&str; 4],
    joints: usize,
) -> [Option<Arc<Animation>>; 4] {
    names.map(|name| {
        let a = disc.catalog.get(name)?;
        Some(Arc::new(Animation::read(&disc.rom, a.offset, joints)?))
    })
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
    let Some(skeleton) = disc
        .catalog
        .get("cKF_bs_r_boy_1")
        .and_then(|a| Skeleton::read(&disc.rom, a.offset))
    else {
        warn!("player skeleton not found on the disc");
        return;
    };
    let animations = read_animations(&disc, PLAYER_ANIMATIONS, skeleton.joints.len());
    let src = RigSource {
        skeleton,
        animations,
        segments: player_segments(&disc, PlayerLook::default()),
        tluts: Vec::new(),
        scale: MODEL_SCALE,
        eye_frames: player_face_frames(&disc, PlayerLook::default(), 0, EYE_TEX_NUM),
        mouth_frames: player_face_frames(&disc, PlayerLook::default(), EYE_TEX_NUM, MOUTH_TEX_NUM),
        face: FaceKind::Player,
    };
    let assets = (&mut *meshes, &mut *materials, &mut *images, &mut *bindposes);
    let parts = spawn_rig(
        &mut commands,
        &disc.rom,
        src,
        player,
        "player model".into(),
        assets,
    );
    if parts == 0 {
        warn!("player animations not found on the disc");
        return;
    }
    commands
        .entity(player)
        .remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>)>();
    info!("player model from the disc: {parts} parts");
}

/// `npc_draw_data_tbl` entry size (`aNPC_draw_data_c`).
const DRAW_DATA_SIZE: u32 = 0x6C;
/// Segment and TLUT slot the villager draw sets (`ac_npc_draw`) for the
/// body: texture in segment B, palette in TLUT 15. Eyes and mouth use
/// segments 8 and 9 like the player's.
const SEG_NPC_BODY: u8 = 0x0B;
const TLUT_NPC_BODY: usize = 15;
/// The villager draw also puts the shirt texture in segment A and its
/// palette in TLUT 14 (`ac_npc_draw`). Our TLUT presets are addresses, so
/// the palette is served from an otherwise unused segment.
const SEG_NPC_CLOTH: u8 = 0x0A;
const TLUT_NPC_CLOTH: usize = 14;
const SEG_NPC_CLOTH_PALETTE: u8 = 0x0E;
/// `npc_def_list` entry (`mNpc_Default_Data_c`): cloth item u16,
/// catchphrase u16, umbrella s8, padded to 6 bytes.
const NPC_DEFAULT_SIZE: u32 = 6;
/// `ITM_CLOTH000`: shirt items index the clothing banks from here
/// (`aNPC_getP_cloth_tex_rom`).
const ITM_CLOTH000: u16 = 0x2400;

/// Bytes of one eye or mouth texture (32x16, 4 bits per texel).
const NPC_FACE_TEX_SIZE: usize = 0x100;
/// Upper bound on a body texture (`*_tmem_txt`); segments only bound reads.
const NPC_BODY_TEX_MAX: usize = 0x2000;

/// Index into `npc_draw_data_tbl` (NPC id - `NPC_START`) for our
/// villagers, from the disc's `npc_name_str_table.bin`.
fn villager_draw_index(name: &str) -> Option<u32> {
    match name {
        "Bob" => Some(0),
        "Maple" => Some(51),
        "Chief" => Some(187),
        "Pippy" => Some(206),
        _ => None,
    }
}

/// A villager's rig source from its `npc_draw_data_tbl` entry: skeleton,
/// body texture and palette, first eye and mouth textures, scale.
fn villager_source(disc: &GameDisc, index: u32) -> Option<RigSource> {
    let rom = &disc.rom;
    let entry = disc.catalog.get("npc_draw_data_tbl")?.offset + index * DRAW_DATA_SIZE;
    let ptr = |o: u32| match rom.pointer_at(RomSource::Rel, entry + o) {
        Some(Target::File(RomSource::Rel, p)) => Some(p),
        _ => None,
    };
    let bytes = |p: Option<u32>, len: usize| -> Option<Arc<[u8]>> {
        let p = p? as usize;
        let rel = rom.bytes(RomSource::Rel);
        Some(rel.get(p..(p + len).min(rel.len()))?.into())
    };
    let skeleton = Skeleton::read(rom, ptr(0x04)?)?;
    let mut segments = Segments::default();
    segments.set_segment(SEG_NPC_BODY, bytes(ptr(0x08), NPC_BODY_TEX_MAX)?);
    if let Some(eye) = bytes(ptr(0x10), NPC_FACE_TEX_SIZE) {
        segments.set_segment(SEG_EYE, eye);
    }
    if let Some(mouth) = bytes(ptr(0x30), NPC_FACE_TEX_SIZE) {
        segments.set_segment(SEG_MOUTH, mouth);
    }
    // Default shirt (`mNpc_SetDefAnimal`: npc_def_list[id].cloth), from the
    // same banks as the player's.
    let mut tluts: Vec<(usize, u32)> = ptr(0x0C)
        .map(|p| vec![(TLUT_NPC_BODY, p)])
        .unwrap_or_default();
    let cloth = disc.catalog.get("npc_def_list").and_then(|d| {
        let b = rom.slice(RomSource::Rel, d.offset + index * NPC_DEFAULT_SIZE, 2)?;
        u16::from_be_bytes([b[0], b[1]]).checked_sub(ITM_CLOTH000)
    });
    if let Some(cloth) = cloth.map(usize::from) {
        let tex = disc
            .player_cloth
            .get(cloth * CLOTH_TEX_SIZE..(cloth + 1) * CLOTH_TEX_SIZE);
        let pal = disc
            .player_cloth_palettes
            .get(cloth * PALETTE_SIZE..(cloth + 1) * PALETTE_SIZE);
        if let (Some(tex), Some(pal)) = (tex, pal) {
            segments.set_segment(SEG_NPC_CLOTH, tex.into());
            segments.set_segment(SEG_NPC_CLOTH_PALETTE, pal.into());
            tluts.push((TLUT_NPC_CLOTH, (SEG_NPC_CLOTH_PALETTE as u32) << 24));
        }
    }
    // `aNPC_draw_tex_data_c`: eye_texture[8] at +0x08, mouth_texture[6] at
    // +0x28 (entry +0x10 / +0x30).
    let eye_frames = (0..8)
        .map(|i| bytes(ptr(0x10 + i * 4), NPC_FACE_TEX_SIZE))
        .collect();
    let mouth_frames = (0..6)
        .map(|i| bytes(ptr(0x30 + i * 4), NPC_FACE_TEX_SIZE))
        .collect();
    let raw = rom.slice(RomSource::Rel, entry + 0x54, 4)?;
    let scale = f32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]);
    let animations = read_animations(disc, NPC_ANIMATIONS, skeleton.joints.len());
    Some(RigSource {
        skeleton,
        animations,
        segments,
        tluts,
        scale: scale / 40.0,
        eye_frames,
        mouth_frames,
        face: FaceKind::Villager,
    })
}

/// Marks a character whose disc model has been tried.
#[derive(Component)]
struct CharacterModelDone;

/// Gives each villager its disc model, once the disc is loaded.
#[allow(clippy::too_many_arguments)]
fn spawn_villager_models(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    villagers: Query<(Entity, &Villager), Without<CharacterModelDone>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
    mut bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    let Some(disc) = disc else { return };
    for (entity, villager) in &villagers {
        commands.entity(entity).insert(CharacterModelDone);
        let Some(src) = villager_draw_index(villager.name).and_then(|i| villager_source(&disc, i))
        else {
            warn!("no disc model for villager {}", villager.name);
            continue;
        };
        let assets = (&mut *meshes, &mut *materials, &mut *images, &mut *bindposes);
        let name = format!("{} model", villager.name);
        let parts = spawn_rig(&mut commands, &disc.rom, src, entity, name, assets);
        if parts > 0 {
            commands
                .entity(entity)
                .remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>)>();
            info!(
                "villager {} model from the disc: {parts} parts",
                villager.name
            );
        }
    }
}

/// Steps a rig's animation to `motion` at `speed` frames per 60 Hz tick.
fn play(
    rig: &mut CharacterRig,
    motion: Motion,
    speed: f32,
    ticks: f32,
    transforms: &mut Query<&mut Transform>,
) {
    if motion != rig.playing && rig.animations[motion as usize].is_some() {
        rig.playing = motion;
        rig.frame = 1.0;
    }
    let Some(anim) = rig.animations[rig.playing as usize].clone() else {
        return;
    };
    // Repeat between frame 1 and the last (`cKF_FrameControl_repeat_proc`);
    // a greeting plays once and holds its last frame (stop mode).
    let last = anim.frames.max(2) as f32;
    rig.frame += speed * ticks;
    if rig.playing == Motion::Greet {
        rig.frame = rig.frame.min(last);
    } else {
        while rig.frame >= last {
            rig.frame -= last - 1.0;
        }
    }
    let locals = anim.pose(rig.frame).local_transforms(&rig.skeleton);
    for (&joint, local) in rig.joints.iter().zip(locals) {
        if let Ok(mut t) = transforms.get_mut(joint) {
            *t = local;
        }
    }
}

/// World units per second above which a villager walks / runs (between
/// their wander speeds, see `npc.rs`).
const VILLAGER_WALK_MIN: f32 = 0.3;
const VILLAGER_RUN_MIN: f32 = 2.5;

/// Plays each rig's animation: the player's from its locomotion (turning
/// the model to its facing), villagers' from how fast they move.
fn animate_characters(
    time: Res<Time>,
    players: Query<(&Locomotion, &Facing), With<Player>>,
    villagers: Query<(&GlobalTransform, &Villager)>,
    mut rigs: Query<(Entity, &ChildOf, &mut CharacterRig)>,
    mut transforms: Query<&mut Transform>,
) {
    let dt = time.delta_secs();
    let ticks = dt * 60.0;
    for (root, child_of, mut rig) in &mut rigs {
        let owner = child_of.parent();
        if let Ok((loco, facing)) = players.get(owner) {
            // Facing angle as the actor's shape rotation: 0 = +z, 90 deg = +x.
            if let Ok(mut t) = transforms.get_mut(root) {
                t.rotation = Quat::from_rotation_y(facing.dir.x.atan2(facing.dir.z));
            }
            let motion = Motion::of(loco.state());
            let speed = if motion == Motion::Wait {
                WAIT_SPEED
            } else {
                loco.anim_speed()
            };
            play(&mut rig, motion, speed, ticks, &mut transforms);
        } else if let Ok((g, villager)) = villagers.get(owner) {
            let pos = g.translation();
            let moved = rig.last_pos.map_or(0.0, |p| (pos - p).with_y(0.0).length());
            rig.last_pos = Some(pos);
            let speed = if dt > 0.0 { moved / dt } else { 0.0 };
            let motion = if villager.is_greeting() {
                Motion::Greet
            } else if speed >= VILLAGER_RUN_MIN {
                Motion::Run
            } else if speed >= VILLAGER_WALK_MIN {
                Motion::Walk
            } else {
                Motion::Wait
            };
            play(&mut rig, motion, WAIT_SPEED, ticks, &mut transforms);
        }
    }
}

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_player_model,
                spawn_villager_models,
                animate_characters,
            )
                .chain(),
        );
    }
}
