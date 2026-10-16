//! Quests (villager requests), on the translated quest code.
//!
//! From the decomp (`rustimal_logic::quest`, `quest_gen`):
//! - type and kind: `select_quest_type` / `select_quest_kind`, gated by
//!   `occur_ok` (season rules, e.g. bug contests only Mar-Nov) and the
//!   75% `quest_attempt_roll`;
//! - completion: deliveries need the parcel at the recipient; contests use
//!   `contest_complete` (fruit: the item is in your pockets; fish/bug: you
//!   carry one of that category);
//! - time limits: `limit_days` (deliveries 2 days, contests 1-3);
//! - rewards: the category is rolled from `DELIVERY_REWARDS` /
//!   `ERRAND_REWARDS` / `CONTEST_REWARDS` with `prob_table_select`; money
//!   uses the max pay tables and `scaled_pay` (+/-10%); a reward that
//!   doesn't fit in your pockets is owed and handed over next time you talk
//!   (`give_reward`);
//! - friendship: declining -3, completing +3 (`quest::friendship`).
//!
//! The slice supports normal deliveries, furniture errands and fruit /
//! fish / bug contests. Other original kinds (lost items, foreign or
//! removed-villager deliveries, soccer, snowman, flower and letter
//! contests) need systems the slice doesn't have, so rolling one fails the
//! attempt like a failed occur check. Ours: the dialogue text, the errand
//! chain counter (`used_num` = requests completed + 1), and the expiry
//! friendship penalty.

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy::text::FontSize;
use rustimal_logic::quest::{self, ckind, dkind, ekind, qtype, reward};
use rustimal_logic::quest_gen;
use serde::{Deserialize, Serialize};

use crate::dialogue::{DialogueState, VillagerTalk};
use crate::house::HouseState;
use crate::input::{Action, Actions, InteractSet};
use crate::interaction::{PlayerInventory, TalkEvent};
use crate::items;
use crate::npc::Villager;
use crate::states::GameplaySet;

/// What kind of errand the player is running.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum QuestKind {
    /// Carry a parcel from `giver` to `recipient`.
    Delivery { giver: String, recipient: String },
    /// Bring `amount` fruit to `giver`.
    ErrandFruit { giver: String, amount: u32 },
    /// Catch a fish for `giver`.
    CatchFish { giver: String },
    /// Catch a bug for `giver`.
    CatchBug { giver: String },
    /// Buy `item` from Nook's for `giver`.
    BuyFurniture { giver: String, item: String },
}

/// A live quest. Serializable so F5/F9 can persist it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveQuest {
    pub kind: QuestKind,
    /// Game-clock day it was accepted (`clock::Now::day_number`).
    #[serde(default)]
    pub accepted_day: i64,
    /// Days allowed (`quest::limit_days`); 0 = no limit.
    #[serde(default)]
    pub limit_days: i32,
    /// Errand chain counter for the reward tier.
    #[serde(default)]
    pub used_num: u8,
}

/// What a finished quest pays.
#[derive(Clone, Copy, Debug)]
enum Reward {
    Bells(u32),
    Item(items::ItemId),
}

/// A villager with a `!` marker, holding a generated offer.
#[derive(Clone, Debug)]
pub struct PendingOffer {
    pub giver: String,
    pub kind: QuestKind,
    pub limit_days: i32,
}

/// Quest runtime state.
#[derive(Resource)]
pub struct QuestState {
    pub active: Option<ActiveQuest>,
    cooldown: f32,
    pub completed: u32,
    rng: u64,
    /// A reward that didn't fit in the pockets: (giver, reward), handed over
    /// next time you talk to them (retail `give_reward`).
    owed: Option<(String, Reward)>,
}

impl Default for QuestState {
    fn default() -> Self {
        Self {
            active: None,
            // First quest shows up quickly so playtesters see the system.
            cooldown: 20.0,
            completed: 0,
            rng: 0x51ED_752B_9C3A_11A7,
            owed: None,
        }
    }
}

impl QuestState {
    fn rng_next(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x
    }
}

/// Villager name currently offering a quest (marker shown).
#[derive(Resource, Default)]
pub struct QuestGiver(pub Option<PendingOffer>);

