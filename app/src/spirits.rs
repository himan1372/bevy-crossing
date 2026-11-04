//! Spooky's spirits (`ac_ins_hitodama.c`, the spirit parts of `ac_event_manager.c`, `ac_set_ovl_insect.c` and
//! `ac_insect_clip.c_inc`).
//!
//! Each ghost night five different acres are picked (`ghost_start`: block x `1 + RANDOM(5)`, block z `2 + RANDOM(4)`).
//! Once the player has found Spooky (`mEv_GHOST_FLAG_ACTIVE`) a spirit floats in each of those acres: it is placed
//! when the player comes into the acre (`aSOI_ins_renew_check_range_table`: spawn type SPIRIT) and leaves its acre's
//! entry while it lives (`aIHD_unregist_set_block_table`). It drifts slowly in loose circles (`aIHD_fly`: speed 0.3
//! per tick, a heading that turns 40 to 104 units per tick, and a bob), within 8 cells of its acre's origin. A swing of
//! the net catches it (`mHD_prepare_drop_wisp` and `mPlib_Get_space_putin_item_forHITODAMA`): the catch adds one to a
//! stack in the pockets (`ITM_SPIRIT0..=4` hold 1..=5; a full stack of five starts a new one in a free pocket). A
//! spirit that is not caught goes when the player leaves its acre and is registered in another free acre
//! (`aINS_destruct`); they all fade when the ghost event is over.
//!
//! Not built: the glow's point light, the net-escape ("let escape") and avoid animations and the catch message.

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::town_gen::{ACRE_DEPTH, ACRE_WIDTH};
use rustimal_logic::wisp::wisp as consts;

use crate::event_today::TodayEvents;
use crate::interaction::PlayerInventory;
use crate::player::Player;
use crate::spooky::SpookyEvent;
use crate::states::GameplaySet;
use crate::town::Town;

/// `aIHD_RANGE`: 8 units of 40.
const RANGE: f32 = 8.0;
/// `target_speed` 0.3 per 60 Hz tick, in cells per second.
const SPEED: f32 = 0.3 * 60.0 / 40.0;
/// Hover height over the ground.
const HEIGHT: f32 = 1.4;

/// `hitodama_block_data`: where each of the five spirits waits (`None` while it floats).
#[derive(Resource, Default)]
pub(crate) struct SpiritBlocks {
    day: i64,
    blocks: [Option<(usize, usize)>; consts::SPIRIT_NUM as usize],
}

#[derive(Component)]
pub(crate) struct Spirit {
    slot: usize,
    block: (usize, usize),
    angle: f32,
    turn: f32,
    phase: f32,
}

/// The spirits are out: the ghost event runs, the player found Spooky tonight and has not returned the spirits.
fn active(today: &TodayEvents, spooky: &SpookyEvent) -> bool {
    let now = crate::clock::now();
    today.is_active(ev::GHOST, now.hour)
        && spooky.day == now.day_number
        && spooky.found
        && !spooky.returned
}

/// `ghost_start`: the night's five acres.
fn roll_blocks(
    today: Res<TodayEvents>,
    mut blocks: ResMut<SpiritBlocks>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let now = crate::clock::now();
    if !today.is_active(ev::GHOST, now.hour) || blocks.day == now.day_number {
        return;
    }
    blocks.day = now.day_number;
    let mut picked: Vec<(usize, usize)> = Vec::new();
    while picked.len() < blocks.blocks.len() {
        // Block numbers 1..=5 and 2..=5 are acres 0..=4 and 1..=4 of the port's grid (no border blocks).
        let at = (
            rand.0.random(ACRE_WIDTH as i32) as usize,
            1 + rand.0.random(4) as usize,
        );
        if !picked.contains(&at) {
            picked.push(at);
        }
    }
    for (slot, at) in picked.into_iter().enumerate() {
        blocks.blocks[slot] = Some(at);
    }
    info!("Spirit acres tonight: {:?}", blocks.blocks);
}

/// A random clear cell of an acre, away from its edge.
fn spot_in(
    town: &Town,
    block: (usize, usize),
    rand: &mut crate::game_rand::GameRand,
) -> Option<Vec3> {
    for _ in 0..40 {
        let (cx, cz) = (
            2 + rand.0.random(12) as usize,
            2 + rand.0.random(12) as usize,
        );
        let at = Town::cell_to_world(block.0, block.1, cx, cz);
        if town.is_clear_ground(at) {
            return Some(at.with_y(town.ground_y(at) + HEIGHT));
        }
    }
    None
}

