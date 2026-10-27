//! The retail warning panel (`m_warning_ovl.c`) over the pockets screen: the
//! disc's `att_win_model` frame growing from the screen centre with its
//! yellow lines, closed by A, B or Start. Wording, layout and timing are
//! `rustimal_logic::warning`.
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use rustimal_logic::warning::{Panel, Phase, Warning};

use crate::game_disc::GameDisc;
use crate::input::{Action, Actions};
use crate::msg_window::{text_mesh, MsgAssets};
use crate::n64_material::N64Material;
use crate::pockets_disc::Paint;
use crate::shop_nook::editor::{Skin, Stores};

/// Menu depth over the pockets screen.
const Z: f32 = 260.0;
/// `col_type0`: the base ink.
const INK: [u8; 4] = [255, 255, 0, 255];

/// The warning on screen, if any.
#[derive(Resource, Default)]
pub(crate) struct WarningPanel {
    active: Option<Active>,
    skin: Skin,
    /// The answer of the last panel with a question: 0 Yes, 1 No (`menu_info->data1`).
    answer: Option<u8>,
}

struct Active {
    warning: &'static Warning,
    panel: Panel,
    /// 60 Hz frames not yet spent.
    accum: f32,
    root: Option<Entity>,
    /// The scale and answer the entities were last built for.
    built: Option<(f32, u8)>,
    /// `warning_ovl->selected`: 1 (No) when it opens.
    selected: u8,
}

impl WarningPanel {
    pub(crate) fn open(&mut self, warning: &'static Warning) {
        // One panel at a time: a second request while it is up changes nothing.
        if self.active.is_some() {
            return;
        }
        self.active = Some(Active {
            warning,
            panel: Panel::new(),
            accum: 0.0,
            root: None,
            built: None,
            selected: 1,
        });
        self.answer = None;
    }

    /// The Yes (0) or No (1) the last question was closed with, once.
    pub(crate) fn take_answer(&mut self) -> Option<u8> {
        self.answer.take()
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
        let choice = active.warning.has_choice();
        // `mWR_move_Play`: C-left / C-right move between Yes and No, B answers No, A or Start
        // answers the selection (the Yes chime is 0x435).
        if choice {
            if actions.consume(Action::MoveLeft) && active.selected == 1 {
                active.selected = 0;
                crate::sound::se(crate::sound::se_id::CURSOR);
            }
            if actions.consume(Action::MoveRight) && active.selected == 0 {
                active.selected = 1;
                crate::sound::se(crate::sound::se_id::CURSOR);
            }
        }
        let back = actions.consume(Action::Cancel) | actions.consume(Action::Back);
        let mut confirm = false;
        for action in [Action::Confirm, Action::Interact, Action::Advance] {
            confirm |= actions.consume(action);
        }
        if back || confirm {
            if back {
                active.selected = 1;
            } else if choice && active.selected == 0 {
                crate::sound::se(0x435);
            }
            if choice {
                state.answer = Some(active.selected);
            }
            active.panel.dismiss();
        }
    }
    let Some(active) = state.active.as_mut() else {
        return;
    };
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
    if active.built == Some((scale, active.selected)) {
        return;
    }
    active.built = Some((scale, active.selected));
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
    let selected = active.selected;
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
        // `col_type0` yellow; the answers are white, the selected one pink (`col_type1/2`).
        let ink = if !warning.has_choice() || index < 3 {
            INK
        } else if index == 3 + usize::from(selected) {
            [255, 175, 175, 255]
        } else {
            [255, 255, 255, 255]
        };
        if let Some(mesh) = text_mesh(line.text, Vec2::new(x, y), &font.widths, ink, usize::MAX) {
            paint.put(mesh, font.font.clone(), Color::WHITE, Z + 1.0);
        }
    }
    if warning.has_choice() {
        // `mWR_set_frame_dl`: the cursor model beside the selected answer.
        let cursor = state.skin.window(
            &disc,
            "warning-cursor",
            &["att_win_mode", "att_win_cursor_model"],
            [255, 255, 255, 255],
            &mut Stores {
                meshes: &mut meshes,
                n64: &mut n64,
                images: &mut images,
                materials: &mut materials,
            },
        );
        let x = -120.0 * warning.scale_x - 8.0 + warning.lines[3 + usize::from(selected)].x;
        let down: f32 = warning.lines[..4].iter().map(|l| l.y).sum();
        let y = 60.0 * warning.scale_y - (down + 8.0);
        for (index, (mesh, material)) in cursor.iter().enumerate() {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(x * 16.0, y * 16.0, Z + 0.5 + index as f32 * 0.01),
                RenderLayers::layer(1),
                ChildOf(root),
            ));
        }
    }
}

pub(crate) struct WarningPanelPlugin;

impl Plugin for WarningPanelPlugin {
    fn build(&self, app: &mut App) {
        // Right after the actions are read, so the keys that answer a warning never reach the
        // screen it covers.
        app.init_resource::<WarningPanel>().add_systems(
            PreUpdate,
            update
                .after(crate::input::update_focus)
                .before(crate::pockets::toggle_pockets)
                .run_if(in_state(crate::states::PauseState::Running)),
        );
    }
}
