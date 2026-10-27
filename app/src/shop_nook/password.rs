//! The counter's "Enter a password" service: `aNSC_pc_*` in
//! `ac_npc_shop_common.c`, the `m_passwordChk_ovl` entry window as a typed
//! two-line field, and `m_mail_password_check.c`'s validation. User and
//! popularity-contest codes are redeemed; card-e answers with its message.
//! Famicom, magazine and mini card-e codes are not redeemed yet.
use bevy::camera::visibility::RenderLayers;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use rustimal_logic::furniture::{ftr_idx_2_item_no, room_item_from_furniture, Direction};
use rustimal_logic::password as pw;

use crate::game_disc::GameDisc;
use crate::goods::{self, Kind};
use crate::input::{Action, Actions, InputFocus};
use crate::items;
use crate::msg_window::{quad_mesh, MsgAssets};
use crate::pockets_disc::Paint;

/// `aNSC_MSG_PSWD_*` (`ac_npc_shop_common.h`), in the shop's own numbering.
pub(super) mod msg {
    pub const NO_ITEM: usize = 300;
    pub const INFO: usize = 301;
    pub const SAME_NAME: usize = 302;
    pub const CONFIRM: usize = 303;
    pub const CANCEL: usize = 305;
    pub const CHECK: usize = 306;
    pub const INVENTORY_FULL: usize = 311;
    pub const SAY: usize = 312;
    pub const WRONG: usize = 313;
    pub const DECLINE: usize = 314;
    pub const GOOD_FAMICON: usize = 315;
    pub const GOOD_NPC: usize = 316;
    pub const GOOD_MAGAZINE_1: usize = 317;
    pub const CARDE: usize = 318;
    pub const GOOD_USER: usize = 319;
    pub const GOOD_CARDEM: usize = 320;
    pub const GOOD_MAGAZINE_2: usize = 322;
    pub const GIFT_OUT: usize = 323;
    pub const BAD: usize = 324;
}

/// Presents Nook hands over before he refuses more
/// (`Common_Get(unk_nook_present_count) >= 3`).
pub(super) const PRESENT_LIMIT: u32 = 3;
pub(super) const PRESENT_CONDITION: u32 = 1;
const NPC_NUM: i32 = 236;
const SPECIAL_NPC_NUM: i32 = 32;
const RSV_NO: u16 = 0xFFFF;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Mode {
    /// The 28-letter code to redeem (`m_passwordChk_ovl`).
    #[default]
    Code,
    /// Who a new code is for: town, then player (`m_passwordMake_ovl`).
    Receiver,
}

/// A receiver's town and player name, space padded to eight bytes each.
pub(super) type Receiver = ([u8; 8], [u8; 8]);

/// The entry window and what the counter remembers between questions.
#[derive(Resource)]
pub(crate) struct PasswordUi {
    pub open: bool,
    mode: Mode,
    town: [u8; 8],
    name: [u8; 8],
    lens: [usize; 2],
    field: usize,
    /// Set when the receiver window closes: the names, or `None` when
    /// the player backed out.
    pub receiver: Option<Option<Receiver>>,
    lines: [[u8; 14]; 2],
    selected: usize,
    cursor: usize,
    /// Set when the window closes: all 28 characters, spaces when unfinished.
    pub done: Option<[u8; 28]>,
    /// `shop_common->password_str`, kept so a retry starts from it.
    text: [u8; 28],
    /// Presents handed over since the game started.
    pub presents: u32,
    rng: u32,
    just_opened: bool,
    revision: u64,
}

impl Default for PasswordUi {
    fn default() -> Self {
        Self {
            open: false,
            mode: Mode::Code,
            town: [pw::PW_CHAR_SPACE; 8],
            name: [pw::PW_CHAR_SPACE; 8],
            lens: [0; 2],
            field: 0,
            receiver: None,
            lines: [[pw::PW_CHAR_SPACE; 14]; 2],
            selected: 0,
            cursor: 0,
            done: None,
            text: [pw::PW_CHAR_SPACE; 28],
            presents: 0,
            rng: 0,
            just_opened: false,
            revision: 0,
        }
    }
}

