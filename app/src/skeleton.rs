//! Skeletons and keyframe animations (`c_keyframe.c`), read from the disc.
//!
//! A skeleton (`cKF_Skeleton_R_c`) is a depth-first joint list; each joint
//! has an optional display list, a child count, flags and a translation.
//! An animation (`cKF_Animation_R_c`) gives, per frame, the root
//! translation and every joint's rotation: each value is either fixed or
//! Hermite-interpolated between keys (`cKF_KeyCalc`). Rotations are stored
//! in tenths of a degree.
//!
//! Drawing (`cKF_Si3_draw_SV_R_child`) multiplies each joint's matrix as
//! translate, then rotate Z, Y, X (`Matrix_softcv3_mult`); joints with a
//! display list get the next matrix in segment D, which skinned meshes
//! load (`gsSPMatrix`) to place vertices on other joints.

use bevy::prelude::*;
use disc::{RomImage, RomSource, Target};

/// `cKF_ANIMATION_BIT_*` per joint in the flag table: translation x/y/z
/// (root only), then rotation x/y/z.
const BIT_TRANS_X: u8 = 1 << 5;
const BIT_ROT_X: u8 = 1 << 2;

/// Size of one `cKF_Joint_R_c`.
const JOINT_SIZE: u32 = 12;

/// One joint of a skeleton.
#[derive(Clone, Debug, PartialEq)]
pub struct Joint {
    /// REL offset of the joint's display list, if it draws anything.
    pub model: Option<u32>,
    pub children: u8,
    /// `cKF_JOINT_FLAG_DISP_XLU` (bit 0): drawn translucent.
    pub flags: u8,
    pub translation: [i16; 3],
}

/// A `cKF_Skeleton_R_c`.
#[derive(Clone, Debug)]
pub struct Skeleton {
    pub joints: Vec<Joint>,
    /// Parent of each joint (`None` for the root), from the child counts.
    pub parents: Vec<Option<usize>>,
}

/// REL offset a relocated pointer at `offset` points to.
fn rel_pointer(rom: &RomImage, offset: u32) -> Option<u32> {
    match rom.pointer_at(RomSource::Rel, offset)? {
        Target::File(RomSource::Rel, p) => Some(p),
        _ => None,
    }
}

fn be16(b: &[u8], o: usize) -> i16 {
    i16::from_be_bytes([b[o], b[o + 1]])
}

impl Skeleton {
    /// Reads the skeleton at REL offset `offset`.
    pub fn read(rom: &RomImage, offset: u32) -> Option<Self> {
        let head = rom.slice(RomSource::Rel, offset, 8)?;
        let count = head[0] as u32;
        let table = rel_pointer(rom, offset + 4)?;
        let bytes = rom.slice(RomSource::Rel, table, count * JOINT_SIZE)?;
        let joints: Vec<Joint> = (0..count)
            .map(|i| {
                let b = &bytes[(i * JOINT_SIZE) as usize..];
                Joint {
                    model: rel_pointer(rom, table + i * JOINT_SIZE),
                    children: b[4],
                    flags: b[5],
                    translation: [be16(b, 6), be16(b, 8), be16(b, 10)],
                }
            })
            .collect();
        let parents = parents_of(&joints)?;
        Some(Self { joints, parents })
    }

    /// Joint index for each segment-D matrix slot: joints with a display
    /// list, in drawing (list) order.
    pub fn matrix_joints(&self) -> Vec<usize> {
        (0..self.joints.len())
            .filter(|&j| self.joints[j].model.is_some())
            .collect()
    }
}

/// Parents from the depth-first child counts; `None` if they don't add up.
fn parents_of(joints: &[Joint]) -> Option<Vec<Option<usize>>> {
    fn walk(
        joints: &[Joint],
        next: &mut usize,
        parent: Option<usize>,
        out: &mut Vec<Option<usize>>,
    ) -> Option<()> {
        let me = *next;
        joints.get(me)?;
        out.push(parent);
        *next += 1;
        for _ in 0..joints[me].children {
            walk(joints, next, Some(me), out)?;
        }
        Some(())
    }
    let mut out = Vec::with_capacity(joints.len());
    let mut next = 0;
    walk(joints, &mut next, None, &mut out)?;
    (out.len() == joints.len()).then_some(out)
}

