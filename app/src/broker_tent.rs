//! Crazy Redd's tent (`ac_br_shop.c`, `data/scene/broker_shop.c`): the structure on a lot while his event runs, and
//! the room inside it (`grd_yamishop_model`, field `mFI_FIELD_ROOM_BROKER_SHOP`).
//!
//! Outside (`ac_br_shop_move.c_inc`): the tent (`obj_{s,w}_yamishop`, three cells square) stands where
//! `broker_start` puts it (`make_FG_somewhere_lot4sale`) and Redd two cells south of it (`unit.z + 2`). The player
//! walks in standing at (x, z + 60 game units) within sqrt(800) facing north (E there works too).
//!
//! Inside: the player starts at (100, 200) game units; Redd (`SP_NPC_EV_BROKER2`) at unit (4, 4); the three pieces of the
//! event on display at units (2, 2), (4, 2) and (2, 4) (`aBD_SetFurnitureFg`). The talks are in `redd.rs`.
//! Walking south into the entrance (or E there) asks Redd for his goodbye, then leaves for (x, z + 100) outside.
//!
//! Not built: the tent's ground offsets and collision outside, its window glass and light, the shadow, Redd following the
//! player about the room, the sold sign on a sold slot, and the tent's music (the town's plays).

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;

use crate::game_disc::GameDisc;
use crate::indoor_room::IndoorRoom;
use crate::input::{Action, Actions, InputFocus};
use crate::interaction::Facing;
use crate::locomotion::Locomotion;
use crate::n64_material::N64Material;
use crate::player::Player;
use crate::redd::BrokerEvent;
use crate::shop::{ShopLocation, ShopReturnPos};
use crate::states::GameplaySet;
use crate::town::Town;

/// The room's origin (cells), away from the town and the other rooms.
pub const TENT_ORIGIN: Vec3 = Vec3::new(0.0, 0.0, -3000.0);
const UNIT: f32 = 40.0;
/// `BROKER_SHOP_player_data`: (100, 0, 200) game units.
const ENTRY: Vec2 = Vec2::new(100.0, 200.0);
/// The pieces' units (`aBD_SetFurnitureFg`'s `set_data`).
pub const DISPLAY_UNITS: [(i32, i32); 3] = [(2, 2), (4, 2), (2, 4)];
/// `broker_shop_actable`: Redd at unit (4, 4).
const REDD_UNIT: (i32, i32) = (4, 4);
/// Seconds inside before Redd says hello (the door demo).
pub const ARRIVAL: f32 = 0.6;

/// The tent now: where it stands outside, the room, and what the flow between the two needs.
#[derive(Resource, Default)]
pub struct Tent {
    /// The tent's centre outside, while the event runs.
    pub at: Option<Vec3>,
    pub room: Option<IndoorRoom>,
    built: bool,
    /// Seconds since the player came in (`None`: not inside).
    pub inside_for: Option<f32>,
    /// The player asked to leave: Redd says goodbye first.
    pub exit_request: bool,
    /// The goodbye is over: leave now.
    pub exit_now: bool,
    /// Redd has gone in after the outside talk (he is back with the next time the tent is left).
    pub redd_inside: bool,
}

#[derive(Component)]
struct TentExterior {
    day: i64,
}

#[derive(Component)]
struct TentDisplay;

/// Redd inside the tent (`SP_NPC_EV_BROKER2`).
#[derive(Component)]
pub struct Redd2;

/// Room-local game units to a point in the world.
pub fn local(p: Vec2) -> Vec3 {
    TENT_ORIGIN + Vec3::new(p.x / UNIT, 0.0, p.y / UNIT)
}

/// A clear 3 x 3 lot with the approach to its south clear: the nearest to the day's pick
/// (`make_FG_somewhere_lot4sale`), as the tent's centre.
pub fn lot(town: &Town, items: &crate::field_items::FieldItems, day: i64) -> Option<Vec3> {
    let pick = crate::joan::spot(town, items, day)?;
    let clear = |p: Vec3| town.is_clear_ground(p) && !town.in_retail_obstacle(p);
    let fits = |g: Vec3| {
        (-1..=1).all(|dz| (-1..=1).all(|dx| clear(g + Vec3::new(dx as f32, 0.0, dz as f32))))
            && (2..=3).all(|dz| clear(g + Vec3::new(0.0, 0.0, dz as f32)))
    };
    let mut best: Option<(f32, Vec3)> = None;
    for dz in -10i32..=10 {
        for dx in -10i32..=10 {
            let g = pick + Vec3::new(dx as f32, 0.0, dz as f32);
            let d = (g - pick).length();
            if fits(g) && best.is_none_or(|(b, _)| d < b) {
                best = Some((d, g));
            }
        }
    }
    best.map(|(_, g)| g.with_y(town.ground_y(g)))
}

