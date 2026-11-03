//! K.K. Slider (`SP_NPC_TOTAKEKE`), the Saturday-night musician.
//!
//! He is out while the original scheduler says `mEv_EVENT_KK_SLIDER` is active (every Saturday 20:00 to
//! 23:59, only when he is that week's visitor; see `event_schedule`). `staffroll_start` in
//! `ac_event_manager.c` puts him in block A-3, middle of the acre (`BlockOrUnit_c b = {1, 3}, u = {7, 7}`;
//! the port's playable acres start one block in, so that is acre (0, 2)).
//! Port adaptation: that cell can be under a building (the museum, in this town layout), so he stands on the
//! nearest clear ground south of it instead (buildings face south).
//!
//! First slice: he appears and leaves with the schedule and wears his disc model. Not built yet: his
//! guitar animation (`aNPC_ANIM_ENSOU_E1`), the orange box and speaker (`mikanbox`, `majin5`), the live
//! song (`BGM_TOTAKEKE_LIVE0 + song`) and the talk tree (`totakeke.rs` has the message and song logic).

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::event_today::TodayEvents;
use crate::shop::ShopLocation;
use crate::states::GameplaySet;
use crate::town::Town;

/// ALL_NPC_NUM + (SP_NPC_TOTAKEKE - SP_NPC_START).
const KK_NPC: u32 = 238 + 93;
/// His stage: acre (0, 2), the middle cell.
const STAGE_ACRE: (usize, usize) = (0, 2);
const STAGE_CELL: (usize, usize) = (7, 7);

#[derive(Component)]
struct KkSlider {
    /// The game day he came out for.
    day: i64,
}

/// His spot: the original's stage cell, or the nearest clear ground when a building stands on it (the
/// museum does in some layouts, and the original places him there without checking).
fn stage(town: &Town) -> Vec3 {
    let centre = Town::cell_to_world(STAGE_ACRE.0, STAGE_ACRE.1, STAGE_CELL.0, STAGE_CELL.1);
    let mut best: Option<(f32, Vec3)> = None;
    // South only: the buildings' doors face south, and north of one is its back wall.
    for dz in 0i32..=12 {
        for dx in -12i32..=12 {
            let at = centre + Vec3::new(dx as f32, 0.0, dz as f32) * crate::town::CELL_SIZE;
            let blocked = |p: Vec3| !town.is_clear_ground(p) || town.in_retail_obstacle(p);
            // A cell of margin all round, so he does not stand against a wall.
            if [Vec3::ZERO, Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z]
                .iter()
                .any(|o| blocked(at + *o))
            {
                continue;
            }
            let d = (at - centre).length();
            if best.is_none_or(|(b, _)| d < b) {
                best = Some((d, at));
            }
        }
    }
    let at = best.map_or(centre, |(_, at)| at);
    at.with_y(town.ground_y(at))
}

/// Brings him out for the evening and sends him home after.
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &KkSlider)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let out = today.is_active(ev::KK_SLIDER, now.hour)
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    for (entity, kk) in &existing {
        if !out || kk.day != now.day_number {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    let at = stage(&town);
    commands.spawn((
        Name::new("K.K. Slider"),
        KkSlider {
            day: now.day_number,
        },
        NpcClip::default(),
        NpcGait::default(),
        NpcModel(KK_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y),
    ));
    info!("K.K. Slider is on stage at {at:?}");
}

pub struct KkSliderPlugin;

impl Plugin for KkSliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            visit.after(crate::event_today::refresh).in_set(GameplaySet),
        );
    }
}
