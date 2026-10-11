//! Dialogue: villager conversations and the bottom text box.
//!
//! What a villager says is chosen by the translated GameCube topic tree
//! (see [`crate::talk`]); how long they'll keep chatting follows the
//! translated talk-patience rules in `rustimal_logic::interaction`
//! (`mNpc_CountTalkNum` / `mNpc_GetOverImpatient`, temper table indexed by
//! personality — a retail quirk the port keeps). All text is original.
//!
//! Ours (not traced in the decomp): how moods change. A villager is Happy
//! about a quarter of the time (re-rolled when their patience resets) and
//! always after you finish their quest.

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy::text::FontSize;
use rustimal_logic::interaction::{Patience, TalkInfo, TEMPER_TABLE};
use rustimal_logic::quest::reward;
use rustimal_logic::request_selector;

use crate::input::{Action, Actions, InteractSet};
use crate::interaction::{PlayerInventory, TalkEvent};
use crate::items;
use crate::npc::Villager;
use crate::states::GameplaySet;
use crate::talk::{self, TalkContext, Topic, TradeKind};

/// Dialogue box state.
#[derive(Resource, Default)]
pub struct DialogueState {
    /// Whether the box is currently shown.
    pub active: bool,
    lines: Vec<String>,
    index: usize,
    /// A trade offer waiting on Y/N at the last line.
    trade: Option<TradeDeal>,
}

/// A villager offering to swap one of their goods for something you carry
/// (`request_selector`: the wanted item is picked uniformly among eligible
/// carried items, the offer comes from `build_trade_offer`).
struct TradeDeal {
    villager: &'static str,
    wanted: items::ItemId,
    offered: items::ItemId,
}

/// Marker for the dialogue box node.
#[derive(Component)]
struct DialogueBox;

/// Marker for the dialogue text.
#[derive(Component)]
struct DialogueText;

/// Marker for the inventory HUD text.
#[derive(Component)]
struct HudText;

/// The GameCube logic rate; talk timers count retail frames.
const RETAIL_FPS: f32 = 30.0;
/// `mNpc_TalkEndMove` sets the talk timer to this many frames.
const TALK_END_TIMER: u16 = 1000;
/// Chance a villager is in a Happy mood (re-rolled when patience resets).
const HAPPY_CHANCE: u32 = 25;

/// A villager's conversation state.
#[derive(Component)]
pub struct VillagerTalk {
    info: TalkInfo,
    pub happy: bool,
    /// Friendship with the player (quest::friendship deltas).
    pub friendship: i32,
    rolled: bool,
    /// Fractional retail frames carried between updates.
    frame_acc: f32,
}

impl Default for VillagerTalk {
    fn default() -> Self {
        Self {
            info: TalkInfo::new(),
            happy: false,
            friendship: 0,
            rolled: false,
            frame_acc: 0.0,
        }
    }
}

/// xorshift for topic rolls (same pattern as the other plugins).
#[derive(Resource)]
struct TalkRng(u64);

impl TalkRng {
    /// 0..n
    fn below(&mut self, n: u32) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        ((x >> 32) as u32) % n.max(1)
    }
}

