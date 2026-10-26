//! aSL_JudgeRenewShop/RenewShop and ordinary mSP_ShopOpen calendar policy.
use bevy::prelude::*;
use rustimal_logic::{game_time as gt, shop::ShopState};
use serde::{Deserialize, Serialize};

use super::{ShopData, ShopLocation};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Progress {
    pub visitor: bool,
    pub pending: bool,
    pub renewal: (u16, u8, u8),
    pub notice: bool,
}

impl From<&ShopState> for Progress {
    fn from(s: &ShopState) -> Self {
        Self {
            visitor: s.visitor_flag,
            pending: s.upgrading_today,
            renewal: s.renewal_ymd,
            notice: s.send_upgrade_notice,
        }
    }
}

fn date(now: crate::clock::Now) -> (u16, u8, u8) {
    (now.year, now.month, now.day)
}

fn shift(d: (u16, u8, u8), days: i32) -> (u16, u8, u8) {
    let mut t = gt::RtcTime {
        year: d.0,
        month: d.1,
        day: d.2,
        ..default()
    };
    gt::add_days(&mut t, days);
    (t.year, t.month, t.day)
}

fn excluded(d: (u16, u8, u8)) -> bool {
    d.2 == gt::get_days_by_month(d.0, d.1)
        || (d.1 == 11 && d.2 == gt::weekly_day(d.0, 11, 4, gt::THURSDAY) + 1)
}

pub(crate) fn in_renovation(s: &ShopState, now: crate::clock::Now) -> bool {
    s.upgrading_today && date(now) >= shift(s.renewal_ymd, -1)
}

pub(crate) fn is_open(s: &ShopState, now: crate::clock::Now) -> bool {
    !in_renovation(s, now)
        && if now.day == gt::get_days_by_month(now.year, now.month) {
            now.hour >= 10 && now.hour < if s.shop_level as u8 == 1 { 23 } else { 22 }
        } else {
            s.shop_level.normally_open(now.hour)
        }
}

/// Called in town only: no remodel beneath an active shop transaction.
pub(crate) fn advance(s: &mut ShopState, now: crate::clock::Now) -> bool {
    let today = date(now);
    if s.upgrading_today && today < shift(s.renewal_ymd, -2) {
        s.upgrading_today = false;
        s.send_upgrade_notice = false;
    }
    if s.renewal_due(today) {
        let notice = s.send_upgrade_notice;
        let changed = s.complete_renewal(today);
        s.send_upgrade_notice = notice;
        return changed;
    }
    if !s.upgrading_today && !(0..=2).any(|n| excluded(shift(today, n))) {
        s.schedule_renewal(today, if s.shop_level as u8 == 1 { 7 } else { 9 }, None);
    }
    false
}

pub(crate) fn tick(
    mut shop: ResMut<ShopData>,
    location: Res<State<ShopLocation>>,
    mut layout: ResMut<super::layout::Layout>,
    disc: Option<Res<crate::game_disc::GameDisc>>,
    identity: Res<crate::save::PlayerIdentity>,
    mut mailbox: ResMut<crate::mail::HouseMailbox>,
) {
    if *location != ShopLocation::Town {
        return;
    }
    layout.upper = false;
    let now = crate::clock::now();
    if advance(&mut shop.shop, now) {
        shop.stock_day = None;
        info!("Shop renovation complete: {:?}", shop.shop.shop_level);
    }
    let s = &mut shop.shop;
    if !s.send_upgrade_notice || date(now) < shift(s.renewal_ymd, -1) {
        return;
    }
    let Some(text) = disc.as_deref().and_then(|d| d.text.as_ref()) else {
        return;
    };
    if !mailbox.0.has_room() {
        // aSL_SetRenewalChiraswhi_Notice clears the one-shot flag even when
        // the resident's mailbox had no free slot for the leaflet.
        s.send_upgrade_notice = false;
        return;
    }
    let (year, month, day) = shift(s.renewal_ymd, -1);
    let mut ctx = crate::messages::MsgContext {
        player_name: identity.player.clone(),
        free: vec![String::new(); 20],
        ..default()
    };
    ctx.free[0] = text.string(0x66D + month as usize - 1).unwrap_or_default();
    ctx.free[1] = text.string(0x64E + day as usize - 1).unwrap_or_default();
    ctx.free[2] = year.to_string();
    let tier = s.shop_level as usize - usize::from(!s.upgrading_today && s.shop_level as u8 > 0);
    if let Some(t) = text.letter([0x18, 0x19, 0x1A, 0x1A][tier.min(3)], &ctx) {
        mailbox.0.push(crate::mail::Letter {
            sender: "Nook".into(),
            header: Some(t.header),
            body: t.body,
            footer: Some(t.footer),
            paper: 55,
            ..default()
        });
        s.send_upgrade_notice = false;
    }
}
