//! New Year's first shrine visit (`ac_hatumode_control.c`, `ac_hatumode_npc0.c`, `ac_hatumode_clip_move.c_inc`,
//! `ac_hatumode_clip_think.c_inc`).
//!
//! While `mEv_EVENT_NEW_YEARS_DAY` runs (January 1, 06:00-09:59) five villagers (`SP_NPC_EV_HATUMODE_0..4`) stand in front
//! of the shrine, at units (7,9) (6,10) (9,10) (6,12) (9,12) moved by `(20,-10) (40,40) (-40,40) (40,0) (-40,0)` units
//! (`aHTMD_clip_set_start`), facing north. The first asks the player to pray with them (`base_msg`, Yes / No: yes
//! `base_msg + 1`, no `base_msg + 2`); the others chat `base_msg + 3 / 6 / 9 / 12 + RANDOM(3)` by their place in the
//! queue relative to the first (`aHN0_make_msg`). The bases are `7679 7697 7661 7715 7733 7751` by looks.
//!
//! Praying with them: the player is walked (demo walk, 2 units a tick) to `(20, 120)` and then `(20, 85)` units from the
//! shrine (`aHTMD_clip_player_move`). The shrine then speaks without a name (`aHTC_request`): with 50 Bells `4396`, yes
//! `0x112D` and the coin (`aHTC_pay`, `aHTC_saisen`: 50 Bells paid), no `0x112E`; without the Bells `4398` (`0x112E`).
//! The prayer ends the visit.
//!
//! Not built: the villagers' own queue walking, bows and prayers (the 45 think states), the pray and coin-throw
//! animations, the shrine's bell, the second and third positions of the original's walk, the red message window.

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::messages::MsgContext;
use crate::player::Player;
use crate::roster::TownRoster;
use crate::shop::ShopLocation;
use crate::states::GameplaySet;
use crate::town::Town;

const SOUND: u8 = 2;
const UNIT: f32 = 40.0;
const BASE_MSG: [u16; 6] = [7679, 7697, 7661, 7715, 7733, 7751];
/// Units and start nudges of the five villagers.
const UNITS: [(usize, usize); 5] = [(7, 9), (6, 10), (9, 10), (6, 12), (9, 12)];
const NUDGE: [(f32, f32); 5] = [
    (20.0, -10.0),
    (40.0, 40.0),
    (-40.0, 40.0),
    (40.0, 0.0),
    (-40.0, 0.0),
];
/// `aHTMD_clip_player_move`: where the player is walked, from the well, in units.
const WALK: [(f32, f32); 2] = [(20.0, 120.0), (20.0, 85.0)];
const PRICE: u32 = 50;
/// The demo walk's speed: 2 units a tick.
const SPEED: f32 = 2.0 * 60.0 / UNIT;
/// How long the prayer lasts, in seconds.
const PRAYER: f32 = 3.0;

#[derive(Component)]
struct Visitor {
    day: i64,
    place: usize,
    looks: u8,
}

#[derive(Default)]
enum Phase {
    #[default]
    Idle,
    /// The first villager's question is up.
    Asked,
    /// A closing line is up; the talk ends with it.
    Line,
    /// The villager's `base + 1` is up: the walk follows.
    Joining,
    /// The player is being walked: the leg (0 or 1).
    Walking(usize),
    /// The shrine's question is up.
    Prompt,
    /// `0x112D` or the no-money line is up; the prayer follows (`true` when the coin goes).
    Said(bool),
    /// Praying: seconds left.
    Praying(f32),
}

#[derive(Resource, Default)]
struct Visit {
    phase: Phase,
}

fn well(town: &Town) -> Option<Vec3> {
    town.retail
        .as_ref()?
        .buildings
        .iter()
        .find(|b| b.item == 0x5825)
        .map(|b| b.origin)
}