/// Build the UI: hidden dialogue box + inventory HUD.
fn setup_ui(mut commands: Commands) {
    // Dialogue box — bottom of screen, hidden until a talk starts.
    commands
        .spawn((
            DialogueBox,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(24.0),
                left: Val::Px(24.0),
                right: Val::Px(24.0),
                padding: UiRect::all(Val::Px(18.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.88)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                DialogueText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

    // Inventory HUD — top-left corner.
    commands.spawn((
        HudText,
        Text::new("Bells: 0   Fruit: 0"),
        TextFont {
            font_size: FontSize::Px(20.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
    println!("Dialogue UI ready");
}

/// Start a conversation when a TalkEvent arrives.
///
/// Talks the quest system claimed (offers / turn-ins) are skipped here —
/// the quest plugin shows its own panel for those.
#[allow(clippy::too_many_arguments)]
fn on_talk(
    mut events: MessageReader<TalkEvent>,
    mut villagers: Query<(&Villager, &mut VillagerTalk)>,
    mut dialogue: ResMut<DialogueState>,
    mut rng: ResMut<TalkRng>,
    mut inventory: ResMut<PlayerInventory>,
    mailbox: Res<crate::mail::Mailbox>,
    quest_claim: Res<crate::quest::QuestClaim>,
    mut box_query: Query<&mut Visibility, With<DialogueBox>>,
    mut text_query: Query<&mut Text, With<DialogueText>>,
) {
    for event in events.read() {
        if quest_claim.0 == Some(event.villager) {
            continue;
        }
        let Ok((villager, mut chat)) = villagers.get_mut(event.villager) else {
            continue;
        };
        let looks = villager.looks as usize;
        if !chat.rolled {
            chat.happy = rng.below(100) < HAPPY_CHANCE;
            chat.rolled = true;
        }
        let say = |line: &str| format!("{}: {line}", villager.name);

        let patience = chat.info.patience(looks);
        let mut lines = Vec::new();
        dialogue.trade = None;
        if patience == Patience::Annoyed {
            lines.push(say(talk::annoyed(looks)));
            println!("{} is too annoyed to talk", villager.name);
        } else {
            let now = crate::clock::now();
            let ctx = TalkContext {
                looks,
                happy: chat.happy,
                free_pocket: inventory.pockets.find_free_slot().is_some(),
                bells: inventory.bells,
                has_letter_from_them: mailbox.letters.iter().any(|l| l.sender == villager.name),
                carried_catch: inventory
                    .pockets
                    .pockets
                    .iter()
                    .copied()
                    .find(|&it| items::is_fish(it) || items::is_bug(it)),
                carried_furniture: inventory
                    .pockets
                    .pockets
                    .iter()
                    .copied()
                    .find(|&it| items::is_furniture(it)),
                month: now.month,
                season: now.season,
                hour: now.hour,
            };
            let (topic, msg_no) = talk::choose(&ctx, &mut |n| rng.below(n));
            // The topic's subject: the gift handed over, or the item admired.
            let item = match topic {
                Topic::Gift => {
                    let gift = if rng.below(2) == 0 {
                        items::FRUIT
                    } else {
                        let n = items::FURNITURE_CATALOG.len() as u32;
                        items::furniture(items::FURNITURE_CATALOG[rng.below(n) as usize])
                            .unwrap_or(items::FRUIT)
                    };
                    // The topic is only chosen with a free pocket.
                    inventory.add(gift);
                    Some(items::name(gift))
                }
                Topic::Trade(kind) => {
                    let deal = trade_offer(kind, &inventory, &mut |n| rng.below(n));
                    let name = deal.as_ref().map(|d| items::name(d.wanted));
                    if let Some((wanted, offered)) = deal.map(|d| (d.wanted, d.offered)) {
                        dialogue.trade = Some(TradeDeal {
                            villager: villager.name,
                            wanted,
                            offered,
                        });
                    }
                    name
                }
                _ => None,
            };
            lines.push(say(talk::greeting(looks, rng.below(2) as usize)));
            lines.push(say(&talk::lines(
                topic,
                &ctx,
                item.as_deref(),
                rng.below(2) as usize,
            )));
            if topic == Topic::Gift {
                lines.push(format!(
                    "(You got the {}!)",
                    item.as_deref().unwrap_or("item")
                ));
            }
            if let Some(deal) = dialogue.trade.as_ref() {
                lines.push(say(&format!(
                    "Would you trade it for my {}?\n(Y: Trade    N: No thanks)",
                    items::name(deal.offered)
                )));
            } else if patience == Patience::MildlyAnnoyed {
                lines.push(say(talk::mildly_annoyed(looks)));
            }
            println!(
                "Talking to {} ({}, {}): {topic:?}, msg #{msg_no:#06x}, patience {patience:?}",
                villager.name,
                villager.personality.english_name(),
                if chat.happy { "happy" } else { "normal mood" },
            );
        }

        // mNpc_CountTalkNum (counts only while the timer from the last talk
        // runs), then mNpc_TalkEndMove: restart the talk timer, and the
        // unlock timer that eventually forgives a pestered villager.
        chat.info.count_talk(looks);
        chat.info.timer = TALK_END_TIMER;
        chat.info.unlock_timer = TEMPER_TABLE[looks.min(TEMPER_TABLE.len() - 1)].0;

        dialogue.active = true;
        dialogue.index = 0;
        dialogue.lines = lines;
        if let Ok(mut vis) = box_query.single_mut() {
            *vis = Visibility::Visible;
        }
        if let Ok(mut text) = text_query.single_mut() {
            text.0 = dialogue.lines[0].clone();
        }
    }
}

/// Build a trade offer for a trade topic: the item they want is picked
/// uniformly among your eligible carried items
/// (`request_selector::pick_random_eligible`), and what they offer comes
/// from three candidate goods (`build_trade_offer`).
fn trade_offer(
    kind: TradeKind,
    inventory: &PlayerInventory,
    rng: &mut dyn FnMut(u32) -> u32,
) -> Option<TradeDeal> {
    let pockets = &inventory.pockets.pockets;
    let eligible: Vec<u8> = pockets
        .iter()
        .map(|&it| match kind {
            TradeKind::Catch(_) => items::is_fish(it) || items::is_bug(it),
            TradeKind::Furniture(_) => items::is_furniture(it),
        } as u8)
        .collect();
    let count = eligible.iter().filter(|&&e| e != 0).count() as u32;
    if count == 0 {
        return None;
    }
    let wanted = request_selector::pick_random_eligible(pockets, &eligible, rng(count))?;
    // Candidate goods: reward goods and catalog furniture, never the wanted item.
    let pool: Vec<items::ItemId> = items::FURNITURE_CATALOG
        .iter()
        .filter_map(|n| items::furniture(n))
        .chain(
            [
                reward::STATIONERY,
                reward::CLOTH,
                reward::CARPET,
                reward::WALLPAPER,
            ]
            .into_iter()
            .filter_map(items::reward_good),
        )
        .filter(|&it| it != wanted.item)
        .collect();
    let pick = |rng: &mut dyn FnMut(u32) -> u32| pool[rng(pool.len() as u32) as usize];
    let goods = [
        (pick(rng), rng(10)),
        (pick(rng), rng(10)),
        (pick(rng), rng(10)),
    ];
    let offer = request_selector::build_trade_offer(
        Some(wanted),
        goods,
        request_selector::TradeItemMode::Random,
        rng(3),
    );
    Some(TradeDeal {
        villager: "",
        wanted: wanted.item,
        offered: offer.offered,
    })
}

/// Run the talk timers in retail frames. When the unlock timer runs out the
/// villager forgets being pestered (talk count reset) and rolls a new mood.
fn tick_talk_timers(
    time: Res<Time>,
    mut rng: ResMut<TalkRng>,
    mut villagers: Query<&mut VillagerTalk>,
) {
    let frames = time.delta().as_secs_f32() * RETAIL_FPS;
    for mut chat in villagers.iter_mut() {
        chat.frame_acc += frames;
        let whole = chat.frame_acc.floor();
        if whole < 1.0 {
            continue;
        }
        chat.frame_acc -= whole;
        let step = whole as u16;
        chat.info.timer = chat.info.timer.saturating_sub(step);
        if chat.info.unlock_timer > 0 {
            chat.info.unlock_timer = chat.info.unlock_timer.saturating_sub(step);
            if chat.info.unlock_timer == 0 {
                chat.info.talk_num = 0;
                chat.happy = rng.below(100) < HAPPY_CHANCE;
            }
        }
    }
}

/// Space or E advances the dialogue; the last line dismisses it.
///
/// Runs in [`InteractSet::Ui`], after talk: the E press that opened the
/// talk was already consumed, so it can't also advance it.
fn dialogue_input(
    mut actions: ResMut<Actions>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut box_query: Query<&mut Visibility, With<DialogueBox>>,
    mut text_query: Query<&mut Text, With<DialogueText>>,
) {
    if !dialogue.active {
        return;
    }
    // A trade offer on the last line waits for Y / N.
    if dialogue.index + 1 >= dialogue.lines.len() && dialogue.trade.is_some() {
        let yes = actions.just_pressed(Action::Yes);
        let no = actions.just_pressed(Action::No) || actions.just_pressed(Action::Cancel);
        if !yes && !no {
            return;
        }
        if let Some(deal) = dialogue.trade.take() {
            let line = if yes && inventory.remove(deal.wanted) {
                // Removing the wanted item freed a pocket for the offered one.
                inventory.add(deal.offered);
                println!(
                    "Traded {} for {}",
                    items::name(deal.wanted),
                    items::name(deal.offered)
                );
                format!(
                    "(You traded your {} for the {}!)",
                    items::name(deal.wanted),
                    items::name(deal.offered)
                )
            } else {
                format!("{}: Oh well, maybe next time!", deal.villager)
            };
            dialogue.lines.push(line);
            dialogue.index += 1;
            if let Ok(mut text) = text_query.single_mut() {
                text.0 = dialogue.lines[dialogue.index].clone();
            }
        }
        return;
    }
    if !(actions.just_pressed(Action::Advance) || actions.consume(Action::Interact)) {
        return;
    }
    dialogue.index += 1;
    if dialogue.index >= dialogue.lines.len() {
        dialogue.active = false;
        dialogue.lines.clear();
        dialogue.index = 0;
        if let Ok(mut vis) = box_query.single_mut() {
            *vis = Visibility::Hidden;
        }
        println!("Dialogue closed");
    } else if let Ok(mut text) = text_query.single_mut() {
        text.0 = dialogue.lines[dialogue.index].clone();
    }
}

/// Keep the HUD in sync with the inventory.
fn update_hud(inventory: Res<PlayerInventory>, mut text_query: Query<&mut Text, With<HudText>>) {
    if !inventory.is_changed() {
        return;
    }
    if let Ok(mut text) = text_query.single_mut() {
        text.0 = format!(
            "Bells: {}   Pockets: {}/{}   (I: open pockets)",
            inventory.bells,
            inventory.used(),
            rustimal_logic::inventory::POCKETS_SLOT_COUNT
        );
    }
}

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DialogueState::default())
            .insert_resource(TalkRng(0x7A1C_5EED))
            .add_systems(Startup, setup_ui)
            .add_systems(
                Update,
                on_talk
                    .after(crate::quest::QuestInputSet)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                dialogue_input.in_set(InteractSet::Ui).in_set(GameplaySet),
            )
            .add_systems(Update, (update_hud, tick_talk_timers).in_set(GameplaySet));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustimal_logic::species::{FishType, InsectType};

    #[test]
    fn trade_offer_wants_an_eligible_carried_item() {
        let mut inv = PlayerInventory::default();
        let mut s = 7u32;
        let mut rng = |n: u32| {
            s = s.wrapping_mul(1103515245).wrapping_add(12345);
            (s >> 16) % n
        };
        // Nothing eligible: no deal.
        inv.add(items::FRUIT);
        assert!(trade_offer(TradeKind::Catch(0), &inv, &mut rng).is_none());
        // With a fish and a bug, the wanted item is one of them.
        let fish = items::fish(FishType::Carp);
        let bug = items::bug(InsectType::CommonButterfly);
        inv.add(fish);
        inv.add(bug);
        let mut wanted = std::collections::HashSet::new();
        for _ in 0..50 {
            let deal = trade_offer(TradeKind::Catch(0), &inv, &mut rng).unwrap();
            assert!(deal.wanted == fish || deal.wanted == bug);
            assert_ne!(deal.offered, deal.wanted);
            wanted.insert(deal.wanted);
        }
        assert_eq!(wanted.len(), 2, "uniform pick reaches every eligible item");
    }
}
