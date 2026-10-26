//! Furniture models from the disc (`ac_my_room`).
//!
//! Each furniture index has an `aFTR_PROFILE` in `furniture_quality`:
//! opaque and translucent display lists, an optional palette, a rig or a
//! texture animation, its scale and footprint shape. Furniture without
//! lists draws its `cKF_bs_r_int_*` skeleton from its own draw function
//! (`cKF_Si3_draw_R_SV`), standing still in the animation's first frame
//! (`cKF_SkeletonInfo_R_init_standard_stop`, speed 0). Audited common storage
//! doors retain their runtime joints for the player-room open/close demo.
//!
//! Placement follows `aMR_FurnitureCt`: the item's low two bits are the
//! rotation (0/90/180/270 degrees), the model sits on its unit's centre
//! (a 2x2 piece on the corner its four units share), and 1x2 pieces
//! reach into the neighbouring unit the rotation points at.

use std::sync::Arc;

use bevy::prelude::*;
use disc::{RomSource, Target};
use rustimal_logic::furniture::{FurnitureProfile, SetType, ShapeType};

use crate::disc_gfx::{run_display_lists, Segments};
use crate::game_disc::GameDisc;
use crate::n64_material::N64Material;
use crate::skeleton::{place_on_joints, Animation, Skeleton};

pub(crate) mod candle;
pub(crate) mod clocks;
pub(crate) mod cuckoo;
pub(crate) mod fan;
pub(crate) mod gong;
pub(crate) mod house_clock;
pub(crate) mod lawnmower;
pub(crate) mod rigs;
pub(crate) mod slot;
pub(crate) mod sprinkler;
pub(crate) mod storage;
pub(crate) mod television;

/// Game units per room unit (`mFI_UT_WORLDSIZE_X_F`).
const UNIT: f32 = 40.0;
/// Bytes per texture-animation frame we hand segment 8 (a 64x64 4-bit
/// texture; the lists only read what they load).
const TEX_ANIM_FRAME: usize = 0x800;

/// A furniture `aFTR_PROFILE`.
struct Profile {
    index: u32,
    name: String,
    opaque: Vec<u32>,
    translucent: Vec<u32>,
    palette: Option<u32>,
    rig: Option<u32>,
    tex_anim: Option<u32>,
    /// Where things placed on it stand (`aFTR_PROFILE.height`).
    height: f32,
    scale: f32,
    shape: ShapeType,
}

/// `mRmTp_FtrItemNo2FtrIdx`: the furniture index of a furniture item
/// (`FTR0` items from 0x1000, `FTR1` from 0x3000 continuing at 0x400).
pub fn furniture_index(item: u16) -> Option<u32> {
    match item >> 12 {
        1 => Some(u32::from(item - 0x1000) >> 2),
        3 => Some(0x400 + (u32::from(item - 0x3000) >> 2)),
        _ => None,
    }
}

/// Playtest storage, resolved from the runtime profile table rather than a
/// second hardcoded numeric item mapping. Requires its original animated rig.
pub(crate) fn playtest_storage(disc: &GameDisc, drawers: bool) -> Option<u16> {
    let wanted = disc.catalog.get(if drawers {
        "iam_sum_blue_lowchest01"
    } else {
        "iam_sum_blue_cab01"
    })?;
    let table = disc.catalog.get("furniture_quality")?;
    let index = (0..table.size / 4).find(|&i| {
        matches!(disc.rom.pointer_at(RomSource::Rel, table.offset + i * 4),
            Some(Target::File(RomSource::Rel, p)) if p == wanted.offset)
    })?;
    let profile = read_profile(disc, index)?;
    let (_, animation) = profile_skeleton(disc, &profile)?;
    animation?;
    storage::open_end(&profile.name)?;
    Some(rustimal_logic::furniture::ftr_idx_2_item_no(
        index as u16,
        rustimal_logic::furniture::Direction::South,
    ))
}

fn shape_type(b: u8) -> ShapeType {
    match b {
        0 => ShapeType::TypeB90,
        1 => ShapeType::TypeB180,
        2 => ShapeType::TypeB270,
        3 => ShapeType::TypeB0,
        5 => ShapeType::TypeC,
        _ => ShapeType::TypeA,
    }
}

