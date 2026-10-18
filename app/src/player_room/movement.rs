//! One grabbed furniture actor at a time. Persistent destinations commit before
//! animation, as in aMR_Place{Push,Pull,Kurukuru}Furniture; F9 drops presentation.
use super::*;
use crate::skeleton::{Animation, Skeleton};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Push,
    Pull,
    Left,
    Right,
}

struct Models {
    player: [Animation; 5],
    furniture: [Animation; 2],
}

pub(super) struct Grip {
    slot: usize,
    facing: Vec3,
}

struct Visual {
    slot: usize,
    from: Vec3,
    to: Vec3,
    yaw: f32,
    turn: f32,
}

struct Carried {
    item: u16,
    x: i32,
    z: i32,
    from: Vec3,
    to: Vec3,
    yaw: f32,
    angle: u8,
}

pub(super) struct Active {
    kind: Kind,
    ticks: f32,
    rotation: f32,
    pivot: Option<Vec3>,
    visuals: Vec<Visual>,
    carried: Vec<Carried>,
    player_from: Vec3,
    player_to: Vec3,
}

#[derive(Default)]
pub(super) struct Movement {
    pub(super) grip: Option<Grip>,
    pub(super) active: Option<Active>,
    models: Option<Models>,
    hold_frame: f32,
    direction_ticks: f32,
    last_dir: Vec3,
    rotation_ready: bool,
}

impl Movement {
    pub(super) fn reset(&mut self) {
        self.grip = None;
        self.active = None;
        self.direction_ticks = 0.0;
        self.last_dir = Vec3::ZERO;
        self.rotation_ready = true;
    }

    pub(super) fn load(&mut self, disc: Option<&GameDisc>) {
        self.models = disc.and_then(|disc| {
            let n = disc
                .catalog
                .get("cKF_bs_r_boy_1")
                .and_then(|s| Skeleton::read(&disc.rom, s.offset))?
                .joints
                .len();
            let read = |name: &str, n| {
                disc.catalog
                    .get(name)
                    .and_then(|a| Animation::read(&disc.rom, a.offset, n))
            };
            Some(Models {
                player: [
                    read("cKF_ba_r_ply_1_push1", n)?,
                    read("cKF_ba_r_ply_1_pull1", n)?,
                    read("cKF_ba_r_ply_1_Lturn1", n)?,
                    read("cKF_ba_r_ply_1_Rturn1", n)?,
                    read("cKF_ba_r_ply_1_hold_wait1", n)?,
                ],
                furniture: [
                    read("cKF_ba_r_furniture_push", 1)?,
                    read("cKF_ba_r_furniture_pull", 1)?,
                ],
            })
        });
        if disc.is_some() && self.models.is_none() {
            warn!("furniture movement unavailable: required original clips could not be decoded");
        }
    }

    pub(super) fn animation(&self) -> Option<(&Animation, f32, u8)> {
        let models = self.models.as_ref()?;
        if let Some(a) = &self.active {
            let i = a.kind as usize;
            Some((
                &models.player[i],
                (1.0 + a.ticks * 0.5).min(models.player[i].frames as f32),
                i as u8,
            ))
        } else {
            self.grip.as_ref()?;
            Some((&models.player[4], self.hold_frame.max(1.0), 4))
        }
    }

    fn progress(&self) -> f32 {
        let (Some(active), Some(models)) = (&self.active, &self.models) else {
            return 1.0;
        };
        match active.kind {
            Kind::Left | Kind::Right => active.rotation / 90.0,
            Kind::Push | Kind::Pull => {
                let anim = &models.furniture[active.kind as usize];
                let delay = if active.kind == Kind::Push { 3.0 } else { 2.0 };
                let frame = (1.0 + (active.ticks - delay).max(0.0) * 0.5).min(anim.frames as f32);
                let start = anim.pose(1.0).translation;
                let delta = anim.pose(anim.frames as f32).translation - start;
                if delta.length_squared() > 0.0 {
                    ((anim.pose(frame).translation - start).dot(delta) / delta.length_squared())
                        .clamp(0.0, 1.0)
                } else {
                    ((frame - 1.0) / (anim.frames.max(2) as f32 - 1.0)).clamp(0.0, 1.0)
                }
            }
        }
    }
}

fn center(a: &ftr::FurnitureActor) -> Vec3 {
    let c = if a.original_shape_type == ShapeType::TypeC {
        1.0
    } else {
        0.5
    };
    corner() + Vec3::new(a.cell_x as f32 + c, 0.0, a.cell_z as f32 + c)
}

