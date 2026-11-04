//! Spooky the ghost (`SP_NPC_EV_GHOST`, "Wisp" in the decomp; `ac_ev_ghost.c`, `_talk.c_inc`, `_schedule.c_inc`).
//!
//! He is due on the ghost day the weekly picker rolls (`event_schedule`: 2 to 4 days ahead, kept for a week) and is out
//! from midnight to 04:00 (`aEGH_time_over`) unless the player has already given him the spirits that night. He is a
//! free-standing actor (`make_actor_in_free_block`): the port puts him on clear ground near the middle of a night-seeded
//! acre. He does not appear before he is found when the town has fewer than `aEGH_MINIMUM_GRASS_COUNT` weeds.
//!
//! The approach (`dt_tbl`, indexed by `think_act`; the decomp's enum names do not match the table, so it is read by
//! index): unseen, he speaks first from 5 cells (`0x2ED3 + RANDOM(5)`, think 2); then within 2 cells he shows himself
//! (`0x2EE2 + RANDOM(5)`: the found bit and the active flag are set, think 4); if the player walks 5.5 cells away he
//! calls from 7 cells (`0x2ED8 + RANDOM(5)`, think 3). Found, he is translucent (alpha 140) and a talk the player starts
//! (think 6) is `0x2EE7 + RANDOM(5)` with no spirits, `0x2EEB + n` with `n` of 1..=4, and with five or more `0x2EF0`:
//! the spirits are taken and three favours are offered (`aEGH_select_wait`): clear the weeds (`0x2EF1..=0x2EF5` by the
//! weed count; the weeds are removed), a new roof colour (`aEGH_select_roof`), or an item the player does not own yet
//! (`aEGH_not_collect_get`). Afterwards he leaves for the night (the returned bit).
//!
//! Port adaptations: translucency and the vanish animation are not drawn (he is hidden until found, then drawn plainly,
//! and removed when he leaves); the weeds go at once (the original sets `clear_grass` for the next day change).
//!
//! Not built: the spirits themselves in the world (hitodama in the five spirit acres), so the spirits for the favours
//! are not catchable yet; the camera moves of the forced talks.

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::wisp;
use serde::{Deserialize, Serialize};

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::field_items::FieldItems;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::messages::MsgContext;
use crate::player::Player;
use crate::save::LoadedSave;
use crate::shop::ShopLocation;
use crate::shop_nook::handover::{HandOver, Item, Party, Request};
use crate::states::GameplaySet;
use crate::town::Town;
use rustimal_logic::town_gen::{ACRE_DEPTH, ACRE_WIDTH};

/// ALL_NPC_NUM + (SP_NPC_EV_GHOST - SP_NPC_START).
const SPOOKY_NPC: u32 = 238 + 111;
/// `l_sp_actor_name` sound id of `SP_NPC_EV_GHOST`.
const SPOOKY_SOUND: u8 = 2;
/// `actor->talk_distance` values of the think states, in cells (units of 40).
const SPEAK_FROM: f32 = 200.0 / 40.0;
const COME_BACK_FROM: f32 = 280.0 / 40.0;
const SHOW_FROM: f32 = 80.0 / 40.0;
const WALKED_AWAY: f32 = 220.0 / 40.0;
const NORMAL_TALK: f32 = 55.0 / 40.0;

/// The event's save area (`mEv_gst_c`) for the one player and the shared flags.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpookyEvent {
    /// The game day this belongs to (`renew_time`).
    pub day: i64,
    /// `okoruhito_str_no`: the string of the angry one (0x62E + n).
    pub angry: u8,
    /// The player found him (`aEGH_BIT_FOUND_GHOST`).
    pub found: bool,
    /// The player gave him the spirits (`mEv_GHOST_FLAG_RETURNED_SPIRITS`).
    pub returned: bool,
}

#[derive(Component)]
struct Spooky {
    day: i64,
    phase: Phase,
}

/// The think states Spooky goes through (`dt_tbl` indices).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    /// Think 2: unseen, speaks from 5 cells.
    Hidden,
    /// Think 4: speaks (shows himself) within 2 cells.
    Near,
    /// Think 3: the player walked away; calls from 7 cells.
    Far,
    /// Think 0: waiting after calling; the player coming back within 5 cells brings back `Near`.
    Waiting,
    /// Think 6: found; the player's own talk.
    Found,
}

/// Startup: the saved event area.
fn apply_loaded(loaded: Res<LoadedSave>, mut event: ResMut<SpookyEvent>) {
    if let Some(save) = loaded.0.as_ref() {
        *event = save.spooky_event.clone().unwrap_or_default();
    }
}

/// Cheap integer mixer for the per-night picks.
fn mix(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ce_b9fe_1a85_1a85);
    x ^ (x >> 33)
}

