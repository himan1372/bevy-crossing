//! Bounded native adapter for ac_my_room_action's atomic foreground movement.
//! Reuses furniture ownership/encoding; no rendering or C ABI.
use crate::furniture::{
    self as f, Direction, FurnitureProfile, MyRoomRuntime, RoomLayer, ShapeType,
};

/// Propose the whole move on copies. The caller validates the player's swept
/// collision path before publishing it. Contents stay on their owning actors.
/// The first adapter keeps the saved primary cell as the rotation pivot; C's
/// contact-dependent TypeB pivot switching remains a separate fidelity gap.
pub fn relocate(
    room: &MyRoomRuntime,
    layers: &[RoomLayer; 4],
    slot: usize,
    target: (i32, i32, Direction),
    max_cell: i32,
    profile: &dyn Fn(u16) -> Option<FurnitureProfile>,
) -> Result<(MyRoomRuntime, [RoomLayer; 4], Vec<usize>), &'static str> {
    let actor = room
        .actors
        .get(slot)
        .and_then(Option::as_ref)
        .ok_or("That furniture is gone.")?;
    if actor.layer != 0 {
        return Err("Move the base instead.");
    }
    let p = profile(actor.name).ok_or("Can't move that furniture.")?;
    let turn = (target.2 as i32 - ((actor.angle_y / 90.0).round() as i32)).rem_euclid(4);
    let shape = if p.shape == ShapeType::TypeB0 {
        [
            ShapeType::TypeB0,
            ShapeType::TypeB90,
            ShapeType::TypeB180,
            ShapeType::TypeB270,
        ][target.2 as usize]
    } else {
        p.shape
    };
    let mut edits = vec![(slot, target.0, target.1, target.2, shape)];
    // aMR_RequestItemToFitFurniture: supported top actors travel with the base.
    // Refuse loose/unowned foreground rather than silently losing it.
    for &(dx, dz) in actor
        .shape_type
        .unit_offsets()
        .iter()
        .take(actor.shape_type.unit_count())
    {
        let (x, z) = (actor.cell_x + dx, actor.cell_z + dz);
        if layers[1].get(x, z) != Some(0) && room.occupancy[1][z as usize][x as usize].is_none() {
            return Err("Clear the top first.");
        }
    }
    let center = if p.shape == ShapeType::TypeC {
        1.0
    } else {
        0.5
    };
    for (id, child) in room
        .actors
        .iter()
        .enumerate()
        .filter_map(|(i, a)| a.as_ref().map(|a| (i, a)))
    {
        if child.layer != 1
            || room.occupancy[0][child.cell_z as usize][child.cell_x as usize] != Some(slot as u16)
        {
            continue;
        }
        if child.shape_type != ShapeType::TypeA {
            return Err("Clear the top first.");
        }
        let (mut x, mut z) = (
            (child.cell_x - actor.cell_x) as f32 + 0.5 - center,
            (child.cell_z - actor.cell_z) as f32 + 0.5 - center,
        );
        for _ in 0..turn {
            (x, z) = (z, -x);
        }
        let x = x + target.0 as f32 + center - 0.5;
        let z = z + target.1 as f32 + center - 0.5;
        if (x - x.round()).abs() > 0.01 || (z - z.round()).abs() > 0.01 {
            return Err("Clear the top first.");
        }
        let dir = match (((child.angle_y / 90.0).round() as i32) + turn).rem_euclid(4) {
            1 => Direction::East,
            2 => Direction::North,
            3 => Direction::West,
            _ => Direction::South,
        };
        edits.push((
            id,
            x.round() as i32,
            z.round() as i32,
            dir,
            ShapeType::TypeA,
        ));
    }
    let mut next = room.clone();
    let mut fg = layers.clone();
    for &(id, _, _, _, _) in &edits {
        let a = room.actors[id].as_ref().ok_or("That furniture is gone.")?;
        f::set_furniture_to_fg(
            &mut fg[a.layer],
            a.name,
            a.shape_type,
            Direction::South,
            a.cell_x,
            a.cell_z,
            false,
        );
        next.release(id as u16);
    }
    // Conservative swept-area guard for TypeB turns. This deliberately refuses
    // some retail contact-pivot moves rather than clipping adjacent furniture.
    if turn != 0 && actor.shape_type.unit_count() == 2 {
        let mut cells = actor.shape_type.unit_offsets()[..2]
            .iter()
            .map(|&(x, z)| (actor.cell_x + x, actor.cell_z + z))
            .collect::<Vec<_>>();
        cells.extend(
            shape.unit_offsets()[..2]
                .iter()
                .map(|&(x, z)| (target.0 + x, target.1 + z)),
        );
        let min_x = cells.iter().map(|p| p.0).min().unwrap_or(target.0);
        let max_x = cells.iter().map(|p| p.0).max().unwrap_or(target.0);
        let min_z = cells.iter().map(|p| p.1).min().unwrap_or(target.1);
        let max_z = cells.iter().map(|p| p.1).max().unwrap_or(target.1);
        for z in min_z..=max_z {
            for x in min_x..=max_x {
                if !(1..=max_cell).contains(&x)
                    || !(1..=max_cell).contains(&z)
                    || fg[0].get(x, z) != Some(0)
                {
                    return Err("No room to turn here.");
                }
            }
        }
    }
    // aMR_SearchNextSituation: large collidable silhouettes may not scrape
    // another profile that also requests rotation clearance.
    if turn != 0 && p.check_rotation != 0 {
        let neighbours: &[(i32, i32)] = match actor.shape_type {
            ShapeType::TypeA => &[(0, -1), (-1, 0), (0, 1), (1, 0)],
            ShapeType::TypeC => &[
                (0, -1),
                (-1, 0),
                (-1, 1),
                (0, 2),
                (1, 2),
                (2, 1),
                (2, 0),
                (1, -1),
            ],
            _ => &[],
        };
        for &(dx, dz) in neighbours {
            let (x, z) = (actor.cell_x + dx, actor.cell_z + dz);
            if !(0..16).contains(&x) || !(0..16).contains(&z) {
                return Err("No room to turn here.");
            }
            if next.occupancy[0][z as usize][x as usize]
                .and_then(|id| next.actors[id as usize].as_ref())
                .and_then(|a| profile(a.name))
                .is_some_and(|p| p.check_rotation != 0)
            {
                return Err("No room to turn here.");
            }
        }
    }
    for &(id, x, z, dir, shape) in &edits {
        let a = next.actors[id].as_mut().ok_or("That furniture is gone.")?;
        let layer = a.layer;
        for &(dx, dz) in shape.unit_offsets().iter().take(shape.unit_count()) {
            let (cx, cz) = (x + dx, z + dz);
            if !(1..=max_cell).contains(&cx)
                || !(1..=max_cell).contains(&cz)
                || fg[layer].get(cx, cz) != Some(0)
            {
                return Err("There's no room to move.");
            }
        }
        a.cell_x = x;
        a.cell_z = z;
        a.shape_type = shape;
        a.angle_y = dir as u8 as f32 * 90.0;
        a.angle_y_target = a.angle_y;
        a.s_angle_y = ((dir as u16) << 14) as i16;
        let name = a.name;
        if layer == 1 && next.occupancy[0][z as usize][x as usize] != Some(slot as u16) {
            return Err("Clear the top first.");
        }
        if !next.occupy(layer, shape, x, z, id as u16) {
            return Err("There's no room to move.");
        }
        f::set_furniture_to_fg(&mut fg[layer], name, shape, dir, x, z, true);
    }
    // Save switches from the new owners rather than leaving bits at old cells.
    for (layer, fg_layer) in fg.iter_mut().enumerate() {
        f::save_switch_data(fg_layer, &|x, z| {
            next.actors
                .iter()
                .flatten()
                .find(|a| a.layer == layer && a.cell_x == x && a.cell_z == z)
                .map(|a| a.switch_bit)
        });
    }
    Ok((next, fg, edits.iter().map(|e| e.0).collect()))
}
