//! The lighthouse and Tortimer's lighthouse quest (`ac_toudai.c`, `ac_toudai_move.c_inc`, `ac_lighthouse_switch.c`,
//! `ac_present_demo_move.c_inc`, `ac_present_npc_schedule.c_inc`, `m_soncho.c`: `mSC_LightHouse_*`).
//!
//! The lighthouse (`TOUDAI`) stands on the town's southern coast (block (3,6), unit (5,7)). Its door is on the north
//! side: the player walks south into it, within 20 game units of its x and 65 north of it (`aTOU_check_door_pos`), and the
//! room `rom_toudai_model` opens (`LIGHTHOUSE_player_data`: (120, 100) game units, facing south). The room's switch is at
//! (180, 100): in the quest's week, between 18:00 and 05:00, the player throws it (E beside it, `aLS_CheckPlayerAction`) and
//! `mSC_LightHouse_Switch_On` stamps that day and the player. Outside the quest week the lamp lights by itself at night
//! (`aLS_AutoSwitch`); in it, the lamp shines only on the nights its switch was thrown (`mSC_LightHouse_Switch_Check`).
//!
//! The reward is the present demo (`aPRD_setup_present`): in the ten days after the week, when the player is in town,
//! Tortimer (the present NPC) comes up to them. Seven switched days earn the present (`FTR_TAK_TOUDAI` in January,
//! `FTR_IKE_JNY_MAKADA01` in February; `0x33FD` / `0x3414`, or `0x33FF` / `0x3416` with full pockets, when nothing is
//! given and nothing cleared); a started quest that fell short gets `0x3400` / `0x3417`, and is cleared.
//!
//! Not built: the tower's disc model (the tower is a placeholder; its skeleton draws in the NPC render path), the lamp's
//! beam and the lens' turning, the switch's and pole's skeleton models and animation, the player's own switch-throwing
//! animation (a timed pause), the automatic door walk-in at dusk (`mSC_LightHouse_In_Check`), the other presents of the
//! present demo (birthday, the golden tools).

use bevy::prelude::*;

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::event_room::{Door, EventRooms, RoomKind};
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::player::Player;
use crate::save::PlayerIdentity;
use crate::shop::ShopLocation;
use crate::states::GameplaySet;
use crate::town::Town;

const UNIT: f32 = 40.0;
const SOUND: u8 = 2;
/// ALL_NPC_NUM + (SP_NPC_EV_SONCHO - SP_NPC_START): Tortimer.
const TORTIMER_NPC: u32 = 238 + 116;
/// The switch in the room, game units (`aLS_CheckPlayerSwitchPositionAngle`).
const SWITCH: Vec2 = Vec2::new(180.0, 100.0);
/// How long throwing the switch holds the player, seconds.
const THROW: f32 = 2.0;

#[derive(Component)]
struct Tower;

#[derive(Component)]
struct Lamp;

#[derive(Component)]
struct SwitchBox;

#[derive(Component)]
struct Guest {
    day: i64,
    talked: bool,
}

/// The lighthouse's base, `aTOU_actor_ct`: the unit centre less 20 game units each way.
fn origin() -> Vec3 {
    Town::cell_to_world(2, 5, 5, 7) + Vec3::new(-0.5, 0.0, -0.5)
}

fn date() -> ((u16, u8, u8), u8) {
    let now = crate::clock::now();
    ((now.year, now.month, now.day), now.hour)
}

/// Night, the lamp's hours (`sec >= 64800 || sec < 18000`).
fn night() -> bool {
    let secs = crate::clock::now().day_secs;
    !(18_000..64_800).contains(&secs)
}

