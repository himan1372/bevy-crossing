//! Phase 3: mail system — mailbox, letters, writing, replies.
//!
//! Letter *scoring* uses `rustimal_logic::letter_score::score_letter`
//! (the decomp-verified 7-check scorer from `m_mail_check_ovl.c`:
//! `mMck_check_key_hit_nes`). The low-level `mail` logic module models
//! GameCube byte-array formats (`Mail_c`, GCI regions), so mailbox
//! storage, delivery, and the UI are Bevy-native; nothing from
//! `logic::mail` is called here.
//!
//! Letters are written on stationery from the pockets ("Write"), may carry
//! a present (an item carried onto the letter), and are posted at the post
//! office counter (post.rs delivers, villagers reply, the museum answers).
//! Letters from others are the disc's own (`GameText::letter`).
//!
//! Simplification vs. retail: the pockets' ten letters and the house
//! mailbox's ten are one store here.

use bevy::ecs::message::MessageReader;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::text::FontSize;
use rustimal_logic::letter_score::MAIL_BODY_LEN;
use serde::{Deserialize, Serialize};

use crate::house::{HouseState, PlayerHouse};
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::Facing;
use crate::npc::Villager;
use crate::player::Player;
use crate::states::GameplaySet;

/// Max letters the mailbox holds (decomp `cap::MAILBOX`).
pub const MAILBOX_CAP: usize = 10;
/// E-key range for the mailbox (ground plane). Short, so standing at the
/// house door doesn't reach it.
const MAILBOX_RANGE: f32 = 2.2;
/// Facing-cone cosine threshold (~60° either side).
const MAILBOX_DOT: f32 = 0.5;
/// Max characters in a player-written letter.
const MAX_LETTER_LEN: usize = 200;

/// Whom a written letter is for (Mail_nm_c type PLAYER / NPC / MUSEUM).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recipient {
    Villager(String),
    Museum,
}

impl Recipient {
    pub fn name(&self) -> &str {
        match self {
            Recipient::Villager(name) => name,
            Recipient::Museum => crate::post::MUSEUM_NAME,
        }
    }
}

fn is_zero(v: &u16) -> bool {
    *v == 0
}

/// A single letter. Serializable so the save file can persist it.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Letter {
    pub sender: String,
    pub subject: String,
    pub body: String,
    pub read: bool,
    /// Original stationery index; older port letters use the first paper.
    #[serde(default)]
    pub paper: u8,
    /// Stable position in the ten-slot letter grid; absent in older saves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<u8>,
    /// The header and footer of a letter from the disc (the recipient's
    /// name already in place); None: the plain "To" / "From" lines.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footer: Option<String>,
    /// The present inside (0: none).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub present: u16,
    /// The player put the present in (mMl_check_set_present_myself): it
    /// comes back out unwrapped.
    #[serde(default)]
    pub present_mine: bool,
    /// A letter the player wrote (mMl_FONT_SEND), and whom it is for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<Recipient>,
}

/// The player's mailbox: received letters plus sent ones.
#[derive(Resource, Default)]
pub struct Mailbox {
    pub letters: Vec<Letter>,
    pub sent: Vec<Letter>,
}

impl Mailbox {
    pub(crate) fn has_room(&self) -> bool {
        self.letters.len() < MAILBOX_CAP
    }

    /// Puts a letter in the first free slot; false when full.
    pub(crate) fn push(&mut self, letter: Letter) -> bool {
        push_letter(self, letter)
    }

    pub(crate) fn slots(&self) -> [Option<usize>; MAILBOX_CAP] {
        let mut slots = [None; MAILBOX_CAP];
        let mut placed = vec![false; self.letters.len()];
        for (index, letter) in self.letters.iter().enumerate() {
            if let Some(slot) = letter.slot.map(usize::from).filter(|&s| s < MAILBOX_CAP) {
                if slots[slot].is_none() {
                    slots[slot] = Some(index);
                    placed[index] = true;
                }
            }
        }
        for (index, done) in placed.into_iter().enumerate() {
            if !done {
                if let Some(slot) = slots.iter_mut().find(|s| s.is_none()) {
                    *slot = Some(index);
                }
            }
        }
        slots
    }

    pub(crate) fn letter_index(&self, slot: usize) -> Option<usize> {
        self.slots().get(slot).copied().flatten()
    }

