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
    /// The villager talking (their mouth moves while the box is up).
    pub speaker: Option<Entity>,
    lines: Vec<String>,
    index: usize,
    /// A trade offer waiting on Y/N at the last line.
    trade: Option<TradeDeal>,
    /// The highlighted choice while [`DialogueState::choices`] is up.
    pub choice: usize,
    /// The talk window is still typing the page out: E waits.
    pub revealing: bool,
    /// `mDemo_Set_talk_window_color`: a window colour other than the villagers' default.
    pub(crate) window_color: Option<[u8; 4]>,
    /// `mMsg_SET_LOCKCONTINUE`: the page cannot be continued while an actor's demo runs.
    pub(crate) held: bool,
    /// Actor controllers own these choices and commit their own transactions.
    pub(crate) external_name: Option<&'static str>,
    /// The special NPC's `npc_draw_data_tbl` index and `l_sp_actor_name`
    /// sound id, for its voice.
    pub(crate) external_voice: Option<(u32, u8)>,
    pub(crate) external_result: Option<usize>,
    external_choices: Option<Vec<String>>,
    /// A question the last page asks (the game's own message): its
    /// answers and branches, with the speaker's name and the message
    /// context to render the chosen branch.
    ask: Option<(
        crate::messages::MsgChoice,
        &'static str,
        crate::messages::MsgContext,
    )>,
}

/// The trade question's answers (first = yes).
const TRADE_CHOICES: [&str; 2] = ["Let's trade!", "No thanks."];

impl DialogueState {
    /// Furniture owns its result just as an NPC controller does, but has no
    /// speaking actor (and must not animate a villager's mouth).
    pub(crate) fn begin_furniture(&mut self, page: String, choices: Option<Vec<String>>) {
        *self = Self {
            active: true,
            lines: vec![page],
            external_name: Some(""),
            external_choices: choices,
            ..default()
        };
    }
    pub(crate) fn choices_for_actor(&self) -> Option<&Vec<String>> {
        self.external_choices.as_ref()
    }

    pub(crate) fn set_actor_choices(&mut self, choices: Vec<String>) {
        self.external_choices = Some(choices);
    }

    pub(crate) fn begin_external(
        &mut self,
        speaker: Entity,
        pages: Vec<String>,
        choices: Option<Vec<String>>,
    ) {
        // m_npc.c l_sp_actor_name: Nook's sound_id is 2.
        self.begin_special(
            speaker,
            "Tom Nook",
            (crate::character::NOOK, 2),
            pages,
            choices,
        );
    }

    /// A special NPC's talk: name plate, voice and its own choices.
    pub(crate) fn begin_special(
        &mut self,
        speaker: Entity,
        name: &'static str,
        voice: (u32, u8),
        pages: Vec<String>,
        choices: Option<Vec<String>>,
    ) {
        *self = Self {
            active: true,
            speaker: Some(speaker),
            lines: pages,
            external_name: Some(name),
            external_voice: Some(voice),
            external_choices: choices,
            ..default()
        };
    }

    /// The answers on offer, while the last page waits on one.
    pub fn choices(&self) -> Option<Vec<String>> {
        if !self.active || self.index + 1 < self.lines.len() {
            return None;
        }
        if self.trade.is_some() {
            return Some(TRADE_CHOICES.iter().map(|s| s.to_string()).collect());
        }
        if self.external_name.is_some() {
            return self.external_choices.clone();
        }
        self.ask.as_ref().map(|(c, ..)| c.labels.clone())
    }

    /// The plain box's text: the page, plus the choices as a key hint.
    fn shown_text(&self) -> String {
        let page = crate::messages::markup::strip(self.page().unwrap_or_default());
        match self.choices() {
            // Any number of answers (a clerk may offer just one).
            Some(c) if c.len() == 2 => format!("{page}\n(Y: {}    N: {})", c[0], c[1]),
            Some(c) => format!("{page}\n({})", c.join("    ")),
            None => page,
        }
    }

