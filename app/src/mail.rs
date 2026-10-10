//! Phase 3: mail system — mailbox, letters, writing, replies.
//!
//! Letter *scoring* uses `rustimal_logic::letter_score::score_letter`
//! (the decomp-verified 7-check scorer from `m_mail_check_ovl.c`:
//! `mMck_check_key_hit_nes`). The low-level `mail` logic module models
//! GameCube byte-array formats (`Mail_c`, GCI regions), so mailbox
//! storage, delivery, and the UI are Bevy-native; nothing from
//! `logic::mail` is called here.
//!
//! Simplifications vs. retail:
//! - No Post Office transit queue or twice-daily delivery schedule —
//!   letters land in the mailbox immediately.
//! - Retail dialogue text lives in ARAM message scripts; all letter text
//!   here is original writing.
//! - A villager's "thank you" for a sent letter arrives as a reply letter
//!   (scored through the real `LetterRank` tiers) rather than in person.

use bevy::ecs::message::MessageReader;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::text::FontSize;
use rustimal_logic::letter_score::{score_letter, LetterRank, TrigramMode, MAIL_BODY_LEN};
use serde::{Deserialize, Serialize};

use crate::dialogue::DialogueState;
use crate::house::{HouseState, PlayerHouse};
use crate::interaction::{Facing, TalkEvent};
use crate::npc::Villager;
use crate::player::Player;

/// Max letters the mailbox holds (decomp `cap::MAILBOX`).
pub const MAILBOX_CAP: usize = 10;
/// E-key range for the mailbox.
const MAILBOX_RANGE: f32 = 3.5;
/// Facing-cone cosine threshold (same as interaction).
const MAILBOX_DOT: f32 = 0.2;
/// Chance a talked-to villager sends a letter afterwards.
const VILLAGER_LETTER_CHANCE: f32 = 0.15;
/// Max characters in a player-written letter.
const MAX_LETTER_LEN: usize = 200;
/// "You have mail!" notification duration.
const NOTIFY_DURATION: f32 = 3.0;

/// A single letter. Serializable so the save file can persist it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Letter {
    pub sender: String,
    pub subject: String,
    pub body: String,
    pub read: bool,
}

/// The player's mailbox: received letters plus sent ones.
#[derive(Resource, Default)]
pub struct Mailbox {
    pub letters: Vec<Letter>,
    pub sent: Vec<Letter>,
}

/// Mail UI state. Other plugins read `open` to yield the E key.
#[derive(Resource, Default)]
pub struct MailUi {
    pub open: bool,
    selected: usize,
    reading: Option<usize>,
    writing: bool,
    write_recipient: usize,
    write_text: String,
}

/// System set for the mailbox E-key check — runs before the town
/// `interact` system (same slot as `HouseInputSet`).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MailInputSet;

/// World position of the mailbox (set at startup).
#[derive(Resource, Default)]
struct MailboxPos(Vec3);

/// UI markers.
#[derive(Component)]
struct MailPanel;
#[derive(Component)]
struct MailListText;
#[derive(Component)]
struct MailBodyText;
#[derive(Component)]
struct MailHintText;
#[derive(Component)]
struct MailNotifyText;

/// "You have mail!" countdown.
#[derive(Resource, Default)]
struct MailNotifyTimer(f32);

/// Simple xorshift RNG (same pattern as npc.rs).
#[derive(Resource)]
struct MailRng(u64);
impl MailRng {
    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }
    fn chance(&mut self, p: f32) -> bool {
        (self.next_u32() as f32 / u32::MAX as f32) < p
    }
}

/// Starter letters (original writing).
fn starter_letters() -> Vec<Letter> {
    vec![
        Letter {
            sender: "Mom".to_string(),
            subject: "Thinking of you".to_string(),
            body: "Hi sweetie! I hope you're settling into your new town. \
                I packed some extra cookies in the cupboard — the chocolate chip \
                ones, your favorite. Don't stay up too late, and remember to \
                write! Love, Mom"
                .to_string(),
            read: false,
        },
        Letter {
            sender: "Mom".to_string(),
            subject: "Don't forget!".to_string(),
            body: "It's me again! Your father says hi. He wanted me to remind \
                you to water any flowers you plant. A little water goes a long \
                way, just like a little kindness. Take care of yourself out \
                there! Love, Mom"
                .to_string(),
            read: false,
        },
        Letter {
            sender: "Tom Nook".to_string(),
            subject: "Welcome!".to_string(),
            body: "Welcome to town! This is Tom Nook, your friendly \
                shopkeeper. My store is open every day with everything a new \
                homeowner needs. And speaking of homes — do come by when you're \
                ready to talk about your mortgage! No rush. Well... a little \
                rush. Your friend, Tom Nook"
                .to_string(),
            read: false,
        },
    ]
}

