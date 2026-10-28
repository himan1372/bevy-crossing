//! The notice board's paper (`kei_win_model`), its entry number and date, the control hints and
//! the shared disc font on the UI camera (`m_notice_ovl.c`'s `mNT_set_dl`). Editing uses the
//! shop editor's keyboard.
use super::*;
use crate::game_disc::GameDisc;
use crate::msg_window::MsgAssets;
use crate::n64_material::N64Material;
use crate::pockets_disc::Paint;
use bevy::camera::visibility::RenderLayers;

type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;

/// The control hints (`mNT_set_key_dl`): the key display, the two page arrows, and the stick
/// marker (`kei_win_stT_model`, its picture chosen by the stick's area).
struct Hints {
    keys: Model,
    next: Model,
    previous: Model,
    stick: Model,
}

#[derive(Resource, Default)]
pub(super) struct NoticeAssets {
    frame: Option<Model>,
    hints: Option<Hints>,
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
    (board::Play, usize, usize),
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
        (view.motion.mode, view.motion.now, view.motion.disp),
    );
    if assets.shown == Some(signature) && assets.root.is_some() {
        return;
    }
    if assets.frame.is_none() {
        let mut build = |names: &[&str], segments: &crate::disc_gfx::Segments| {
            let lists: Vec<_> = names
                .iter()
                .filter_map(|n| disc.catalog.get(n).map(|s| s.offset))
                .collect();
            let sink = crate::disc_gfx::run_display_lists(&disc.rom, &lists);
            crate::disc_town::sink_meshes_scaled(
                disc,
                &sink,
                None,
                true,
                segments,
                16.0,
                (&mut meshes, &mut n64, &mut images),
            )
        };
        let none = crate::disc_gfx::Segments::default();
        assets.frame = Some(build(&["kei_win_model"], &none));
        // `kei_win_st_tex_tbl[mED_STICK_AREA_CENTER]`: the stick at rest.
        let mut stick = crate::disc_gfx::Segments::default();
        if let Some(texture) = disc.catalog.get("kei_win_st1_tex_rgb_ia8") {
            if let Some(bytes) = disc
                .rom
                .slice(disc::RomSource::Rel, texture.offset, texture.size)
            {
                stick.set_segment(8, bytes.into());
            }
        }
        assets.hints = Some(Hints {
            keys: build(&["kei_hyouji_model"], &none),
            next: build(&["kei_win_yaji1T_mode", "kei_win_yaji1T_model"], &none),
            previous: build(&["kei_win_yaji1T_mode", "kei_win_yaji2T_model"], &none),
            stick: build(&["kei_win_stT_model"], &stick),
        });
    }
    if let Some(root) = assets.root.take() {
        commands.entity(root).despawn();
    }
    let root = commands
        .spawn((
            Name::new("notice board"),
            BoardRoot,
            Transform::from_xyz(0.0, view.slide.pos * 16.0, 0.0),
            Visibility::default(),
        ))
        .id();
    assets.root = Some(root);
    assets.shown = Some(signature);
    let motion = view.motion;
    let count = post.notices.len();
    // The arriving page, and the leaving one while a page turns (`mNT_set_dl`).
    let mut pages = vec![(motion.now, Slot::Arriving)];
    if motion.disp != motion.now {
        pages.push((motion.disp, Slot::Leaving));
    }
    for (page, slot) in pages {
        let group = commands
            .spawn((
                slot,
                Transform::default(),
                Visibility::default(),
                ChildOf(root),
            ))
            .id();
        let mut paint = Paint {
            commands: &mut commands,
            meshes: &mut meshes,
            materials: &mut materials,
            root: group,
        };
        spawn_page(&mut paint, &assets.frame, &font, view, &post.notices, page);
    }
    if let (Some(hints), true) = (&assets.hints, view.mode == Mode::Read || !motion.idle()) {
        // `mNT_set_key_dl`: the key display, the page arrows that lead somewhere, the stick.
        let group = commands
            .spawn((
                Slot::Hints,
                Transform::default(),
                Visibility::default(),
                ChildOf(root),
            ))
            .id();
        let mut parts: Vec<(&Model, f32, f32)> = vec![(&hints.keys, 0.0, 0.0)];
        if view.page + 1 < count {
            parts.push((&hints.next, 0.1, 0.0));
        }
        if view.page != 0 {
            parts.push((&hints.previous, 0.2, 0.0));
        }
        // The stick marker sits three pixels to the right (`Matrix_translate(3, 0, 0)`).
        parts.push((&hints.stick, 0.3, 3.0 * 16.0));
        for (model, z, x) in parts {
            for (i, (mesh, material)) in model.iter().enumerate() {
                commands.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(x, 0.0, 201.0 + z + i as f32 * 0.01),
                    RenderLayers::layer(1),
                    ChildOf(group),
                ));
            }
        }
    }
    if view.mode == Mode::Edit && motion.idle() {
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
    let ink = [30, 0, 0, 255];
    let hints: Vec<String> = match view.mode {
        _ if !motion.idle() => Vec::new(),
        Mode::Read => Vec::new(),
        Mode::Edit => {
            let lines = native::lines(&view.draft, &font.widths).len();
            vec![
                "Enter: key  Bksp  Space  F1: Caps  F3: type  Ctrl+X: accent".into(),
                format!(
                    "Ctrl+arrows: cursor   F4: finish   Esc: discard   {}/192  {}/6",
                    view.draft.len(),
                    lines
                ),
            ]
        }
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
    let mut paint = Paint {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        root,
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
            Vec2::new(
                64.0,
                if view.mode == Mode::Edit { 0.0 } else { 166.0 } + row as f32 * 12.0,
            ),
            ink,
            230.0,
        );
    }
}