/// `l_typeB0_table`: a 1x2 piece's shape for each rotation.
fn rotated_shape(shape: ShapeType, rotation: u16) -> ShapeType {
    match shape {
        ShapeType::TypeB0 => [
            ShapeType::TypeB0,
            ShapeType::TypeB90,
            ShapeType::TypeB180,
            ShapeType::TypeB270,
        ][rotation as usize & 3],
        s => s,
    }
}

fn read_profile(disc: &GameDisc, index: u32) -> Option<Profile> {
    let rom = &disc.rom;
    let table = disc.catalog.get("furniture_quality")?;
    if index >= table.size / 4 {
        return None;
    }
    let rel = |o: u32| match rom.pointer_at(RomSource::Rel, o) {
        Some(Target::File(RomSource::Rel, p)) => Some(p),
        _ => None,
    };
    let p = rel(table.offset + index * 4)?;
    let b = rom.slice(RomSource::Rel, p + 0x20, 9)?;
    let f = |o: usize| f32::from_be_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
    Some(Profile {
        index,
        name: disc
            .catalog
            .containing(RomSource::Rel, p)
            .map_or_else(|| format!("furniture {index}"), |a| a.name.to_string()),
        opaque: [0, 4].iter().filter_map(|&o| rel(p + o)).collect(),
        translucent: [8, 12].iter().filter_map(|&o| rel(p + o)).collect(),
        palette: rel(p + 0x14),
        rig: rel(p + 0x18),
        tex_anim: rel(p + 0x1C),
        height: f(0),
        scale: f(4),
        shape: shape_type(b[8]),
    })
}

/// Persistent gameplay fields from aFTR_PROFILE and aMR_layer_set_info.
/// The latter is a separate table, not a field of the retail profile.
pub(crate) fn gameplay_profile(disc: &GameDisc, index: u16) -> Option<FurnitureProfile> {
    let p = read_profile(disc, u32::from(index))?;
    let table = disc.catalog.get("furniture_quality")?;
    let Target::File(RomSource::Rel, offset) = disc
        .rom
        .pointer_at(RomSource::Rel, table.offset + u32::from(index) * 4)?
    else {
        return None;
    };
    let b = disc.rom.slice(RomSource::Rel, offset + 0x28, 8)?;
    let sets = disc.catalog.get("aMR_layer_set_info")?;
    let set = *disc
        .rom
        .slice(sets.source, sets.offset, sets.size)?
        .get(index as usize)?;
    Some(FurnitureProfile {
        height: p.height,
        scale: p.scale,
        shape: p.shape,
        move_bg_type: b[1],
        check_rotation: b[2],
        kankyo_map: b[3],
        contact_action: b[4],
        interaction_type: u16::from_be_bytes([b[6], b[7]]),
        set_type: match set {
            1 => SetType::Surface,
            2 => SetType::OnSurface,
            _ => SetType::Normal,
        },
        unique_behavior: disc.rom.pointer_at(RomSource::Rel, offset + 0x30).is_some(),
    })
}

/// Resolve the same draw paths used by spawn_furniture before consuming a
/// pocket item. Unsupported models remain in pockets.
pub(crate) fn can_render(disc: &GameDisc, index: u16) -> bool {
    let Some(p) = read_profile(disc, u32::from(index)) else {
        return false;
    };
    if p.name == "iam_yaz_candle" {
        return candle::available(disc);
    }
    if television::kind(&p.name).is_some() {
        return television::available(disc, &p);
    }
    if slot::is_slot(index) {
        return slot::available(disc, &p);
    }
    if fan::is_fan(index) {
        return fan::available(disc, &p);
    }
    if gong::is_gong(index) {
        return gong::available(disc, &p);
    }
    if cuckoo::is_cuckoo(index) {
        return cuckoo::available(disc, &p);
    }
    if sprinkler::is_sprinkler(index) {
        return sprinkler::available(disc, &p);
    }
    if lawnmower::is_lawnmower(index) {
        return lawnmower::available(disc, &p);
    }
    if clocks::spec(&p.name).is_some() {
        return clocks::available(disc, &p);
    }
    if !p.opaque.is_empty() || !p.translucent.is_empty() || profile_skeleton(disc, &p).is_some() {
        return true;
    }
    let (opaque, translucent) = drawn_lists(disc, &p, &mut Segments::default());
    !opaque.is_empty() || !translucent.is_empty()
}