/// A letter a villager sends after talking (original writing).
fn villager_letter(name: &str) -> Letter {
    Letter {
        sender: name.to_string(),
        subject: "Hi hi!".to_string(),
        body: format!(
            "It's {name}! I had such a nice time chatting with you today. \
            We should do it again soon — maybe by the river? Anyway, just \
            wanted to say hi! — {name}"
        ),
        read: false,
    }
}

/// Reply to a player-sent letter, tiered by the real `LetterRank`.
fn reply_letter(recipient: &str, rank: LetterRank) -> Letter {
    let body = match rank {
        LetterRank::Ok => format!(
            "Your letter made my whole day! You write the nicest things. \
            Let's hang out soon — I'll bring snacks! — {recipient}"
        ),
        _ => format!(
            "Thanks for the letter... I think? I read it three times and I'm \
            still not quite sure what it said. Maybe we should just talk in \
            person next time! — {recipient}"
        ),
    };
    Letter {
        sender: recipient.to_string(),
        subject: "Re: your letter".to_string(),
        body,
        read: false,
    }
}

/// Push a letter, respecting the mailbox cap. Returns false when full.
fn push_letter(mailbox: &mut Mailbox, letter: Letter) -> bool {
    if mailbox.letters.len() >= MAILBOX_CAP {
        return false;
    }
    mailbox.letters.push(letter);
    true
}

/// Pack typed text into the fixed-size body the scorer expects
/// (space-padded, like the decomp's `CHAR_SPACE` fill).
fn letter_body_bytes(text: &str) -> [u8; MAIL_BODY_LEN] {
    let mut body = [0x20u8; MAIL_BODY_LEN];
    for (i, b) in text.bytes().take(MAIL_BODY_LEN).enumerate() {
        body[i] = b;
    }
    body
}

/// Build the UI: hidden mail panel + "you have mail" notification.
fn setup_mail_ui(mut commands: Commands) {
    commands
        .spawn((
            MailPanel,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(22.0),
                right: Val::Percent(22.0),
                top: Val::Percent(14.0),
                bottom: Val::Percent(14.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.07, 0.09, 0.14, 0.96)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Static title — no marker, never updated.
            parent.spawn((
                Text::new("Mailbox"),
                TextFont {
                    font_size: FontSize::Px(28.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.5)),
            ));
            parent.spawn((
                MailListText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                MailBodyText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                MailHintText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.75)),
            ));
        });

    // "You have mail!" notification — top-center, below the save notice.
    commands.spawn((
        MailNotifyText,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(22.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.95, 0.6)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(52.0),
            left: Val::Percent(50.0),
            ..default()
        },
        Visibility::Hidden,
    ));
    println!("Mail UI ready");
}

/// Spawn the mailbox mesh next to the player house and seed letters
/// (from the save file when one was loaded, else the starters).
fn spawn_mailbox(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    house: Option<Res<PlayerHouse>>,
    loaded: Option<Res<crate::save::LoadedSave>>,
    mut mailbox: ResMut<Mailbox>,
) {
    let Some(house) = house else { return };
    let pos = house.door_pos + Vec3::new(2.4, 0.0, 0.8);
    commands.insert_resource(MailboxPos(pos));

    // Post.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.14, 1.2, 0.14))),
        MeshMaterial3d(materials.add(Color::srgb(0.42, 0.28, 0.16))),
        Transform::from_translation(pos + Vec3::new(0.0, 0.6, 0.0)),
    ));
    // Blue mail box.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.7, 0.5, 0.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.15, 0.35, 0.85))),
        Transform::from_translation(pos + Vec3::new(0.0, 1.35, 0.0)),
    ));
    // Little red flag.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.08, 0.35, 0.08))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.15, 0.15))),
        Transform::from_translation(pos + Vec3::new(0.32, 1.6, 0.0)),
    ));

    if let Some(save) = loaded.as_ref().and_then(|l| l.0.as_ref()) {
        mailbox.letters = save.letters.clone();
        mailbox.sent = save.sent_letters.clone();
        println!("Restored {} letters from save", mailbox.letters.len());
    } else {
        mailbox.letters = starter_letters();
        println!("Seeded starter letters");
    }
}