    fn normalize_slots(&mut self) {
        for (slot, index) in self.slots().into_iter().enumerate() {
            if let Some(index) = index {
                self.letters[index].slot = Some(slot as u8);
            }
        }
    }

    fn swap_slots(&mut self, from: usize, to: usize) {
        if from >= MAILBOX_CAP || to >= MAILBOX_CAP {
            return;
        }
        self.normalize_slots();
        let slots = self.slots();
        if let Some(index) = slots[from] {
            self.letters[index].slot = Some(to as u8);
        }
        if let Some(index) = slots[to] {
            self.letters[index].slot = Some(from as u8);
        }
    }

    fn discard_slot(&mut self, slot: usize) {
        self.normalize_slots();
        if let Some(index) = self.letter_index(slot) {
            self.letters.remove(index);
        }
    }
}

/// Mail UI state. Other plugins read `open` to yield the E key.
#[derive(Resource, Default)]
pub struct MailUi {
    pub open: bool,
    pub(crate) selected: usize,
    pub(crate) reading: Option<usize>,
    pub(crate) writing: bool,
    pub(crate) write_recipient: usize,
    pub(crate) write_text: String,
    pub(crate) page: usize,
    pub(crate) from_pockets: bool,
    pub(crate) menu: Option<usize>,
    pub(crate) discard: bool,
    pub(crate) grabbed: Option<usize>,
    /// The letter the menu is for: (written by the player, has a present).
    pub(crate) menu_letter: (bool, bool),
    /// Writing on the stationery in this pocket slot (its paper index).
    pub(crate) write_paper: Option<(usize, u8)>,
    /// Rewriting this letter (by index) rather than a new one.
    pub(crate) rewriting: Option<usize>,
    /// Who the letter may go to (mAD: villagers, and the museum once its
    /// address is known).
    pub(crate) write_recipients: Vec<Recipient>,
}

impl MailUi {
    pub(crate) fn cancel_action(&mut self) {
        self.menu = None;
        self.discard = false;
        self.grabbed = None;
        crate::sound::se(crate::sound::se_id::MENU_EXIT);
    }

    /// mTG_def_rmail / _rmail_pre / _smail / _smail_pre.
    pub(crate) fn action_options(&self) -> Vec<&'static str> {
        let (written, present) = self.menu_letter;
        if self.discard {
            return vec!["Yes", "No"];
        }
        vec![
            if written { "Rewrite" } else { "Read" },
            "Grab",
            if present { "Present" } else { "Throw Away" },
            "Quit",
        ]
    }

    /// mTG_def_rmail and dump confirmation; mHD-style grab/drop swaps whole mail.
    pub(crate) fn letter_actions(
        &mut self,
        actions: &mut Actions,
        mailbox: &mut Mailbox,
        slot: usize,
        from_pockets: bool,
        inventory: &mut crate::interaction::PlayerInventory,
    ) -> bool {
        if (self.menu.is_some() || self.grabbed.is_some())
            && (actions.consume(Action::Cancel) || actions.consume(Action::Back))
        {
            self.cancel_action();
            return true;
        }
        if let Some(choice) = self.menu {
            let down =
                actions.just_pressed(Action::MoveDown) || actions.just_pressed(Action::MenuDown);
            let up = actions.just_pressed(Action::MoveUp) || actions.just_pressed(Action::MenuUp);
            if up || down {
                self.menu = Some(
                    (choice as i32 + i32::from(down) - i32::from(up))
                        .rem_euclid(self.action_options().len() as i32)
                        as usize,
                );
                crate::sound::se(crate::sound::se_id::CURSOR);
                return true;
            }
            if actions.consume(Action::Confirm) || actions.consume(Action::Interact) {
                self.menu = None;
                if self.discard {
                    if choice == 0 {
                        mailbox.discard_slot(slot);
                    }
                    self.discard = false;
                } else {
                    let label = self.action_options().get(choice).copied().unwrap_or("Quit");
                    let index = mailbox.letter_index(slot);
                    match (label, index) {
                        ("Read", Some(index)) => {
                            self.read_from_pockets(index);
                            self.from_pockets = from_pockets;
                        }
                        ("Rewrite", Some(index)) => {
                            let letter = &mailbox.letters[index];
                            self.open = true;
                            self.writing = true;
                            self.from_pockets = from_pockets;
                            self.rewriting = Some(index);
                            self.write_paper = None;
                            self.write_text = letter.body.clone();
                            self.write_recipients.clear();
                            self.write_recipient = 0;
                        }
                        ("Grab", _) => self.grabbed = Some(slot),
                        ("Present", Some(index)) => {
                            // mTG_present_proc: into the pockets, wrapped
                            // unless the player put it there.
                            let letter = &mut mailbox.letters[index];
                            let free = inventory.pockets.pockets.iter().position(|&i| i == 0);
                            if let Some(free) = free {
                                inventory.pockets.pockets[free] = letter.present;
                                inventory
                                    .pockets
                                    .set_item_cond(free, u32::from(!letter.present_mine));
                                letter.present = 0;
                                letter.present_mine = false;
                            } else {
                                crate::sound::se(crate::sound::se_id::MENU_EXIT);
                            }
                        }
                        ("Throw Away", _) => {
                            self.discard = true;
                            self.menu = Some(1);
                        }
                        _ => {}
                    }
                }
                crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
            }
            return true;
        }
        if actions.consume(Action::Confirm) || actions.consume(Action::Interact) {
            if let Some(from) = self.grabbed.take() {
                mailbox.swap_slots(from, slot);
            } else if let Some(index) = mailbox.letter_index(slot) {
                let letter = &mailbox.letters[index];
                self.menu_letter = (letter.to.is_some(), letter.present != 0);
                self.menu = Some(0);
                self.from_pockets = from_pockets;
            }
            crate::sound::se(crate::sound::se_id::CHOICE_OPEN);
            return true;
        }
        false
    }
    pub(crate) fn listing(&self) -> bool {
        self.open && self.reading.is_none() && !self.writing
    }

    pub(crate) fn read_from_pockets(&mut self, index: usize) {
        self.open = true;
        self.reading = Some(index);
        self.page = 0;
        self.from_pockets = true;
    }
}

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

