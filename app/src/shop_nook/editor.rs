//! The retail editor's character grid (`m_editor_ovl.c`) drawn from the disc:
//! the controller panel, the L/R/A/B/X/Y/Start buttons and their pressed
//! faces, the D-pad and stick, and the ten by four keys with their letters.
//! The state machine behind it is `rustimal_logic::editor_keyboard`.
use std::collections::HashMap;
use std::sync::Arc;

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use disc::RomSource;
use rustimal_logic::editor_keyboard as kb;

use crate::game_disc::GameDisc;
use crate::msg_window::MsgAssets;
use crate::n64_material::N64Material;
use crate::pockets_disc::Paint;

pub(super) type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;

/// What the on-screen controller shows as pressed.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Held {
    pub a: bool,
    pub b: bool,
    pub l: bool,
    pub y: bool,
    pub pad: Option<kb::Direction>,
    pub stick: Option<kb::Direction>,
}

/// Disc-built models, kept between redraws.
#[derive(Default)]
pub(super) struct Skin {
    models: HashMap<String, Model>,
    key_image: Option<Handle<Image>>,
}

/// The pieces every draw needs.
pub(super) struct Stores<'a> {
    pub meshes: &'a mut bevy::asset::Assets<Mesh>,
    pub n64: &'a mut bevy::asset::Assets<N64Material>,
    pub images: &'a mut bevy::asset::Assets<Image>,
    pub materials: &'a mut bevy::asset::Assets<StandardMaterial>,
}

/// Menu depth of the editor over the window behind it.
pub(super) const Z: f32 = 210.0;
/// Overlay units a screen pixel spans (`msg_window::SCALE`).
const PX: f32 = 16.0;

/// `mED_StringsDraw_keyboard`.
const COLUMN_X: [f32; kb::COLUMNS] = [
    60.0, 76.0, 92.0, 108.0, 124.0, 140.0, 156.0, 172.0, 188.0, 204.0,
];
const ROW_Y: [f32; kb::ROWS] = [133.0, 149.0, 165.0, 181.0];
const ROW_SLIDE: [f32; kb::ROWS] = [0.0, 3.0, 7.0, 10.0];
/// `model_type` in `mED_KeyDraw_keyboard`: which diagonal splits each key.
const KEY_DIAGONAL: [u8; kb::COLUMNS * kb::ROWS] = [
    0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0,
    1, 1, 1, 1, 0, 0, 0, 0,
];

impl Skin {
    /// A model of display lists run together, with the primitive colour game
    /// code sets first and a texture in segment 8 where the list wants one.
    fn model(
        &mut self,
        disc: &GameDisc,
        key: String,
        names: &[&str],
        prim: [u8; 4],
        segment8: Option<&str>,
        assets: &mut Stores,
    ) -> Model {
        if let Some(model) = self.models.get(&key) {
            return model.clone();
        }
        let lists: Vec<u32> = names
            .iter()
            .filter_map(|name| disc.catalog.get(name).map(|s| s.offset))
            .collect();
        let sink = crate::disc_gfx::run_display_lists_with_prim(&disc.rom, &lists, prim, 255);
        let mut segments = crate::disc_gfx::Segments::default();
        if let Some(texture) = segment8.and_then(|name| disc.catalog.get(name)) {
            if let Some(bytes) = disc.rom.slice(RomSource::Rel, texture.offset, texture.size) {
                segments.set_segment(8, Arc::from(bytes));
            }
        }
        let model = crate::disc_town::sink_meshes_scaled(
            disc,
            &sink,
            None,
            true,
            &segments,
            PX,
            (&mut *assets.meshes, &mut *assets.n64, &mut *assets.images),
        );
        self.models.insert(key, model.clone());
        model
    }

    /// A window frame: its own mode list and model, run together.
    pub(super) fn window(
        &mut self,
        disc: &GameDisc,
        key: &str,
        names: &[&str],
        prim: [u8; 4],
        assets: &mut Stores,
    ) -> Model {
        self.model(disc, format!("window|{key}"), names, prim, None, assets)
    }