/// `aSOI_ins_renew_check_range_table` and `aINS_destruct`: a spirit comes into the player's acre, and goes from one
/// the player left (to a new acre) or when the event is over.
#[allow(clippy::too_many_arguments)]
fn place_spirits(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    town: Res<Town>,
    today: Res<TodayEvents>,
    spooky: Res<SpookyEvent>,
    mut blocks: ResMut<SpiritBlocks>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    player: Query<&Transform, With<Player>>,
    spirits: Query<(Entity, &Spirit)>,
) {
    let on = active(&today, &spooky);
    let here = player
        .single()
        .ok()
        .and_then(|p| Town::cell_indices_of(p.translation))
        .map(|(ax, az, ..)| (ax, az));
    for (entity, spirit) in &spirits {
        if !on || Some(spirit.block) != here {
            commands.entity(entity).despawn();
            if on {
                register_elsewhere(&mut blocks, spirit.slot, here, &mut rand);
            }
        }
    }
    let (true, Some(here)) = (on, here) else {
        return;
    };
    if spirits.iter().any(|(_, s)| s.block == here) {
        return;
    }
    let Some(slot) = blocks.blocks.iter().position(|b| *b == Some(here)) else {
        return;
    };
    let Some(at) = spot_in(&town, here, &mut rand) else {
        return;
    };
    // `aIHD_unregist_set_block_table`.
    blocks.blocks[slot] = None;
    commands.spawn((
        Name::new("Spirit"),
        Spirit {
            slot,
            block: here,
            angle: rand.0.fqrand() * std::f32::consts::TAU,
            turn: [40.0, 56.0, 80.0, 104.0][rand.0.random(4) as usize] * 60.0 / 65536.0
                * std::f32::consts::TAU,
            phase: rand.0.fqrand() * std::f32::consts::TAU,
        },
        Mesh3d(meshes.add(Sphere::new(0.22))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.7, 1.0, 0.8, 0.85),
            emissive: LinearRgba::new(1.2, 2.4, 1.6, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(at),
    ));
    info!("A spirit floats in acre {here:?}");
}

/// `aINS_regist_set_block_table`: the spirit's entry moves to a random acre that is neither the player's nor taken.
fn register_elsewhere(
    blocks: &mut SpiritBlocks,
    slot: usize,
    here: Option<(usize, usize)>,
    rand: &mut crate::game_rand::GameRand,
) {
    for _ in 0..200 {
        let at = (
            rand.0.random(ACRE_WIDTH as i32) as usize,
            rand.0.random(ACRE_DEPTH as i32) as usize,
        );
        if Some(at) != here && !blocks.blocks.contains(&Some(at)) {
            blocks.blocks[slot] = Some(at);
            info!("A spirit waits in acre {at:?}");
            return;
        }
    }
}

/// `aIHD_fly`: a slow loop with a bob, inside the acre's range.
fn drift(time: Res<Time>, town: Res<Town>, mut spirits: Query<(&mut Spirit, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut s, mut t) in &mut spirits {
        s.angle += s.turn * dt * 4.0;
        s.phase += dt * 2.0;
        let origin = Town::cell_to_world(s.block.0, s.block.1, 0, 0);
        let centre = origin + Vec3::new(RANGE, 0.0, RANGE);
        // Far from the middle: aim back at it (`aIHD_calc_move_drt`).
        let away = (t.translation - centre).with_y(0.0);
        if away.length() > RANGE * 0.5 {
            let back = (-away).normalize_or_zero();
            let want = back.x.atan2(back.z);
            let mut d = (want - s.angle).rem_euclid(std::f32::consts::TAU);
            if d > std::f32::consts::PI {
                d -= std::f32::consts::TAU;
            }
            s.angle += d.clamp(-1.0, 1.0) * dt * 1.5;
        }
        let step = Vec3::new(s.angle.sin(), 0.0, s.angle.cos()) * SPEED * dt;
        t.translation += step;
        let ground = town.ground_y(t.translation);
        t.translation.y = ground + HEIGHT + 0.25 * s.phase.sin();
    }
}

/// The net: the nearest spirit in reach is caught; `Some(slot)` when it was kept. Pockets full: `None`.
pub(crate) fn catch(inventory: &mut PlayerInventory) -> bool {
    // `mPlib_Get_space_putin_item_forHITODAMA`: a stack of four or fewer takes it.
    let stack = inventory
        .pockets
        .pockets
        .iter()
        .position(|&i| (consts::SPIRIT0..consts::SPIRIT0 + 4).contains(&i));
    match stack {
        Some(slot) => {
            inventory.pockets.pockets[slot] += 1;
            true
        }
        None => inventory.add(consts::SPIRIT0),
    }
}

pub struct SpiritsPlugin;

impl Plugin for SpiritsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpiritBlocks>().add_systems(
            Update,
            (roll_blocks, place_spirits, drift)
                .chain()
                .after(crate::event_today::refresh)
                .in_set(GameplaySet),
        );
    }
}