    /// The page shown in the box, while it's up.
    pub fn page(&self) -> Option<&str> {
        self.active
            .then(|| self.lines.get(self.index).map(String::as_str))
            .flatten()
    }
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
pub(crate) struct DialogueBox;

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

impl VillagerTalk {
    /// Too pestered to talk (`Patience::Annoyed`).
    pub fn annoyed(&self, looks: u8) -> bool {
        self.info.patience(looks as usize) == Patience::Annoyed
    }
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
        crate::dev_overlay::DevOverlay,
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
    mut roster: Option<ResMut<crate::roster::TownRoster>>,
    disc: Option<Res<crate::game_disc::GameDisc>>,
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
                leaving: roster
                    .as_deref()
                    .zip(villager.npc_index)
                    .is_some_and(|(r, i)| r.is_leaver(i)),
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
                        crate::goods::random(crate::goods::Kind::Furniture, &mut |n| rng.below(n))
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
            // The game's own words for this message, when the disc has them.
            let disc_pages = disc.as_deref().and_then(|d| {
                let text = d.text.as_ref()?;
                let msg = disc_message_context(d, villager, item.as_deref(), &now);
                let (pages, ask) = text.talk(msg_no as usize, &msg, &mut |n| rng.below(n));
                (!pages.is_empty()).then_some((pages, ask.map(|a| (a, villager.name, msg))))
            });
            match disc_pages {
                Some((pages, ask)) => {
                    lines.extend(pages.iter().map(|p| say(p)));
                    if let Some((a, ..)) = &ask {
                        println!("{} asks: {:?}", villager.name, a.labels);
                    }
                    dialogue.ask = ask;
                }
                None => {
                    lines.push(say(talk::greeting(looks, rng.below(2) as usize)));
                    lines.push(say(&talk::lines(
                        topic,
                        &ctx,
                        item.as_deref(),
                        rng.below(2) as usize,
                    )));
                }
            }
            if topic == Topic::Gift {
                lines.push(format!(
                    "(You got the {}!)",
                    item.as_deref().unwrap_or("item")
                ));
            }
            if let Some(deal) = dialogue.trade.as_ref() {
                lines.push(say(&format!(
                    "Would you trade it for my {}?",
                    items::name(deal.offered)
                )));
            } else if patience == Patience::MildlyAnnoyed {
                lines.push(say(talk::mildly_annoyed(looks)));
            }
            println!(
                "Talking to {} ({}, {}{}): {topic:?}, msg #{msg_no:#06x}, patience {patience:?}",
                villager.name,
                villager.personality.english_name(),
                if chat.happy { "happy" } else { "normal mood" },
                if ctx.leaving {
                    ", thinking of moving"
                } else {
                    ""
                },
            );
        }

        // mNpc_CountTalkNum (counts only while the timer from the last talk
        // runs), then mNpc_TalkEndMove: restart the talk timer, and the
        // unlock timer that eventually forgives a pestered villager.
        chat.info.count_talk(looks);
        chat.info.timer = TALK_END_TIMER;
        chat.info.unlock_timer = TEMPER_TABLE[looks.min(TEMPER_TABLE.len() - 1)].0;

        dialogue.active = true;
        dialogue.speaker = Some(event.villager);
        if let (Some(roster), Some(idx)) = (roster.as_deref_mut(), villager.npc_index) {
            if !roster.met.contains(&idx) {
                roster.met.push(idx);
            }
        }
        dialogue.index = 0;
        dialogue.choice = 0;
        if dialogue.trade.is_some() {
            dialogue.ask = None;
        }
        dialogue.lines = lines;
        if let Ok(mut vis) = box_query.single_mut() {
            *vis = Visibility::Visible;
        }
        if let Ok(mut text) = text_query.single_mut() {
            text.0 = dialogue.shown_text();
        }
    }
}