/// A `cKF_Animation_R_c`, with its tables read.
#[derive(Clone, Debug)]
pub struct Animation {
    pub flags: Vec<u8>,
    pub data: Vec<i16>,
    pub keys: Vec<i16>,
    pub fixed: Vec<i16>,
    pub frames: i16,
}

fn read_i16s(rom: &RomImage, offset: Option<u32>, count: usize) -> Option<Vec<i16>> {
    if count == 0 {
        return Some(Vec::new());
    }
    let b = rom.slice(RomSource::Rel, offset?, (count * 2) as u32)?;
    Some((0..count).map(|i| be16(b, i * 2)).collect())
}

impl Animation {
    /// Reads the animation at REL offset `offset` for a skeleton of
    /// `joint_count` joints.
    pub fn read(rom: &RomImage, offset: u32, joint_count: usize) -> Option<Self> {
        let head = rom.slice(RomSource::Rel, offset, 20)?;
        let frames = be16(head, 18);
        let flags = rom
            .slice(
                RomSource::Rel,
                rel_pointer(rom, offset)?,
                joint_count as u32,
            )?
            .to_vec();
        // Animated components: the root's translation bits, then three
        // rotation bits per joint.
        let animated = (flags[0] & (BIT_TRANS_X | BIT_TRANS_X >> 1 | BIT_TRANS_X >> 2)).count_ones()
            as usize
            + flags
                .iter()
                .map(|f| (f & 0b111).count_ones() as usize)
                .sum::<usize>();
        let fixed_count = 3 + 3 * joint_count - animated;
        let keys = read_i16s(rom, rel_pointer(rom, offset + 8), animated)?;
        let key_total: usize = keys.iter().map(|&k| k.max(0) as usize).sum();
        let data = read_i16s(rom, rel_pointer(rom, offset + 4), key_total * 3)?;
        let fixed = read_i16s(rom, rel_pointer(rom, offset + 12), fixed_count)?;
        Some(Self {
            flags,
            data,
            keys,
            fixed,
            frames,
        })
    }

    /// The pose at `frame` (`cKF_SkeletonInfo_R_play`): root translation
    /// and each joint's rotation in radians (x, y, z).
    pub fn pose(&self, frame: f32) -> Pose {
        let mut key_i = 0;
        let mut fixed_i = 0;
        let mut data_i = 0usize;
        let mut value = |animated: bool| -> i16 {
            if animated {
                let n = self.keys.get(key_i).copied().unwrap_or(0);
                key_i += 1;
                let v = key_calc(&self.data[data_i.min(self.data.len())..], n, frame);
                data_i += n.max(0) as usize * 3;
                v
            } else {
                let v = self.fixed.get(fixed_i).copied().unwrap_or(0);
                fixed_i += 1;
                v
            }
        };
        let f0 = self.flags.first().copied().unwrap_or(0);
        let translation = Vec3::new(
            value(f0 & BIT_TRANS_X != 0) as f32,
            value(f0 & BIT_TRANS_X >> 1 != 0) as f32,
            value(f0 & BIT_TRANS_X >> 2 != 0) as f32,
        );
        let rotations = self
            .flags
            .iter()
            .map(|&f| {
                let mut r = [0.0; 3];
                for (c, bit) in [BIT_ROT_X, BIT_ROT_X >> 1, BIT_ROT_X >> 2]
                    .into_iter()
                    .enumerate()
                {
                    // Tenths of a degree, wrapped to an s16 binary angle.
                    let deg = (value(f & bit != 0) as f32 * 0.1).rem_euclid(360.0);
                    r[c] = deg_to_s16(deg) as f32 / 32768.0 * std::f32::consts::PI;
                }
                Vec3::from(r)
            })
            .collect();
        Pose {
            translation,
            rotations,
        }
    }
}

/// `DEG2SHORT_ANGLE`: degrees to a signed 16-bit binary angle.
fn deg_to_s16(deg: f32) -> i16 {
    (deg * (65536.0 / 360.0)) as i32 as i16
}

/// `cKF_HermitCalc`.
fn hermite(t: f32, tension: f32, p0: f32, p1: f32, m0: f32, m1: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    let h = -(t3 * 2.0) + 3.0 * t2;
    let h10 = t + (t3 - t2 * 2.0);
    let h11 = t3 - t2;
    (1.0 - h) * p0 + h * p1 + tension * (h10 * m0 + h11 * m1)
}

