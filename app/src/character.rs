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

use crate::dialogue::VillagerTalk;
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

/// The player's look (face type, shirt), kept in the save.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct PlayerLook {
    /// `mPr_FACE_TYPE*`, 0-7.
    pub face: usize,
    /// Shirt index into the clothing bank.
    pub cloth: usize,
    /// `mPr_SEX_FEMALE`: the face bank holds the girls' faces after the boys' (face + sex * 8).
    pub female: bool,
}

/// Shirts a new boy can start in (`mPr_boy_cloth_table`: ITM_CLOTH000-007).
const NEW_BOY_CLOTHS: usize = 8;
/// `mPr_FACE_TYPE_NUM`.
const FACE_TYPES: usize = 8;

impl PlayerLook {
    /// A new player's look (`mPr_GetRandomFace`, `mPr_GetRandomCloth`),
    /// from any random `seed`.
    pub fn new_player(seed: u32) -> Self {
        let mut x = seed | 1;
        let mut next = || {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            x as usize
        };
        Self {
            face: next() % FACE_TYPES,
            cloth: next() % NEW_BOY_CLOTHS,
            female: false,
        }
    }
}

/// The segments the player draw points at for `look` (boy, not swollen,
/// eyes and mouth in their first, neutral frames).
pub(crate) fn player_segments(disc: &GameDisc, look: PlayerLook) -> Segments {
    let slice = |bank: &std::sync::Arc<[u8]>, start: usize, len: usize| -> std::sync::Arc<[u8]> {
        bank.get(start..start + len).unwrap_or_default().into()
    };
    // `mPlib_Get_UseFaceRom_index` (texture): face + sex * 8 + swell * 16.
    let face = (look.face.min(7) + usize::from(look.female) * 8) * FACE_SIZE;
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
    let face = (look.face.min(7) + usize::from(look.female) * 8) * FACE_SIZE;
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
    /// Villager moods (`aNPC_ANIM_WAIT_KI1` / `WALK_KI1`: joy,
    /// `WAIT_DO1` / `WALK_DO1`: anger).
    WaitJoy = 4,
    WalkJoy = 5,
    WaitAnger = 6,
    WalkAnger = 7,
}

/// Animations per rig, indexed by [`Motion`].
const MOTIONS: usize = 8;

/// A villager's mood, which picks the stand/walk variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Feel {
    Normal,
    Joy,
    Anger,
}

impl Motion {
    /// The mood's variant of standing or walking, if it has one.
    fn with_feel(self, feel: Feel) -> Self {
        match (self, feel) {
            (Motion::Wait, Feel::Joy) => Motion::WaitJoy,
            (Motion::Walk, Feel::Joy) => Motion::WalkJoy,
            (Motion::Wait, Feel::Anger) => Motion::WaitAnger,
            (Motion::Walk, Feel::Anger) => Motion::WalkAnger,
            (m, _) => m,
        }
    }

