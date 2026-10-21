//! The player's catalog side of the shop (`Private_c`): the collection
//! record of everything held (mPr_SetItemCollectBit), Nook's lottery
//! tickets that did not fit in the pockets (lotto_ticket_mail_storage), and
//! catalog orders (catalog_orders[5]).
//!
//! The collection bit is set whenever an item sits in the pockets in normal
//! condition (every mPr_SetPossessionItem with mPr_ITEM_COND_NORMAL, and a
//! present once opened). Tickets that did not fit, then catalog orders,
//! arrive at the start of play (mPO_first_work ->
//! mPO_delivery_one_address_special_mail) as the disc's letters: 0x057 with
//! the tickets (stacks of five; FREE_STR4/5 the month and its last day),
//! 0x049 + shop level with the ordered item (FREE_STR0 its name), on
//! stationery 55, into the house mailbox while it has room.
use bevy::prelude::*;
use rustimal_logic::collection::ItemCollection;
use rustimal_logic::special_delivery::{self as sd, CatalogOrders, TicketOverflow};
use serde::{Deserialize, Serialize};

use crate::game_disc::GameDisc;
use crate::mail::{HouseMailbox, Letter};

/// ITM_PAPER55 - ITM_PAPER_START.
const DELIVERY_PAPER: u8 = 55;
/// mString_MONTH_START / mString_DAY_START.
const MONTH_STRINGS: usize = 0x66D;
const DAY_STRINGS: usize = 0x64E;

#[derive(Resource, Default)]
pub struct Catalog {
    pub collection: ItemCollection,
    pub tickets: TicketOverflow,
    pub orders: CatalogOrders,
    /// Shop_c.lottery_items: this month's raffle prizes, and the month.
    pub lottery: [u16; 3],
    pub lottery_month: u8,
}

/// The catalog state in the save.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CatalogSave {
    pub furniture: Vec<u32>,
    pub wall: Vec<u32>,
    pub carpet: Vec<u32>,
    pub paper: Vec<u32>,
    pub music: Vec<u32>,
    /// lotto_ticket_expiry_month, lotto_ticket_mail_storage.
    pub tickets: (u8, u8),
    /// (item, shop level) per order slot; 0 empty.
    pub orders: Vec<(u16, u8)>,
    /// The raffle prizes and their month.
    #[serde(default)]
    pub lottery: [u16; 3],
    #[serde(default)]
    pub lottery_month: u8,
}

impl Catalog {
    pub fn save(&self) -> CatalogSave {
        let c = &self.collection;
        CatalogSave {
            furniture: c.furniture.to_vec(),
            wall: c.wall.to_vec(),
            carpet: c.carpet.to_vec(),
            paper: c.paper.to_vec(),
            music: c.music.to_vec(),
            tickets: (self.tickets.expiry_month, self.tickets.mail_storage),
            orders: self
                .orders
                .slots
                .iter()
                .map(|o| (o.item, o.shop_level))
                .collect(),
            lottery: self.lottery,
            lottery_month: self.lottery_month,
        }
    }

    pub fn restore(&mut self, saved: Option<&CatalogSave>) {
        *self = Self::default();
        let Some(s) = saved else { return };
        let fill = |dst: &mut [u32], src: &[u32]| {
            for (d, s) in dst.iter_mut().zip(src) {
                *d = *s;
            }
        };
        let c = &mut self.collection;
        fill(&mut c.furniture, &s.furniture);
        fill(&mut c.wall, &s.wall);
        fill(&mut c.carpet, &s.carpet);
        fill(&mut c.paper, &s.paper);
        fill(&mut c.music, &s.music);
        self.tickets.expiry_month = s.tickets.0;
        self.tickets.mail_storage = s.tickets.1;
        self.lottery = s.lottery;
        self.lottery_month = s.lottery_month;
        for (slot, &(item, shop_level)) in self.orders.slots.iter_mut().zip(&s.orders) {
            slot.item = item;
            slot.shop_level = shop_level;
        }
    }

    /// aNSC_setup_ticket_remain: one more ticket waiting for the mail
    /// (reset on a new month; at most 255).
    pub fn ticket_by_mail(&mut self, month: u8) {
        self.tickets.add_ticket(month);
    }
}