/// `cKF_KeyCalc`: `keys` holds `n` (frame, value, tangent) triples.
fn key_calc(keys: &[i16], n: i16, frame: f32) -> i16 {
    let n = n.max(0) as usize;
    let key = |i: usize| -> (f32, i16, f32) {
        let k = keys.get(i * 3..i * 3 + 3).unwrap_or(&[0, 0, 0]);
        (k[0] as f32, k[1], k[2] as f32)
    };
    if n == 0 {
        return 0;
    }
    if key(0).0 >= frame {
        return key(0).1;
    }
    if key(n - 1).0 <= frame {
        return key(n - 1).1;
    }
    for next in 1..n {
        let (f1, v1, m1) = key(next);
        if f1 > frame {
            let (f0, v0, m0) = key(next - 1);
            let span = f1 - f0;
            if span == 0.0 {
                return v0;
            }
            let t = (frame - f0) / span;
            let calc = hermite(t, span / 30.0, v0 as f32, v1 as f32, m0, m1);
            return (calc as f64 + 0.5) as i32 as i16;
        }
    }
    key(n - 1).1
}

/// A skeleton pose: root translation (model units) and per-joint rotation.
#[derive(Clone, Debug)]
pub struct Pose {
    pub translation: Vec3,
    pub rotations: Vec<Vec3>,
}

impl Pose {
    /// Each joint's local transform (`Matrix_softcv3_mult`: translate,
    /// then rotate Z, Y, X): the root uses the pose's translation, the
    /// others their skeleton translation.
    pub fn local_transforms(&self, skeleton: &Skeleton) -> Vec<Transform> {
        skeleton
            .joints
            .iter()
            .enumerate()
            .map(|(j, joint)| {
                let t = if j == 0 {
                    self.translation
                } else {
                    Vec3::from(joint.translation.map(|v| v as f32))
                };
                let r = self.rotations.get(j).copied().unwrap_or(Vec3::ZERO);
                Transform::from_translation(t).with_rotation(Quat::from_euler(
                    EulerRot::ZYX,
                    r.z,
                    r.y,
                    r.x,
                ))
            })
            .collect()
    }
}

impl Skeleton {
    /// Every joint's model-space matrix in `frame` of `anim` (the rest
    /// pose without one), for drawing the skeleton still.
    pub fn world_matrices(&self, anim: Option<&Animation>, frame: f32) -> Vec<Mat4> {
        let locals: Vec<Transform> = match anim {
            Some(a) => a.pose(frame).local_transforms(self),
            None => self
                .joints
                .iter()
                .map(|j| Transform::from_translation(Vec3::from(j.translation.map(f32::from))))
                .collect(),
        };
        self.world_from_locals(&locals)
    }

    /// Every joint's model-space matrix from its local transform (e.g.
    /// two animation tracks mixed per joint).
    pub fn world_from_locals(&self, locals: &[Transform]) -> Vec<Mat4> {
        let mut world: Vec<Mat4> = Vec::with_capacity(locals.len());
        for (j, local) in locals.iter().enumerate() {
            let parent = self.parents[j].map_or(Mat4::IDENTITY, |p| world[p]);
            world.push(parent * local.to_matrix());
        }
        world
    }

    /// Display lists of the drawing joints, in drawing order (the same
    /// order as [`Skeleton::matrix_joints`]).
    pub fn lists(&self) -> Vec<u32> {
        self.joints.iter().filter_map(|j| j.model).collect()
    }
}