/// Push a letter, respecting the mailbox cap. Returns false when full.
fn push_letter(mailbox: &mut Mailbox, mut letter: Letter) -> bool {
    if mailbox.letters.len() >= MAILBOX_CAP {
        return false;
    }
    mailbox.normalize_slots();
    letter.slot = mailbox
        .slots()
        .iter()
        .position(Option::is_none)
        .map(|s| s as u8);
    mailbox.letters.push(letter);
    true
}

/// Pack typed text into the fixed-size body the scorer expects
/// (space-padded, like the decomp's `CHAR_SPACE` fill).
pub(crate) fn letter_body_bytes(text: &str) -> [u8; MAIL_BODY_LEN] {
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
    town: Res<crate::town::Town>,
    loaded: Option<Res<crate::save::LoadedSave>>,
    mut mailbox: ResMut<Mailbox>,
) {
    let Some(house) = house else { return };
    if let Some(save) = loaded.as_ref().and_then(|l| l.0.as_ref()) {
        mailbox.letters = save.letters.clone();
        mailbox.sent = save.sent_letters.clone();
        println!("Restored {} letters from save", mailbox.letters.len());
    }
    // The disc town draws the real mailbox at the layout's MAILBOX0.
    if let Some(cell) = town.retail.as_ref().and_then(|r| r.mailboxes[0]) {
        let pos = crate::disc_town::mailbox_position(cell);
        info!("player mailbox at {pos}");
        commands.insert_resource(MailboxPos(pos));
        return;
    }
    // Beside the house (right of the door), clear of the doorway.
    let pos = house.door_pos + Vec3::new(3.0, 0.0, 0.8);
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
}

