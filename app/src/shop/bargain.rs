//! m_event init_special_event / ac_event_manager init_sp_bargain adapters.
use super::{ShopData, ShopLocation, StockItem};
use bevy::prelude::*;
use rustimal_logic::game_time as gt;
use serde::{Deserialize, Serialize};

type Date = (u16, u8, u8);
fn date(n: crate::clock::Now) -> Date {
    (n.year, n.month, n.day)
}
fn shift(d: Date, days: i32) -> Date {
    let mut t = gt::RtcTime {
        year: d.0,
        month: d.1,
        day: d.2,
        ..default()
    };
    gt::add_days(&mut t, days);
    (t.year, t.month, t.day)
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Offer {
    pub kind: u8,
    pub items: Vec<StockItem>,
    pub announced: bool,
    pub balloon: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Calendar {
    /// Index in native SHOP, DESIGNER, BROKER, ARTIST, CARPET, GYPSY order.
    pub event: u8,
    pub created: Date,
    pub date: Date,
    pub hour: u8,
    /// The field rank the event gap was last rolled with.
    pub rank: u8,
    pub offer: Option<Offer>,
}
impl Default for Calendar {
    fn default() -> Self {
        Self {
            event: 255,
            created: (0, 0, 0),
            date: (0, 0, 0),
            hour: 12,
            rank: 0,
            offer: None,
        }
    }
}
impl Calendar {
    fn valid(&self) -> bool {
        self.event < 6
            && (2000..=2099).contains(&self.date.0)
            && (1..=12).contains(&self.date.1)
            && self.date.2 > 0
            && self.date.2 <= gt::get_days_by_month(self.date.0, self.date.1)
            && self.hour <= 23
            && (self.event != 0 || (12..=19).contains(&self.hour))
            && self
                .offer
                .as_ref()
                .is_none_or(|o| [0, 2, 3, 4].contains(&o.kind) && o.items.len() <= 5)
    }
    pub fn closed(&self, now: crate::clock::Now) -> bool {
        self.valid()
            && self.event == 0
            && self.date == date(now)
            && (now.hour == self.hour - 1 || now.hour == self.hour + 1)
    }
    pub fn active(&self, now: crate::clock::Now) -> bool {
        self.valid()
            && self.event == 0
            && self.date == date(now)
            && now.hour == self.hour
            && self.offer.is_some()
    }
    pub fn bargain_date(&self) -> Option<Date> {
        (self.valid() && self.event == 0).then_some(self.date)
    }
    fn expired(&self, now: crate::clock::Now) -> bool {
        if !self.valid() || date(now) < self.created {
            return true;
        }
        let end = if self.event == 0 {
            self.date
        } else {
            shift(self.date, 1)
        };
        let hour = [23, 5, 17, 5, 5, 20][usize::from(self.event)];
        (date(now), now.hour) > (end, hour)
    }
    fn schedule(&mut self, now: crate::clock::Now, id: u32, renovating: bool) {
        let mut seed = (id & 0xFFFFFF).wrapping_add(1).wrapping_add(
            u32::from(now.year) - u32::from(now.month) + u32::from(now.day) + u32::from(now.hour),
        );
        let mut gap = 1
            + (u32::from(now.day) + u32::from(now.month) * (now.day_secs % 60))
                % u32::from(7 - self.rank.min(6));
        if gap == 1 {
            gap = 2;
        }
        let target = shift(date(now), gap as i32);
        let sale = (
            now.year,
            11,
            gt::weekly_day(now.year, 11, 4, gt::THURSDAY) + 1,
        );
        for _ in 0..12 {
            let mut event = (seed % 6) as u8;
            seed = seed.wrapping_add(1);
            if event == self.event {
                continue;
            }
            let mut target = target;
            if date(now) <= sale && sale <= target {
                event = 2;
                target = sale;
            }
            if event == 0
                && (renovating
                    || target.2 == gt::get_days_by_month(target.0, target.1)
                    || (target.1 == 1 && target.2 <= 3))
            {
                continue;
            }
            if event == 2 && target.1 == 7 && target.2 == 4 {
                continue;
            }
            if event == 5 && target.1 == 12 && target.2 == 31 {
                continue;
            }
            self.event = event;
            self.created = date(now);
            self.date = target;
            self.hour = match event {
                0 => 12 + (seed % 8) as u8,
                2 => 18,
                5 => 21,
                _ => 6,
            };
            self.offer = None;
            info!(
                "Special event scheduled index={event} date={target:?} hour={}",
                self.hour
            );
            return;
        }
    }
}

pub(crate) fn tick(
    mut shop: ResMut<ShopData>,
    location: Res<State<ShopLocation>>,
    disc: Option<Res<crate::game_disc::GameDisc>>,
    identity: Res<crate::save::PlayerIdentity>,
    mut items: Option<ResMut<crate::field_items::FieldItems>>,
    mut rank: ResMut<crate::field_rank::FieldRank>,
    catalog: Res<crate::catalog::Catalog>,
    mut mailbox: ResMut<crate::mail::HouseMailbox>,
) {
    if *location != ShopLocation::Town {
        return;
    }
    let Some(disc) = disc else {
        shop.sale_view = false;
        return;
    };
    let now = crate::clock::now();
    let id = u32::from(identity.id);
    if id == 0 {
        return; // not assigned yet (`save::assign_player_id`)
    }
    if shop.events.expired(now) {
        // `mFAs_SetFieldRank` runs just before the gap is rolled.
        if let Some(items) = items.as_deref_mut() {
            rank.refresh(items);
        }
        if let Some(rank) = rank.rank {
            shop.events.rank = rank;
        }
        let renovating = shop.shop.upgrading_today;
        shop.events.schedule(now, id, renovating);
    }
    if shop.events.event == 0
        && shop.events.valid()
        && date(now) >= shift(shop.events.date, -4)
        && shop.events.offer.is_none()
    {
        let cat_idx = usize::from(shop.events.date.2 % 4);
        let kind = [0, 3, 4, 2][cat_idx];
        let count =
            (1 + now.day_secs as usize % 60 % 3).min([1, 2, 3, 5][shop.shop.shop_level as usize]);
        let mut rng = u64::from(id) ^ now.day_number as u64 ^ 0x91E10DA5C79E7B1D;
        let mut below = |n: u32| {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            (rng % u64::from(n.max(1))) as u32
        };
        if let Some(items) =
            crate::goods::bargain_stock(kind, count, &catalog.collection, &mut below)
        {
            shop.events.offer = Some(Offer {
                kind,
                items: items.into_iter().map(StockItem::new).collect(),
                ..default()
            });
        }
    }
    let start = shop.events.hour;
    let when = shop.events.date;
    let tier = shop.shop.shop_level as usize;
    if let Some(offer) = shop
        .events
        .offer
        .as_mut()
        .filter(|o| !o.announced && mailbox.0.has_room())
    {
        if let Some(text) = disc.text.as_ref() {
            let mut ctx = crate::messages::MsgContext {
                player_name: identity.player.clone(),
                free: vec![String::new(); 20],
                ..default()
            };
            ctx.free[0] = offer.items.len().to_string();
            for (i, item) in offer.items.iter().enumerate() {
                ctx.free[7 + i] = item.name.clone();
            }
            ctx.free[17] = text
                .string(0x66D + usize::from(when.1) - 1)
                .unwrap_or_default();
            ctx.free[18] = text
                .string(0x64E + usize::from(when.2) - 1)
                .unwrap_or_default();
            ctx.free[19] = format!(
                "{} {}",
                if start == 12 { 12 } else { start - 12 },
                text.string(2).unwrap_or_default()
            );
            let cat_idx = match offer.kind {
                0 => 0,
                3 => 1,
                4 => 2,
                _ => 3,
            };
            if let Some(letter) = text.letter(
                rustimal_logic::leaflet::shop_sale_handbill(tier, cat_idx) as usize,
                &ctx,
            ) {
                if mailbox.0.has_room() {
                    mailbox.0.push(crate::mail::Letter {
                        sender: "Nook".into(),
                        header: Some(letter.header),
                        body: letter.body,
                        footer: Some(letter.footer),
                        paper: 55,
                        ..default()
                    });
                    offer.announced = true;
                }
            }
        }
    }
    shop.sale_view = shop.events.active(now);
}