/// E at the mailbox: open the mail UI.
///
/// Runs in `MailInputSet`, before the town `interact` system, so the same
/// E press can't also trigger talk/pickup.
fn mailbox_interact(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<HouseState>>,
    shop_loc: Res<State<crate::shop::ShopLocation>>,
    dialogue: Res<DialogueState>,
    tool: Res<crate::ecology::EquippedTool>,
    mut ui: ResMut<MailUi>,
    shop_ui: Res<crate::shop::ShopUi>,
    quest_ui: Res<crate::quest::QuestUi>,
    mailbox_pos: Option<Res<MailboxPos>>,
    player_query: Query<(&Transform, &Facing), With<Player>>,
    mut panel_query: Query<&mut Visibility, With<MailPanel>>,
) {
    // Inside the house or shop, E belongs to the doors.
    if *state != HouseState::Town || *shop_loc != crate::shop::ShopLocation::Town {
        return;
    }
    if ui.open || shop_ui.open || dialogue.active || quest_ui.open {
        return;
    }
    // A held tool claims the E press for the ecology plugin.
    if *tool != crate::ecology::EquippedTool::None {
        return;
    }
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Some(pos) = mailbox_pos else { return };
    let Ok((transform, facing)) = player_query.single() else {
        return;
    };
    let to_box = pos.0 - transform.translation;
    let dist = to_box.length();
    if dist > MAILBOX_RANGE || dist < 0.001 {
        return;
    }
    if facing.dir.dot(to_box / dist) < MAILBOX_DOT {
        return;
    }
    ui.open = true;
    ui.selected = 0;
    ui.reading = None;
    ui.writing = false;
    if let Ok(mut vis) = panel_query.single_mut() {
        *vis = Visibility::Visible;
    }
    println!("Mailbox opened");
}

/// Show the "you have mail" notification.
fn show_mail_notify(
    text: &str,
    timer: &mut ResMut<MailNotifyTimer>,
    query: &mut Query<
        (&mut Text, &mut Visibility),
        (With<MailNotifyText>, Without<MailPanel>),
    >,
) {
    if let Ok((mut t, mut vis)) = query.single_mut() {
        t.0 = text.to_string();
        *vis = Visibility::Visible;
    }
    timer.0 = NOTIFY_DURATION;
}

