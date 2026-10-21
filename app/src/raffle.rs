//! The monthly raffle (fukubiki, the last day of the month): Nook in his
//! raffle coat (SP_NPC_SHOP_MASTERSP, ac_npc_shop_mastersp_talk.c_inc)
//! takes five of this month's lottery tickets from the pockets for a turn
//! of the drum.
//!
//! Talk: 0x10D1 asks; five tickets -> 0x10D4 and the draw (else 0x10D3).
//! The tickets go first (a loss still costs them); RANDOM(100) below 5 wins
//! the first prize, below 15 the second, below 35 the third, else nothing,
//! and a prize already won is a loss. A win: 0x10D9 / 0x10DA / 0x10DB, the
//! prize handed over wrapped (0x10E3, mPr_ITEM_COND_PRESENT), then 0x10DD
//! (another go?) or 0x10DC when out of tickets or prizes. A loss: 0x10D5,
//! then 0x10D7 (another go?) or 0x10D6.
//!
//! The prizes (Shop_c.lottery_items, mSP_MakeLotteryList) are three from
//! the lottery list, one never yet held first when there is one, made anew
//! each month. Not drawn yet: Nook walking to the drum and the prize
//! display, the drum and its ball (the drum's sound plays).
use bevy::prelude::*;
use rustimal_logic::special_delivery as sd;

use crate::dialogue::DialogueState;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::player::Player;
use crate::shop::ShopLocation;
use crate::shop_nook::Shopkeeper;
use crate::states::GameplaySet;

/// The drum turning before the ball comes out (garagara frame 46).
const SPIN_SECONDS: f32 = 3.0;
/// NA_SE_51: the raffle drum.
const DRUM_SE: u16 = 0x51;

/// Whether today is the raffle (mSP_CheckFukubikiDay).
pub fn raffle_day() -> bool {
    let now = crate::clock::now();
    rustimal_logic::shop::tanuki_shop_status(now.year, now.month, now.day, false)
        == rustimal_logic::shop::TanukiShopStatus::Fukubiki
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// A question is up (0x10D1, 0x10D7 / 0x10DD).
    Ask,
    /// The drum turning; then the result line.
    Spin(f32),
    Finish,
}

#[derive(Resource, Default)]
struct Raffle {
    step: Step,
    /// 0-2 a prize tier, 3 no win.
    result: usize,
}

/// mSP_MakeLotteryList, once a month.
fn lineup(mut catalog: ResMut<crate::catalog::Catalog>, mut seed: Local<u32>) {
    let month = crate::clock::now().month;
    if catalog.lottery_month == month
        || crate::goods::list(crate::goods::Kind::Furniture, 5).is_empty()
    {
        return;
    }
    if *seed == 0 {
        *seed = 0x2F6B_7A11 ^ u32::from(month);
    }
    let mut rand = |n: usize| {
        *seed ^= *seed << 13;
        *seed ^= *seed >> 17;
        *seed ^= *seed << 5;
        *seed as usize % n.max(1)
    };
    let list = crate::goods::list(crate::goods::Kind::Furniture, 5);
    let unheld: Vec<u16> = list
        .iter()
        .copied()
        .filter(|&i| !catalog.collection.has(i))
        .collect();
    let first = (!unheld.is_empty()).then(|| unheld[rand(unheld.len())]);
    let mut rest = Vec::new();
    let mut guard = 0;
    while rest.len() < 3 && guard < 200 {
        guard += 1;
        let item = list[rand(list.len())];
        if Some(item) != first && !rest.contains(&item) {
            rest.push(item);
        }
    }
    catalog.lottery = sd::make_lottery_lineup(first, &rest);
    catalog.lottery_month = month;
    info!("raffle prizes: {:x?}", catalog.lottery);
}

/// check_lottery_ticket: five of this month's tickets in normal condition.
fn has_tickets(inventory: &PlayerInventory, month: u8) -> bool {
    let mut count = 0;
    for (slot, &item) in inventory.pockets.pockets.iter().enumerate() {
        if inventory.pockets.item_cond(slot) == 0
            && sd::is_ticket_item(item)
            && sd::ticket_month(item) == month
        {
            count += sd::ticket_count(item);
        }
    }
    count >= sd::LOTTERY_REQ_TICKETS
}

/// uketori_lottery_ticket.
fn take_tickets(inventory: &mut PlayerInventory, month: u8) {
    let mut req = i16::from(sd::LOTTERY_REQ_TICKETS);
    for slot in 0..inventory.pockets.pockets.len() {
        if req <= 0 {
            break;
        }
        let item = inventory.pockets.pockets[slot];
        if inventory.pockets.item_cond(slot) != 0
            || !sd::is_ticket_item(item)
            || sd::ticket_month(item) != month
        {
            continue;
        }
        let count = i16::from(sd::ticket_count(item));
        inventory.pockets.pockets[slot] = if count <= req {
            0
        } else {
            sd::ticket_get_item(month, (count - req) as u8)
        };
        req -= count;
    }
}

