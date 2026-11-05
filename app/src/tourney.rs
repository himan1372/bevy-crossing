//! The fishing tourney's host (`SP_NPC_ANGLER`, `ac_ev_angler.c`, `ac_ev_angler_move.c_inc`, `ac_turi_clip.c_inc`).
//!
//! He stands at the water while the scheduler says `mEv_EVENT_FISHING_TOURNEY_1` (June) or `_2` (November) is active
//! (06:00-17:59 on the event Sunday). The record starts as a villager's catch (`aTRC_clip_random_topsize`:
//! `mFR_make_NpcRecord`, the rank by the hour). Talking: after 18:00 `0x10EE`; the first talk `0x10E8` (this enters the
//! player); afterwards `0x10EF`, a question. No: the record `0x111E` (the player holds it) or `0x10F0`. Yes: the pockets
//! open. Nothing picked: the record again. A bass (`ITM_FISH05..=07`): `aTRC_clip_get_msgno` (his measuring line, the
//! size by rank filling string 2), then a size above the record wins (`0x1122` or the fish's message `+ 2`, the record
//! is set and a random cloth or furniture of the shop lists is given) and anything else loses (`0x111F..=0x1121` when
//! the player holds the record, else the message `+ 1`). Another fish or item: its message (`0x10F2` for other
//! items) and a choice to take it back (the first answer) or leave it with him. A tool: `0x17E5` and it comes back.
//!
//! Not built: the five villagers who fish, the fishing stands and trophy, the rod's own size (every bass is measured
//! by its species rank, as the original does), the camera.

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::fishing_tourney as native;
use rustimal_logic::inventory::EMPTY_NO;
use serde::{Deserialize, Serialize};

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::messages::MsgContext;
use crate::player::Player;
use crate::pockets::PocketsUi;
use crate::save::LoadedSave;
use crate::shop::ShopLocation;
use crate::shop_nook::handover::{HandOver, Item, Party, Request};
use crate::states::GameplaySet;
use crate::town::Town;

/// ALL_NPC_NUM + (SP_NPC_ANGLER - SP_NPC_START).
const CHIP_NPC: u32 = 238 + 29;
const SOUND: u8 = 2;
/// The measuring unit string of `mString_Load_NumberStringAddUnitFromRom(.., 0x29E)`.
const UNIT_STRING: usize = 0x29E;

/// The event's save area (`aEANG_event_data_c`) for the one player.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TourneyEvent {
    /// The tourney this belongs to (year, month).
    pub key: (u16, u8),
    /// `size`: the record in inches (0: none yet).
    pub size: i32,
    /// The record holder's name (a villager's, or the player's).
    pub top: String,
    /// The player holds the record (`now_top_check`).
    pub top_player: bool,
    /// `talk_flag`: the player has entered.
    pub entered: bool,
}

#[derive(Component)]
struct Chip {
    key: (u16, u8),
}

/// Startup: the saved event area.
fn apply_loaded(loaded: Res<LoadedSave>, mut event: ResMut<TourneyEvent>) {
    if let Some(save) = loaded.0.as_ref() {
        *event = save.tourney_event.clone().unwrap_or_default();
    }
}

/// Cheap integer mixer for the per-event picks.
fn mix(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ce_b9fe_1a85_1a85);
    x ^ (x >> 33)
}

/// Clear ground next to the water, in an acre picked from `seed`.
fn shore_spot(town: &Town, seed: u64) -> Option<Vec3> {
    let mut tiles: Vec<_> = town.river_tiles.iter().copied().collect();
    tiles.sort_unstable();
    if tiles.is_empty() {
        return None;
    }
    let first = mix(seed) as usize % tiles.len();
    for step in 0..tiles.len() {
        let (ax, az, cx, cz) = tiles[(first + step) % tiles.len()];
        let water = Town::cell_to_world(ax, az, cx, cz);
        for (dx, dz) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
            let at = water + Vec3::new(dx, 0.0, dz);
            if town.is_clear_ground(at) && !town.in_retail_obstacle(at) {
                return Some(at.with_y(town.ground_y(at)));
            }
        }
    }
    None
}

fn running(today: &TodayEvents, hour: u8) -> bool {
    today.is_active(ev::FISHING_TOURNEY_1, hour) || today.is_active(ev::FISHING_TOURNEY_2, hour)
}