impl PasswordUi {
    /// `RANDOM_F(100.0)` for a magazine code. The port keeps its own
    /// generator, not the original's global state.
    pub(super) fn next_roll(&mut self) -> f32 {
        if self.rng == 0 {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(1, |d| d.subsec_nanos());
            self.rng = nanos | 1;
        }
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng >> 8) as f32 / (1u32 << 24) as f32 * 100.0
    }

    /// `aNSC_pc_input_pw_start_wait_init`: a fresh question clears the text.
    pub(super) fn clear(&mut self) {
        self.text = [pw::PW_CHAR_SPACE; 28];
    }

    /// `mSM_OVL_PASSWORDMAKE`: empty town and player name fields.
    pub(super) fn begin_receiver(&mut self) {
        self.mode = Mode::Receiver;
        self.town = [pw::PW_CHAR_SPACE; 8];
        self.name = [pw::PW_CHAR_SPACE; 8];
        self.lens = [0; 2];
        self.field = 0;
        self.open = true;
        self.receiver = None;
        self.just_opened = true;
        self.revision += 1;
    }

    fn finish_receiver(&mut self, accepted: bool) {
        self.receiver = Some(accepted.then_some((self.town, self.name)));
        self.open = false;
        self.revision += 1;
    }

    /// `mSM_OVL_PASSWORDCHK`: both lines start from the remembered text.
    pub(super) fn begin(&mut self) {
        self.mode = Mode::Code;
        self.lines[0].copy_from_slice(&self.text[..14]);
        self.lines[1].copy_from_slice(&self.text[14..]);
        let filled = self
            .text
            .iter()
            .position(|&c| c == pw::PW_CHAR_SPACE)
            .unwrap_or(28);
        self.selected = usize::from(filled >= 14);
        self.cursor = if filled == 28 { 14 } else { filled % 14 };
        self.open = true;
        self.done = None;
        self.just_opened = true;
        self.revision += 1;
    }

    fn finish(&mut self) {
        self.text[..14].copy_from_slice(&self.lines[0]);
        self.text[14..].copy_from_slice(&self.lines[1]);
        self.done = Some(self.text);
        self.open = false;
        self.revision += 1;
    }

    /// Closes the window without a result, as leaving the scene does.
    pub(super) fn close(&mut self) {
        if self.open || self.done.is_some() || self.receiver.is_some() {
            self.open = false;
            self.done = None;
            self.receiver = None;
            self.revision += 1;
        }
    }
}

/// What `aNSC_pc_check_password` decided.
pub(super) enum Outcome {
    /// The code was left unfinished (`aNSC_PSW_RES_9`).
    Decline,
    /// `aNSC_PSW_RES_0`: not a code the counter takes.
    Wrong,
    /// A real code made for another player or town.
    Bad,
    /// A card-e code: Nook explains and gives nothing.
    CardE,
    /// A valid code: the message that announces it, and the item.
    Present { message: usize, item: u16 },
    /// A magazine code that did not win (`aNSC_PSW_GOOD_MAGAZINE_2`).
    Lost,
}

/// A decoded code's recipient and item for the message fields.
pub(super) struct Decoded {
    pub town: String,
    pub player: String,
    pub item: u16,
}

fn name_bytes(name: &str) -> [u8; 8] {
    let mut out = [pw::PW_CHAR_SPACE; 8];
    for (slot, ch) in out.iter_mut().zip(name.chars()) {
        *slot = crate::messages::ac_code_opt(ch).unwrap_or(pw::PW_CHAR_SPACE);
    }
    out
}

/// A text name as eight space-padded game bytes.
pub(super) fn padded(name: &str) -> [u8; 8] {
    name_bytes(name)
}

/// The text of an eight-byte name field.
pub(super) fn field_text(bytes: &[u8; 8]) -> String {
    name_text(bytes)
}

/// `aNSC_check_possession_item_make_password`: pocket items Nook can trade
/// for a password (plain condition, something he sells).
pub(super) fn tradeable(pockets: &rustimal_logic::inventory::Inventory) -> bool {
    pockets
        .pockets
        .iter()
        .enumerate()
        .any(|(slot, &item)| item != 0 && pockets.item_cond(slot) == 0 && present_user(item))
}

/// `mMpswd_make_password` for a user code: the two code lines as game
/// characters, ready for the message fields.
pub(super) fn make_user_code(town: &[u8; 8], name: &[u8; 8], item: u16) -> [u8; 28] {
    pw::make_password(pw::PW_TYPE_USER, 1, name, town, item, 0, 0)
}

