//! `mEE_editEndChk_ovl_draw`: the "Is this OK?" question and, after one accept, the answer window
//! with its cursor, built from the disc's `lat_kakunin_*` / `lat_sentaku*_*` models.
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use rustimal_logic::edit_end_check::{EditEndCheck, ANSWERS};

use crate::game_disc::GameDisc;
use crate::msg_window::MsgAssets;
use crate::pockets_disc::Paint;
use crate::shop_nook::editor::{Model, Skin, Stores};

/// `mEE_win_data_c` fields in menu pixels.
struct Window {
    win: &'static str,
    cursor: &'static str,
    char_at: Vec2,
    answer: Vec2,
    question: Vec2,
}

/// `win_data[mEE_TYPE_BOARD]` (mail, diary) and `win_data[1]`.
fn window(answers: usize) -> Window {
    if answers <= 2 {
        Window {
            win: "lat_sentaku2_winT_model",
            cursor: "lat_sentaku2_c_model",
            char_at: Vec2::new(-45.0, 43.0),
            answer: Vec2::new(65.0, -74.0),
            question: Vec2::ZERO,
        }
    } else {
        Window {
            win: "lat_sentaku_winT_model",
            cursor: "lat_sentaku_c_model",
            char_at: Vec2::new(-81.0, 58.0),
            answer: Vec2::new(60.0, -71.0),
            question: Vec2::new(1.0, -4.0),
        }
    }
}

/// Spawns the window's pieces as children of `root`, which the caller places at
/// `-check.slide.pos * 16` and despawns each frame. `z` is the window's depth.
pub(crate) fn spawn(
    commands: &mut Commands,
    root: Entity,
    skin: &mut Skin,
    disc: &GameDisc,
    font: &MsgAssets,
    stores: &mut Stores,
    check: &EditEndCheck,
    z: f32,
) {
    let data = window(check.answers);
    let mode = "lat_kakunin_DL_mode";
    let alpha = check.question_alpha();
    let waku = skin.window(
        disc,
        "kakunin-waku",
        &[mode, "lat_kakunin_wakuT_model"],
        [255; 4],
        stores,
    );
    let question = skin.window(
        disc,
        &format!("kakunin-c-{alpha}"),
        &[mode, "lat_kakunin_c_model"],
        [0, 0, 255, alpha],
        stores,
    );
    let place = |commands: &mut Commands, model: &Model, at: Vec3, scale: f32| {
        for (index, (mesh, material)) in model.iter().enumerate() {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(at + Vec3::Z * index as f32 * 0.01)
                    .with_scale(Vec3::new(scale, scale, 1.0)),
                RenderLayers::layer(1),
                ChildOf(root),
            ));
        }
    };
    let at = (data.question * 16.0).extend(z);
    place(commands, &waku, at, 1.0);
    place(commands, &question, at + Vec3::Z * 0.2, 1.0);
    Paint {
        commands,
        meshes: stores.meshes,
        materials: stores.materials,
        root,
    }
    .text(
        font,
        "Is this OK?",
        Vec2::new(107.0 + data.question.x, 194.0 - data.question.y),
        [80, 80, 230, 255],
        z + 2.0,
    );
    if !check.moving_in {
        return;
    }
    let win = skin.window(disc, data.win, &[mode, data.win], [255; 4], stores);
    let cursor = skin.window(disc, data.cursor, &[mode, data.cursor], [255; 4], stores);
    let scale = check.scale;
    place(commands, &win, (data.answer * 16.0).extend(z + 0.4), scale);
    if scale >= 1.0 {
        let at = Vec3::new(
            data.answer.x * 16.0,
            (data.answer.y - check.selected as f32 * 16.0) * 16.0,
            z + 0.6,
        );
        place(commands, &cursor, at, 1.0);
    }
    let mut paint = Paint {
        commands,
        meshes: stores.meshes,
        materials: stores.materials,
        root,
    };
    let x = 160.0 + data.answer.x + data.char_at.x * scale;
    let mut y = 120.0 - data.answer.y - data.char_at.y * scale;
    let size = ((scale * 32.0).round() as u8).max(1);
    for (i, answer) in ANSWERS.iter().take(check.answers).enumerate() {
        let color = if i == check.selected {
            [100, 130, 245, 255]
        } else {
            [165, 185, 185, 255]
        };
        let text = format!(
            "{}{}{answer}",
            crate::messages::markup::LINE_SCALE,
            crate::messages::markup::arg(size)
        );
        paint.text(font, &text, Vec2::new(x, y), color, z + 2.0);
        y += scale * 16.0;
    }
}
