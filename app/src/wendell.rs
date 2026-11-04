//! Wendell (`SP_NPC_ARTIST`), the hungry artist.
//!
//! He is out while the original scheduler says `mEv_EVENT_ARTIST` is active (06:00 on his day until 05:59 the next
//! morning; `event_schedule` with the special-event calendar), on free ground like Joan and Katrina.
//!
//! Talking (`ac_ev_artist_move.c_inc`, kernel `event_visitors`): a player who already has his wallpaper gets `0x2FB`.
//! Otherwise `0x2F5` and a question: Give opens the pockets in the give mode, No is `0x2F6`. Whatever is picked leaves
//! the pockets (`mPr_SetPossessionItem(.., EMPTY_NO)`): nothing `0x2F6`; a fish (`ITEM1` category 3) turns into a
//! wallpaper of the event list that is not in the event's `walls` yet, in the same slot (`0x2F9`, he hands it over,
//! `0x2FA`); fruit `0x2F8` (eaten); a tool `0x2FC` and it is handed back into the first free pocket; anything else
//! `0x2F7`. The event's save (`mEv_artist_c`: used, pids, walls) is kept as [`ArtistEvent`] per special event.
//!
//! Not built: the event manager's walk-away after the talk (`mEv_STATUS_TALK`).

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::event_visitors as native;
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
use rustimal_logic::inventory::EMPTY_NO;

/// ALL_NPC_NUM + (SP_NPC_ARTIST - SP_NPC_START).
const WENDELL_NPC: u32 = 238;
/// `l_sp_actor_name` sound id of `SP_NPC_ARTIST`.
const WENDELL_SOUND: u8 = 3;
/// `mSP_LIST_EVENT`: the wallpaper list he draws from.
const EVENT_LIST: usize = 3;

/// The event's save area (`mEv_artist_c`), for the one player: the wallpapers already given. It belongs to one
/// special event (`rolled`: when the calendar rolled it) and starts empty with the next.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtistEvent {
    /// `dates[SPECIAL0]` and `dates[SPECIAL1]` of the event this belongs to.
    pub rolled: (u16, u16),
    /// `walls[..used]`.
    pub walls: Vec<u16>,
}

impl ArtistEvent {
    /// `aEART_check_present`: this player already got a wallpaper.
    fn has_present(&self) -> bool {
        !self.walls.is_empty()
    }
}

#[derive(Component)]
struct Wendell {
    /// The game day he came out for.
    day: i64,
}

/// Startup: the saved event area.
fn apply_loaded(loaded: Res<LoadedSave>, mut artist: ResMut<ArtistEvent>) {
    if let Some(save) = loaded.0.as_ref() {
        *artist = save.artist_event.clone().unwrap_or_default();
    }
}