/// aNSC_check_item_with_ticket: furniture, clothing, wallpaper, carpets and
/// shop umbrellas come with a ticket.
pub fn comes_with_ticket(item: u16) -> bool {
    match item >> 12 {
        1 | 3 => true,
        2 => matches!((item >> 8) & 0xF, 4 | 6 | 7) || (0x2204..0x222C).contains(&item),
        _ => false,
    }
}

/// aNSC_check_same_month_ticket: onto a stack of this month's tickets
/// (fewer than five) or into an empty slot; false when neither fits.
pub fn give_ticket(pockets: &mut [u16], month: u8) -> bool {
    for slot in pockets.iter_mut() {
        if sd::is_ticket_item(*slot)
            && sd::ticket_month(*slot) == month
            && sd::ticket_count(*slot) < 5
        {
            *slot = sd::ticket_get_item(month, sd::ticket_count(*slot) + 1);
            return true;
        }
    }
    if let Some(slot) = pockets.iter_mut().find(|s| **s == 0) {
        *slot = sd::ticket_get_item(month, 1);
        return true;
    }
    false
}

fn load(loaded: Res<crate::save::LoadedSave>, mut catalog: ResMut<Catalog>) {
    catalog.restore(loaded.0.as_ref().and_then(|s| s.catalog.as_ref()));
}

/// mPr_SetPossessionItem's collect bit for what the pockets hold.
fn collect(inventory: Res<crate::interaction::PlayerInventory>, mut catalog: ResMut<Catalog>) {
    if !inventory.is_changed() {
        return;
    }
    for (slot, &item) in inventory.pockets.pockets.iter().enumerate() {
        if item != 0 && inventory.pockets.item_cond(slot) == 0 && !catalog.collection.has(item) {
            catalog.collection.set(item);
        }
    }
}

/// mPO_first_delivery_proc: once, at the start of play.
fn first_delivery(
    disc: Option<Res<GameDisc>>,
    mut catalog: ResMut<Catalog>,
    mut house_box: ResMut<HouseMailbox>,
    identity: Res<crate::save::PlayerIdentity>,
    mut done: Local<bool>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *done {
        return;
    }
    *done = true;
    let mut model = sd::Mailbox::default();
    let used = house_box.0.letters.len().min(model.slots.len());
    for slot in model.slots.iter_mut().take(used) {
        *slot = Some(sd::SpecialMail {
            template: 0,
            present: 0,
        });
    }
    let month = catalog.tickets.expiry_month;
    let (tickets, orders) = {
        let c = &mut *catalog;
        (&mut c.tickets, &mut c.orders)
    };
    let outcome = sd::deliver_special_mail(tickets, orders, &mut model);
    if outcome.ticket_mails == 0 && outcome.order_mails == 0 {
        return;
    }
    let text = disc.text.as_ref().expect("checked above");
    let now = crate::clock::now();
    for mail in model.slots.iter().skip(used).flatten() {
        let mut ctx = crate::messages::MsgContext {
            player_name: identity.player.clone(),
            free: vec![String::new(); 20],
            items: vec![String::new(); 5],
            item_articles: vec![0; 5],
            ..default()
        };
        if sd::is_ticket_item(mail.present) {
            let m = month.clamp(1, 12);
            let days = days_in_month(now.year, m);
            ctx.free[4] = text
                .string(MONTH_STRINGS + m as usize - 1)
                .unwrap_or_default();
            ctx.free[5] = text
                .string(DAY_STRINGS + days as usize - 1)
                .unwrap_or_default();
        } else {
            ctx.free[0] = crate::items::name(mail.present);
        }
        let Some(t) = text.letter(mail.template as usize, &ctx) else {
            continue;
        };
        house_box.0.push(Letter {
            sender: "Nook".into(),
            header: Some(t.header),
            body: t.body,
            footer: Some(t.footer),
            paper: DELIVERY_PAPER,
            present: mail.present,
            ..default()
        });
        info!(
            "special delivery: mail {:#05x} with {:#06x}",
            mail.template, mail.present
        );
    }
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        2 if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

pub struct CatalogPlugin;

impl Plugin for CatalogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Catalog>()
            .add_systems(Startup, load)
            .add_systems(
                Update,
                (collect, first_delivery).in_set(crate::states::GameplaySet),
            );
    }
}
