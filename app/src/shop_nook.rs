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

pub(crate) mod editor;
pub(crate) mod guide;
pub(crate) mod handover;
pub(crate) mod password;
pub(crate) mod preview;
pub(crate) mod sales;

#[derive(Component, Default)]
pub(crate) struct Shopkeeper {
    pub walking: bool,
    /// Run clip while he crosses to the drum or a prize (`aNPC_ACT_RUN`).
    pub running: bool,
    pub(crate) next_zone: Option<usize>,
    /// The department-store twins: 0 Timmy (`SP_NPC_MAMEDANUKI0`), 1 Tommy.
    pub(crate) twin: Option<u8>,
}

/// `aNMD_selectIdx`: which twin takes the next display sale, and the one in the conversation.
#[derive(Resource, Default)]
pub(crate) struct Twins {
    pub(crate) selected: u8,
    pub(crate) talking: u8,
    /// A keeper has been asked to come and speak: the player is held (`mDemo_TYPE_SPEAK`).
    pub(crate) waiting: bool,
    /// Timmy's `aNSC_ACTION_SAY_HELLO_APPROACH`: he steps forward while he greets.
    pub(crate) approach: bool,
    /// The opening house talk is one whose keeper walks forward (`home::nook_msg::walks`).
    pub(crate) house_walk: bool,
    /// `talk_start_tim` ran out: he stops (`aNSC_ACTION_WAIT`) and turns to the player.
    pub(crate) expired: bool,
    /// The goodbye at the stairs (`aNSC_ACTION_GOODBYE_WAIT`), asked for by `department_stairs`.
    pub(crate) goodbye: Goodbye,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Goodbye {
    #[default]
    Idle,
    /// The player reached the stairs; the selected twin has not spoken yet.
    Wanted,
    Speaking,
    /// The message ended: the stairs may be taken.
    Done,
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// The part-time job's guide is speaking; the rest of his messages follow in order.
    Guide(Vec<usize>),
    Request,
    Other,
    BargainWelcome,
    BargainBalloon,
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
        sale: sales::Sale,
    },
    Dispose {
        sale: sales::Sale,
    },
    Duplicate {
        one: sales::Sale,
        all: sales::Sale,
    },
    Finish,
    /// The twins' goodbye (35) at the stairs is up.
    Goodbye,
    /// The twins' normal greeting (22) on arrival is up.
    Welcome,
    /// A display was asked for; the keeper is on his way to speak. `ticks` is -1 until the first frame.
    Greeting {
        index: usize,
        ticks: f32,
    },
    /// The password question (312) is up; the entry window opens next.
    PasswordSay,
    /// The entry window is open.
    PasswordEntry,
    /// A wrong or foreign code: try again, or give up.
    PasswordRetry,
    /// A refusal that returns to the request menu.
    PasswordBack,
    /// A valid code: the present arrives when the message ends.
    PasswordPresent {
        item: u16,
    },
    /// The receiver question (301/302) is up; the names window opens next.
    SendInfo,
    /// The receiver window is open.
    SendReceiver,
    /// 303: is this who the code is for (answers lead to 304, 301, 308).
    SendConfirm {
        next: Vec<Option<usize>>,
    },
    /// 304 is up; the pockets open to pick the item.
    SendPickWait,
    /// The pockets are open in password mode.
    SendPicking,
    /// 305: nothing was picked; try again?
    SendRetry {
        next: Vec<Option<usize>>,
    },
    /// 306: trade this item for the code (answers lead to 307, 301, 308).
    SendCheck {
        slot: usize,
        item: u16,
        next: Vec<Option<usize>>,
    },
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
    context: ShopContext,
    /// The request menu included the catalog (choice 1).
    catalog_offered: bool,
    other_offered: bool,
    /// The Other things menu included Enter a password (choice 2 of 4).
    password_offered: bool,
    /// Who the code being made is for: town and player names.
    send_to: Option<password::Receiver>,
}

#[derive(Default)]
struct ShopContext {
    text: MsgContext,
    upper: bool,
    /// Which twin speaks upstairs: 0 Timmy, 1 Tommy.
    twin: u8,
}
impl std::ops::Deref for ShopContext {
    type Target = MsgContext;
    fn deref(&self) -> &MsgContext {
        &self.text
    }
}
impl std::ops::DerefMut for ShopContext {
    fn deref_mut(&mut self) -> &mut MsgContext {
        &mut self.text
    }
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

fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &ShopContext,
    n: usize,
) -> Option<crate::messages::MsgChoice> {
    let text = disc.text.as_ref()?;
    let msg = if ctx.upper {
        if n == 15 {
            0x173D
        } else {
            n + match n {
                0..200 => 0x16FD,
                200..300 => 0x2DE7,
                _ => 0x3CF0,
            }
        }
    } else {
        message(n)
    };
    let (pages, choice) = text.talk(msg, ctx, &mut |_| 0);
    let labels = choice.as_ref().map(|c| c.labels.clone());
    if ctx.upper {
        let (name, npc) = if ctx.twin == 0 {
            ("Timmy", 276)
        } else {
            ("Tommy", 293)
        };
        dialogue.begin_special(actor, name, (npc, 2), pages, labels);
    } else {
        dialogue.begin_external(actor, pages, labels);
    }
    info!("Shop message {msg:#06x}");
    choice
}

