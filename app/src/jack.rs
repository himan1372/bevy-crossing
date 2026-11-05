//! Jack the pumpkin (`SP_NPC_HALLOWEEN`, `ac_ev_pumpkin.c`, `ac_ev_pumpkin_talk.c_inc`) on Halloween.
//!
//! He is out while the original scheduler says `mEv_EVENT_HALLOWEEN` is active (October 31, 18:00 until midnight),
//! shown at the wade like Jingle (`halloween_in`), and walks to another beach spot after each talk
//! (`halloween_behind`). The port places him as Jingle is placed.
//!
//! Talking (`aEPK_set_talk_info`): a talk in the acre of the last one is `0x9BC + RANDOM(6)`; otherwise the greeting
//! `0x98A`, then the question `0x9B5` (Treat / Trick). Treat opens the pockets in the give mode. Nothing picked:
//! `0x9B6`, then the trick. Candy (`ITM_FOOD_CANDY`): taken, `0x9B9`, and a present is handed over (one in twelve a
//! carpet `ITM_CARPET17`, one in twelve a wall `ITM_WALL17`, else a piece of the Halloween furniture list), `0x9BA`.
//! A tool (net, rod, golden net, golden rod): handed over, `0x9BB`, handed back, then the trick. Anything else:
//! taken, `0x9B8`, then the trick. The trick is the one of the costumed villagers (`aEPK_get_trick_type`: a swap of
//! a pocket for a Halloween2 furniture piece or the costume shirt), then `0x9B7`.
//!
//! Port adaptations: the present box (`ITM_PRESENT`) is replaced by handing the piece itself over; no cloth-change
//! animation. Not built: his wandering between talks.

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::halloween as native;
use rustimal_logic::inventory::EMPTY_NO;

use crate::character::{NpcClip, NpcGait, NpcModel, PlayerLook};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::goods::Kind;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::messages::MsgContext;
use crate::player::Player;
use crate::pockets::PocketsUi;
use crate::shop::ShopLocation;
use crate::shop_nook::handover::{HandOver, Item, Party, Request};
use crate::states::GameplaySet;
use crate::town::{Town, ACRE_SIZE};
use rustimal_logic::town_gen::{ACRE_DEPTH, ACRE_WIDTH};

/// ALL_NPC_NUM + (SP_NPC_HALLOWEEN - SP_NPC_START).
const JACK_NPC: u32 = 238 + 37;
/// `l_sp_actor_name` sound id of `SP_NPC_HALLOWEEN`.
const JACK_SOUND: u8 = 2;
/// `mSP_LIST_HALLOWEEN`.
const HALLOWEEN_LIST: usize = 6;
/// `ITM_CARPET17` and `ITM_WALL17`.
const CARPET17: u16 = 0x2611;
const WALL17: u16 = 0x2711;

#[derive(Component)]
struct Jack {
    day: i64,
    /// Talks so far, to pick the next spot.
    moves: i64,
    /// `common_p->bx, bz`: the acre of the last talk.
    block: (i32, i32),
}