/// The tower on the coast, its door, and its lamp.
fn tower(
    mut commands: Commands,
    town: Option<Res<Town>>,
    identity: Res<PlayerIdentity>,
    existing: Query<(), With<Tower>>,
    mut lamps: Query<&mut PointLight, With<Lamp>>,
    mut rooms: ResMut<EventRooms>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(town) = town else { return };
    let at = origin();
    let (now, hour) = date();
    // At dusk in the quest's week, before the switch, the door takes the player in by itself (`mSC_LightHouse_In_Check`).
    let auto = identity.lighthouse.in_check(now, hour);
    if !existing.is_empty() {
        rooms.set_door(
            RoomKind::Lighthouse,
            Door {
                check: at + Vec3::new(0.0, 0.0, -0.8),
                radius: 0.9,
                exit: at + Vec3::new(0.0, 0.0, -1.75),
                south: true,
                auto,
            },
        );
    }
    if existing.is_empty() {
        let ground = town.ground_y(at);
        let root = commands
            .spawn((
                Tower,
                Name::new("lighthouse"),
                Mesh3d(meshes.add(Cylinder::new(0.9, 5.0))),
                MeshMaterial3d(materials.add(Color::srgb(0.93, 0.93, 0.9))),
                Transform::from_translation(at + Vec3::new(0.0, ground + 2.5, 0.3)),
            ))
            .id();
        commands.spawn((
            Name::new("lighthouse lamp room"),
            Mesh3d(meshes.add(Cylinder::new(0.7, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.9, 0.2, 0.15),
                ..default()
            })),
            Transform::from_translation(Vec3::new(0.0, 3.0, 0.0)),
            ChildOf(root),
        ));
        commands.spawn((
            Lamp,
            PointLight {
                color: Color::srgb(1.0, 0.95, 0.6),
                intensity: 0.0,
                range: 30.0,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 3.2, 0.0)),
            ChildOf(root),
        ));
        // The door is on the north side; the player walks south into it.
        rooms.set_door(
            RoomKind::Lighthouse,
            Door {
                check: at + Vec3::new(0.0, 0.0, -0.8),
                radius: 0.9,
                exit: at + Vec3::new(0.0, 0.0, -1.75),
                south: true,
                auto,
            },
        );
        info!("The lighthouse stands at {at:?}");
    }
    // The lamp: dark by day, lit at night when its switch was thrown (or no quest runs).
    let lit = night() && identity.lighthouse.switch_check(now, hour);
    for mut lamp in &mut lamps {
        lamp.intensity = if lit { 4_000_000.0 } else { 0.0 };
    }
}

/// The switch in the room, and throwing it.
#[allow(clippy::too_many_arguments)]
fn switch(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<State<ShopLocation>>,
    rooms: Res<EventRooms>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    mut identity: ResMut<PlayerIdentity>,
    player: Query<(&Transform, &Facing), With<Player>>,
    mut boxes: Query<(&mut MeshMaterial3d<StandardMaterial>, &Transform), With<SwitchBox>>,
    mut thrown: Local<Option<f32>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let inside =
        *state.get() == ShopLocation::EventRoom && rooms.current() == Some(RoomKind::Lighthouse);
    if boxes.is_empty() {
        // The switch's stand-in.
        let at = RoomKind::Lighthouse.local(SWITCH);
        commands.spawn((
            SwitchBox,
            Name::new("lighthouse switch"),
            Mesh3d(meshes.add(Cuboid::new(0.5, 0.9, 0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.6, 0.6, 0.65))),
            Transform::from_translation(at + Vec3::Y * 0.45),
        ));
        return;
    }
    if !inside {
        *thrown = None;
        return;
    }
    let (now, hour) = date();
    // The lever glows once today's switch is thrown.
    if let Ok((mut material, _)) = boxes.single_mut() {
        let on = identity.lighthouse.switch_check(now, hour)
            && identity.lighthouse.period(now) == crate::lighthouse::Period::Week;
        material.0 = materials.add(StandardMaterial {
            base_color: if on {
                Color::srgb(1.0, 0.9, 0.4)
            } else {
                Color::srgb(0.6, 0.6, 0.65)
            },
            emissive: if on {
                LinearRgba::new(2.0, 1.6, 0.4, 1.0)
            } else {
                LinearRgba::BLACK
            },
            ..default()
        });
    }
    if let Some(left) = *thrown {
        let left = left - time.delta_secs();
        if left <= 0.0 {
            *thrown = None;
            identity.lighthouse.switch_on(now);
            *focus = InputFocus::World;
            info!(
                "The lighthouse switch is thrown (day {})",
                identity.lighthouse.day(now)
            );
        } else {
            *thrown = Some(left);
            *focus = InputFocus::Transition;
        }
        return;
    }
    if !focus.is_world() || !actions.just_pressed(Action::Interact) {
        return;
    }
    let Ok((at, _)) = player.single() else { return };
    let switch_at = RoomKind::Lighthouse.local(SWITCH);
    let close = (at.translation - switch_at).with_y(0.0).length() < 33.6 / UNIT + 0.4;
    let can = identity.lighthouse.period(now) == crate::lighthouse::Period::Week
        && !(5..18).contains(&hour)
        && !identity.lighthouse.switch_check(now, hour);
    if close && can {
        actions.consume(Action::Interact);
        *thrown = Some(THROW);
        *focus = InputFocus::Transition;
    }
}

fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &crate::messages::MsgContext,
    msg: u16,
) {
    let Some(text) = disc.text.as_ref() else {
        return;
    };
    let (pages, _) = text.talk_asking(usize::from(msg), ctx, &mut |_| 0);
    dialogue.begin_special(actor, "Tortimer", (TORTIMER_NPC, SOUND), pages, None);
    info!("Lighthouse present message {msg:#06x}");
}