/// What the game fills into a villager's message: names, their
/// catchphrase (`npc_def_list[id].catchphrase_str_idx` in the string bank),
/// the topic's item, and the date and time.
pub(crate) fn disc_message_context(
    disc: &crate::game_disc::GameDisc,
    villager: &Villager,
    item: Option<&str>,
    now: &crate::clock::Now,
) -> crate::messages::MsgContext {
    /// `mNpc_Default_Data_c`: cloth u16, catchphrase string u16, umbrella.
    const DEFAULT_SIZE: u32 = 6;
    let tail = villager
        .npc_index
        .and_then(|i| {
            let list = disc.catalog.get("npc_def_list")?;
            let b = disc
                .rom
                .slice(disc::RomSource::Rel, list.offset + i * DEFAULT_SIZE + 2, 2)?;
            disc.text
                .as_ref()?
                .string(u16::from_be_bytes([b[0], b[1]]) as usize)
        })
        .unwrap_or_default();
    let item = item.unwrap_or_default().to_string();
    crate::messages::MsgContext {
        player_name: PLAYER_NAME.to_string(),
        talk_name: villager.name.to_string(),
        tail,
        // Which slot a message reads depends on the message; the topic's
        // item is the only thing we fill in, so offer it everywhere.
        free: vec![item.clone(); 20],
        items: vec![item; 5],
        item_articles: Vec::new(),
        determination: String::new(),
        year: chrono::Datelike::year(&chrono::Local::now()) as u16,
        month: now.month,
        weekday: now.weekday,
        day: now.day,
        hour: now.hour,
        minute: now.minute,
    }
}

/// The player's name until the game asks for one (the train ride).
const PLAYER_NAME: &str = "Player";

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
    use crate::goods::Kind;
    let pool: Vec<items::ItemId> = [
        Kind::Furniture,
        Kind::Furniture,
        Kind::Paper,
        Kind::Cloth,
        Kind::Carpet,
        Kind::Wallpaper,
    ]
    .into_iter()
    .map(|k| crate::goods::random(k, rng))
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