/// Quest panel state. Other plugins read `open` to yield the E key.
#[derive(Resource, Default)]
pub struct QuestUi {
    pub open: bool,
    mode: QuestUiMode,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum QuestUiMode {
    #[default]
    Offer,
    TurnIn,
}

/// Talk events the quest system claims, so the normal dialogue box stays shut.
#[derive(Resource, Default)]
pub struct QuestClaim(pub Option<Entity>);

/// System set for the talk interceptor — runs after `interact` sends the
/// `TalkEvent`, before dialogue's `on_talk` reads it.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestInputSet;

/// Floating `!` above a quest giver's head.
#[derive(Component)]
struct QuestMarker;

/// Marker for the quest panel node.
#[derive(Component)]
struct QuestPanel;

/// Marker for the quest panel text.
#[derive(Component)]
struct QuestPanelText;

/// Marker for the quest HUD line.
#[derive(Component)]
struct QuestHudText;

/// Marker for the quest notification text.
#[derive(Component)]
struct QuestNotifyText;

/// Countdown for the notification.
#[derive(Resource, Default)]
struct QuestNotifyTimer(f32);

const NOTIFY_DURATION: f32 = 3.0;
const QUEST_COOLDOWN_SECS: f32 = 300.0;
const DECLINE_COOLDOWN_SECS: f32 = 120.0;
/// After a failed attempt (roll, occur check, kind not in the slice).
const RETRY_COOLDOWN_SECS: f32 = 30.0;

/// Decomp (type, kind) for a quest.
fn type_kind(kind: &QuestKind) -> (u8, u8) {
    match kind {
        QuestKind::Delivery { .. } => (qtype::DELIVERY, dkind::NORMAL),
        QuestKind::BuyFurniture { .. } => (qtype::ERRAND, ekind::REQUEST),
        QuestKind::ErrandFruit { .. } => (qtype::CONTEST, ckind::FRUIT),
        QuestKind::CatchFish { .. } => (qtype::CONTEST, ckind::FISH),
        QuestKind::CatchBug { .. } => (qtype::CONTEST, ckind::INSECT),
    }
}

fn giver_of(kind: &QuestKind) -> &str {
    match kind {
        QuestKind::Delivery { giver, .. }
        | QuestKind::ErrandFruit { giver, .. }
        | QuestKind::CatchFish { giver }
        | QuestKind::CatchBug { giver }
        | QuestKind::BuyFurniture { giver, .. } => giver,
    }
}

/// Roll a quest for `giver` (`select_quest_type` -> `select_quest_kind` ->
/// `occur_ok`). `None` means this attempt fails (as retail does on a failed
/// occur check) — including kinds the slice can't run yet.
fn generate_quest(
    rolls: [u32; 3],
    giver: &str,
    others: &[String],
    stock: &[String],
    now: &crate::clock::Now,
) -> Option<(QuestKind, i32)> {
    let qt = quest_gen::select_quest_type(false, rolls[0]);
    let qk = quest_gen::select_quest_kind(qt, false, rolls[1])?;
    let ctx = quest_gen::OccurCtx {
        month: now.month,
        day: now.day,
        hour: now.hour,
        local_player: true,
        ..Default::default()
    };
    if !quest_gen::occur_ok(qt, qk, &ctx) {
        return None;
    }
    let giver_s = giver.to_string();
    let kind = match (qt, qk) {
        (qtype::DELIVERY, dkind::NORMAL) => {
            let candidates: Vec<&String> = others.iter().filter(|n| *n != giver).collect();
            let recipient = candidates.get(rolls[2] as usize % candidates.len().max(1))?;
            QuestKind::Delivery {
                giver: giver_s,
                recipient: recipient.to_string(),
            }
        }
        (qtype::ERRAND, ekind::REQUEST) => QuestKind::BuyFurniture {
            giver: giver_s,
            item: stock.get(rolls[2] as usize % stock.len().max(1))?.clone(),
        },
        (qtype::CONTEST, ckind::FRUIT) => QuestKind::ErrandFruit {
            giver: giver_s,
            amount: 1,
        },
        (qtype::CONTEST, ckind::FISH) => QuestKind::CatchFish { giver: giver_s },
        (qtype::CONTEST, ckind::INSECT) => QuestKind::CatchBug { giver: giver_s },
        _ => return None, // not in the slice yet
    };
    // Contests are "in progress" (progress 1) while you work on them.
    let progress = if qt == qtype::CONTEST { 1 } else { 0 };
    let limit = quest::limit_days(qt, qk, progress).unwrap_or(0);
    Some((kind, limit))
}

/// Days left on a quest today (`None` = no limit).
fn days_left(active: &ActiveQuest, today: i64) -> Option<i64> {
    // Clamped: a quest saved under another day numbering never goes negative.
    let elapsed = (today - active.accepted_day).max(0);
    (active.limit_days > 0).then(|| active.limit_days as i64 - elapsed)
}

fn limit_phrase(days: i32) -> String {
    match days {
        0 => String::new(),
        1 => "\n(Please do it by tomorrow.)".to_string(),
        n => format!("\n(You have {n} days.)"),
    }
}

fn offer_text(offer: &PendingOffer) -> String {
    let ask = match &offer.kind {
        QuestKind::Delivery { recipient, .. } => {
            format!("Could you take this parcel to {recipient} for me?")
        }
        QuestKind::ErrandFruit { .. } => "I'm craving fruit! Could you bring me one?".to_string(),
        QuestKind::CatchFish { .. } => {
            "I'd love a fresh fish! Could you catch one for me?".to_string()
        }
        QuestKind::CatchBug { .. } => {
            "Have you seen the bugs around here? Catch one for me!".to_string()
        }
        QuestKind::BuyFurniture { item, .. } => {
            format!("I have my eye on a {item} at the shop. Could you get one for me?")
        }
    };
    format!(
        "{}: {}\nI'll make it worth your while!{}\n\nY: Accept    N: Decline",
        offer.giver,
        ask,
        limit_phrase(offer.limit_days)
    )
}

fn turnin_text(active: &ActiveQuest) -> String {
    let what = match &active.kind {
        QuestKind::Delivery { recipient, .. } => format!("Deliver the parcel to {recipient}?"),
        QuestKind::ErrandFruit { .. } => "Hand over a fruit?".to_string(),
        QuestKind::CatchFish { .. } => "Hand over a fish from your pockets?".to_string(),
        QuestKind::CatchBug { .. } => "Hand over a bug from your pockets?".to_string(),
        QuestKind::BuyFurniture { item, .. } => format!("Give the {item}?"),
    };
    format!("{what}\n\nY: Yes    N: Not yet")
}

fn hud_text(active: &ActiveQuest, today: i64) -> String {
    let task = match &active.kind {
        QuestKind::Delivery { recipient, .. } => format!("Bring parcel to {recipient}"),
        QuestKind::ErrandFruit { giver, .. } => format!("Bring a fruit to {giver}"),
        QuestKind::CatchFish { giver } => format!("Bring a fish to {giver}"),
        QuestKind::CatchBug { giver } => format!("Bring a bug to {giver}"),
        QuestKind::BuyFurniture { giver, item } => format!("Get a {item} for {giver}"),
    };
    match days_left(active, today) {
        Some(0) => format!("Quest: {task} (due today)"),
        Some(1) => format!("Quest: {task} (1 day left)"),
        Some(n) => format!("Quest: {task} ({n} days left)"),
        None => format!("Quest: {task}"),
    }
}

/// Is this talk a valid turn-in for the active quest?
fn turn_in_ready(active: &ActiveQuest, villager_name: &str, inventory: &PlayerInventory) -> bool {
    let (_, qk) = type_kind(&active.kind);
    match &active.kind {
        QuestKind::Delivery { recipient, .. } => {
            villager_name == recipient && inventory.count(items::PARCEL) > 0
        }
        QuestKind::BuyFurniture { giver, item } => {
            villager_name == giver
                && inventory
                    .pockets
                    .pockets
                    .iter()
                    .any(|&id| id != 0 && items::name(id) == *item)
        }
        _ => {
            // Contests: the decomp completion rule.
            let qual = quest::ContestQual {
                item_possessed: inventory.count(items::FRUIT) > 0,
                player_id_absent: true,
                owns_category_item: match qk {
                    ckind::FISH => inventory.count_where(items::is_fish) > 0,
                    ckind::INSECT => inventory.count_where(items::is_bug) > 0,
                    _ => false,
                },
                ..quest::ContestQual::none()
            };
            villager_name == giver_of(&active.kind) && quest::contest_complete(qk, 1, &qual)
        }
    }
}

/// Roll the reward from the decomp tables (`prob_table_select`).
fn roll_reward(active: &ActiveQuest, rng: &mut dyn FnMut() -> u64) -> Reward {
    let (qt, qk) = type_kind(&active.kind);
    let (table, pay) = match qt {
        qtype::DELIVERY => (
            &quest::DELIVERY_REWARDS[qk as usize],
            quest::DELIVERY_MAX_PAY[qk as usize],
        ),
        qtype::ERRAND => {
            let tier = quest::errand_tier(active.used_num);
            (&quest::ERRAND_REWARDS[tier], quest::ERRAND_PAY[tier])
        }
        _ => (
            &quest::CONTEST_REWARDS[qk as usize],
            quest::CONTEST_MAX_PAY[qk as usize],
        ),
    };
    let category = quest::prob_table_select(table, (rng() % 100) as u8);
    match category {
        reward::MONEY => {
            let dir = (rng() % 2) as u8;
            let frac = (rng() % 1000) as f32 / 1000.0;
            Reward::Bells(quest::scaled_pay(pay, dir, frac, 0))
        }
        c => {
            use crate::goods::Kind;
            let kind = match c {
                reward::FTR => Some(Kind::Furniture),
                reward::STATIONERY => Some(Kind::Paper),
                reward::CLOTH => Some(Kind::Cloth),
                reward::CARPET => Some(Kind::Carpet),
                reward::WALLPAPER => Some(Kind::Wallpaper),
                _ => None,
            };
            let item = match kind {
                Some(k) => crate::goods::random(k, &mut |n| (rng() % u64::from(n.max(1))) as u32),
                None => items::reward_good(c).unwrap_or(items::FRUIT),
            };
            Reward::Item(item)
        }
    }
}

/// Hand a reward over; `false` if it doesn't fit (it's then owed).
fn give_reward(reward: Reward, inventory: &mut PlayerInventory) -> bool {
    match reward {
        Reward::Bells(n) => {
            inventory.bells += n;
            true
        }
        Reward::Item(id) => inventory.add(id),
    }
}

fn reward_name(reward: Reward) -> String {
    match reward {
        Reward::Bells(n) => format!("{n} bells"),
        Reward::Item(id) => format!("a {}", items::name(id)),
    }
}

/// Change a villager's friendship by name.
fn adjust_friendship(talks: &mut Query<(&Villager, &mut VillagerTalk)>, name: &str, delta: i32) {
    for (v, mut t) in talks.iter_mut() {
        if v.name == name {
            t.friendship += delta;
            println!("{name}: friendship {delta:+} -> {}", t.friendship);
        }
    }
}

/// Build the UI: hidden quest panel, quest HUD line, notification text.
fn setup_quest_ui(mut commands: Commands) {
    commands
        .spawn((
            QuestPanel,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(25.0),
                right: Val::Percent(25.0),
                top: Val::Percent(30.0),
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.07, 0.03, 0.94)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                QuestPanelText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.95, 0.75)),
            ));
        });

    commands.spawn((
        QuestHudText,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.4)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(12.0),
            ..default()
        },
        Visibility::Hidden,
    ));

    commands.spawn((
        QuestNotifyText,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(22.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(70.0),
            left: Val::Percent(30.0),
            right: Val::Percent(30.0),
            ..default()
        },
        Visibility::Hidden,
    ));
    println!("Quest UI ready");
}

