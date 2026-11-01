//! Shared disc message/choice models and font, with a usable no-disc fallback.
use super::{Menu, View};
use crate::game_disc::GameDisc;
use crate::msg_window::{choice_layout, quad_mesh, screen, MsgAssets};
use crate::pockets_disc::Paint;
use crate::states::PauseState;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::text::FontSize;
#[derive(Component)]
pub(super) struct Fallback;
#[derive(Default)]
pub(super) struct Shown {
    key: Option<View>,
    root: Option<Entity>,
}
pub(super) fn setup(mut commands: Commands) {
    commands.spawn((
        Fallback,
        Name::new("pause fallback"),
        Text::default(),
        TextFont {
            font_size: FontSize::Px(25.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(25.0),
            left: Val::Percent(18.0),
            max_width: Val::Percent(70.0),
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.92)),
        Visibility::Hidden,
    ));
}
#[allow(clippy::too_many_arguments)] // Existing overlay stores are independent Bevy resources.
pub(super) fn draw(
    mut commands: Commands,
    state: Res<State<PauseState>>,
    menu: Res<Menu>,
    disc: Option<Res<GameDisc>>,
    font: Option<Res<MsgAssets>>,
    identity: Res<crate::save::PlayerIdentity>,
    mut shown: Local<Shown>,
    mut fallback: Query<(&mut Text, &mut Visibility), With<Fallback>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if *state.get() != PauseState::Paused {
        if let Some(root) = shown.root.take() {
            commands.entity(root).despawn();
        }
        shown.key = None;
        for (_, mut visibility) in &mut fallback {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
        }
        return;
    }
    let view = menu.view(disc.as_deref(), &identity);
    let native = font
        .as_ref()
        .is_some_and(|f| !f.window.is_empty() && !f.choice_body.is_empty());
    for (mut text, mut visibility) in &mut fallback {
        *visibility = if native {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        let answers = view
            .answers
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{} {s}", if i == view.selected { ">" } else { " " }))
            .collect::<Vec<_>>()
            .join("\n");
        let value = format!("Paused\n\n{}\n\n{answers}\n\n{}", view.body, view.hint);
        if text.0 != value {
            text.0 = value;
        }
    }
    if shown.key.as_ref() == Some(&view) {
        return;
    }
    if let Some(root) = shown.root.take() {
        commands.entity(root).despawn();
    }
    shown.key = Some(view.clone());
    let Some(font) = font.filter(|_| native) else {
        return;
    };
    let layer = RenderLayers::layer(1);
    let root = commands
        .spawn((
            Name::new("retail pause save"),
            Transform::default(),
            Visibility::default(),
            layer.clone(),
        ))
        .id();
    shown.root = Some(root);
    const Z: f32 = 300.0;
    for (mesh, material) in &font.window {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(screen(Vec2::new(160.0, 185.4)).with_z(Z)),
            layer.clone(),
            ChildOf(root),
        ));
    }
    let mut paint = Paint {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        root,
    };
    paint.text(&font, "Paused", Vec2::new(16.0, 16.0), [255; 4], Z + 2.0);
    paint.text(&font, view.hint, Vec2::new(16.0, 34.0), [255; 4], Z + 2.0);
    paint.text(
        &font,
        &view.body,
        Vec2::new(64.0, 153.4),
        [50, 60, 50, 255],
        Z + 2.0,
    );
    if view.answers.is_empty() {
        return;
    }
    let answers: Vec<_> = view.answers.iter().map(String::as_str).collect();
    let (mut centre, scale, at) = choice_layout(&answers, &font.widths);
    // This paused PC overlay has no speaking actor/camera to make room for long save labels.
    // Keep the original choice geometry above the message so its question remains readable.
    centre.y -= 64.0;
    for (mesh, material) in &font.choice_body {
        paint.commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(screen(centre).with_z(Z + 3.0))
                .with_scale(scale.extend(1.0)),
            layer.clone(),
            ChildOf(root),
        ));
    }
    for (i, answer) in answers.iter().enumerate() {
        let origin = at + centre - Vec2::new(160.0, 120.0) + Vec2::Y * (i as f32 * 16.0);
        let color = if i == view.selected {
            paint.put(
                quad_mesh(origin - Vec2::new(16.0, 0.0), Vec2::splat(16.0)),
                font.cursor.clone(),
                Color::srgba_u8(0, 195, 185, 255),
                Z + 5.0,
            );
            [120, 50, 50, 255]
        } else {
            [180, 150, 110, 255]
        };
        paint.text(&font, answer, origin, color, Z + 4.0);
    }
}
