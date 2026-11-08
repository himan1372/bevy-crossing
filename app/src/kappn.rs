//! Kapp'n's boat to the island (`ac_boat.c`, `ac_boat_demo.c`, `ac_npc_sendo.c`; the route and songs are
//! `logic/src/boat.rs`).
//!
//! In the original the island, its resident and the trip's data come over a Game Boy Advance cable. On PC the island is
//! a built-in default one (the decomp's own PC build skips the GBA chain too: `aSEN_pl_ride_on_check2_talk_proc` goes
//! straight to "climb aboard"): the island is the two ocean blocks of the town grid (`ISLAND_LEFT` / `ISLAND_RIGHT`, blocks
//! (4,8) and (5,8)). Kapp'n asks for its name before the first crossing.
//!
//! The dock: Kapp'n (`SP_NPC_SENDO`, model 238 + 114) rows his dinghy at unit (5,10) of block (5,6), `aBT_chk_point[0]`.
//! Talking to him: `0x3073` (Yes, I'll cross / No, I won't): No `0x3074`; the first Yes gives `0x3079`, the native island
//! name window, then `0x30BE`. Later crossings use `0x3076`. On the island the same boat waits at the last checkpoint
//! and his question is `0x3080` (leaving: yes `0x3083`, no `0x3081`).
//!
//! The voyage (`aBTD_*`): the player rides the boat along the eight checkpoints at `0.625` units a frame. Kapp'n calls out
//! the departure (`0x30BF` to the island, `0x3084` home), sings the first part of a shanty (`0x3091 / 0x3163` + song to the
//! island, `0x3096 / 0x3168` home, by the player's sex; never the song of the last trip that way), says something to the
//! player (`0x309B..` by the same patterns, `+ RANDOM(10)`), sings the second part (`0x30AF / 0x3181 + RANDOM(10)`), and the
//! boat touches the wharf: the player steps out and he calls the arrival (`0x307A` at the island, `0x308B` home).
//!
//! Not built: the boat's and Kapp'n's models (a box, the capsule model of the sailor), the rain and night variants
//! (`0x30B9`, `0x308C`), the oar animation and the songs' music, the
//! GBA save of the island, and the island's own scenes (see `lighthouse.rs`).

use std::collections::VecDeque;

use bevy::prelude::*;
use rustimal_logic::boat;

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::Facing;
use crate::player::Player;
use crate::shop::ShopLocation;
use crate::states::GameplaySet;
use crate::town::Town;

const SOUND: u8 = 2;
/// ALL_NPC_NUM + (SP_NPC_SENDO - SP_NPC_START).
const KAPPN_NPC: u32 = 238 + 114;
/// Cells a second: `FRAMERATE_ADJ(0.625)` game units a frame.
const SPEED: f32 = 0.625 * 60.0 / 40.0;
/// Seconds at the wharf before the player steps out (`FRAMERATE_TIMER(190)`).
const WHARF: f32 = 190.0 / 60.0;
/// Where the boat's passengers sit, relative to it.
const SEAT: Vec3 = Vec3::new(0.0, 0.8, 0.0);

/// `aBT_chk_point[]` in world cells (the map's cell grid starts at block 0; the playable acres at (-40, -48)).
fn route() -> [Vec2; 8] {
    let mut out = [Vec2::ZERO; 8];
    for (i, &(x, z)) in boat::CHK_POINTS.iter().enumerate() {
        out[i] = Vec2::new(x / 40.0 - 56.0, z / 40.0 - 64.0);
    }
    out
}

#[derive(Component)]
struct Boat;

#[derive(Component)]
struct Kappn;

/// The island's state: where the boat is, which songs were last sung.
#[derive(Resource)]
struct Island {
    at_island: bool,
    last_to: u8,
    last_from: u8,
}

impl Default for Island {
    fn default() -> Self {
        Self {
            at_island: false,
            last_to: boat::song::LAST_SONG_INIT,
            last_from: boat::song::LAST_SONG_INIT,
        }
    }
}

enum Step {
    Idle,
    /// A line is up; the talk ends with it.
    Line,
    /// The question is up (leave / cross).
    Asked,
    /// `0x3079`: Kapp'n explains the first island's name.
    NameIntro,
    /// The mandatory `mLE_TYPE_ISLAND_NAME` window owns input.
    Naming,
    /// The "climb aboard" line is up: the voyage follows.
    Board,
    /// Sailing: the distance along the route, the lines still to say, the wharf timer.
    Sailing {
        to_island: bool,
        distance: f32,
        queue: VecDeque<u16>,
        wharf: f32,
        lines_done: bool,
    },
    /// The arrival line is up.
    Arrival,
}