fn show_panel(
    text: &str,
    panel: &mut Query<&mut Visibility, (With<QuestPanel>, Without<QuestNotifyText>)>,
    body: &mut Query<&mut Text, (With<QuestPanelText>, Without<QuestNotifyText>)>,
) {
    if let Ok(mut vis) = panel.single_mut() {
        *vis = Visibility::Visible;
    }
    if let Ok(mut t) = body.single_mut() {
        t.0 = text.to_string();
    }
}

fn hide_panel(panel: &mut Query<&mut Visibility, (With<QuestPanel>, Without<QuestNotifyText>)>) {
    if let Ok(mut vis) = panel.single_mut() {
        *vis = Visibility::Hidden;
    }
}

fn notify(
    msg: &str,
    timer: &mut ResMut<QuestNotifyTimer>,
    query: &mut Query<(&mut Visibility, &mut Text), (With<QuestNotifyText>, Without<QuestPanel>)>,
) {
    if let Ok((mut vis, mut text)) = query.single_mut() {
        text.0 = msg.to_string();
        *vis = Visibility::Visible;
    }
    timer.0 = NOTIFY_DURATION;
}

fn tick_notify(
    time: Res<Time>,
    mut timer: ResMut<QuestNotifyTimer>,
    mut query: Query<&mut Visibility, With<QuestNotifyText>>,
) {
    if timer.0 <= 0.0 {
        return;
    }
    timer.0 -= time.delta().as_secs_f32();
    if timer.0 <= 0.0 {
        if let Ok(mut vis) = query.single_mut() {
            *vis = Visibility::Hidden;
        }
    }
}

