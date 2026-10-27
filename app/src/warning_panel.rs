//! The retail warning panel (`m_warning_ovl.c`) over the pockets screen: the
//! disc's `att_win_model` frame growing from the screen centre with its
//! yellow lines, closed by A, B or Start. Wording, layout and timing are
//! `rustimal_logic::warning`.
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use rustimal_logic::warning::{Panel, Phase, Warning};

use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InteractSet};
use crate::msg_window::{text_mesh, MsgAssets};
use crate::n64_material::N64Material;
use crate::pockets_disc::Paint;
use crate::shop_nook::editor::{Skin, Stores};
use crate::states::GameplaySet;

/// Menu depth over the pockets screen.
const Z: f32 = 260.0;
/// `col_type0`: the base ink.
const INK: [u8; 4] = [255, 255, 0, 255];

/// The warning on screen, if any.
#[derive(Resource, Default)]
pub(crate) struct WarningPanel {
    active: Option<Active>,
    skin: Skin,
}

struct Active {
    warning: &'static Warning,
    panel: Panel,
    /// 60 Hz frames not yet spent.
    accum: f32,
    root: Option<Entity>,
    /// The scale the entities were last built for.
    built: Option<f32>,
}

impl WarningPanel {
    pub(crate) fn open(&mut self, warning: &'static Warning) {
        self.active = Some(Active {
            warning,
            panel: Panel::new(),
            accum: 0.0,
            root: None,
            built: None,
        });
    }

    /// True from `open` until the panel has shrunk away.
    pub(crate) fn is_open(&self) -> bool {
        self.active.is_some()
    }

    /// Drops it at once (scene change, reload).
    pub(crate) fn clear(&mut self, commands: &mut Commands) {
        if let Some(root) = self.active.take().and_then(|a| a.root) {
            commands.entity(root).despawn();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut actions: ResMut<Actions>,
    mut state: ResMut<WarningPanel>,
    disc: Option<Res<GameDisc>>,
    font: Option<Res<MsgAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut n64: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let state = &mut *state;
    let Some(active) = state.active.as_mut() else {
        return;
    };
    if active.panel.phase == Phase::Waiting {
        let mut pressed = false;
        for action in [
            Action::Confirm,
            Action::Cancel,
            Action::Back,
            Action::Interact,
            Action::Advance,
        ] {
            pressed |= actions.consume(action);
        }
        if pressed {
            active.panel.dismiss();
        }
    }
    active.accum += time.delta_secs() * 60.0;
    while active.accum >= 1.0 {
        active.accum -= 1.0;
        active.panel.frame();
    }
    if active.panel.phase == Phase::Gone {
        state.clear(&mut commands);
        return;
    }
    let scale = active.panel.scale;
    if active.built == Some(scale) {
        return;
    }
    active.built = Some(scale);
    if let Some(root) = active.root.take() {
        commands.entity(root).despawn();
    }
    let (Some(disc), Some(font)) = (disc, font) else {
        return;
    };
    // The whole panel grows about the screen centre.
    let root = commands
        .spawn((
            Name::new("warning panel"),
            Transform::from_scale(Vec3::new(scale, scale, 1.0)),
            Visibility::default(),
            RenderLayers::layer(1),
        ))
        .id();
    active.root = Some(root);
    let warning = active.warning;
    let mut stores = Stores {
        meshes: &mut meshes,
        n64: &mut n64,
        images: &mut images,
        materials: &mut materials,
    };
    let model = state.skin.window(
        &disc,
        "warning",
        &["att_win_mode", "att_win_model"],
        [255, 255, 255, 255],
        &mut stores,
    );
    for (index, (mesh, material)) in model.iter().enumerate() {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(0.0, 0.0, Z + index as f32 * 0.01).with_scale(Vec3::new(
                warning.scale_x,
                warning.scale_y,
                1.0,
            )),
            RenderLayers::layer(1),
            ChildOf(root),
        ));
    }
    let mut paint = Paint {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        root,
    };
    for (index, line) in warning.lines.iter().enumerate() {
        let (x, y) = warning.line_origin(index);
        if let Some(mesh) = text_mesh(line.text, Vec2::new(x, y), &font.widths, INK, usize::MAX) {
            paint.put(mesh, font.font.clone(), Color::WHITE, Z + 1.0);
        }
    }
}

pub(crate) struct WarningPanelPlugin;

impl Plugin for WarningPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WarningPanel>().add_systems(
            Update,
            update
                .before(crate::pockets::pockets_input)
                .in_set(InteractSet::Ui)
                .in_set(GameplaySet),
        );
    }
}