/// Fills the message fields a code's two lines use (`aNSC_set_pw_password_str`).
pub(super) fn fill_code_lines(ctx: &mut crate::messages::MsgContext, code: &[u8; 28]) {
    let line = |bytes: &[u8]| -> String {
        bytes
            .iter()
            .map(|&b| crate::messages::char_of(b).unwrap_or('?'))
            .collect()
    };
    ctx.items[3] = line(&code[..14]);
    ctx.items[4] = line(&code[14..]);
}

fn name_text(bytes: &[u8; 8]) -> String {
    bytes
        .iter()
        .map(|&b| crate::messages::char_of(b).unwrap_or('?'))
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// `mMpswd_check_name`: both names are this player's and this town's.
fn names_match(fields: &pw::PwFields, player: &str, town: &str) -> bool {
    fields.str0 == name_bytes(player) && fields.str1 == name_bytes(town)
}

/// `mMpswd_check_npc_code`.
fn npc_code_ok(fields: &pw::PwFields) -> bool {
    (fields.npc_type == 0 && fields.npc_code < NPC_NUM)
        || (fields.npc_type == 1 && fields.npc_code < SPECIAL_NPC_NUM)
}

/// `aNSC_pc_check_password`, for the codes this port redeems.
pub(super) fn check(
    disc: &GameDisc,
    text: &[u8; 28],
    player: &str,
    town: &str,
    roll: f32,
) -> (Outcome, Option<Decoded>) {
    if text[27] == pw::PW_CHAR_SPACE {
        return (Outcome::Decline, None);
    }
    let Some(data) = pw::decode_code(text) else {
        return (Outcome::Wrong, None);
    };
    let fields = pw::unpack(&data);
    if !fields.checksum_ok() || !present_ok(disc, &fields) {
        return (Outcome::Wrong, None);
    }
    let decoded = Decoded {
        town: name_text(&fields.str1),
        player: name_text(&fields.str0),
        item: fields.item,
    };
    let item = fields.item;
    // The name-checked kinds answer Bad for someone else's code.
    let named = |message: usize| {
        if names_match(&fields, player, town) {
            Outcome::Present { message, item }
        } else {
            Outcome::Bad
        }
    };
    let outcome = match fields.type_ {
        pw::PW_TYPE_FAMICOM if fields.hit_rate_index == 1 => named(msg::GOOD_FAMICON),
        pw::PW_TYPE_POPULAR if fields.hit_rate_index == 1 && npc_code_ok(&fields) => {
            named(msg::GOOD_NPC)
        }
        pw::PW_TYPE_USER if fields.hit_rate_index == 1 => named(msg::GOOD_USER),
        pw::PW_TYPE_CARD_E => Outcome::CardE,
        pw::PW_TYPE_CARD_E_MINI if fields.hit_rate_index == 1 => Outcome::Present {
            message: msg::GOOD_CARDEM,
            item,
        },
        // A magazine code wins with the table's chance for its rate.
        pw::PW_TYPE_MAGAZINE => match usize::try_from(fields.hit_rate_index)
            .ok()
            .and_then(|rate| pw::PW_MAGAZINE_HIT_RATE.get(rate))
        {
            Some(&chance) if roll < chance => Outcome::Present {
                message: msg::GOOD_MAGAZINE_1,
                item,
            },
            Some(_) => Outcome::Lost,
            None => Outcome::Wrong,
        },
        _ => Outcome::Wrong,
    };
    (outcome, Some(decoded))
}

/// `mMpswd_check_present`. A reserved item is refused rather than handed over.
fn present_ok(disc: &GameDisc, fields: &pw::PwFields) -> bool {
    match fields.type_ {
        pw::PW_TYPE_FAMICOM => famicom_item(fields.item),
        pw::PW_TYPE_USER => present_user(fields.item),
        _ => fields.item != RSV_NO && present_other(disc, fields.item),
    }
}

/// `pswd_famicom_list`: the fifteen Famicom furniture indices, south facing
/// (`mMpswd_check_present_famicom`).
const FAMICOM_LIST: [u16; 15] = [
    0x36a, 0x36b, 0x36c, 0x36d, 0x36e, 0x36f, 0x370, 0x377, 0x371, 0x372, 0x373, 0x374, 0x375,
    0x376, 0x378,
];

fn famicom_item(item: u16) -> bool {
    FAMICOM_LIST
        .iter()
        .any(|&index| ftr_idx_2_item_no(index, Direction::South) == item)
}

fn listed(kind: Kind, item: u16, lists: &[usize]) -> bool {
    lists
        .iter()
        .any(|&index| goods::list(kind, index).contains(&item))
}

/// Lists A-C: the common, uncommon and rare lists together.
const ABC: [usize; 3] = [0, 1, 2];
/// A-C, event, train, lottery and the event present chumon list.
const FURNITURE_LISTS: [usize; 7] = [0, 1, 2, 3, 4, 5, 18];

fn price_of(item: u16) -> bool {
    items::price(item).is_some_and(|price| price != 0)
}

/// An item1 the shop trades (`mMpswd_check_present_user`'s item1 branch).
fn item1_sold(item: u16) -> bool {
    if (0x2400..0x24FF).contains(&item) {
        return listed(Kind::Cloth, item, &ABC) && price_of(item);
    }
    if (0x2204..=0x222B).contains(&item)
        || (0x2244..=0x224B).contains(&item)
        || (0x2B00..=0x2B0F).contains(&item)
        || (0x2254..=0x225B).contains(&item)
        || (0x224C..=0x2253).contains(&item)
        || (0x2239..=0x223C).contains(&item)
        || (0x2200..=0x2203).contains(&item)
    {
        return price_of(item);
    }
    match (item >> 8) & 0xF {
        // Fruit.
        8 => price_of(item),
        // Stationery: the full stack of four sets the price.
        0 => {
            let stack = 0x20C0 + (item & 0x3F);
            listed(Kind::Paper, stack, &ABC) && price_of(stack)
        }
        7 => listed(Kind::Wallpaper, item, &ABC) && price_of(item),
        6 => listed(Kind::Carpet, item, &ABC) && price_of(item),
        _ => false,
    }
}

/// `mMpswd_check_present_user`: something Nook sells, at a price.
pub(crate) fn present_user(item: u16) -> bool {
    match item >> 12 {
        1 | 3 => match room_item_from_furniture(item) {
            Some(item1) => item1_sold(item1),
            None => listed(Kind::Furniture, item, &FURNITURE_LISTS) && price_of(item),
        },
        2 => item1_sold(item),
        _ => false,
    }
}

/// `mMpswd_check_present_other`: what other code types may carry.
fn present_other(disc: &GameDisc, item: u16) -> bool {
    match item >> 12 {
        1 | 3 => {
            let Some(index) = rustimal_logic::collection::furniture_idx(item) else {
                return false;
            };
            let kind = disc
                .catalog
                .get("mRmTp_birth_type")
                .and_then(|a| disc.rom.slice(disc::RomSource::Rel, a.offset, a.size))
                .and_then(|table| table.get(index).copied());
            // Clothing, umbrellas, bugs, fish, unobtainable, e-reader,
            // personal designs, Famicom and diaries are not obtainable.
            kind.is_some_and(|kind| !matches!(kind, 8..=11 | 16 | 24 | 27 | 28 | 31))
        }
        2 => match (item >> 8) & 0xF {
            2 => {
                (0x2204..=0x2223).contains(&item)
                    || (0x2244..=0x224B).contains(&item)
                    || (0x2254..=0x225B).contains(&item)
                    || (0x224C..=0x2253).contains(&item)
            }
            0 => item >= 0x20C0,
            3 | 0xD => false,
            _ => true,
        },
        _ => false,
    }
}

/// The message fields `aNSC_set_pw_password_str` and `aNSC_set_pw_info_str`
/// fill: the two code lines, the recipient's town and name, and the item.
pub(super) fn fill_context(
    disc: &GameDisc,
    ctx: &mut crate::messages::MsgContext,
    text: &[u8; 28],
    decoded: &Decoded,
) {
    let line = |bytes: &[u8]| -> String {
        bytes
            .iter()
            .map(|&b| crate::messages::char_of(b).unwrap_or('?'))
            .collect()
    };
    ctx.items[3] = line(&text[..14]);
    ctx.items[4] = line(&text[14..]);
    ctx.free[6] = decoded.town.clone();
    ctx.free[7] = decoded.player.clone();
    ctx.items[2] = items::name(decoded.item);
    ctx.item_articles[2] = super::item_article(disc, decoded.item);
}

/// Types the code into the two fixed-width lines (`mED_input_pw_chk_line`).
pub(crate) fn input(
    mut ui: ResMut<PasswordUi>,
    mut actions: ResMut<Actions>,
    mut events: MessageReader<KeyboardInput>,
    focus: Res<InputFocus>,
) {
    if !ui.open || *focus != InputFocus::Password {
        events.clear();
        return;
    }
    if ui.just_opened {
        events.clear();
        ui.just_opened = false;
        return;
    }
    if ui.mode == Mode::Receiver {
        receiver_input(&mut ui, &mut actions, &mut events);
        return;
    }
    let mut changed = false;
    for event in events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match event.key_code {
            KeyCode::Escape => {
                // Cancelling closes with whatever is typed; unfinished text
                // reaches the counter as a declined code.
                ui.finish();
                break;
            }
            KeyCode::Backspace => {
                let state = &mut *ui;
                let (first, second) = state.lines.split_at_mut(1);
                pw::backspace(
                    &mut first[0],
                    &mut second[0],
                    &mut state.selected,
                    &mut state.cursor,
                );
                changed = true;
                continue;
            }
            _ => {}
        }
        if let Key::Character(text) = &event.logical_key {
            for ch in text.chars() {
                let Some(code) = typed_code(ch) else {
                    continue;
                };
                let state = &mut *ui;
                let selected = state.selected;
                let mut cursor = state.cursor;
                let result = pw::line_insert(&mut state.lines[selected], &mut cursor, code);
                state.cursor = cursor;
                changed |= result != pw::LineInsert::Rejected;
                if result == pw::LineInsert::LineFull && selected == 0 {
                    state.selected = 1;
                    state.cursor = 0;
                }
            }
        }
    }
    if ui.open && changed {
        ui.revision += 1;
        if pw::needs_charset_check(&ui.lines[1]) && pw::all_usable(&ui.lines[0], &ui.lines[1]) {
            ui.finish();
        }
    }
    // Typed keys also map to gameplay actions; none may leak through.
    for action in [
        Action::Confirm,
        Action::Cancel,
        Action::Back,
        Action::Interact,
        Action::Pause,
    ] {
        actions.consume(action);
    }
}