/// Brings the five villagers to the shrine.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    roster: Option<Res<TownRoster>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Visitor)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let on = today.is_active(ev::NEW_YEARS_DAY, now.hour);
    let out =
        on && *state.get() == ShopLocation::Town && *house.get() == crate::house::HouseState::Town;
    for (entity, v) in &existing {
        if !out || v.day != now.day_number {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    let Some(roster) = roster else { return };
    for (i, entry) in roster.residents.iter().take(5).enumerate() {
        let Some(at) = crate::tortimer::shrine_cell(&town, UNITS[i].0, UNITS[i].1) else {
            continue;
        };
        let at = at + Vec3::new(NUDGE[i].0, 0.0, NUDGE[i].1) / UNIT;
        commands.spawn((
            Name::new(format!("{} (hatsumode)", entry.name)),
            Visitor {
                day: now.day_number,
                place: i,
                looks: entry.looks,
            },
            NpcClip::default(),
            NpcGait::default(),
            NpcModel(entry.index),
            Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
            // Facing north, toward the shrine.
            Transform::from_translation(at + Vec3::Y)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        ));
    }
    info!("The New Year visitors stand before the shrine");
}

fn context(identity: &crate::save::PlayerIdentity, disc: &GameDisc, index: u32) -> MsgContext {
    let now = crate::clock::now();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Villager".into(),
        tail: crate::dialogue::catchphrase(disc, index),
        free: vec![String::new(); 20],
        items: vec![String::new(); 5],
        item_articles: vec![0; 5],
        town_name: identity.town.clone(),
        year: now.year,
        month: now.month,
        day: now.day,
        hour: now.hour,
        minute: now.minute,
        weekday: now.weekday,
        ..default()
    }
}

