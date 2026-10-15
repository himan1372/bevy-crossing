//! The houses' mailboxes (`ac_mailbox.c`, `ac_mailbox_move.c_inc`).
//!
//! `cKF_bs_r_obj_s/w_post` on the layout's `MAILBOX0..3` units, turned
//! like their houses (even 90 deg, odd 0). Two animation tracks share the
//! skeleton (`cKF_SkeletonInfo_R_combine_play`, `part_tbl`): joints 2
//! and 3 (the flag) follow the flag track, the rest the body's rest pose.
//! The flag (`aMBX_check_flag`) rises when there's mail (`flag_on1`
//! 1-17), then waves (`flag_on_wait1` 1-31, looping), and falls when the
//! mailbox is empty (`flag_off1` 1-17), all at 0.5 frames a tick; the
//! winter skeleton uses the summer animations.
//!
//! The flag tip (joint 3) is drawn after `post_flag_saki_model_type0`
//! (`flag2_TA` in segment 8) or, while waving, `_type1` with the
//! `flag_on_wait1` EVW texture animation in segment 9: a 30 fps pattern
//! over two textures (`evw_anime_texanime`).

use bevy::prelude::*;
use disc::{RomSource, Target};

use crate::disc_gfx::{Part, Segments, TriangleSink, LIST_TAG};
use crate::game_disc::GameDisc;
use crate::n64_material::N64Material;
use crate::skeleton::{Animation, Skeleton};
use crate::town::Town;

type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;

/// The flag joints: `part_tbl` = { 0, 0, 0, 1, 1, 0, 0 } is read as
/// `part_table[joint + 1]` (`cKF_SkeletonInfo_R_combine_rotation`), so
/// joints 2 (the arm, `flag_ne`) and 3 (the tip, `flag_saki`).
const FLAG_JOINTS: [usize; 2] = [2, 3];
/// The flag tip, drawn after its type DL (`aMBX_actor_draw_before`).
const TIP_JOINT: usize = 3;
/// `aMBX_setup_flag_anime` frame speed.
const FLAG_SPEED: f32 = 0.5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Flag {
    Up,
    UpWait,
    Down,
}

/// One flag animation: frames `start..=end` of `anim`.
struct FlagAnim {
    anim: Animation,
    start: f32,
    end: f32,
}

/// What every mailbox shares.
#[derive(Resource)]
struct MailboxModels {
    skeleton: Skeleton,
    /// The body track's locals (rest pose, frame 1 of `cKF_ba_r_obj_s_post`).
    body: Vec<Transform>,
    up: FlagAnim,
    wait: FlagAnim,
    down: FlagAnim,
    /// Tip: down/rising, then the EVW textures.
    tip_still: Model,
    tip_wave: Vec<Model>,
    /// `animation_pattern` (texture per 30 fps frame).
    pattern: Vec<u8>,
}

/// A mailbox's flag state.
#[derive(Component)]
pub struct MailboxFlag {
    house: usize,
    flag: Flag,
    frame: f32,
    /// Joint index -> its part entity.
    joints: Vec<(usize, Entity)>,
    tip_parts: Vec<Entity>,
}

fn rel_ptr(disc: &GameDisc, at: u32) -> Option<u32> {
    match disc.rom.pointer_at(RomSource::Rel, at) {
        Some(Target::File(RomSource::Rel, p)) => Some(p),
        _ => None,
    }
}

fn bytes(disc: &GameDisc, at: u32, len: u32) -> Option<std::sync::Arc<[u8]>> {
    disc.rom.slice(RomSource::Rel, at, len).map(Into::into)
}

/// Splits a sink by the list that loaded each triangle (`LIST_TAG | i`),
/// keeping vertices in their joint's own space.
fn split_by_list(sink: &TriangleSink, lists: usize) -> Vec<TriangleSink> {
    let mut out: Vec<TriangleSink> = (0..lists).map(|_| TriangleSink::default()).collect();
    for part in &sink.parts {
        for tri in part.vertices.chunks(3) {
            let v = &tri[0];
            if v.mtx & 0xFF00_0000 != LIST_TAG {
                continue;
            }
            let Some(dst) = out.get_mut((v.mtx & 0x00FF_FFFF) as usize) else {
                continue;
            };
            match dst.parts.iter_mut().find(|p| p.key == part.key) {
                Some(p) => p.vertices.extend_from_slice(tri),
                None => dst.parts.push(Part {
                    key: part.key,
                    vertices: tri.to_vec(),
                }),
            }
        }
    }
    out
}

