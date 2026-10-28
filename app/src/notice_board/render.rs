//! The notice board's paper and the shared disc font on the UI camera (the diary's
//! paper and keyboard until the notice overlay's own art is built).
use super::*;
use crate::game_disc::GameDisc;
use crate::msg_window::MsgAssets;
use crate::n64_material::N64Material;
use crate::pockets_disc::Paint;
use bevy::camera::visibility::RenderLayers;

#[derive(Resource, Default)]
pub(super) struct NoticeAssets {
    frame: Option<Vec<(Handle<Mesh>, Handle<N64Material>)>>,
    root: Option<Entity>,
    shown: Option<Shown>,
    skin: crate::shop_nook::editor::Skin,
}

type Shown = (
    usize,
    Mode,
    usize,
    usize,
    u64,
    usize,
    kb::Keyboard,
    crate::shop_nook::editor::Held,
    usize,
);

/// How far the paper rolls up to make room for the keyboard.
const LIFT: f32 = 100.0;

#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    mut commands: Commands,
    ui: Res<NoticeUi>,
    post: Res<crate::post::Post>,
    disc: Option<Res<GameDisc>>,
    font: Res<MsgAssets>,
    mut assets: ResMut<NoticeAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut n64: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(view) = ui.0.as_ref() else {
        if let Some(root) = assets.root.take() {
            commands.entity(root).despawn();
        }
        assets.shown = None;
        return;
    };
    let Some(disc) = disc.as_deref() else { return };
    let signature = (
        view.page,
        view.mode,
        view.scroll,
        view.choice,
        view.revision,
        view.cursor,
        view.keyboard,
        view.held,
        post.notices.len(),
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
            Name::new("notice board"),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    assets.root = Some(root);
    assets.shown = Some(signature);
    let editing = view.mode == Mode::Edit;
    let lift = if editing { LIFT } else { 0.0 };
    if let Some(frame) = &assets.frame {
        for (i, (mesh, material)) in frame.iter().enumerate() {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.0, lift * 16.0, 200.0 + i as f32 * 0.1),
                RenderLayers::layer(1),
                ChildOf(root),
            ));
        }
    }
    if editing {
        let mut stores = crate::shop_nook::editor::Stores {
            meshes: &mut meshes,
            n64: &mut n64,
            images: &mut images,
            materials: &mut materials,
        };
        crate::shop_nook::editor::spawn(
            &mut commands,
            root,
            &mut assets.skin,
            disc,
            &font,
            &view.keyboard,
            view.held,
            &mut stores,
        );
    }
    let mut paint = Paint {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        root,
    };
    let ink = [70, 40, 50, 255];
    let count = post.notices.len();
    let shown: Option<&Notice> = (view.mode == Mode::Read)
        .then(|| post.notices.get(view.page))
        .flatten();
    let title = if editing || view.mode == Mode::Confirm {
        "New notice".to_string()
    } else if count == 0 {
        "Notice board".to_string()
    } else {
        format!("Notice {}/{}", view.page + 1, count)
    };
    paint.text(&font, &title, Vec2::new(64.0, 36.0 - lift), ink, 230.0);
    let text: Vec<u8> = match (view.mode, shown) {
        (Mode::Read, Some(n)) => {
            n.message[..native::text_len(&n.message).min(n.message.len())].to_vec()
        }
        _ => view.draft.clone(),
    };
    let lines = native::lines(&text, &font.widths);
    let (cursor_line, cursor_x) = native::cursor_line(&text, view.cursor, &font.widths);
    if view.mode == Mode::Read && count == 0 {
        paint.text(
            &font,
            "(Nothing posted yet.)",
            Vec2::new(64.0, 64.0),
            ink,
            230.0,
        );
    } else {
        for (row, line) in lines
            .iter()
            .skip(view.scroll)
            .take(if editing { EDIT_LINES } else { 6 })
            .enumerate()
        {
            let y = if editing { 28.0 } else { 64.0 } + row as f32 * 16.0;
            paint.text(&font, &display(line), Vec2::new(64.0, y), ink, 230.0);
            if editing && view.scroll + row == cursor_line {
                let at = Vec2::new(64.0 + cursor_x as f32, y);
                if let Some(caret) =
                    paint.text(&font, "_", at, crate::shop_nook::editor::CARET_INK, 230.0)
                {
                    paint
                        .commands
                        .entity(caret)
                        .insert(crate::shop_nook::editor::Caret(at));
                }
            }
        }
    }
    if let Some(n) = shown {
        let (y, m, d, h, mi) = n.time;
        paint.text(
            &font,
            &format!("{y}/{m}/{d} {h:02}:{mi:02}"),
            Vec2::new(64.0, 150.0),
            ink,
            230.0,
        );
    }
    let hints: Vec<String> = match view.mode {
        Mode::Read => vec![
            "Left/Right: page   Up: newest   Down: oldest".into(),
            "Enter: write a notice   Esc: close".into(),
        ],
        Mode::Edit => vec![
            "Enter: key  Bksp  Space  F1: Caps  F3: type  Ctrl+X: accent".into(),
            format!(
                "Ctrl+arrows: cursor   F4: finish   Esc: discard   {}/192  {}/6",
                view.draft.len(),
                lines.len()
            ),
        ],
        Mode::Confirm => {
            let options = ["Post it", "Keep writing", "Never mind"];
            let mut hints = vec!["Post this notice?".to_string()];
            hints.extend(
                options
                    .iter()
                    .enumerate()
                    .map(|(i, o)| format!("{} {o}", if view.choice == i { ">" } else { " " })),
            );
            hints
        }
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
            Vec2::new(64.0, if editing { 0.0 } else { 166.0 } + row as f32 * 12.0),
            ink,
            230.0,
        );
    }
}