    /// Played once and held rather than looped.
    fn once(self) -> bool {
        self == Motion::Greet
    }
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
const PLAYER_ANIMATIONS: [&str; MOTIONS] = [
    "cKF_ba_r_ply_1_wait1",
    "cKF_ba_r_ply_1_walk1",
    "cKF_ba_r_ply_1_run1",
    "",
    "",
    "",
    "",
    "",
];

/// Villager animations (shared by every species' 26-joint skeleton).
/// Talking (`aNPC_ANIM_TALK1`) uses `wait1` too, with the mouth moving.
const NPC_ANIMATIONS: [&str; MOTIONS] = [
    "cKF_ba_r_npc_1_wait1",
    "cKF_ba_r_npc_1_walk1",
    "cKF_ba_r_npc_1_run1",
    "cKF_ba_r_npc_1_aisatu1",
    "cKF_ba_r_npc_1_wait_ki1",
    "cKF_ba_r_npc_1_walk_ki1",
    "cKF_ba_r_npc_1_wait_do1",
    "cKF_ba_r_npc_1_walk_do1",
];

/// A drawn skeleton: its joints' entities and animations.
#[derive(Component)]
pub struct CharacterRig {
    skeleton: Arc<Skeleton>,
    joints: Vec<Entity>,
    animations: [Option<Arc<Animation>>; MOTIONS],
    playing: Motion,
    frame: f32,
    /// Owner's last world position (villagers: motion from movement).
    last_pos: Option<Vec3>,
    net_active: bool,
    net_held: bool,
    furniture_phase: Option<u8>,
    toy_phase: Option<(u16, bool)>,
    pose_morph: f32,
    /// The special clip playing (`NpcClip`), by name.
    clip: Option<(&'static str, Arc<Animation>, ClipEyes)>,
}

/// The eye pattern an NPC animation asks for (`aNPC_Animation_c`: `eye_seq_p`, `eye_seq_type`,
/// `eye_seq_stop_frame`): a pattern for every frame, a pattern held for the whole clip (type 0:
/// the sleeping old woman's closed eyes), or none (the villagers' random blinks).
#[derive(Clone, Default)]
pub(crate) struct ClipEyes {
    table: Option<Arc<[u8]>>,
    kind: i16,
    stop: i16,
}

impl ClipEyes {
    fn read(disc: &GameDisc, offset: u32, frames: usize) -> Self {
        let rom = &disc.rom;
        let table = match rom.pointer_at(RomSource::Rel, offset + 0x24) {
            Some(Target::File(RomSource::Rel, p)) => rom
                .bytes(RomSource::Rel)
                .get(p as usize..p as usize + frames)
                .map(Arc::from),
            _ => None,
        };
        let short = |o: u32| {
            rom.slice(RomSource::Rel, offset + o, 2)
                .map_or(0, |b| i16::from_be_bytes([b[0], b[1]]))
        };
        Self {
            table,
            kind: short(0x28),
            stop: short(0x2A),
        }
    }

    /// The eye pattern at `frame` (1-based), if the clip decides it.
    fn pattern(&self, frame: f32) -> Option<u8> {
        if let Some(table) = &self.table {
            let i = (frame.floor() as usize).saturating_sub(1);
            return table.get(i.min(table.len().saturating_sub(1))).copied();
        }
        (self.kind == 0 && self.stop >= 0).then_some(self.stop as u8)
    }
}

/// A special NPC's own disc animation in place of standing (Copper's
/// exercises and dozing): `cKF_ba_r_npc_1_*` by name, looped (counting
/// `loops`) or played once and held (`done`). `morph` is the blend in
/// frames (the clip's `morph_counter`).
#[derive(Component, Default)]
pub struct NpcClip {
    pub name: Option<&'static str>,
    pub once: bool,
    pub morph: f32,
    pub loops: u32,
    pub done: bool,
}

/// A special NPC's stride: 0 standing, 1 walking, 2 running.
#[derive(Component, Default, Clone, Copy)]
pub struct NpcGait(pub u8);

impl NpcClip {
    pub fn start(&mut self, name: &'static str, once: bool, morph: f32) {
        *self = Self {
            name: Some(name),
            once,
            morph,
            ..default()
        };
    }

    pub fn stop(&mut self, morph: f32) {
        *self = Self { morph, ..default() };
    }
}

impl CharacterRig {
    /// m_player_draw.c_inc::Player_actor_draw_After: right hand callback20.
    /// The eye pattern the playing clip asks for, if it does (`aNPC_tex_anm_ctrl_fix_seq` and the
    /// stop frame of a type-0 sequence).
    pub(crate) fn eye_pattern(&self) -> Option<u8> {
        let (_, _, eyes) = self.clip.as_ref()?;
        eyes.pattern(self.frame)
    }

    pub(crate) fn right_hand(&self) -> Option<Entity> {
        self.joints.get(20).copied()
    }
    pub(crate) fn joint_count(&self) -> usize {
        self.joints.len()
    }
    /// Player_actor_draw_After_feel: joint callback25 supplies ef_kigae's centre.
    pub(crate) fn feel_joint(&self) -> Option<Entity> {
        self.joints.get(25).copied()
    }
    /// Player_actor_draw_After_Larm2: callback16, VecX(1100).
    pub(crate) fn left_hand_position(
        &self,
        root: &Transform,
        player: &Transform,
        transforms: &Query<&Transform>,
    ) -> Option<Vec3> {
        self.left_hand_point(root, player, transforms, Vec3::X * 1100.0)
    }