/// aMR_Fname2ConpoIndex / Na_MDPlayerPos's original device gain classes.
pub(crate) fn music_boost(disc: &GameDisc, index: u16) -> f32 {
    music_device(disc, index).0
}

pub(crate) fn music_device(disc: &GameDisc, index: u16) -> (f32, u8, bool) {
    match read_profile(disc, index as u32).map(|p| p.name).as_deref() {
        Some(
            "iam_sum_stereo01" | "iam_sum_chikuon01" | "iam_sum_chikuon02" | "iam_sum_jukebox"
            | "iam_sum_lv_stereo",
        ) => (1.08, 0x30, true),
        Some("iam_sum_radio01" | "iam_sum_radio02" | "iam_sum_casse01") => (1.4, 0xE3, false),
        _ => (1.0, 0, false),
    }
}

/// The skeleton a profile draws: its rig's, or the `cKF_bs_r_int_*` one
/// its draw function names (`iam_<name>` -> `cKF_bs_r_int_<name>`).
fn profile_skeleton(disc: &GameDisc, profile: &Profile) -> Option<(Skeleton, Option<Animation>)> {
    let rom = &disc.rom;
    let rel = |o: u32| match rom.pointer_at(RomSource::Rel, o) {
        Some(Target::File(RomSource::Rel, p)) => Some(p),
        _ => None,
    };
    let (skel, anim) = match profile.rig {
        Some(rig) => (rel(rig)?, rel(rig + 4)),
        None => {
            let mut base = profile.name.strip_prefix("iam_")?.to_string();
            if let Some(balloon) = rigs::balloon_base(profile.index) {
                base = balloon;
            }
            // ac_sum_classicchest01's ctor/draw explicitly use clchest01.
            if base == "sum_classicchest01" {
                base = "sum_clchest01".to_string();
            }
            // Gyroids share one profile; `aHnwCommon_ct` picks the
            // skeleton from `hnw_cKF_bs_r_table` by furniture index.
            if base == "hnw_common" {
                let n = match profile.index.checked_sub(GYROID_FIRST)? {
                    i @ 0..=24 => i + 1,
                    25 => 126,
                    26 => 127,
                    _ => 1,
                };
                base = format!("hnw{n:03}");
            }
            let off = |kind: &str| {
                disc.catalog
                    .get(&format!("cKF_{kind}_r_int_{base}"))
                    .map(|a| a.offset)
            };
            (off("bs")?, off("ba"))
        }
    };
    let skeleton = Skeleton::read(rom, skel)?;
    let anim = anim.and_then(|a| Animation::read(rom, a, skeleton.joints.len()));
    Some((skeleton, anim))
}

/// First gyroid furniture index (`aHnwCommon_ct`: `name - 0x16C`).
const GYROID_FIRST: u32 = 0x16C;
/// First umbrella furniture index (`aFU_dw`: `name - 0x342`).
const UMBRELLA_FIRST: u32 = 0x342;

/// First mannequin furniture index (`FTR_CLOTH_START`): one per shirt.
const MANNEQUIN_FIRST: u32 = 0x1EB;
/// A shirt texture and its palette (`tex_boy.bin` / `pallet_boy.bin`).
const CLOTH_TEX_SIZE: usize = 0x200;
const CLOTH_PALETTE_SIZE: usize = 0x20;