/// One page of the board (`mNT_set_page_dl`): the paper, the entry number and date, the text.
/// `page` is a post's index, or `WRITE_PAGE` for the one being written.
fn spawn_page(
    paint: &mut Paint,
    frame: &Option<Model>,
    font: &MsgAssets,
    view: &View,
    notices: &[Notice],
    page: usize,
) {
    let writing = page == board::WRITE_PAGE;
    let editing = writing && view.mode == Mode::Edit;
    let lift = if editing { LIFT } else { 0.0 };
    if let Some(frame) = frame {
        for (i, (mesh, material)) in frame.iter().enumerate() {
            paint.commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.0, lift * 16.0, 200.0 + i as f32 * 0.1),
                RenderLayers::layer(1),
                ChildOf(paint.root),
            ));
        }
    }
    // `mNT_set_page_dl`: the body is dark red at full size, the header blue at three quarters.
    let ink = [30, 0, 0, 255];
    let blue = [0, 0, 255, 255];
    let shown: Option<&Notice> = if writing { None } else { notices.get(page) };
    if !writing && shown.is_none() {
        paint.text(
            font,
            "(Nothing posted yet.)",
            Vec2::new(63.0, 63.0),
            ink,
            230.0,
        );
        return;
    }
    let small = |s: &str| {
        format!(
            "{}{}{}",
            crate::messages::markup::LINE_SCALE,
            crate::messages::markup::arg(24),
            s
        )
    };
    let entry = if writing {
        (notices.len() + 1).min(board::POSTS)
    } else {
        page + 1
    };
    paint.text(
        font,
        &small(&format!("entry {entry:>2}")),
        Vec2::new(63.0, 46.0 - lift),
        blue,
        230.0,
    );
    let (year, month, day, _, _) = shown.map(|n| n.time).unwrap_or_else(|| {
        let now = crate::clock::now();
        (now.year, now.month, now.day, now.hour, now.minute)
    });
    // `mNT_set_day_strings_dl`: "Month" ends at x 213, then the day, a comma and the year.
    const MONTHS: [&str; 13] = [
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
        "???",
    ];
    let name = MONTHS[usize::from(month.clamp(1, 13)) - 1];
    let width = crate::msg_window::text_width(name, &font.widths) * 0.75;
    let y = 46.0 - lift;
    paint.text(font, &small(name), Vec2::new(213.0 - width, y), blue, 230.0);
    paint.text(
        font,
        &small(&format!("{day:>2}")),
        Vec2::new(215.0, y),
        blue,
        230.0,
    );
    paint.text(font, &small(","), Vec2::new(227.0, y), blue, 230.0);
    paint.text(
        font,
        &small(&format!("{year:04}")),
        Vec2::new(233.0, y),
        blue,
        230.0,
    );
    let text: Vec<u8> = match shown {
        Some(n) => n.message[..native::text_len(&n.message).min(n.message.len())].to_vec(),
        None => view.draft.clone(),
    };
    let lines = native::lines(&text, &font.widths);
    let (cursor_line, cursor_x) = native::cursor_line(&text, view.cursor, &font.widths);
    let scroll = if writing { view.scroll } else { 0 };
    for (row, line) in lines
        .iter()
        .skip(scroll)
        .take(if editing { EDIT_LINES } else { 6 })
        .enumerate()
    {
        let y = if editing { 28.0 } else { 63.0 } + row as f32 * 16.0;
        paint.text(font, &display(line), Vec2::new(63.0, y), ink, 230.0);
        if editing && scroll + row == cursor_line {
            let at = Vec2::new(63.0 + cursor_x as f32, y);
            if let Some(caret) =
                paint.text(font, "_", at, crate::shop_nook::editor::CARET_INK, 230.0)
            {
                paint
                    .commands
                    .entity(caret)
                    .insert(crate::shop_nook::editor::Caret(at));
            }
        }
    }
}

/// The board itself, and what moves on it each frame.
#[derive(Component)]
pub(super) struct BoardRoot;

#[derive(Component, Clone, Copy)]
pub(super) enum Slot {
    Arriving,
    Leaving,
    Hints,
}

/// Slides the board and turns its pages (`menu_info->position`, `control_position`); one
/// pixel is 16 overlay units.
pub(super) fn animate(
    ui: Res<NoticeUi>,
    mut roots: Query<&mut Transform, (With<BoardRoot>, Without<Slot>)>,
    mut slots: Query<(&Slot, &mut Transform), Without<BoardRoot>>,
) {
    let Some(view) = ui.0.as_ref() else { return };
    let motion = view.motion;
    for mut transform in &mut roots {
        transform.translation.y = view.slide.pos * 16.0;
    }
    let side = if motion.now > motion.disp {
        -320.0
    } else {
        320.0
    };
    for (slot, mut transform) in &mut slots {
        match slot {
            Slot::Arriving => transform.translation.x = motion.x * 16.0,
            Slot::Leaving => transform.translation.x = (motion.x + side) * 16.0,
            // The hints are drawn from the screen's frame, not the sliding board's.
            Slot::Hints => {
                transform.translation.y = (motion.control - 2.0 * view.slide.pos) * 16.0;
            }
        }
    }
}
