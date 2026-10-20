//! Room-insect chirps: the cage callbacks' sAdo_RoomIncectPos calls, run
//! through the native `Na_RoomIncectPos` scheduler once per 60 Hz frame.
//!
//! Houses run the furniture sequence (Na_SceneMode 2) whose group port 2 is
//! the beat; the field stops it. A reported sound plays as a positional
//! trigger at the cage (Na_OngenTrgStart) and restarts a stopped cicada or
//! cricket clip.
//!
//! Retail quirk, reproduced: sequence 246 advances its port-2 beat only when
//! group callback 1 (`0xBE 01`) returns 0, and the game never registers that
//! callback, so the beat stays 0 in rooms and countdowns never run. A new
//! insect sounds once only if all six trigger slots hold other sounds
//! (`SE_ROOM_INS_DATA` ends in 0, matching a free slot); otherwise it stays
//! quiet and its stopped clip does not play.

use bevy::prelude::*;
use rustimal_logic::room_insect::RoomInsects;

use super::{PlayerRoom, RoomPiece};
use crate::furniture::rigs::FurnitureRig;
use crate::house::{CurrentRoom, HouseState};
use crate::sound::Sound;

/// `graph_dt_60hz_ticks` caps catch-up at four frames.
const MAX_TICKS: u32 = 4;

#[derive(Default)]
pub(super) struct Insects {
    slots: RoomInsects,
    ticks: f32,
    rng: u32,
}

/// `Nap_GetRandom` stand-in (xorshift32).
fn random(state: &mut u32) -> u16 {
    let mut x = if *state == 0 { 0x2545_F491 } else { *state };
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    (x >> 16) as u16
}

#[allow(clippy::too_many_arguments)]
pub(super) fn update(
    mut room: ResMut<PlayerRoom>,
    current: Res<CurrentRoom>,
    house: Res<State<HouseState>>,
    sound: Option<Res<Sound>>,
    time: Res<Time>,
    mut rigs: Query<(Entity, &mut FurnitureRig, &RoomPiece, &GlobalTransform)>,
    mut inside: Local<bool>,
) {
    let in_house = *house.get() == HouseState::Inside;
    if in_house != *inside {
        *inside = in_house;
        if let Some(sound) = sound.as_deref() {
            sound.furniture_sequence(in_house);
        }
    }
    let Some(sound) = sound else { return };
    if !crate::house::is_player_room(&current, &house) {
        room.insects.slots.clear();
        room.insects.ticks = 0.0;
        return;
    }
    room.insects.ticks += time.delta_secs() * 60.0;
    let ticks = (room.insects.ticks as u32).min(MAX_TICKS);
    room.insects.ticks = (room.insects.ticks - ticks as f32).min(1.0);
    for _ in 0..ticks {
        let beat = sound.furniture_beat();
        let triggers = sound.trigger_ids();
        for (entity, mut rig, piece, transform) in &mut rigs {
            let Some(argument) = rig.chirp else { continue };
            // aFTR_CAN_PLAY_SE: not while the piece is being born or removed.
            if !room.clock_sound_allowed(piece) {
                continue;
            }
            // The source keys slots by actor address; any live nonzero id works.
            let id = (entity.to_bits() as u32).wrapping_add(1);
            let Insects { slots, rng, .. } = &mut room.insects;
            let mut draw = || random(rng);
            if let Some(sound_id) = slots.report(id, argument, beat, &mut draw, &triggers) {
                crate::sound::se_at(sound_id, transform.translation());
                rig.chirped = true;
            }
        }
        room.insects.slots.retire();
    }
}
