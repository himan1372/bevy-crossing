//! The port's development overlay: the bells/pockets, tool, clock, house and
//! quest lines. Retail Animal Crossing shows no field HUD (time comes from
//! the wall clocks in its buildings and furniture clocks, bells from the
//! pockets screen), so the overlay is hidden by default; F10 toggles it for
//! playtesting. `Node::display` is used because some of these texts manage
//! their own `Visibility`.
use bevy::prelude::*;

use crate::input::{Action, Actions};

/// Marks a persistent development-only text.
#[derive(Component)]
pub(crate) struct DevOverlay;

#[derive(Resource, Default)]
pub(crate) struct DevOverlayShown(pub bool);

fn toggle(mut shown: ResMut<DevOverlayShown>, mut actions: ResMut<Actions>) {
    if actions.consume(Action::DevOverlay) {
        shown.0 = !shown.0;
        info!("dev overlay {}", if shown.0 { "shown" } else { "hidden" });
    }
}

fn apply(shown: Res<DevOverlayShown>, mut nodes: Query<&mut Node, With<DevOverlay>>) {
    let display = if shown.0 {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut nodes {
        if node.display != display {
            node.display = display;
        }
    }
}

pub(crate) struct DevOverlayPlugin;

impl Plugin for DevOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DevOverlayShown>()
            .add_systems(Update, (toggle, apply).chain());
    }
}
