//! Saved Stalk Market and the existing world's Sunday-growth adapter.
use super::*;
use rustimal_logic::qrand::Qrand;
use rustimal_logic::{game_time as gt, turnips};
use serde::{Deserialize, Serialize};

type Date = (u16, u8, u8);

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Market {
    pub prices: [u16; 7],
    pub trend: u8,
    pub week: Date,
    pub growth_day: Option<i64>,
    pub seen_date: Option<Date>,
    /// The game day Joan last spoke to this player (her weekly memory).
    pub joan_spoken: Option<i64>,
    #[serde(skip)]
    observed: Option<Date>,
}

impl Market {
    pub fn price(&self) -> u16 {
        self.prices[usize::from(crate::clock::now().weekday)]
    }

    /// Kabu_manager runs at startup and calendar-date changes, not each frame.
    fn refresh(&mut self, now: crate::clock::Now, seed: u32) {
        let today = (now.year, now.month, now.day);
        if self.observed == Some(today) {
            return;
        }
        self.observed = Some(today);
        let mut week = gt::RtcTime {
            year: now.year,
            month: now.month,
            day: now.day,
            ..default()
        };
        gt::add_days(&mut week, -i32::from(now.weekday));
        let current = (week.year, week.month, week.day);
        let valid = self.trend < 3
            && (2000..=2099).contains(&self.week.0)
            && self.week.1 > 0
            && self.week.1 <= 12
            && self.week.2 > 0
            && self.week.2 <= gt::get_days_by_month(self.week.0, self.week.1)
            && self.prices.iter().all(|&p| p > 0 && p <= 2000);
        let delta = if valid {
            gt::interval_days_ymd(
                &gt::RtcYmd {
                    year: self.week.0,
                    month: self.week.1,
                    day: self.week.2,
                },
                &gt::RtcYmd {
                    year: now.year,
                    month: now.month,
                    day: now.day,
                },
            )
        } else {
            100
        };
        // lbRTC_IsOverTime returns OVER on equality: the next Sunday midnight
        // renews the week; exactly seven days backward remains in range.
        let elapsed = i64::from(delta) * 86400 + i64::from(now.day_secs);
        let same_sunday = valid && today == self.week;
        if !valid || same_sunday || !(-7 * 86400..7 * 86400).contains(&elapsed) {
            // The week's prices come from the town seed and the date, so they
            // stay the same across a reload: their own `qrand` stream.
            let mix = u64::from(seed)
                ^ (now.day_number as u64).wrapping_mul(0x9E3779B97F4A7C15)
                ^ u64::from(now.day_secs)
                ^ 0xA0761D6478BD642F;
            let mut stream = Qrand::new((mix ^ (mix >> 32)) as u32);
            let mut random = || stream.fqrand();
            let (trend, prices) = turnips::schedule(
                self.trend,
                same_sunday.then_some(self.prices[0]),
                &mut random,
            );
            self.trend = trend;
            self.prices = prices;
            self.week = current;
        }
    }
}

pub(crate) fn tick(
    mut shop: ResMut<ShopData>,
    location: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    seed: Res<crate::town::TownSeed>,
    mut inventory: ResMut<PlayerInventory>,
    mut field: Option<ResMut<crate::field_items::FieldItems>>,
    mut room: ResMut<crate::player_room::PlayerRoom>,
    mut floors: ResMut<crate::player_room::floors::HomeFloors>,
) {
    let now = crate::clock::now();
    let market = &mut shop.turnips;
    // m_time updates prices on every calendar-date change, including indoors.
    // Possession/field spoilage belongs to the separate town growth pass.
    market.refresh(now, seed.0);
    if *location != ShopLocation::Town || *house != crate::house::HouseState::Town {
        return;
    }
    let today = (now.year, now.month, now.day);
    let weekday = if now.hour < 6 {
        (now.weekday + 6) % 7
    } else {
        now.weekday
    };
    let spoil = market.seen_date.is_some_and(|d| today < d)
        || market.growth_day.is_some_and(|old| {
            now.day_number > old && now.day_number.saturating_sub(old) > i64::from(weekday)
        });
    // mAGrw_GrowBlockFgItem clears old spoiled ground items before it spoils
    // this pass's fresh ones. Possessions and house layers are retained.
    if market.growth_day.is_some_and(|old| now.day_number > old) {
        if let Some(field) = &mut field {
            for item in field.acres.iter_mut().flatten() {
                if *item == 0x2F03 {
                    *item = 0;
                }
            }
        }
    }
    if spoil {
        for item in &mut inventory.pockets.pockets {
            turnips::spoil(item);
        }
        if let Some(field) = &mut field {
            for item in field.acres.iter_mut().flatten() {
                turnips::spoil(item);
            }
        }
        room.spoil_turnips();
        for saved in floors.saved.iter_mut().flatten() {
            saved.spoil_turnips();
        }
        info!("Turnips spoiled at growth boundary/date reversal");
    }
    market.growth_day = Some(now.day_number);
    market.seen_date = Some(today);
}
