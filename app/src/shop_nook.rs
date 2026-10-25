//! Ordinary Cranny transactions from `ac_npc_shop_common`, and the four
//! `ac_npc_shop_master_move` routing zones. Disc scripts provide all dialogue.
use bevy::prelude::*;

use crate::dialogue::DialogueState;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::messages::MsgContext;
use crate::player::Player;
use crate::pockets::PocketsUi;
use crate::shop::{ShopData, ShopLocation, SHOP_ORIGIN};
use crate::shop_display::ShopDisplay;
use crate::states::GameplaySet;

pub(crate) mod preview;

#[derive(Component, Default)]
pub(crate) struct Shopkeeper {
    pub walking: bool,
    /// Run clip while he crosses to the drum or a prize (`aNPC_ACT_RUN`).
    pub running: bool,
    next_zone: Option<usize>,
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    Request,
    SellStart,
    Pockets,
    ShowQuestion {
        index: usize,
        item: u16,
        price: u32,
    },
    ChangingCloth {
        index: usize,
        item: u16,
        price: u32,
        decision: Option<usize>,
    },
    Purchase {
        index: usize,
        item: u16,
        price: u32,
    },
    Offer {
        slot: usize,
        item: u16,
        condition: u32,
        price: u32,
    },
    Finish,
    /// 0x10A6 is on screen; the catalog opens when the player continues.
    CatalogIntro,
    /// The catalog screen is up (`mSM_OVL_CATALOG`).
    Browsing,
    /// 0x10A7: yes pays and files the order.
    Order {
        item: u16,
        price: u32,
    },
    /// A house question (offer, basement or roof colour): the disc's own
    /// branches, the answers and which colour page is showing.
    House {
        next: Vec<Option<usize>>,
        labels: Vec<String>,
        page: u8,
    },
}

#[derive(Resource, Default)]
pub(crate) struct Conversation {
    step: Step,
    context: MsgContext,
    /// The request menu included the catalog (choice 1).
    catalog_offered: bool,
}

/// Seconds inside before Nook raises the house (the door demo's length).
const ARRIVAL: f32 = 0.6;

// aNSC_get_msg_no, ordinary Nook (not the department-store twins).
fn message(n: usize) -> usize {
    n + match n {
        0..100 => 0x107B,
        100..200 => 0x2BAF,
        200..300 => 0x2DE6,
        _ => 0x3CD2,
    }
}

fn say(disc: &GameDisc, dialogue: &mut DialogueState, actor: Entity, ctx: &MsgContext, n: usize) {
    let Some(text) = disc.text.as_ref() else {
        return;
    };
    let (pages, choice) = text.talk(message(n), ctx, &mut |_| 0);
    dialogue.begin_external(actor, pages, choice.map(|c| c.labels));
    info!("Nook message {:#06x}", message(n));
}

/// Show the disc message `msg` (an absolute number); returns its question.
fn say_abs(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &MsgContext,
    msg: usize,
) -> Option<crate::messages::MsgChoice> {
    let text = disc.text.as_ref()?;
    let (pages, choice) = text.talk(msg, ctx, &mut |_| 0);
    dialogue.begin_external(actor, pages, choice.as_ref().map(|c| c.labels.clone()));
    info!("Nook message {msg:#06x}");
    choice
}

/// Continue the house talk at `msg`.
fn house_say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    flow: &mut Conversation,
    msg: usize,
    page: u8,
) {
    flow.step = match say_abs(disc, dialogue, actor, &flow.context, msg) {
        Some(choice) => Step::House {
            next: choice.next,
            labels: choice.labels,
            page,
        },
        None => Step::Finish,
    };
}

