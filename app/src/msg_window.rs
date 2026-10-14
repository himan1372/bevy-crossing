//! The talk window from the disc (`m_msg_draw_window.c_inc`,
//! `m_msg_draw_font.c_inc`, `m_font_main.c_inc`).
//!
//! Drawn on its own orthographic camera in the game's screen space: 320 x
//! 240 at 16 units a pixel (`mFont_CulcOrthoMatrix`). The window is
//! `mMsg_init_disp` + `con_kaiwa2_modelT` tinted by the window colour
//! (prim; villagers' talks default to `mDemo_WINDOW_COLOR` 235/255/235),
//! centred at (160, 185.4) (`mMsg_window`); with a speaker, the name plate
//! `con_kaiwaname_modelT` in the speaker's colour (male / female / other,
//! `mMsg_MainSetup_Appear`) and their name at (61 + centring, 64) in the
//! window's space. Text runs from the window centre - (96, 32), 16 pixels
//! a line, in (50, 60, 50) (`mMsg_init_FontColor`).
//!
//! Answers (`m_choice_draw.c_inc`) go in `mChoice_init_disp` +
//! `con_sentaku2_modelT` (prim 255/255/155), sized to the longest answer
//! and the number of lines (`mChoice_Set_DisplayScaleAndDisplayPos`), text
//! in (180, 150, 110), the highlighted one in (120, 50, 50) after the
//! `FONT_nes_tex_choice` cursor in (0, 195, 185). While no answer is
//! wanted the page ends in `FONT_nes_tex_next` at (257, 136) in the
//! window's space, in (0, 0, 255) (`mMsg_DrawWindowTurnButton`).
//!
//! The font is `FONT_nes_tex_font1`: an I4 sheet of 16 x 16 cells of 12 x
//! 16 pixels, character `c` at cell (c % 16, c / 16); with the "cut" flag
//! each is drawn `12 - offset_data[c]` wide from its cell's left edge
//! (`mFont_GetCodeWidth`, `mFontChar_gppDrawRect`).

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ClearColorConfig, ScalingMode};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use disc::RomSource;

use crate::dialogue::{DialogueBox, DialogueState};
use crate::disc_gfx::Segments;
use crate::game_disc::GameDisc;
use crate::n64_material::N64Material;
use crate::npc::Villager;
use crate::states::AppState;

type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;

/// The overlay's render layer.
const LAYER: usize = 1;
/// `mFont_SCALE_F`: projection units per screen pixel.
const SCALE: f32 = 16.0;
const SCREEN: Vec2 = Vec2::new(320.0, 240.0);
/// `mMsg_window.center_x/y`.
const WINDOW_CENTRE: Vec2 = Vec2::new(160.0, 185.4);
/// `mFont_TEX_CHAR_WIDTH/HEIGHT`.
const CELL: Vec2 = Vec2::new(12.0, 16.0);
/// `mDemo_WINDOW_COLOR_*`.
const WINDOW_COLOR: [u8; 4] = [235, 255, 235, 255];
/// `mMsg_init_FontColor`.
const TEXT_COLOR: [u8; 4] = [50, 60, 50, 255];
/// Name plate (background, text) by sex: male, female, other.
const PLATE_COLORS: [([u8; 4], [u8; 4]); 3] = [
    ([70, 245, 255, 255], [0, 0, 15, 255]),
    ([235, 140, 210, 255], [45, 0, 30, 255]),
    ([185, 255, 0, 255], [0, 30, 0, 255]),
];
/// `ANIMAL_NAME_LEN`.
const NAME_LEN: f32 = 8.0;
/// `mChoice_ct` colours: text, highlighted text, cursor.
const CHOICE_TEXT: [u8; 4] = [180, 150, 110, 255];
const CHOICE_SELECTED: [u8; 4] = [120, 50, 50, 255];
const CHOICE_CURSOR: [u8; 4] = [0, 195, 185, 255];
/// `mMsg_window.continue_button_color`, and where the arrow sits.
const NEXT_COLOR: [u8; 4] = [0, 0, 255, 255];
const NEXT_AT: Vec2 = Vec2::new(257.0, 136.0);
/// Font marks are 16 x 16 (`mFont_Get_MarkTex_sizeW/H`).
const MARK: f32 = 16.0;

/// The 2D overlay camera (not the town camera).
#[derive(Component)]
pub struct OverlayCamera;

