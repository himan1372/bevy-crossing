//! Phase 3: quest/errand system.
//!
//! Quest plumbing comes from the decomp-verified logic crate:
//! - `quest_gen::select_quest_type` picks DELIVERY / ERRAND / CONTEST from
//!   the retail type tables (`QUEST_TYPE_TABLE_QST`)
//! - `quest::quest_attempt_roll` (75% pass) gates whether a villager gets a `!`
//! - `quest::pc_quest_base_pay` gives the retail bell pay per type/kind
//!   (delivery 200, errand tier-2 500; contest kinds pay 0 in bells in
//!   retail — item rewards — so catch quests use a flat 400 here)
//! The offer/turn-in UI, `!` markers, HUD line, and Bevy-side generation
//! (delivery recipient, fruit/furniture/catch variants) are new.

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy::text::FontSize;
use rustimal_logic::quest::{self, ckind, dkind, qtype};
use rustimal_logic::quest_gen;
use serde::{Deserialize, Serialize};

use crate::dialogue::DialogueState;
use crate::house::HouseState;
use crate::interaction::{PlayerInventory, TalkEvent};
use crate::npc::Villager;

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
    pub reward: u32,
    /// Fish/bug counts when accepted — turn-in needs a *new* catch.
    pub fish_baseline: u32,
    pub bugs_baseline: u32,
}

/// A villager with a `!` marker, holding a generated offer.
#[derive(Clone, Debug)]
pub struct PendingOffer {
    pub giver: String,
    pub kind: QuestKind,
    pub reward: u32,
}

/// Quest runtime state.
#[derive(Resource)]
pub struct QuestState {
    pub active: Option<ActiveQuest>,
    cooldown: f32,
    pub completed: u32,
    rng: u64,
}

