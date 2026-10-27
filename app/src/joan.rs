//! Joan (SP_NPC_KABUPEDDLER), the turnip peddler who visits on Sunday mornings.
//!
//! Schedule (`m_event_schedule.c_inc`): every Sunday from 06:00 to 11:59. She
//! stands in a free spot of a block picked from the day (`ac_event_manager.c`
//! `turnipbuyer_in` / `search_free_unit`). Talking (`ac_ev_kabuPeddler.c`): the
//! first time this Sunday 0x06FD (with her turnip lesson), afterwards 0x0718;
//! the sale menu offers 10, 50 or 100 turnips at Sunday's price. The answers
//! link through the disc script; the sale itself happens when the price line
//! ends (`aEKPD_sell_check_after`): a free pocket first, then the Bells, and
//! 0x0711 asks again.
use bevy::prelude::*;

use crate::character::{NpcClip, NpcGait};
use crate::dialogue::DialogueState;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::messages::MsgContext;
use crate::player::Player;
use crate::shop::ShopLocation;
use crate::shop_nook::handover::{HandOver, Item, Party, Request};
use crate::states::GameplaySet;
use crate::town::Town;
use rustimal_logic::town_gen::{ACRE_DEPTH, ACRE_WIDTH};

/// ALL_NPC_NUM + (SP_NPC_KABUPEDDLER - SP_NPC_START); `l_sp_actor_name` sound 8.
const JOAN_NPC: u32 = 238 + 7;
const JOAN_SOUND: u8 = 8;
/// Her walking pace in world units a second (a villager's `WALK_SPEED`).
const WALK_SPEED: f32 = 1.6;
/// First and last hour she is out (`mEv_EVENT_KABU_PEDDLER`).
const FIRST_HOUR: u8 = 6;
const LAST_HOUR: u8 = 11;

#[derive(Component)]
struct Joan {
    /// The game day she came out for.
    day: i64,
    /// Where she is walking to after a talk.
    goal: Option<Vec3>,
    /// Talks so far, to vary where she goes next.
    talks: u32,
}

/// Whether Joan is out now: Sunday morning.
fn visiting(now: &crate::clock::Now) -> bool {
    now.weekday == 0 && (FIRST_HOUR..=LAST_HOUR).contains(&now.hour)
}

/// `search_free_unit`, over this town's acres: the day picks an acre, and her
/// spot is the clear, empty unit nearest its middle.
fn spot(town: &Town, items: &crate::field_items::FieldItems, day: i64) -> Option<Vec3> {
    let n = (ACRE_WIDTH * ACRE_DEPTH) as i64;
    for cur in (1..=n).rev() {
        let pick = ((day * 31 + cur * 3 + 9 * i64::from(crate::clock::now().month)).rem_euclid(n))
            as usize;
        let (ax, az) = (pick % ACRE_WIDTH, pick / ACRE_WIDTH);
        let mut best: Option<(f32, Vec3)> = None;
        for cz in 0..16 {
            for cx in 0..16 {
                let at = Town::cell_to_world(ax, az, cx, cz);
                let free = items.at(at).is_some_and(|(_, _, item)| item == 0);
                if !free || !town.is_clear_ground(at) {
                    continue;
                }
                let middle = Town::cell_to_world(ax, az, 8, 8);
                let distance = (at - middle).length();
                if best.is_none_or(|(d, _)| distance < d) {
                    best = Some((distance, at));
                }
            }
        }
        if let Some((_, at)) = best {
            return Some(at.with_y(town.ground_y(at)));
        }
    }
    None
}

/// Brings Joan out for her morning and sends her home after.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    items: Option<Res<crate::field_items::FieldItems>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Joan)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let out = visiting(&now)
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    for (entity, joan) in &existing {
        if !out || joan.day != now.day_number {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    let Some(items) = items else { return };
    let Some(at) = spot(&town, &items, now.day_number) else {
        return;
    };
    commands.spawn((
        Name::new("Joan"),
        Joan {
            day: now.day_number,
            goal: None,
            talks: 0,
        },
        NpcClip::default(),
        NpcGait::default(),
        crate::character::NpcModel(JOAN_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y),
    ));
    info!("Joan is out at {at:?}");
}

fn context(identity: &crate::save::PlayerIdentity) -> MsgContext {
    let now = crate::clock::now();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Joan".into(),
        free: vec![String::new(); 20],
        items: vec![String::new(); 5],
        item_articles: vec![0; 5],
        month: now.month,
        day: now.day,
        hour: now.hour,
        minute: now.minute,
        weekday: now.weekday,
        ..default()
    }
}