/// What the window is drawn with.
#[derive(Resource)]
struct MsgAssets {
    font: Handle<Image>,
    /// `mFont_GetCodeWidth(c, TRUE)`.
    widths: [u8; 256],
    window: Model,
    /// Name plate per `PLATE_COLORS` entry.
    plates: [Model; 3],
    choice_body: Model,
    /// `FONT_nes_tex_choice` / `FONT_nes_tex_next`.
    cursor: Handle<Image>,
    next: Handle<Image>,
}

/// The window currently drawn, and what for.
#[derive(Resource, Default)]
struct Shown {
    key: Option<(String, Option<Entity>, Option<usize>)>,
    root: Option<Entity>,
}

/// Screen pixel (y down) to overlay units.
fn screen(p: Vec2) -> Vec3 {
    Vec3::new(
        (p.x - SCREEN.x * 0.5) * SCALE,
        (SCREEN.y * 0.5 - p.y) * SCALE,
        0.0,
    )
}

fn setup(
    mut commands: Commands,
    disc: Res<GameDisc>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
    mut boxes: Query<&mut Node, With<DialogueBox>>,
) {
    let cat = &disc.catalog;
    let Some(font) = cat.get("FONT_nes_tex_font1").and_then(|a| {
        let data = disc.rom.slice(RomSource::Rel, a.offset, a.size)?;
        disc::gx_texture::decode(data, disc::gx_texture::GxFormat::I4, 192, 256, None)
    }) else {
        warn!("font not found on the disc; keeping the plain talk box");
        return;
    };
    let font = images.add(coverage_image(&font, 192, 256));
    let mark = |name: &str| {
        cat.get(name)
            .and_then(|a| disc.rom.slice(RomSource::Rel, a.offset, a.size))
            .and_then(|d| disc::gx_texture::decode(d, disc::gx_texture::GxFormat::I4, 16, 16, None))
            .map(|t| coverage_image(&t, 16, 16))
    };
    let (Some(cursor), Some(next)) = (mark("FONT_nes_tex_choice"), mark("FONT_nes_tex_next"))
    else {
        warn!("font marks not found on the disc");
        return;
    };
    let (cursor, next) = (images.add(cursor), images.add(next));
    let mut widths = [CELL.x as u8; 256];
    if let Some(off) = cat
        .get("offset_data$396")
        .and_then(|a| disc.rom.slice(RomSource::Rel, a.offset, 256))
    {
        for (w, o) in widths.iter_mut().zip(off) {
            *w = (CELL.x as u8).saturating_sub(*o);
        }
    }
    let (Some(init), Some(body), Some(plate), Some(choice_init), Some(choice)) = (
        cat.get("mMsg_init_disp"),
        cat.get("con_kaiwa2_modelT"),
        cat.get("con_kaiwaname_modelT"),
        cat.get("mChoice_init_disp"),
        cat.get("con_sentaku2_modelT"),
    ) else {
        warn!("talk window not found on the disc");
        return;
    };
    let mut build = |setup: u32, list: u32, prim: [u8; 4]| {
        let sink = crate::disc_gfx::run_display_lists_with_colors(
            &disc.rom,
            &[setup, list],
            prim,
            255,
            [0; 4],
            &[],
        );
        crate::disc_town::sink_meshes_scaled(
            &disc,
            &sink,
            None,
            true,
            &Segments::default(),
            1.0,
            (&mut *meshes, &mut *materials, &mut *images),
        )
    };
    let window = build(init.offset, body.offset, WINDOW_COLOR);
    let plates = PLATE_COLORS.map(|(bg, _)| build(init.offset, plate.offset, bg));
    // con_sentaku2_modelT sets its own prim (255, 255, 155).
    let choice_body = build(choice_init.offset, choice.offset, [255; 4]);
    commands.insert_resource(MsgAssets {
        font,
        widths,
        window,
        plates,
        choice_body,
        cursor,
        next,
    });
    // The overlay camera: the game's 2D projection, drawn over the town.
    commands.spawn((
        OverlayCamera,
        Camera3d::default(),
        Camera {
            order: 10,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        bevy::core_pipeline::tonemapping::Tonemapping::None,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: SCREEN.y * SCALE,
            },
            near: -1000.0,
            far: 1000.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::default(),
        RenderLayers::layer(LAYER),
    ));
    // The disc window replaces the plain box.
    for mut node in &mut boxes {
        node.display = Display::None;
    }
}

