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