/// Fade the notification out.
fn tick_mail_notify(
    time: Res<Time>,
    mut timer: ResMut<MailNotifyTimer>,
    mut query: Query<&mut Visibility, With<MailNotifyText>>,
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

/// Send the composed letter: score it with the decomp-verified scorer and
/// queue a tiered reply (middle tier sends no reply, per `sends_reply`).
#[allow(clippy::too_many_arguments)]
fn send_letter(
    ui: &mut MailUi,
    mailbox: &mut Mailbox,
    names: &[String],
    timer: &mut ResMut<MailNotifyTimer>,
    notify_query: &mut Query<
        (&mut Text, &mut Visibility),
        (With<MailNotifyText>, Without<MailPanel>),
    >,
) {
    let recipient = names
        .get(ui.write_recipient)
        .cloned()
        .unwrap_or_else(|| "???".to_string());
    let text = ui.write_text.trim().to_string();
    ui.writing = false;
    ui.write_text.clear();
    if text.is_empty() {
        show_mail_notify("Blank letter discarded.", timer, notify_query);
        return;
    }
    let total = score_letter(&letter_body_bytes(&text), TrigramMode::Intended).total();
    let rank = LetterRank::of_score(total);
    mailbox.sent.push(Letter {
        sender: "You".to_string(),
        subject: format!("To {recipient}"),
        body: text,
        read: true,
    });
    match rank {
        LetterRank::Ok => {
            if push_letter(mailbox, reply_letter(&recipient, rank)) {
                show_mail_notify(
                    &format!("{recipient} loved your letter! (score {total})"),
                    timer,
                    notify_query,
                );
            } else {
                show_mail_notify("Mailbox full!", timer, notify_query);
            }
        }
        LetterRank::Bad => {
            if push_letter(mailbox, reply_letter(&recipient, rank)) {
                show_mail_notify(
                    &format!("{recipient} seemed confused... (score {total})"),
                    timer,
                    notify_query,
                );
            } else {
                show_mail_notify("Mailbox full!", timer, notify_query);
            }
        }
        LetterRank::Middle => {
            show_mail_notify(
                &format!("Letter sent! (score {total} — no reply)"),
                timer,
                notify_query,
            );
        }
    }
    println!("Sent letter to {recipient} (score {total}, rank {rank:?})");
}

/// Mail UI keyboard handling: list nav, reading, and composing.
#[allow(clippy::too_many_arguments)]
fn mail_ui_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut key_events: MessageReader<KeyboardInput>,
    mut ui: ResMut<MailUi>,
    mut mailbox: ResMut<Mailbox>,
    villagers: Query<&Villager>,
    mut timer: ResMut<MailNotifyTimer>,
    mut notify_query: Query<
        (&mut Text, &mut Visibility),
        (With<MailNotifyText>, Without<MailPanel>),
    >,
    mut panel_query: Query<&mut Visibility, (With<MailPanel>, Without<MailNotifyText>)>,
) {
    if !ui.open {
        return;
    }

    // --- Reading view ---
    if let Some(idx) = ui.reading {
        if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::Backspace) {
            if let Some(letter) = mailbox.letters.get_mut(idx) {
                letter.read = true;
            }
            ui.reading = None;
        }
        return;
    }

    // --- Compose view ---
    if ui.writing {
        // Drain typed characters from the key events (printable characters
        // arrive as `Key::Character`; control keys like Enter are handled
        // via `ButtonInput` below).
        for ev in key_events.read() {
            if ev.state != ButtonState::Pressed {
                continue;
            }
            if let Key::Character(s) = &ev.logical_key {
                for c in s.chars() {
                    if c.is_ascii() && !c.is_ascii_control() && ui.write_text.len() < MAX_LETTER_LEN
                    {
                        ui.write_text.push(c);
                    }
                }
            }
        }
        let names: Vec<String> = villagers.iter().map(|v| v.name.to_string()).collect();
        if keyboard.just_pressed(KeyCode::Escape) {
            ui.writing = false;
            ui.write_text.clear();
            return;
        }
        if !names.is_empty() {
            if keyboard.just_pressed(KeyCode::ArrowUp) {
                ui.write_recipient = ui.write_recipient.saturating_sub(1);
                return;
            }
            if keyboard.just_pressed(KeyCode::ArrowDown) {
                ui.write_recipient = (ui.write_recipient + 1).min(names.len() - 1);
                return;
            }
        }
        if keyboard.just_pressed(KeyCode::Backspace) {
            ui.write_text.pop();
            return;
        }
        if keyboard.just_pressed(KeyCode::Enter) {
            send_letter(&mut ui, &mut mailbox, &names, &mut timer, &mut notify_query);
        }
        return;
    }

    // --- List view ---
    if keyboard.just_pressed(KeyCode::Escape) {
        ui.open = false;
        if let Ok(mut vis) = panel_query.single_mut() {
            *vis = Visibility::Hidden;
        }
        println!("Mailbox closed");
        return;
    }
    // Drain any stray key events so a queued 'n' can't leak into a
    // letter the player starts composing below.
    key_events.clear();
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        ui.selected = ui.selected.saturating_sub(1);
        return;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        ui.selected = (ui.selected + 1).min(mailbox.letters.len().saturating_sub(1));
        return;
    }
    if keyboard.just_pressed(KeyCode::Enter) && !mailbox.letters.is_empty() {
        ui.reading = Some(ui.selected.min(mailbox.letters.len() - 1));
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyN) {
        ui.writing = true;
        ui.write_recipient = 0;
        ui.write_text.clear();
    }
}

