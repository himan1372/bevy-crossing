//! Source aSG_DrawGoodsPicture mode1: foreground goods on floor/surfaces.
//! Bounded diary family; storage contents are never displayed as room goods.
use super::*;

#[derive(Component, Clone, Copy)]
pub(super) struct Piece {
    pub item: u16,
    pub layer: usize,
    pub x: i32,
    pub z: i32,
}

fn visible(room: &PlayerRoom) -> Vec<Piece> {
    let mut pieces = Vec::new();
    for layer in 0..2 {
        for z in 0..16 {
            for x in 0..16 {
                let item = room.layers[layer].items[z][x];
                if !(0x2B00..=0x2B0F).contains(&item) {
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
                });
            }
        }
    }
    pieces
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
    for piece in visible(room) {
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
