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
use crate::input::{Action, Actions};
use crate::messages::markup::{self, Item};
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
pub(crate) struct MsgAssets {
    pub(crate) font: Handle<Image>,
    /// `mFont_GetCodeWidth(c, TRUE)`.
    pub(crate) widths: [u8; 256],
    window: Model,
    /// Name plate per `PLATE_COLORS` entry.
    plates: [Model; 3],
    pub(crate) choice_body: Model,
    /// Bounds cached before the meshes are extracted to RenderWorld.
    pub(crate) choice_bounds: (Vec3, Vec3),
    /// `FONT_nes_tex_choice` / `FONT_nes_tex_next`.
    pub(crate) cursor: Handle<Image>,
    next: Handle<Image>,
}

/// The window currently drawn, and what for.
#[derive(Resource, Default)]
struct Shown {
    choice_open: bool,
    key: Option<(String, Option<Entity>, Option<usize>, usize)>,
    root: Option<Entity>,
}

/// Screen pixel (y down) to overlay units.
pub(crate) fn screen(p: Vec2) -> Vec3 {
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
    let mut choice_bounds = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
    for (mesh, _) in &choice_body {
        if let Some(points) = meshes
            .get(mesh)
            .and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION))
            .and_then(|v| v.as_float3())
        {
            for &point in points {
                choice_bounds.0 = choice_bounds.0.min(Vec3::from(point));
                choice_bounds.1 = choice_bounds.1.max(Vec3::from(point));
            }
        }
    }
    commands.insert_resource(MsgAssets {
        font,
        widths,
        window,
        plates,
        choice_body,
        choice_bounds,
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
pub(crate) fn quad_mesh(at: Vec2, size: Vec2) -> Mesh {
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
pub(crate) fn choice_layout(answers: &[&str], widths: &[u8; 256]) -> (Vec2, Vec2, Vec2) {
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
/// `\n` starting a new 16-pixel line, the first `limit` pieces of the page
/// (`messages::markup`), each character in its own colour or `color`.
pub(crate) fn text_mesh(
    text: &str,
    origin: Vec2,
    widths: &[u8; 256],
    color: [u8; 4],
    limit: usize,
) -> Mesh {
    let (mut pos, mut uv, mut idx, mut col) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut cursor = origin;
    let (mut line_scale, mut char_scale) = (1.0, 1.0);
    for item in markup::parse(text).into_iter().take(limit) {
        let (ch, rgb) = match item {
            Item::Char(ch, rgb) => (ch, rgb),
            Item::NewLine => {
                cursor = Vec2::new(origin.x, cursor.y + CELL.y);
                line_scale = 1.0;
                char_scale = 1.0;
                continue;
            }
            Item::CharScale(scale) => {
                char_scale = f32::from(scale) / 32.0;
                continue;
            }
            Item::LineScale(scale) => {
                line_scale = f32::from(scale) / 32.0;
                continue;
            }
            Item::Pause(_) | Item::Status(_) => continue,
        };
        let [r, g, b] = rgb.unwrap_or([color[0], color[1], color[2]]);
        let tint = Color::srgba_u8(r, g, b, color[3])
            .to_linear()
            .to_f32_array();
        let c = crate::messages::ac_code(ch);
        let w = widths[c as usize] as f32;
        let scale = line_scale * char_scale;
        char_scale = 1.0;
        let cell = Vec2::new((c % 16) as f32 * CELL.x, (c / 16) as f32 * CELL.y);
        let base = pos.len() as u32;
        for (dx, dy) in [(0.0, 0.0), (w, 0.0), (w, CELL.y), (0.0, CELL.y)] {
            let p = screen(cursor + Vec2::new(dx, dy) * scale);
            pos.push([p.x, p.y, 0.0]);
            // Half a texel inside the cell: filtered, scaled text would
            // otherwise pick up the neighbouring glyphs' edge rows.
            let u = (cell.x + dx.clamp(0.5, (w - 0.5).max(0.5))) / 192.0;
            let v = (cell.y + dy.clamp(0.5, CELL.y - 0.5)) / 256.0;
            uv.push([u, v]);
            col.push(tint);
        }
        idx.extend([base, base + 2, base + 1, base, base + 3, base + 2]);
        cursor.x += w * scale;
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, col)
    .with_inserted_indices(Indices::U32(idx))
}

pub(crate) fn text_width(text: &str, widths: &[u8; 256]) -> f32 {
    let (mut line, mut next, mut width, mut maximum) = (1.0, 1.0, 0.0_f32, 0.0_f32);
    for item in markup::parse(text) {
        match item {
            Item::Char(c, _) => {
                width += widths[crate::messages::ac_code(c) as usize] as f32 * line * next;
                next = 1.0;
            }
            Item::CharScale(v) => next = f32::from(v) / 32.0,
            Item::LineScale(v) => line = f32::from(v) / 32.0,
            Item::NewLine => {
                maximum = maximum.max(width);
                width = 0.0;
                line = 1.0;
                next = 1.0;
            }
            Item::Pause(_) | Item::Status(_) => {}
        }
    }
    maximum.max(width)
}

/// The page the window shows: the plain box's "Name: " dropped, since the
/// name plate says who's talking.
fn window_page<'a>(page: &'a str, speaker: Option<&Villager>) -> &'a str {
    speaker
        .and_then(|v| page.strip_prefix(v.name))
        .and_then(|p| p.strip_prefix(": "))
        .unwrap_or(page)
}

/// How far the page has been typed out (`mMsg_Main_Cursol_ControlCursol`).
#[derive(Resource, Default)]
struct Reveal {
    page: Option<String>,
    speaker: Option<Entity>,
    /// Pieces of the page shown so far (`markup::Item`s).
    shown: usize,
    total: usize,
    /// `cursor_timer`, in 60 Hz ticks.
    timer: f32,
    line_scale: u8,
    char_scale: u8,
}

impl Reveal {
    fn done(&self) -> bool {
        self.shown >= self.total
    }
}

/// Types the page out: a character every 2 ticks (30 a second), twice as
/// fast while B (run) is held, waiting out `CURSOR_SET_TIME` pauses.
fn advance_reveal(
    time: Res<Time>,
    actions: Res<Actions>,
    villagers: Query<&Villager>,
    disc: Option<Res<GameDisc>>,
    mut dialogue: ResMut<DialogueState>,
    mut reveal: ResMut<Reveal>,
) {
    let Some(page) = dialogue.page() else {
        if reveal.speaker.is_some() {
            crate::sound::voice_end();
        }
        *reveal = Reveal::default();
        dialogue.revealing = false;
        return;
    };
    let speaker = dialogue.speaker.and_then(|e| villagers.get(e).ok());
    let page = window_page(page, speaker).to_string();
    if reveal.page.as_deref() != Some(page.as_str()) || reveal.speaker != dialogue.speaker {
        let items = markup::parse(&page);
        if reveal.speaker.is_some() {
            crate::sound::voice_end();
        }
        if let Some(speaker) = speaker {
            // aNPC_draw_data_c::voice_type (0x62), not the roster index.
            let voice_id = speaker
                .npc_index
                .and_then(|index| {
                    let disc = disc.as_ref()?;
                    let table = disc.catalog.get("npc_draw_data_tbl")?;
                    let bytes = disc.rom.slice(table.source, table.offset, table.size)?;
                    let offset = (index as usize).checked_mul(0x6C)?.checked_add(0x62)?;
                    let data: [u8; 2] = bytes.get(offset..offset + 2)?.try_into().ok()?;
                    Some(u16::from_be_bytes(data))
                })
                .unwrap_or(0);
            crate::sound::voice_begin(
                rustimal_logic::na_voice::villager_spec(speaker.looks),
                voice_id,
            );
        } else if let Some((npc, sound_id)) = dialogue.external_voice {
            let voice_id = disc
                .as_ref()
                .and_then(|disc| {
                    let table = disc.catalog.get("npc_draw_data_tbl")?;
                    let offset = npc * 0x6C + 0x62;
                    let b = disc
                        .rom
                        .slice(table.source, table.offset.checked_add(offset)?, 2)?;
                    Some(u16::from_be_bytes([b[0], b[1]]))
                })
                .unwrap_or(0);
            crate::sound::voice_begin(sound_id, voice_id);
        }
        *reveal = Reveal {
            total: items.len(),
            page: Some(page.clone()),
            speaker: dialogue.speaker,
            line_scale: 32,
            char_scale: 32,
            ..default()
        };
    }
    let fast = if actions.pressed(Action::Run) {
        2.0
    } else {
        1.0
    };
    reveal.timer -= time.delta_secs() * 60.0 * fast;
    let items = markup::parse(&page);
    // Control boundaries terminate phoneme lookahead. Supported Unicode
    // returns through the same charset mapping used to draw the disc font.
    let phoneme = |i: usize| match items.get(i) {
        Some(Item::Char(ch, _)) => {
            crate::messages::ac_code_opt(*ch).map_or(0x85, rustimal_logic::na_voice::phoneme_of)
        }
        Some(Item::NewLine) => 0x84,
        _ => 0xFF,
    };
    let was_done = reveal.done();
    let speaking = speaker.is_some() || dialogue.external_name.is_some();
    while reveal.timer <= 0.0 && !reveal.done() {
        match items[reveal.shown] {
            Item::Char(_, rgb) => {
                reveal.timer += 2.0;
                if speaking {
                    let i = reveal.shown;
                    let scale = (f32::from(reveal.line_scale) * f32::from(reveal.char_scale) / 32.0
                        + 0.5)
                        .clamp(0.0, 255.0) as u8;
                    crate::sound::voice_character(
                        [phoneme(i), phoneme(i + 1), phoneme(i + 2)],
                        scale,
                        phoneme(i) == 0x80
                            && scale == 32
                            && rgb.unwrap_or([TEXT_COLOR[0], TEXT_COLOR[1], TEXT_COLOR[2]])
                                == [50, 60, 50],
                    );
                }
                reveal.char_scale = 32;
            }
            Item::Pause(t) => reveal.timer += t as f32,
            Item::NewLine => {
                reveal.line_scale = 32;
                reveal.char_scale = 32;
                if speaking {
                    crate::sound::voice_character([0x84, 0xFF, 0xFF], 32, false);
                }
            }
            Item::Status(status) => {
                if speaking {
                    crate::sound::voice_status(status);
                }
            }
            Item::CharScale(scale) => reveal.char_scale = scale,
            Item::LineScale(scale) => reveal.line_scale = scale,
        }
        reveal.shown += 1;
    }
    if !was_done && reveal.done() && speaking {
        crate::sound::voice_end();
    }
    dialogue.revealing = !reveal.done();
}

/// Shows the window for the dialogue's current page.
#[allow(clippy::too_many_arguments)]
fn draw_window(
    mut commands: Commands,
    assets: Res<MsgAssets>,
    dialogue: Res<DialogueState>,
    reveal: Res<Reveal>,
    villagers: Query<&Villager>,
    mut shown: ResMut<Shown>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut text_materials: ResMut<Assets<StandardMaterial>>,
) {
    let choice_open = reveal.done() && dialogue.choices().is_some();
    if choice_open && !shown.choice_open {
        crate::sound::se(crate::sound::se_id::CHOICE_OPEN);
    }
    shown.choice_open = choice_open;
    let key = dialogue.page().map(|p| {
        (
            p.to_string(),
            dialogue.speaker,
            dialogue.choices().map(|_| dialogue.choice),
            reveal.shown,
        )
    });
    if key == shown.key {
        return;
    }
    if let Some(root) = shown.root.take() {
        commands.entity(root).despawn();
    }
    shown.key = key.clone();
    let Some((page, speaker, _, typed)) = key else {
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
    let speaker = speaker.and_then(|e| villagers.get(e).ok());
    let page = window_page(&page, speaker);
    if speaker.is_some() || dialogue.external_name.is_some_and(|name| !name.is_empty()) {
        let sex = if speaker.is_some_and(|v| crate::roster::looks_female(v.looks)) {
            1
        } else {
            0
        };
        let (_, name_color) = PLATE_COLORS[sex];
        model(&mut commands, &assets.plates[sex], window_at, 1.0);
        // mMsg_Set_client_actor_p: centred in 9 pixels a letter, in the
        // window's space (drawn under mMsg_SetMatrix).
        let name = speaker
            .map(|v| v.name)
            .or(dialogue.external_name)
            .unwrap_or_default();
        let width = text_width(name, &assets.widths);
        let local = Vec2::new(61.0 + (9.0 * NAME_LEN - width) * 0.5, 64.0);
        let origin = WINDOW_CENTRE + local - SCREEN * 0.5;
        put(
            &mut commands,
            text_mesh(name, origin, &widths, name_color, usize::MAX),
            &font,
            [255; 4],
            2.0,
        );
    }
    let origin = WINDOW_CENTRE - Vec2::new(96.0, 32.0);
    put(
        &mut commands,
        text_mesh(page, origin, &widths, TEXT_COLOR, typed),
        &font,
        [255; 4],
        2.0,
    );

    // The arrow and the answers wait for the page to finish typing.
    if !reveal.done() {
        return;
    }
    match dialogue.choices() {
        Some(answers) => {
            let answers: Vec<&str> = answers.iter().map(String::as_str).collect();
            let (centre, scale, at) = choice_layout(&answers, &assets.widths);
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
                    text_mesh(answer, origin, &widths, color, usize::MAX),
                    &font,
                    [255; 4],
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

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct RevealSet;

pub struct MsgWindowPlugin;

impl Plugin for MsgWindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Shown>()
            .init_resource::<Reveal>()
            .add_systems(
                OnEnter(AppState::InGame),
                setup.run_if(resource_exists::<GameDisc>),
            )
            .add_systems(
                Update,
                (advance_reveal.in_set(RevealSet), draw_window)
                    .chain()
                    .run_if(resource_exists::<MsgAssets>),
            );
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