/// Quest timeouts, then cooldown ticking + picking a new quest giver.
#[allow(clippy::too_many_arguments)]
fn quest_cooldown_tick(
    time: Res<Time>,
    mut commands: Commands,
    mut quest: ResMut<QuestState>,
    mut giver: ResMut<QuestGiver>,
    mut inventory: ResMut<PlayerInventory>,
    house_state: Res<State<HouseState>>,
    shop_loc: Res<State<crate::shop::ShopLocation>>,
    shop_data: Res<crate::shop::ShopData>,
    // Only villagers out in town can offer a quest.
    villagers: Query<(Entity, &Villager), Without<crate::npc::Indoors>>,
    mut talks: Query<(&Villager, &mut VillagerTalk)>,
    mut timer: ResMut<QuestNotifyTimer>,
    mut notify_query: Query<
        (&mut Visibility, &mut Text),
        (With<QuestNotifyText>, Without<QuestPanel>),
    >,
) {
    let now = crate::clock::now();
    // Time limit (`quest::limit_days`): past it, the request lapses.
    if let Some(active) = quest.active.as_ref() {
        if days_left(active, now.day_number).is_some_and(|d| d < 0) {
            let who = giver_of(&active.kind).to_string();
            if matches!(active.kind, QuestKind::Delivery { .. }) {
                inventory.remove(items::PARCEL);
            }
            quest.active = None;
            quest.cooldown = DECLINE_COOLDOWN_SECS;
            adjust_friendship(&mut talks, &who, quest::friendship::FAIL_MSG_0X2B8);
            notify(
                &format!("You ran out of time for {who}'s request."),
                &mut timer,
                &mut notify_query,
            );
            println!("Quest for {who} expired");
        }
    }
    if quest.active.is_some() || giver.0.is_some() {
        return;
    }
    if *house_state != HouseState::Town || *shop_loc != crate::shop::ShopLocation::Town {
        return;
    }
    quest.cooldown -= time.delta().as_secs_f32();
    if quest.cooldown > 0.0 {
        return;
    }
    // Retail gates quest attempts on a roll; 3-in-4 passes.
    let attempt = (quest.rng_next() % 256) as u8;
    if !quest::quest_attempt_roll(attempt) {
        quest.cooldown = RETRY_COOLDOWN_SECS;
        return;
    }
    let list: Vec<(Entity, String)> = villagers
        .iter()
        .map(|(e, v)| (e, v.name.to_string()))
        .collect();
    if list.is_empty() {
        quest.cooldown = RETRY_COOLDOWN_SECS;
        return;
    }
    let (entity, name) = list[(quest.rng_next() as usize) % list.len()].clone();
    let all_names: Vec<String> = talks.iter().map(|(v, _)| v.name.to_string()).collect();
    let stock_names: Vec<String> = shop_data.stock.iter().map(|s| s.name.clone()).collect();
    let rolls = [
        quest.rng_next() as u32,
        quest.rng_next() as u32,
        quest.rng_next() as u32,
    ];
    let Some((kind, limit_days)) = generate_quest(rolls, &name, &all_names, &stock_names, &now)
    else {
        // Failed occur check / kind not in the slice: try again later.
        quest.cooldown = RETRY_COOLDOWN_SECS;
        return;
    };
    giver.0 = Some(PendingOffer {
        giver: name.clone(),
        kind,
        limit_days,
    });
    // Floating "!" above the giver's head.
    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            QuestMarker,
            Text2d::new("!"),
            TextFont {
                font_size: FontSize::Px(64.0),
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.85, 0.1)),
            Transform::from_translation(Vec3::new(0.0, 1.7, 0.0)),
        ));
    });
    println!("{name} has a quest (marker placed)");
}