/// The receiver window: type the town, Tab to the player name, Enter to
/// finish (the retail Start button) once the town is not empty.
fn receiver_input(
    ui: &mut PasswordUi,
    actions: &mut Actions,
    events: &mut MessageReader<KeyboardInput>,
) {
    for event in events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        let field = ui.field;
        match event.key_code {
            KeyCode::Escape => {
                ui.finish_receiver(false);
                break;
            }
            KeyCode::Enter | KeyCode::NumpadEnter => {
                if ui.lens[0] > 0 {
                    ui.finish_receiver(true);
                    break;
                }
                continue;
            }
            KeyCode::Tab => {
                ui.field = 1 - field;
                ui.revision += 1;
                continue;
            }
            KeyCode::Backspace => {
                if ui.lens[field] > 0 {
                    ui.lens[field] -= 1;
                    let at = ui.lens[field];
                    let text = if field == 0 {
                        &mut ui.town
                    } else {
                        &mut ui.name
                    };
                    text[at] = pw::PW_CHAR_SPACE;
                    ui.revision += 1;
                }
                continue;
            }
            _ => {}
        }
        let typed: Vec<char> = match &event.logical_key {
            Key::Character(text) => text.chars().collect(),
            Key::Space => vec![' '],
            _ => continue,
        };
        for ch in typed {
            let code = (ch.is_ascii_graphic() || ch == ' ')
                .then(|| crate::messages::ac_code_opt(ch))
                .flatten();
            let field = ui.field;
            if let Some(code) = code.filter(|_| ui.lens[field] < 8) {
                let at = ui.lens[field];
                let text = if field == 0 {
                    &mut ui.town
                } else {
                    &mut ui.name
                };
                text[at] = code;
                ui.lens[field] += 1;
                ui.revision += 1;
            }
        }
    }
    // Typed keys also map to gameplay actions; none may leak through.
    for action in [
        Action::Confirm,
        Action::Cancel,
        Action::Back,
        Action::Interact,
        Action::Pause,
    ] {
        actions.consume(action);
    }
}