/// A refusal or return that ends on the request menu: the menu's own
/// answers are what the player picks next.
fn ask_menu(dialogue: &DialogueState, flow: &mut Conversation) {
    if dialogue.choices_for_actor().is_some() {
        flow.catalog_offered = true;
        flow.other_offered = true;
        flow.step = Step::Request;
    } else {
        flow.step = Step::PasswordBack;
    }
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

/// A guide talk: its message strings and acquaintance first, then the messages.
fn run_guide(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    flow: &mut Conversation,
    memory: &mut crate::villager_memory::VillagerMemory,
    talk: guide::Talk,
) {
    for (slot, text) in talk.free {
        if let Some(free) = flow.context.free.get_mut(slot) {
            *free = text;
        }
    }
    if let Some(name) = talk.meet {
        memory.meet(&name);
    }
    start_guide(disc, dialogue, actor, flow, talk.chain);
}

/// The guide's messages, one after the other.
fn start_guide(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    flow: &mut Conversation,
    mut chain: Vec<usize>,
) {
    if chain.is_empty() {
        return;
    }
    let first = chain.remove(0);
    say_abs(disc, dialogue, actor, &flow.context, first);
    flow.step = Step::Guide(chain);
}

fn context(identity: &crate::save::PlayerIdentity, upper: bool) -> ShopContext {
    let now = crate::clock::now();
    ShopContext {
        upper,
        twin: 0,
        text: MsgContext {
            player_name: identity.player.clone(),
            talk_name: if upper { "Timmy" } else { "Tom Nook" }.into(),
            free: vec![String::new(); 20],
            items: vec![String::new(); 5],
            item_articles: vec![0; 5],
            month: now.month,
            day: now.day,
            hour: now.hour,
            minute: now.minute,
            weekday: now.weekday,
            ..default()
        },
    }
}

/// The keeper entity for a twin index (the only keeper elsewhere).
fn keeper<'a>(
    nook: &'a Query<(Entity, &Transform, &Shopkeeper)>,
    twin: u8,
) -> Option<(Entity, &'a Transform)> {
    nook.iter()
        .find(|(_, _, k)| k.twin.is_none_or(|t| t == twin))
        .map(|(entity, transform, _)| (entity, transform))
}

fn set_twin(ctx: &mut ShopContext, twin: u8) {
    ctx.twin = twin;
    ctx.text.talk_name = if twin == 0 { "Timmy" } else { "Tommy" }.into();
}

/// The display conversation once the keeper has come over (`aNSC_message_ctrl_force_talk_start`).
fn begin_display_sale(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    flow: &mut Conversation,
    index: usize,
    stock: &crate::shop::StockItem,
) {
    flow.context.items[0] = stock.name.clone();
    flow.context.item_articles[0] = item_article(disc, stock.item);
    flow.context.free[4] = stock.price.to_string();
    say(
        disc,
        dialogue,
        actor,
        &flow.context,
        if (0x222D..=0x2238).contains(&stock.item) {
            202
        } else if preview::Surface::of(stock.item) == Some(preview::Surface::Shirt) {
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
            index,
            item: stock.item,
            price: stock.price,
        }
    } else {
        Step::Purchase {
            index,
            item: stock.item,
            price: stock.price,
        }
    };
}

/// `aNSC_message_ctrl`: while the keeper has asked to speak, he keeps following the player; the
/// talk starts once he is within 85 units (the twins) or 80 (the others) and has turned to the
/// player, or when `talk_start_tim` (180 frames) runs out.
const GREETING_FRAMES: f32 = 180.0;

fn greeting_distance(upper: bool) -> f32 {
    if upper {
        85.0
    } else {
        80.0
    }
}