/// Moves the vertices of a still skeleton's lists (run in
/// [`Skeleton::lists`] order) onto their joints: a vertex either keeps
/// its own list's joint (`LIST_TAG | i`) or names a segment-D matrix slot.
pub fn place_on_joints(
    sink: &mut crate::disc_gfx::TriangleSink,
    skeleton: &Skeleton,
    world: &[Mat4],
) {
    let matrix_joints = skeleton.matrix_joints();
    for v in sink.parts.iter_mut().flat_map(|p| p.vertices.iter_mut()) {
        let slot = match v.mtx >> 24 {
            t if t << 24 == crate::disc_gfx::LIST_TAG => v.mtx & 0x00FF_FFFF,
            0x0D => (v.mtx & 0x00FF_FFFF) / 0x40,
            _ => continue,
        };
        let Some(m) = matrix_joints.get(slot as usize).and_then(|&j| world.get(j)) else {
            continue;
        };
        v.pos = m.transform_point3(Vec3::from(v.pos)).to_array();
        v.normal = m
            .transform_vector3(Vec3::from(v.normal))
            .normalize_or_zero()
            .to_array();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn joint(children: u8) -> Joint {
        Joint {
            model: None,
            children,
            flags: 0,
            translation: [0; 3],
        }
    }

    #[test]
    fn parents_follow_child_counts() {
        // 0 -> (1 -> 2), 3
        let j = vec![joint(2), joint(1), joint(0), joint(0)];
        assert_eq!(
            parents_of(&j).unwrap(),
            vec![None, Some(0), Some(1), Some(0)]
        );
        assert!(parents_of(&[joint(2), joint(0)]).is_none());
    }

    #[test]
    fn key_calc_holds_ends_and_interpolates() {
        // Keys at frames 1 and 31: values 0 and 100, flat tangents.
        let keys = [1, 0, 0, 31, 100, 0];
        assert_eq!(key_calc(&keys, 2, 0.0), 0);
        assert_eq!(key_calc(&keys, 2, 40.0), 100);
        // Halfway: Hermite basis gives exactly the midpoint.
        assert_eq!(key_calc(&keys, 2, 16.0), 50);
    }

    #[test]
    fn fixed_pose_values() {
        // One joint, nothing animated: translation (1,2,3), rotation x 90 deg.
        let anim = Animation {
            flags: vec![0],
            data: vec![],
            keys: vec![],
            fixed: vec![1, 2, 3, 900, 0, 0],
            frames: 1,
        };
        let pose = anim.pose(1.0);
        assert_eq!(pose.translation, Vec3::new(1.0, 2.0, 3.0));
        assert!((pose.rotations[0].x - std::f32::consts::FRAC_PI_2).abs() < 1e-3);
    }

    /// Probe (needs the disc): room model bounds in cells.
    #[test]
    #[ignore]
    fn probe_room_bounds() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut d = disc::Disc::open_path(std::path::Path::new(&path)).unwrap();
        let rom = RomImage::from_disc(&mut d).unwrap();
        let cat = disc::AssetCatalog::builtin(&rom);
        for name in [
            "rom_myhome1_floor_model",
            "rom_myhome1_wall_model",
            "rom_shop1f_model",
            "rom_shop1w_model",
        ] {
            let Some(a) = cat.get(name) else { continue };
            let sink = crate::disc_gfx::run_display_lists(&rom, &[a.offset]);
            let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
            for v in sink.parts.iter().flat_map(|p| &p.vertices) {
                lo = lo.min(Vec3::from(v.pos));
                hi = hi.max(Vec3::from(v.pos));
            }
            let tex: Vec<String> = sink
                .parts
                .iter()
                .filter_map(|p| {
                    p.key
                        .tex
                        .map(|t| format!("{:x}/{:x}", t.img_addr, t.tlut_addr))
                })
                .collect();
            println!(
                "{name}: {} parts, cells {:?}..{:?}, tex {:?}",
                sink.parts.len(),
                lo * 0.0625 / 40.0,
                hi * 0.0625 / 40.0,
                tex
            );
        }
    }

    /// Probe (needs the disc): door centroid of posed structures.
    #[test]
    #[ignore]
    fn probe_posed_doors() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut d = disc::Disc::open_path(std::path::Path::new(&path)).unwrap();
        let rom = RomImage::from_disc(&mut d).unwrap();
        let cat = disc::AssetCatalog::builtin(&rom);
        for name in [
            "obj_s_myhome1",
            "obj_s_house3",
            "obj_s_shop1",
            "obj_s_tailor",
            "obj_s_yubinkyoku",
        ] {
            let skel =
                Skeleton::read(&rom, cat.get(&format!("cKF_bs_r_{name}")).unwrap().offset).unwrap();
            let anim = Animation::read(
                &rom,
                cat.get(&format!("cKF_ba_r_{name}")).unwrap().offset,
                skel.joints.len(),
            )
            .unwrap();
            let locals = anim.pose(1.0).local_transforms(&skel);
            let mut world: Vec<Mat4> = Vec::new();
            for (j, l) in locals.iter().enumerate() {
                let p = skel.parents[j].map_or(Mat4::IDENTITY, |p| world[p]);
                world.push(p * l.to_matrix());
            }
            let door = cat.get(&format!("{name}_door_model")).map(|a| a.offset);
            let (mut c, mut n, mut lo, mut hi) = (
                Vec3::ZERO,
                0.0f32,
                Vec3::splat(f32::MAX),
                Vec3::splat(f32::MIN),
            );
            for (j, joint) in skel.joints.iter().enumerate() {
                let Some(list) = joint.model else { continue };
                for v in crate::disc_gfx::run_display_lists(&rom, &[list])
                    .parts
                    .iter()
                    .flat_map(|p| p.vertices.clone())
                {
                    let w = world[j].transform_point3(Vec3::from(v.pos));
                    lo = lo.min(w);
                    hi = hi.max(w);
                    if Some(list) == door {
                        c += w;
                        n += 1.0;
                    }
                }
            }
            let pose = anim.pose(1.0);
            for (j, r) in pose.rotations.iter().enumerate() {
                if *r != Vec3::ZERO || skel.joints[j].translation != [0; 3] {
                    println!(
                        "   joint {j} rot(deg) {:?} t {:?}",
                        *r * (180.0 / std::f32::consts::PI),
                        skel.joints[j].translation
                    );
                }
            }
            println!(
                "   root translation {:?} anim frames {} flags {:?}",
                pose.translation, anim.frames, anim.flags
            );
            println!(
                "{name}: door centroid {:?} bounds {lo:?}..{hi:?} root rot {:?}",
                c / n.max(1.0),
                anim.pose(1.0).rotations[0]
            );
        }
    }

    /// Probe (needs the disc): a house skeleton's joints and their lists.
    #[test]
    #[ignore]
    fn probe_house_skeleton() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut d = disc::Disc::open_path(std::path::Path::new(&path)).unwrap();
        let rom = RomImage::from_disc(&mut d).unwrap();
        let cat = disc::AssetCatalog::builtin(&rom);
        for name in [
            "cKF_bs_r_obj_s_house3",
            "cKF_bs_r_obj_s_myhome1",
            "cKF_bs_r_obj_s_shop1",
        ] {
            let Some(a) = cat.get(name) else {
                println!("{name}: missing");
                continue;
            };
            let k = Skeleton::read(&rom, a.offset).unwrap();
            println!("{name}: {} joints", k.joints.len());
            for (i, j) in k.joints.iter().enumerate() {
                let m = j
                    .model
                    .and_then(|m| cat.containing(RomSource::Rel, m))
                    .map(|a| a.name.to_string());
                println!(
                    "  {i} parent {:?} t {:?} {:?}",
                    k.parents[i], j.translation, m
                );
                if let Some(list) = j.model {
                    let sink = crate::disc_gfx::run_display_lists(&rom, &[list]);
                    let mut mtx = std::collections::BTreeMap::<u32, usize>::new();
                    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
                    for v in sink.parts.iter().flat_map(|p| &p.vertices) {
                        *mtx.entry(v.mtx).or_default() += 1;
                        for c in 0..3 {
                            lo[c] = lo[c].min(v.pos[c]);
                            hi[c] = hi[c].max(v.pos[c]);
                        }
                    }
                    println!("     mtx {mtx:x?} bounds {lo:?}..{hi:?}");
                }
            }
        }
    }

    /// Probe (needs the disc): villager names and their draw-data entries.
    #[test]
    #[ignore]
    fn probe_villager_draw_data() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut d = disc::Disc::open_path(std::path::Path::new(&path)).unwrap();
        let names = disc::Rarc::parse(d.read_file("forest_2nd.arc").unwrap()).unwrap();
        let names = names.file("npc_name_str_table.bin").unwrap().to_vec();
        let rom = RomImage::from_disc(&mut d).unwrap();
        let cat = disc::AssetCatalog::builtin(&rom);
        let tbl = cat.get("npc_draw_data_tbl").unwrap().offset;
        let sym = |off: u32| {
            rel_pointer(&rom, off)
                .and_then(|p| {
                    cat.containing(RomSource::Rel, p)
                        .map(|a| a.name.to_string())
                })
                .unwrap_or_default()
        };
        for sk in [
            "cKF_bs_r_cat_1",
            "cKF_bs_r_cbr_1",
            "cKF_bs_r_wol_1",
            "cKF_bs_r_rbt_1",
        ] {
            let k = Skeleton::read(&rom, cat.get(sk).unwrap().offset).unwrap();
            println!(
                "{sk}: {} joints, {} drawn",
                k.joints.len(),
                k.matrix_joints().len()
            );
        }
        for i in 236..262u32 {
            let e = tbl + i * 0x6C;
            println!(
                "sp {i}: skel {} tex {} scale {:?}",
                sym(e + 4),
                sym(e + 8),
                rom.slice(RomSource::Rel, e + 0x54, 4)
                    .map(|b| f32::from_be_bytes([b[0], b[1], b[2], b[3]]))
            );
        }
        for i in 0..names.len() / 8 {
            let name: String = names[i * 8..i * 8 + 8]
                .iter()
                .take_while(|&&c| c != 0 && c != 0x20 || false)
                .map(|&c| c as char)
                .collect();
            let e = tbl + i as u32 * 0x6C;
            let b = rom.slice(RomSource::Rel, e, 0x6C).unwrap();
            let scale = f32::from_be_bytes([b[0x54], b[0x55], b[0x56], b[0x57]]);
            println!(
                "{i:3} {name:8} bank {} {} skel {} tex {} pal {} eye0 {} mouth0 {} scale {scale} eye_h {}",
                be16(b, 0), be16(b, 2), sym(e + 4), sym(e + 8), sym(e + 12), sym(e + 16), sym(e + 0x30), b[0x5E]
            );
        }
    }

    /// Probe (needs the disc): disc files and archive entries for the
    /// player's textures.
    #[test]
    #[ignore]
    fn probe_player_texture_files() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut d = disc::Disc::open_path(std::path::Path::new(&path)).unwrap();
        let names: Vec<String> = d.files().iter().map(|f| format!("{f:?}")).collect();
        for n in &names {
            println!("file {n}");
        }
        for arc in ["forest_1st.arc", "forest_2nd.arc"] {
            if let Ok(bytes) = d.read_file(arc) {
                if let Some(r) = disc::Rarc::parse(bytes) {
                    for f in r.files() {
                        println!("{arc}: {f:?}");
                    }
                }
            }
        }
    }

    /// Probe (needs the disc): the player skeleton and an animation.
    #[test]
    #[ignore]
    fn probe_player_skeleton() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut d = disc::Disc::open_path(std::path::Path::new(&path)).unwrap();
        let rom = RomImage::from_disc(&mut d).unwrap();
        let cat = disc::AssetCatalog::builtin(&rom);
        let skel = Skeleton::read(&rom, cat.get("cKF_bs_r_boy_1").unwrap().offset).unwrap();
        println!(
            "{} joints, {} with models",
            skel.joints.len(),
            skel.matrix_joints().len()
        );
        for (i, j) in skel.joints.iter().enumerate() {
            println!("{i:2} parent {:?} {:?}", skel.parents[i], j);
        }
        let anim = Animation::read(
            &rom,
            cat.get("cKF_ba_r_ply_1_wait1").unwrap().offset,
            skel.joints.len(),
        )
        .unwrap();
        println!(
            "wait1: {} frames, {} keys, {} fixed",
            anim.frames,
            anim.keys.len(),
            anim.fixed.len()
        );
        println!("{:?}", anim.pose(1.0));
        let head = skel.joints[24].model.unwrap();
        let words = rom.slice(RomSource::Rel, head, 8 * 14).unwrap();
        for c in words.chunks(8) {
            println!(
                "cmd {:02x} {:08x} {:08x}",
                c[0],
                u32::from_be_bytes([c[0], c[1], c[2], c[3]]),
                u32::from_be_bytes([c[4], c[5], c[6], c[7]])
            );
        }
        for &j in &skel.matrix_joints() {
            let list = skel.joints[j].model.unwrap();
            let sink = crate::disc_gfx::run_display_lists(&rom, &[list]);
            let mut mtx = std::collections::BTreeMap::<(u32, u8), usize>::new();
            for v in sink.parts.iter().flat_map(|p| &p.vertices) {
                *mtx.entry((v.mtx, v.mtx_flag)).or_default() += 1;
            }
            println!("joint {j}: {} parts, matrices {:x?}", sink.parts.len(), mtx);
        }
    }
}
