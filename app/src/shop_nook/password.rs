//! The counter's "Enter a password" service: `aNSC_pc_*` in
//! `ac_npc_shop_common.c`, the `m_passwordChk_ovl` entry window as a typed
//! two-line field, and `m_mail_password_check.c`'s validation. User and
//! popularity-contest codes are redeemed; card-e answers with its message.
//! Famicom, magazine and mini card-e codes are not redeemed yet.
use bevy::prelude::*;
use rustimal_logic::furniture::{ftr_idx_2_item_no, room_item_from_furniture, Direction};
use rustimal_logic::password as pw;

use crate::game_disc::GameDisc;
use crate::goods::{self, Kind};
use crate::items;

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

mod ui;
pub(super) use ui::Receiver;
pub(crate) use ui::{draw, input, PasswordAssets, PasswordUi};

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