/// Ends the talk and hides the plain box.
fn close(dialogue: &mut DialogueState, box_query: &mut Query<&mut Visibility, With<DialogueBox>>) {
    dialogue.active = false;
    dialogue.speaker = None;
    dialogue.lines.clear();
    dialogue.index = 0;
    dialogue.ask = None;
    if let Ok(mut vis) = box_query.single_mut() {
        *vis = Visibility::Hidden;
    }
    println!("Dialogue closed");
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
    (disc, mut rng): (Option<Res<crate::game_disc::GameDisc>>, ResMut<TalkRng>),
) {
    if !dialogue.active || dialogue.revealing || dialogue.held {
        return;
    }
    if dialogue.external_name.is_some() {
        if let Some(choices) = dialogue.choices() {
            let before = dialogue.choice;
            let n = choices.len();
            if actions.consume(Action::MoveUp) || actions.consume(Action::MenuUp) {
                dialogue.choice = (dialogue.choice + n - 1) % n;
            }
            if actions.consume(Action::MoveDown) || actions.consume(Action::MenuDown) {
                dialogue.choice = (dialogue.choice + 1) % n;
            }
            crate::sound::cursor_changed(before, dialogue.choice);
            let cancel = actions.consume(Action::Cancel) || actions.consume(Action::Back);
            if cancel
                || actions.consume(Action::Confirm)
                || actions.consume(Action::Advance)
                || actions.consume(Action::Interact)
            {
                dialogue.external_result = Some(if cancel { n - 1 } else { dialogue.choice });
                dialogue.active = false;
                crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
            }
        } else if actions.consume(Action::Advance)
            || actions.consume(Action::Confirm)
            || actions.consume(Action::Interact)
        {
            dialogue.index += 1;
            if dialogue.index >= dialogue.lines.len() {
                dialogue.external_result = Some(usize::MAX);
                dialogue.active = false;
            }
            crate::sound::se(crate::sound::se_id::PAGE_OKURI);
        }
        if let Ok(mut text) = text_query.single_mut() {
            text.0 = dialogue.shown_text();
        }
        return;
    }
    // A trade offer on the last line waits on its choice: up/down picks,
    // E/Enter answers; Y / N answer directly.
    if let Some(choices) = dialogue.choices() {
        let n = choices.len();
        let before = dialogue.choice;
        if actions.just_pressed(Action::MenuUp) || actions.just_pressed(Action::MoveUp) {
            dialogue.choice = (dialogue.choice + n - 1) % n;
        }
        if actions.just_pressed(Action::MenuDown) || actions.just_pressed(Action::MoveDown) {
            dialogue.choice = (dialogue.choice + 1) % n;
        }
        crate::sound::cursor_changed(before, dialogue.choice);
        let pick = actions.just_pressed(Action::Confirm)
            || actions.just_pressed(Action::Advance)
            || actions.consume(Action::Interact);
        // The game's own question: go on down the chosen branch.
        if pick && dialogue.trade.is_none() {
            crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
            if let Some((ask, speaker, ctx)) = dialogue.ask.take() {
                let next = ask.next.get(dialogue.choice).copied().flatten();
                let branch = disc
                    .as_deref()
                    .and_then(|d| d.text.as_ref())
                    .zip(next)
                    .map(|(text, no)| text.talk(no, &ctx, &mut |n| rng.below(n)));
                dialogue.choice = 0;
                match branch {
                    Some((pages, ask)) if !pages.is_empty() => {
                        dialogue
                            .lines
                            .extend(pages.iter().map(|p| format!("{speaker}: {p}")));
                        dialogue.ask = ask.map(|a| (a, speaker, ctx));
                        dialogue.index += 1;
                        if let Ok(mut text) = text_query.single_mut() {
                            text.0 = dialogue.shown_text();
                        }
                    }
                    _ => close(&mut dialogue, &mut box_query),
                }
            }
            return;
        }
        if dialogue.trade.is_none() {
            return;
        }
        let yes = actions.just_pressed(Action::Yes) || (pick && dialogue.choice == 0);
        let no = actions.just_pressed(Action::No)
            || actions.just_pressed(Action::Cancel)
            || (pick && dialogue.choice != 0);
        if !yes && !no {
            return;
        }
        crate::sound::se(if no {
            crate::sound::se_id::MENU_EXIT
        } else {
            crate::sound::se_id::CHOICE_CONFIRM
        });
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
                text.0 = dialogue.shown_text();
            }
        }
        return;
    }
    if !(actions.just_pressed(Action::Advance) || actions.consume(Action::Interact)) {
        return;
    }
    crate::sound::se(crate::sound::se_id::PAGE_OKURI);
    dialogue.index += 1;
    if dialogue.index >= dialogue.lines.len() {
        close(&mut dialogue, &mut box_query);
    } else if let Ok(mut text) = text_query.single_mut() {
        text.0 = dialogue.shown_text();
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

/// F8: compare normal speech, accents and melody notes with each resident.
/// Includes residents indoors so their schedules cannot hide a voice type.
fn debug_voice(
    mut actions: ResMut<Actions>,
    focus: Res<crate::input::InputFocus>,
    villagers: Query<(Entity, &Villager)>,
    mut dialogue: ResMut<DialogueState>,
    mut next: Local<usize>,
    mut boxes: Query<&mut Visibility, With<DialogueBox>>,
    mut texts: Query<&mut Text, With<DialogueText>>,
) {
    if !focus.is_world() || dialogue.active || !actions.consume(Action::DebugVoice) {
        return;
    }
    let mut residents: Vec<_> = villagers.iter().collect();
    residents.sort_by_key(|(_, v)| (v.looks, v.npc_index, v.name));
    if residents.is_empty() {
        return;
    }
    let (entity, v) = residents[*next % residents.len()];
    *next = (*next + 1) % residents.len();
    // Default-colored notes select voice 0; a colored note retains this
    // resident's voice_type (mFontSentence_animal_voice_se).
    let mut resident_melody = String::from("My melody: ");
    resident_melody.push(crate::messages::markup::COLOR_CHARS);
    resident_melody.extend([75, 95, 155, 1].map(crate::messages::markup::arg));
    resident_melody.push('♪');
    *dialogue = DialogueState {
        active: true,
        speaker: Some(entity),
        lines: vec![
            "Hello there! How are you doing today?".to_string(),
            "Café, déjà vu!\nSeñor, où êtes-vous?".to_string(),
            "A little melody: ♪".to_string(),
            resident_melody,
        ],
        ..default()
    };
    let spec = rustimal_logic::na_voice::villager_spec(v.looks);
    let sequence = rustimal_logic::na_voice::voice_sequence(spec);
    info!(
        "Voice comparison: {} ({}, looks {}), spec {}, sequence {:?}. E advances through normal/accent/default-melody/resident-melody pages, then closes; F8 selects the next resident.",
        v.name, v.personality.english_name(), v.looks, spec, sequence
    );
    for mut visibility in &mut boxes {
        *visibility = Visibility::Visible;
    }
    for mut text in &mut texts {
        text.0 = dialogue.shown_text();
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
            .add_systems(Update, debug_voice.after(on_talk).in_set(GameplaySet))
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