impl Default for QuestState {
    fn default() -> Self {
        Self {
            active: None,
            // First quest shows up quickly so playtesters see the system.
            cooldown: 20.0,
            completed: 0,
            rng: 0x51ED_752B_9C3A_11A7,
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

/// Build a quest from a type-table roll. `others` are villager names to pick
/// a delivery recipient from; `stock` are Nook's shelf names.
fn generate_quest(roll: u32, giver: &str, others: &[String], stock: &[String]) -> (QuestKind, u32) {
    match quest_gen::select_quest_type(false, roll) {
        t if t == qtype::DELIVERY => {
            let candidates: Vec<&String> = others.iter().filter(|n| *n != giver).collect();
            let recipient = candidates
                .get((roll as usize) % candidates.len().max(1))
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Bob".to_string());
            let reward = quest::pc_quest_base_pay(qtype::DELIVERY, dkind::NORMAL, 1).max(200);
            (
                QuestKind::Delivery {
                    giver: giver.to_string(),
                    recipient,
                },
                reward,
            )
        }
        t if t == qtype::ERRAND => {
            if roll % 2 == 0 {
                // used_num=2 -> ERRAND_PAY tier 2 = 500 bells.
                let reward = quest::pc_quest_base_pay(qtype::ERRAND, 0, 2).max(300);
                (
                    QuestKind::ErrandFruit {
                        giver: giver.to_string(),
                        amount: 3,
                    },
                    reward,
                )
            } else {
                let item = stock
                    .get((roll as usize) % stock.len().max(1))
                    .cloned()
                    .unwrap_or_else(|| "Wooden Chair".to_string());
                let reward = quest::pc_quest_base_pay(qtype::ERRAND, 0, 3).max(300);
                (
                    QuestKind::BuyFurniture {
                        giver: giver.to_string(),
                        item,
                    },
                    reward,
                )
            }
        }
        // CONTEST -> catch quests. Retail contest bell pay is 0 (item
        // rewards instead), so catch quests pay a flat 400 here.
        _ => {
            if roll % 2 == 0 {
                let reward = quest::pc_quest_base_pay(qtype::CONTEST, ckind::FISH, 0).max(400);
                (
                    QuestKind::CatchFish {
                        giver: giver.to_string(),
                    },
                    reward,
                )
            } else {
                let reward = quest::pc_quest_base_pay(qtype::CONTEST, ckind::INSECT, 0).max(400);
                (
                    QuestKind::CatchBug {
                        giver: giver.to_string(),
                    },
                    reward,
                )
            }
        }
    }
}

fn offer_text(offer: &PendingOffer) -> String {
    let ask = match &offer.kind {
        QuestKind::Delivery { recipient, .. } => {
            format!("Could you take this parcel to {recipient} for me?")
        }
        QuestKind::ErrandFruit { amount, .. } => {
            format!("Could you bring me {amount} fruit? I need them for a recipe!")
        }
        QuestKind::CatchFish { .. } => {
            "I'd love a fresh fish! Could you catch one for me?".to_string()
        }
        QuestKind::CatchBug { .. } => {
            "Have you seen the bugs around here? Catch one for me!".to_string()
        }
        QuestKind::BuyFurniture { item, .. } => {
            format!("I have my eye on a {item} at Nook's. Could you buy one for me?")
        }
    };
    format!(
        "{}: {}\nReward: {} bells\n\nY: Accept    N: Decline",
        offer.giver, ask, offer.reward
    )
}

fn turnin_text(active: &ActiveQuest) -> String {
    let what = match &active.kind {
        QuestKind::Delivery { recipient, .. } => format!("Deliver the parcel to {recipient}?"),
        QuestKind::ErrandFruit { amount, .. } => format!("Hand over {amount} fruit?"),
        QuestKind::CatchFish { .. } => "Show off your catch?".to_string(),
        QuestKind::CatchBug { .. } => "Show off your catch?".to_string(),
        QuestKind::BuyFurniture { item, .. } => format!("Give the {item}?"),
    };
    format!(
        "{what}\nReward: {} bells\n\nY: Yes    N: Not yet",
        active.reward
    )
}

fn hud_text(active: &ActiveQuest) -> String {
    let task = match &active.kind {
        QuestKind::Delivery { recipient, .. } => format!("Bring parcel to {recipient}"),
        QuestKind::ErrandFruit { giver, amount } => format!("Bring {amount} fruit to {giver}"),
        QuestKind::CatchFish { giver } => format!("Catch a fish for {giver}"),
        QuestKind::CatchBug { giver } => format!("Catch a bug for {giver}"),
        QuestKind::BuyFurniture { giver, item } => format!("Buy {item} for {giver}"),
    };
    format!("Quest: {task}")
}

/// Is this talk a valid turn-in for the active quest?
fn turn_in_ready(active: &ActiveQuest, villager_name: &str, inventory: &PlayerInventory) -> bool {
    match &active.kind {
        QuestKind::Delivery { recipient, .. } => villager_name == recipient,
        QuestKind::ErrandFruit { giver, amount } => {
            villager_name == giver && inventory.fruit >= *amount
        }
        QuestKind::CatchFish { giver } => {
            villager_name == giver && inventory.fish > active.fish_baseline
        }
        QuestKind::CatchBug { giver } => {
            villager_name == giver && inventory.bugs > active.bugs_baseline
        }
        QuestKind::BuyFurniture { giver, item } => {
            villager_name == giver && inventory.furniture.iter().any(|f| f == item)
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
    panel: &mut Query<&mut Visibility, With<QuestPanel>>,
    body: &mut Query<&mut Text, With<QuestPanelText>>,
) {
    if let Ok(mut vis) = panel.single_mut() {
        *vis = Visibility::Visible;
    }
    if let Ok(mut t) = body.single_mut() {
        t.0 = text.to_string();
    }
}

fn hide_panel(panel: &mut Query<&mut Visibility, With<QuestPanel>>) {
    if let Ok(mut vis) = panel.single_mut() {
        *vis = Visibility::Hidden;
    }
}

fn notify(
    msg: &str,
    timer: &mut ResMut<QuestNotifyTimer>,
    query: &mut Query<(&mut Visibility, &mut Text), With<QuestNotifyText>>,
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

/// Cooldown ticking + picking a new quest giver with a `!` marker.
fn quest_cooldown_tick(
    time: Res<Time>,
    mut commands: Commands,
    mut quest: ResMut<QuestState>,
    mut giver: ResMut<QuestGiver>,
    house_state: Res<State<HouseState>>,
    shop_loc: Res<State<crate::shop::ShopLocation>>,
    shop_data: Res<crate::shop::ShopData>,
    villagers: Query<(Entity, &Villager)>,
) {
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
        quest.cooldown = 60.0;
        return;
    }
    let list: Vec<(Entity, String)> = villagers
        .iter()
        .map(|(e, v)| (e, v.name.to_string()))
        .collect();
    if list.is_empty() {
        quest.cooldown = 60.0;
        return;
    }
    let (entity, name) = list[(quest.rng_next() as usize) % list.len()].clone();
    let all_names: Vec<String> = list.iter().map(|(_, n)| n.clone()).collect();
    let stock_names: Vec<String> = shop_data.stock.iter().map(|s| s.name.clone()).collect();
    let roll = (quest.rng_next() % u32::MAX as u64) as u32;
    let (kind, reward) = generate_quest(roll, &name, &all_names, &stock_names);
    giver.0 = Some(PendingOffer {
        giver: name.clone(),
        kind,
        reward,
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
    quest: Res<QuestState>,
    giver: Res<QuestGiver>,
    house_state: Res<State<HouseState>>,
    shop_loc: Res<State<crate::shop::ShopLocation>>,
    dialogue: Res<DialogueState>,
    mut claim: ResMut<QuestClaim>,
    mut qui: ResMut<QuestUi>,
    inventory: Res<PlayerInventory>,
    mut panel: Query<&mut Visibility, With<QuestPanel>>,
    mut body: Query<&mut Text, With<QuestPanelText>>,
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
        // Quest offer?
        if let Some(offer) = giver.0.as_ref() {
            if offer.giver == villager.name {
                claim.0 = Some(event.villager);
                qui.open = true;
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
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut ui: ResMut<QuestUi>,
    mut quest: ResMut<QuestState>,
    mut giver: ResMut<QuestGiver>,
    mut claim: ResMut<QuestClaim>,
    mut inventory: ResMut<PlayerInventory>,
    mut mailbox: ResMut<crate::mail::Mailbox>,
    markers: Query<Entity, With<QuestMarker>>,
    mut panel: Query<&mut Visibility, With<QuestPanel>>,
    mut timer: ResMut<QuestNotifyTimer>,
    mut notify_query: Query<(&mut Visibility, &mut Text), With<QuestNotifyText>>,
) {
    if !ui.open {
        return;
    }
    let accepted = keyboard.just_pressed(KeyCode::KeyY);
    let declined = keyboard.just_pressed(KeyCode::KeyN) || keyboard.just_pressed(KeyCode::Escape);
    if !accepted && !declined {
        return;
    }
    let close = |ui: &mut ResMut<QuestUi>,
                 claim: &mut ResMut<QuestClaim>,
                 panel: &mut Query<&mut Visibility, With<QuestPanel>>| {
        ui.open = false;
        claim.0 = None;
        hide_panel(panel);
    };
    match ui.mode {
        QuestUiMode::Offer => {
            if accepted {
                if let Some(offer) = giver.0.take() {
                    quest.active = Some(ActiveQuest {
                        kind: offer.kind,
                        reward: offer.reward,
                        fish_baseline: inventory.fish,
                        bugs_baseline: inventory.bugs,
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
                giver.0 = None;
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
                    // Apply per-kind handovers.
                    match &active.kind {
                        QuestKind::ErrandFruit { amount, .. } => {
                            inventory.fruit = inventory.fruit.saturating_sub(*amount);
                        }
                        QuestKind::BuyFurniture { item, .. } => {
                            if let Some(pos) = inventory.furniture.iter().position(|f| f == item) {
                                inventory.furniture.remove(pos);
                            }
                        }
                        _ => {}
                    }
                    inventory.bells += active.reward;
                    quest.completed += 1;
                    quest.cooldown = QUEST_COOLDOWN_SECS;
                    let giver_name = match &active.kind {
                        QuestKind::Delivery { giver, .. } => giver.clone(),
                        QuestKind::ErrandFruit { giver, .. } => giver.clone(),
                        QuestKind::CatchFish { giver } => giver.clone(),
                        QuestKind::CatchBug { giver } => giver.clone(),
                        QuestKind::BuyFurniture { giver, .. } => giver.clone(),
                    };
                    notify(
                        &format!("Quest complete! +{} bells", active.reward),
                        &mut timer,
                        &mut notify_query,
                    );
                    // Thank-you letter in the mailbox.
                    mailbox.letters.push(crate::mail::Letter {
                        sender: giver_name.clone(),
                        subject: "Thank you!".to_string(),
                        body: format!(
                            "Thanks for helping me out! You're the best!\n\n- {giver_name}"
                        ),
                        read: false,
                    });
                    println!("Quest complete (+{} bells)", active.reward);
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
    if !quest.is_changed() {
        return;
    }
    if let Ok((mut vis, mut text)) = text_query.single_mut() {
        match quest.active.as_ref() {
            Some(active) => {
                text.0 = hud_text(active);
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
                    .after(crate::interaction::interact),
            )
            .add_systems(
                Update,
                (
                    quest_cooldown_tick,
                    marker_bob,
                    quest_ui_input,
                    update_quest_hud,
                    tick_notify,
                ),
            );
    }
}