/// Several lines in one window; true when the last asks a question.
fn say_all(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    msgs: &[usize],
    ctx: &crate::messages::MsgContext,
) -> bool {
    let Some(text) = disc.text.as_ref() else {
        return false;
    };
    let mut pages = Vec::new();
    let mut labels = None;
    for &msg in msgs {
        let (more, choice) = text.talk(msg, ctx, &mut |_| 0);
        pages.extend(more);
        labels = choice.map(|c| c.labels);
        info!("raffle message {msg:#06x}");
    }
    let asks = labels.is_some();
    dialogue.begin_external(actor, pages, labels);
    asks
}

fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    msg: usize,
    ctx: &crate::messages::MsgContext,
) -> bool {
    let Some(text) = disc.text.as_ref() else {
        return false;
    };
    let (pages, choice) = text.talk(msg, ctx, &mut |_| 0);
    let asks = choice.is_some();
    dialogue.begin_external(actor, pages, choice.map(|c| c.labels));
    info!("raffle message {msg:#06x}");
    asks
}

#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    time: Res<Time>,
    location: Res<State<ShopLocation>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), With<Player>>,
    nook: Query<(Entity, &Transform), With<Shopkeeper>>,
    mut raffle: ResMut<Raffle>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut catalog: ResMut<crate::catalog::Catalog>,
    identity: Res<crate::save::PlayerIdentity>,
    mut seed: Local<u32>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *location != ShopLocation::Inside || !raffle_day() {
        return;
    }
    let Ok((actor, nook_at)) = nook.single() else {
        return;
    };
    let month = crate::clock::now().month;
    let mut ctx = crate::messages::MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Tom Nook".into(),
        free: vec![String::new(); 20],
        items: vec![String::new(); 5],
        item_articles: vec![0; 5],
        ..default()
    };
    let draw = |raffle: &mut Raffle,
                inventory: &mut PlayerInventory,
                catalog: &crate::catalog::Catalog,
                seed: &mut u32| {
        // fukubiki_before_process.
        take_tickets(inventory, month);
        if *seed == 0 {
            *seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0x5EED, |d| d.subsec_nanos() | 1);
        }
        *seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        let roll = sd::lottery_roll(((*seed >> 16) % 100) as u8) as usize;
        raffle.result = if roll < 3 && !sd::prize_available(catalog.lottery[roll]) {
            3
        } else {
            roll
        };
        raffle.step = Step::Spin(0.0);
        crate::sound::se(DRUM_SE);
        info!("raffle draw: {}", raffle.result);
    };

    if let Step::Spin(t) = raffle.step {
        {
            // 0x10D4 stays up while the drum turns; the result follows on in
            // the same window (the original continues each line into the
            // next as Nook fetches the prize).
            let t = t + time.delta_secs();
            if t < SPIN_SECONDS {
                raffle.step = Step::Spin(t);
                return;
            }
            *focus = InputFocus::Dialogue;
            let mut msgs = Vec::new();
            if raffle.result < 3 {
                // to_atari / aSHM_give_player: wrapped, the slot marked sold.
                let prize = catalog.lottery[raffle.result];
                ctx.free[0] = crate::items::name(prize);
                if let Some(slot) = inventory.pockets.pockets.iter().position(|&i| i == 0) {
                    inventory.pockets.pockets[slot] = prize;
                    inventory.pockets.set_item_cond(slot, 1);
                }
                catalog.lottery[raffle.result] = sd::RSV_SHOP_SOLD_FTR;
                info!("raffle prize {prize:#06x}");
                msgs.push(0x10D9 + raffle.result);
                msgs.push(0x10E3);
                let prizes_left = catalog.lottery.iter().any(|&p| sd::prize_available(p));
                msgs.push(if prizes_left && has_tickets(&inventory, month) {
                    0x10DD
                } else {
                    0x10DC
                });
            } else {
                msgs.push(0x10D5);
                msgs.push(if has_tickets(&inventory, month) {
                    0x10D7
                } else {
                    0x10D6
                });
            }
            raffle.step = if say_all(&disc, &mut dialogue, actor, &msgs, &ctx) {
                Step::Ask
            } else {
                Step::Finish
            };
            return;
        }
    }

    if let Some(choice) = dialogue.external_result.take() {
        *focus = InputFocus::Dialogue;
        match (std::mem::take(&mut raffle.step), choice) {
            // aSHM_talk_try / aSHM_talk_retry.
            (Step::Ask, 0) if has_tickets(&inventory, month) => {
                say(&disc, &mut dialogue, actor, 0x10D4, &ctx);
                draw(&mut raffle, &mut inventory, &catalog, &mut seed);
            }
            (Step::Ask, 0) => {
                say(&disc, &mut dialogue, actor, 0x10D3, &ctx);
                raffle.step = Step::Finish;
            }
            _ => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
            }
        }
        return;
    }

    if !matches!(raffle.step, Step::Idle | Step::Finish)
        || !focus.is_world()
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    let delta = (nook_at.translation - at.translation).with_y(0.0);
    if delta.length() > 1.8 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    raffle.step = if say(&disc, &mut dialogue, actor, 0x10D1, &ctx) {
        Step::Ask
    } else {
        Step::Finish
    };
}

pub struct RafflePlugin;

impl Plugin for RafflePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Raffle>()
            .add_systems(Update, lineup.in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