/// E at the mailbox: open the mail UI.
///
/// Runs in [`InteractSet::Objects`]: opening consumes the press, so the same
/// E press can't also trigger talk/pickup.
fn mailbox_interact(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<HouseState>>,
    shop_loc: Res<State<crate::shop::ShopLocation>>,
    tool: Res<crate::ecology::EquippedTool>,
    mut ui: ResMut<MailUi>,
    mailbox_pos: Option<Res<MailboxPos>>,
    player_query: Query<(&Transform, &Facing), With<Player>>,
    mut panel_query: Query<&mut Visibility, With<MailPanel>>,
) {
    // Inside the house or shop, E belongs to the doors.
    if *state != HouseState::Town || *shop_loc != crate::shop::ShopLocation::Town {
        return;
    }
    if ui.open || !focus.is_world() {
        return;
    }
    // A held tool claims the E press for the ecology plugin.
    if *tool != crate::ecology::EquippedTool::None {
        return;
    }
    if !actions.just_pressed(Action::Interact) {
        return;
    }
    let Some(pos) = mailbox_pos else { return };
    let Ok((transform, facing)) = player_query.single() else {
        return;
    };
    // Ground plane only: the box is at y = 0, the player at y = 1.
    let to_box = Vec3::new(
        pos.0.x - transform.translation.x,
        0.0,
        pos.0.z - transform.translation.z,
    );
    let dist = to_box.length();
    if dist > MAILBOX_RANGE || dist < 0.001 {
        return;
    }
    if facing.dir.dot(to_box / dist) < MAILBOX_DOT || !actions.consume(Action::Interact) {
        return;
    }
    ui.open = true;
    ui.from_pockets = false;
    ui.page = 0;
    ui.menu = None;
    ui.grabbed = None;
    ui.discard = false;
    crate::sound::se(crate::sound::se_id::PANEL_OPEN);
    ui.selected = 0;
    ui.reading = None;
    ui.writing = false;
    if let Ok(mut vis) = panel_query.single_mut() {
        *vis = Visibility::Visible;
    }
    println!("Mailbox opened");
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

/// Finish writing: a new letter on the stationery (one sheet used) or the
/// rewritten one, kept in the letters until posted (mMl_FONT_SEND).
fn finish_letter(
    ui: &mut MailUi,
    mailbox: &mut Mailbox,
    inventory: &mut crate::interaction::PlayerInventory,
    player: &str,
) {
    let Some(to) = ui.write_recipients.get(ui.write_recipient).cloned() else {
        return;
    };
    let text = ui.write_text.trim_end().to_string();
    ui.writing = false;
    ui.write_text.clear();
    if let Some(index) = ui.rewriting.take() {
        if let Some(letter) = mailbox.letters.get_mut(index) {
            letter.body = text;
            letter.to = Some(to);
        }
    } else if let Some((slot, paper)) = ui.write_paper.take() {
        // A stack of stationery is ITM_PAPER + (count - 1) * 64.
        let sheet = inventory.pockets.pockets[slot];
        inventory.pockets.pockets[slot] = if sheet >= 0x2040 { sheet - 0x40 } else { 0 };
        push_letter(
            mailbox,
            Letter {
                sender: player.to_string(),
                body: text,
                read: true,
                paper,
                to: Some(to),
                ..default()
            },
        );
    }
    crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
    if ui.from_pockets {
        ui.open = false;
    }
}

/// Mail UI keyboard handling: list nav, reading, and composing.
#[allow(clippy::too_many_arguments)]
fn mail_ui_input(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    mut key_events: MessageReader<KeyboardInput>,
    mut ui: ResMut<MailUi>,
    mut mailbox: ResMut<Mailbox>,
    villagers: Query<&Villager>,
    (mut inventory, post, identity): (
        ResMut<crate::interaction::PlayerInventory>,
        Res<crate::post::Post>,
        Res<crate::save::PlayerIdentity>,
    ),
    mut panel_query: Query<&mut Visibility, (With<MailPanel>, Without<MailNotifyText>)>,
    font: Option<Res<crate::msg_window::MsgAssets>>,
    renderer: Option<Res<crate::pockets_disc::PocketAssets>>,
) {
    if !ui.open || *focus != InputFocus::Mail {
        return;
    }

    // --- Reading view ---
    if let Some(idx) = ui.reading {
        if actions.just_pressed(Action::Cancel) || actions.just_pressed(Action::Back) {
            if let Some(letter) = mailbox.letters.get_mut(idx) {
                letter.read = true;
            }
            ui.reading = None;
            ui.page = 0;
            if ui.from_pockets {
                ui.open = false;
            }
            crate::sound::se(crate::sound::se_id::PAGE_OKURI);
        }
        if let Some((letter, font)) = mailbox.letters.get(idx).zip(font.as_deref()) {
            let count = crate::mail_disc::page_count(&letter.body, &font.widths);
            if actions.just_pressed(Action::MoveRight) || actions.just_pressed(Action::Confirm) {
                ui.page = (ui.page + 1).min(count.saturating_sub(1));
            } else if actions.just_pressed(Action::MoveLeft) {
                ui.page = ui.page.saturating_sub(1);
            }
        }
        return;
    }

    // --- Compose view ---
    if ui.writing {
        // Drain typed characters from the key events (printable characters
        // arrive as `Key::Character`; control keys like Enter are handled
        // via `Actions` below).
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
        if ui.write_recipients.is_empty() {
            let mut names: Vec<String> = villagers.iter().map(|v| v.name.to_string()).collect();
            names.sort_unstable();
            ui.write_recipients = names.into_iter().map(Recipient::Villager).collect();
            if post.museum.contacted {
                ui.write_recipients.push(Recipient::Museum);
            }
            // Rewriting keeps the letter's addressee.
            if let Some(to) = ui
                .rewriting
                .and_then(|i| mailbox.letters.get(i))
                .and_then(|l| l.to.clone())
            {
                ui.write_recipient = ui
                    .write_recipients
                    .iter()
                    .position(|r| *r == to)
                    .unwrap_or(0);
            }
        }
        let names = &ui.write_recipients;
        if actions.just_pressed(Action::Cancel) {
            ui.writing = false;
            ui.write_paper = None;
            ui.rewriting = None;
            ui.write_text.clear();
            if ui.from_pockets {
                ui.open = false;
            }
            crate::sound::se(crate::sound::se_id::MENU_EXIT);
            return;
        }
        if !names.is_empty() {
            let count = names.len();
            if actions.just_pressed(Action::MenuUp) {
                let before = ui.write_recipient;
                ui.write_recipient = ui.write_recipient.saturating_sub(1);
                crate::sound::cursor_changed(before, ui.write_recipient);
                return;
            }
            if actions.just_pressed(Action::MenuDown) {
                let before = ui.write_recipient;
                ui.write_recipient = (ui.write_recipient + 1).min(count - 1);
                crate::sound::cursor_changed(before, ui.write_recipient);
                return;
            }
        }
        if actions.just_pressed(Action::Back) {
            if ui.write_text.pop().is_some() {
                crate::sound::se(crate::sound::se_id::CURSOR);
            }
            return;
        }
        if actions.just_pressed(Action::Confirm) {
            finish_letter(&mut ui, &mut mailbox, &mut inventory, &identity.player);
        }
        return;
    }

    // --- List view ---
    if renderer.as_ref().is_some_and(|r| r.hand_busy())
        && !actions.just_pressed(Action::Cancel)
        && !actions.just_pressed(Action::Back)
    {
        actions.consume(Action::Confirm);
        actions.consume(Action::Interact);
        return;
    }
    let slot = ui.selected;
    if ui.letter_actions(&mut actions, &mut mailbox, slot, false, &mut inventory) {
        return;
    }
    if actions.just_pressed(Action::Cancel) {
        ui.open = false;
        // ac_mailbox opens mSM_OVL_INVENTORY in mailbox mode; its exit
        // is NA_SE_31, rather than the separate mailbox overlay's zoom-down.
        crate::sound::se(crate::sound::se_id::MENU_EXIT);
        if let Ok(mut vis) = panel_query.single_mut() {
            *vis = Visibility::Hidden;
        }
        println!("Mailbox closed");
        return;
    }
    // Drain any stray key events so a queued 'n' can't leak into a
    // letter the player starts composing below.
    key_events.clear();
    if font.is_some()
        && (actions.just_pressed(Action::MoveLeft)
            || actions.just_pressed(Action::MoveRight)
            || actions.just_pressed(Action::MoveUp)
            || actions.just_pressed(Action::MoveDown))
    {
        let dx = i32::from(actions.just_pressed(Action::MoveRight))
            - i32::from(actions.just_pressed(Action::MoveLeft));
        let dy = i32::from(actions.just_pressed(Action::MoveDown))
            - i32::from(actions.just_pressed(Action::MoveUp));
        let col = (ui.selected as i32 % 2 + dx).rem_euclid(2);
        let row = (ui.selected as i32 / 2 + dy).rem_euclid(5);
        ui.selected = (row * 2 + col) as usize;
        crate::sound::se(crate::sound::se_id::CURSOR);
        return;
    }
    if actions.just_pressed(Action::MenuUp) {
        let before = ui.selected;
        ui.selected = ui.selected.saturating_sub(1);
        crate::sound::cursor_changed(before, ui.selected);
        return;
    }
    if actions.just_pressed(Action::MenuDown) {
        let before = ui.selected;
        ui.selected = (ui.selected + 1).min(MAILBOX_CAP - 1);
        crate::sound::cursor_changed(before, ui.selected);
    }
}

/// Re-render the mail UI text whenever state changes.
#[allow(clippy::too_many_arguments)]
fn refresh_mail_ui(
    ui: Res<MailUi>,
    mailbox: Res<Mailbox>,
    villagers: Query<&Villager>,
    font: Option<Res<crate::msg_window::MsgAssets>>,
    mut panels: Query<&mut Node, With<MailPanel>>,
    mut list_query: Query<
        (&mut Text, &mut Visibility),
        (
            With<MailListText>,
            Without<MailBodyText>,
            Without<MailHintText>,
        ),
    >,
    mut body_query: Query<
        (&mut Text, &mut Visibility),
        (
            With<MailBodyText>,
            Without<MailListText>,
            Without<MailHintText>,
        ),
    >,
    mut hint_query: Query<
        &mut Text,
        (
            With<MailHintText>,
            Without<MailListText>,
            Without<MailBodyText>,
        ),
    >,
) {
    for mut panel in &mut panels {
        panel.display = if font.is_some() {
            Display::None
        } else {
            Display::Flex
        };
    }
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
        *body_vis = Visibility::Inherited;
        if let Some(letter) = mailbox.letters.get(idx) {
            body_text.0 = format!(
                "From: {}\nSubject: {}\n\n{}",
                letter.sender, letter.subject, letter.body
            );
        }
        hint_text.0 = "[Esc]/[Backspace] back to list".to_string();
    } else if ui.writing {
        *list_vis = Visibility::Hidden;
        *body_vis = Visibility::Inherited;
        let mut names: Vec<String> = villagers.iter().map(|v| v.name.to_string()).collect();
        names.sort_unstable();
        let name = names
            .get(ui.write_recipient)
            .map(|s| s.as_str())
            .unwrap_or("???");
        body_text.0 = format!("To: {name}   (Up/Down to change)\n\n{}_", ui.write_text);
        hint_text.0 = "[Enter] send   [Esc] cancel".to_string();
    } else {
        *list_vis = Visibility::Inherited;
        *body_vis = Visibility::Hidden;
        let mut s = String::new();
        for (i, index) in mailbox.slots().into_iter().enumerate() {
            let marker = if i == ui.selected { ">" } else { " " };
            let Some(index) = index else {
                s.push_str(&format!("{marker} (empty)\n"));
                continue;
            };
            let letter = &mailbox.letters[index];
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
        hint_text.0 = if let Some(choice) = ui.menu {
            format!(
                "{}\n{}",
                if ui.discard {
                    "Throw it out?"
                } else {
                    "Letter"
                },
                ui.action_options()
                    .iter()
                    .enumerate()
                    .map(|(i, s)| format!("{} {s}", if i == choice { ">" } else { " " }))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else if ui.grabbed.is_some() {
            "[Up/Down] move   [Enter] place/swap   [Esc] cancel".into()
        } else {
            "[Up/Down] select   [Enter] actions   [Esc] close".into()
        };
    }
}

pub struct MailPlugin;

impl Plugin for MailPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Mailbox::default())
            .insert_resource(MailUi::default())
            .insert_resource(MailNotifyTimer::default())
            .add_systems(Startup, setup_mail_ui)
            .add_plugins(crate::mail_disc::MailDiscPlugin)
            .add_systems(
                Startup,
                spawn_mailbox.after(crate::house::find_player_house),
            )
            .add_systems(
                Update,
                mailbox_interact
                    .in_set(InteractSet::Objects)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                mail_ui_input.in_set(InteractSet::Ui).in_set(GameplaySet),
            )
            .add_systems(
                Update,
                (refresh_mail_ui, tick_mail_notify).in_set(GameplaySet),
            );
    }
}
