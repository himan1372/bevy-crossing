//! The player's room from the disc (`ac_my_indoor`): the house's floor
//! and wall models (`rom_myhome1_floor` / `_wall`), patterned with the
//! carpet and wallpaper the house starts with
//! (`l_mHm_player_room_default_data`), and the starting furniture.
//!
//! Patterns live in `player_room_floor.bin` (0x2020 bytes each: a 16-colour
//! palette then four 64x64 4-bit textures) and `player_room_wall.bin`
//! (0x1020: palette then two textures). The floor model reads its
//! textures from segments 8-B and its palette from C; the wall model reads
//! segments 8 and 9 and its palette from A (`aMI` draw).

use std::sync::Arc;

use bevy::prelude::*;
use disc::RomSource;

use crate::disc_gfx::{decode_texture, run_display_lists, Segments};
use crate::game_disc::GameDisc;
use crate::n64_material::{part_mesh, texture_image, CombinerUniform, N64Material};
use crate::player_room::decorating::Decoration;

/// `aMI` draws the room at scale 0.0625; 40 game units per cell.
const ROOM_SCALE: f32 = 0.0625 / 40.0;
/// The small room's floor spans model cells 1..5 in x and z at height 1:
/// its centre goes on the interior origin.
const ROOM_CENTRE: Vec3 = Vec3::new(3.0, 1.0, 3.0);
/// Half the room's inside width, in cells.
pub const ROOM_HALF: f32 = 2.0;
/// rom_myhome1_wall_v[0..6], drawn with rom_myhome_enter2_tex:
/// after ROOM_SCALE / ROOM_CENTRE, the wooden landing is x[-1,1], z[2,4].
pub(crate) const ENTRANCE_HALF_WIDTH: f32 = 1.0;
pub(crate) const ENTRANCE_END_Z: f32 = 4.0;

const FLOOR_BLOCK: usize = 0x2020;
const WALL_BLOCK: usize = 0x1020;
const PALETTE: usize = 0x20;
const TEXTURE: usize = 0x800;

/// Only the player shell changes when decorating; other interiors stay intact.
#[derive(Component)]
pub(crate) struct PlayerRoomShell(pub Decoration);

/// A piece of furniture in the room: (centre relative to the room
/// centre, half extents x/z) for collisions.
pub struct RoomFurniture {
    pub centre: Vec3,
    pub half: Vec2,
}

fn slice(bank: &Arc<[u8]>, start: usize, len: usize) -> Arc<[u8]> {
    bank.get(start..start + len).unwrap_or_default().into()
}

/// Builds the player's room at `origin`; returns the furniture to collide
/// with, or `None` without the disc's room.
pub fn spawn_player_room(
    commands: &mut Commands,
    disc: &GameDisc,
    origin: Vec3,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) -> Option<Vec<RoomFurniture>> {
    let decoration = Decoration::defaults(disc)?;
    spawn_decorated_player_room(
        commands,
        disc,
        origin,
        decoration,
        (meshes, materials, images),
    )?;
    Some(Vec::new())
}

pub(crate) fn spawn_decorated_player_room(
    commands: &mut Commands,
    disc: &GameDisc,
    origin: Vec3,
    decoration: Decoration,
    assets: RoomAssets,
) -> Option<Entity> {
    if !decoration.available(disc) {
        return None;
    }
    let floor_list = disc.catalog.get("rom_myhome1_floor_model")?.offset;
    let wall_list = disc.catalog.get("rom_myhome1_wall_model")?.offset;
    let (wall, floor) = (usize::from(decoration.wall), usize::from(decoration.floor));
    let shell = spawn_shell(
        commands,
        disc,
        "player room",
        origin,
        ROOM_CENTRE,
        &[(floor_list, false)],
        &[(wall_list, false)],
        &[],
        floor,
        wall,
        assets,
    );
    commands.entity(shell).insert(PlayerRoomShell(decoration));
    info!("player room from the disc: wallpaper {wall}, carpet {floor}");
    Some(shell)
}

/// Floor segments (`aMI` / `aSI` draw): textures in 8-B, palette in C.
fn floor_segments(disc: &GameDisc, floor: usize) -> Segments {
    let mut segs = Segments::default();
    let fb = floor * FLOOR_BLOCK;
    segs.set_segment(0x0C, slice(&disc.player_room_floors, fb, PALETTE));
    for i in 0..4 {
        segs.set_segment(
            0x08 + i as u8,
            slice(
                &disc.player_room_floors,
                fb + PALETTE + i * TEXTURE,
                TEXTURE,
            ),
        );
    }
    segs
}

/// Wall segments: textures in 8 and 9, palette in A.
fn wall_segments(disc: &GameDisc, wall: usize) -> Segments {
    let mut segs = Segments::default();
    let wb = wall * WALL_BLOCK;
    segs.set_segment(0x0A, slice(&disc.player_room_walls, wb, PALETTE));
    for i in 0..2 {
        segs.set_segment(
            0x08 + i as u8,
            slice(&disc.player_room_walls, wb + PALETTE + i * TEXTURE, TEXTURE),
        );
    }
    segs
}

