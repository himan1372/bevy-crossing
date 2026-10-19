//! Outdoor level sources; sequences/samples stay in the runtime disc reader.
use crate::ground_map::GroundMap;
use crate::house::HouseState;
use crate::input::InputFocus;
use crate::player::Player;
use crate::shop::ShopLocation;
use crate::sound::Sound;
use crate::town::{Town, ACRE_SIZE};
use bevy::prelude::*;
use rustimal_logic::ambience::{Mixer, Source};

#[derive(Default)]
pub(crate) struct AmbienceState {
    mixer: Mixer,
    sources: Vec<Source>,
}

pub(crate) fn update_ambience(
    sound: Option<Res<Sound>>,
    town: Res<Town>,
    house: Res<State<HouseState>>,
    shop: Res<State<ShopLocation>>,
    focus: Res<InputFocus>,
    player: Query<&GlobalTransform, With<Player>>,
    bugs: Query<(Entity, &crate::ecology::Bug, &GlobalTransform)>,
    mut state: Local<AmbienceState>,
    toy: Res<crate::player_toys::ToyAction>,
) {
    let Some(sound) = sound else { return };
    let (Some(ground), Ok(player)) = (&town.ground, player.single()) else {
        sound.ambience(state.mixer.clear());
        return;
    };
    if *house.get() != HouseState::Town || *shop.get() != ShopLocation::Town {
        let position = player.translation();
        let sources = (toy.pin_level > 0.0).then_some(Source {
            key: u64::MAX,
            id: 0x4D,
            position: (position * 40.0).to_array(),
        });
        let mut levels = state.mixer.update(
            sources,
            rustimal_logic::ambience::microphone((position * 40.0).to_array()),
            !focus.is_world(),
        );
        for level in levels.iter_mut().flatten() {
            level.volume *= toy.pin_level;
        }
        sound.ambience(levels);
        return;
    }
    let position = player.translation();
    let origin = GroundMap::block_corner(0, 0);
    let block = [
        ((position.x - origin.x) / ACRE_SIZE).floor() as i32,
        ((position.z - origin.y) / ACRE_SIZE).floor() as i32,
    ];
    let marine = if (0..7).contains(&block[0]) && (0..10).contains(&block[1]) {
        town.retail.as_ref().is_some_and(|retail| {
            rustimal_logic::ambience::marine_block_type(
                retail.field.blocks[block[1] as usize * 7 + block[0] as usize],
            )
        })
    } else {
        sound.ambience(state.mixer.clear());
        return;
    };
    let water_id = if marine { 0x1C } else { 0x0B };
    // aFD_OperateWaterSound selects the two closest matching metadata
    // sources in adjacent acres; shore searches only the current row.
    let mut water: [Option<(f32, Source)>; 2] = [None; 2];
    let state = &mut *state;
    let sources = &mut state.sources;
    sources.clear();
    for s in &ground.sound_sources {
        let dx = s.block[0] as i32 - block[0];
        let dz = s.block[1] as i32 - block[1];
        if dx.abs() > 1 || dz.abs() > 1 {
            continue;
        }
        let source = Source {
            key: s.key,
            id: s.id,
            position: (s.position * 40.0).to_array(),
        };
        if s.id == 0x0C {
            sources.push(source);
        } else if s.id == water_id && (!marine || dz == 0) {
            let distance = (s.position - position).xz().length_squared();
            if let Some(i) = water
                .iter()
                .position(|w| w.is_none_or(|(d, _)| distance < d))
            {
                if i == 0 {
                    water[1] = water[0];
                }
                water[i] = Some((distance, source));
            }
        }
    }
    sources.extend(water.into_iter().flatten().map(|(_, s)| s));
    if toy.pin_level > 0.0 {
        sources.push(Source {
            key: u64::MAX,
            id: 0x4D,
            position: (position * 40.0).to_array(),
        });
    }
    for (entity, bug, transform) in &bugs {
        if let Some(id) = bug.ai.as_ref().and_then(|ai| ai.level_sound(false)) {
            sources.push(Source {
                key: entity.to_bits() | (1u64 << 63),
                id,
                position: (transform.translation() * 40.0).to_array(),
            });
        }
    }
    // Fixed outdoor camera's source microphone convention (m_camera2):
    // player + (0, 240, 77) game units; independent of player facing.
    let microphone = rustimal_logic::ambience::microphone((position * 40.0).to_array());
    let quiet = matches!(
        *focus,
        InputFocus::Paused | InputFocus::Pockets | InputFocus::Mail
    );
    let mut levels = state
        .mixer
        .update(sources.iter().copied(), microphone, quiet);
    for level in levels.iter_mut().flatten() {
        if level.key == u64::MAX {
            level.volume *= toy.pin_level;
        }
    }
    sound.ambience(levels);
}