/// The tent outside while the event runs; gone after.
#[allow(clippy::too_many_arguments)]
fn spawn_tent(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    town: Res<Town>,
    items: Option<Res<crate::field_items::FieldItems>>,
    today: Res<crate::event_today::TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &TentExterior)>,
    mut tent: ResMut<Tent>,
    (mut meshes, mut materials, mut images): (
        ResMut<Assets<Mesh>>,
        ResMut<Assets<N64Material>>,
        ResMut<Assets<Image>>,
    ),
) {
    let now = crate::clock::now();
    // Until 18:00 of the next day he is still the previous day's visitor.
    let day = if now.hour < 18 {
        now.day_number - 1
    } else {
        now.day_number
    };
    let on = today.is_active(ev::BROKER_SALE, now.hour);
    let in_town =
        *state.get() == ShopLocation::Town && *house.get() == crate::house::HouseState::Town;
    for (entity, e) in &existing {
        if !on || e.day != day {
            commands.entity(entity).despawn();
            tent.at = None;
        }
    }
    if !on {
        tent.at = None;
        return;
    }
    if !in_town || !existing.is_empty() {
        return;
    }
    let (Some(disc), Some(items)) = (disc, items) else {
        return;
    };
    let Some(at) = lot(&town, &items, day) else {
        return;
    };
    let winter = crate::clock::now().month == 12 || crate::clock::now().month <= 2;
    let parts = crate::disc_town::broker_tent_model(
        &disc,
        winter,
        (&mut meshes, &mut materials, &mut images),
    );
    if parts.is_empty() {
        return;
    }
    let root = commands
        .spawn((
            TentExterior { day },
            Name::new("Redd's tent"),
            Transform::from_translation(at),
            Visibility::default(),
        ))
        .id();
    for (mesh, material, local) in parts {
        commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), local, ChildOf(root)));
    }
    tent.at = Some(at);
    tent.redd_inside = false;
    info!("Redd's tent stands at {at:?}");
}

/// Builds the room once (`grd_yamishop_model`).
fn build_room(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    mut tent: ResMut<Tent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(disc) = disc else { return };
    if tent.built {
        return;
    }
    tent.built = true;
    tent.room = IndoorRoom::build(
        &mut commands,
        &disc,
        "broker tent room",
        TENT_ORIGIN,
        ("grd_yamishop_model", Some("grd_yamishop_modelT")),
        (&mut meshes, &mut materials, &mut images),
    );
    if tent.room.is_none() {
        warn!("Redd's tent room is missing from the disc");
    }
}

/// `aBRS_check_player` / `aBRS_open_wait`: standing at the tent's door facing north, walking or pressing E.
#[allow(clippy::too_many_arguments)]
fn enter(
    mut commands: Commands,
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    tool: Res<crate::ecology::EquippedTool>,
    mut tent: ResMut<Tent>,
    mut next: ResMut<NextState<ShopLocation>>,
    mut player: Query<(&mut Transform, &mut Facing, &Locomotion), With<Player>>,
    mut flow: ResMut<crate::redd::Talk>,
) {
    if *state.get() != ShopLocation::Town
        || *house.get() != crate::house::HouseState::Town
        || !focus.is_world()
    {
        return;
    }
    let (Some(at), Ok((mut transform, mut facing, loco))) = (tent.at, player.single_mut()) else {
        return;
    };
    if tent.room.is_none() {
        return;
    }
    // Within sqrt(800) game units of (x, z + 60), facing between 0x6000 and 0xA000 (north).
    let door = at + Vec3::new(0.0, 0.0, 60.0 / UNIT);
    let d = (transform.translation - door).with_y(0.0) * UNIT;
    let yaw = facing.dir.x.atan2(facing.dir.z).to_degrees().abs();
    let facing_ok = yaw > 135.0;
    let pressed =
        actions.just_pressed(Action::Interact) && *tool == crate::ecology::EquippedTool::None;
    let walking = loco.state() != rustimal_logic::player_move::LocomotionState::Wait;
    if d.length_squared() >= 800.0 || !facing_ok || !(walking || pressed) {
        return;
    }
    if pressed {
        actions.consume(Action::Interact);
    }
    // aBRS_rewrite_out_data: out at (x, z + 100).
    commands.insert_resource(ShopReturnPos(
        (at + Vec3::new(0.0, 0.0, 100.0 / UNIT)).with_y(transform.translation.y),
    ));
    transform.translation = local(ENTRY) + Vec3::Y;
    facing.dir = Vec3::NEG_Z;
    tent.inside_for = Some(0.0);
    tent.exit_request = false;
    tent.exit_now = false;
    flow.reset_visit();
    crate::sound::door_sound(None, true);
    next.set(ShopLocation::BrokerTent);
    info!("entered Redd's tent");
}