fn blocked(runtime: &MyRoomRuntime, profiles: &HashMap<u16, FurnitureProfile>, pos: Vec3) -> bool {
    // f32 rounding at the distant interior origin can place a stopped player
    // fractionally inside house collision's contact boundary (0.08 game units).
    const RADIUS: f32 = 0.398;
    let p = pos - corner();
    if !crate::house::player_room_walkable(pos, RADIUS) {
        return true;
    }
    runtime
        .actors
        .iter()
        .flatten()
        .filter(|a| a.layer == 0)
        .any(|a| {
            if profiles
                .get(&a.name)
                .is_some_and(|p| p.interaction_type & ftr::interaction::NO_COLLISION != 0)
            {
                return false;
            }
            a.shape_type
                .unit_offsets()
                .iter()
                .take(a.shape_type.unit_count())
                .any(|&(dx, dz)| {
                    let (x, z) = ((a.cell_x + dx) as f32, (a.cell_z + dz) as f32);
                    p.x + RADIUS > x + 0.05
                        && p.x - RADIUS < x + 0.95
                        && p.z + RADIUS > z + 0.05
                        && p.z - RADIUS < z + 0.95
                })
        })
}

type Relocation = (
    MyRoomRuntime,
    [RoomLayer; 4],
    Vec<usize>,
    Vec<Carried>,
    loose::Angles,
);

/// aMR_RequestItemToFitFurniture / UnFit: collect relative surface cells on
/// copies, reuse the native actor move, then restore goods at the destination.
fn relocate(
    room: &PlayerRoom,
    slot: usize,
    target: (i32, i32, Direction),
    long_turn: Option<&rustimal_logic::furniture_motion::LongTurn>,
) -> Result<Relocation, &'static str> {
    let actor = room.runtime.actors[slot]
        .as_ref()
        .ok_or("That furniture is gone.")?;
    let mut layers = room.layers.clone();
    let mut carried = Vec::new();
    let mut angles = room.loose_angles;
    let turns = (target.2 as i32 - (actor.angle_y / 90.0).round() as i32).rem_euclid(4) as u8;
    let pivot = if actor.original_shape_type == ShapeType::TypeC {
        1.0
    } else {
        0.5
    };
    for &(dx, dz) in actor
        .shape_type
        .unit_offsets()
        .iter()
        .take(actor.shape_type.unit_count())
    {
        let (x, z) = (actor.cell_x + dx, actor.cell_z + dz);
        let item = layers[1].get(x, z).ok_or("Clear the top first.")?;
        if item == 0 || room.runtime.occupancy[1][z as usize][x as usize].is_some() {
            continue;
        }
        if !loose::supports(item)
            || !room.profiles.get(&actor.name).is_some_and(|p| {
                p.set_type == SetType::Surface && !ftr::is_storage(p.interaction_type)
            })
        {
            return Err("Clear the top first.");
        }
        layers[1].set(x, z, 0);
        angles[z as usize][x as usize] = 0;
        let from = corner() + Vec3::new(x as f32 + 0.5, 0.0, z as f32 + 0.5);
        let mut offset = Vec2::new(dx as f32 + 0.5 - pivot, dz as f32 + 0.5 - pivot);
        for _ in 0..turns {
            offset = Vec2::new(offset.y, -offset.x);
        }
        let (x, z) = (
            target.0 + (offset.x + pivot - 0.5).round() as i32,
            target.1 + (offset.y + pivot - 0.5).round() as i32,
        );
        let old_angle =
            room.loose_angles[(actor.cell_z + dz) as usize][(actor.cell_x + dx) as usize];
        carried.push(Carried {
            item,
            x,
            z,
            from,
            to: corner() + Vec3::new(x as f32 + 0.5, 0.0, z as f32 + 0.5),
            yaw: old_angle as f32 * std::f32::consts::FRAC_PI_2,
            angle: (old_angle + turns) & 3,
        });
    }
    let (next, mut layers, ids) = rustimal_logic::furniture_motion::relocate(
        &room.runtime,
        &layers,
        slot,
        target,
        long_turn,
        4,
        &|name| room.profiles.get(&name).cloned(),
    )?;
    for piece in &carried {
        if layers[1].get(piece.x, piece.z) != Some(0)
            || next.occupancy[1][piece.z as usize][piece.x as usize].is_some()
            || next.occupancy[0][piece.z as usize][piece.x as usize] != Some(slot as u16)
        {
            return Err("There's no room to move.");
        }
        layers[1].set(piece.x, piece.z, piece.item);
        angles[piece.z as usize][piece.x as usize] = piece.angle;
    }
    Ok((next, layers, ids, carried, angles))
}

