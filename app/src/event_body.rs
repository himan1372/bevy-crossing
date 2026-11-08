//! What the event characters share: a body that blocks (the player cannot walk through them, they do not walk through
//! each other), the walking / running stride from how fast they actually move, and an idle clip held while they stand.
//!
//! The original's NPCs have a collision pipe (`collision.pipe.attribute.pipe.radius`, 20 units in the hatsumode) that the
//! player and other actors push against, and `aNPC_set_anime_speed` picks the stride from the speed. The port's event
//! actors move by script, so each module asks [`step_clear`] before taking a step and this module does the rest.

use bevy::prelude::*;

use std::collections::HashMap;

use crate::character::{NpcClip, NpcGait};
use crate::game_disc::GameDisc;
use crate::n64_material::N64Material;

/// The meshes and materials of one disc model.
pub(crate) type PropParts = Vec<(Handle<Mesh>, Handle<N64Material>)>;

/// Disc models the event props use (baskets, balls, the rope), built once by symbol name.
#[derive(Resource, Default)]
pub(crate) struct Props(HashMap<&'static str, PropParts>);

impl Props {
    /// The parts of the model drawn by `symbols` in turn (cached under the first); empty when the disc has no such model (the caller keeps its stand-in shape).
    pub(crate) fn get(
        &mut self,
        disc: &GameDisc,
        symbols: &[&'static str],
        palette: Option<u32>,
        segment8: Option<&'static str>,
        assets: (
            &mut Assets<Mesh>,
            &mut Assets<N64Material>,
            &mut Assets<Image>,
        ),
    ) -> PropParts {
        self.get_season(disc, symbols, palette, false, segment8, assets)
    }

    /// As [`Props::get`], with the winter palette table for structures that have winter models.
    pub(crate) fn get_season(
        &mut self,
        disc: &GameDisc,
        symbols: &[&'static str],
        palette: Option<u32>,
        winter: bool,
        segment8: Option<&'static str>,
        (meshes, materials, images): (
            &mut Assets<Mesh>,
            &mut Assets<N64Material>,
            &mut Assets<Image>,
        ),
    ) -> PropParts {
        self.0
            .entry(symbols[0])
            .or_insert_with(|| {
                // The lists the draw code runs in turn (translucent, then opaque, for the stalls and tables).
                let mut lists = Vec::new();
                for symbol in symbols {
                    let Some(model) = disc.catalog.get(symbol) else {
                        warn!("event prop {symbol} is not on the disc");
                        return Vec::new();
                    };
                    lists.push(model.offset);
                }
                let colours = crate::disc_town::DrawColors {
                    prim: [255; 4],
                    prim_lod: 128,
                    env: [0; 4],
                };
                // `aSTR_PAL_*`: the structure palette the draw code puts in segment 8.
                let palette =
                    palette.and_then(|i| crate::disc_town::structure_palette(disc, i, winter));
                // `gSPSegment(8, ...)`: models whose draw code points segment 8 at a vertex array (the tug
                // of war's rope, `tol_rope_1_v`, drawn unbent here).
                let built = if let Some(v) = segment8.and_then(|name| disc.catalog.get(name)) {
                    crate::disc_town::build_model_segments(
                        disc,
                        &lists,
                        palette,
                        colours.prim,
                        &[(8, v.offset)],
                        (meshes, materials, images),
                    )
                } else {
                    crate::disc_town::build_model_full(
                        disc,
                        &lists,
                        palette,
                        colours,
                        false,
                        (&crate::disc_gfx::Segments::default(), &[]),
                        (meshes, materials, images),
                    )
                };
                built
            })
            .clone()
    }
}

/// Hangs a model's parts under `root`.
pub(crate) fn attach(commands: &mut Commands, root: Entity, parts: &PropParts) {
    for (mesh, material) in parts {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            ChildOf(root),
        ));
    }
}

/// The collision pipe's radius, cells (`pipe.radius = 20` game units of 40).
pub(crate) const RADIUS: f32 = 0.5;
/// How near the player's centre may come to a body's, cells.
pub(crate) const PLAYER_REACH: f32 = 0.8;
/// Speeds, cells a second, above which the stride is a walk and a run.
const WALK_MIN: f32 = 0.3;
const RUN_MIN: f32 = 2.4;

/// An event character with a body.
#[derive(Component, Default)]
pub(crate) struct EventBody {
    last: Option<Vec3>,
}

/// A clip held while the character stands (`talk_info.default_animation`): a disc clip by its full name.
#[derive(Component)]
pub(crate) struct Idle(pub &'static str);

/// Whether a step of `me` from `from` to `to` is free of the other bodies and the player: a body that is already
/// nearer than `reach` does not block a step away from it.
pub(crate) fn step_clear(
    from: Vec3,
    to: Vec3,
    others: impl IntoIterator<Item = Vec3>,
    player: Option<Vec3>,
) -> bool {
    let near = |body: Vec3, reach: f32| {
        let was = (from - body).with_y(0.0).length();
        let now = (to - body).with_y(0.0).length();
        now < reach && now < was
    };
    !others.into_iter().any(|b| near(b, RADIUS * 2.0))
        && !player.is_some_and(|p| near(p, PLAYER_REACH))
}

/// The stride from the speed over the frame.
fn gait(time: Res<Time>, mut bodies: Query<(&Transform, &mut NpcGait, &mut EventBody)>) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    for (at, mut gait, mut body) in &mut bodies {
        let speed = body
            .last
            .map_or(0.0, |p| (at.translation - p).with_y(0.0).length() / dt);
        body.last = Some(at.translation);
        // A jump (teleport, a room change) is not a run.
        gait.0 = if speed > 20.0 {
            0
        } else if speed > RUN_MIN {
            2
        } else if speed > WALK_MIN {
            1
        } else {
            0
        };
    }
}

/// Holds an idle clip while no other clip is playing.
fn idle(mut idles: Query<(&Idle, &mut NpcClip, &NpcGait)>) {
    for (idle, mut clip, gait) in &mut idles {
        if gait.0 != 0 {
            if clip.name == Some(idle.0) {
                clip.stop(8.0);
            }
        } else if clip.name.is_none() {
            clip.start(idle.0, false, 8.0);
        }
    }
}

pub struct EventBodyPlugin;

impl Plugin for EventBodyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Props>().add_systems(
            Update,
            (gait, idle).chain().in_set(crate::states::GameplaySet),
        );
    }
}