/// Bob the `!` markers.
fn marker_bob(time: Res<Time>, mut markers: Query<&mut Transform, With<QuestMarker>>) {
    let bob = (time.elapsed().as_secs_f32() * 3.0).sin() * 0.15;
    for mut t in markers.iter_mut() {
        t.translation.y = 1.7 + bob;
    }
}

/// Intercept talks: quest offers and turn-ins claim the TalkEvent so the
/// normal dialogue box stays shut (see `QuestClaim`).
#[allow(clippy::too_many_arguments)]
fn quest_talk_intercept(
    mut events: MessageReader<TalkEvent>,
    villagers: Query<&Villager>,
    mut quest: ResMut<QuestState>,
    giver: Res<QuestGiver>,
    house_state: Res<State<HouseState>>,
    shop_loc: Res<State<crate::shop::ShopLocation>>,
    dialogue: Res<DialogueState>,
    mut claim: ResMut<QuestClaim>,
    mut qui: ResMut<QuestUi>,
    mut inventory: ResMut<PlayerInventory>,
    mut panel: Query<&mut Visibility, (With<QuestPanel>, Without<QuestNotifyText>)>,
    mut body: Query<&mut Text, (With<QuestPanelText>, Without<QuestNotifyText>)>,
    mut timer: ResMut<QuestNotifyTimer>,
    mut notify_query: Query<
        (&mut Visibility, &mut Text),
        (With<QuestNotifyText>, Without<QuestPanel>),
    >,
) {
    if *house_state != HouseState::Town || *shop_loc != crate::shop::ShopLocation::Town {
        return;
    }
    if qui.open || dialogue.active {
        return;
    }
    for event in events.read() {
        let Ok(villager) = villagers.get(event.villager) else {
            continue;
        };
        // A reward they still owe you (it didn't fit last time)?
        let owed = quest
            .owed
            .as_ref()
            .filter(|(who, _)| who == villager.name)
            .map(|(_, r)| *r);
        if let Some(owed) = owed {
            {
                if give_reward(owed, &mut inventory) {
                    quest.owed = None;
                    if matches!(owed, Reward::Item(_)) {
                        crate::sound::se(crate::sound::se_id::POCKET_RUSTLE);
                    }
                    notify(
                        &format!(
                            "{} gave you the reward they owed: {}!",
                            villager.name,
                            reward_name(owed)
                        ),
                        &mut timer,
                        &mut notify_query,
                    );
                    println!("{} handed over an owed reward", villager.name);
                }
            }
        }
        // Quest offer?
        if let Some(offer) = giver.0.as_ref() {
            if offer.giver == villager.name {
                claim.0 = Some(event.villager);
                qui.open = true;
                crate::sound::se(crate::sound::se_id::CHOICE_OPEN);
                qui.mode = QuestUiMode::Offer;
                show_panel(&offer_text(offer), &mut panel, &mut body);
                println!("Quest offered by {}", offer.giver);
                continue;
            }
        }
        // Turn-in?
        if let Some(active) = quest.active.as_ref() {
            if turn_in_ready(active, villager.name, &inventory) {
                claim.0 = Some(event.villager);
                qui.open = true;
                qui.mode = QuestUiMode::TurnIn;
                show_panel(&turnin_text(active), &mut panel, &mut body);
                println!("Quest turn-in with {}", villager.name);
            }
        }
    }
}

