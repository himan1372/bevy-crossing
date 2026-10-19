//! Original diary paper/frame and shared runtime disc font on the UI camera.
use super::*;
use crate::msg_window::MsgAssets;
use crate::n64_material::N64Material;
use crate::pockets_disc::Paint;
use bevy::camera::visibility::RenderLayers;

#[derive(Resource, Default)]
pub(super) struct DiaryAssets {
    frame: Option<Vec<(Handle<Mesh>, Handle<N64Material>)>>,
    root: Option<Entity>,
    shown: Option<(usize, Mode, usize, usize, u64)>,
}

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    mut commands: Commands,
    room: Res<PlayerRoom>,
    disc: Option<Res<GameDisc>>,
    font: Res<MsgAssets>,
    mut assets: ResMut<DiaryAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut n64: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(view) = room.diary_ui.as_ref() else {
        if let Some(root) = assets.root.take() {
            commands.entity(root).despawn();
        }
        assets.shown = None;
        return;
    };
    let Some(disc) = disc.as_deref() else { return };
    let signature = (
        view.month,
        view.mode,
        view.scroll,
        view.choice,
        view.revision,
    );
    if assets.shown == Some(signature) && assets.root.is_some() {
        return;
    }
    if assets.frame.is_none() {
        let lists: Vec<_> = [
            "dia_init_mode_letter",
            "dia_win_wT_model",
            "dia_win_fusenT_model",
        ]
        .iter()
        .filter_map(|n| disc.catalog.get(n).map(|s| s.offset))
        .collect();
        let sink = crate::disc_gfx::run_display_lists(&disc.rom, &lists);
        assets.frame = Some(crate::disc_town::sink_meshes_scaled(
            disc,
            &sink,
            None,
            true,
            &crate::disc_gfx::Segments::default(),
            16.0,
            (&mut meshes, &mut n64, &mut images),
        ));
    }
    if let Some(root) = assets.root.take() {
        commands.entity(root).despawn();
    }
    let root = commands
        .spawn((
            Name::new("monthly diary"),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    assets.root = Some(root);
    assets.shown = Some(signature);
    if let Some(frame) = &assets.frame {
        for (i, (mesh, material)) in frame.iter().enumerate() {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.0, 0.0, 200.0 + i as f32 * 0.1),
                RenderLayers::layer(1),
                ChildOf(root),
            ));
        }
    }
    let mut paint = Paint {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        root,
    };
    let ink = [70, 40, 50, 255];
    paint.text(
        &font,
        &format!("{} diary", MONTH_NAMES[view.month]),
        Vec2::new(64.0, 36.0),
        ink,
        230.0,
    );
    let lines = native::lines(&view.draft, &font.widths);
    if view.draft.is_empty() && view.mode == Mode::Read {
        paint.text(
            &font,
            "(Nothing written yet.)",
            Vec2::new(64.0, 64.0),
            ink,
            230.0,
        );
    } else {
        for (row, line) in lines
            .iter()
            .skip(view.scroll)
            .take(native::VISIBLE_LINES)
            .enumerate()
        {
            let mut text = display(line);
            if view.mode == Mode::Edit && view.scroll + row + 1 == lines.len() {
                text.push('_');
            }
            paint.text(
                &font,
                &text,
                Vec2::new(64.0, 64.0 + row as f32 * 16.0),
                ink,
                230.0,
            );
        }
    }
    let hints: Vec<String> = match view.mode {
        Mode::Read => vec![
            "Left/Right: month   Enter: write".into(),
            "Up/Down: scroll   PgUp/PgDn: 10 lines".into(),
            "Esc: close".into(),
        ],
        Mode::Edit => vec![
            "Type: text   Enter: new line".into(),
            "Ctrl+Enter: finish   Esc: discard draft".into(),
            format!(
                "{} / 992 bytes    {} / 31 lines",
                view.draft.len(),
                lines.len()
            ),
        ],
        Mode::Confirm => vec![
            "Is this OK?".into(),
            format!(
                "{} Yes    {} Rewrite",
                if view.choice == 0 { ">" } else { " " },
                if view.choice == 1 { ">" } else { " " }
            ),
            "Up/Down: choose   Enter: accept   Esc: rewrite".into(),
        ],
    };
    for (row, hint) in hints.iter().enumerate() {
        let small = format!(
            "{}{}{}",
            crate::messages::markup::LINE_SCALE,
            crate::messages::markup::arg(16),
            hint
        );
        paint.text(
            &font,
            &small,
            Vec2::new(64.0, 166.0 + row as f32 * 12.0),
            ink,
            230.0,
        );
    }
}
