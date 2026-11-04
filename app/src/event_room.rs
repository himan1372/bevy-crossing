//! Generic event rooms: a small disc room (a field BG model, drawn by [`IndoorRoom`]) behind a door of some structure
//! outside, the pattern `broker_tent.rs` first used for Redd's tent.
//!
//! The structure's owner registers a [`Door`] for its [`RoomKind`] while the structure stands (and clears it after).
//! This module then builds the room from the disc once, carries the player in when they stand at the door point facing
//! it (walking, or E without a tool), lets them walk about the room, and carries them out when they walk into the
//! entrance facing out (or press E there), to the door's exit point (`ShopReturnPos`). While inside,
//! `ShopLocation::EventRoom` holds and [`EventRooms::current`] names the room.
//!
//! The room's own furnishing (villagers, props) is its owner's: the owner reads `current()` and spawns / despawns by it.
//!
//! Not built: the wipe transitions between scenes, the rooms' lights and effects, and `broker_tent.rs` on this path.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::game_disc::GameDisc;
use crate::indoor_room::IndoorRoom;
use crate::input::{Action, Actions, InputFocus};
use crate::interaction::Facing;
use crate::locomotion::Locomotion;
use crate::n64_material::N64Material;
use crate::player::Player;
use crate::shop::{ShopLocation, ShopReturnPos};
use crate::states::GameplaySet;

const UNIT: f32 = 40.0;

/// The rooms this path knows.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum RoomKind {
    /// The kamakura's room (`rom_kamakura_model`).
    Kamakura,
}

/// A room's fixed data (`<scene>_info`, `<SCENE>_player_data`).
struct Spec {
    name: &'static str,
    /// Where the room is drawn, away from the town and the other rooms (cells).
    origin: Vec3,
    models: (&'static str, Option<&'static str>),
    /// Where the player stands on coming in, room-local game units.
    entry: Vec2,
}

impl RoomKind {
    fn spec(self) -> Spec {
        match self {
            RoomKind::Kamakura => Spec {
                name: "kamakura room",
                origin: Vec3::new(0.0, 0.0, -3500.0),
                models: ("rom_kamakura_model", Some("rom_kamakura_modelT")),
                entry: Vec2::new(160.0, 260.0),
            },
        }
    }

    /// Room-local game units to a point in the world.
    pub(crate) fn local(self, p: Vec2) -> Vec3 {
        self.spec().origin + Vec3::new(p.x / UNIT, 0.0, p.y / UNIT)
    }
}

/// A structure's door while it stands.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Door {
    /// Where the player must stand (world, cells) and within how far (cells).
    pub check: Vec3,
    pub radius: f32,
    /// Where the player comes out (world), the ground height taken from the player.
    pub exit: Vec3,
}

#[derive(Resource, Default)]
pub(crate) struct EventRooms {
    rooms: HashMap<RoomKind, IndoorRoom>,
    doors: HashMap<RoomKind, Door>,
    current: Option<RoomKind>,
    /// Seconds since the player came in.
    inside_for: Option<f32>,
}

impl EventRooms {
    pub(crate) fn set_door(&mut self, kind: RoomKind, door: Door) {
        self.doors.insert(kind, door);
    }

    pub(crate) fn clear_door(&mut self, kind: RoomKind) {
        self.doors.remove(&kind);
    }

    /// The room the player is in, if any.
    pub(crate) fn current(&self) -> Option<RoomKind> {
        self.current
    }
}

/// Builds a room once, when its door first stands.
fn build(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    mut rooms: ResMut<EventRooms>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
    mut failed: Local<Vec<RoomKind>>,
) {
    let Some(disc) = disc else { return };
    let kinds: Vec<RoomKind> = rooms.doors.keys().copied().collect();
    for kind in kinds {
        if rooms.rooms.contains_key(&kind) || failed.contains(&kind) {
            continue;
        }
        let spec = kind.spec();
        match IndoorRoom::build(
            &mut commands,
            &disc,
            spec.name,
            spec.origin,
            spec.models,
            (&mut meshes, &mut materials, &mut images),
        ) {
            Some(room) => {
                rooms.rooms.insert(kind, room);
            }
            None => {
                warn!("The {} is missing from the disc", spec.name);
                failed.push(kind);
            }
        }
    }
}