/// `aTRC_clip_fish_rndsize` for a rank.
fn rnd_size(rank: i32, rand: &mut crate::game_rand::GameRand) -> i32 {
    native::pc_fish_size_from_rank(rank, rand.0.fqrand())
}

/// `mFR_make_NpcRecord`.
fn npc_record(hour: u8, rand: &mut crate::game_rand::GameRand) -> i32 {
    let rank = match native::pc_fish_npc_rank_branch(i32::from(hour)) {
        0 => 0,
        1 => rand.0.random(2),
        _ => rand.0.random(3),
    };
    rnd_size(rank, rand)
}

/// Brings Chip to the water for the event.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Chip)>,
    villagers: Query<&crate::npc::Villager>,
    mut event: ResMut<TourneyEvent>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let on = running(&today, now.hour);
    let key = (now.year, now.month);
    let out =
        on && *state.get() == ShopLocation::Town && *house.get() == crate::house::HouseState::Town;
    for (entity, c) in &existing {
        if !out || c.key != key {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    // A new tourney starts a new area with a villager's record (`aTRC_clip_random_topsize`).
    if event.key != key {
        let names: Vec<&str> = villagers.iter().map(|v| v.name).collect();
        let top = if names.is_empty() {
            "Chip".to_string()
        } else {
            names[rand.0.random(names.len() as i32) as usize].to_string()
        };
        *event = TourneyEvent {
            key,
            size: npc_record(now.hour, &mut rand),
            top,
            top_player: false,
            entered: false,
        };
        info!("Tourney record: {} inches, {}", event.size, event.top);
    }
    let Some(at) = shore_spot(&town, u64::from(key.0) * 16 + u64::from(key.1)) else {
        return;
    };
    commands.spawn((
        Name::new("Chip"),
        Chip { key },
        NpcClip::default(),
        NpcGait::default(),
        crate::event_body::EventBody::default(),
        NpcModel(CHIP_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y),
    ));
    info!("The tourney host is at the water, {at:?}");
}

#[derive(Clone, Debug)]
enum After {
    /// The talk ends.
    Close,
    /// A bass line is up: the measurement follows (`size` was drawn with it).
    Measure(u16, i32),
    /// The winner's line is up: the reward is handed over next.
    Reward(u16),
    /// A tool line is up: it goes back.
    Return(u16),
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// A line is up.
    Said(After),
    /// `0x10EF` is up: Yes / No.
    Asked,
    /// The pockets are open in the give mode.
    Picking,
    /// The offered item is crossing to him.
    Taking(u16),
    /// A fish or item line is up with a keep / leave choice.
    Choice(u16),
    /// An item is crossing; the talk ends after.
    Giving,
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
}

fn context(
    identity: &crate::save::PlayerIdentity,
    disc: &GameDisc,
    event: &TourneyEvent,
) -> MsgContext {
    let now = crate::clock::now();
    let unit = disc
        .text
        .as_ref()
        .and_then(|t| t.string(UNIT_STRING))
        .unwrap_or_default();
    let mut free = vec![String::new(); 20];
    free[0] = if event.top_player {
        identity.player.clone()
    } else {
        event.top.clone()
    };
    free[1] = format!("{}{unit}", event.size);
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Chip".into(),
        free,
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
    msg: u16,
) -> bool {
    let Some(text) = disc.text.as_ref() else {
        return false;
    };
    let (pages, asked) = text.talk_asking(usize::from(msg), ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "Chip",
        (CHIP_NPC, SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Chip message {msg:#06x}");
    asked.is_some()
}

/// He faces the player while they talk.
fn face(
    time: Res<Time>,
    flow: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<Chip>)>,
    mut chip: Query<&mut Transform, With<Chip>>,
) {
    let (Ok(at), Ok(mut transform)) = (player.single(), chip.single_mut()) else {
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

/// The record's line when nothing is brought (`0x111E` for the holder, else `0x10F0`).
fn record_msg(event: &TourneyEvent) -> u16 {
    if event.top_player {
        0x111E
    } else {
        0x10F0
    }
}

/// The reward (`aEANG_SelectRandomItem`): a cloth or a furniture piece of the shop lists, half and half.
fn reward(rand: &mut crate::game_rand::GameRand) -> Option<u16> {
    use crate::goods::{delivery_item, Kind};
    let kind = if rand.0.fqrand() > 0.5 {
        Kind::Cloth
    } else {
        Kind::Furniture
    };
    let mut below = |n: u32| rand.0.random(n as i32) as u32;
    delivery_item(kind, &[], &mut below)
}

/// The talk, as `aEANG_talk_request` and the talk procs.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), (With<Player>, Without<Chip>)>,
    chip: Query<(Entity, &Transform), With<Chip>>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut pockets: ResMut<PocketsUi>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut event: ResMut<TourneyEvent>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, actor_at)) = chip.single() else {
        return;
    };
    let now = crate::clock::now();

    // The pockets are open: `aEANG_menu_close_wait`, `aEANG_msg_win_open_wait`.
    if matches!(flow.step, Step::Picking) {
        if pockets.open {
            return;
        }
        pockets.give_any = false;
        *focus = InputFocus::Dialogue;
        match pockets.sell_slot.take() {
            None => {
                let ctx = context(&identity, &disc, &event);
                say(&disc, &mut dialogue, actor, &ctx, record_msg(&event));
                flow.step = Step::Said(After::Close);
            }
            Some(slot) => {
                let item = inventory.pockets.pockets[slot];
                info!("Chip is shown {item:#06x}");
                // The item is taken to be looked at (the take mode removes it from the pockets).
                inventory.pockets.pockets[slot] = EMPTY_NO;
                inventory.pockets.set_item_cond(slot, 0);
                handover.start(Request {
                    item: Item::Goods(item),
                    giver: Party::Player,
                    taker: Party::Npc(actor),
                });
                flow.step = Step::Taking(item);
            }
        }
        return;
    }
    // The item has reached him: `aEANG_item_check`.
    if let Step::Taking(item) = flow.step {
        if handover.busy() {
            *focus = InputFocus::Dialogue;
            return;
        }
        let ctx = context(&identity, &disc, &event);
        let is_fish = (0x2300..0x2400).contains(&item);
        let is_tool = crate::halloween::is_tool(item);
        let kind = native::pc_angler_item_type(is_fish, is_tool, i32::from(item));
        let line = native::pc_angler_fish_msgno(i32::from(item)) as u16;
        match kind {
            native::ITEM_TYPE_BASS => {
                // `aEANG_demo_measure_init`: the size by the species' rank.
                let size = rnd_size(native::pc_fish_measure_rank(i32::from(item)), &mut rand);
                let mut ctx = ctx;
                ctx.free[2] = format!(
                    "{size}{}",
                    disc.text
                        .as_ref()
                        .and_then(|t| t.string(UNIT_STRING))
                        .unwrap_or_default()
                );
                say(&disc, &mut dialogue, actor, &ctx, line);
                flow.step = Step::Said(After::Measure(item, size));
            }
            native::ITEM_TYPE_TOOL => {
                say(&disc, &mut dialogue, actor, &ctx, 0x17E5);
                flow.step = Step::Said(After::Return(item));
            }
            native::ITEM_TYPE_FISH => {
                say(&disc, &mut dialogue, actor, &ctx, line);
                flow.step = Step::Choice(item);
            }
            _ => {
                say(&disc, &mut dialogue, actor, &ctx, 0x10F2);
                flow.step = Step::Choice(item);
            }
        }
        return;
    }
    if matches!(flow.step, Step::Giving) {
        if handover.busy() {
            *focus = InputFocus::Dialogue;
        } else {
            *dialogue = DialogueState::default();
            *focus = InputFocus::World;
            flow.step = Step::Idle;
        }
        return;
    }

    // A line was read (or the player answered).
    if dialogue.speaker == Some(actor) {
        if let Some(choice) = dialogue.external_result.take() {
            *focus = InputFocus::Dialogue;
            let give_back = |item: u16,
                             inventory: &mut PlayerInventory,
                             handover: &mut HandOver,
                             dialogue: &mut DialogueState,
                             flow: &mut Talk| {
                // `aEANG_demo_henkyaku`: the item goes back to the pockets.
                *dialogue = DialogueState::default();
                inventory.add(item);
                handover.start(Request {
                    item: Item::Goods(item),
                    giver: Party::Npc(actor),
                    taker: Party::Player,
                });
                flow.step = Step::Giving;
            };
            match std::mem::take(&mut flow.step) {
                Step::Asked => {
                    if choice == 0 {
                        // `aEANG_msg_to_menu`: the pockets open.
                        *pockets = PocketsUi {
                            open: true,
                            give_any: true,
                            ..default()
                        };
                        *dialogue = DialogueState::default();
                        *focus = InputFocus::Pockets;
                        crate::sound::se(crate::sound::se_id::MENU_PAUSE);
                        flow.step = Step::Picking;
                    } else {
                        let ctx = context(&identity, &disc, &event);
                        say(&disc, &mut dialogue, actor, &ctx, record_msg(&event));
                        flow.step = Step::Said(After::Close);
                    }
                }
                Step::Choice(item) => {
                    if choice == 0 {
                        give_back(
                            item,
                            &mut inventory,
                            &mut handover,
                            &mut dialogue,
                            &mut flow,
                        );
                    } else {
                        // Left with him.
                        *dialogue = DialogueState::default();
                        *focus = InputFocus::World;
                    }
                }
                Step::Said(After::Measure(item, size)) => {
                    // `aEANG_demo_measure`.
                    let ctx = context(&identity, &disc, &event);
                    if native::pc_fish_record_beaten(event.size, size) {
                        let line = native::pc_angler_fish_msgno(i32::from(item)) as u16;
                        let msg = if event.top_player { 0x1122 } else { line + 2 };
                        event.size = size;
                        event.top = identity.player.clone();
                        event.top_player = true;
                        let prize = reward(&mut rand);
                        let mut ctx = ctx;
                        if let Some(p) = prize {
                            ctx.items = vec![crate::items::name(p); 5];
                        }
                        ctx.free[1] = format!("{size}");
                        say(&disc, &mut dialogue, actor, &ctx, msg);
                        flow.step = Step::Said(prize.map_or(After::Close, After::Reward));
                        info!("A new tourney record: {size} inches");
                    } else {
                        let line = native::pc_angler_fish_msgno(i32::from(item));
                        let msg = if event.top_player && (0x2305..=0x2307).contains(&item) {
                            0x111F + (item - 0x2305)
                        } else {
                            line as u16 + 1
                        };
                        say(&disc, &mut dialogue, actor, &ctx, msg);
                        flow.step = Step::Said(After::Close);
                    }
                }
                Step::Said(After::Reward(prize)) => {
                    *dialogue = DialogueState::default();
                    inventory.add(prize);
                    info!("Chip gives {prize:#06x} ({})", crate::items::name(prize));
                    handover.start(Request {
                        item: Item::Goods(prize),
                        giver: Party::Npc(actor),
                        taker: Party::Player,
                    });
                    flow.step = Step::Giving;
                }
                Step::Said(After::Return(item)) => {
                    give_back(
                        item,
                        &mut inventory,
                        &mut handover,
                        &mut dialogue,
                        &mut flow,
                    );
                }
                Step::Said(After::Close) | Step::Idle | Step::Picking | Step::Taking(_) => {
                    *dialogue = DialogueState::default();
                    *focus = InputFocus::World;
                }
                Step::Giving => {}
            }
        }
        return;
    }

    // The player starts a talk.
    if !matches!(flow.step, Step::Idle)
        || !focus.is_world()
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    let delta = (actor_at.translation - at.translation).with_y(0.0);
    if delta.length() > 1.8 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    let ctx = context(&identity, &disc, &event);
    if !native::pc_fish_time_open(i32::from(now.hour)) {
        say(&disc, &mut dialogue, actor, &ctx, 0x10EE);
        flow.step = Step::Said(After::Close);
    } else if event.entered {
        say(&disc, &mut dialogue, actor, &ctx, 0x10EF);
        flow.step = Step::Asked;
    } else {
        // `entry_on`.
        event.entered = true;
        say(&disc, &mut dialogue, actor, &ctx, 0x10E8);
        flow.step = Step::Said(After::Close);
    }
}

pub struct TourneyPlugin;

impl Plugin for TourneyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .init_resource::<TourneyEvent>()
            .add_systems(Startup, apply_loaded)
            .add_systems(
                Update,
                visit.after(crate::event_today::refresh).in_set(GameplaySet),
            )
            .add_systems(Update, face.in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
