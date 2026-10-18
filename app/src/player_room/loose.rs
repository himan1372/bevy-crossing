//! Source aSG_DrawGoodsPicture mode1: foreground goods on floor/surfaces.
//! Audited loose goods; storage contents are never displayed as room goods.
use super::*;

/// Original SG_angle_y is runtime-only; quarter turns suffice for this adapter.
pub(super) type Angles = [[u8; 16]; 16];

/// ITM_PAPER_START..END includes all 64 designs and four encoded quantities.
/// Keep the complete item number: placing/picking up a bundle moves it intact.
/// Fruit excludes the adjacent mushroom/candy codes; shells include coral.
pub(super) fn supports(item: u16) -> bool {
    (0x2000..=0x20FF).contains(&item)
        || (0x2B00..=0x2B0F).contains(&item)
        || (0x2800..=0x2804).contains(&item)
        || item == 0x2807
        || (0x2514..=0x251B).contains(&item)
}

#[derive(Component, Clone, Copy)]
pub(super) struct Piece {
    pub item: u16,
    pub layer: usize,
    pub x: i32,
    pub z: i32,
    pub rotates: bool,
}

pub(super) fn position(room: &PlayerRoom, piece: Piece) -> Vec3 {
    let height = if piece.layer == 1 {
        room.runtime.occupancy[0][piece.z as usize][piece.x as usize]
            .and_then(|id| room.runtime.actors[id as usize].as_ref())
            .and_then(|a| room.profiles.get(&a.name))
            .map_or(0.0, |p| p.height / 40.0)
    } else {
        0.0
    };
    corner() + Vec3::new(piece.x as f32 + 0.5, height, piece.z as f32 + 0.5)
}

fn placed(item: u16, layer: usize, x: i32, z: i32) -> Piece {
    Piece {
        item,
        layer,
        x,
        z,
        rotates: false,
    }
}

fn visible(room: &PlayerRoom) -> Vec<Piece> {
    let mut pieces = Vec::new();
    for layer in 0..2 {
        for z in 0..16 {
            for x in 0..16 {
                let item = room.layers[layer].items[z][x];
                if !supports(item) {
                    continue;
                }
                let storage = room.runtime.occupancy[0][z][x]
                    .and_then(|id| room.runtime.actors[id as usize].as_ref())
                    .and_then(|a| room.profiles.get(&a.name))
                    .is_some_and(|p| ftr::is_storage(p.interaction_type));
                if layer == 1 && storage {
                    continue;
                }
                pieces.push(Piece {
                    item,
                    layer,
                    x: x as i32,
                    z: z as i32,
                    rotates: false,
                });
            }
        }
    }
    pieces
}

/// mTG_room_put_proc: front surface first, then current floor cell and the
/// eight adjacent units in alternating facing order. Commit only after choosing.
pub(super) fn place(
    room: &mut PlayerRoom,
    item: u16,
    pos: Vec3,
    facing: Vec3,
    disc: &GameDisc,
) -> Result<Piece, &'static str> {
    if !crate::shop_display::has_goods_model(disc, item) {
        return Err("That item isn't available on the selected disc.");
    }
    let offsets = [
        (0, 1),
        (1, 1),
        (1, 0),
        (1, -1),
        (0, -1),
        (-1, -1),
        (-1, 0),
        (-1, 1),
    ];
    // Original eight-way half-sector thresholds: (-22.5,22.5] faces south.
    let sector = ((facing.x.atan2(facing.z) / std::f32::consts::FRAC_PI_4 - 0.5).ceil() as i32)
        .rem_euclid(8) as usize;
    let (cx, cz) = player_cell(pos);
    let (dx, dz) = offsets[sector];
    let (x, z) = (cx + dx, cz + dz);
    let in_room = |x: i32, z: i32| (1..=4).contains(&x) && (1..=4).contains(&z);
    if in_room(x, z) {
        let below = room.runtime.occupancy[0][z as usize][x as usize]
            .and_then(|id| room.runtime.actors[id as usize].as_ref())
            .and_then(|a| room.profiles.get(&a.name));
        if ftr::judge_place_2nd_layer(
            below.is_some_and(|p| {
                p.set_type == SetType::Surface && !ftr::is_storage(p.interaction_type)
            }),
            room.layers[1].get(x, z) == Some(0)
                && room.runtime.occupancy[1][z as usize][x as usize].is_none(),
        ) {
            room.layers[1].set(x, z, item);
            room.loose_angles[z as usize][x as usize] = 0;
            room.dirty = true;
            return Ok(placed(item, 1, x, z));
        }
    }
    let clear = |x: i32, z: i32| {
        in_room(x, z)
            && room.layers[0].get(x, z) == Some(0)
            && room.layers[1].get(x, z) == Some(0)
            && room
                .runtime
                .occupancy
                .iter()
                .all(|layer| layer[z as usize][x as usize].is_none())
    };
    let chosen = std::iter::once((cx, cz))
        .chain([0, 1, -1, 2, -2, 3, -3, 4].into_iter().filter_map(|turn| {
            let (dx, dz) = offsets[(sector as i32 + turn).rem_euclid(8) as usize];
            // No placing through a diagonal corner blocked on both routes.
            (dx == 0 || dz == 0 || clear(cx + dx, cz) || clear(cx, cz + dz))
                .then_some((cx + dx, cz + dz))
        }))
        .find(|&(x, z)| clear(x, z));
    let (x, z) = chosen.ok_or("There's no room to put that here.")?;
    room.layers[0].set(x, z, item);
    room.dirty = true;
    Ok(placed(item, 0, x, z))
}

pub(super) fn target(room: &PlayerRoom, pos: Vec3, facing: Vec3) -> Option<Piece> {
    visible(room)
        .into_iter()
        .filter_map(|piece| {
            let delta = (corner() + Vec3::new(piece.x as f32 + 0.5, 0.0, piece.z as f32 + 0.5)
                - pos)
                .with_y(0.0);
            let forward = delta.dot(facing);
            (forward > 0.0
                && (delta - facing * forward).length() <= 0.55
                && delta.length() <= if piece.layer == 0 { 1.25 } else { 1.4 })
            .then_some((piece, delta.length_squared()))
        })
        .min_by(|a, b| b.0.layer.cmp(&a.0.layer).then_with(|| a.1.total_cmp(&b.1)))
        .map(|(piece, _)| piece)
}

pub(super) fn refresh(
    commands: &mut Commands,
    room: &PlayerRoom,
    disc: &GameDisc,
    existing: &Query<Entity, With<Piece>>,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<crate::n64_material::N64Material>,
        &mut Assets<Image>,
    ),
) {
    for entity in existing {
        commands.entity(entity).despawn();
    }
    for mut piece in visible(room) {
        piece.rotates =
            piece.layer == 1 && crate::shop_display::goods_rotates_on_surface(disc, piece.item);
        let lift = if piece.layer == 1 {
            room.runtime.occupancy[0][piece.z as usize][piece.x as usize]
                .and_then(|id| room.runtime.actors[id as usize].as_ref())
                .and_then(|a| room.profiles.get(&a.name))
                .map_or(0.0, |p| p.height / 40.0)
        } else {
            0.0
        };
        if let Some(entity) = crate::shop_display::spawn_goods(
            commands,
            disc,
            piece.item,
            (piece.x, piece.z),
            corner(),
            lift,
            (meshes, materials, images),
        ) {
            commands.entity(entity).insert(piece);
        }
    }
}