type RoomAssets<'a> = (
    &'a mut Assets<Mesh>,
    &'a mut Assets<N64Material>,
    &'a mut Assets<Image>,
);

/// A room's shell at `origin`: floor, wall and other lists at the room
/// scale, with model cell `centre` on the origin.
#[allow(clippy::too_many_arguments)]
fn spawn_shell(
    commands: &mut Commands,
    disc: &GameDisc,
    name: &str,
    origin: Vec3,
    centre: Vec3,
    floor_lists: &[(u32, bool)],
    wall_lists: &[(u32, bool)],
    other_lists: &[(u32, bool)],
    floor: usize,
    wall: usize,
    (meshes, materials, images): RoomAssets,
) -> Entity {
    let room = commands
        .spawn((
            Name::new(name.to_string()),
            crate::footsteps::IndoorFloor {
                origin,
                index: floor,
            },
            Transform::from_translation(origin - centre).with_scale(Vec3::splat(ROOM_SCALE)),
            Visibility::default(),
        ))
        .id();
    let floor_segs = floor_segments(disc, floor);
    let wall_segs = wall_segments(disc, wall);
    for &(l, blend) in floor_lists {
        spawn_lists(
            commands,
            disc,
            room,
            &[l],
            None,
            &floor_segs,
            blend,
            meshes,
            materials,
            images,
        );
    }
    for &(l, blend) in wall_lists {
        spawn_lists(
            commands,
            disc,
            room,
            &[l],
            None,
            &wall_segs,
            blend,
            meshes,
            materials,
            images,
        );
    }
    for &(l, blend) in other_lists {
        spawn_blend_lists(commands, disc, room, l, blend, meshes, materials, images);
    }
    room
}

/// The shop's floor spans model cells 0.875..7.125: centre (4, 1, 4).
const SHOP_CENTRE: Vec3 = Vec3::new(4.0, 1.0, 4.0);
/// Half the shop's inside width, in cells.
pub const SHOP_HALF: f32 = 3.125;

/// Nook's Cranny's room (`ac_shop_indoor`, shop level 1): floor, walls and
/// the clothes rack, opaque and translucent, with the shop's default
/// carpet and wallpaper (`aSI_floor_default_table` /
/// `aSI_wall_default_table`). Returns false without the disc's room.
pub fn spawn_shop_room(
    commands: &mut Commands,
    disc: &GameDisc,
    origin: Vec3,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) -> bool {
    let list = |name: &str| disc.catalog.get(name).map(|a| a.offset);
    let (Some(floor), Some(wall)) = (list("rom_shop1f_model"), list("rom_shop1w_model")) else {
        return false;
    };
    let table = |name: &str| {
        let a = disc.catalog.get(name)?;
        let b = disc.rom.slice(RomSource::Rel, a.offset, 2)?;
        // `(u8)ITM_*`: the low byte is the pattern number.
        Some((i16::from_be_bytes([b[0], b[1]]) & 0xFF) as usize)
    };
    let floor_no = table("aSI_floor_default_table").unwrap_or(0);
    let wall_no = table("aSI_wall_default_table").unwrap_or(0);
    let mut floors = vec![(floor, false)];
    floors.extend(list("rom_shop1f_modelT").map(|l| (l, true)));
    let mut walls = vec![(wall, false)];
    walls.extend(list("rom_shop1w_modelT").map(|l| (l, true)));
    let others: Vec<(u32, bool)> = [
        ("rom_shop1_fuku_model", false),
        ("rom_shop1_fuku_modelT", true),
    ]
    .iter()
    .filter_map(|&(n, blend)| list(n).map(|l| (l, blend)))
    .collect();
    let assets = (&mut *meshes, &mut *materials, &mut *images);
    spawn_shell(
        commands,
        disc,
        "shop room",
        origin,
        SHOP_CENTRE,
        &floors,
        &walls,
        &others,
        floor_no,
        wall_no,
        assets,
    );
    info!("shop room from the disc: wallpaper {wall_no}, carpet {floor_no}");
    true
}

/// A list drawn as-is (its own textures), blended when translucent.
#[allow(clippy::too_many_arguments)]
fn spawn_blend_lists(
    commands: &mut Commands,
    disc: &GameDisc,
    parent: Entity,
    list: u32,
    blend: bool,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) {
    let segs = Segments::default();
    for part in &run_display_lists(&disc.rom, &[list]).parts {
        let texture = part.key.tex.as_ref().and_then(|k| {
            decode_texture(&disc.rom, k, None, &segs).map(|rgba| images.add(texture_image(rgba, k)))
        });
        let material = materials.add(N64Material {
            combiner: CombinerUniform::from_key(&part.key, false, blend),
            texture,
            env: default(),
            blend,
        });
        commands.spawn((
            Mesh3d(meshes.add(part_mesh(part, 1.0))),
            MeshMaterial3d(material),
            ChildOf(parent),
        ));
    }
}

/// `mNpc_NpcHouseData_c`: type, palette, wall, floor u8, then two layer
/// ids (8 bytes, one per villager).
const HOUSE_DATA_SIZE: u32 = 8;