/// Walking south into the entrance (or E there) asks for the goodbye; when it is said the player leaves.
#[allow(clippy::too_many_arguments)]
fn leave(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    mut next: ResMut<NextState<ShopLocation>>,
    return_pos: Option<Res<ShopReturnPos>>,
    mut tent: ResMut<Tent>,
    mut player: Query<(&mut Transform, &mut Facing), With<Player>>,
) {
    if *state.get() != ShopLocation::BrokerTent {
        return;
    }
    let Ok((mut transform, mut facing)) = player.single_mut() else {
        return;
    };
    if tent.exit_now {
        tent.exit_now = false;
        tent.exit_request = false;
        tent.inside_for = None;
        // The tent's own scene reloads: Redd is outside again.
        tent.redd_inside = false;
        if let Some(r) = return_pos {
            transform.translation = r.0;
        }
        facing.dir = Vec3::Z;
        crate::sound::door_sound(None, false);
        next.set(ShopLocation::Town);
        info!("left Redd's tent");
        return;
    }
    if !focus.is_world() || tent.exit_request {
        return;
    }
    let near = (transform.translation - local(ENTRY)).with_y(0.0).length() < 0.6;
    let into = actions.pressed(Action::MoveDown) || actions.just_pressed(Action::Interact);
    if near && facing.dir.z > 0.7 && into {
        actions.consume(Action::Interact);
        tent.exit_request = true;
    }
}

/// Walking about the room.
fn movement(
    actions: Res<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    time: Res<Time>,
    mut tent: ResMut<Tent>,
    mut player: Query<(&mut Transform, &mut Facing, &mut Locomotion), With<Player>>,
) {
    if *state.get() != ShopLocation::BrokerTent {
        return;
    }
    let dt = time.delta_secs();
    if let Some(t) = tent.inside_for.as_mut() {
        *t += dt;
    }
    if !focus.is_world() {
        return;
    }
    let Ok((mut transform, mut facing, mut loco)) = player.single_mut() else {
        return;
    };
    if let Some(room) = tent.room.as_ref() {
        room.walk(&actions, dt, (&mut transform, &mut facing, &mut loco));
    }
}

/// Redd inside, and the pieces on display; both go when the player leaves.
#[allow(clippy::too_many_arguments)]
fn furnish(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    event: Res<BrokerEvent>,
    mut tent: ResMut<Tent>,
    redd: Query<Entity, With<Redd2>>,
    shown: Query<Entity, With<TentDisplay>>,
    mut changed: Local<Option<(bool, [u16; 3])>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let inside = *state.get() == ShopLocation::BrokerTent;
    // Redd inside is one actor for the whole visit: only coming in and going out change him (the dialogue holds him).
    if !inside {
        for e in &redd {
            commands.entity(e).despawn();
        }
    }
    let now = Some((inside, event.items));
    if *changed == now {
        return;
    }
    let was_inside = changed.map(|(inside, _)| inside).unwrap_or(false);
    *changed = now;
    for e in &shown {
        commands.entity(e).despawn();
    }
    let Some(disc) = disc else { return };
    if !inside {
        return;
    }
    // The pieces the event still has, solid on their units.
    let mut solid = Vec::new();
    for (slot, &(ux, uz)) in DISPLAY_UNITS.iter().enumerate() {
        let item = event.items[slot];
        if item == 0 {
            continue;
        }
        solid.push((ux, uz));
        let spot = Vec2::new(ux as f32 + 0.5, uz as f32 + 0.5);
        let top = tent
            .room
            .as_ref()
            .and_then(|r| r.height_at(spot))
            .unwrap_or(0.0);
        // The piece itself, standing on its unit (`ac_my_room` draws what `aBD_SetFurnitureFg` put there).
        if let Some(entity) = crate::furniture::spawn_furniture(
            &mut commands,
            &disc,
            item,
            (ux, uz),
            TENT_ORIGIN,
            top,
            &mut meshes,
            &mut materials,
            &mut images,
        ) {
            commands.entity(entity).insert(TentDisplay);
        }
    }
    if let Some(room) = tent.room.as_mut() {
        room.set_furniture(solid);
    }
    // `SP_NPC_EV_BROKER2` at unit (4, 4).
    if was_inside || !redd.is_empty() {
        return;
    }
    let spot = Vec2::new(REDD_UNIT.0 as f32 + 0.5, REDD_UNIT.1 as f32 + 0.5);
    commands.spawn((
        Redd2,
        Name::new("Crazy Redd (inside)"),
        crate::character::NpcClip::default(),
        crate::character::NpcGait::default(),
        crate::character::NpcModel(crate::redd::REDD_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(local(spot * UNIT) + Vec3::Y).looking_to(Vec3::Z, Vec3::Y),
    ));
}

pub struct BrokerTentPlugin;

impl Plugin for BrokerTentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tent>().add_systems(
            Update,
            (
                build_room,
                spawn_tent.after(crate::event_today::refresh),
                enter,
                leave,
                movement,
                furnish,
            )
                .chain()
                .in_set(GameplaySet),
        );
    }
}