/// Brings him out for his day and sends him off when the event ends.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    items: Option<Res<crate::field_items::FieldItems>>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Wendell)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let out = today.is_active(ev::ARTIST, now.hour)
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    // After midnight he is still the previous day's visitor.
    let day = if now.hour < 6 {
        now.day_number - 1
    } else {
        now.day_number
    };
    for (entity, w) in &existing {
        if !out || w.day != day {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    let Some(items) = items else { return };
    let Some(at) = crate::joan::spot(&town, &items, day) else {
        return;
    };
    commands.spawn((
        Name::new("Wendell"),
        Wendell { day },
        NpcClip::default(),
        NpcGait::default(),
        NpcModel(WENDELL_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y),
    ));
    info!("Wendell is out at {at:?}");
}

/// A new special event starts the event area empty (`mEv_reserve_save_area`).
fn reset_for_new_event(shop: Option<Res<crate::shop::ShopData>>, mut artist: ResMut<ArtistEvent>) {
    let Some((ev::ARTIST, slots)) = shop.and_then(|s| s.events.special_event()) else {
        return;
    };
    let rolled = (slots[0], slots[1]);
    if artist.rolled != rolled {
        *artist = ArtistEvent {
            rolled,
            walls: Vec::new(),
        };
    }
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// `0x2F5` is up with its Give / No question.
    Hungry,
    /// The pockets are open in the give mode.
    Picking,
    /// A line is up, then `After`.
    Said(After),
    /// An item is crossing.
    Hand(After),
}

#[derive(Clone, Copy, Debug)]
enum After {
    /// The talk ends.
    Close,
    /// Wendell hands the wallpaper over, then `0x2FA`.
    GiveWall(u16),
    /// Wendell hands the tool back.
    ReturnTool(u16),
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
}

fn context(identity: &crate::save::PlayerIdentity) -> MsgContext {
    let now = crate::clock::now();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Wendell".into(),
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

/// Shows disc message `msg` in Wendell's voice; true when it ends on a question.
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
        "Wendell",
        (WENDELL_NPC, WENDELL_SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Wendell message {msg:#06x}");
    asked.is_some()
}

/// `aEART_item_check`: the item's class (`None`: not a classified one, i.e. other).
fn classify(item: u16) -> u8 {
    native::wendell_classify(((item >> 12) & 0xF) as u8, ((item >> 8) & 0xF) as u8)
}

/// He turns to face the player while they talk.
fn face(
    time: Res<Time>,
    flow: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<Wendell>)>,
    mut wendell: Query<&mut Transform, With<Wendell>>,
) {
    let (Ok(at), Ok(mut transform)) = (player.single(), wendell.single_mut()) else {
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

/// The talk, as `aEART_set_talk_info_talk_request`, `aEART_say_hungry` and the menu steps.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), With<Player>>,
    wendell: Query<(Entity, &Transform), With<Wendell>>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut pockets: ResMut<PocketsUi>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    mut artist: ResMut<ArtistEvent>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, actor_at)) = wendell.single() else {
        return;
    };
    let ctx = context(&identity);
    let end = |dialogue: &mut DialogueState, focus: &mut InputFocus, flow: &mut Talk| {
        *dialogue = DialogueState::default();
        *focus = InputFocus::World;
        flow.step = Step::Idle;
    };

    // The pockets are open: `aEART_menu_close_wait` / `aEART_msg_win_open_wait`.
    if matches!(flow.step, Step::Picking) {
        if pockets.open {
            return;
        }
        pockets.give_any = false;
        *focus = InputFocus::Dialogue;
        let slot = pockets.sell_slot.take();
        let item = slot.map_or(EMPTY_NO, |s| inventory.pockets.pockets[s]);
        let (msg, after) = if item == EMPTY_NO {
            (native::WENDELL_MSG[0], After::Close)
        } else {
            let slot = slot.unwrap_or_default();
            // `mPr_SetPossessionItem(.., slot, EMPTY_NO, ..)`: it leaves the pockets whatever it is.
            inventory.pockets.pockets[slot] = EMPTY_NO;
            inventory.pockets.set_item_cond(slot, 0);
            let class = classify(item);
            let msg = native::wendell_msg_for(class);
            info!("Wendell is given {item:#06x} (class {class})");
            let after = match class {
                native::wendell_class::FISH => {
                    // `aEART_demo_start_wait_init`: a wallpaper not given yet takes the slot.
                    let list = crate::goods::list(crate::goods::Kind::Wallpaper, EVENT_LIST);
                    let fresh: Vec<u16> = list
                        .iter()
                        .copied()
                        .filter(|w| !artist.walls.contains(w))
                        .collect();
                    let pool = if fresh.is_empty() { &list } else { &fresh };
                    if pool.is_empty() {
                        After::Close
                    } else {
                        let present = pool[rand.0.random(pool.len() as i32) as usize];
                        inventory.pockets.pockets[slot] = present;
                        inventory.pockets.set_item_cond(slot, 0);
                        artist.walls.push(present);
                        info!("Wendell gives the wallpaper {present:#06x}");
                        After::GiveWall(present)
                    }
                }
                native::wendell_class::TOOL => After::ReturnTool(item),
                _ => After::Close,
            };
            (msg, after)
        };
        say(&disc, &mut dialogue, actor, &ctx, msg);
        flow.step = Step::Said(after);
        return;
    }

    // An item is crossing.
    if let Step::Hand(after) = flow.step {
        *focus = InputFocus::Dialogue;
        if handover.busy() {
            return;
        }
        match after {
            After::GiveWall(_) => {
                // `aEART_demo_start_wait`: `0x2FA` follows the hand-over.
                say(&disc, &mut dialogue, actor, &ctx, 0x02FA);
                flow.step = Step::Said(After::Close);
            }
            After::ReturnTool(item) => {
                // `mPr_SetFreePossessionItem`: the tool is back in the first free pocket.
                inventory.add(item);
                end(&mut dialogue, &mut focus, &mut flow);
            }
            After::Close => end(&mut dialogue, &mut focus, &mut flow),
        }
        return;
    }

    // Only his own answers; other talkers in town take theirs.
    let answer = if dialogue.speaker == Some(actor) {
        dialogue.external_result.take()
    } else {
        None
    };
    if let Some(choice) = answer {
        *focus = InputFocus::Dialogue;
        match std::mem::take(&mut flow.step) {
            Step::Hungry => match choice {
                0 => {
                    // `aEART_say_hungry`: Give opens the pockets.
                    *pockets = PocketsUi {
                        open: true,
                        give_any: true,
                        ..default()
                    };
                    *dialogue = DialogueState::default();
                    *focus = InputFocus::Pockets;
                    crate::sound::se(crate::sound::se_id::MENU_PAUSE);
                    flow.step = Step::Picking;
                }
                _ => {
                    say(&disc, &mut dialogue, actor, &ctx, native::WENDELL_MSG[0]);
                    flow.step = Step::Said(After::Close);
                }
            },
            Step::Said(After::GiveWall(present)) => {
                *dialogue = DialogueState::default();
                handover.start(Request {
                    item: Item::Goods(present),
                    giver: Party::Npc(actor),
                    taker: Party::Player,
                });
                flow.step = Step::Hand(After::GiveWall(present));
            }
            Step::Said(After::ReturnTool(item)) => {
                *dialogue = DialogueState::default();
                handover.start(Request {
                    item: Item::Goods(item),
                    giver: Party::Npc(actor),
                    taker: Party::Player,
                });
                flow.step = Step::Hand(After::ReturnTool(item));
            }
            Step::Said(After::Close) | Step::Idle | Step::Picking | Step::Hand(_) => {
                end(&mut dialogue, &mut focus, &mut flow);
            }
        }
        return;
    }
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
    if delta.length() > 1.6 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    // `aEART_set_talk_info_talk_request` and `aEART_talk_init`.
    if artist.has_present() {
        say(&disc, &mut dialogue, actor, &ctx, 0x02FB);
        flow.step = Step::Said(After::Close);
    } else {
        let asked = say(&disc, &mut dialogue, actor, &ctx, 0x02F5);
        flow.step = if asked {
            Step::Hungry
        } else {
            Step::Said(After::Close)
        };
    }
}

pub struct WendellPlugin;

impl Plugin for WendellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .init_resource::<ArtistEvent>()
            .add_systems(Startup, apply_loaded)
            .add_systems(
                Update,
                (
                    reset_for_new_event,
                    visit.after(crate::event_today::refresh),
                )
                    .chain()
                    .in_set(GameplaySet),
            )
            .add_systems(Update, face.in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