/// `mFI_GetItemNumField(GRASS_A, GRASS_C)`: the weeds in the town.
fn weeds(items: &FieldItems) -> usize {
    items
        .acres
        .iter()
        .flat_map(|a| a.iter())
        .filter(|&&i| matches!(i, 0x0008..=0x000A))
        .count()
}

/// Clear ground near the middle of an acre picked from `seed`.
fn free_spot(town: &Town, seed: u64) -> Option<Vec3> {
    let (x_acres, z_acres) = (ACRE_WIDTH - 2, ACRE_DEPTH - 3);
    let n = x_acres * z_acres;
    let first = mix(seed) as usize % n;
    for step in 0..n {
        let k = (first + step) % n;
        let (ax, az) = (1 + k % x_acres, 2 + k / x_acres);
        let mut best: Option<(f32, Vec3)> = None;
        for cz in 2..14 {
            for cx in 2..14 {
                let at = Town::cell_to_world(ax, az, cx, cz);
                if !town.is_clear_ground(at) || town.in_retail_obstacle(at) {
                    continue;
                }
                let d = (cx as f32 - 7.5).abs() + (cz as f32 - 7.5).abs();
                if best.is_none_or(|(b, _)| d < b) {
                    best = Some((d, at));
                }
            }
        }
        if let Some((_, at)) = best {
            return Some(at.with_y(town.ground_y(at)));
        }
    }
    None
}