/// Standing at a door facing it, walking or pressing E.
#[allow(clippy::too_many_arguments)]
fn enter(
    mut commands: Commands,
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    tool: Res<crate::ecology::EquippedTool>,
    mut rooms: ResMut<EventRooms>,
    mut next: ResMut<NextState<ShopLocation>>,
    mut player: Query<(&mut Transform, &mut Facing, &Locomotion), With<Player>>,
) {
    if *state.get() != ShopLocation::Town
        || *house.get() != crate::house::HouseState::Town
        || !focus.is_world()
    {
        return;
    }
    let Ok((mut transform, mut facing, loco)) = player.single_mut() else {
        return;
    };
    let pressed =
        actions.just_pressed(Action::Interact) && *tool == crate::ecology::EquippedTool::None;
    let walking = loco.state() != rustimal_logic::player_move::LocomotionState::Wait;
    if !(walking || pressed) {
        return;
    }
    // Facing between 135 and 225 degrees, the door's side.
    let yaw = facing.dir.x.atan2(facing.dir.z).to_degrees().abs();
    if yaw <= 135.0 {
        return;
    }
    let found = rooms.doors.iter().find(|(kind, door)| {
        rooms.rooms.contains_key(kind)
            && (transform.translation - door.check).with_y(0.0).length() < door.radius
    });
    let Some((&kind, &door)) = found else { return };
    if pressed {
        actions.consume(Action::Interact);
    }
    commands.insert_resource(ShopReturnPos(door.exit.with_y(transform.translation.y)));
    transform.translation = kind.local(kind.spec().entry) + Vec3::Y;
    facing.dir = Vec3::NEG_Z;
    rooms.current = Some(kind);
    rooms.inside_for = Some(0.0);
    crate::sound::door_sound(None, true);
    next.set(ShopLocation::EventRoom);
    info!("entered the {}", kind.spec().name);
}

/// Walking south into the entrance (or E there) leaves.
fn leave(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    mut next: ResMut<NextState<ShopLocation>>,
    return_pos: Option<Res<ShopReturnPos>>,
    mut rooms: ResMut<EventRooms>,
    mut player: Query<(&mut Transform, &mut Facing), With<Player>>,
) {
    if *state.get() != ShopLocation::EventRoom || !focus.is_world() {
        return;
    }
    let (Some(kind), Ok((mut transform, mut facing))) = (rooms.current, player.single_mut()) else {
        return;
    };
    let near = (transform.translation - kind.local(kind.spec().entry))
        .with_y(0.0)
        .length()
        < 0.6;
    let into = actions.pressed(Action::MoveDown) || actions.just_pressed(Action::Interact);
    if !(near && facing.dir.z > 0.7 && into) {
        return;
    }
    actions.consume(Action::Interact);
    if let Some(r) = return_pos {
        transform.translation = r.0;
    }
    facing.dir = Vec3::Z;
    rooms.current = None;
    rooms.inside_for = None;
    crate::sound::door_sound(None, false);
    next.set(ShopLocation::Town);
    info!("left the {}", kind.spec().name);
}

/// Walking about the room.
fn movement(
    actions: Res<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    time: Res<Time>,
    mut rooms: ResMut<EventRooms>,
    mut player: Query<(&mut Transform, &mut Facing, &mut Locomotion), With<Player>>,
) {
    if *state.get() != ShopLocation::EventRoom {
        return;
    }
    let dt = time.delta_secs();
    if let Some(t) = rooms.inside_for.as_mut() {
        *t += dt;
    }
    if !focus.is_world() {
        return;
    }
    let (Some(kind), Ok((mut transform, mut facing, mut loco))) =
        (rooms.current, player.single_mut())
    else {
        return;
    };
    if let Some(room) = rooms.rooms.get(&kind) {
        room.walk(&actions, dt, (&mut transform, &mut facing, &mut loco));
    }
}

pub struct EventRoomPlugin;

impl Plugin for EventRoomPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventRooms>().add_systems(
            Update,
            (build, enter, leave, movement).chain().in_set(GameplaySet),
        );
    }
}