#[allow(clippy::too_many_arguments)] // One atomic room/player/input transaction.
pub(super) fn advance(
    mut room: ResMut<PlayerRoom>,
    current: Res<CurrentRoom>,
    state: Res<State<HouseState>>,
    tool: Res<crate::ecology::EquippedTool>,
    time: Res<Time>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    mut player: Query<
        (
            &mut Transform,
            &mut Facing,
            &mut crate::locomotion::Locomotion,
        ),
        With<Player>,
    >,
    mut dialogue: ResMut<DialogueState>,
) {
    if !crate::house::is_player_room(&current, &state) || room.movement.models.is_none() {
        room.movement.reset();
        return;
    }
    // GameplaySet stops while paused; this also handles a modal taking focus.
    if !matches!(*focus, InputFocus::World | InputFocus::Furniture) {
        return;
    }
    let Ok((mut transform, mut facing, mut loco)) = player.single_mut() else {
        return;
    };
    let ticks = time.delta_secs() * 60.0;
    if room.presentation.busy() || room.loose_presentation.busy() || room.storage.busy() {
        *loco = crate::locomotion::Locomotion::default();
        *focus = InputFocus::Furniture;
        return;
    }
    if room.movement.active.is_some() {
        *loco = crate::locomotion::Locomotion::default();
        let Some(active) = room.movement.active.as_mut() else {
            return;
        };
        active.ticks += ticks;
        if matches!(active.kind, Kind::Left | Kind::Right) && active.ticks > 3.0 {
            // aMR_GetEllipseSpeed(5.6,90), bounded substeps preserve the 60 Hz curve.
            let mut remaining = ticks.min((active.ticks - 3.0).max(0.0));
            while remaining > 0.0 {
                let step = remaining.min(1.0);
                let delta = 90.0 - active.rotation;
                let speed = 5.6 * (1.0 - (delta - 90.0).powi(2) / 8100.0).max(0.0).sqrt();
                active.rotation = if delta <= speed * step + 1.0 {
                    90.0
                } else {
                    active.rotation + speed * step
                };
                remaining -= step;
            }
        }
        let fraction = room.movement.progress();
        if let Some(active) = room.movement.active.as_ref() {
            transform.translation = active.player_from.lerp(active.player_to, fraction);
            let models = room.movement.models.as_ref();
            let end = models.map_or(1.0, |m| {
                (m.player[active.kind as usize].frames as f32 - 1.0) * 2.0
            });
            if fraction >= 1.0 && active.ticks >= end {
                transform.translation = active.player_to;
                room.movement.active = None;
            }
        }
        *focus = InputFocus::Furniture;
        return;
    }
    if !actions.pressed(Action::FurnitureGrab)
        || actions.consume(Action::Cancel)
        || *tool != crate::ecology::EquippedTool::None
    {
        room.movement.reset();
        *focus = InputFocus::World;
        return;
    }
    if room.movement.grip.is_none() {
        let dir = if facing.dir.x.abs() > facing.dir.z.abs() {
            Vec3::X * facing.dir.x.signum()
        } else {
            Vec3::Z * facing.dir.z.signum()
        };
        let target = room
            .runtime
            .actors
            .iter()
            .enumerate()
            .filter_map(|(slot, a)| {
                let a = a.as_ref()?;
                if a.layer != 0 {
                    return None;
                }
                a.shape_type
                    .unit_offsets()
                    .iter()
                    .take(a.shape_type.unit_count())
                    .filter_map(|&(dx, dz)| {
                        let pos = corner()
                            + Vec3::new(
                                (a.cell_x + dx) as f32 + 0.5,
                                0.0,
                                (a.cell_z + dz) as f32 + 0.5,
                            );
                        let delta = (pos - transform.translation).with_y(0.0);
                        let distance = delta.dot(dir);
                        (distance > 0.6
                            && distance <= 1.15
                            && (delta - dir * distance).length() < 0.4)
                            .then_some((slot, distance))
                    })
                    .min_by(|a, b| a.1.total_cmp(&b.1))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1));
        let Some((slot, _)) = target else { return };
        info!("furniture grip: actor {slot}");
        room.movement.grip = Some(Grip { slot, facing: dir });
        room.movement.hold_frame = 1.0;
        room.movement.rotation_ready = true;
        facing.dir = dir;
    }
    *focus = InputFocus::Furniture;
    *loco = crate::locomotion::Locomotion::default();
    if let Some(models) = &room.movement.models {
        let end = models.player[4].frames.max(2) as f32;
        room.movement.hold_frame =
            1.0 + (room.movement.hold_frame - 1.0 + ticks * 0.5).rem_euclid(end - 1.0);
    }
    let dir = actions.move_dir();
    if dir == Vec3::ZERO {
        room.movement.direction_ticks = 0.0;
        room.movement.rotation_ready = true;
        room.movement.last_dir = dir;
        return;
    }
    if dir.x != 0.0 && dir.z != 0.0 {
        return;
    }
    if dir != room.movement.last_dir {
        room.movement.direction_ticks = 0.0;
        room.movement.last_dir = dir;
    }
    room.movement.direction_ticks += ticks;
    if room.movement.direction_ticks <= 16.0 {
        return;
    }
    let Some(grip) = room.movement.grip.as_ref() else {
        return;
    };
    let (slot, front) = (grip.slot, grip.facing);
    let dot = dir.dot(front);
    let kind = if dot > 0.5 {
        Kind::Push
    } else if dot < -0.5 {
        Kind::Pull
    } else if dir.dot(Vec3::new(front.z, 0.0, -front.x)) > 0.0 {
        Kind::Left
    } else {
        Kind::Right
    };
    let rotating = matches!(kind, Kind::Left | Kind::Right);
    if rotating && !room.movement.rotation_ready {
        return;
    }
    room.movement.direction_ticks = 0.0;
    room.movement.rotation_ready = false;
    let Some(old) = room.runtime.actors[slot].as_ref() else {
        room.movement.reset();
        return;
    };
    let old_dir = (old.angle_y / 90.0).round() as u8;
    let long_turn = if rotating && old.shape_type.unit_count() == 2 {
        let outward = if front.x > 0.5 {
            Direction::West
        } else if front.x < -0.5 {
            Direction::East
        } else if front.z > 0.5 {
            Direction::North
        } else {
            Direction::South
        };
        let (dx, dz) = old.shape_type.unit_offsets()[1];
        let middle = corner()
            + Vec3::new(
                old.cell_x as f32 + 0.5 + dx as f32 * 0.5,
                0.0,
                old.cell_z as f32 + 0.5 + dz as f32 * 0.5,
            );
        // aMR_ContactRange: directed edges run W->E, S->N, E->W,
        // N->S for outward up/left/down/right, respectively.
        let second_half = match outward {
            Direction::North => transform.translation.x >= middle.x,
            Direction::West => transform.translation.z <= middle.z,
            Direction::South => transform.translation.x <= middle.x,
            Direction::East => transform.translation.z >= middle.z,
        };
        rustimal_logic::furniture_motion::long_turn(old, outward, second_half, kind == Kind::Left)
    } else {
        None
    };
    let target = if let Some(plan) = &long_turn {
        plan.target
    } else if rotating {
        (
            old.cell_x,
            old.cell_z,
            direction(old_dir.wrapping_add(if kind == Kind::Left { 1 } else { 3 })),
        )
    } else {
        (
            old.cell_x + dir.x as i32,
            old.cell_z + dir.z as i32,
            direction(old_dir),
        )
    };
    let pivot = long_turn
        .as_ref()
        .map(|p| corner() + Vec3::new(p.pivot.0 as f32 + 0.5, 0.0, p.pivot.1 as f32 + 0.5));
    let result = relocate(&room, slot, target, long_turn.as_ref());
    let result = result.and_then(|(next, layers, ids, carried, angles)| {
        let destination = transform.translation + if rotating { Vec3::ZERO } else { dir };
        // Check the complete player path against all non-moving actors and the
        // room walls; check the destination against the committed furniture.
        let mut static_room = room.runtime.clone();
        for &id in &ids {
            static_room.actors[id] = None;
        }
        if rotating {
            let before = room.runtime.actors[slot]
                .as_ref()
                .ok_or("That furniture is gone.")?;
            let after = next.actors[slot]
                .as_ref()
                .ok_or("That furniture is gone.")?;
            if before.shape_type.unit_count() == 2 {
                let cells = before.shape_type.unit_offsets()[..2]
                    .iter()
                    .map(|&(x, z)| (before.cell_x + x, before.cell_z + z))
                    .chain(
                        after.shape_type.unit_offsets()[..2]
                            .iter()
                            .map(|&(x, z)| (after.cell_x + x, after.cell_z + z)),
                    )
                    .collect::<Vec<_>>();
                let min_x = cells.iter().map(|p| p.0).min().ok_or("Can't turn there.")? as f32;
                let max_x =
                    cells.iter().map(|p| p.0).max().ok_or("Can't turn there.")? as f32 + 1.0;
                let min_z = cells.iter().map(|p| p.1).min().ok_or("Can't turn there.")? as f32;
                let max_z =
                    cells.iter().map(|p| p.1).max().ok_or("Can't turn there.")? as f32 + 1.0;
                let p = transform.translation - corner();
                if p.x + 0.398 > min_x + 0.05
                    && p.x - 0.398 < max_x - 0.05
                    && p.z + 0.398 > min_z + 0.05
                    && p.z - 0.398 < max_z - 0.05
                {
                    return Err("Step clear before turning.");
                }
            }
        }
        if (0..=16).any(|i| {
            blocked(
                &static_room,
                &room.profiles,
                transform.translation.lerp(destination, i as f32 / 16.0),
            )
        }) || blocked(&next, &room.profiles, destination)
        {
            return Err("No room to move there.");
        }
        let visuals = ids
            .iter()
            .filter_map(|&id| {
                let old = room.runtime.actors[id].as_ref()?;
                let new = next.actors[id].as_ref()?;
                Some(Visual {
                    slot: id,
                    from: center(old),
                    to: center(new),
                    yaw: old.angle_y.to_radians(),
                    turn: if kind == Kind::Left {
                        std::f32::consts::FRAC_PI_2
                    } else if kind == Kind::Right {
                        -std::f32::consts::FRAC_PI_2
                    } else {
                        0.0
                    },
                })
            })
            .collect();
        room.runtime = next;
        room.layers = layers;
        room.loose_angles = angles;
        room.dirty = true;
        room.movement.active = Some(Active {
            kind,
            ticks: 0.0,
            rotation: 0.0,
            pivot,
            visuals,
            carried,
            player_from: transform.translation,
            player_to: destination,
        });
        info!(
            "furniture motion: actor {slot} to ({},{}) rotation {}",
            target.0, target.1, target.2 as u8
        );
        Ok(())
    });
    if let Err(message) = result {
        room.movement.reset();
        notice(&mut dialogue, &mut room, &mut focus, message);
    }
}