/// Lists a profile without its own picks in its draw function: umbrellas
/// from `aFU_data`, mannequins wearing their shirt (`aFM_dw`: texture in
/// segment 8, palette in 9), or the `int_<name>_on` / `_off` lists
/// switchable furniture draws (`aSumFruittv01_dw`), with its first
/// screen frame in segment 8.
fn drawn_lists(disc: &GameDisc, profile: &Profile, segs: &mut Segments) -> (Vec<u32>, Vec<u32>) {
    let list = |n: &str| disc.catalog.get(n).map(|a| a.offset);
    let base = profile.name.trim_start_matches("iam_");
    if base == "fmanekin" {
        let cloth = profile.index.saturating_sub(MANNEQUIN_FIRST) as usize;
        let bank = |b: &Arc<[u8]>, size: usize| -> Arc<[u8]> {
            b.get(cloth * size..(cloth + 1) * size)
                .unwrap_or_default()
                .into()
        };
        segs.set_segment(8, bank(&disc.player_cloth, CLOTH_TEX_SIZE));
        segs.set_segment(9, bank(&disc.player_cloth_palettes, CLOTH_PALETTE_SIZE));
        return (
            list("obj_shop_manekin_model").into_iter().collect(),
            Vec::new(),
        );
    }
    if base == "fumbrella" {
        let n = (profile.index.wrapping_sub(UMBRELLA_FIRST) & 0x1F) + 1;
        let m = format!("obj_shop_umb{n:02}_model");
        return (
            list(&m).into_iter().collect(),
            list(&format!("{m}T")).into_iter().collect(),
        );
    }
    let names = |parts: &[&str]| -> Vec<u32> {
        parts
            .iter()
            .filter_map(|p| list(&format!("int_{base}{p}")))
            .collect()
    };
    if let Some(a) = list(&format!("int_{base}_front1_TA_tex_txt")) {
        let rel = disc.rom.bytes(RomSource::Rel);
        let end = (a as usize + TEX_ANIM_FRAME).min(rel.len());
        segs.set_segment(8, rel.get(a as usize..end).unwrap_or_default().into());
    }
    (
        names(&["_on_model", "_off_model", "_model"]),
        names(&["_onT_model", "_offT_model", "_modelT"]),
    )
}

/// aMR_DrawRegistModel is followed by the profile's unique draw callback.
/// These four lamp callbacks append static lists to POLY_OPA_DISP, including
/// the T-named parts. They do not branch on switch_bit or create a light.
fn lamp_draw_extras(disc: &GameDisc, profile: &Profile) -> Vec<u32> {
    let names: &[&str] = match profile.name.as_str() {
        "iam_sum_bla_lanp" => &["int_sum_bla_lanp_off_model", "int_sum_bla_lanp_offT_model"],
        "iam_sum_whi_lanp" => &["int_sum_whi_lanp_offT_model"],
        "iam_sum_gre_lanp01" => &["int_sum_gre_lanp01_offT_model"],
        "iam_sum_hal_lanp01" => &["int_sum_hal_lanp01_offT_model"],
        _ => &[],
    };
    names
        .iter()
        .filter_map(|name| disc.catalog.get(name).map(|a| a.offset))
        .collect()
}

/// How high things put on furniture `item` stand, in room units.
pub fn top_height(disc: &GameDisc, item: u16) -> f32 {
    furniture_index(item)
        .and_then(|i| read_profile(disc, i))
        .map_or(0.0, |p| p.height / UNIT)
}

/// The room units a placed piece covers, from its primary unit.
pub fn footprint(disc: &GameDisc, item: u16, ut: (i32, i32)) -> Vec<(i32, i32)> {
    let Some(profile) = furniture_index(item).and_then(|i| read_profile(disc, i)) else {
        return vec![ut];
    };
    let shape = rotated_shape(profile.shape, item & 3);
    shape.unit_offsets()[..shape.unit_count()]
        .iter()
        .map(|&(dx, dz)| (ut.0 + dx, ut.1 + dz))
        .collect()
}