    /// A cached model with the plain draw state of the editor (`kai_sousa_mode`).
    fn part(
        &mut self,
        disc: &GameDisc,
        list: &str,
        prim: [u8; 4],
        segment8: Option<&str>,
        assets: &mut Stores,
    ) -> Model {
        let key = format!("{list}|{prim:?}|{segment8:?}");
        self.model(disc, key, &["kai_sousa_mode", list], prim, segment8, assets)
    }

    /// `testbutton`: the 16 by 16 key face, mirrored across each key.
    fn key_image(&mut self, disc: &GameDisc, assets: &mut Stores) -> Option<Handle<Image>> {
        if self.key_image.is_none() {
            let symbol = disc.catalog.get("testbutton")?;
            let bytes = disc.rom.slice(RomSource::Rel, symbol.offset, symbol.size)?;
            let rgba =
                disc::gx_texture::decode(bytes, disc::gx_texture::GxFormat::I4, 16, 16, None)?;
            let mut image = Image::new(
                Extent3d {
                    width: 16,
                    height: 16,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                rgba,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD,
            );
            image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::MirrorRepeat,
                address_mode_v: ImageAddressMode::MirrorRepeat,
                ..ImageSamplerDescriptor::linear()
            });
            self.key_image = Some(assets.images.add(image));
        }
        self.key_image.clone()
    }
}

/// The window behind the editor (`pwd_win_*` / `pas_win_*`).
pub(super) fn place_frame(commands: &mut Commands, root: Entity, model: &Model) {
    place_frame_at(commands, root, model, 0.0);
}

pub(super) fn place_frame_at(commands: &mut Commands, root: Entity, model: &Model, dz: f32) {
    place(
        commands,
        root,
        model,
        Vec3::new(0.0, 0.0, 200.0 + dz),
        false,
    );
}

fn place(commands: &mut Commands, root: Entity, model: &Model, at: Vec3, rotate_half_turn: bool) {
    for (index, (mesh, material)) in model.iter().enumerate() {
        let mut transform = Transform::from_translation(at + Vec3::Z * index as f32 * 0.01);
        if rotate_half_turn {
            transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
        }
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            transform,
            RenderLayers::layer(1),
            ChildOf(root),
        ));
    }
}

fn small(text: &str) -> String {
    format!(
        "{}{}{text}",
        crate::messages::markup::LINE_SCALE,
        crate::messages::markup::arg(16)
    )
}

/// The controller's face for the held stick direction (`mED_KeyDraw_3D_stick`).
fn stick_texture(direction: Option<kb::Direction>) -> &'static str {
    match direction {
        None => "kai_sousa_3Dst_tex_rgb_ia8",
        Some(kb::Direction::Left | kb::Direction::Right) => "kai_sousa_3Dst4_tex_rgb_ia8",
        Some(kb::Direction::Up) => "kai_sousa_3Dst2_tex_rgb_ia8",
        Some(kb::Direction::Down) => "kai_sousa_3Dst6_tex_rgb_ia8",
    }
}

/// `mED_KeyDraw_J_key`: the D-pad face.
fn pad_texture(direction: Option<kb::Direction>) -> &'static str {
    match direction {
        None => "kai_sousa_controllpad1_tex_rgb_ia8",
        Some(kb::Direction::Left) => "kai_sousa_controllpad4_tex_rgb_ia8",
        Some(kb::Direction::Down) => "kai_sousa_controllpad3_tex_rgb_ia8",
        Some(kb::Direction::Up) => "kai_sousa_controllpad2_tex_rgb_ia8",
        Some(kb::Direction::Right) => "kai_sousa_controllpad5_tex_rgb_ia8",
    }
}