pub(super) fn animate_roots(
    room: Res<PlayerRoom>,
    mut pieces: Query<(&RoomPiece, &mut Transform), Without<loose::Piece>>,
    mut goods: Query<(&loose::Piece, &mut Transform), Without<RoomPiece>>,
) {
    let progress = room.movement.progress();
    for (piece, mut transform) in &mut pieces {
        let Some(actor) = room.runtime.actors.get(piece.slot).and_then(Option::as_ref) else {
            continue;
        };
        let target = center(actor);
        let (position, yaw) = room
            .movement
            .active
            .as_ref()
            .and_then(|a| {
                let visual = a.visuals.iter().find(|v| v.slot == piece.slot)?;
                let parent = a.visuals.first()?;
                let position = if visual.turn != 0.0 {
                    let pivot = a.pivot.unwrap_or(parent.from);
                    let pivot_to = a.pivot.unwrap_or(parent.to);
                    pivot.lerp(pivot_to, progress)
                        + Quat::from_rotation_y(visual.turn * progress) * (visual.from - pivot)
                } else {
                    visual.from.lerp(visual.to, progress)
                };
                Some((position, visual.yaw + visual.turn * progress))
            })
            .unwrap_or((target, actor.angle_y.to_radians()));
        transform.translation.x = position.x;
        transform.translation.z = position.z;
        transform.rotation = Quat::from_rotation_y(yaw);
    }
    for (piece, mut transform) in &mut goods {
        let target = corner() + Vec3::new(piece.x as f32 + 0.5, 0.0, piece.z as f32 + 0.5);
        let (position, yaw) = room
            .movement
            .active
            .as_ref()
            .and_then(|active| {
                let carried = active.carried.iter().find(|v| {
                    piece.layer == 1 && v.item == piece.item && v.x == piece.x && v.z == piece.z
                })?;
                let parent = active.visuals.first()?;
                let position = if parent.turn != 0.0 {
                    let pivot = active.pivot.unwrap_or(parent.from);
                    let pivot_to = active.pivot.unwrap_or(parent.to);
                    pivot.lerp(pivot_to, progress)
                        + Quat::from_rotation_y(parent.turn * progress) * (carried.from - pivot)
                } else {
                    carried.from.lerp(carried.to, progress)
                };
                Some((position, carried.yaw + parent.turn * progress))
            })
            .unwrap_or((
                target,
                room.loose_angles[piece.z as usize][piece.x as usize] as f32
                    * std::f32::consts::FRAC_PI_2,
            ));
        transform.translation.x = position.x;
        transform.translation.z = position.z;
        transform.rotation = Quat::from_rotation_y(if piece.rotates { yaw } else { 0.0 });
    }
}
