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
}

/// The window currently drawn, and what for.
#[derive(Resource, Default)]
struct Shown {
    key: Option<(String, Option<Entity>)>,
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
    // White, with the intensity as coverage (the font combiner draws the
    // prim colour through TEXEL0's alpha).
    let rgba: Vec<u8> = font.chunks(4).flat_map(|t| [255, 255, 255, t[0]]).collect();
    let font = images.add(Image::new(
        Extent3d {
            width: 192,
            height: 256,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ));
    let mut widths = [CELL.x as u8; 256];
    if let Some(off) = cat
        .get("offset_data$396")
        .and_then(|a| disc.rom.slice(RomSource::Rel, a.offset, 256))
    {
        for (w, o) in widths.iter_mut().zip(off) {
            *w = (CELL.x as u8).saturating_sub(*o);
        }
    }
    let (Some(init), Some(body), Some(plate)) = (
        cat.get("mMsg_init_disp"),
        cat.get("con_kaiwa2_modelT"),
        cat.get("con_kaiwaname_modelT"),
    ) else {
        warn!("talk window not found on the disc");
        return;
    };
    let mut build = |list: u32, prim: [u8; 4]| {
        let sink = crate::disc_gfx::run_display_lists_with_colors(
            &disc.rom,
            &[init.offset, list],
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
    let window = build(body.offset, WINDOW_COLOR);
    let plates = PLATE_COLORS.map(|(bg, _)| build(plate.offset, bg));
    commands.insert_resource(MsgAssets {
        font,
        widths,
        window,
        plates,
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
    let key = dialogue.page().map(|p| (p.to_string(), dialogue.speaker));
    if key == shown.key {
        return;
    }
    if let Some(root) = shown.root.take() {
        commands.entity(root).despawn();
    }
    shown.key = key.clone();
    let Some((page, speaker)) = key else { return };
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
    let mut text = |commands: &mut Commands, s: &str, origin: Vec2, color: [u8; 4], z: f32| {
        let material = text_materials.add(StandardMaterial {
            base_color: Color::srgba_u8(color[0], color[1], color[2], color[3]),
            base_color_texture: Some(assets.font.clone()),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        commands.spawn((
            Mesh3d(meshes.add(text_mesh(s, origin, &assets.widths))),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, 0.0, z),
            layer.clone(),
            ChildOf(root),
        ));
    };
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
        text(&mut commands, v.name, origin, name_color, 2.0);
    }
    let origin = WINDOW_CENTRE - Vec2::new(96.0, 32.0);
    text(&mut commands, page, origin, TEXT_COLOR, 2.0);
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