/// Y/N (or Esc) handling for the quest panel.
#[allow(clippy::too_many_arguments)]
fn quest_ui_input(
    actions: Res<Actions>,
    mut commands: Commands,
    mut ui: ResMut<QuestUi>,
    mut quest: ResMut<QuestState>,
    mut giver: ResMut<QuestGiver>,
    mut claim: ResMut<QuestClaim>,
    mut inventory: ResMut<PlayerInventory>,
    mut mailbox: ResMut<crate::mail::Mailbox>,
    mut talks: Query<(&Villager, &mut VillagerTalk)>,
    markers: Query<Entity, With<QuestMarker>>,
    mut panel: Query<&mut Visibility, (With<QuestPanel>, Without<QuestNotifyText>)>,
    mut timer: ResMut<QuestNotifyTimer>,
    mut notify_query: Query<
        (&mut Visibility, &mut Text),
        (With<QuestNotifyText>, Without<QuestPanel>),
    >,
) {
    if !ui.open {
        return;
    }
    let accepted = actions.just_pressed(Action::Yes);
    let declined = actions.just_pressed(Action::No) || actions.just_pressed(Action::Cancel);
    if !accepted && !declined {
        return;
    }
    // These modal prompts use the existing dialogue choice feedback.
    crate::sound::se(if accepted {
        crate::sound::se_id::CHOICE_CONFIRM
    } else {
        crate::sound::se_id::MENU_EXIT
    });
    let close =
        |ui: &mut ResMut<QuestUi>,
         claim: &mut ResMut<QuestClaim>,
         panel: &mut Query<&mut Visibility, (With<QuestPanel>, Without<QuestNotifyText>)>| {
            ui.open = false;
            claim.0 = None;
            hide_panel(panel);
        };
    match ui.mode {
        QuestUiMode::Offer => {
            if accepted {
                let is_delivery = giver
                    .0
                    .as_ref()
                    .is_some_and(|o| matches!(o.kind, QuestKind::Delivery { .. }));
                // A delivery hands you the parcel, which needs a free pocket.
                if is_delivery && !inventory.add(items::PARCEL) {
                    notify(
                        "Your pockets are full! Make room for the parcel first.",
                        &mut timer,
                        &mut notify_query,
                    );
                    close(&mut ui, &mut claim, &mut panel);
                    return;
                }
                if let Some(offer) = giver.0.take() {
                    if is_delivery {
                        crate::sound::se(crate::sound::se_id::POCKET_RUSTLE);
                    }
                    quest.active = Some(ActiveQuest {
                        kind: offer.kind,
                        accepted_day: crate::clock::now().day_number,
                        limit_days: offer.limit_days,
                        used_num: (quest.completed + 1).min(u8::MAX as u32) as u8,
                    });
                    for e in markers.iter() {
                        commands.entity(e).despawn();
                    }
                    notify(
                        "Quest accepted! Check the HUD for details.",
                        &mut timer,
                        &mut notify_query,
                    );
                    println!("Quest accepted");
                }
            } else {
                if let Some(offer) = giver.0.take() {
                    adjust_friendship(&mut talks, &offer.giver, quest::friendship::REJECT_QUEST);
                }
                for e in markers.iter() {
                    commands.entity(e).despawn();
                }
                quest.cooldown = DECLINE_COOLDOWN_SECS;
                println!("Quest declined");
            }
            close(&mut ui, &mut claim, &mut panel);
        }
        QuestUiMode::TurnIn => {
            if accepted {
                if let Some(active) = quest.active.take() {
                    // Give/receive-putaway use GASAGOSO; no invented quest fanfare.
                    crate::sound::se(crate::sound::se_id::POCKET_RUSTLE);
                    // Hand over what was asked for.
                    match &active.kind {
                        QuestKind::Delivery { .. } => {
                            inventory.remove(items::PARCEL);
                        }
                        QuestKind::ErrandFruit { amount, .. } => {
                            for _ in 0..(*amount).max(1) {
                                inventory.remove(items::FRUIT);
                            }
                        }
                        QuestKind::CatchFish { .. } => {
                            inventory.remove_first_where(items::is_fish);
                        }
                        QuestKind::CatchBug { .. } => {
                            inventory.remove_first_where(items::is_bug);
                        }
                        QuestKind::BuyFurniture { item, .. } => {
                            if let Some(&id) = inventory
                                .pockets
                                .pockets
                                .iter()
                                .find(|&&id| id != 0 && items::name(id) == *item)
                            {
                                inventory.remove(id);
                            }
                        }
                    }
                    let giver_name = giver_of(&active.kind).to_string();
                    let reward = roll_reward(&active, &mut || quest.rng_next());
                    let msg = if give_reward(reward, &mut inventory) {
                        format!("Quest complete! You got {}.", reward_name(reward))
                    } else {
                        // Retail give_reward: keep it for next time.
                        quest.owed = Some((giver_name.clone(), reward));
                        format!(
                            "Quest complete! Your pockets are full, so {giver_name} will give you {} later.",
                            reward_name(reward)
                        )
                    };
                    adjust_friendship(&mut talks, &giver_name, quest::friendship::NORMAL_REWARD);
                    quest.completed += 1;
                    quest.cooldown = QUEST_COOLDOWN_SECS;
                    notify(&msg, &mut timer, &mut notify_query);
                    // Thank-you letter in the mailbox.
                    mailbox.letters.push(crate::mail::Letter {
                        sender: giver_name.clone(),
                        subject: "Thank you!".to_string(),
                        body: format!(
                            "Thanks for helping me out! You're the best!\n\n- {giver_name}"
                        ),
                        read: false,
                        paper: 0,
                    });
                    println!("Quest complete: reward {reward:?}");
                }
            }
            close(&mut ui, &mut claim, &mut panel);
        }
    }
}

