//! The kamakura (`ac_kamakura.c`, `ac_kamakura_move.c_inc`, `ac_kamakura_npc0.c`, `data/scene/kamakura.c`): the first user
//! of the generic event room (`event_room.rs`).
//!
//! While `mEv_EVENT_KAMAKURA` runs (January 2 to February 23) the snow hut stands at unit (4,7) (`l_kk_map`); here in the
//! shrine acre, a box standing for `kamakura_DL_model`. Its door is checked at (x, z + 40) within 40 game units, facing
//! south (`aKKR_check_player`); the player comes out at (x, z + 86) (`aKKR_rewrite_out_data`). Inside (`rom_kamakura_model`)
//! they stand at (160, 260) and the hut's villager (`SP_NPC_EV_KAMAKURA_0`, the first resident's model here) greets them
//! with `hello_msg[looks]` (`6367 6376 6358 6385 6394 6403`, the first variant).
//!
//! Not built: the hut's model, its lit window and shadow, the room's candle rings and mochi, the villager's place in the
//! room (a guess), and the quest manager's full talk (hello variants by Bells and items, errands, gifts).

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::event_room::{Door, EventRooms, RoomKind};
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::Facing;
use crate::player::Player;
use crate::roster::TownRoster;
use crate::shop::ShopLocation;
use crate::states::GameplaySet;
use crate::town::Town;

const UNIT: f32 = 40.0;
const HELLO: [u16; 6] = [6367, 6376, 6358, 6385, 6394, 6403];
/// The villager's place in the room, room-local game units (a guess).
const VILLAGER: Vec2 = Vec2::new(200.0, 120.0);

#[derive(Component)]
struct Hut {
    day: i64,
}

#[derive(Component)]
struct Keeper {
    looks: u8,
    index: u32,
}

/// The hut outside while the event runs; its door registered with the event room.
#[allow(clippy::too_many_arguments)]
fn hut(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Hut)>,
    mut rooms: ResMut<EventRooms>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let now = crate::clock::now();
    let on = today.is_active(ev::KAMAKURA, now.hour);
    let in_town =
        *state.get() == ShopLocation::Town && *house.get() == crate::house::HouseState::Town;
    for (entity, h) in &existing {
        if !on || h.day != now.day_number {
            commands.entity(entity).despawn();
            rooms.clear_door(RoomKind::Kamakura);
        }
    }
    if !on || !in_town || !existing.is_empty() {
        return;
    }
    let Some(at) = crate::tortimer::shrine_cell(&town, 4, 7) else {
        return;
    };
    commands.spawn((
        Hut {
            day: now.day_number,
        },
        Name::new("kamakura"),
        Mesh3d(meshes.add(Cuboid::new(3.0, 2.0, 3.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.92, 0.95, 1.0))),
        Transform::from_translation(at + Vec3::Y),
    ));
    rooms.set_door(
        RoomKind::Kamakura,
        Door {
            check: at + Vec3::new(0.0, 0.0, 40.0 / UNIT),
            radius: 40.0 / UNIT,
            exit: at + Vec3::new(0.0, 0.0, 86.0 / UNIT),
        },
    );
    info!("The kamakura stands at {at:?}");
}

/// The villager inside, for the visit.
fn keeper(
    mut commands: Commands,
    rooms: Res<EventRooms>,
    roster: Option<Res<TownRoster>>,
    existing: Query<Entity, With<Keeper>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let inside = rooms.current() == Some(RoomKind::Kamakura);
    if !inside {
        for e in &existing {
            commands.entity(e).despawn();
        }
        return;
    }
    if !existing.is_empty() {
        return;
    }
    let Some(entry) = roster.as_ref().and_then(|r| r.residents.first()) else {
        return;
    };
    commands.spawn((
        Name::new("kamakura villager"),
        Keeper {
            looks: entry.looks,
            index: entry.index,
        },
        NpcClip::default(),
        NpcGait::default(),
        NpcModel(entry.index),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(RoomKind::Kamakura.local(VILLAGER) + Vec3::Y)
            .looking_to(Vec3::Z, Vec3::Y),
    ));
}

#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), (With<Player>, Without<Keeper>)>,
    keeper: Query<(Entity, &Transform, &Keeper)>,
    mut dialogue: ResMut<DialogueState>,
    identity: Res<crate::save::PlayerIdentity>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state.get() != ShopLocation::EventRoom {
        return;
    }
    // A line ends when it is read.
    if dialogue.active
        && dialogue.speaker.is_some_and(|e| keeper.get(e).is_ok())
        && dialogue.external_result.take().is_some()
    {
        *dialogue = DialogueState::default();
        *focus = InputFocus::World;
        return;
    }
    if !focus.is_world() || !actions.just_pressed(Action::Interact) {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    let Some((actor, _, k)) = keeper.iter().find(|(_, t, _)| {
        let delta = (t.translation - at.translation).with_y(0.0);
        delta.length() < 1.8 && facing.dir.dot(delta.normalize_or_zero()) >= 0.5
    }) else {
        return;
    };
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    let ctx = crate::fireworks::context(&identity, &disc, k.index, 0);
    let msg = HELLO[usize::from(k.looks).min(5)];
    let Some(text) = disc.text.as_ref() else {
        return;
    };
    let (pages, _) = text.talk_asking(usize::from(msg), &ctx, &mut |_| 0);
    dialogue.begin_special(actor, "Villager", (k.index, 2), pages, None);
    info!("Kamakura message {msg}");
}

pub struct KamakuraPlugin;

impl Plugin for KamakuraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (hut.after(crate::event_today::refresh), keeper).in_set(GameplaySet),
        )
        .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