/// Runs the mailbox lists in joint order with the tip's type DL before
/// joint 3's list; returns each drawing joint's sink.
fn run_post(disc: &GameDisc, skel: &Skeleton, tip_dl: u32) -> Vec<(usize, TriangleSink)> {
    let joints: Vec<usize> = skel.matrix_joints();
    let mut entries = Vec::new();
    let mut owner = Vec::new();
    for &j in &joints {
        if j == TIP_JOINT {
            entries.push(tip_dl);
            owner.push(None);
        }
        if let Some(list) = skel.joints[j].model {
            entries.push(list);
            owner.push(Some(j));
        }
    }
    let sink = crate::disc_gfx::run_display_lists_with_colors(
        &disc.rom,
        &entries,
        [255; 4],
        255,
        [0; 4],
        &[],
    );
    split_by_list(&sink, entries.len())
        .into_iter()
        .zip(owner)
        .filter_map(|(s, j)| j.map(|j| (j, s)))
        .collect()
}

fn load(
    disc: &GameDisc,
    winter: bool,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> Option<(MailboxModels, Vec<(usize, Model)>)> {
    let cat = &disc.catalog;
    let season = if winter { "w" } else { "s" };
    let skel = Skeleton::read(
        &disc.rom,
        cat.get(&format!("cKF_bs_r_obj_{season}_post"))?.offset,
    )?;
    let n = skel.joints.len();
    let anim = |name: &str| Animation::read(&disc.rom, cat.get(name)?.offset, n);
    let body = anim("cKF_ba_r_obj_s_post")?
        .pose(1.0)
        .local_transforms(&skel);
    let flag = |name: &str| -> Option<FlagAnim> {
        Some(FlagAnim {
            anim: anim(name)?,
            start: 1.0,
            end: if name.ends_with("wait1") { 31.0 } else { 17.0 },
        })
    };
    let palette = cat.get(&format!("obj_{season}_post_pal")).map(|a| a.offset);
    let type0 = cat.get("post_flag_saki_model_type0")?.offset;
    let type1 = cat.get("post_flag_saki_model_type1")?.offset;
    let mut build = |sink: &TriangleSink, segs: &Segments| {
        crate::disc_town::sink_meshes(
            disc,
            sink,
            palette,
            false,
            segs,
            (&mut *meshes, &mut *materials, &mut *images),
        )
    };

    // Still tip: flag2_TA in segment 8 (ANIME_1).
    let flag2 = cat.get(&format!("obj_{season}_post_flag2_TA_tex_txt"))?;
    let mut still = Segments::default();
    still.set_segment(0x08, bytes(disc, flag2.offset, flag2.size)?);
    let parts = run_post(disc, &skel, type0);
    let mut joint_models = Vec::new();
    let mut tip_still = Vec::new();
    for (j, sink) in &parts {
        let model = build(sink, &still);
        if *j == TIP_JOINT {
            tip_still = model;
        } else {
            joint_models.push((*j, model));
        }
    }

    // Waving tip: EVW_ANIME_DATA -> EVW_ANIME_TEXANIME.
    let evw = cat
        .get(&format!("obj_{season}_post_flag_on_wait1_evw_anime"))?
        .offset;
    let seg = disc.rom.slice(RomSource::Rel, evw, 1)?[0] as i8;
    let tex = rel_ptr(disc, evw + 4)?;
    let head = disc.rom.slice(RomSource::Rel, tex, 2)?;
    let frames = u16::from_be_bytes([head[0], head[1]]) as u32;
    let table = rel_ptr(disc, tex + 4)?;
    let pattern = disc
        .rom
        .slice(RomSource::Rel, rel_ptr(disc, tex + 8)?, frames.max(1))?
        .to_vec();
    let textures = pattern.iter().copied().max().unwrap_or(0) as u32 + 1;
    let mut tip_wave = Vec::new();
    for k in 0..textures {
        let Some(t) = rel_ptr(disc, table + k * 4) else {
            break;
        };
        let size = cat.containing(RomSource::Rel, t).map_or(0x100, |a| a.size);
        let mut segs = Segments::default();
        segs.set_segment(7 + seg.unsigned_abs(), bytes(disc, t, size)?);
        let sink = run_post(disc, &skel, type1)
            .into_iter()
            .find(|(j, _)| *j == TIP_JOINT)
            .map(|(_, s)| s)?;
        tip_wave.push(build(&sink, &segs));
    }

    Some((
        MailboxModels {
            body,
            up: flag("cKF_ba_r_obj_s_post_flag_on1")?,
            wait: flag("cKF_ba_r_obj_s_post_flag_on_wait1")?,
            down: flag("cKF_ba_r_obj_s_post_flag_off1")?,
            skeleton: skel,
            tip_still,
            tip_wave,
            pattern,
        },
        joint_models,
    ))
}

/// Model units to scene units (`OBJECT_SCALE`), as a joint's transform:
/// meshes are built pre-scaled, so only the translation scales.
fn joint_transform(m: Mat4) -> Transform {
    let (s, r, t) = m.to_scale_rotation_translation();
    Transform {
        translation: t * crate::disc_town::OBJECT_SCALE,
        rotation: r,
        scale: s,
    }
}

impl MailboxModels {
    /// Joint matrices with the flag joints at `frame` of `flag`.
    fn pose(&self, flag: Flag, frame: f32) -> Vec<Mat4> {
        let anim = match flag {
            Flag::Up => &self.up,
            Flag::UpWait => &self.wait,
            Flag::Down => &self.down,
        };
        let flag_locals = anim.anim.pose(frame).local_transforms(&self.skeleton);
        let mut locals = self.body.clone();
        for j in FLAG_JOINTS {
            if let (Some(l), Some(f)) = (locals.get_mut(j), flag_locals.get(j)) {
                *l = *f;
            }
        }
        self.skeleton.world_from_locals(&locals)
    }
}

/// Spawns the mailboxes with their flags down (or waving when `mail`).
pub fn spawn_mailboxes(
    commands: &mut Commands,
    disc: &GameDisc,
    town: &Town,
    winter: bool,
    assets: (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) {
    let Some(retail) = &town.retail else { return };
    let Some((models, joint_models)) = load(disc, winter, assets) else {
        warn!("mailbox model not found on the disc");
        return;
    };
    for (house, cell) in retail.mailboxes.iter().enumerate() {
        let Some(&cell) = cell.as_ref() else { continue };
        let at = crate::disc_town::mailbox_position(cell);
        let yaw = if house % 2 == 0 {
            std::f32::consts::FRAC_PI_2
        } else {
            0.0
        };
        let root = commands
            .spawn((
                crate::disc_town::DiscObject,
                Name::new(format!("mailbox {house}")),
                Transform::from_xyz(at.x, town.ground_y(at), at.z)
                    .with_rotation(Quat::from_rotation_y(yaw)),
                Visibility::default(),
            ))
            .id();
        let world = models.pose(Flag::Down, models.down.end);
        let mut joints = Vec::new();
        let mut spawn_joint = |commands: &mut Commands, j: usize, model: &Model| {
            let joint = commands
                .spawn((
                    joint_transform(world[j]),
                    Visibility::default(),
                    ChildOf(root),
                ))
                .id();
            let parts: Vec<Entity> = model
                .iter()
                .map(|(mesh, material)| {
                    commands
                        .spawn((
                            Mesh3d(mesh.clone()),
                            MeshMaterial3d(material.clone()),
                            ChildOf(joint),
                        ))
                        .id()
                })
                .collect();
            joints.push((j, joint));
            parts
        };
        for (j, model) in &joint_models {
            spawn_joint(commands, *j, model);
        }
        let tip_parts = spawn_joint(commands, TIP_JOINT, &models.tip_still);
        commands.entity(root).insert(MailboxFlag {
            house,
            // aMBX_actor_ct: anim_idx1 starts at NUM, so check_flag goes
            // straight to UP_WAIT with mail, else plays FLAG_DOWN.
            flag: Flag::Down,
            frame: models.down.end,
            joints,
            tip_parts,
        });
    }
    commands.insert_resource(models);
}

/// `aMBX_check_flag` + the flag track playing.
fn animate_flags(
    time: Res<Time>,
    models: Res<MailboxModels>,
    mail: Res<crate::mail::Mailbox>,
    mut flags: Query<(&GlobalTransform, &mut MailboxFlag)>,
    mut transforms: Query<&mut Transform>,
    mut parts: Query<(&mut Mesh3d, &mut MeshMaterial3d<N64Material>)>,
    mut first: Local<bool>,
) {
    let ticks = time.delta_secs() * 60.0;
    // Only the player's house (0) gets letters here; unread ones stand in
    // for the mail the retail mailbox holds.
    let has_mail = mail.letters.iter().any(|l| !l.read);
    for (position, mut f) in &mut flags {
        let mail_here = f.house == 0 && has_mail;
        let want = match (mail_here, f.flag) {
            (false, _) => Flag::Down,
            (true, Flag::UpWait) => Flag::UpWait,
            (true, _) if !*first => Flag::UpWait,
            (true, Flag::Up) => Flag::Up,
            (true, Flag::Down) => Flag::Up,
        };
        if want != f.flag {
            debug!("mailbox {} flag {:?} -> {want:?}", f.house, f.flag);
            f.flag = want;
            f.frame = 1.0;
        }
        let anim = match f.flag {
            Flag::Up => &models.up,
            Flag::UpWait => &models.wait,
            Flag::Down => &models.down,
        };
        let previous_frame = f.frame;
        f.frame += FLAG_SPEED * ticks;
        // aMBX_setup_flag_se: one sound crossing frame 15, not per frame.
        if previous_frame < 15.0 && f.frame >= 15.0 {
            let id = match f.flag {
                Flag::Up => Some(crate::sound::se_id::MAILBOX_FLAG_UP),
                Flag::Down => Some(crate::sound::se_id::MAILBOX_FLAG_DOWN),
                Flag::UpWait => None,
            };
            if let Some(id) = id {
                crate::sound::se_at(id, position.translation());
            }
        }
        if f.frame >= anim.end {
            match f.flag {
                Flag::UpWait => {
                    f.frame = anim.start + (f.frame - anim.start) % (anim.end - anim.start)
                }
                Flag::Up => {
                    f.flag = Flag::UpWait;
                    f.frame = 1.0;
                }
                Flag::Down => f.frame = anim.end,
            }
        }
        let world = models.pose(f.flag, f.frame);
        for &(j, e) in &f.joints {
            if FLAG_JOINTS.contains(&j) {
                if let Ok(mut t) = transforms.get_mut(e) {
                    *t = joint_transform(world[j]);
                }
            }
        }
        // The tip's texture: still, or the 30 fps EVW pattern.
        let tip = if f.flag == Flag::UpWait && !models.pattern.is_empty() {
            let phase = (time.elapsed_secs() * 30.0) as usize % models.pattern.len();
            models
                .tip_wave
                .get(models.pattern[phase] as usize)
                .unwrap_or(&models.tip_still)
        } else {
            &models.tip_still
        };
        for (e, (mesh, material)) in f.tip_parts.iter().zip(tip) {
            if let Ok((mut m, mut mat)) = parts.get_mut(*e) {
                if m.0 != *mesh {
                    m.0 = mesh.clone();
                    mat.0 = material.clone();
                }
            }
        }
    }
    *first = true;
}

pub struct MailboxPlugin;

impl Plugin for MailboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            animate_flags.run_if(
                in_state(crate::states::AppState::InGame)
                    .and_then(resource_exists::<MailboxModels>)
                    .and_then(resource_exists::<crate::mail::Mailbox>),
            ),
        );
    }
}