/// Spawns furniture `item` standing on room unit `ut`, with `corner` the
/// world position of unit (0, 0)'s corner on the floor. Returns the
/// entity, or `None` for an item with no model on the disc.
#[allow(clippy::too_many_arguments)]
pub fn spawn_furniture(
    commands: &mut Commands,
    disc: &GameDisc,
    item: u16,
    ut: (i32, i32),
    corner: Vec3,
    lift: f32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) -> Option<Entity> {
    let profile = read_profile(disc, furniture_index(item)?)?;
    let rotation = item & 3;
    let mut centre = Vec2::new(ut.0 as f32 + 0.5, ut.1 as f32 + 0.5);
    if profile.shape == ShapeType::TypeC {
        centre += Vec2::splat(0.5);
    }
    let parent = commands
        .spawn((
            Name::new(profile.name.trim_start_matches("iam_").to_string()),
            Transform::from_translation(corner + Vec3::new(centre.x, lift, centre.y))
                .with_rotation(Quat::from_rotation_y(
                    f32::from(rotation) * std::f32::consts::FRAC_PI_2,
                ))
                .with_scale(Vec3::splat(profile.scale / UNIT)),
            Visibility::default(),
        ))
        .id();

    let mut segs = Segments::default();
    if profile.name == "iam_yaz_candle" {
        candle::spawn(commands, disc, parent, (meshes, materials, images));
        return Some(parent);
    }
    if slot::is_slot(profile.index as u16) {
        slot::spawn(
            commands,
            disc,
            parent,
            &profile,
            (meshes, materials, images),
        );
        return Some(parent);
    }
    if lawnmower::is_lawnmower(profile.index as u16) {
        lawnmower::spawn(
            commands,
            disc,
            parent,
            &profile,
            (meshes, materials, images),
        );
        return Some(parent);
    }
    if sprinkler::is_sprinkler(profile.index as u16) {
        sprinkler::spawn(
            commands,
            disc,
            parent,
            &profile,
            (meshes, materials, images),
        );
        return Some(parent);
    }
    if cuckoo::is_cuckoo(profile.index as u16) {
        cuckoo::spawn(
            commands,
            disc,
            parent,
            &profile,
            (meshes, materials, images),
        );
        return Some(parent);
    }
    if gong::is_gong(profile.index as u16) {
        gong::spawn(
            commands,
            disc,
            parent,
            &profile,
            (meshes, materials, images),
        );
        return Some(parent);
    }
    if fan::is_fan(profile.index as u16) {
        fan::spawn(
            commands,
            disc,
            parent,
            &profile,
            (meshes, materials, images),
        );
        return Some(parent);
    }
    if let Some(kind) = television::kind(&profile.name) {
        television::spawn(
            commands, disc, parent, &profile, kind, meshes, materials, images,
        );
        return Some(parent);
    }
    // `aFTR_tex_anim_c.animation[0]`: the first frame's texture.
    if let Some(Target::File(RomSource::Rel, a)) = profile
        .tex_anim
        .and_then(|t| disc.rom.pointer_at(RomSource::Rel, t))
        .and_then(|frames| match frames {
            Target::File(RomSource::Rel, f) => disc.rom.pointer_at(RomSource::Rel, f),
            _ => None,
        })
    {
        let rel = disc.rom.bytes(RomSource::Rel);
        let end = (a as usize + TEX_ANIM_FRAME).min(rel.len());
        let bytes: Arc<[u8]> = rel.get(a as usize..end).unwrap_or_default().into();
        segs.set_segment(8, bytes);
    }

    let skeleton = (profile.opaque.is_empty() && profile.translucent.is_empty())
        .then(|| profile_skeleton(disc, &profile))
        .flatten();
    let (opaque, translucent) = if skeleton.is_some() {
        (Vec::new(), Vec::new())
    } else if profile.opaque.is_empty() && profile.translucent.is_empty() {
        drawn_lists(disc, &profile, &mut segs)
    } else {
        let mut opaque = profile.opaque.clone();
        opaque.extend(lamp_draw_extras(disc, &profile));
        (opaque, profile.translucent.clone())
    };
    let assets = (&mut *meshes, &mut *materials, &mut *images);
    if !opaque.is_empty() || !translucent.is_empty() {
        let (meshes, materials, images) = assets;
        for (lists, blend) in [(&opaque, false), (&translucent, true)] {
            if lists.is_empty() {
                continue;
            }
            let sink = run_display_lists(&disc.rom, lists);
            crate::interior::spawn_sink(
                commands,
                disc,
                parent,
                &sink,
                profile.palette,
                &segs,
                blend,
                meshes,
                materials,
                images,
            );
        }
    } else if let Some((skeleton, anim)) = skeleton {
        if let Some((spec, animation)) = clocks::spec(&profile.name).zip(anim.as_ref()) {
            clocks::spawn(
                commands,
                disc,
                parent,
                &profile,
                spec,
                skeleton,
                animation.clone(),
                &segs,
                assets,
            );
            return Some(parent);
        }
        if let Some(animation) = anim.as_ref().filter(|_| rigs::animated(&profile)) {
            rigs::spawn(
                commands,
                disc,
                parent,
                &profile,
                skeleton,
                animation.clone(),
                &segs,
                assets,
            );
            return Some(parent);
        }
        // Only audited common door constructors use this animated path.
        // Other per-type furniture keeps its existing fixed-pose renderer.
        if let Some((end, animation)) = storage::open_end(&profile.name).zip(anim.as_ref()) {
            storage::spawn(
                commands,
                disc,
                parent,
                skeleton,
                animation.clone(),
                end,
                profile.palette,
                &segs,
                assets,
            );
            return Some(parent);
        }
        let lists = skeleton.lists();
        let mut sink = run_display_lists(&disc.rom, &lists);
        let world = skeleton.world_matrices(anim.as_ref(), 1.0);
        place_on_joints(&mut sink, &skeleton, &world);
        let (meshes, materials, images) = assets;
        crate::interior::spawn_sink(
            commands,
            disc,
            parent,
            &sink,
            profile.palette,
            &segs,
            false,
            meshes,
            materials,
            images,
        );
    } else {
        warn!("no model for furniture {} ({item:#06x})", profile.name);
    }
    Some(parent)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Probe (needs the disc): furniture indices with no model.
    #[test]
    #[ignore]
    fn probe_furniture_models() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let disc = crate::game_disc::load(path.into()).unwrap();
        let mut missing = Vec::new();
        for index in 0..disc.catalog.get("furniture_quality").unwrap().size / 4 {
            let Some(p) = read_profile(&disc, index) else {
                continue;
            };
            let drawn = drawn_lists(&disc, &p, &mut Segments::default());
            if p.opaque.is_empty()
                && p.translucent.is_empty()
                && profile_skeleton(&disc, &p).is_none()
                && drawn.0.is_empty()
                && drawn.1.is_empty()
            {
                missing.push(format!("{index:#x} {}", p.name));
            }
        }
        println!("{} without a model: {missing:?}", missing.len());
    }

    /// Probe (needs the disc): the shop's layout.
    #[test]
    #[ignore]
    fn probe_shop_layout() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let disc = crate::game_disc::load(path.into()).unwrap();
        let recs = rustimal_logic::parse_fg_records(&disc.fgdata).unwrap();
        let lists: Vec<u32> = [
            "rom_shop1f_model",
            "rom_shop1f_modelT",
            "rom_shop1w_model",
            "rom_shop1w_modelT",
            "rom_shop1_fuku_model",
            "rom_shop1_fuku_modelT",
        ]
        .iter()
        .filter_map(|n| disc.catalog.get(n).map(|a| a.offset))
        .collect();
        let sink = run_display_lists(&disc.rom, &lists);
        let r = recs.iter().find(|r| r.fg_id == 0x22).unwrap();
        for (i, &it) in r.items.iter().enumerate() {
            if it >> 8 != 0xFE {
                continue;
            }
            let (x, z) = ((i % 16) as f32, (i / 16) as f32);
            let k = 40.0 / 0.0625;
            let top = sink.height_at((x + 0.5) * k, (z + 0.5) * k).map(|y| y / k);
            println!("slot {it:04x} at ({x},{z}) top {top:?}");
        }
        for id in [0x22u16, 0x23] {
            let Some(r) = recs.iter().find(|r| r.fg_id == id) else {
                continue;
            };
            println!("fg {id:#x}");
            for z in 0..16 {
                let row: Vec<String> = (0..16)
                    .map(|x| format!("{:04x}", r.items[z * 16 + x]))
                    .collect();
                println!("{}", row.join(" "));
            }
        }
    }

    #[test]
    fn furniture_indices() {
        assert_eq!(furniture_index(0x1000), Some(0));
        assert_eq!(furniture_index(0x1073), Some(0x1C));
        assert_eq!(furniture_index(0x3004), Some(0x401));
        assert_eq!(furniture_index(0x2000), None);
        assert_eq!(furniture_index(0xFFFE), None);
    }

    #[test]
    fn one_by_two_turns_with_the_item() {
        assert_eq!(rotated_shape(ShapeType::TypeB0, 0), ShapeType::TypeB0);
        assert_eq!(rotated_shape(ShapeType::TypeB0, 1), ShapeType::TypeB90);
        assert_eq!(rotated_shape(ShapeType::TypeC, 3), ShapeType::TypeC);
    }
}