/// Shows disc message `msg`. Returns the message that asks a question, and
/// the question.
fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &MsgContext,
    msg: usize,
) -> Option<(usize, crate::messages::MsgChoice)> {
    let text = disc.text.as_ref()?;
    let (pages, asked) = text.talk_asking(msg, ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "Joan",
        (JOAN_NPC, JOAN_SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Joan message {msg:#06x}");
    asked
}

/// `ITM_KABU_10`, `ITM_KABU_50`, `ITM_KABU_100`.
const BUNDLES: [(u16, u32); 3] = [(0x2F00, 10), (0x2F01, 50), (0x2F02, 100)];
/// The sale question that follows a sale (`0x0711`).
const MORE: usize = 0x0711;
const MORE_ASKS: usize = 0x0719;

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// A question is up; its answers lead to these messages.
    Ask {
        asked: usize,
        next: Vec<Option<usize>>,
    },
    /// A bundle's price line is up; the sale happens when it ends.
    Price { bundle: usize, again: bool },
    /// The last line before she is done.
    Finish,
    /// The money and then the turnips change hands (`MONEY_GET_DEMO`,
    /// `KABU_TRANS_DEMO`); the question follows.
    Demo { stage: u8 },
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
    context: MsgContext,
    /// The talk is over: she moves on (`turnipbuyer_behind`).
    leave: bool,
}

/// She turns to face the player while they talk.
fn face(
    time: Res<Time>,
    flow: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<Joan>)>,
    mut joan: Query<&mut Transform, With<Joan>>,
) {
    let (Ok(at), Ok(mut transform)) = (player.single(), joan.single_mut()) else {
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

/// Sets her price lines: one, ten, fifty and a hundred turnips at today's price
/// (`aEKPD_set_price_str`).
fn price_strings(ctx: &mut MsgContext, price: u32) {
    for (slot, count) in [1, 10, 50, 100].into_iter().enumerate() {
        ctx.free[slot] = (price * count).to_string();
    }
}

/// Follows a question the script left open, or finishes.
fn ask(step: &mut Step, asked: Option<(usize, crate::messages::MsgChoice)>) {
    *step = match asked {
        Some((asked, choice)) => Step::Ask {
            asked,
            next: choice.next,
        },
        None => Step::Finish,
    };
}

#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), With<Player>>,
    joan: Query<(Entity, &Transform), With<Joan>>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    mut shop: ResMut<crate::shop::ShopData>,
    identity: Res<crate::save::PlayerIdentity>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, actor_at)) = joan.single() else {
        return;
    };
    let price = u32::from(shop.turnips.price());
    // Only her own answers; other talkers in town take theirs.
    let answer = if dialogue.speaker == Some(actor) {
        dialogue.external_result.take()
    } else {
        None
    };
    if let Some(choice) = answer {
        *focus = InputFocus::Dialogue;
        match std::mem::take(&mut flow.step) {
            Step::Ask { asked, next } => {
                // Four answers are the sale menu: three bundles and "no".
                let buying = next.len() == 4;
                let target = next.get(choice).copied().flatten();
                let Some(msg) = target else {
                    *dialogue = DialogueState::default();
                    *focus = InputFocus::World;
                    flow.leave = true;
                    return;
                };
                let asking = say(&disc, &mut dialogue, actor, &flow.context, msg);
                if buying && choice < 3 {
                    flow.step = Step::Price {
                        bundle: choice,
                        // 0x0719 is the question's own line inside 0x0711.
                        again: matches!(asked, MORE | MORE_ASKS),
                    };
                } else {
                    ask(&mut flow.step, asking);
                }
            }
            Step::Price { bundle, again } => {
                let (item, count) = BUNDLES[bundle];
                let cost = price * count;
                // aEKPD_sell_check_after: room in the pockets, then the Bells.
                let slot = inventory
                    .pockets
                    .pockets
                    .iter()
                    .position(|&pocket| pocket == 0);
                let msg = match slot {
                    Some(slot) if inventory.bells >= cost => {
                        inventory.bells -= cost;
                        inventory.pockets.pockets[slot] = item;
                        inventory.pockets.set_item_cond(slot, 0);
                        crate::sound::se(crate::sound::se_id::REGISTER);
                        info!("Joan sold {count} turnips for {cost}");
                        // The money goes across first, then the bundle.
                        *dialogue = DialogueState::default();
                        handover.start(Request {
                            item: Item::Money,
                            giver: Party::Player,
                            taker: Party::Npc(actor),
                        });
                        flow.step = Step::Demo { stage: 0 };
                        return;
                    }
                    Some(_) if again => 0x0717,
                    Some(_) => 0x0710,
                    None if again => 0x0716,
                    None => 0x070F,
                };
                let asking = say(&disc, &mut dialogue, actor, &flow.context, msg);
                ask(&mut flow.step, asking);
            }
            Step::Idle | Step::Finish | Step::Demo { .. } => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
                flow.leave = true;
            }
        }
        return;
    }
    if let Step::Demo { stage } = flow.step {
        if handover.busy() {
            return;
        }
        *focus = InputFocus::Dialogue;
        if stage == 0 {
            handover.start(Request {
                item: Item::Turnips,
                giver: Party::Npc(actor),
                taker: Party::Player,
            });
            flow.step = Step::Demo { stage: 1 };
        } else {
            let asking = say(&disc, &mut dialogue, actor, &flow.context, MORE);
            ask(&mut flow.step, asking);
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
    flow.context = context(&identity);
    price_strings(&mut flow.context, price);
    // aEKPD_talk_init: she remembers who she has spoken to this Sunday.
    let today = crate::clock::now().day_number;
    let met = shop.turnips.joan_spoken == Some(today);
    shop.turnips.joan_spoken = Some(today);
    let asking = say(
        &disc,
        &mut dialogue,
        actor,
        &flow.context,
        if met { 0x0718 } else { 0x06FD },
    );
    ask(&mut flow.step, asking);
}

/// A clear unit a few steps off with a clear line to it, picked from the
/// talk count (`walk_actor_at_wade` moves her to a new free unit).
fn wander_goal(
    town: &Town,
    items: &crate::field_items::FieldItems,
    from: Vec3,
    seed: u32,
) -> Option<Vec3> {
    (0..16u32).find_map(|k| {
        let angle = (seed + k * 5) as f32 * 0.9;
        let dist = 8.0 - (k / 4) as f32 * 1.5;
        let at = from + Vec3::new(angle.cos(), 0.0, angle.sin()) * dist;
        let free = items.at(at).is_some_and(|(_, _, item)| item == 0);
        let line_clear = (1..=8).all(|i| town.is_clear_ground(from.lerp(at, i as f32 / 8.0)));
        (free && line_clear).then(|| at.with_y(town.ground_y(at)))
    })
}

/// Walks her to a new spot after a talk, at a villager's pace.
fn walk(
    time: Res<Time>,
    town: Res<Town>,
    items: Option<Res<crate::field_items::FieldItems>>,
    dialogue: Res<DialogueState>,
    mut flow: ResMut<Talk>,
    mut joan: Query<(Entity, &mut Transform, &mut Joan, &mut NpcGait)>,
) {
    let (Some(items), Ok((entity, mut transform, mut joan, mut gait))) = (items, joan.single_mut())
    else {
        return;
    };
    if dialogue.active && dialogue.speaker == Some(entity) {
        gait.0 = 0;
        return;
    }
    if flow.leave && matches!(flow.step, Step::Idle) {
        flow.leave = false;
        joan.talks += 1;
        let seed = joan
            .talks
            .wrapping_add(crate::clock::now().day_number as u32);
        joan.goal = wander_goal(&town, &items, transform.translation, seed);
    }
    let Some(goal) = joan.goal else {
        gait.0 = 0;
        return;
    };
    let to = (goal - transform.translation).with_y(0.0);
    let step = WALK_SPEED * time.delta_secs();
    if to.length() <= step {
        transform.translation = goal + Vec3::Y;
        joan.goal = None;
        gait.0 = 0;
        return;
    }
    let dir = to.normalize();
    let next = transform.translation + dir * step;
    if !town.is_clear_ground(next) {
        joan.goal = None;
        gait.0 = 0;
        return;
    }
    transform.translation = next.with_y(town.ground_y(next) + 1.0);
    let want = Quat::from_rotation_y(dir.x.atan2(dir.z));
    transform.rotation = transform
        .rotation
        .slerp(want, (time.delta_secs() * 8.0).min(1.0));
    gait.0 = 1;
}

pub struct JoanPlugin;

impl Plugin for JoanPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .add_systems(Update, (visit, face, walk).in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