fn context(identity: &crate::save::PlayerIdentity) -> MsgContext {
    let now = crate::clock::now();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Tom Nook".into(),
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

/// mIN_get_item_article: read the disc tables rather than guessing from
/// the spelling ("an hourglass", "some paper", furniture exceptions).
pub(crate) fn item_article(disc: &GameDisc, item: u16) -> u8 {
    const TABLES: [&str; 16] = [
        "itemArt_Paper",
        "itemArt_Money",
        "itemArt_Tool",
        "itemArt_Fish",
        "itemArt_Cloth",
        "itemArt_Etc",
        "itemArt_Carpet",
        "itemArt_Wall",
        "itemArt_Fruit",
        "itemArt_Plant",
        "itemArt_MiniDisk",
        "itemArt_Diary",
        "itemArt_Ticket",
        "itemArt_Insect",
        "itemArt_Hukubukuro",
        "itemArt_Kabu",
    ];
    let (symbol, index) = if crate::items::is_furniture(item) {
        (
            "ftrArt",
            crate::furniture::furniture_index(item).map_or(usize::MAX, |i| i as usize),
        )
    } else {
        (
            TABLES[usize::from((item >> 8) & 15)],
            usize::from(item & 255),
        )
    };
    disc.catalog
        .get(symbol)
        .and_then(|a| disc.rom.slice(a.source, a.offset, a.size))
        .and_then(|b| b.get(index))
        .copied()
        .unwrap_or(0)
}

/// A forward unit, as in aNSC_message_ctrl_talk_request_normal_day. Unit
/// centres, rather than rendered mesh bounds, also work on table merchandise.
fn aimed_display(pos: Vec3, facing: Vec3, display: &ShopDisplay) -> bool {
    let delta = (display.position - pos).with_y(0.0);
    let forward = delta.dot(facing);
    forward > 0.0 && forward <= 1.65 && (delta - facing * forward).length() <= 0.48
}

// This controller needs the actor, inventory, dialogue and pockets at the
// same confirmation point; separating their writes would permit partial sales.
#[allow(clippy::too_many_arguments)]
pub(crate) fn interact(
    disc: Option<Res<GameDisc>>,
    location: Res<State<ShopLocation>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    mut player: Query<(&Transform, &Facing, &mut crate::locomotion::Locomotion), With<Player>>,
    nook: Query<(Entity, &Transform), With<Shopkeeper>>,
    displays: Query<&ShopDisplay>,
    mut flow: ResMut<Conversation>,
    mut dialogue: ResMut<DialogueState>,
    mut pockets: ResMut<PocketsUi>,
    mut inventory: ResMut<PlayerInventory>,
    mut shop: ResMut<ShopData>,
    mut catalog_ui: ResMut<crate::catalog_screen::CatalogUi>,
    identity: Res<crate::save::PlayerIdentity>,
    (mut home, mut inside_for, time, mut catalog, mut preview): (
        Option<ResMut<crate::home::PlayerHouse>>,
        Local<Option<f32>>,
        Res<Time>,
        ResMut<crate::catalog::Catalog>,
        ResMut<preview::Preview>,
    ),
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *location != ShopLocation::Inside {
        preview.clear();
        if !matches!(flow.step, Step::Idle) {
            flow.step = Step::Idle;
            *dialogue = DialogueState::default();
            *focus = InputFocus::World;
        }
        *inside_for = None;
        if catalog_ui.open {
            catalog_ui.open = false;
            catalog_ui.result = None;
        }
        return;
    }
    let Ok((actor, nook_transform)) = nook.single() else {
        preview.clear();
        return;
    };
    if let Step::ChangingCloth {
        index,
        item,
        price,
        decision,
    } = flow.step
    {
        if preview.changing() {
            return;
        }
        flow.step = Step::Purchase { index, item, price };
        if let Some(choice) = decision {
            preview.clear();
            dialogue.external_result = Some(choice);
        } else if preview.active.is_some() {
            say(&disc, &mut dialogue, actor, &flow.context, 19);
        } else {
            say(&disc, &mut dialogue, actor, &flow.context, 31);
            flow.step = Step::Finish;
        }
    }
    if matches!(flow.step, Step::Browsing) {
        if catalog_ui.open {
            return;
        }
        let picked = catalog_ui.result.take();
        *focus = InputFocus::Dialogue;
        match picked {
            Some(Some(item)) => {
                let price = crate::catalog_lists::order_price(item);
                flow.context.items[1] = crate::items::name(item);
                flow.context.item_articles[1] = item_article(&disc, item);
                flow.context.free[3] = price.to_string();
                if price == 0 {
                    say(&disc, &mut dialogue, actor, &flow.context, 45);
                    flow.step = Step::Finish;
                } else {
                    say(&disc, &mut dialogue, actor, &flow.context, 44);
                    flow.step = Step::Order { item, price };
                }
            }
            _ => {
                say(&disc, &mut dialogue, actor, &flow.context, 46);
                flow.step = Step::Finish;
            }
        }
        return;
    }
    // On the raffle day Nook runs the raffle instead (raffle.rs).
    if crate::raffle::raffle_day() {
        return;
    }
    // aNSC_start_wait: Nook speaks about the house once the player is in
    // (after the door, as the original's entrance demo), and they stand.
    let since = inside_for.get_or_insert(0.0);
    let was = *since;
    *since += time.delta_secs();
    if was < ARRIVAL && *since >= ARRIVAL && matches!(flow.step, Step::Idle) {
        if let Some(home) = home.as_mut() {
            let today = crate::home::today();
            if let Some(n) = home.0.nook_greeting(&mut inventory.loan, today) {
                info!(
                    "Nook house talk {n:#04x}: {:?}, loan {}",
                    home.0, inventory.loan
                );
                if let Ok((_, _, mut loco)) = player.single_mut() {
                    *loco = crate::locomotion::Locomotion::default();
                }
                flow.context = context(&identity);
                *focus = InputFocus::Dialogue;
                house_say(&disc, &mut dialogue, actor, &mut flow, message(n), 0);
                return;
            }
        }
    }
    if matches!(flow.step, Step::Pockets) {
        if pockets.open {
            return;
        }
        pockets.selling = false;
        *focus = InputFocus::Dialogue;
        if let Some(slot) = pockets.sell_slot.take() {
            let item = inventory.pockets.pockets[slot];
            let condition = inventory.pockets.item_cond(slot);
            let price = crate::items::sell_price(item);
            flow.context.items[0] = crate::items::name(item);
            flow.context.item_articles[0] = item_article(&disc, item);
            flow.context.free[2] = price.unwrap_or(0).to_string();
            if condition != 0 || price.is_none() {
                // Recheck the handoff if the slot changed after selection.
                say(&disc, &mut dialogue, actor, &flow.context, 29);
                flow.step = Step::Finish;
            } else {
                say(&disc, &mut dialogue, actor, &flow.context, 27);
                flow.step = Step::Offer {
                    slot,
                    item,
                    condition,
                    price: price.unwrap_or(0),
                };
            }
        } else {
            say(&disc, &mut dialogue, actor, &flow.context, 29);
            flow.step = Step::Finish;
        }
        return;
    }
    if let Some(choice) = dialogue.external_result.take() {
        *focus = InputFocus::Dialogue;
        let step = std::mem::take(&mut flow.step);
        match step {
            Step::Request if choice == 0 => {
                say(&disc, &mut dialogue, actor, &flow.context, 26);
                flow.step = Step::SellStart;
            }
            Step::Request if choice == 1 && flow.catalog_offered => {
                if catalog.orders.free_slot().is_none() {
                    say(&disc, &mut dialogue, actor, &flow.context, 49);
                    flow.step = Step::Finish;
                } else {
                    say(&disc, &mut dialogue, actor, &flow.context, 43);
                    flow.step = Step::CatalogIntro;
                }
            }
            Step::CatalogIntro => {
                catalog_ui.open_from(|item| catalog.collection.has(item));
                *dialogue = DialogueState::default();
                *focus = InputFocus::Pockets;
                flow.step = Step::Browsing;
                crate::sound::se(crate::sound::se_id::MENU_PAUSE);
            }
            Step::Order { item, price } => {
                let response = if choice != 0 {
                    46
                } else if inventory.bells < price {
                    48
                } else {
                    match catalog.orders.place(item, shop.shop.shop_level as u8) {
                        Ok(slot) => {
                            inventory.bells -= price;
                            shop.shop.record_purchase(price);
                            crate::shop::maybe_upgrade(&mut shop.shop);
                            crate::sound::se(crate::sound::se_id::REGISTER);
                            info!("catalog order slot={slot} item={item:#06x} price={price}");
                            47
                        }
                        Err(_) => 49,
                    }
                };
                say(&disc, &mut dialogue, actor, &flow.context, response);
                flow.step = Step::Finish;
            }
            Step::SellStart => {
                *pockets = PocketsUi {
                    open: true,
                    selling: true,
                    ..default()
                };
                *dialogue = DialogueState::default();
                *focus = InputFocus::Pockets;
                flow.step = Step::Pockets;
                crate::sound::se(crate::sound::se_id::MENU_PAUSE);
            }
            Step::ShowQuestion { index, item, price } => {
                let valid = shop
                    .stock
                    .get(index)
                    .is_some_and(|s| s.item == item && !s.sold && s.price == price);
                if choice >= 2 || !valid {
                    preview.clear();
                    say(&disc, &mut dialogue, actor, &flow.context, 31);
                    flow.step = Step::Finish;
                } else if choice == 0 && preview.start(&disc, index, item) {
                    if preview.shirt() {
                        *dialogue = DialogueState::default();
                        flow.step = Step::ChangingCloth {
                            index,
                            item,
                            price,
                            decision: None,
                        };
                    } else {
                        say(&disc, &mut dialogue, actor, &flow.context, 41);
                        flow.step = Step::Purchase { index, item, price };
                    }
                } else {
                    say(
                        &disc,
                        &mut dialogue,
                        actor,
                        &flow.context,
                        if preview::Surface::of(item) == Some(preview::Surface::Shirt) {
                            20
                        } else {
                            42
                        },
                    );
                    flow.step = Step::Purchase { index, item, price };
                }
            }
            Step::Purchase { index, item, price } => {
                if preview.shirt() {
                    preview.restore_shirt();
                    *dialogue = DialogueState::default();
                    flow.step = Step::ChangingCloth {
                        index,
                        item,
                        price,
                        decision: Some(choice),
                    };
                    return;
                }
                preview.clear();
                let valid = shop
                    .stock
                    .get(index)
                    .is_some_and(|s| s.item == item && !s.sold && s.price == price);
                let response = if choice != 0 || !valid {
                    31
                } else if inventory.bells < price {
                    33
                } else if !inventory.add(item) {
                    39
                } else {
                    inventory.bells -= price;
                    shop.stock[index].sold = true;
                    shop.shop.record_purchase(price);
                    crate::shop::maybe_upgrade(&mut shop.shop);
                    crate::sound::se(crate::sound::se_id::REGISTER);
                    info!("Nook purchase stock={index} item={item:#06x} price={price}");
                    // aNSC_sell_answer0: a lottery ticket with furniture,
                    // clothing, wallpaper, carpets and umbrellas, onto this
                    // month's stack or a free slot (GIVE_TICKET), else by
                    // mail (MAIL_TICKET); otherwise the tool's own line.
                    let month = crate::clock::now().month;
                    if crate::catalog::comes_with_ticket(item) {
                        if crate::catalog::give_ticket(&mut inventory.pockets.pockets, month) {
                            51
                        } else {
                            catalog.ticket_by_mail(month);
                            52
                        }
                    } else {
                        match item {
                            0x2200 => 101,
                            0x2201 => 103,
                            0x2202 => 100,
                            0x2203 => 102,
                            0x251E => 104,
                            _ => 34,
                        }
                    }
                };
                say(&disc, &mut dialogue, actor, &flow.context, response);
                flow.step = Step::Finish;
            }
            Step::Offer {
                slot,
                item,
                condition,
                price,
            } => {
                let valid = inventory.pockets.pockets[slot] == item
                    && inventory.pockets.item_cond(slot) == condition;
                let response = if choice != 0 || !valid {
                    29
                } else if !sell_slot(&mut inventory, slot, price) {
                    200
                } else {
                    shop.shop.record_sale(price);
                    crate::shop::maybe_upgrade(&mut shop.shop);
                    crate::sound::se(crate::sound::se_id::REGISTER);
                    info!("Nook sale slot={slot} item={item:#06x} payout={price}");
                    28
                };
                say(&disc, &mut dialogue, actor, &flow.context, response);
                flow.step = Step::Finish;
            }
            Step::Request => {
                say(&disc, &mut dialogue, actor, &flow.context, 24);
                flow.step = Step::Finish;
            }
            Step::House { next, labels, page } => {
                use crate::home::nook_msg::{PALLET_SELECTED, REHOUSE_2_BASEMENT};
                let target = next.get(choice).copied().flatten();
                flow.context.determination = labels.get(choice).cloned().unwrap_or_default();
                // Roof colour pages: three colours and "Another color...".
                let colours = next.len() == 4
                    && next[..3]
                        .iter()
                        .all(|n| *n == Some(message(PALLET_SELECTED)));
                // REHOUSE_2_OFFER: expand, or add a basement.
                let basement = next.len() == 2 && next[1] == Some(message(REHOUSE_2_BASEMENT));
                let today = crate::home::today();
                if let Some(home) = home.as_mut() {
                    if colours && choice < 3 {
                        let pal = page * 3 + choice as u8;
                        let name = labels.get(choice).cloned().unwrap_or_default();
                        let ok = home.0.order_expansion(&mut inventory.loan, pal, today);
                        info!("house expansion ordered ({ok}): roof {pal} {name}");
                    } else if basement && choice == 1 {
                        let ok = home.0.order_basement(&mut inventory.loan, today);
                        info!("basement ordered ({ok})");
                    }
                }
                let page = if colours && choice == 3 {
                    (page + 1) % 4
                } else {
                    page
                };
                match target {
                    Some(msg) => house_say(&disc, &mut dialogue, actor, &mut flow, msg, page),
                    None => {
                        *dialogue = DialogueState::default();
                        *focus = InputFocus::World;
                    }
                }
            }
            _ => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
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
    let Ok((transform, facing, _)) = player.single() else {
        return;
    };
    let display = displays
        .iter()
        .filter(|d| aimed_display(transform.translation, facing.dir, d))
        .min_by(|a, b| {
            a.position
                .distance_squared(transform.translation)
                .total_cmp(&b.position.distance_squared(transform.translation))
        });
    flow.context = context(&identity);
    if let Some(display) = display {
        let Some(stock) = shop.stock.get(display.stock_index) else {
            return;
        };
        if stock.sold {
            actions.consume(Action::Interact);
            return;
        }
        // Paint changes the house rather than entering pockets: keep that
        // service deferred instead of selling a useless paint-can item.
        if (0x222D..=0x2238).contains(&stock.item) {
            return;
        }
        actions.consume(Action::Interact);
        *focus = InputFocus::Dialogue;
        flow.context.items[0] = stock.name.clone();
        flow.context.item_articles[0] = item_article(&disc, stock.item);
        flow.context.free[4] = stock.price.to_string();
        say(
            &disc,
            &mut dialogue,
            actor,
            &flow.context,
            if preview::Surface::of(stock.item) == Some(preview::Surface::Shirt) {
                18
            } else if preview::Surface::of(stock.item).is_some() {
                40
            } else if (0x2000..=0x20FF).contains(&stock.item) {
                222
            } else {
                30
            },
        );
        flow.step = if preview::Surface::of(stock.item).is_some() {
            Step::ShowQuestion {
                index: display.stock_index,
                item: stock.item,
                price: stock.price,
            }
        } else {
            Step::Purchase {
                index: display.stock_index,
                item: stock.item,
                price: stock.price,
            }
        };
    } else {
        let delta = (nook_transform.translation - transform.translation).with_y(0.0);
        if delta.length() > 1.8 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
            return;
        }
        actions.consume(Action::Interact);
        *focus = InputFocus::Dialogue;
        say(&disc, &mut dialogue, actor, &flow.context, 23);
        // Sell, the catalog, and cancel. The turnip and password lines stay
        // off until those services exist. Choice 1 is the catalog
        // (aNSC_MSG_SHOW_CATALOGUE).
        flow.catalog_offered = false;
        if let Some(options) = dialogue.choices_for_actor() {
            if options.len() >= 3 {
                let first = options[0].clone();
                let catalog_label = options[1].clone();
                let last = options.last().cloned().unwrap_or_default();
                dialogue.set_actor_choices(vec![first, catalog_label, last]);
                flow.catalog_offered = true;
            } else if let (Some(first), Some(last)) =
                (options.first().cloned(), options.last().cloned())
            {
                dialogue.set_actor_choices(vec![first, last]);
            }
        }
        flow.step = Step::Request;
    }
}

/// Original wallet overflow policy: exchange 30,000 bells into bags. Stage
/// the complete pocket update first so failure cannot consume the offered item.
fn sell_slot(inventory: &mut PlayerInventory, slot: usize, price: u32) -> bool {
    let Some(mut bells) = inventory.bells.checked_add(price) else {
        return false;
    };
    let mut pockets = inventory.pockets.clone();
    pockets.pockets[slot] = 0;
    pockets.set_item_cond(slot, 0);
    while bells >= 99999 {
        let Some(empty) = pockets.pockets.iter().position(|&i| i == 0) else {
            return false;
        };
        // ITM_MONEY_30000 (m_private / the translated buried-items table).
        pockets.pockets[empty] = 0x2102;
        pockets.set_item_cond(empty, 0);
        bells -= 30000;
    }
    inventory.pockets = pockets;
    inventory.bells = bells;
    true
}

fn zone(p: Vec3) -> usize {
    if p.x < 120.0 {
        if p.z < 180.0 {
            1
        } else {
            2
        }
    } else if p.z < 160.0 {
        0
    } else {
        3
    }
}

fn follow(
    disc: Option<Res<GameDisc>>,
    location: Res<State<ShopLocation>>,
    focus: Res<InputFocus>,
    time: Res<Time>,
    raffle: Res<crate::raffle::RaffleNook>,
    player: Query<&Transform, With<Player>>,
    mut nook: Query<(&mut Transform, &mut Shopkeeper), Without<Player>>,
    obstacles: Res<crate::shop::ShopObstacles>,
) {
    let Ok((mut transform, mut keeper)) = nook.single_mut() else {
        return;
    };
    keeper.walking = false;
    keeper.running = false;
    if disc.is_none() || *location != ShopLocation::Inside {
        return;
    }
    if let Some(goal) = raffle.at {
        // Raffle day: run to the drum, or to the prize he is picking up.
        // aNPC_ACT_RUN, 4 game units per 30 Hz frame.
        let corner = SHOP_ORIGIN - Vec3::new(4.0, 0.0, 4.0);
        let pos = (transform.translation - corner) * 40.0;
        let d = Vec3::new(goal.x - pos.x, 0.0, goal.y - pos.z);
        let at_drum = goal.distance(crate::raffle::NOOK_AT_DRUM) < 1.0;
        let desired = if d.length_squared() < 9.0 && at_drum {
            0.0
        } else {
            d.x.atan2(d.z)
        };
        let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let change = (desired - yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        let dt = time.delta_secs().min(0.1);
        let turn = 11.25_f32.to_radians() * 30.0 * dt;
        transform.rotation = Quat::from_rotation_y(yaw + change.clamp(-turn, turn));
        let arrived = d.length_squared() < 9.0;
        if !arrived && change.abs() < std::f32::consts::FRAC_PI_2 {
            let step =
                transform.rotation * Vec3::Z * (4.0 * 30.0 / 40.0 * dt).min(d.length() / 40.0);
            let before = transform.translation;
            let x = before.x + step.x;
            if !crate::shop::blocked_at(x, before.z, 0.2, &obstacles) {
                transform.translation.x = x;
            }
            let z = before.z + step.z;
            if !crate::shop::blocked_at(transform.translation.x, z, 0.2, &obstacles) {
                transform.translation.z = z;
            }
            keeper.walking = transform.translation.distance_squared(before) > 0.000001;
            keeper.running = keeper.walking;
        }
        return;
    }
    let Ok(player) = player.single() else { return };
    let corner = SHOP_ORIGIN - Vec3::new(4.0, 0.0, 4.0);
    let pos = (transform.translation - corner) * 40.0;
    let target = (player.translation - corner) * 40.0;
    let delta = (target - pos).with_y(0.0);
    let current_zone = zone(pos);
    let player_zone = zone(target);
    let can_walk = focus.is_world()
        && delta.length() >= 80.0
        && !(target.z < 120.0 && (current_zone == 2 || current_zone == 3));
    let direction = if can_walk && (current_zone != player_zone || keeper.next_zone.is_some()) {
        const NEXT: [[usize; 4]; 4] = [[0, 1, 1, 1], [0, 1, 2, 2], [1, 1, 2, 3], [2, 2, 2, 3]];
        const POINTS: [(f32, f32); 4] = [
            (220.0, 140.0),
            (100.0, 140.0),
            (100.0, 220.0),
            (220.0, 220.0),
        ];
        let next = *keeper
            .next_zone
            .get_or_insert(NEXT[current_zone][player_zone]);
        let d = Vec3::new(POINTS[next].0 - pos.x, 0.0, POINTS[next].1 - pos.z);
        if d.length_squared() < 200.0 {
            keeper.next_zone =
                (current_zone != player_zone).then_some(NEXT[current_zone][player_zone]);
        }
        d
    } else {
        keeper.next_zone = None;
        delta
    };
    let dt = time.delta_secs().min(0.1);
    if direction.length_squared() > 0.001 {
        let desired = direction.x.atan2(direction.z);
        let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let change = (desired - yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        let turn = 11.25_f32.to_radians() * 30.0 * dt;
        transform.rotation = Quat::from_rotation_y(yaw + change.clamp(-turn, turn));
        if can_walk && change.abs() < std::f32::consts::FRAC_PI_2 {
            // aNSC_set_walk_spd=1, run_spd=4 game units per 30 Hz frame.
            let speed = if delta.length() < 110.0 { 1.0 } else { 4.0 } * 30.0 / 40.0;
            let step = transform.rotation * Vec3::Z * (speed * dt).min(direction.length() / 40.0);
            if (player.translation - transform.translation - step)
                .with_y(0.0)
                .length()
                >= 1.0
            {
                let before = transform.translation;
                let x = before.x + step.x;
                // aNSC_BGcheck uses an eight-game-unit radius (8 / 40).
                if !crate::shop::blocked_at(x, before.z, 0.2, &obstacles) {
                    transform.translation.x = x;
                }
                let z = before.z + step.z;
                if !crate::shop::blocked_at(transform.translation.x, z, 0.2, &obstacles) {
                    transform.translation.z = z;
                }
                keeper.walking = transform.translation.distance_squared(before) > 0.000001;
            }
        }
    }
}

pub(crate) struct ShopNookPlugin;

fn reset(
    mut preview: ResMut<preview::Preview>,
    mut flow: ResMut<Conversation>,
    mut dialogue: ResMut<DialogueState>,
    mut pockets: ResMut<PocketsUi>,
    mut catalog_ui: ResMut<crate::catalog_screen::CatalogUi>,
) {
    preview.clear();
    *flow = Conversation::default();
    *catalog_ui = crate::catalog_screen::CatalogUi::default();
    if dialogue.external_name.is_some() {
        *dialogue = DialogueState::default();
    }
    if pockets.selling {
        *pockets = PocketsUi::default();
    }
}

impl Plugin for ShopNookPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Conversation>()
            .init_resource::<preview::Preview>()
            .init_resource::<preview::Mannequin>()
            .add_systems(
                OnExit(crate::states::AppState::InGame),
                (reset, preview::textures, preview::mannequin).chain(),
            )
            .add_systems(
                Update,
                (
                    preview::advance.before(interact),
                    interact.in_set(InteractSet::Objects),
                    preview::textures.after(interact),
                    preview::mannequin
                        .after(interact)
                        .after(crate::shop_display::show_goods),
                )
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                follow
                    .after(crate::locomotion::MovementSet)
                    .after(crate::raffle::stage)
                    .in_set(GameplaySet),
            );
    }
}