/// Keep the quest HUD line in sync.
fn update_quest_hud(
    quest: Res<QuestState>,
    mut text_query: Query<(&mut Visibility, &mut Text), With<QuestHudText>>,
) {
    if let Ok((mut vis, mut text)) = text_query.single_mut() {
        match quest.active.as_ref() {
            Some(active) => {
                let label = hud_text(active, crate::clock::now().day_number);
                if text.0 != label {
                    text.0 = label;
                }
                *vis = Visibility::Visible;
            }
            None => {
                *vis = Visibility::Hidden;
            }
        }
    }
}

pub struct QuestPlugin;

impl Plugin for QuestPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(QuestState::default())
            .insert_resource(QuestGiver::default())
            .insert_resource(QuestUi::default())
            .insert_resource(QuestClaim::default())
            .insert_resource(QuestNotifyTimer::default())
            .add_systems(Startup, setup_quest_ui)
            .add_systems(
                Update,
                quest_talk_intercept
                    .in_set(QuestInputSet)
                    .after(crate::interaction::interact)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                quest_ui_input.in_set(InteractSet::Ui).in_set(GameplaySet),
            )
            .add_systems(
                Update,
                (
                    quest_cooldown_tick,
                    marker_bob,
                    update_quest_hud,
                    tick_notify,
                )
                    .in_set(GameplaySet),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(month: u8, d: u8) -> crate::clock::Now {
        crate::clock::Now {
            month,
            day: d,
            weekday: 1,
            hour: 12,
            minute: 0,
            day_secs: 12 * 3600,
            day_number: 1000,
            season: rustimal_logic::game_time::Season::Summer,
        }
    }

    fn names() -> Vec<String> {
        vec!["Maple".into(), "Pippy".into(), "Bob".into()]
    }

    #[test]
    fn generation_only_yields_slice_kinds_and_respects_seasons() {
        let stock = vec!["Bookshelf".to_string()];
        let mut seen = std::collections::HashSet::new();
        let mut december_bug = false;
        for t in 0..3 {
            for k in 0..7 {
                let rolls = [t, k, 1];
                if let Some((kind, limit)) =
                    generate_quest(rolls, "Maple", &names(), &stock, &day(6, 1))
                {
                    let (qt, qk) = type_kind(&kind);
                    assert_eq!(
                        quest::limit_days(qt, qk, if qt == qtype::CONTEST { 1 } else { 0 })
                            .unwrap(),
                        limit
                    );
                    seen.insert((qt, qk));
                }
                if let Some((QuestKind::CatchBug { .. }, _)) =
                    generate_quest(rolls, "Maple", &names(), &stock, &day(12, 15))
                {
                    december_bug = true;
                }
            }
        }
        for (qt, qk) in &seen {
            assert!(matches!(
                (*qt, *qk),
                (qtype::DELIVERY, dkind::NORMAL)
                    | (qtype::ERRAND, ekind::REQUEST)
                    | (qtype::CONTEST, ckind::FRUIT | ckind::FISH | ckind::INSECT)
            ));
        }
        assert!(
            seen.contains(&(qtype::CONTEST, ckind::INSECT)),
            "June bug contests occur"
        );
        assert!(!december_bug, "occur_ok blocks bug contests in December");
    }

    #[test]
    fn delivery_rewards_follow_the_decomp_table() {
        let active = ActiveQuest {
            kind: QuestKind::Delivery {
                giver: "Maple".into(),
                recipient: "Bob".into(),
            },
            accepted_day: 0,
            limit_days: 2,
            used_num: 1,
        };
        let mut s = 0x1234_5678_9abc_def0u64;
        let mut rng = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let (mut ftr, mut money, mut worn) = (0, 0, 0);
        for _ in 0..3000 {
            match roll_reward(&active, &mut rng) {
                Reward::Bells(n) => {
                    money += 1;
                    assert!((180..=220).contains(&n), "scaled_pay of 200 +/-10%: {n}");
                }
                Reward::Item(id) if items::is_furniture(id) => ftr += 1,
                Reward::Item(id) => {
                    assert_eq!(Some(id), items::reward_good(reward::WORN_CLOTH));
                    worn += 1;
                }
            }
        }
        let pct = |n: i32| n as f64 / 30.0;
        assert!((pct(ftr) - 40.0).abs() < 4.0, "ftr {}", pct(ftr));
        assert!((pct(money) - 30.0).abs() < 4.0, "money {}", pct(money));
        assert!((pct(worn) - 30.0).abs() < 4.0, "worn {}", pct(worn));
    }

    #[test]
    fn contest_turn_in_uses_contest_complete() {
        let active = ActiveQuest {
            kind: QuestKind::CatchFish {
                giver: "Bob".into(),
            },
            accepted_day: 0,
            limit_days: 3,
            used_num: 1,
        };
        let mut inv = PlayerInventory::default();
        assert!(!turn_in_ready(&active, "Bob", &inv));
        inv.add(items::bug(
            rustimal_logic::species::InsectType::CommonButterfly,
        ));
        assert!(!turn_in_ready(&active, "Bob", &inv), "a bug isn't a fish");
        inv.add(items::fish(rustimal_logic::species::FishType::Carp));
        assert!(turn_in_ready(&active, "Bob", &inv));
        assert!(!turn_in_ready(&active, "Maple", &inv), "only the giver");
    }

    #[test]
    fn days_left_counts_down() {
        let active = ActiveQuest {
            kind: QuestKind::CatchBug {
                giver: "Bob".into(),
            },
            accepted_day: 100,
            limit_days: 2,
            used_num: 1,
        };
        assert_eq!(days_left(&active, 100), Some(2));
        assert_eq!(days_left(&active, 102), Some(0));
        assert_eq!(days_left(&active, 103), Some(-1));
    }
}