/// Spawns the whole editor under `root`: panel, buttons, keys and letters.
pub(super) fn spawn(
    commands: &mut Commands,
    root: Entity,
    skin: &mut Skin,
    disc: &GameDisc,
    font: &MsgAssets,
    keyboard: &kb::Keyboard,
    held: Held,
    assets: &mut Stores,
) {
    let white = [255, 255, 255, 255];
    let at = |z: f32| Vec3::new(0.0, 0.0, Z + z);

    // The grey panel and the letter board.
    for (n, list) in [
        "kai_sousa_shitaT_model",
        "kai_sousa_controllerT_model",
        "kai_sousa_controller2T_model",
        "kai_sousa_mojibanT_model",
    ]
    .into_iter()
    .enumerate()
    {
        let model = skin.part(disc, list, white, None, assets);
        place(commands, root, &model, at(n as f32 * 0.1), false);
    }

    // L: the case button and its frame (`mED_KeyDraw_L_button`).
    let upper = keyboard.shift == kb::Shift::Upper;
    let l_prim = if upper {
        [215, 30, 30, 255]
    } else {
        [30, 30, 215, 255]
    };
    let model = skin.part(disc, "kai_sousa_lwaku_model", l_prim, None, assets);
    place(commands, root, &model, at(1.0), false);
    if keyboard.mode == kb::InputMode::Letter {
        let caps = if upper {
            "kai_sousa_small_tex_rgb_i4"
        } else {
            "kai_sousa_caps_tex_rgb_i4"
        };
        let model = skin.part(disc, "kai_sousa_lmoji_model", white, Some(caps), assets);
        place(commands, root, &model, at(1.1), false);
    }
    let l_face = if held.l {
        "kai_sousa_lbutton2_tex_rgb_ia8"
    } else {
        "kai_sousa_lbutton_tex_rgb_ia8"
    };
    let model = skin.part(
        disc,
        "kai_sousa_lbuttonT_model",
        white,
        Some(l_face),
        assets,
    );
    place(commands, root, &model, at(1.2), false);

    // R: space.
    let model = skin.part(
        disc,
        "kai_sousa_rbuttonT_model",
        white,
        Some("kai_sousa_rbutton_tex_rgb_ia8"),
        assets,
    );
    place(commands, root, &model, at(1.3), false);
    let model = skin.part(disc, "kai_sousa_spaceT_model", white, None, assets);
    place(commands, root, &model, at(1.4), false);

    // A, B, X, Y and Start.
    let a_face = if held.a {
        "kai_sousa_button1b_tex_rgb_ia8"
    } else {
        "kai_sousa_button1a_tex_rgb_ia8"
    };
    let model = skin.part(
        disc,
        "kai_sousa_abuttonT_model",
        white,
        Some(a_face),
        assets,
    );
    place(commands, root, &model, at(2.0), false);
    let b_face = if held.b {
        "kai_sousa_button2b_tex_rgb_ia8"
    } else {
        "kai_sousa_button2a_tex_rgb_ia8"
    };
    for (n, list) in ["kai_sousa_bbuttonT_model", "kai_sousa_cancelT_model"]
        .into_iter()
        .enumerate()
    {
        let model = skin.part(disc, list, white, Some(b_face), assets);
        place(commands, root, &model, at(2.1 + n as f32 * 0.1), false);
    }
    for (n, list) in ["kai_sousa_henkan_model", "kai_sousa_xbuttonT_model"]
        .into_iter()
        .enumerate()
    {
        // These lists draw with the primitive colour the L button left behind.
        let model = skin.part(
            disc,
            list,
            l_prim,
            Some("kai_sousa_xbutton_tex_rgb_ia8"),
            assets,
        );
        place(commands, root, &model, at(2.3 + n as f32 * 0.1), false);
    }
    let y_face = if held.y {
        "kai_sousa_ybutton2_tex_rgb_ia8"
    } else {
        "kai_sousa_ybutton_tex_rgb_ia8"
    };
    let model = skin.part(disc, "kai_sousa_kirikae_model", white, Some(y_face), assets);
    place(commands, root, &model, at(2.5), false);
    for (n, (list, mode)) in [
        ("kai_sousa_letter_model", kb::InputMode::Letter),
        ("kai_sousa_sign_model", kb::InputMode::Sign),
        ("kai_sousa_mark_model", kb::InputMode::Mark),
    ]
    .into_iter()
    .enumerate()
    {
        let prim = if keyboard.mode == mode {
            [225, 255, 255, 255]
        } else {
            [155, 155, 160, 255]
        };
        let model = skin.part(disc, list, prim, Some(y_face), assets);
        place(commands, root, &model, at(2.6 + n as f32 * 0.1), false);
    }
    let model = skin.part(
        disc,
        "kai_sousa_ybuttonT_model",
        white,
        Some(y_face),
        assets,
    );
    place(commands, root, &model, at(2.9), false);
    for (n, list) in ["kai_sousa_startbuttonT_model", "kai_sousa_endT_model"]
        .into_iter()
        .enumerate()
    {
        let model = skin.part(disc, list, white, None, assets);
        place(commands, root, &model, at(3.0 + n as f32 * 0.1), false);
    }

    // The D-pad and the stick.
    for (n, list) in ["kai_sousa_controllpadT_model", "kai_sousa_cursorT_model"]
        .into_iter()
        .enumerate()
    {
        let model = skin.part(disc, list, white, Some(pad_texture(held.pad)), assets);
        place(commands, root, &model, at(3.2 + n as f32 * 0.1), false);
    }
    let flipped = held.stick == Some(kb::Direction::Right);
    let stick_at = Vec3::new(
        if flipped { -111.0 } else { -110.0 } * PX,
        -50.0 * PX,
        Z + 3.4,
    );
    // The base is listed before the matrix with the stick's offset loads.
    let model = skin.part(disc, "kai_sousa_3DT_model", white, None, assets);
    place(commands, root, &model, at(3.4), false);
    let model = skin.part(
        disc,
        "kai_sousa_3DstT_model",
        white,
        Some(stick_texture(held.stick)),
        assets,
    );
    place(commands, root, &model, stick_at + Vec3::Z * 0.1, flipped);

    spawn_keys(commands, root, skin, disc, font, keyboard, assets);
}