/// The present demo: Tortimer comes to the player with the quest's present (or its sorry line).
#[allow(clippy::too_many_arguments)]
fn present(
    mut commands: Commands,
    time: Res<Time>,
    disc: Option<Res<GameDisc>>,
    town: Option<Res<Town>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut identity: ResMut<PlayerIdentity>,
    mut inventory: ResMut<PlayerInventory>,
    mut dialogue: ResMut<DialogueState>,
    player: Query<&Transform, (With<Player>, Without<Guest>)>,
    mut guests: Query<(Entity, &mut Transform, &mut Guest)>,
    mut offered: Local<i64>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let (Some(disc), Some(town)) = (disc.filter(|d| d.text.is_some()), town) else {
        return;
    };
    if *state.get() != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let (now, _) = date();
    let today = crate::clock::now().day_number;
    let event = identity.lighthouse.event_check(now);
    let Ok(player_at) = player.single() else {
        return;
    };
    let player_at = player_at.translation;
    // He leaves once the talk is over.
    if let Ok((entity, mut at, mut guest)) = guests.single_mut() {
        let to = (player_at - at.translation).with_y(0.0);
        if !guest.talked && to.length() > 1.6 {
            let dir = to.normalize();
            let next = at.translation + dir * 3.5 * time.delta_secs();
            if town.can_step(at.translation, next) {
                at.translation = next;
                at.translation.y = town.ground_y(next) + 1.0;
            } else {
                // Blocked: he arrives all the same (the demo does not stall).
                at.translation = player_at - dir * 1.5;
            }
            at.rotation = Quat::from_rotation_y(dir.x.atan2(dir.z));
            *focus = InputFocus::Transition;
            return;
        }
        if !guest.talked {
            guest.talked = true;
            at.rotation = Quat::from_rotation_y(-to.x.atan2(-to.z));
            *focus = InputFocus::Dialogue;
            let item = identity.lighthouse.present_item();
            let free = inventory
                .pockets
                .pockets
                .contains(&rustimal_logic::inventory::EMPTY_NO);
            let mut ctx = crate::fireworks::context(&identity, &disc, TORTIMER_NPC, 0);
            ctx.items = vec![crate::items::name(item); 5];
            let msg = match (event, free) {
                (crate::lighthouse::Event::JanContributed, true) => 0x33FD,
                (crate::lighthouse::Event::JanContributed, false) => 0x33FF,
                (crate::lighthouse::Event::FebContributed, true) => 0x3414,
                (crate::lighthouse::Event::FebContributed, false) => 0x3416,
                (crate::lighthouse::Event::JanStarted, _) => 0x3400,
                (crate::lighthouse::Event::FebStarted, _) => 0x3417,
                (crate::lighthouse::Event::None, _) => 0x3400,
            };
            say(&disc, &mut dialogue, entity, &ctx, msg);
            // `aPST_present_send_start_wait_talk_proc`: the item goes into the pockets and the event is cleared.
            match (event, free) {
                (
                    crate::lighthouse::Event::JanContributed
                    | crate::lighthouse::Event::FebContributed,
                    true,
                ) => {
                    inventory.add(item);
                    identity.lighthouse.event_clear(now);
                    info!("Tortimer gives the lighthouse present {item:#06x}");
                }
                (
                    crate::lighthouse::Event::JanStarted | crate::lighthouse::Event::FebStarted,
                    _,
                ) => {
                    identity.lighthouse.event_clear(now);
                }
                _ => {}
            }
            return;
        }
        // The last line is read: he goes.
        if dialogue.speaker == Some(entity)
            && (dialogue.active || dialogue.external_result.is_some())
            && dialogue.external_result.take().is_some()
        {
            *dialogue = DialogueState::default();
            *focus = InputFocus::World;
            commands.entity(entity).despawn();
        }
        return;
    }
    // Is there a present for the player today?
    if event == crate::lighthouse::Event::None
        || *offered == today
        || !focus.is_world()
        || dialogue.active
    {
        return;
    }
    *offered = today;
    let from = player_at + Vec3::new(5.0, 0.0, 0.0);
    let from = from.with_y(town.ground_y(from) + 1.0);
    commands.spawn((
        Guest {
            day: today,
            talked: false,
        },
        Name::new("Tortimer (present)"),
        NpcClip::default(),
        NpcGait::default(),
        crate::event_body::EventBody::default(),
        NpcModel(TORTIMER_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(from),
    ));
    info!("Tortimer comes with the lighthouse quest's present ({event:?})");
}

/// A guest left over from another day goes.
fn tidy(mut commands: Commands, guests: Query<(Entity, &Guest)>) {
    let today = crate::clock::now().day_number;
    for (entity, guest) in &guests {
        if guest.day != today {
            commands.entity(entity).despawn();
        }
    }
}

pub struct ToudaiPlugin;

impl Plugin for ToudaiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (tower, switch, tidy).in_set(GameplaySet))
            .add_systems(
                Update,
                present.in_set(InteractSet::Talk).in_set(GameplaySet),
            );
    }
}