    /// A point in joint 16's frame (`aNPC_set_left_hand_item` uses its origin).
    pub(crate) fn left_hand_point(
        &self,
        root: &Transform,
        player: &Transform,
        transforms: &Query<&Transform>,
        local: Vec3,
    ) -> Option<Vec3> {
        let locals = self
            .joints
            .iter()
            .map(|&j| transforms.get(j).copied().ok())
            .collect::<Option<Vec<_>>>()?;
        let world = self.skeleton.world_from_locals(&locals);
        Some((player.to_matrix() * root.to_matrix() * world.get(16)?).transform_point3(local))
    }
}

/// Which joint places a vertex: its load-time matrix (see module docs).
fn joint_of(v: &DecodedVtx, own: usize, matrix_joints: &[usize]) -> usize {
    // Anything but a segment-D matrix (incl. the list tag) is the joint's own.
    if v.mtx >> 24 != MATRIX_SEGMENT {
        return own;
    }
    let slot = ((v.mtx & 0x00FF_FFFF) / MATRIX_SIZE) as usize;
    matrix_joints.get(slot).copied().unwrap_or(own)
}

/// A part's mesh in joint-local model units, skinned to `joint_of`.
pub(crate) fn skinned_part_mesh(part: &Part, own: usize, matrix_joints: &[usize]) -> Mesh {
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
        if part.key.texgen.is_some() {
            // emu64 texture_gen reads the same vertex bytes as normals.
            let normals: Vec<[f32; 3]> = v.iter().map(|v| v.normal).collect();
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        } else {
            mesh.compute_flat_normals();
        }
    }
    mesh
}

/// What a rig is drawn from: skeleton, animations, the segments and TLUT
/// slots its draw routine sets, and its scale (model units to world).
struct RigSource {
    skeleton: Skeleton,
    animations: [Option<Arc<Animation>>; MOTIONS],
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
                texture1: None,
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
            let preview = (src.face == FaceKind::Player
                && part
                    .key
                    .tex
                    .is_some_and(|k| k.img_addr >> 24 == u32::from(SEG_CLOTH)))
            .then(|| {
                crate::shop_nook::preview::PreviewTexture::new(
                    crate::shop_nook::preview::Surface::Shirt,
                    part.key,
                    materials
                        .get(&material)
                        .expect("just added player material"),
                )
            });
            let mut entity = commands.spawn((
                Mesh3d(meshes.add(skinned_part_mesh(part, j, &matrix_joints))),
                MeshMaterial3d(material),
                SkinnedMesh {
                    inverse_bindposes: inverse.clone(),
                    joints: joints.clone(),
                },
                ChildOf(root),
            ));
            if let Some(preview) = preview {
                entity.insert(preview);
            }
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
        net_active: false,
        net_held: false,
        furniture_phase: None,
        toy_phase: None,
        pose_morph: 0.0,
        clip: None,
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
    names: [&str; MOTIONS],
    joints: usize,
) -> [Option<Arc<Animation>>; MOTIONS] {
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
    look: Option<Res<PlayerLook>>,
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
    let look = look.map_or_else(PlayerLook::default, |l| *l);
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
        segments: player_segments(&disc, look),
        tluts: Vec::new(),
        scale: MODEL_SCALE,
        eye_frames: player_face_frames(&disc, look, 0, EYE_TEX_NUM),
        mouth_frames: player_face_frames(&disc, look, EYE_TEX_NUM, MOUTH_TEX_NUM),
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
    // Only villagers have a default shirt (`npc_def_list` has one entry
    // per villager); special NPCs wear what their texture shows.
    let cloth = disc
        .catalog
        .get("npc_def_list")
        .filter(|d| index < d.size / NPC_DEFAULT_SIZE)
        .and_then(|d| {
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

/// An entity drawn as the `npc_draw_data_tbl` entry at this index
/// (special NPCs follow the villagers: `ALL_NPC_NUM` + id - `SP_NPC_START`).
#[derive(Component, Clone, Copy)]
pub struct NpcModel(pub u32);

/// Tom Nook in the first shop (`SP_NPC_SHOP_MASTER`, `cKF_bs_r_rcn_1`).
pub const NOOK: u32 = 238 + 8;

/// Gives special NPCs (Nook...) their disc model.
#[allow(clippy::too_many_arguments)]
fn spawn_npc_models(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    npcs: Query<(Entity, &NpcModel), Without<CharacterModelDone>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
    mut bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    let Some(disc) = disc else { return };
    for (entity, npc) in &npcs {
        commands.entity(entity).insert(CharacterModelDone);
        let Some(src) = villager_source(&disc, npc.0) else {
            warn!("no disc model for NPC {}", npc.0);
            continue;
        };
        let assets = (&mut *meshes, &mut *materials, &mut *images, &mut *bindposes);
        let parts = spawn_rig(
            &mut commands,
            &disc.rom,
            src,
            entity,
            format!("npc {} model", npc.0),
            assets,
        );
        if parts > 0 {
            commands
                .entity(entity)
                .remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>)>();
        }
    }
}

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
        let Some(src) = villager.npc_index.and_then(|i| villager_source(&disc, i)) else {
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
) -> u32 {
    if motion != rig.playing && rig.animations[motion as usize].is_some() {
        rig.playing = motion;
        rig.frame = 1.0;
    }
    let Some(anim) = rig.animations[rig.playing as usize].clone() else {
        return 0;
    };
    // Repeat between frame 1 and the last (`cKF_FrameControl_repeat_proc`);
    // a greeting plays once and holds its last frame (stop mode).
    let last = anim.frames.max(2) as f32;
    let contacts = if matches!(
        rig.playing,
        Motion::Walk | Motion::Run | Motion::WalkJoy | Motion::WalkAnger
    ) {
        rustimal_logic::footsteps::contacts(rig.frame, speed * ticks, last)
    } else {
        0
    };
    rig.frame += speed * ticks;
    if rig.playing.once() {
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
    contacts
}

/// Steps an `NpcClip`, loading it from the disc when it changes; false
/// when there is none (or it is not on the disc), to play standing instead.
fn play_clip(
    rig: &mut CharacterRig,
    clip: &mut NpcClip,
    disc: Option<&GameDisc>,
    ticks: f32,
    transforms: &mut Query<&mut Transform>,
) -> bool {
    let current = rig.clip.as_ref().map(|(n, _, _)| *n);
    if clip.name != current {
        rig.clip = clip.name.and_then(|name| {
            let a = disc?.catalog.get(name)?;
            let anim = Animation::read(&disc?.rom, a.offset, rig.skeleton.joints.len())?;
            let eyes = ClipEyes::read(disc?, a.offset, anim.frames.max(1) as usize);
            Some((name, Arc::new(anim), eyes))
        });
        rig.frame = 1.0;
        // Back to standing restarts its loop too.
        rig.playing = Motion::Wait;
        rig.pose_morph = clip.morph;
    }
    let Some((_, anim, _)) = rig.clip.clone() else {
        return false;
    };
    let last = anim.frames.max(2) as f32;
    rig.frame += WAIT_SPEED * ticks;
    if clip.once {
        if rig.frame >= last {
            rig.frame = last;
            clip.done = true;
        }
    } else {
        while rig.frame >= last {
            rig.frame -= last - 1.0;
            clip.loops += 1;
        }
    }
    let locals = anim.pose(rig.frame).local_transforms(&rig.skeleton);
    for (&joint, local) in rig.joints.iter().zip(locals) {
        if let Ok(mut t) = transforms.get_mut(joint) {
            *t = local;
        }
    }
    true
}

/// World units per second above which a villager walks / runs (between
/// their wander speeds, see `npc.rs`).
const VILLAGER_WALK_MIN: f32 = 0.3;
const VILLAGER_RUN_MIN: f32 = 2.5;

/// Plays each rig's animation: the player's from its locomotion (turning
/// the model to its facing), villagers' from how fast they move.
#[allow(clippy::too_many_arguments)] // Keep foot contacts tied to the same pose and movement snapshot.
fn animate_characters(
    time: Res<Time>,
    mut players: Query<
        (&Locomotion, &Facing, Option<&mut NpcClip>),
        (With<Player>, Without<NpcModel>),
    >,
    villagers: Query<(&GlobalTransform, &Villager, Option<&VillagerTalk>)>,
    mut npcs: Query<
        (
            Option<&crate::shop_nook::Shopkeeper>,
            Option<&mut NpcClip>,
            Option<&NpcGait>,
        ),
        (With<NpcModel>, Without<Player>),
    >,
    disc: Option<Res<GameDisc>>,
    mut rigs: Query<(Entity, &ChildOf, &mut CharacterRig)>,
    mut transforms: Query<&mut Transform>,
    focus: Res<crate::input::InputFocus>,
    mut footsteps: MessageWriter<crate::footsteps::Footstep>,
    net: Res<crate::player_tools::NetAction>,
    net_models: Option<Res<crate::player_tools::NetModels>>,
    room: Res<crate::player_room::PlayerRoom>,
    toys: Res<crate::player_toys::ToyAction>,
    preview: Res<crate::shop_nook::preview::Preview>,
) {
    let dt = time.delta_secs();
    let ticks = dt * 60.0;
    for (root, child_of, mut rig) in &mut rigs {
        let owner = child_of.parent();
        if let Ok((loco, facing, mut arm)) = players.get_mut(owner) {
            // Facing angle as the actor's shape rotation: 0 = +z, 90 deg = +x.
            if let Ok(mut t) = transforms.get_mut(root) {
                if preview.shirt() {
                    let (yaw, _, _) = t.rotation.to_euler(EulerRot::YXZ);
                    let delta = (-yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                        - std::f32::consts::PI;
                    let fraction = 1.0 - 0.5_f32.sqrt().powf(ticks);
                    let angle_unit = std::f32::consts::TAU / 65536.0;
                    let step = (delta * fraction)
                        .abs()
                        .clamp(50.0 * angle_unit * ticks, 2500.0 * angle_unit * ticks)
                        .min(delta.abs());
                    t.rotation = Quat::from_rotation_y(yaw + step * delta.signum());
                } else {
                    t.rotation = Quat::from_rotation_y(facing.dir.x.atan2(facing.dir.z));
                }
            }
            let furniture = room.animation();
            if let Some((animation, frame)) = preview.animation() {
                let locals = animation.pose(frame).local_transforms(&rig.skeleton);
                let weight = ((frame - 1.0) / 10.0).clamp(0.0, 1.0);
                for (&joint, local) in rig.joints.iter().zip(locals) {
                    if let Ok(mut t) = transforms.get_mut(joint) {
                        t.translation = t.translation.lerp(local.translation, weight);
                        t.rotation = t.rotation.slerp(local.rotation, weight);
                    }
                }
                rig.last_pos = transforms.get(owner).ok().map(|t| t.translation);
                rig.pose_morph = 5.0;
                continue;
            }
            if rig.furniture_phase != furniture.map(|(_, _, phase)| phase) {
                rig.furniture_phase = furniture.map(|(_, _, phase)| phase);
                rig.pose_morph = 5.0;
            }
            if let Some((animation, frame, _)) = furniture {
                let mut pose = animation.pose(frame);
                // Actor translation belongs to the collision-checked room
                // transaction; do not apply clip root motion a second time.
                let start = animation.pose(1.0).translation;
                pose.translation.x = start.x;
                pose.translation.z = start.z;
                let locals = pose.local_transforms(&rig.skeleton);
                let weight = if rig.pose_morph > 0.0 {
                    (ticks * 0.5 / rig.pose_morph).min(1.0)
                } else {
                    1.0
                };
                for (&joint, local) in rig.joints.iter().zip(locals) {
                    if let Ok(mut t) = transforms.get_mut(joint) {
                        t.translation = t.translation.lerp(local.translation, weight);
                        t.rotation = t.rotation.slerp(local.rotation, weight);
                    }
                }
                rig.pose_morph = (rig.pose_morph - ticks * 0.5).max(0.0);
                rig.last_pos = transforms.get(owner).ok().map(|t| t.translation);
                continue;
            }
            // A hand-over clip (give, reach, put away) replaces the locomotion pose.
            if let Some(arm) = arm.as_deref_mut().filter(|arm| arm.name.is_some()) {
                let previous = (rig.pose_morph > 0.0).then(|| {
                    rig.joints
                        .iter()
                        .map(|&joint| transforms.get(joint).copied().unwrap_or_default())
                        .collect::<Vec<_>>()
                });
                if play_clip(&mut rig, arm, disc.as_deref(), ticks, &mut transforms) {
                    if let Some(previous) = previous {
                        let weight = (ticks / rig.pose_morph).min(1.0);
                        for (&joint, old) in rig.joints.iter().zip(previous) {
                            if let Ok(mut t) = transforms.get_mut(joint) {
                                t.translation = old.translation.lerp(t.translation, weight);
                                t.rotation = old.rotation.slerp(t.rotation, weight);
                            }
                        }
                        rig.pose_morph = (rig.pose_morph - ticks).max(0.0);
                    }
                    rig.last_pos = transforms.get(owner).ok().map(|t| t.translation);
                    continue;
                }
            }
            let action = net_models.as_ref().filter(|_| net.busy());
            if rig.net_active != action.is_some() || rig.net_held != net.held {
                rig.net_active = action.is_some();
                rig.net_held = net.held;
                rig.pose_morph = 5.0; // source morph_counter=-5, advanced0.5/tick
            }
            if let Some(models) = action {
                let locals = models
                    .player_swing
                    .pose(net.player_frame)
                    .local_transforms(&rig.skeleton);
                let weight = if rig.pose_morph > 0.0 {
                    (ticks * 0.5 / rig.pose_morph).min(1.0)
                } else {
                    1.0
                };
                for (&joint, local) in rig.joints.iter().zip(locals) {
                    if let Ok(mut t) = transforms.get_mut(joint) {
                        t.translation = t.translation.lerp(local.translation, weight);
                        t.rotation = t.rotation.slerp(local.rotation, weight);
                    }
                }
                rig.pose_morph = (rig.pose_morph - ticks * 0.5).max(0.0);
                rig.last_pos = transforms.get(owner).ok().map(|t| t.translation);
                continue;
            }
            if rig.toy_phase != toys.phase() {
                rig.toy_phase = toys.phase();
                rig.pose_morph = 5.0;
            }
            let previous_pose = (rig.pose_morph > 0.0).then(|| {
                rig.joints
                    .iter()
                    .map(|&joint| transforms.get(joint).copied().unwrap_or_default())
                    .collect::<Vec<_>>()
            });
            let motion = Motion::of(loco.state());
            let speed = if motion == Motion::Wait {
                WAIT_SPEED
            } else {
                loco.anim_speed()
            };
            let position = transforms.get(owner).ok().map(|t| t.translation);
            let moved = position
                .zip(rig.last_pos)
                .map_or(0.0, |(p, previous)| (p - previous).with_y(0.0).length());
            rig.last_pos = position;
            let contacts = play(&mut rig, motion, speed, ticks, &mut transforms);
            if let Some(models) = net_models.as_ref().filter(|_| net.held) {
                let hold = models.player_hold.pose(1.0).local_transforms(&rig.skeleton);
                for ((&joint, local), &mask) in rig.joints.iter().zip(hold).zip(&models.hold_mask) {
                    if mask == 1 {
                        if let Ok(mut t) = transforms.get_mut(joint) {
                            t.rotation = local.rotation;
                        }
                    }
                }
            }
            if let Some((clip, frame, mask)) = toys.pose() {
                let locals = clip.pose(frame).local_transforms(&rig.skeleton);
                for ((&joint, local), &part) in rig.joints.iter().zip(locals).zip(mask) {
                    if part == 0 {
                        if let Ok(mut t) = transforms.get_mut(joint) {
                            t.rotation = local.rotation;
                        }
                    }
                }
            }
            if let Some(previous) = previous_pose {
                let weight = (ticks * 0.5 / rig.pose_morph).min(1.0);
                for (&joint, old) in rig.joints.iter().zip(previous) {
                    if let Ok(mut t) = transforms.get_mut(joint) {
                        t.translation = old.translation.lerp(t.translation, weight);
                        t.rotation = old.rotation.slerp(t.rotation, weight);
                    }
                }
                rig.pose_morph = (rig.pose_morph - ticks * 0.5).max(0.0);
            }
            // Collision, menus and warps must not turn a stale locomotion
            // state into audible steps. Contact timing still follows the rig.
            let moving = dt > 0.0
                && moved > 0.0001
                && moved <= crate::locomotion::FULL_TILT_SPEED * dt * 1.5;
            if focus.is_world() && moving && dt <= 0.1 {
                if let Some(position) = position {
                    for _ in 0..contacts {
                        footsteps.write(crate::footsteps::Footstep {
                            position,
                            running: motion == Motion::Run,
                            npc: false,
                        });
                    }
                }
            }
        } else if let Ok((keeper, clip, gait)) = npcs.get_mut(owner) {
            let previous = (rig.pose_morph > 0.0).then(|| {
                rig.joints
                    .iter()
                    .map(|&joint| transforms.get(joint).copied().unwrap_or_default())
                    .collect::<Vec<_>>()
            });
            let played = clip.is_some_and(|mut clip| {
                play_clip(&mut rig, &mut clip, disc.as_deref(), ticks, &mut transforms)
            });
            if !played {
                let (motion, speed) = match gait.map(|g| g.0) {
                    Some(2) => (Motion::Run, 1.0),
                    Some(1) => (Motion::Walk, WAIT_SPEED),
                    _ if keeper.is_some_and(|n| n.running) => (Motion::Run, 1.0),
                    _ if keeper.is_some_and(|n| n.walking) => (Motion::Walk, WAIT_SPEED),
                    _ => (Motion::Wait, WAIT_SPEED),
                };
                play(&mut rig, motion, speed, ticks, &mut transforms);
            }
            if let Some(previous) = previous {
                let weight = (ticks / rig.pose_morph).min(1.0);
                for (&joint, old) in rig.joints.iter().zip(previous) {
                    if let Ok(mut t) = transforms.get_mut(joint) {
                        t.translation = old.translation.lerp(t.translation, weight);
                        t.rotation = old.rotation.slerp(t.rotation, weight);
                    }
                }
                rig.pose_morph = (rig.pose_morph - ticks).max(0.0);
            }
        } else if let Ok((g, villager, talk)) = villagers.get(owner) {
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
            let feel = match talk {
                Some(t) if t.annoyed(villager.looks) => Feel::Anger,
                Some(t) if t.happy => Feel::Joy,
                _ => Feel::Normal,
            };
            let contacts = play(
                &mut rig,
                motion.with_feel(feel),
                WAIT_SPEED,
                ticks,
                &mut transforms,
            );
            // npc_1_walk1/run1/walk_ki1/walk_do1 all contact at frames 1/9.
            // Real displacement rejects stale walk poses, collisions and warps.
            if focus.is_world() && dt > 0.0 && dt <= 0.1 && moved > 0.0001 && moved <= 12.0 * dt {
                for _ in 0..contacts {
                    footsteps.write(crate::footsteps::Footstep {
                        position: pos,
                        running: motion == Motion::Run,
                        npc: true,
                    });
                }
            }
        }
    }
}

pub struct CharacterPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CharacterAnimationSet;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_player_model,
                spawn_villager_models,
                spawn_npc_models,
                animate_characters
                    .after(crate::shop_nook::interact)
                    .after(crate::locomotion::MovementSet)
                    .in_set(CharacterAnimationSet),
            )
                .chain(),
        );
    }
}
