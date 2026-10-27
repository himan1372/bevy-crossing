//! The town's beauty rating (`m_field_assessment.c`), the input of the villager
//! move-in check, the insect hatch odds and the special-event spacing. The
//! scoring is `rustimal_logic::field_assessment`; this module feeds it the
//! town's field items and keeps the result.
//!
//! The original stores the rank in a global (`l_field_assessment_rank`, never
//! saved) refreshed by `mFAs_SetFieldRank` at game start, after the daily
//! regrowth and before a special event is scheduled; the callers read the
//! stored value. Only the trash-over flag (`dust_flag`) is saved. The
//! perfect-town streak (golden axe) is not kept here.
use bevy::prelude::*;
use rustimal_logic::field_assessment as fa;

use crate::field_items::FieldItems;

#[derive(Resource, Default, Clone, Debug)]
pub(crate) struct FieldRank {
    /// The stored rank, 0 through 6; none until the town's items are known.
    pub rank: Option<u8>,
    /// Game day of the last refresh (cleared to force one).
    pub day: Option<i64>,
}

/// Where the dump's sign stands, searched like `mAGrw_SearchDump`.
fn dump(items: &FieldItems) -> Option<fa::DumpUnit> {
    items.acres.iter().enumerate().find_map(|(acre, units)| {
        let unit = units.iter().position(|&item| item == fa::DUMP)?;
        Some(fa::DumpUnit {
            block: (1 + acre % fa::FG_BLOCK_X_NUM, 1 + acre / fa::FG_BLOCK_X_NUM),
            unit: ((unit & 15) as i32, (unit >> 4) as i32),
        })
    })
}

impl FieldRank {
    /// `mFAs_SetFieldRank`.
    pub fn refresh(&mut self, items: &mut FieldItems) {
        let Ok(acres) =
            <&[[u16; fa::UT_TOTAL_NUM]; fa::FG_BLOCK_TOTAL_NUM]>::try_from(items.acres.as_slice())
        else {
            return;
        };
        let dump = dump(items);
        let mut random3 = || 0;
        let mut score = 0;
        let mut scanned = None;
        if items.dust_flag {
            let (remaining, _) = fa::dust_count(acres, dump, true, &mut random3);
            let (flag, rescan) = fa::dust_flag_gate(true, remaining);
            items.dust_flag = flag;
            if !rescan {
                scanned = Some(0);
            }
        }
        if scanned.is_none() {
            let scan = fa::scan(
                acres,
                dump,
                None::<fa::ConditionCtx<'_, fn() -> f32>>,
                true,
                &mut random3,
            );
            if scan.dust_over {
                items.dust_flag = true;
            }
            info!(
                "field assessment: trees {} flowers {} weeds {} trash {}, acres {} perfect {} good {} bad, score {}",
                scan.totals.0,
                scan.totals.1,
                scan.totals.2,
                scan.dust,
                scan.perfect_acres,
                scan.good_acres,
                scan.bad_acres,
                scan.score
            );
            score = scan.score;
        }
        let rank = fa::rank_from_score(score);
        if self.rank != Some(rank) {
            info!("field rank {rank} (trash flag {})", items.dust_flag);
        }
        self.rank = Some(rank);
        self.day = Some(crate::clock::now().day_number);
    }
}

/// Refreshes the rating when the town's items first exist and when the game
/// day turns (`m_all_grow_ovl.c` calls it after each regrowth).
fn daily(items: Option<ResMut<FieldItems>>, mut rank: ResMut<FieldRank>) {
    let Some(mut items) = items else { return };
    if rank.rank.is_none() || rank.day != Some(crate::clock::now().day_number) {
        rank.refresh(&mut items);
    }
}

/// The rank the move-in check sees at start-up (`mFAs_SetFieldRank` runs
/// just before `mNpc_Grow`): the town is not built yet, so its field items are
/// laid out from the retail layout for the seed, with the save's changes and
/// trash flag on them. Only asked for when a move-in could actually happen.
pub(crate) fn rank_at_start(
    disc: &crate::game_disc::GameDisc,
    seed: u32,
    save: Option<&crate::save::SaveData>,
) -> Option<u8> {
    let pool: Vec<u16> = (0..15).collect();
    let mut plan = rustimal_logic::town_gen::generate(seed, &pool, 6)?;
    let (retail, _) = crate::disc_town::build_retail_town(disc, &mut plan, None);
    let mut items = FieldItems::new(retail.items, plan.seed);
    if let Some(changes) = save.and_then(|s| s.field_items.as_ref()) {
        items.restore(changes);
    }
    items.dust_flag = save.is_some_and(|s| s.dust_flag);
    let mut rank = FieldRank::default();
    rank.refresh(&mut items);
    info!("move-in check: field rank {:?}", rank.rank);
    rank.rank
}

pub struct FieldRankPlugin;

impl Plugin for FieldRankPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FieldRank>().add_systems(Update, daily);
    }
}