/// A typed character as a password character: letters, `%`, `&`, `@` and
/// digits (0 and 1 read as `O` and `l`, as the check does) and `#` for the
/// retail grid's 0xD1 symbol.
fn typed_code(ch: char) -> Option<u8> {
    let code = match ch {
        '0' => b'O',
        '1' => b'l',
        // The retail grid's 0xD1 symbol prints as a hash.
        '#' => 0xD1,
        c if c.is_ascii_alphanumeric() || matches!(c, '%' | '&' | '@') => c as u8,
        _ => return None,
    };
    pw::usable_letter(code).then_some(code)
}

#[derive(Resource, Default)]
pub(crate) struct PasswordAssets {
    root: Option<Entity>,
    shown: Option<u64>,
    white: Option<Handle<Image>>,
}

/// The entry window: a plain panel, the remembered text in 14-letter rows.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw(
    mut commands: Commands,
    ui: Res<PasswordUi>,
    font: Option<Res<MsgAssets>>,
    mut assets: ResMut<PasswordAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !ui.open {
        if let Some(root) = assets.root.take() {
            commands.entity(root).despawn();
        }
        assets.shown = None;
        return;
    }
    let Some(font) = font else { return };
    if assets.shown == Some(ui.revision) && assets.root.is_some() {
        return;
    }
    if let Some(root) = assets.root.take() {
        commands.entity(root).despawn();
    }
    let white = assets
        .white
        .get_or_insert_with(|| {
            images.add(Image::new_fill(
                bevy::render::render_resource::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                bevy::render::render_resource::TextureDimension::D2,
                &[255, 255, 255, 255],
                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                bevy::asset::RenderAssetUsages::RENDER_WORLD,
            ))
        })
        .clone();
    let root = commands
        .spawn((
            Name::new("password entry"),
            Transform::default(),
            Visibility::default(),
            RenderLayers::layer(1),
        ))
        .id();
    assets.root = Some(root);
    assets.shown = Some(ui.revision);
    let mut paint = Paint {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        root,
    };
    paint.put(
        quad_mesh(Vec2::new(48.0, 64.0), Vec2::new(224.0, 110.0)),
        white,
        Color::srgba(1.0, 0.97, 0.88, 0.96),
        200.0,
    );
    let ink = [70, 40, 50, 255];
    let small = format!(
        "{}{}",
        crate::messages::markup::LINE_SCALE,
        crate::messages::markup::arg(16)
    );
    if ui.mode == Mode::Receiver {
        paint.text(
            &font,
            "Enter a receiver.",
            Vec2::new(60.0, 72.0),
            ink,
            230.0,
        );
        for (row, (label, text)) in [("Town", &ui.town), ("Player", &ui.name)]
            .into_iter()
            .enumerate()
        {
            let y = 96.0 + row as f32 * 22.0;
            let active = ui.field == row;
            let color = if active { [200, 60, 60, 255] } else { ink };
            paint.text(
                &font,
                &format!("{small}{label}"),
                Vec2::new(60.0, y + 2.0),
                color,
                230.0,
            );
            for (column, &code) in text.iter().enumerate() {
                let here = active && column == ui.lens[row].min(7);
                let shown = if column < ui.lens[row] {
                    crate::messages::char_of(code).unwrap_or('?').to_string()
                } else if here {
                    "_".into()
                } else {
                    ".".into()
                };
                paint.text(
                    &font,
                    &shown,
                    Vec2::new(116.0 + column as f32 * 14.5, y),
                    color,
                    230.0,
                );
            }
        }
        paint.text(
            &font,
            &format!("{small}Type the names   Tab: switch   Enter: finish"),
            Vec2::new(60.0, 148.0),
            ink,
            230.0,
        );
        paint.text(
            &font,
            &format!("{small}Esc: back"),
            Vec2::new(60.0, 160.0),
            ink,
            230.0,
        );
        return;
    }
    paint.text(&font, "Password", Vec2::new(60.0, 72.0), ink, 230.0);
    for (row, line) in ui.lines.iter().enumerate() {
        for (column, &code) in line.iter().enumerate() {
            let here = row == ui.selected && column == ui.cursor;
            let shown = if code == pw::PW_CHAR_SPACE {
                if here {
                    '_'.to_string()
                } else {
                    '.'.to_string()
                }
            } else {
                crate::messages::char_of(code).unwrap_or('?').to_string()
            };
            let color = if here { [200, 60, 60, 255] } else { ink };
            paint.text(
                &font,
                &shown,
                Vec2::new(60.0 + column as f32 * 14.5, 96.0 + row as f32 * 22.0),
                color,
                230.0,
            );
        }
    }
    paint.text(
        &font,
        &format!("{small}Type the 28 letters   Backspace: erase"),
        Vec2::new(60.0, 148.0),
        ink,
        230.0,
    );
    paint.text(
        &font,
        &format!("{small}Esc: cancel"),
        Vec2::new(60.0, 160.0),
        ink,
        230.0,
    );
}