/// An I4 texture as white with its intensity as coverage (the font
/// combiner draws the prim colour through TEXEL0's alpha).
fn coverage_image(texels: &[u8], width: u32, height: u32) -> Image {
    let rgba: Vec<u8> = texels
        .chunks(4)
        .flat_map(|t| [255, 255, 255, t[0]])
        .collect();
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// One `size` quad at `at` (screen pixels), the whole texture on it.
fn quad_mesh(at: Vec2, size: Vec2) -> Mesh {
    let mut pos = Vec::new();
    for (dx, dy) in [(0.0, 0.0), (size.x, 0.0), (size.x, size.y), (0.0, size.y)] {
        let p = screen(at + Vec2::new(dx, dy));
        pos.push([p.x, p.y, 0.0]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
    )
    .with_inserted_indices(Indices::U32(vec![0, 2, 1, 0, 3, 2]))
}

/// `mChoice_Set_DisplayScaleAndDisplayPos`: the window's centre (after
/// it slides in), its x/y scale, and where the first answer starts (all
/// screen pixels).
fn choice_layout(answers: &[&str], widths: &[u8; 256]) -> (Vec2, Vec2, Vec2) {
    let n = answers.len() as f32;
    let dw = answers
        .iter()
        .map(|a| text_width(a, widths))
        .fold(0.0, f32::max);
    let dot_w = (dw - 24.0) / 96.0;
    let dot_h = (n * 16.0 - 32.0) / 32.0;
    let scale = Vec2::new(
        dot_w * 0.583_333_4 + 1.0 / 2.4,
        dot_h * 0.314_285_7 + 0.685_714_3,
    );
    let mut text = Vec2::new(dot_w * -48.0 + 152.0, dot_h * -20.057_144 + 105.0);
    let mut centre = Vec2::new(242.0 + dot_w * -35.0, 169.0 + dot_h * 20.057_144);
    if answers.len() > 4 {
        text.y += (n - 3.0) * 2.0;
        centre.y -= (n - 4.0) * 16.0;
    }
    (centre, scale, text)
}

/// A text mesh: one quad per character from `origin` (screen pixels),
/// `\n` starting a new 16-pixel line.
fn text_mesh(text: &str, origin: Vec2, widths: &[u8; 256]) -> Mesh {
    let (mut pos, mut uv, mut idx) = (Vec::new(), Vec::new(), Vec::new());
    let mut cursor = origin;
    for ch in text.chars() {
        if ch == '\n' {
            cursor = Vec2::new(origin.x, cursor.y + CELL.y);
            continue;
        }
        let c = crate::messages::ac_code(ch);
        let w = widths[c as usize] as f32;
        let cell = Vec2::new((c % 16) as f32 * CELL.x, (c / 16) as f32 * CELL.y);
        let base = pos.len() as u32;
        for (dx, dy) in [(0.0, 0.0), (w, 0.0), (w, CELL.y), (0.0, CELL.y)] {
            let p = screen(cursor + Vec2::new(dx, dy));
            pos.push([p.x, p.y, 0.0]);
            uv.push([(cell.x + dx) / 192.0, (cell.y + dy) / 256.0]);
        }
        idx.extend([base, base + 2, base + 1, base, base + 3, base + 2]);
        cursor.x += w;
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
    .with_inserted_indices(Indices::U32(idx))
}

fn text_width(text: &str, widths: &[u8; 256]) -> f32 {
    text.chars()
        .map(|c| widths[crate::messages::ac_code(c) as usize] as f32)
        .sum()
}

/// Shows the window for the dialogue's current page.
#[allow(clippy::too_many_arguments)]
fn draw_window(
    mut commands: Commands,
    assets: Res<MsgAssets>,
    dialogue: Res<DialogueState>,
    villagers: Query<&Villager>,
    mut shown: ResMut<Shown>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut text_materials: ResMut<Assets<StandardMaterial>>,
) {
    let key = dialogue.page().map(|p| {
        (
            p.to_string(),
            dialogue.speaker,
            dialogue.choices().map(|_| dialogue.choice),
        )
    });
    if key == shown.key {
        return;
    }
    if let Some(root) = shown.root.take() {
        commands.entity(root).despawn();
    }
    shown.key = key.clone();
    let Some((page, speaker, _)) = key else {
        return;
    };
    let layer = RenderLayers::layer(LAYER);
    let root = commands
        .spawn((
            Name::new("talk window"),
            Transform::default(),
            Visibility::default(),
            layer.clone(),
        ))
        .id();
    shown.root = Some(root);
    let window_at = screen(WINDOW_CENTRE);
    let model = |commands: &mut Commands, model: &Model, at: Vec3, z: f32| {
        for (mesh, material) in model {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(at.with_z(z)),
                layer.clone(),
                ChildOf(root),
            ));
        }
    };
    model(&mut commands, &assets.window, window_at, 0.0);
    // Text and marks: a mesh in screen space, `image` tinted `color`.
    let mut put =
        |commands: &mut Commands, mesh: Mesh, image: &Handle<Image>, color: [u8; 4], z: f32| {
            let material = text_materials.add(StandardMaterial {
                base_color: Color::srgba_u8(color[0], color[1], color[2], color[3]),
                base_color_texture: Some(image.clone()),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            });
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                Transform::from_xyz(0.0, 0.0, z),
                layer.clone(),
                ChildOf(root),
            ));
        };
    let font = assets.font.clone();
    let widths = assets.widths;
    let mut page = page.as_str();
    if let Some(v) = speaker.and_then(|e| villagers.get(e).ok()) {
        // The plain box writes "Name: "; the name plate says it here.
        page = page
            .strip_prefix(v.name)
            .and_then(|p| p.strip_prefix(": "))
            .unwrap_or(page);
        let sex = if crate::roster::looks_female(v.looks) {
            1
        } else {
            0
        };
        let (_, name_color) = PLATE_COLORS[sex];
        model(&mut commands, &assets.plates[sex], window_at, 1.0);
        // mMsg_Set_client_actor_p: centred in 9 pixels a letter, in the
        // window's space (drawn under mMsg_SetMatrix).
        let width = text_width(v.name, &assets.widths);
        let local = Vec2::new(61.0 + (9.0 * NAME_LEN - width) * 0.5, 64.0);
        let origin = WINDOW_CENTRE + local - SCREEN * 0.5;
        put(
            &mut commands,
            text_mesh(v.name, origin, &widths),
            &font,
            name_color,
            2.0,
        );
    }
    let origin = WINDOW_CENTRE - Vec2::new(96.0, 32.0);
    put(
        &mut commands,
        text_mesh(page, origin, &widths),
        &font,
        TEXT_COLOR,
        2.0,
    );

    match dialogue.choices() {
        Some(answers) => {
            let (centre, scale, at) = choice_layout(answers, &assets.widths);
            for (mesh, material) in &assets.choice_body {
                commands.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(screen(centre).with_z(3.0))
                        .with_scale(scale.extend(1.0)),
                    layer.clone(),
                    ChildOf(root),
                ));
            }
            // Answers are laid out in the choice window's space.
            let mut y = at.y;
            for (i, answer) in answers.iter().enumerate() {
                let origin = Vec2::new(at.x, y) + centre - SCREEN * 0.5;
                let color = if i == dialogue.choice {
                    let at = origin - Vec2::new(16.0, 0.0);
                    put(
                        &mut commands,
                        quad_mesh(at, Vec2::splat(MARK)),
                        &assets.cursor,
                        CHOICE_CURSOR,
                        5.0,
                    );
                    CHOICE_SELECTED
                } else {
                    CHOICE_TEXT
                };
                put(
                    &mut commands,
                    text_mesh(answer, origin, &widths),
                    &font,
                    color,
                    4.0,
                );
                y += CELL.y;
            }
        }
        None => {
            let at = WINDOW_CENTRE + NEXT_AT - SCREEN * 0.5;
            put(
                &mut commands,
                quad_mesh(at, Vec2::splat(MARK)),
                &assets.next,
                NEXT_COLOR,
                3.0,
            );
        }
    }
}

pub struct MsgWindowPlugin;

impl Plugin for MsgWindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Shown>()
            .add_systems(
                OnEnter(AppState::InGame),
                setup.run_if(resource_exists::<GameDisc>),
            )
            .add_systems(Update, draw_window.run_if(resource_exists::<MsgAssets>));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three answers about 90 px wide (Philip's retail shot: "I need
    /// work!" / "I'm bored!" / "My mistake!") sit to the right of the
    /// talk text, their first line near (177, 154) on screen.
    #[test]
    fn choice_window_sits_right_of_the_talk_text() {
        let mut widths = [0u8; 256];
        widths[b'x' as usize] = 9;
        let answers = ["xxxxxxxxxx"; 3];
        let (centre, scale, text) = choice_layout(&answers, &widths);
        let first = text + centre - SCREEN * 0.5;
        assert!((first.x - 177.0).abs() < 2.0, "{first:?}");
        assert!((first.y - 154.0).abs() < 2.0, "{first:?}");
        assert!(scale.x > 0.5 && scale.y > 0.8, "{scale:?}");
    }
}