/// The keys: their faces from `kai_sousa2_v`, then each key's letter.
fn spawn_keys(
    commands: &mut Commands,
    root: Entity,
    skin: &mut Skin,
    disc: &GameDisc,
    font: &MsgAssets,
    keyboard: &kb::Keyboard,
    assets: &mut Stores,
) {
    let Some(image) = skin.key_image(disc, assets) else {
        return;
    };
    let vertices = disc
        .catalog
        .get("kai_sousa2_v")
        .and_then(|s| {
            disc.rom.slice(
                RomSource::Rel,
                s.offset,
                kb::COLUMNS as u32 * kb::ROWS as u32 * 64,
            )
        })
        .map(<[u8]>::to_vec)
        .unwrap_or_default();
    let upper_letters =
        keyboard.mode == kb::InputMode::Letter && keyboard.shift == kb::Shift::Upper;
    let selected_tint = if upper_letters {
        Color::srgb_u8(0, 0, 205)
    } else {
        Color::srgb_u8(205, 0, 0)
    };
    // The space key is the disc's own mark (`lat_sousa_spT_model`).
    let space = |skin: &mut Skin, assets: &mut Stores, ink: [u8; 4]| {
        skin.model(
            disc,
            format!("space|{ink:?}"),
            &["lat_sousa_spT_model"],
            ink,
            None,
            assets,
        )
    };
    let space_idle = space(skin, assets, [35, 30, 55, 255]);
    let space_selected = space(skin, assets, [255, 255, 255, 255]);
    let mut paint = Paint {
        commands,
        meshes: &mut *assets.meshes,
        materials: &mut *assets.materials,
        root,
    };
    let be16 = |bytes: &[u8], at: usize| i16::from_be_bytes([bytes[at], bytes[at + 1]]);
    for row in 0..kb::ROWS {
        for column in 0..kb::COLUMNS {
            let key = column + row * kb::COLUMNS;
            let base = key * 4 * 16;
            if vertices.len() < base + 64 {
                continue;
            }
            let mut positions = Vec::new();
            let mut uvs = Vec::new();
            for corner in 0..4 {
                let at = base + corner * 16;
                positions.push([
                    f32::from(be16(&vertices, at)) * PX,
                    f32::from(be16(&vertices, at + 2)) * PX,
                    0.0,
                ]);
                // 10.5 fixed point over a 16 texel face that mirrors across.
                uvs.push([
                    f32::from(be16(&vertices, at + 8)) / 512.0,
                    f32::from(be16(&vertices, at + 10)) / 512.0,
                ]);
            }
            let triangles = if KEY_DIAGONAL[key] != 0 {
                vec![0, 1, 2, 3, 0, 2]
            } else {
                vec![0, 1, 2, 1, 3, 2]
            };
            let mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_indices(Indices::U32(triangles));
            let selected = (column, row) == (keyboard.column, keyboard.row);
            let tint = if selected {
                selected_tint
            } else {
                Color::WHITE
            };
            paint.put(mesh, image.clone(), tint, Z + 4.0);
        }
    }
    // The letters.
    for (row, (&row_y, &slide)) in ROW_Y.iter().zip(&ROW_SLIDE).enumerate() {
        for (column, &column_x) in COLUMN_X.iter().enumerate() {
            let code = keyboard.code(column, row);
            if code == kb::CHAR_SPACE_WIDE {
                continue;
            }
            let selected = (column, row) == (keyboard.column, keyboard.row);
            let ink = if selected {
                [255, 255, 255, 255]
            } else {
                [35, 30, 55, 255]
            };
            let origin = Vec2::new(column_x + slide + 2.0, row_y);
            if code == kb::CHAR_SPACE {
                let model = if selected {
                    &space_selected
                } else {
                    &space_idle
                };
                space_mark(paint.commands, root, model, origin);
            } else {
                letter(&mut paint, font, code, origin, ink, Z + 5.0);
            }
        }
    }
    // The chosen key again on the letter board (`mED_StringsDraw_select`).
    if let Some(code) = keyboard.selected() {
        let origin = Vec2::new(242.0, 180.0);
        if code == kb::CHAR_SPACE {
            space_mark(paint.commands, root, &space_selected, origin);
        } else {
            letter(
                &mut paint,
                font,
                code,
                origin,
                [255, 255, 255, 255],
                Z + 5.0,
            );
        }
    }
}

/// `mED_StringsDraw_spaceCode`: the mark sits at the key's text origin.
fn space_mark(commands: &mut Commands, root: Entity, model: &Model, origin: Vec2) {
    let x = 6.5 + (origin.x - 160.0);
    let y = -(8.5 + (origin.y - 120.0));
    for (mesh, material) in model {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(x * PX, y * PX, Z + 5.0).with_scale(Vec3::new(0.75, 1.0, 1.0)),
            RenderLayers::layer(1),
            ChildOf(root),
        ));
    }
}

/// One key's character.
fn letter(paint: &mut Paint, font: &MsgAssets, code: u8, origin: Vec2, ink: [u8; 4], z: f32) {
    if let Some(ch) = crate::messages::char_of(code) {
        paint.text(font, &ch.to_string(), origin, ink, z);
    }
}

/// The one-line key help shown above the window.
pub(super) fn hint(paint: &mut Paint, font: &MsgAssets, text: &str) {
    paint.text(
        font,
        &small(text),
        Vec2::new(24.0, 6.0),
        [255, 255, 255, 220],
        Z + 6.0,
    );
}