/// Rolls the night's save area, then brings him out and sends him off.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    items: Option<Res<FieldItems>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut existing: Query<(Entity, &Spooky, &mut Visibility)>,
    mut event: ResMut<SpookyEvent>,
    flow: Res<Talk>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let on = today.is_active(ev::GHOST, now.hour);
    let out =
        on && *state.get() == ShopLocation::Town && *house.get() == crate::house::HouseState::Town;
    for (entity, s, mut visible) in &mut existing {
        let gone = event.returned && matches!(flow.step, Step::Idle);
        if !on || s.day != now.day_number || gone {
            commands.entity(entity).despawn();
            if !on {
                // `aEGH_byebye_init`: the found bit goes with him.
                event.found = false;
            }
        } else {
            *visible = if event.found {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
    if !on {
        return;
    }
    // `aEGH_actor_ct`: a new night resets the area and picks the angry one's string.
    if event.day != now.day_number {
        *event = SpookyEvent {
            day: now.day_number,
            angry: rand.0.random(32) as u8,
            found: false,
            returned: false,
        };
    }
    if !out || event.returned || !existing.is_empty() {
        return;
    }
    // Unfound, he needs weeds to haunt.
    if !event.found {
        let Some(items) = items else { return };
        if weeds(&items) < usize::from(wisp::wisp::MINIMUM_GRASS_COUNT) {
            return;
        }
    }
    let Some(at) = free_spot(&town, now.day_number as u64 ^ 0x57_00_4b) else {
        return;
    };
    let phase = if event.found {
        Phase::Found
    } else {
        Phase::Hidden
    };
    commands.spawn((
        Name::new("Spooky"),
        Spooky {
            day: now.day_number,
            phase,
        },
        NpcClip::default(),
        NpcGait::default(),
        NpcModel(SPOOKY_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        if event.found {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        },
        Transform::from_translation(at + Vec3::Y),
    ));
    info!("Spooky is out at {at:?} ({phase:?})");
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// A plain line is up; the talk ends with it. `found` marks the line that shows him.
    Line,
    /// `0x2EF0` is up (five spirits): the answer picks the favour.
    Favour(Vec<Option<usize>>),
    /// The spirits are crossing; the favour follows.
    Handing(usize, Vec<Option<usize>>),
    /// A favour's line is up; `Some(item)` gives the item, `weeds` clears them after.
    Result(Option<u16>, bool),
    /// The roof colours are up (`next`: follow-ups; the three shown start at `from`).
    Roof(usize, Vec<Option<usize>>),
    /// The item is crossing.
    ItemHanding,
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
}

fn context(
    identity: &crate::save::PlayerIdentity,
    disc: &GameDisc,
    event: &SpookyEvent,
    item: Option<u16>,
) -> MsgContext {
    let now = crate::clock::now();
    let mut free = vec![String::new(); 20];
    if let Some(angry) = disc
        .text
        .as_ref()
        .and_then(|t| t.string(0x62E + usize::from(event.angry)))
    {
        free[1] = angry;
    }
    let name = item.map(crate::items::name).unwrap_or_default();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Spooky".into(),
        free,
        items: vec![name; 5],
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

/// Shows disc message `msg` in Spooky's voice; the choices' follow-ups come back when it asks.
fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &MsgContext,
    msg: u16,
) -> Option<Vec<Option<usize>>> {
    let text = disc.text.as_ref()?;
    let (pages, asked) = text.talk_asking(usize::from(msg), ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "Spooky",
        (SPOOKY_NPC, SPOOKY_SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Spooky message {msg:#06x}");
    asked.map(|(_, c)| c.next)
}

/// He turns to face the player while they talk.
fn face(
    time: Res<Time>,
    flow: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<Spooky>)>,
    mut spooky: Query<&mut Transform, With<Spooky>>,
) {
    let (Ok(at), Ok(mut transform)) = (player.single(), spooky.single_mut()) else {
        return;
    };
    if matches!(flow.step, Step::Idle) {
        return;
    }
    let to = (at.translation - transform.translation).with_y(0.0);
    if to.length() > 0.01 {
        let want = Quat::from_rotation_y(to.x.atan2(to.z));
        transform.rotation = transform
            .rotation
            .slerp(want, (time.delta_secs() * 8.0).min(1.0));
    }
}

/// `aEGH_hitodama_num`: the spirits in the pockets (a stack item holds 1..=5).
fn spirits(inventory: &PlayerInventory) -> u32 {
    inventory
        .pockets
        .pockets
        .iter()
        .map(|&i| u32::from(wisp::stack_count(i)))
        .sum()
}

/// `aEGH_delete_hitodama`.
fn take_spirits(inventory: &mut PlayerInventory) {
    for slot in 0..inventory.pockets.pockets.len() {
        if wisp::item_is_spirit(inventory.pockets.pockets[slot]) {
            inventory.pockets.pockets[slot] = rustimal_logic::inventory::EMPTY_NO;
            inventory.pockets.set_item_cond(slot, 0);
        }
    }
}

/// `aEGH_not_collect_get`'s fallback: a random piece of the shop lists (the collection check is not modelled).
fn favour_item(rand: &mut crate::game_rand::GameRand) -> u16 {
    use crate::goods::{list, Kind};
    let kinds = [Kind::Furniture, Kind::Carpet, Kind::Wallpaper, Kind::Cloth];
    let kind = kinds[rand.0.random(kinds.len() as i32) as usize];
    let list = list(kind, rand.0.random(4) as usize);
    if list.is_empty() {
        return 0x2807; // a peach, the original's fallback
    }
    list[rand.0.random(list.len() as i32) as usize]
}

/// The talk, as the force and norm talk requests and the talk procs.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), (With<Player>, Without<Spooky>)>,
    mut spooky: Query<(Entity, &Transform, &mut Spooky)>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    mut items: Option<ResMut<FieldItems>>,
    mut house_save: ResMut<crate::home::PlayerHouse>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut event: ResMut<SpookyEvent>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, actor_at, mut me)) = spooky.single_mut() else {
        return;
    };
    let ctx = context(&identity, &disc, &event, None);
    let answered = if dialogue.speaker == Some(actor) {
        dialogue.external_result.take()
    } else {
        None
    };
    if let Some(choice) = answered {
        *focus = InputFocus::Dialogue;
        let chosen = if choice == usize::MAX { 0 } else { choice };
        match std::mem::take(&mut flow.step) {
            Step::Line | Step::Idle => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
            }
            Step::Favour(next) => {
                // `aEGH_give_me_wait`: the spirits go to him first.
                *dialogue = DialogueState::default();
                take_spirits(&mut inventory);
                handover.start(Request {
                    item: Item::Goods(wisp::wisp::SPIRIT0),
                    giver: Party::Player,
                    taker: Party::Npc(actor),
                });
                flow.step = Step::Handing(chosen, next);
            }
            Step::Result(item, clear) => {
                *dialogue = DialogueState::default();
                if clear {
                    clear_weeds(items.as_deref_mut());
                }
                if let Some(item) = item {
                    inventory.add(item);
                    info!("Spooky gives {item:#06x} ({})", crate::items::name(item));
                    handover.start(Request {
                        item: Item::Goods(item),
                        giver: Party::Npc(actor),
                        taker: Party::Player,
                    });
                    flow.step = Step::ItemHanding;
                } else {
                    *focus = InputFocus::World;
                }
            }
            Step::Roof(from, next) => {
                // `aEGH_select_roof`: three colours at a time; the fourth answer shows the next three.
                if chosen >= 3 {
                    let from = (from + 3) % 12;
                    let msg = next.get(chosen).copied().flatten();
                    let ctx = &ctx;
                    match msg {
                        Some(m) => {
                            flow.step = say(&disc, &mut dialogue, actor, ctx, m as u16)
                                .map_or(Step::Line, |n| Step::Roof(from, n));
                        }
                        None => *focus = InputFocus::World,
                    }
                } else {
                    let pal = ((from + chosen) % 12) as u8;
                    house_save.0.outlook_pal = pal;
                    house_save.0.next_outlook_pal = pal;
                    info!("Spooky paints the roof, palette {pal}");
                    *dialogue = DialogueState::default();
                    *focus = InputFocus::World;
                }
            }
            Step::Handing(..) | Step::ItemHanding => {}
        }
        return;
    }
    match flow.step {
        Step::Handing(chosen, ref next) => {
            if handover.busy() {
                *focus = InputFocus::Dialogue;
                return;
            }
            let next = next.clone();
            *focus = InputFocus::Dialogue;
            event.returned = true;
            // The favours: 0 weeds, 1 roof, 2 an item.
            match chosen {
                0 => {
                    let n = items.as_deref().map_or(0, weeds) as u32;
                    let msg = wisp::weed_msg(n);
                    say(&disc, &mut dialogue, actor, &ctx, msg);
                    flow.step = Step::Result(None, true);
                }
                1 => {
                    let msg = next.get(1).copied().flatten();
                    match msg {
                        Some(m) => {
                            flow.step = say(&disc, &mut dialogue, actor, &ctx, m as u16)
                                .map_or(Step::Line, |n| Step::Roof(0, n));
                        }
                        None => {
                            *focus = InputFocus::World;
                            flow.step = Step::Idle;
                        }
                    }
                }
                _ => {
                    let item = favour_item(&mut rand);
                    let ctx = context(&identity, &disc, &event, Some(item));
                    let msg = next.get(2).copied().flatten().unwrap_or(0x2EF6);
                    say(&disc, &mut dialogue, actor, &ctx, msg as u16);
                    flow.step = Step::Result(Some(item), false);
                }
            }
            return;
        }
        Step::ItemHanding => {
            if handover.busy() {
                *focus = InputFocus::Dialogue;
            } else {
                *focus = InputFocus::World;
                flow.step = Step::Idle;
            }
            return;
        }
        Step::Idle => {}
        _ => return,
    }
    if !focus.is_world() {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    let delta = (actor_at.translation - at.translation).with_y(0.0);
    let dist = delta.length();
    let dy = (actor_at.translation.y - at.translation.y).abs();
    let near = |range: f32| dist < range && dy < 1.5;

    // The approach: forced talks by phase.
    let forced = match me.phase {
        Phase::Hidden if near(SPEAK_FROM) => Some((0x2ED3 + rand.0.random(5) as u16, Phase::Near)),
        Phase::Near if near(SHOW_FROM) => Some((0x2EE2 + rand.0.random(5) as u16, Phase::Found)),
        Phase::Near if dist > WALKED_AWAY => {
            me.phase = Phase::Far;
            None
        }
        Phase::Far if near(COME_BACK_FROM) => {
            Some((0x2ED8 + rand.0.random(5) as u16, Phase::Waiting))
        }
        Phase::Waiting if dist < SPEAK_FROM => {
            me.phase = Phase::Near;
            None
        }
        _ => None,
    };
    if let Some((msg, after)) = forced {
        *focus = InputFocus::Dialogue;
        if (0x2EE2..0x2EE7).contains(&msg) {
            // He shows himself (`aEGH_bitset_func(aEGH_BIT_FOUND_GHOST)`).
            event.found = true;
        }
        me.phase = after;
        say(&disc, &mut dialogue, actor, &ctx, msg);
        flow.step = Step::Line;
        return;
    }
    // A talk the player starts once he is found (`aEGH_norm_talk_request`).
    if me.phase != Phase::Found || !actions.just_pressed(Action::Interact) {
        return;
    }
    if dist > NORMAL_TALK.max(1.6) || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    let n = spirits(&inventory);
    let msg = wisp::greet_msg(n, rand.0.random(5) as u8);
    let asked = say(&disc, &mut dialogue, actor, &ctx, msg);
    flow.step = if n >= u32::from(wisp::wisp::SPIRIT_NUM) {
        Step::Favour(asked.unwrap_or_default())
    } else {
        Step::Line
    };
}

/// `Save_Set(clear_grass, TRUE)`: the weeds are cleared.
fn clear_weeds(items: Option<&mut FieldItems>) {
    let Some(items) = items else { return };
    let mut cleared = 0;
    for acre in items.acres.iter_mut() {
        for item in acre.iter_mut() {
            if matches!(*item, 0x0008..=0x000A) {
                *item = rustimal_logic::inventory::EMPTY_NO;
                cleared += 1;
            }
        }
    }
    info!("Spooky clears {cleared} weeds");
}

pub struct SpookyPlugin;

impl Plugin for SpookyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .init_resource::<SpookyEvent>()
            .add_systems(Startup, apply_loaded)
            .add_systems(
                Update,
                visit.after(crate::event_today::refresh).in_set(GameplaySet),
            )
            .add_systems(Update, face.in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