/// Re-render the mail UI text whenever state changes.
#[allow(clippy::too_many_arguments)]
fn refresh_mail_ui(
    ui: Res<MailUi>,
    mailbox: Res<Mailbox>,
    villagers: Query<&Villager>,
    mut list_query: Query<
        (&mut Text, &mut Visibility),
        (With<MailListText>, Without<MailBodyText>, Without<MailHintText>),
    >,
    mut body_query: Query<
        (&mut Text, &mut Visibility),
        (With<MailBodyText>, Without<MailListText>, Without<MailHintText>),
    >,
    mut hint_query: Query<
        &mut Text,
        (With<MailHintText>, Without<MailListText>, Without<MailBodyText>),
    >,
) {
    if !ui.is_changed() && !mailbox.is_changed() {
        return;
    }
    let Ok((mut list_text, mut list_vis)) = list_query.single_mut() else {
        return;
    };
    let Ok((mut body_text, mut body_vis)) = body_query.single_mut() else {
        return;
    };
    let Ok(mut hint_text) = hint_query.single_mut() else {
        return;
    };

    if let Some(idx) = ui.reading {
        *list_vis = Visibility::Hidden;
        *body_vis = Visibility::Visible;
        if let Some(letter) = mailbox.letters.get(idx) {
            body_text.0 = format!(
                "From: {}\nSubject: {}\n\n{}",
                letter.sender, letter.subject, letter.body
            );
        }
        hint_text.0 = "[Esc]/[Backspace] back to list".to_string();
    } else if ui.writing {
        *list_vis = Visibility::Hidden;
        *body_vis = Visibility::Visible;
        let names: Vec<String> = villagers.iter().map(|v| v.name.to_string()).collect();
        let name = names
            .get(ui.write_recipient)
            .map(|s| s.as_str())
            .unwrap_or("???");
        body_text.0 = format!("To: {name}   (Up/Down to change)\n\n{}_", ui.write_text);
        hint_text.0 = "[Enter] send   [Esc] cancel".to_string();
    } else {
        *list_vis = Visibility::Visible;
        *body_vis = Visibility::Hidden;
        let mut s = String::new();
        for (i, letter) in mailbox.letters.iter().enumerate() {
            let marker = if i == ui.selected { ">" } else { " " };
            let flag = if letter.read { "    " } else { "NEW " };
            s.push_str(&format!(
                "{marker} [{flag}] {} — {}\n",
                letter.sender, letter.subject
            ));
        }
        if mailbox.letters.is_empty() {
            s.push_str("(no letters yet)\n");
        }
        list_text.0 = s;
        hint_text.0 = "[Up/Down] select   [Enter] read   [N] write   [Esc] close".to_string();
    }
}

/// After talking to a villager, there's a small chance they send a letter.
fn on_talk_maybe_letter(
    mut events: MessageReader<TalkEvent>,
    villagers: Query<&Villager>,
    mut mailbox: ResMut<Mailbox>,
    mut rng: ResMut<MailRng>,
    mut timer: ResMut<MailNotifyTimer>,
    mut notify_query: Query<
        (&mut Text, &mut Visibility),
        (With<MailNotifyText>, Without<MailPanel>),
    >,
) {
    for event in events.read() {
        let Ok(v) = villagers.get(event.villager) else {
            continue;
        };
        if !rng.chance(VILLAGER_LETTER_CHANCE) {
            continue;
        }
        if push_letter(&mut mailbox, villager_letter(v.name)) {
            show_mail_notify(
                &format!("{} sent you a letter!", v.name),
                &mut timer,
                &mut notify_query,
            );
            println!("{} sent a letter", v.name);
        }
    }
}

pub struct MailPlugin;

impl Plugin for MailPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Mailbox::default())
            .insert_resource(MailUi::default())
            .insert_resource(MailNotifyTimer::default())
            .insert_resource(MailRng(0x5EED1234))
            .add_systems(Startup, setup_mail_ui)
            .add_systems(
                Startup,
                spawn_mailbox.after(crate::house::find_player_house),
            )
            .add_systems(Update, mailbox_interact.in_set(MailInputSet))
            .add_systems(
                Update,
                (
                    mail_ui_input,
                    refresh_mail_ui,
                    on_talk_maybe_letter,
                    tick_mail_notify,
                ),
            );
    }
}