/// Brings him out for the evening and sends him off when the event ends.
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Jack)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let out = today.is_active(ev::HALLOWEEN, now.hour)
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    for (entity, j) in &existing {
        if !out || j.day != now.day_number {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    let Some(at) = crate::gulliver::beach_spot(&town, now.day_number ^ 0x6a61) else {
        return;
    };
    commands.spawn((
        Name::new("Jack"),
        Jack {
            day: now.day_number,
            moves: 0,
            block: (0, 0),
        },
        NpcClip::default(),
        NpcGait::default(),
        crate::event_body::EventBody::default(),
        NpcModel(JACK_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y),
    ));
    info!("Jack is out at {at:?}");
}

#[derive(Clone, Copy, Debug)]
enum After {
    /// The trick follows.
    Trick,
    /// The talk ends.
    Close,
    /// A tool was taken for show: it goes back, then the trick.
    ReturnTool(u16),
    /// The candy is in his hands: the present follows.
    Present(u16),
}

#[derive(Clone, Copy, Debug)]
enum Then {
    /// The line and what follows it.
    Say(u16, After),
    /// The trick.
    Trick,
    /// The present is crossing; `0x9BA` follows.
    Presented,
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// `0x98A` is up: the question follows.
    Greeting,
    /// The question is up (`next`: each answer's own follow-up line).
    Asked(Vec<Option<usize>>),
    /// The pockets are open in the give mode.
    Picking,
    /// A line is up.
    Said(After),
    /// An item is crossing.
    Hand(Then),
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
}

fn context(identity: &crate::save::PlayerIdentity) -> MsgContext {
    let now = crate::clock::now();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Jack".into(),
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

/// Shows disc message `msg` in Jack's voice; the choices' follow-ups come back when it asks.
fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    identity: &crate::save::PlayerIdentity,
    msg: u16,
) -> Vec<Option<usize>> {
    let ctx = context(identity);
    let Some(text) = disc.text.as_ref() else {
        return Vec::new();
    };
    let (pages, asked) = text.talk_asking(usize::from(msg), &ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "Jack",
        (JACK_NPC, JACK_SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Jack message {msg:#06x}");
    asked.map(|(_, c)| c.next).unwrap_or_default()
}

/// He turns to face the player while they talk.
fn face(
    time: Res<Time>,
    flow: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<Jack>)>,
    mut jack: Query<&mut Transform, With<Jack>>,
) {
    let (Ok(at), Ok(mut transform)) = (player.single(), jack.single_mut()) else {
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

/// The acre a world position is in (`play->block_table`).
fn acre_of(at: Vec3) -> (i32, i32) {
    let ox = ACRE_WIDTH as f32 * ACRE_SIZE / 2.0;
    let oz = ACRE_DEPTH as f32 * ACRE_SIZE / 2.0;
    (
        ((at.x + ox) / ACRE_SIZE).floor() as i32,
        ((at.z + oz) / ACRE_SIZE).floor() as i32,
    )
}

/// `aEPK_decide_present`.
fn decide_present(rand: &mut crate::game_rand::GameRand) -> Option<u16> {
    match rand.0.random(12) {
        0 => Some(CARPET17),
        1 => Some(WALL17),
        _ => {
            let list = crate::goods::list(Kind::Furniture, HALLOWEEN_LIST);
            (!list.is_empty()).then(|| list[rand.0.random(list.len() as i32) as usize])
        }
    }
}

/// `walk_actor_at_wade`: he goes to another beach spot after a talk.
fn relocate(town: &Town, at: &mut Transform, me: &mut Jack) {
    me.moves += 1;
    let here = acre_of(at.translation);
    for step in 0..4 {
        let seed = (me.day ^ 0x6a61).wrapping_add((me.moves + step) * 7919);
        if let Some(spot) = crate::gulliver::beach_spot(town, seed) {
            if acre_of(spot) != here || step == 3 {
                at.translation = spot + Vec3::Y;
                info!("Jack walks to {spot:?}");
                return;
            }
        }
    }
}

/// The talk, as `aEPK_talk_request`, `aEPK_set_talk_info` and the talk procs.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    town: Res<Town>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), (With<Player>, Without<Jack>)>,
    mut jack: Query<(Entity, &mut Transform, &mut Jack)>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut pockets: ResMut<PocketsUi>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    mut look: ResMut<PlayerLook>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, mut actor_at, mut me)) = jack.single_mut() else {
        return;
    };
    let end = |dialogue: &mut DialogueState,
               focus: &mut InputFocus,
               flow: &mut Talk,
               actor_at: &mut Transform,
               me: &mut Jack| {
        *dialogue = DialogueState::default();
        *focus = InputFocus::World;
        flow.step = Step::Idle;
        relocate(&town, actor_at, me);
    };
    // The trick, then `0x9B7`.
    let do_trick = |dialogue: &mut DialogueState,
                    flow: &mut Talk,
                    inventory: &mut PlayerInventory,
                    look: &mut PlayerLook,
                    rand: &mut crate::game_rand::GameRand| {
        crate::halloween::trick(inventory, look, rand);
        say(&disc, dialogue, actor, &identity, 0x09B7);
        flow.step = Step::Said(After::Close);
    };

    // The pockets are open: `aEPK_menu_close_wait_talk_proc`.
    if matches!(flow.step, Step::Picking) {
        if pockets.open {
            return;
        }
        pockets.give_any = false;
        *focus = InputFocus::Dialogue;
        match pockets.sell_slot.take() {
            None => {
                say(&disc, &mut dialogue, actor, &identity, 0x09B6);
                flow.step = Step::Said(After::Trick);
            }
            Some(slot) => {
                let item = inventory.pockets.pockets[slot];
                info!("Jack is offered {item:#06x}");
                let (msg, after) = if item == native::items::CANDY {
                    match decide_present(&mut rand) {
                        Some(present) => (0x09B9, After::Present(present)),
                        None => (0x09B9, After::Close),
                    }
                } else if crate::halloween::is_tool(item) {
                    (0x09BB, After::ReturnTool(item))
                } else {
                    (0x09B8, After::Trick)
                };
                if !crate::halloween::is_tool(item) {
                    inventory.pockets.pockets[slot] = EMPTY_NO;
                    inventory.pockets.set_item_cond(slot, 0);
                }
                handover.start(Request {
                    item: Item::Goods(item),
                    giver: Party::Player,
                    taker: Party::Npc(actor),
                });
                flow.step = Step::Hand(Then::Say(msg, after));
            }
        }
        return;
    }

    // An item is crossing.
    if let Step::Hand(then) = flow.step {
        if handover.busy() {
            *focus = InputFocus::Dialogue;
            return;
        }
        *focus = InputFocus::Dialogue;
        match then {
            Then::Say(msg, after) => {
                say(&disc, &mut dialogue, actor, &identity, msg);
                flow.step = Step::Said(after);
            }
            Then::Trick => do_trick(
                &mut dialogue,
                &mut flow,
                &mut inventory,
                &mut look,
                &mut rand,
            ),
            Then::Presented => {
                say(&disc, &mut dialogue, actor, &identity, 0x09BA);
                flow.step = Step::Said(After::Close);
            }
        }
        return;
    }

    // A line was read (or the player answered the question).
    if dialogue.speaker == Some(actor) {
        if let Some(choice) = dialogue.external_result.take() {
            *focus = InputFocus::Dialogue;
            match std::mem::take(&mut flow.step) {
                Step::Greeting => {
                    let next = say(&disc, &mut dialogue, actor, &identity, 0x09B5);
                    flow.step = Step::Asked(next);
                }
                Step::Asked(next) => {
                    if choice == 0 {
                        *pockets = PocketsUi {
                            open: true,
                            give_any: true,
                            ..default()
                        };
                        *dialogue = DialogueState::default();
                        *focus = InputFocus::Pockets;
                        crate::sound::se(crate::sound::se_id::MENU_PAUSE);
                        flow.step = Step::Picking;
                    } else if let Some(Some(msg)) = next.get(choice).copied() {
                        say(&disc, &mut dialogue, actor, &identity, msg as u16);
                        flow.step = Step::Said(After::Trick);
                    } else {
                        *dialogue = DialogueState::default();
                        do_trick(
                            &mut dialogue,
                            &mut flow,
                            &mut inventory,
                            &mut look,
                            &mut rand,
                        );
                    }
                }
                Step::Said(After::Trick) => {
                    *dialogue = DialogueState::default();
                    do_trick(
                        &mut dialogue,
                        &mut flow,
                        &mut inventory,
                        &mut look,
                        &mut rand,
                    );
                }
                Step::Said(After::ReturnTool(item)) => {
                    *dialogue = DialogueState::default();
                    handover.start(Request {
                        item: Item::Goods(item),
                        giver: Party::Npc(actor),
                        taker: Party::Player,
                    });
                    flow.step = Step::Hand(Then::Trick);
                }
                Step::Said(After::Present(present)) => {
                    // `aEPK_give_a_present_demo_start_wait_talk_proc`: the piece into a free pocket.
                    *dialogue = DialogueState::default();
                    inventory.add(present);
                    info!(
                        "Jack gives {present:#06x} ({})",
                        crate::items::name(present)
                    );
                    handover.start(Request {
                        item: Item::Goods(present),
                        giver: Party::Npc(actor),
                        taker: Party::Player,
                    });
                    flow.step = Step::Hand(Then::Presented);
                }
                Step::Said(After::Close) | Step::Idle | Step::Picking | Step::Hand(_) => {
                    end(&mut dialogue, &mut focus, &mut flow, &mut actor_at, &mut me);
                }
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
    if delta.length() > 1.6 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    let block = acre_of(at.translation);
    if block == me.block {
        say(
            &disc,
            &mut dialogue,
            actor,
            &identity,
            0x09BC + rand.0.random(6) as u16,
        );
        flow.step = Step::Said(After::Close);
    } else {
        say(&disc, &mut dialogue, actor, &identity, 0x098A);
        flow.step = Step::Greeting;
    }
    me.block = block;
}

pub struct JackPlugin;

impl Plugin for JackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .add_systems(
                Update,
                visit.after(crate::event_today::refresh).in_set(GameplaySet),
            )
            .add_systems(Update, face.in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