#[allow(clippy::too_many_arguments)]
fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &MsgContext,
    name: &'static str,
    msg: u16,
) -> Option<Vec<Option<usize>>> {
    let text = disc.text.as_ref()?;
    let (pages, asked) = text.talk_asking(usize::from(msg), ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        name,
        (0, SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Hatsumode message {msg}");
    asked.map(|(_, c)| c.next)
}

/// The talks, the walk and the prayer.
#[allow(clippy::too_many_arguments)]
fn talk(
    time: Res<Time>,
    disc: Option<Res<GameDisc>>,
    town: Res<Town>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    mut player: Query<(&mut Transform, &mut Facing), (With<Player>, Without<Visitor>)>,
    visitors: Query<(Entity, &Transform, &Visitor, &NpcModel)>,
    mut flow: ResMut<Visit>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let first = visitors.iter().find(|(_, _, v, _)| v.place == 0);
    // The walk and the prayer hold the player (the caller keeps focus frozen).
    match flow.phase {
        Phase::Walking(leg) => {
            let (Some(centre), Ok((mut at, mut facing))) = (well(&town), player.single_mut())
            else {
                return;
            };
            let goal = centre + Vec3::new(WALK[leg].0, 0.0, WALK[leg].1) / UNIT;
            let to = (goal - at.translation).with_y(0.0);
            let step = SPEED * time.delta_secs();
            if to.length() <= step {
                at.translation = goal.with_y(town.ground_y(goal) + 1.0);
                if leg == 0 {
                    flow.phase = Phase::Walking(1);
                } else {
                    // `aHTC_request`: the shrine's own question.
                    facing.dir = Vec3::NEG_Z;
                    at.rotation = Quat::from_rotation_y(std::f32::consts::PI);
                    let Some((actor, _, _, model)) = first else {
                        flow.phase = Phase::Idle;
                        return;
                    };
                    let ctx = context(&identity, &disc, model.0);
                    if inventory.bells >= PRICE {
                        say(&disc, &mut dialogue, actor, &ctx, "", 4396);
                        flow.phase = Phase::Prompt;
                    } else {
                        say(&disc, &mut dialogue, actor, &ctx, "", 4398);
                        flow.phase = Phase::Said(false);
                    }
                    *focus = InputFocus::Dialogue;
                }
            } else {
                let step_v = to.normalize() * step;
                at.translation += step_v;
                at.translation.y = town.ground_y(at.translation) + 1.0;
                facing.dir = to.normalize();
                at.rotation = Quat::from_rotation_y(to.x.atan2(to.z));
                *focus = InputFocus::Transition;
            }
            return;
        }
        Phase::Praying(left) => {
            let left = left - time.delta_secs();
            if left <= 0.0 {
                flow.phase = Phase::Idle;
                *focus = InputFocus::World;
            } else {
                flow.phase = Phase::Praying(left);
                *focus = InputFocus::Transition;
            }
            return;
        }
        _ => {}
    }
    // A line was read, or an answer given.
    if let Some((actor, _, _, model)) = first {
        if dialogue.speaker == Some(actor) {
            if let Some(choice) = dialogue.external_result.take() {
                *focus = InputFocus::Dialogue;
                let ctx = context(&identity, &disc, model.0);
                let base = BASE_MSG[usize::from(first.map_or(0, |f| f.2.looks)).min(5)];
                match std::mem::take(&mut flow.phase) {
                    Phase::Asked => {
                        if choice == 0 {
                            say(&disc, &mut dialogue, actor, &ctx, "Villager", base + 1);
                            flow.phase = Phase::Joining;
                        } else {
                            say(&disc, &mut dialogue, actor, &ctx, "Villager", base + 2);
                            flow.phase = Phase::Line;
                        }
                    }
                    Phase::Joining => {
                        // `aHN0_talk_stop_player`: the player is walked to the shrine.
                        *dialogue = DialogueState::default();
                        flow.phase = Phase::Walking(0);
                        *focus = InputFocus::Transition;
                    }
                    Phase::Prompt => {
                        if choice == 0 {
                            say(&disc, &mut dialogue, actor, &ctx, "", 0x112D);
                            flow.phase = Phase::Said(true);
                        } else {
                            say(&disc, &mut dialogue, actor, &ctx, "", 0x112E);
                            flow.phase = Phase::Said(false);
                        }
                    }
                    Phase::Said(coin) => {
                        // `aHTC_pay`, `aHTC_saisen`, `aHTC_inori`.
                        *dialogue = DialogueState::default();
                        if coin {
                            inventory.bells = inventory.bells.saturating_sub(PRICE);
                            info!("The New Year offering: {PRICE} Bells");
                        }
                        flow.phase = Phase::Praying(PRAYER);
                        *focus = InputFocus::Transition;
                    }
                    Phase::Line | Phase::Idle | Phase::Walking(_) | Phase::Praying(_) => {
                        *dialogue = DialogueState::default();
                        *focus = InputFocus::World;
                    }
                }
                return;
            }
        }
    }
    // A line of another villager ends when it is read.
    if matches!(flow.phase, Phase::Idle)
        && dialogue.active
        && dialogue.speaker.is_some_and(|e| visitors.get(e).is_ok())
        && dialogue.external_result.take().is_some()
    {
        *dialogue = DialogueState::default();
        *focus = InputFocus::World;
        return;
    }
    if !matches!(flow.phase, Phase::Idle)
        || !focus.is_world()
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    let Some((actor, _, v, model)) = visitors.iter().find(|(_, t, _, _)| {
        let delta = (t.translation - at.translation).with_y(0.0);
        delta.length() < 1.8 && facing.dir.dot(delta.normalize_or_zero()) >= 0.5
    }) else {
        return;
    };
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    let ctx = context(&identity, &disc, model.0);
    let base = BASE_MSG[usize::from(v.looks).min(5)];
    if v.place == 0 {
        // `dt_tbl1[0]`: the invitation (`aHN0_talk_saisen_suru`).
        say(&disc, &mut dialogue, actor, &ctx, "Villager", base);
        flow.phase = Phase::Asked;
    } else {
        // `aHN0_make_msg` with the first villager's turn.
        let idx = (4 + v.place) & 3;
        let offset = match idx {
            1 => 3,
            2 => 6,
            3 => 9,
            _ => 12,
        };
        say(
            &disc,
            &mut dialogue,
            actor,
            &ctx,
            "Villager",
            base + offset + rand.0.random(3) as u16,
        );
    }
}

pub struct HatsumodePlugin;

impl Plugin for HatsumodePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Visit>()
            .add_systems(
                Update,
                visit.after(crate::event_today::refresh).in_set(GameplaySet),
            )
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
