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

/// `aMI` draws the room at scale 0.0625; 40 game units per cell.
const ROOM_SCALE: f32 = 0.0625 / 40.0;
/// Furniture is drawn at 0.01 like other actors.
const FURNITURE_SCALE: f32 = 0.01 / 40.0;
/// The small room's floor spans model cells 1..5 in x and z at height 1:
/// its centre goes on the interior origin.
const ROOM_CENTRE: Vec3 = Vec3::new(3.0, 1.0, 3.0);
/// Half the room's inside width, in cells.
pub const ROOM_HALF: f32 = 2.0;

const FLOOR_BLOCK: usize = 0x2020;
const WALL_BLOCK: usize = 0x1020;
const PALETTE: usize = 0x20;
const TEXTURE: usize = 0x800;

/// `mHm_player_room_default_data_c`: wall u8, floor u8, item ut_x i32,
/// ut_z i32, item u16 (0x10 bytes).
const DEFAULT_SIZE: u32 = 0x10;

/// Furniture placed in a new house (`mHm_InitHomeInfo`): the house's own
/// default item, and the orange box in the back-left corner.
const STARTING_FURNITURE: [(&str, &str); 2] = [
    ("int_sum_casse01_on_model", "int_sum_casse01_pal"),
    ("int_nog_mikanbox_model", "int_nog_mikanbox_pal"),
];

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
    let floor_list = disc.catalog.get("rom_myhome1_floor_model")?.offset;
    let wall_list = disc.catalog.get("rom_myhome1_wall_model")?.offset;
    // House 0's starting wallpaper and carpet, and its furniture spot.
    let defaults = disc.catalog.get("l_mHm_player_room_default_data")?;
    let d = disc
        .rom
        .slice(RomSource::Rel, defaults.offset, DEFAULT_SIZE)?;
    let (wall, floor) = (d[0] as usize, d[1] as usize);
    let item_ut = (
        i32::from_be_bytes([d[4], d[5], d[6], d[7]]),
        i32::from_be_bytes([d[8], d[9], d[10], d[11]]),
    );

    let mut floor_segs = Segments::default();
    let fb = floor * FLOOR_BLOCK;
    floor_segs.set_segment(0x0C, slice(&disc.player_room_floors, fb, PALETTE));
    for i in 0..4 {
        floor_segs.set_segment(
            0x08 + i as u8,
            slice(
                &disc.player_room_floors,
                fb + PALETTE + i * TEXTURE,
                TEXTURE,
            ),
        );
    }
    let mut wall_segs = Segments::default();
    let wb = wall * WALL_BLOCK;
    wall_segs.set_segment(0x0A, slice(&disc.player_room_walls, wb, PALETTE));
    for i in 0..2 {
        wall_segs.set_segment(
            0x08 + i as u8,
            slice(&disc.player_room_walls, wb + PALETTE + i * TEXTURE, TEXTURE),
        );
    }

    let room = commands
        .spawn((
            Name::new("player room"),
            Transform::from_translation(origin - ROOM_CENTRE).with_scale(Vec3::splat(ROOM_SCALE)),
            Visibility::default(),
        ))
        .id();
    for (list, segs) in [(floor_list, &floor_segs), (wall_list, &wall_segs)] {
        spawn_lists(
            commands,
            disc,
            room,
            &[list],
            None,
            segs,
            1.0,
            meshes,
            materials,
            images,
        );
    }

    // Starting furniture on the room grid: unit (x, z) covers model cells
    // [x, x+1] (the floor starts at cell 1).
    let spots = [item_ut, (1, 1)];
    let mut out = Vec::new();
    for ((model, pal), (ux, uz)) in STARTING_FURNITURE.iter().zip(spots) {
        let Some(list) = disc.catalog.get(model).map(|a| a.offset) else {
            continue;
        };
        let palette = disc.catalog.get(pal).map(|a| a.offset);
        let centre = Vec3::new(ux as f32 + 0.5, 0.0, uz as f32 + 0.5) - ROOM_CENTRE.with_y(0.0);
        let piece = commands
            .spawn((
                Name::new(model.trim_end_matches("_model").to_string()),
                Transform::from_translation(origin + centre)
                    .with_scale(Vec3::splat(FURNITURE_SCALE)),
                Visibility::default(),
            ))
            .id();
        spawn_lists(
            commands,
            disc,
            piece,
            &[list],
            palette,
            &Segments::default(),
            1.0,
            meshes,
            materials,
            images,
        );
        out.push(RoomFurniture {
            centre,
            half: Vec2::splat(0.5),
        });
    }
    info!("player room from the disc: wallpaper {wall}, carpet {floor}");
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
    scale: f32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) {
    for part in &run_display_lists(&disc.rom, lists).parts {
        let texture = part.key.tex.as_ref().and_then(|k| {
            decode_texture(&disc.rom, k, palette, segs)
                .map(|rgba| images.add(texture_image(rgba, k)))
        });
        let material = materials.add(N64Material {
            combiner: CombinerUniform::from_key(&part.key, false, false),
            texture,
            env: default(),
            blend: false,
        });
        commands.spawn((
            Mesh3d(meshes.add(part_mesh(part, scale))),
            MeshMaterial3d(material),
            ChildOf(parent),
        ));
    }
}
