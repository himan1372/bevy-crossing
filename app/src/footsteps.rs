//! Player and nearby villager foot contacts from the character animation, rendered using
//! the retail footstep rules and the player's runtime disc sound data.
use bevy::prelude::*;
use rustimal_logic::footsteps::{self, FootstepVariants};

use crate::game_disc::GameDisc;
use crate::input::InputFocus;
use crate::states::PauseState;
use crate::town::Town;

#[derive(Message)]
pub(crate) struct Footstep {
    pub position: Vec3,
    pub running: bool,
    pub npc: bool,
}

/// Registered alongside the actual room shell and its floor pattern.
#[derive(Component)]
pub(crate) struct IndoorFloor {
    pub origin: Vec3,
    pub index: usize,
}

fn play_footsteps(
    mut contacts: MessageReader<Footstep>,
    focus: Res<InputFocus>,
    town: Res<Town>,
    disc: Option<Res<GameDisc>>,
    floors: Query<&IndoorFloor>,
    player: Query<&GlobalTransform, With<crate::player::Player>>,
    mut variants: Local<FootstepVariants>,
    mut seed: Local<u32>,
) {
    for step in contacts.read() {
        if !focus.is_world() || disc.is_none() {
            continue;
        }
        let distance = if step.npc {
            let Ok(listener) = player.single() else {
                continue;
            };
            let distance = listener.translation().distance(step.position);
            if distance >= 540.0 / 40.0 {
                continue;
            }
            distance
        } else {
            0.0
        };
        let floor = floors
            .iter()
            .find(|f| (step.position - f.origin).with_y(0.0).length_squared() < 16.0 * 16.0);
        let label = if let Some(floor) = floor {
            disc.as_ref()
                .and_then(|d| {
                    let a = d.catalog.get("SE_FLOOR_DATA")?;
                    let table = d.rom.slice(a.source, a.offset, a.size)?;
                    table
                        .get(floor.index * 2..floor.index * 2 + 2)
                        .map(|v| footsteps::FLOOR_START + u16::from_be_bytes([v[0], v[1]]))
                })
                .unwrap_or(footsteps::WOOD)
        } else {
            let attr = town
                .ground
                .as_ref()
                .and_then(|g| g.sound_attribute_at(step.position.x, step.position.z))
                .unwrap_or(0);
            let winter = crate::clock::now().season == rustimal_logic::game_time::Season::Winter;
            footsteps::outdoor_label(attr, winter)
        };
        *seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
        // Nap_GetRandom's low 16 bits, as in Sou_WalkSe.
        let id = variants.select(label, false, (*seed >> 8) as u16);
        let volume = if step.npc {
            footsteps::npc_volume(distance * 40.0, floor.is_some())
        } else {
            footsteps::volume(step.running, floor.is_some())
        };
        crate::sound::se_volume(id, volume);
    }
}

pub struct FootstepPlugin;

impl Plugin for FootstepPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Footstep>().add_systems(
            PostUpdate,
            play_footsteps.run_if(in_state(PauseState::Running)),
        );
    }
}