fn request(disc: &GameDisc, dialogue: &mut DialogueState, actor: Entity, flow: &mut Conversation) {
    say(disc, dialogue, actor, &flow.context, 23);
    flow.catalog_offered = false;
    flow.other_offered = false;
    if let Some(options) = dialogue.choices_for_actor() {
        if options.len() >= 4 {
            let mut labels = options[..3].to_vec();
            labels.push(options.last().cloned().unwrap_or_default());
            dialogue.set_actor_choices(labels);
            flow.catalog_offered = true;
            flow.other_offered = true;
        } else if options.len() >= 3 {
            dialogue.set_actor_choices(vec![
                options[0].clone(),
                options[1].clone(),
                options.last().cloned().unwrap_or_default(),
            ]);
            flow.catalog_offered = true;
        }
    }
    flow.step = Step::Request;
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
    nook: Query<(Entity, &Transform, &Shopkeeper)>,
    displays: Query<&ShopDisplay>,
    mut flow: ResMut<Conversation>,
    mut dialogue: ResMut<DialogueState>,
    mut pockets: ResMut<PocketsUi>,
    mut inventory: ResMut<PlayerInventory>,
    mut shop: ResMut<ShopData>,
    mut catalog_ui: ResMut<crate::catalog_screen::CatalogUi>,
    mut identity: ResMut<crate::save::PlayerIdentity>,
    layout: Res<crate::shop::layout::Layout>,
    (
        mut home,
        mut inside_for,
        mut inside_actor,
        time,
        mut catalog,
        mut preview,
        mut password,
        mut handover,
        mut twins,
        look,
        mut rand,
        mut guide_state,
        roster,
        town,
        mut memory,
    ): (
        Option<ResMut<crate::home::PlayerHouse>>,
        Local<Option<f32>>,
        Local<Option<Entity>>,
        Res<Time>,
        ResMut<crate::catalog::Catalog>,
        ResMut<preview::Preview>,
        ResMut<password::PasswordUi>,
        ResMut<handover::HandOver>,
        ResMut<Twins>,
        Res<crate::character::PlayerLook>,
        ResMut<crate::game_rand::GameRand>,
        ResMut<guide::GuideState>,
        Option<Res<crate::roster::TownRoster>>,
        Option<Res<crate::town::Town>>,
        ResMut<crate::villager_memory::VillagerMemory>,
    ),
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *location != ShopLocation::Inside {
        twins.goodbye = Goodbye::Idle;
        preview.clear();
        password.close();
        pockets.sending = false;
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
    // The first (or only) keeper is who arrival and the house talk belong to;
    // a running conversation belongs to the twin that began it.
    let Some((first, _, _)) = nook.iter().find(|(_, _, k)| k.twin.is_none_or(|t| t == 0)) else {
        preview.clear();
        return;
    };
    let Some((actor, nook_transform)) = keeper(&nook, flow.context.twin) else {
        preview.clear();
        return;
    };
    if *inside_actor != Some(first) {
        *inside_actor = Some(first);
        *inside_for = Some(0.0);
    }
    if let Step::Greeting { index, ticks } = flow.step {
        // -1 marks the request frame: `calc_talk_start_tim` then starts the 180-frame count.
        let ticks = if ticks < 0.0 {
            0.0
        } else {
            ticks + time.delta_secs() * 60.0
        };
        let Some(stock) = shop.goods().get(index).filter(|g| !g.sold).cloned() else {
            flow.step = Step::Idle;
            return;
        };
        let Ok((player_at, _, _)) = player.single() else {
            return;
        };
        let to_player = (player_at.translation - nook_transform.translation).with_y(0.0);
        let (yaw, _, _) = nook_transform.rotation.to_euler(EulerRot::YXZ);
        let turned = (to_player.x.atan2(to_player.z) - yaw + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        let near = to_player.length() * 40.0 < greeting_distance(layout.upper);
        if (near || ticks >= GREETING_FRAMES) && turned.abs() < 12.0_f32.to_radians() {
            if layout.upper {
                // The selected twin speaks, then the other is next (`aNMD_selectIdx`).
                twins.selected ^= 1;
                info!("Twins: display sale taken by twin {}", flow.context.twin);
            }
            *focus = InputFocus::Dialogue;
            begin_display_sale(&disc, &mut dialogue, actor, &mut flow, index, &stock);
        } else {
            flow.step = Step::Greeting { index, ticks };
        }
        return;
    }
    if matches!(flow.step, Step::PasswordEntry) {
        if password.open {
            return;
        }
        let Some(text) = password.done.take() else {
            flow.step = Step::Idle;
            *focus = InputFocus::World;
            return;
        };
        let roll = password.next_roll();
        let (outcome, decoded) =
            password::check(&disc, &text, &identity.player, &identity.town, roll);
        if let Some(decoded) = &decoded {
            password::fill_context(&disc, &mut flow.context, &text, decoded);
        }
        let (message, step) = match outcome {
            password::Outcome::Decline => (password::msg::DECLINE, Step::PasswordBack),
            password::Outcome::Wrong => (password::msg::WRONG, Step::PasswordRetry),
            password::Outcome::Bad => (password::msg::BAD, Step::PasswordRetry),
            password::Outcome::CardE => (password::msg::CARDE, Step::Finish),
            password::Outcome::Present { message, item } => {
                (message, Step::PasswordPresent { item })
            }
            password::Outcome::Lost => (password::msg::GOOD_MAGAZINE_2, Step::Finish),
        };
        *focus = InputFocus::Dialogue;
        say(&disc, &mut dialogue, actor, &flow.context, message);
        flow.step = step;
        if matches!(flow.step, Step::PasswordBack) {
            ask_menu(&dialogue, &mut flow);
        }
        return;
    }
    // m_passwordMake_ovl closed: back out, the same town, or confirm.
    if matches!(flow.step, Step::SendReceiver) {
        if password.open {
            return;
        }
        let Some(result) = password.receiver.take() else {
            flow.step = Step::Idle;
            *focus = InputFocus::World;
            return;
        };
        *focus = InputFocus::Dialogue;
        match result {
            None => {
                say(&disc, &mut dialogue, actor, &flow.context, 308);
                ask_menu(&dialogue, &mut flow);
            }
            Some((town, name)) => {
                flow.context.free[6] = password::field_text(&town);
                flow.context.free[7] = password::field_text(&name);
                flow.send_to = Some((town, name));
                if town == password::padded(&identity.town) {
                    say(
                        &disc,
                        &mut dialogue,
                        actor,
                        &flow.context,
                        password::msg::SAME_NAME,
                    );
                    flow.step = Step::SendInfo;
                } else {
                    let asked = say(
                        &disc,
                        &mut dialogue,
                        actor,
                        &flow.context,
                        password::msg::CONFIRM,
                    );
                    flow.step = asked.map_or(Step::Finish, |c| Step::SendConfirm { next: c.next });
                }
            }
        }
        return;
    }
    // The pockets closed after picking the item to trade in (or not).
    if matches!(flow.step, Step::SendPicking) {
        if pockets.open {
            return;
        }
        pockets.sending = false;
        *focus = InputFocus::Dialogue;
        // The key that closed the pockets must not also answer the question.
        for action in [
            Action::Cancel,
            Action::Back,
            Action::Confirm,
            Action::Interact,
        ] {
            actions.consume(action);
        }
        match pockets.sell_slot.take() {
            Some(slot) => {
                let item = inventory.pockets.pockets[slot];
                flow.context.items[2] = crate::items::name(item);
                flow.context.item_articles[2] = item_article(&disc, item);
                let asked = say(
                    &disc,
                    &mut dialogue,
                    actor,
                    &flow.context,
                    password::msg::CHECK,
                );
                flow.step = asked.map_or(Step::Finish, |c| Step::SendCheck {
                    slot,
                    item,
                    next: c.next,
                });
            }
            None => {
                let asked = say(
                    &disc,
                    &mut dialogue,
                    actor,
                    &flow.context,
                    password::msg::CANCEL,
                );
                flow.step = asked.map_or(Step::Finish, |c| Step::SendRetry { next: c.next });
            }
        }
        return;
    }
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
    if !layout.upper && was < ARRIVAL && *since >= ARRIVAL && matches!(flow.step, Step::Idle) {
        // `aNSC_start_wait`: the wait type picks the talk. The day after the part-time job
        // (HRATALK) comes first, clearing its flag as the talk starts (message -1, then the
        // approach walk); a pending sale-day balloon next and it replaces the other talks
        // (`aNSC_MSG_BALLOON`, `_DEPART`); then the house talk, then the sale or ordinary
        // greeting.
        if guide::active(&identity) {
            // `aNRG2_think_init_proc`: the guide stands in for the shop master during the job.
            let first = !std::mem::replace(&mut guide_state.greeted, true);
            let residents =
                guide::residents(roster.as_deref(), town.as_deref(), identity.arbeit_trophy);
            let talk = guide::arrival(
                &mut identity,
                &mut inventory,
                &look,
                &residents,
                &mut guide_state,
                &mut rand.0,
                first,
            );
            if let Some(talk) = talk {
                flow.context = context(&identity, false);
                *focus = InputFocus::Dialogue;
                if let Ok((_, _, mut loco)) = player.single_mut() {
                    *loco = crate::locomotion::Locomotion::default();
                }
                let end_job = talk.end_job;
                run_guide(&disc, &mut dialogue, actor, &mut flow, &mut memory, talk);
                if end_job {
                    guide_state.ending = Some(0.0);
                }
            }
            return;
        }
        if identity.events.take_hra_talk() {
            flow.context = context(&identity, false);
            *focus = InputFocus::Dialogue;
            twins.house_walk = true;
            house_say(
                &disc,
                &mut dialogue,
                actor,
                &mut flow,
                rustimal_logic::personal_events::HRATALK_MESSAGE,
                0,
            );
            if let Ok((_, _, mut loco)) = player.single_mut() {
                *loco = crate::locomotion::Locomotion::default();
            }
            return;
        }
        if shop.sale_view
            && shop.events.offer.as_ref().is_some_and(|o| !o.balloon)
            && inventory.pockets.pockets.contains(&0)
        {
            flow.context = context(&identity, false);
            *focus = InputFocus::Dialogue;
            say(
                &disc,
                &mut dialogue,
                actor,
                &flow.context,
                if layout.tier == 3 { 220 } else { 218 },
            );
            flow.step = Step::BargainBalloon;
            if let Ok((_, _, mut loco)) = player.single_mut() {
                *loco = crate::locomotion::Locomotion::default();
            }
            return;
        }
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
                flow.context = context(&identity, false);
                *focus = InputFocus::Dialogue;
                twins.house_walk = crate::home::nook_msg::walks(n);
                house_say(&disc, &mut dialogue, actor, &mut flow, message(n), 0);
                return;
            }
        }
        if shop.sale_view {
            flow.context = context(&identity, false);
            *focus = InputFocus::Dialogue;
            say(
                &disc,
                &mut dialogue,
                actor,
                &flow.context,
                if layout.tier == 3 { 16 } else { 37 },
            );
            flow.step = Step::BargainWelcome;
            if let Ok((_, _, mut loco)) = player.single_mut() {
                *loco = crate::locomotion::Locomotion::default();
            }
            return;
        }
        // `aNSC_get_start_call_msg_no`: no sale, no house talk: START_CALL_NORMAL.
        flow.context = context(&identity, false);
        *focus = InputFocus::Dialogue;
        say(&disc, &mut dialogue, actor, &flow.context, 22);
        flow.step = Step::Welcome;
        if let Ok((_, _, mut loco)) = player.single_mut() {
            *loco = crate::locomotion::Locomotion::default();
        }
        return;
    }
    if layout.upper && was < ARRIVAL && *since >= ARRIVAL && matches!(flow.step, Step::Idle) {
        // `aNMD_actor_ct`: Timmy starts in `aNSC_start_wait` and greets the player (the sale
        // greeting on a sale day, else `START_CALL_NORMAL`); Tommy just waits.
        flow.context = context(&identity, true);
        *focus = InputFocus::Dialogue;
        if shop.sale_view {
            say(&disc, &mut dialogue, actor, &flow.context, 37);
            flow.step = Step::BargainWelcome;
        } else {
            say(&disc, &mut dialogue, actor, &flow.context, 22);
            flow.step = Step::Welcome;
        }
        return;
    }
    if matches!(flow.step, Step::Pockets) {
        if pockets.open {
            return;
        }
        pockets.selling = false;
        *focus = InputFocus::Dialogue;
        let picked = pockets.sale_pick.take();
        pockets.sale_marks = 0;
        if let Some(sale) =
            picked.and_then(|mask| sales::Sale::selected(&inventory, mask, shop.turnips.price()))
        {
            let item = sale.item();
            flow.context.items[0] = crate::items::name(item);
            flow.context.item_articles[0] = item_article(&disc, item);
            if sale.has_quest() {
                // Refuse the whole captured offer; a marked batch must not
                // bypass the selected item's quest protection.
                say(&disc, &mut dialogue, actor, &flow.context, 57);
                flow.step = Step::Finish;
            } else if sale.fresh_turnips() && crate::clock::now().weekday == 0 {
                say(&disc, &mut dialogue, actor, &flow.context, 36);
                flow.step = Step::Finish;
            } else if sale.price == 0 {
                let plural = sale.slots() > 1;
                say(
                    &disc,
                    &mut dialogue,
                    actor,
                    &flow.context,
                    if plural { 216 } else { 56 },
                );
                flow.step = Step::Dispose { sale };
            } else if let Some(all) = sale.duplicates(&inventory) {
                let count = all.quantity();
                flow.context.items[2] = crate::items::name(item);
                flow.context.item_articles[2] = item_article(&disc, item);
                flow.context.free[7] = count.to_string();
                let unit = if item >> 12 == 2 {
                    disc.catalog
                        .get("aNSC_item_tanni_type")
                        .and_then(|a| {
                            disc.rom
                                .pointer_at(a.source, a.offset + u32::from((item >> 8) & 15) * 4)
                        })
                        .and_then(|p| match p {
                            disc::Target::File(source, at) => {
                                disc.rom.slice(source, at + u32::from(item & 255), 1)
                            }
                            _ => None,
                        })
                        .and_then(|b| b.first())
                        .copied()
                        .unwrap_or(0)
                        .min(9)
                } else {
                    0
                };
                let bases = [
                    0x566, 0x575, 0x584, 0x593, 0x5a2, 0x5b1, 0x5c0, 0x5cf, 0x5de, 0x5ed,
                ];
                flow.context.free[8] = disc
                    .text
                    .as_ref()
                    .and_then(|t| t.string(bases[usize::from(unit)] + count as usize - 1))
                    .unwrap_or_default();
                if let Some(one) = sale.one() {
                    say(&disc, &mut dialogue, actor, &flow.context, 15);
                    flow.step = Step::Duplicate { one, all };
                }
            } else {
                flow.context.free[2] = sale.price.to_string();
                say(
                    &disc,
                    &mut dialogue,
                    actor,
                    &flow.context,
                    if sale.slots() > 1 { 214 } else { 27 },
                );
                flow.step = Step::Offer { sale };
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
            Step::Guide(mut rest) => {
                if rest.is_empty() {
                    *dialogue = DialogueState::default();
                    *focus = InputFocus::World;
                } else {
                    let next = rest.remove(0);
                    say_abs(&disc, &mut dialogue, actor, &flow.context, next);
                    flow.step = Step::Guide(rest);
                }
            }
            Step::Goodbye => {
                twins.goodbye = Goodbye::Done;
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
            }
            Step::BargainWelcome => {
                let gift = !layout.upper
                    && shop.events.offer.as_ref().is_some_and(|o| !o.balloon)
                    && inventory.pockets.pockets.contains(&0);
                if gift {
                    say(
                        &disc,
                        &mut dialogue,
                        actor,
                        &flow.context,
                        if layout.tier == 3 { 220 } else { 218 },
                    );
                    flow.step = Step::BargainBalloon;
                } else {
                    flow.step = Step::Idle;
                    *focus = InputFocus::World;
                }
            }
            Step::BargainBalloon => {
                if let Some(slot) = inventory.pockets.pockets.iter().position(|&i| i == 0) {
                    if let Some(offer) = shop.events.offer.as_mut().filter(|o| !o.balloon) {
                        let roll = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map_or(0, |d| d.subsec_nanos());
                        let item = 0x2244 + (roll % 8) as u16;
                        inventory.pockets.pockets[slot] = item;
                        inventory.pockets.set_item_cond(slot, 1);
                        offer.balloon = true;
                        crate::sound::se(0x69);
                        // aNSC_present_balloon_trans_takeout: the wrapped
                        // balloon is handed over like any present.
                        handover.start(handover::Request::nook_present());
                    }
                }
                flow.step = Step::Idle;
                *focus = InputFocus::World;
            }
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
            Step::Request if choice == 2 && flow.other_offered => {
                say(&disc, &mut dialogue, actor, &flow.context, 309);
                // Turnip prices, Send a code, Say code and the way back.
                flow.password_offered = dialogue
                    .choices_for_actor()
                    .is_some_and(|options| options.len() >= 4);
                flow.step = Step::Other;
            }
            Step::Other if choice == 0 => {
                flow.context.free[1] = shop.turnips.price().to_string();
                say(
                    &disc,
                    &mut dialogue,
                    actor,
                    &flow.context,
                    if crate::clock::now().weekday == 0 {
                        36
                    } else {
                        25
                    },
                );
                flow.step = Step::Finish;
            }
            Step::Other if choice == 1 && flow.password_offered => {
                // aNSC_request_Q_answer_wait2, choice 1: something to trade in?
                if password::tradeable(&inventory.pockets) {
                    say(
                        &disc,
                        &mut dialogue,
                        actor,
                        &flow.context,
                        password::msg::INFO,
                    );
                    flow.step = Step::SendInfo;
                } else {
                    say(
                        &disc,
                        &mut dialogue,
                        actor,
                        &flow.context,
                        password::msg::NO_ITEM,
                    );
                    ask_menu(&dialogue, &mut flow);
                }
            }
            Step::Other if choice == 2 && flow.password_offered => {
                // aNSC_request_Q_answer_wait2, choice 2: the gift limit, then
                // a free pocket, before the question.
                let message = if password.presents >= password::PRESENT_LIMIT {
                    password::msg::GIFT_OUT
                } else if !inventory.pockets.pockets.contains(&0) {
                    password::msg::INVENTORY_FULL
                } else {
                    password::msg::SAY
                };
                say(&disc, &mut dialogue, actor, &flow.context, message);
                if message == password::msg::SAY {
                    flow.step = Step::PasswordSay;
                } else {
                    ask_menu(&dialogue, &mut flow);
                }
            }
            Step::Other | Step::PasswordBack => request(&disc, &mut dialogue, actor, &mut flow),
            Step::PasswordSay => {
                password.clear();
                password.begin();
                *dialogue = DialogueState::default();
                flow.step = Step::PasswordEntry;
            }
            Step::PasswordRetry => {
                if choice == 0 {
                    password.begin();
                    *dialogue = DialogueState::default();
                    flow.step = Step::PasswordEntry;
                } else {
                    say(
                        &disc,
                        &mut dialogue,
                        actor,
                        &flow.context,
                        password::msg::DECLINE,
                    );
                    ask_menu(&dialogue, &mut flow);
                }
            }
            Step::SendInfo => {
                password.begin_receiver();
                *dialogue = DialogueState::default();
                flow.step = Step::SendReceiver;
            }
            Step::SendPickWait => {
                *pockets = PocketsUi {
                    open: true,
                    sending: true,
                    ..default()
                };
                *dialogue = DialogueState::default();
                *focus = InputFocus::Pockets;
                crate::sound::se(crate::sound::se_id::MENU_PAUSE);
                flow.step = Step::SendPicking;
            }
            Step::SendConfirm { next } => {
                let target = next.get(choice).copied().flatten();
                if let Some(message) = target {
                    say_abs(&disc, &mut dialogue, actor, &flow.context, message);
                }
                match choice {
                    0 if target.is_some() => flow.step = Step::SendPickWait,
                    1 if target.is_some() => flow.step = Step::SendInfo,
                    _ => ask_menu(&dialogue, &mut flow),
                }
            }
            Step::SendRetry { next } => {
                let target = next.get(choice).copied().flatten();
                if let Some(message) = target {
                    say_abs(&disc, &mut dialogue, actor, &flow.context, message);
                }
                if choice == 0 && target.is_some() {
                    flow.step = Step::SendPickWait;
                } else {
                    ask_menu(&dialogue, &mut flow);
                }
            }
            Step::SendCheck { slot, item, next } => {
                let target = next.get(choice).copied().flatten();
                let valid = inventory.pockets.pockets[slot] == item
                    && inventory.pockets.item_cond(slot) == 0
                    && password::present_user(item);
                match (choice, flow.send_to, target) {
                    (0, Some((town, name)), Some(message)) if valid => {
                        // mMpswd_make_password; the item leaves the pocket now.
                        let code = password::make_user_code(&town, &name, item);
                        inventory.pockets.pockets[slot] = 0;
                        inventory.pockets.set_item_cond(slot, 0);
                        password::fill_code_lines(&mut flow.context, &code);
                        info!("password sent item={item:#06x}");
                        say_abs(&disc, &mut dialogue, actor, &flow.context, message);
                        flow.step = Step::Finish;
                    }
                    (1, _, Some(message)) => {
                        say_abs(&disc, &mut dialogue, actor, &flow.context, message);
                        flow.step = Step::SendInfo;
                    }
                    _ => {
                        say(&disc, &mut dialogue, actor, &flow.context, 308);
                        ask_menu(&dialogue, &mut flow);
                    }
                }
            }
            Step::PasswordPresent { item } => {
                // aNSC_pc_present_trans_takeout_init, once the message ends.
                if inventory
                    .pockets
                    .put(item, password::PRESENT_CONDITION)
                    .is_some()
                {
                    password.presents += 1;
                    handover.start(handover::Request::nook_present());
                    crate::sound::se(crate::sound::se_id::POCKET_RUSTLE);
                    info!("password present item={item:#06x}");
                }
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
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
                    .goods()
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
                    .goods()
                    .get(index)
                    .is_some_and(|s| s.item == item && !s.sold && s.price == price);
                let paint = (0x222D..=0x2238).contains(&item);
                let response = if choice != 0 || !valid {
                    31
                } else if inventory.bells < price {
                    33
                } else if paint && home.is_none() {
                    31
                } else if !paint && !inventory.add(item) {
                    39
                } else {
                    if paint {
                        // Source queues the exterior palette, even with full pockets.
                        if let Some(home) = home.as_mut() {
                            home.0.order_paint(item);
                        }
                    }
                    inventory.bells -= price;
                    shop.goods_mut()[index].sold = true;
                    shop.shop.record_purchase(price);
                    crate::sound::se(crate::sound::se_id::REGISTER);
                    info!("Nook purchase stock={index} item={item:#06x} price={price}");
                    // aNSC_sell_answer0: a lottery ticket with furniture,
                    // clothing, wallpaper, carpets and umbrellas, onto this
                    // month's stack or a free slot (GIVE_TICKET), else by
                    // mail (MAIL_TICKET); otherwise the tool's own line.
                    let month = crate::clock::now().month;
                    if paint {
                        203
                    } else if crate::catalog::comes_with_ticket(item) {
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
            Step::Duplicate { one, all } => {
                let sale = if choice == 0 { all } else { one };
                if !sale.valid(&inventory) || !sale.at_price(shop.turnips.price()) {
                    say(&disc, &mut dialogue, actor, &flow.context, 29);
                    flow.step = Step::Finish;
                } else {
                    flow.context.free[2] = sale.price.to_string();
                    say(&disc, &mut dialogue, actor, &flow.context, 27);
                    flow.step = Step::Offer { sale };
                }
            }
            Step::Dispose { sale } => {
                if choice == 0 && sale.discard(&mut inventory) {
                    info!("Nook disposal slots={}", sale.slots());
                    say(&disc, &mut dialogue, actor, &flow.context, 58);
                } else {
                    say(&disc, &mut dialogue, actor, &flow.context, 59);
                }
                flow.step = Step::Finish;
            }
            Step::Offer { sale } => {
                let response = if choice != 0
                    || !sale.valid(&inventory)
                    || !sale.at_price(shop.turnips.price())
                {
                    29
                } else if let Some(payment) = sale.commit(&mut inventory) {
                    shop.shop.record_sale(sale.price);
                    crate::sound::se(crate::sound::se_id::REGISTER);
                    info!("Nook sale slots={} payout={}", sale.slots(), sale.price);
                    match payment {
                        sales::Payment::Wallet => 28,
                        sales::Payment::Bags => 38,
                    }
                } else {
                    200
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
    if twins.goodbye == Goodbye::Wanted && matches!(flow.step, Step::Idle) && focus.is_world() {
        // `aNSC_goodbye_wait`: the keeper turns and says goodbye; upstairs it is the selected twin
        // (the other, `_WAIT2`, only turns, which `follow_twins` does for both while a
        // conversation runs).
        let twin = if layout.upper { twins.selected } else { 0 };
        flow.context = context(&identity, layout.upper);
        if layout.upper {
            set_twin(&mut flow.context, twin);
        }
        twins.talking = twin;
        let actor = keeper(&nook, twin).map_or(actor, |(entity, _)| entity);
        *focus = InputFocus::Dialogue;
        say(&disc, &mut dialogue, actor, &flow.context, 35);
        flow.step = Step::Goodbye;
        twins.goodbye = Goodbye::Speaking;
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
    flow.context = context(&identity, layout.upper);
    let (mut actor, mut nook_transform) = (actor, nook_transform);
    if let Some(display) = display {
        if layout.upper {
            // A display sale goes to the selected twin, then the other is next.
            set_twin(&mut flow.context, twins.selected);
        }
        let Some(stock) = shop.goods().get(display.stock_index) else {
            return;
        };
        actions.consume(Action::Interact);
        if guide::active(&identity) {
            // `aNRG2_goods_talk_request`: the goods are out of reach during the job.
            *focus = InputFocus::Dialogue;
            start_guide(
                &disc,
                &mut dialogue,
                actor,
                &mut flow,
                vec![guide::GOODS_MESSAGE],
            );
            return;
        }
        if stock.sold {
            return;
        }
        // The keeper requests to speak (`mDemo_TYPE_SPEAK`); the player is held until he is close.
        twins.talking = flow.context.twin;
        flow.step = Step::Greeting {
            index: display.stock_index,
            ticks: -1.0,
        };
    } else {
        // Talking: to whichever twin the player faces.
        let faced = nook.iter().find(|(_, t, k)| {
            let delta = (t.translation - transform.translation).with_y(0.0);
            k.twin.is_some()
                && delta.length() <= 1.8
                && facing.dir.dot(delta.normalize_or_zero()) >= 0.5
        });
        if let Some((entity, t, k)) = faced {
            (actor, nook_transform) = (entity, t);
            set_twin(&mut flow.context, k.twin.unwrap_or(0));
            twins.talking = flow.context.twin;
        }
        let delta = (nook_transform.translation - transform.translation).with_y(0.0);
        if delta.length() > 1.8 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
            return;
        }
        actions.consume(Action::Interact);
        if guide::active(&identity) {
            let residents =
                guide::residents(roster.as_deref(), town.as_deref(), identity.arbeit_trophy);
            if let Some(talk) = guide::talk(
                &mut identity,
                &mut inventory,
                &look,
                &residents,
                &mut guide_state,
                &mut rand.0,
            ) {
                *focus = InputFocus::Dialogue;
                let end_job = talk.end_job;
                run_guide(&disc, &mut dialogue, actor, &mut flow, &mut memory, talk);
                if end_job {
                    guide_state.ending = Some(0.0);
                }
                return;
            }
        }
        *focus = InputFocus::Dialogue;
        if shop.sale_view && layout.tier == 3 && !layout.upper {
            say(&disc, &mut dialogue, actor, &flow.context, 17);
            flow.step = Step::Finish;
            return;
        }
        request(&disc, &mut dialogue, actor, &mut flow);
    }
}

/// `aNSC_decide_next_move_act` for the shop masters: wait within `aNSC_PLAYER_DISTANCE` (60) of
/// the player, walk inside 90 and run beyond. The twins' own variant uses 80 and 110.
const STOP_DISTANCE: f32 = 60.0;
const WALK_DISTANCE: f32 = 90.0;

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
    twins: Res<Twins>,
    player: Query<&Transform, With<Player>>,
    mut nook: Query<(&mut Transform, &mut Shopkeeper), Without<Player>>,
    obstacles: Res<crate::shop::ShopObstacles>,
    layout: Res<crate::shop::layout::Layout>,
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
        let corner = SHOP_ORIGIN - layout.centre().with_y(0.0);
        let pos = (transform.translation - corner) * 40.0;
        let d = Vec3::new(goal.x - pos.x, 0.0, goal.y - pos.z);
        let at_drum = goal.distance(layout.raffle_nook()) < 1.0;
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
    let corner = SHOP_ORIGIN - layout.centre().with_y(0.0);
    if twins.approach && !layout.upper {
        // `aNSC_say_hello_approach`: while he greets, the keeper walks forward (+z) until
        // `aNSC_POS_Z_MAX` (260, 260, 420, 500 by shop) or the talk ends.
        let limit = [260.0, 260.0, 420.0, 500.0][usize::from(layout.tier.min(3))];
        let z = (transform.translation.z - corner.z) * 40.0;
        if z < limit {
            let dt = time.delta_secs().min(0.1);
            let next = transform.translation.z + 30.0 / 40.0 * dt;
            transform.rotation = Quat::IDENTITY;
            if !crate::shop::blocked_at(transform.translation.x, next, 0.2, &obstacles) {
                transform.translation.z = next.min(corner.z + limit / 40.0);
                keeper.walking = true;
            }
        }
        return;
    }
    let Ok(player) = player.single() else { return };
    let pos = (transform.translation - corner) * 40.0;
    let target = (player.translation - corner) * 40.0;
    let delta = (target - pos).with_y(0.0);
    let route_zone = |p| {
        if layout.tier == 0 {
            zone(p)
        } else {
            crate::shop::layout::follow::zone(layout.tier, p)
        }
    };
    let route_next = |c: usize, p: usize| {
        if layout.tier == 0 {
            [[0, 1, 1, 1], [0, 1, 2, 2], [1, 1, 2, 3], [2, 2, 2, 3]][c][p]
        } else {
            crate::shop::layout::follow::next(layout.tier, c, p)
        }
    };
    let current_zone = route_zone(pos);
    let player_zone = route_zone(target);
    let can_walk = !layout.upper
        && (focus.is_world() || twins.waiting)
        && !twins.expired
        && delta.length() >= STOP_DISTANCE
        && if layout.tier == 0 {
            // aNSM_check_safe_zone.
            !(target.z > 280.0 && current_zone == 3)
        } else {
            !crate::shop::layout::follow::safe(layout.tier, target, current_zone)
        };
    let direction = if can_walk && (current_zone != player_zone || keeper.next_zone.is_some()) {
        const POINTS: [(f32, f32); 4] = [
            (220.0, 140.0),
            (100.0, 140.0),
            (100.0, 220.0),
            (220.0, 220.0),
        ];
        let next = *keeper
            .next_zone
            .get_or_insert(route_next(current_zone, player_zone));
        let point = if layout.tier == 0 {
            Vec3::new(POINTS[next].0, 0.0, POINTS[next].1)
        } else {
            crate::shop::layout::follow::point(layout.tier, next)
        };
        let d = (point - pos).with_y(0.0);
        if d.length_squared() < 200.0 {
            keeper.next_zone =
                (current_zone != player_zone).then_some(route_next(current_zone, player_zone));
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
            let speed = if delta.length() < WALK_DISTANCE {
                1.0
            } else {
                4.0
            } * 30.0
                / 40.0;
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

/// The twins on the upper floor (`ac_npc_mamedanuki.c`, the `aNSC_MAMEDANUKI` branch of
/// `aNSC_decide_next_move_act`): each follows the player along the route of the floor's sale type,
/// waits within 80, walks inside 110 and runs beyond.
#[allow(clippy::too_many_arguments)]
fn hold_player(flow: Res<Conversation>, mut twins: ResMut<Twins>) {
    let waiting = matches!(flow.step, Step::Greeting { .. });
    let expired = matches!(flow.step, Step::Greeting { ticks, .. } if ticks >= GREETING_FRAMES);
    if twins.waiting != waiting {
        twins.waiting = waiting;
    }
    if matches!(flow.step, Step::Idle) && twins.house_walk {
        twins.house_walk = false;
    }
    let approach = matches!(flow.step, Step::Welcome | Step::BargainWelcome) || twins.house_walk;
    if twins.approach != approach {
        twins.approach = approach;
    }
    if twins.expired != expired {
        twins.expired = expired;
    }
}

fn follow_twins(
    disc: Option<Res<GameDisc>>,
    location: Res<State<ShopLocation>>,
    focus: Res<InputFocus>,
    time: Res<Time>,
    shop: Res<ShopData>,
    hold: Res<Twins>,
    player: Query<&Transform, With<Player>>,
    mut twins: Query<(&mut Transform, &mut Shopkeeper), Without<Player>>,
    obstacles: Res<crate::shop::ShopObstacles>,
    layout: Res<crate::shop::layout::Layout>,
) {
    use rustimal_logic::twin_route as route;
    if disc.is_none() || *location != ShopLocation::Inside || !layout.upper {
        return;
    }
    let sale = if shop.events.active(crate::clock::now()) {
        // `special_sale_type[]` by the offer's kind.
        match shop.events.offer.as_ref().map(|o| o.kind) {
            Some(0) => route::SaleType::Furniture,
            Some(2) => route::SaleType::Clothing,
            Some(3 | 4) => route::SaleType::Carpet,
            _ => route::SaleType::Normal,
        }
    } else {
        route::SaleType::Normal
    };
    let corner = SHOP_ORIGIN - layout.centre().with_y(0.0);
    let zone_of = |p: Vec3| {
        route::zone(
            sale,
            (p.x / 40.0).floor() as i32,
            (p.z / 40.0).floor() as i32,
        )
    };
    let dt = time.delta_secs().min(0.1);
    let player = player
        .single()
        .ok()
        .map(|t| (t.translation - corner) * 40.0);
    for (mut transform, mut keeper) in &mut twins {
        keeper.walking = false;
        keeper.running = false;
        let Some(target) = player else {
            continue;
        };
        let pos = (transform.translation - corner) * 40.0;
        if hold.approach && keeper.twin == Some(0) {
            // `aNSC_say_hello_approach`: walk toward the player until z < 160, then stand.
            if pos.z >= 160.0 {
                transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
                transform.translation.z =
                    (transform.translation.z - 1.0 * 30.0 / 40.0 * dt).max(corner.z + 160.0 / 40.0);
                keeper.walking = true;
            }
            continue;
        }
        let delta = (target - pos).with_y(0.0);
        let distance = delta.length();
        let (zone, player_zone) = (zone_of(pos), zone_of(target));
        // The wait cases: the player at the door end of the floor, or close.
        let waiting = !(focus.is_world() || hold.waiting)
            || hold.expired
            || (target.z < 120.0 && (zone == 2 || zone == 3))
            || distance < 80.0;
        let travelling = !waiting && zone != player_zone;
        let direction = if travelling {
            let next = *keeper
                .next_zone
                .get_or_insert_with(|| route::next_zone(sale, player_zone, zone));
            let (x, z) = route::point(sale, next);
            let d = (Vec3::new(x, 0.0, z) - pos).with_y(0.0);
            if d.length_squared() < 200.0 {
                keeper.next_zone = Some(route::next_zone(sale, player_zone, zone));
            }
            d
        } else {
            keeper.next_zone = None;
            delta
        };
        if direction.length_squared() <= 0.001 {
            continue;
        }
        let desired = direction.x.atan2(direction.z);
        let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let change = (desired - yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        let turn = 11.25_f32.to_radians() * 30.0 * dt;
        transform.rotation = Quat::from_rotation_y(yaw + change.clamp(-turn, turn));
        if waiting || change.abs() >= std::f32::consts::FRAC_PI_2 {
            continue;
        }
        let run = distance >= 110.0;
        let speed = if run { 4.0 } else { 1.0 } * 30.0 / 40.0;
        let step = transform.rotation * Vec3::Z * (speed * dt).min(direction.length() / 40.0);
        let before = transform.translation;
        let x = before.x + step.x;
        if !crate::shop::blocked_at(x, before.z, 0.2, &obstacles) {
            transform.translation.x = x;
        }
        // `aNMD_actor_move` keeps them off the stair side of the floor (z >= 138).
        let z = (before.z + step.z).max(corner.z + 138.0 / 40.0);
        if !crate::shop::blocked_at(transform.translation.x, z, 0.2, &obstacles) {
            transform.translation.z = z;
        }
        keeper.walking = transform.translation.distance_squared(before) > 0.000001;
        keeper.running = keeper.walking && run;
    }
    // Their collision circles (radius 20) keep them apart; the heavier twin (`aNMD_weight_ctrl`:
    // Timmy 50, Tommy 30) gives way less.
    let mut pairs = twins.iter_combinations_mut();
    while let Some([(mut a, ka), (mut b, kb)]) = pairs.fetch_next() {
        let delta = (b.translation - a.translation).with_y(0.0);
        let gap = delta.length();
        if gap >= 1.0 {
            continue;
        }
        let direction = if gap > 0.0001 { delta / gap } else { Vec3::X };
        let weight = |k: &Shopkeeper| if k.twin == Some(0) { 50.0 } else { 30.0 };
        let (wa, wb) = (weight(&ka), weight(&kb));
        let push = 1.0 - gap;
        a.translation -= direction * push * (wb / (wa + wb));
        b.translation += direction * push * (wa / (wa + wb));
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
            .init_resource::<Twins>()
            .init_resource::<password::PasswordUi>()
            .init_resource::<password::PasswordAssets>()
            .init_resource::<handover::HandOver>()
            .init_resource::<preview::Preview>()
            .init_resource::<editor::CaretClock>()
            .init_resource::<guide::GuideState>()
            .add_systems(Update, guide::track.in_set(GameplaySet))
            .add_systems(Update, guide::send_away.in_set(GameplaySet))
            .add_systems(Update, guide::hand_over_job_item.in_set(GameplaySet))
            .add_systems(PostUpdate, editor::pulse)
            .init_resource::<preview::Mannequin>()
            .init_resource::<preview::effects::Effects>()
            .add_systems(
                OnExit(crate::states::AppState::InGame),
                (
                    reset,
                    preview::textures,
                    preview::mannequin,
                    preview::effects::reset,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    preview::advance.before(interact),
                    interact.in_set(InteractSet::Objects),
                    hold_player.after(interact),
                    (password::input, password::draw, password::scroll)
                        .chain()
                        .after(interact),
                    handover::animate
                        .after(interact)
                        .after(crate::character::CharacterAnimationSet),
                    preview::textures.after(interact),
                    preview::mannequin
                        .after(interact)
                        .after(crate::shop_display::show_goods),
                    preview::effects::update
                        .after(crate::character::CharacterAnimationSet)
                        .after(crate::save::LoadSet)
                        .after(crate::player::camera_follow),
                )
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                follow_twins
                    .after(crate::locomotion::MovementSet)
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