#[derive(Resource)]
struct Voyage {
    step: Step,
}

impl Default for Voyage {
    fn default() -> Self {
        Self { step: Step::Idle }
    }
}

/// The point `distance` cells along a route of points, and the heading there.
fn along(points: &[Vec2], distance: f32) -> (Vec2, Vec2) {
    let mut left = distance;
    for pair in points.windows(2) {
        let seg = pair[1] - pair[0];
        let len = seg.length();
        if left <= len {
            return (
                pair[0] + seg.normalize_or_zero() * left,
                seg.normalize_or_zero(),
            );
        }
        left -= len;
    }
    let last = points[points.len() - 1];
    let dir = (last - points[points.len() - 2]).normalize_or_zero();
    (last, dir)
}

fn route_length(points: &[Vec2]) -> f32 {
    points.windows(2).map(|p| (p[1] - p[0]).length()).sum()
}

/// The boat and Kapp'n at the dock.
fn spawn(
    mut commands: Commands,
    town: Option<Res<Town>>,
    mut island: ResMut<Island>,
    existing: Query<(), With<Boat>>,
    player: Query<&Transform, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !existing.is_empty() || town.is_none() {
        return;
    }
    // Saves store the exterior return position even when made in the cottage.
    // Reconstruct which dock owns the boat after a process restart.
    let Ok(player_at) = player.single() else {
        return;
    };
    island.at_island = Town::on_island(player_at.translation);
    let points = route();
    let at = if island.at_island {
        points[7]
    } else {
        points[0]
    };
    let boat_at = Vec3::new(at.x, 0.1, at.y);
    let root = commands
        .spawn((
            Boat,
            Name::new("Kapp'n's boat"),
            Mesh3d(meshes.add(Cuboid::new(1.4, 0.5, 2.6))),
            MeshMaterial3d(materials.add(Color::srgb(0.55, 0.35, 0.2))),
            Transform::from_translation(boat_at).with_rotation(Quat::from_rotation_y(
                if island.at_island {
                    135f32.to_radians()
                } else {
                    -45f32.to_radians()
                },
            )),
        ))
        .id();
    commands.spawn((
        Kappn,
        Name::new("Kapp'n"),
        NpcClip::default(),
        NpcGait::default(),
        NpcModel(KAPPN_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(Vec3::new(0.0, 0.9, -0.9)),
        ChildOf(root),
    ));
    info!("Kapp'n's boat waits at {boat_at:?}");
}

fn context(identity: &crate::save::PlayerIdentity, disc: &GameDisc) -> crate::messages::MsgContext {
    let mut ctx = crate::fireworks::context(identity, disc, 0, 0);
    // `mMsg_FREE_STR0`: the island's name.
    ctx.free[0] = saved_name(identity).unwrap_or(&identity.town).to_string();
    ctx
}

fn saved_name(identity: &crate::save::PlayerIdentity) -> Option<&str> {
    identity
        .island_name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
}

/// `aSEN_get_msg_no`: fixed conversations have a female bank 210 entries later.
/// Shanty messages already come from `boat`'s gender-specific tables.
fn fixed_message(message: u16, female: bool) -> u16 {
    message + if female { 210 } else { 0 }
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
    let (pages, asked) = text.talk_asking(usize::from(msg), ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "Kapp'n",
        (KAPPN_NPC, SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Kapp'n message {msg:#06x}");
}

#[allow(clippy::too_many_arguments)]
fn talk(
    time: Res<Time>,
    disc: Option<Res<GameDisc>>,
    town: Option<Res<Town>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    mut player: Query<(&mut Transform, &mut Facing), (With<Player>, Without<Boat>, Without<Kappn>)>,
    mut boats: Query<&mut Transform, (With<Boat>, Without<Player>, Without<Kappn>)>,
    kappn: Query<Entity, With<Kappn>>,
    mut dialogue: ResMut<DialogueState>,
    mut island: ResMut<Island>,
    mut voyage: ResMut<Voyage>,
    mut identity: ResMut<crate::save::PlayerIdentity>,
    (look, mut chat, mut names, mut reset): (
        Res<crate::character::PlayerLook>,
        ResMut<crate::island_trade::IslandChat>,
        ResMut<crate::name_entry::NameEntry>,
        MessageReader<crate::sound::ResetAudio>,
    ),
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    if reset.read().count() > 0 {
        if matches!(voyage.step, Step::Naming) {
            names.cancel_island();
            if !names.open() && *focus == InputFocus::NameEntry {
                *focus = InputFocus::World;
            }
        }
        if !matches!(voyage.step, Step::Idle)
            && dialogue.speaker.is_some_and(|actor| kappn.contains(actor))
        {
            *dialogue = DialogueState::default();
            if matches!(*focus, InputFocus::Dialogue | InputFocus::Transition) {
                *focus = InputFocus::World;
            }
        }
        voyage.step = Step::Idle;
        island.at_island = player
            .single()
            .is_ok_and(|(at, _)| Town::on_island(at.translation));
        if let Ok(mut boat) = boats.single_mut() {
            let at = route()[if island.at_island { 7 } else { 0 }];
            boat.translation = Vec3::new(at.x, 0.1, at.y);
            boat.rotation = Quat::from_rotation_y(if island.at_island {
                135f32.to_radians()
            } else {
                -45f32.to_radians()
            });
        }
        return;
    }
    let (Some(disc), Some(town)) = (disc.filter(|d| d.text.is_some()), town) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let (Ok(actor), Ok(mut boat_at)) = (kappn.single(), boats.single_mut()) else {
        return;
    };
    let ctx = context(&identity, &disc);
    let dt = time.delta_secs();
    let female = u8::from(look.female);
    if matches!(voyage.step, Step::Naming) {
        if let Some(bytes) = names.take() {
            let name = crate::messages::plain(&bytes);
            if name.trim().is_empty() {
                names.begin(crate::name_entry::Kind::Island);
                *focus = InputFocus::NameEntry;
            } else {
                identity.island_name = Some(name);
                let ctx = context(&identity, &disc);
                say(
                    &disc,
                    &mut dialogue,
                    actor,
                    &ctx,
                    fixed_message(0x30BE, look.female),
                );
                voyage.step = Step::Board;
                *focus = InputFocus::Dialogue;
            }
        } else {
            *focus = if names.open() {
                InputFocus::NameEntry
            } else {
                InputFocus::Dialogue
            };
        }
        return;
    }
    // The voyage.
    if let Step::Sailing {
        to_island,
        distance,
        queue,
        wharf,
        lines_done,
    } = &mut voyage.step
    {
        let mut points = route().to_vec();
        if !*to_island {
            points.reverse();
        }
        let total = route_length(&points);
        *distance = (*distance + SPEED * dt).min(total);
        let (p, dir) = along(&points, *distance);
        boat_at.translation = Vec3::new(p.x, 0.1, p.y);
        boat_at.rotation = Quat::from_rotation_y(dir.x.atan2(dir.y));
        if let Ok((mut at, mut facing)) = player.single_mut() {
            at.translation = boat_at.translation + SEAT;
            at.rotation = Quat::from_rotation_y(dir.x.atan2(dir.y));
            facing.dir = Vec3::new(dir.x, 0.0, dir.y);
        }
        // A line was read.
        if dialogue.speaker == Some(actor)
            && (dialogue.active || dialogue.external_result.is_some())
            && dialogue.external_result.take().is_some()
        {
            *dialogue = DialogueState::default();
        }
        if dialogue.active {
            *focus = InputFocus::Dialogue;
            return;
        }
        *focus = InputFocus::Transition;
        if let Some(msg) = queue.pop_front() {
            *focus = InputFocus::Dialogue;
            say(&disc, &mut dialogue, actor, &ctx, msg);
            return;
        }
        *lines_done = true;
        if *distance >= total {
            *wharf -= dt;
            if *wharf <= 0.0 {
                // The player steps out on the shore.
                let landing = land(&town, *to_island);
                info!("Kapp'n: the player lands at {landing:?}");
                if let Ok((mut at, mut facing)) = player.single_mut() {
                    at.translation = landing;
                    facing.dir = Vec3::Z;
                }
                island.at_island = *to_island;
                if *to_island {
                    // A new visit: her first talk checks for a trade again (`mNpc_SetIslandCheckFtrMsg`).
                    chat.checked = false;
                }
                let arrival = fixed_message(if *to_island { 0x307A } else { 0x308B }, look.female);
                say(&disc, &mut dialogue, actor, &ctx, arrival);
                *focus = InputFocus::Dialogue;
                voyage.step = Step::Arrival;
            }
        }
        return;
    }
    // An answer, or a line read.
    if dialogue.speaker == Some(actor)
        && (dialogue.active || dialogue.external_result.is_some())
        && !matches!(voyage.step, Step::Idle)
    {
        let Some(choice) = dialogue.external_result.take() else {
            return;
        };
        *focus = InputFocus::Dialogue;
        match std::mem::replace(&mut voyage.step, Step::Idle) {
            Step::Asked => {
                if choice == 0 {
                    // Yes: the GBA chain is skipped (`aSEN_pl_ride_on_check2_talk_proc` on PC).
                    let first_visit = !island.at_island && saved_name(&identity).is_none();
                    let msg = if island.at_island {
                        0x3083
                    } else if !first_visit {
                        0x3076
                    } else {
                        0x3079
                    };
                    say(
                        &disc,
                        &mut dialogue,
                        actor,
                        &ctx,
                        fixed_message(msg, look.female),
                    );
                    voyage.step = if first_visit {
                        Step::NameIntro
                    } else {
                        Step::Board
                    };
                } else {
                    let msg = if island.at_island { 0x3081 } else { 0x3074 };
                    say(
                        &disc,
                        &mut dialogue,
                        actor,
                        &ctx,
                        fixed_message(msg, look.female),
                    );
                    voyage.step = Step::Line;
                }
            }
            Step::NameIntro => {
                *dialogue = DialogueState::default();
                // Keep the owner while LEDIT closes, before Kapp'n consumes its result.
                dialogue.speaker = Some(actor);
                dialogue.external_result = Some(0);
                names.begin(crate::name_entry::Kind::Island);
                voyage.step = Step::Naming;
                *focus = InputFocus::NameEntry;
            }
            Step::Board => {
                *dialogue = DialogueState::default();
                let to_island = !island.at_island;
                // The songs, never the last of this direction (`aBTD_start_call_end_wait_init`).
                let last = if to_island {
                    island.last_to
                } else {
                    island.last_from
                };
                let song = loop {
                    let s = rand.0.random(5) as u8;
                    if boat::song_accepted(s, last) {
                        break s;
                    }
                };
                if to_island {
                    island.last_to = song;
                } else {
                    island.last_from = song;
                }
                let pattern = boat::player_pattern(!to_island, female);
                let queue = VecDeque::from([
                    fixed_message(if to_island { 0x30BF } else { 0x3084 }, look.female),
                    boat::first_part_msg(pattern, song),
                    boat::dialogue_msg(pattern, rand.0.random(10) as u8),
                    boat::second_part_msg(pattern, rand.0.random(10) as u8),
                ]);
                voyage.step = Step::Sailing {
                    to_island,
                    distance: 0.0,
                    queue,
                    wharf: WHARF,
                    lines_done: false,
                };
                *focus = InputFocus::Transition;
            }
            Step::Line | Step::Arrival | Step::Idle | Step::Naming | Step::Sailing { .. } => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
            }
        }
        return;
    }
    if !focus.is_world() || !actions.just_pressed(Action::Interact) {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    let delta = (boat_at.translation - at.translation).with_y(0.0);
    if delta.length() > 3.2 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    let msg = fixed_message(if island.at_island { 0x3080 } else { 0x3073 }, look.female);
    info!(
        "Kapp'n talk: at_island {} named {}",
        island.at_island,
        saved_name(&identity).is_some()
    );
    say(&disc, &mut dialogue, actor, &ctx, msg);
    voyage.step = Step::Asked;
}

/// Where the player steps out: onto the wharf beside the boat (`aBTD_pl_ride_off_*`).
fn land(town: &Town, to_island: bool) -> Vec3 {
    let at = if to_island {
        // The island's wharf runs west from the last checkpoint to the shore.
        Vec3::new(32.5, 0.0, 69.7)
    } else {
        // The home dock is east of the first checkpoint.
        Vec3::new(31.5, 0.0, 43.5)
    };
    at.with_y(town.ground_y(at) + 1.0)
}

pub struct KappnPlugin;

impl Plugin for KappnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Island>()
            .init_resource::<Voyage>()
            .add_systems(Update, spawn.in_set(GameplaySet))
            .add_systems(
                Update,
                talk.after(crate::save::LoadSet)
                    .in_set(InteractSet::Talk)
                    .in_set(GameplaySet),
            );
    }
}
