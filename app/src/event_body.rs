//! What the event characters share: a body that blocks (the player cannot walk through them, they do not walk through
//! each other), the walking / running stride from how fast they actually move, and an idle clip held while they stand.
//!
//! The original's NPCs have a collision pipe (`collision.pipe.attribute.pipe.radius`, 20 units in the hatsumode) that the
//! player and other actors push against, and `aNPC_set_anime_speed` picks the stride from the speed. The port's event
//! actors move by script, so each module asks [`step_clear`] before taking a step and this module does the rest.

use bevy::prelude::*;

use crate::character::{NpcClip, NpcGait};

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
        app.add_systems(
            Update,
            (gait, idle).chain().in_set(crate::states::GameplaySet),
        );
    }
}