/// A villager room's floor spans model cells 1..7: centre (4, 1, 4).
const VILLAGER_ROOM_CENTRE: Vec3 = Vec3::new(4.0, 1.0, 4.0);
/// Half a villager room's inside width, in cells.
pub const VILLAGER_ROOM_HALF: f32 = 3.0;

/// A villager's room (`npc_room01`): `ac_arrange_room` draws the 6x6 room
/// (`rom_myhome2_floor` / `_wall` / `_wallT`) with their own wallpaper and
/// carpet (`npc_house_list` -> `mNpc_RenewalNpcRoom`), and `ac_my_room`
/// their furniture: the main layer and what stands on it (the layouts in
/// `fgnpcdata.bin` named by the house data's layer ids,
/// `mFM_ChangeFGName`). Returns the furniture to collide with.
pub fn spawn_villager_room(
    commands: &mut Commands,
    disc: &GameDisc,
    npc_index: u32,
    origin: Vec3,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) -> Option<Vec<RoomFurniture>> {
    let list = |name: &str| disc.catalog.get(name).map(|a| a.offset);
    let (Some(floor), Some(wall)) = (
        list("rom_myhome2_floor_model"),
        list("rom_myhome2_wall_model"),
    ) else {
        return None;
    };
    let mut walls = vec![(wall, false)];
    walls.extend(list("rom_myhome2_wall_modelT").map(|l| (l, false)));
    let data = disc.catalog.get("npc_house_list").and_then(|a| {
        disc.rom.slice(
            RomSource::Rel,
            a.offset + npc_index * HOUSE_DATA_SIZE,
            HOUSE_DATA_SIZE,
        )
    })?;
    let (wall_no, floor_no) = (data[2] as usize, data[3] as usize);
    let assets = (&mut *meshes, &mut *materials, &mut *images);
    spawn_shell(
        commands,
        disc,
        "villager room",
        origin,
        VILLAGER_ROOM_CENTRE,
        &[(floor, false)],
        &walls,
        &[],
        floor_no,
        wall_no,
        assets,
    );

    let layer = |o: usize| u16::from_be_bytes([data[o], data[o + 1]]);
    let layouts = rustimal_logic::parse_fg_records(&disc.npc_room_layouts).ok()?;
    let items = |id: u16| layouts.iter().find(|r| r.fg_id == id).map(|r| r.items);
    let corner = origin - VILLAGER_ROOM_CENTRE.with_y(0.0);
    let mut out = Vec::new();
    // What stands on each unit of the main layer, for the upper layer.
    let mut under = std::collections::HashMap::new();
    for (i, &item) in items(layer(4)).iter().flatten().enumerate() {
        let ut = ((i % 16) as i32, (i / 16) as i32);
        if crate::furniture::furniture_index(item).is_none() {
            continue;
        }
        crate::furniture::spawn_furniture(
            commands, disc, item, ut, corner, 0.0, meshes, materials, images,
        );
        for cell in crate::furniture::footprint(disc, item, ut) {
            under.insert(cell, item);
            out.push(RoomFurniture {
                centre: corner + Vec3::new(cell.0 as f32 + 0.5, 0.0, cell.1 as f32 + 0.5) - origin,
                half: Vec2::splat(0.5),
            });
        }
    }
    for (i, &item) in items(layer(6)).iter().flatten().enumerate() {
        let ut = ((i % 16) as i32, (i / 16) as i32);
        if crate::furniture::furniture_index(item).is_none() {
            continue;
        }
        let lift = under
            .get(&ut)
            .map_or(0.0, |&below| crate::furniture::top_height(disc, below));
        crate::furniture::spawn_furniture(
            commands, disc, item, ut, corner, lift, meshes, materials, images,
        );
    }
    Some(out)
}

/// Spawns display lists as N64 meshes under `parent`.
#[allow(clippy::too_many_arguments)]
fn spawn_lists(
    commands: &mut Commands,
    disc: &GameDisc,
    parent: Entity,
    lists: &[u32],
    palette: Option<u32>,
    segs: &Segments,
    blend: bool,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) {
    let sink = run_display_lists(&disc.rom, lists);
    spawn_sink(
        commands, disc, parent, &sink, palette, segs, blend, meshes, materials, images,
    );
}

/// Spawns already-run display lists as N64 meshes under `parent`.
#[allow(clippy::too_many_arguments)]
pub fn spawn_sink(
    commands: &mut Commands,
    disc: &GameDisc,
    parent: Entity,
    sink: &crate::disc_gfx::TriangleSink,
    palette: Option<u32>,
    segs: &Segments,
    blend: bool,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) {
    for part in &sink.parts {
        let texture = part.key.tex.as_ref().and_then(|k| {
            decode_texture(&disc.rom, k, palette, segs)
                .map(|rgba| images.add(texture_image(rgba, k)))
        });
        let material = materials.add(N64Material {
            combiner: CombinerUniform::from_key(&part.key, false, blend),
            texture,
            env: default(),
            blend,
        });
        commands.spawn((
            Mesh3d(meshes.add(part_mesh(part, 1.0))),
            MeshMaterial3d(material),
            ChildOf(parent),
        ));
    }
}
