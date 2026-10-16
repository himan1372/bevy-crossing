//! Retail inventory frame/icons: m_inventory_ovl, m_submenu_ovl and m_tag_ovl.
//! All graphics and icon pointer tables are read from the user's disc.
use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use disc::{
    gx_texture::{self, GxFormat, TlutFormat},
    RomSource, Target,
};

use crate::disc_gfx::{run_display_lists_with_colors, Segments};
use crate::game_disc::GameDisc;
use crate::interaction::PlayerInventory;
use crate::msg_window::{quad_mesh, screen, text_mesh, text_width, MsgAssets};
use crate::n64_material::N64Material;
use crate::pockets::PocketsUi;
use crate::skeleton::{Animation, Skeleton};

type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;

struct UiRig {
    skeleton: Skeleton,
    animation: Animation,
    vertices: Vec<Vec<rustimal_logic::graphics::vertex::DecodedVtx>>,
    models: Model,
    frame: f32,
    speed: f32,
    catch: Option<Animation>,
    grabbing: bool,
}

struct PatternTile {
    image: Handle<Image>,
    mask: Vec<u8>,
    width: usize,
}

#[derive(Component)]
struct PocketHand;

#[derive(Resource)]
pub(crate) struct PocketAssets {
    frame: Model,
    preview: Model,
    hand: Model,
    rigs: Vec<UiRig>,
    patterns: Vec<PatternTile>,
    cloth: Vec<u8>,
    scroll: f32,
    scroll_step: i32,
    slide: f32,
    slide_speed: f32,
    slide_ticks: f32,
    slide_flag: bool,
    was_open: bool,
    hand_at: Vec2,
    icons: HashMap<(u16, bool), Handle<Image>>,
    root: Option<Entity>,
    dynamic: Option<Entity>,
    white: Handle<Image>,
}

fn image(rgba: Vec<u8>, w: u32, h: u32) -> Image {
    Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn pointer(disc: &GameDisc, at: u32) -> Option<u32> {
    match disc.rom.pointer_at(RomSource::Rel, at)? {
        Target::File(RomSource::Rel, p) => Some(p),
        _ => None,
    }
}

/// mSM_draw_item: 16 category tables, with stationery stack and singleton
/// exceptions. Table sizes come from the catalog, not an unchecked item index.
fn icon(disc: &GameDisc, item: u16, present: bool) -> Option<Vec<u8>> {
    let named = |name: &str| disc.catalog.get(name).map(|a| a.offset);
    let table = if present {
        named("present_tex_data$762")?
    } else if crate::items::is_furniture(item) {
        // m_name_table.h FTR_START/END: four rotations per enum entry.
        if (0x15B0..=0x17AB).contains(&item) {
            named("haniwa_tex_data$759")?
        } else if (0x1EEC..=0x1F4F).contains(&item) {
            named("fossil_tex_data$760")?
        } else {
            named("furniture_tex_data$758")?
        }
    } else {
        let category = ((item >> 8) & 15) as u32;
        let index = match category {
            0 => (item & 255) as u32 / 64,
            4 | 6 | 7 | 10 | 11 | 14 => 0,
            _ => (item & 255) as u32,
        };
        let tables = named("item_tex_data_table$779")?;
        let table = pointer(disc, tables + category * 4)?;
        // Catalog metadata bounds the selected category table.
        let entry = disc.catalog.containing(RomSource::Rel, table)?;
        let size = entry.size.checked_sub(table - entry.offset)?;
        if index.checked_mul(8)?.checked_add(8)? > size {
            return None;
        }
        table + index * 8
    };
    let pal = pointer(disc, table)?;
    let tex = pointer(disc, table + 4)?;
    gx_texture::decode(
        disc.rom.slice(RomSource::Rel, tex, 512)?,
        GxFormat::C4,
        32,
        32,
        Some((disc.rom.slice(RomSource::Rel, pal, 32)?, TlutFormat::Rgb5A3)),
    )
}

fn build(
    disc: &GameDisc,
    entries: &[&str],
    env: [u8; 4],
    scale: f32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
    patterns: &mut Vec<PatternTile>,
) -> Model {
    let lists: Vec<_> = entries
        .iter()
        .filter_map(|n| disc.catalog.get(n).map(|a| a.offset))
        .collect();
    let inventory_frame = entries.contains(&"inv_mwin_model");
    let sink = if inventory_frame {
        crate::disc_gfx::run_inventory_frame(&disc.rom, &lists)
    } else {
        run_display_lists_with_colors(&disc.rom, &lists, [255; 4], 255, env, &[])
    };
    let model = crate::disc_town::sink_meshes_scaled(
        disc,
        &sink,
        None,
        true,
        &Segments::default(),
        scale,
        (meshes, materials, images),
    );
    // UI I4 masks carry intensity as coverage, as in msg_window's font.
    // The generic GX decoder returns opaque intensity texels.
    for (part, (_, material)) in sink.parts.iter().zip(&model) {
        let Some(key) = part.key.tex.as_ref() else {
            continue;
        };
        if key.format != rustimal_logic::graphics::texture::GxTexFormat::I4 {
            continue;
        }
        if let Some(mut rgba) =
            crate::disc_gfx::decode_texture(&disc.rom, key, None, &Segments::default())
        {
            for pixel in rgba.as_chunks_mut::<4>().0 {
                pixel[3] = pixel[0];
            }
            if let Some(mut mat) = materials.get_mut(material) {
                mat.texture = Some(images.add(crate::n64_material::texture_image(rgba, key)));
            }
        }
    }
    if inventory_frame {
        // mPr_Clear_PrivateInfo defaults background_texture to ITM_CLOTH226.
        // mSM_Object_Exchange_keep_new_MenuTexAndPallet binds this 32x32 tile.
        let cloth = disc
            .player_cloth
            .get(226 * 512..227 * 512)
            .zip(disc.player_cloth_palettes.get(226 * 32..227 * 32))
            .and_then(|(tex, pal)| {
                gx_texture::decode(tex, GxFormat::C4, 32, 32, Some((pal, TlutFormat::Rgb5A3)))
            });
        if let Some(cloth) = cloth {
            for (part, (_, material)) in sink.parts.iter().zip(&model) {
                if !part.key.two_cycle {
                    continue;
                }
                let Some(key) = part.key.tex.as_ref() else {
                    continue;
                };
                let Some(mask) =
                    crate::disc_gfx::decode_texture(&disc.rom, key, None, &Segments::default())
                else {
                    continue;
                };
                let rgba = mask
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .enumerate()
                    .flat_map(|(i, m)| {
                        let x = i % key.width as usize;
                        let y = i / key.width as usize;
                        let c = ((y % 32) * 32 + x % 32) * 4;
                        [cloth[c], cloth[c + 1], cloth[c + 2], m[3]]
                    })
                    .collect();
                if let Some(mut mat) = materials.get_mut(material) {
                    let mut texture = crate::n64_material::texture_image(rgba, key);
                    texture.asset_usage = RenderAssetUsages::default();
                    let handle = images.add(texture);
                    mat.texture = Some(handle.clone());
                    patterns.push(PatternTile {
                        image: handle,
                        mask,
                        width: key.width as usize,
                    });
                }
            }
        }
    }
    model
}

fn setup(
    mut commands: Commands,
    disc: Res<GameDisc>,
    look: Res<crate::character::PlayerLook>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut patterns = Vec::new();
    let mut rigs = Vec::new();
    let mut frame = build(
        &disc,
        &["inv_mwin_mode", "inv_mwin_model"],
        [0; 4],
        16.0,
        &mut meshes,
        &mut materials,
        &mut images,
        &mut patterns,
    );
    for (kind, count, env) in [
        ("b", 15, [100, 100, 255, 255]),
        ("a", 10, [255, 90, 90, 255]),
    ] {
        for n in 1..=count {
            frame.extend(build(
                &disc,
                &[
                    "inv_mwin_item_frame_mode",
                    &format!("inv_mwin_{n}{kind}T_model"),
                ],
                env,
                16.0,
                &mut meshes,
                &mut materials,
                &mut images,
                &mut patterns,
            ));
        }
    }
    frame.extend(build(
        &disc,
        &[
            "inv_mwin_mode",
            "inv_mwin_kuni_model",
            "inv_mwin_kuni2_model",
        ],
        [0; 4],
        16.0,
        &mut meshes,
        &mut materials,
        &mut images,
        &mut patterns,
    ));
    let mut preview = Vec::new();
    if let Some(skeleton) = disc
        .catalog
        .get("cKF_bs_r_boy_1")
        .and_then(|a| Skeleton::read(&disc.rom, a.offset))
    {
        let anim = disc
            .catalog
            .get("cKF_ba_r_ply_1_walk1")
            .and_then(|a| Animation::read(&disc.rom, a.offset, skeleton.joints.len()));
        let mut sink = crate::disc_gfx::run_display_lists(&disc.rom, &skeleton.lists());
        let vertices = sink.parts.iter().map(|p| p.vertices.clone()).collect();
        crate::skeleton::place_on_joints(
            &mut sink,
            &skeleton,
            &skeleton.world_matrices(anim.as_ref(), 1.0),
        );
        preview = crate::disc_town::sink_meshes_scaled(
            &disc,
            &sink,
            None,
            false,
            &crate::character::player_segments(&disc, *look),
            1.0,
            (&mut meshes, &mut materials, &mut images),
        );
        if let Some(animation) = anim {
            for (handle, _) in &preview {
                if let Some(mut mesh) = meshes.get_mut(handle) {
                    mesh.asset_usage = RenderAssetUsages::default();
                }
            }
            rigs.push(UiRig {
                skeleton,
                animation,
                vertices,
                models: preview.clone(),
                frame: 1.0,
                speed: 0.5,
                catch: None,
                grabbing: false,
            });
        }
    }
    info!(
        "retail pockets: {} frame parts, {} player preview parts",
        frame.len(),
        preview.len()
    );
    let mut hand = Vec::new();
    if let Some(skeleton) = disc
        .catalog
        .get("cKF_bs_r_hnd")
        .and_then(|a| Skeleton::read(&disc.rom, a.offset))
    {
        let anim = disc
            .catalog
            .get("cKF_ba_r_hnd_sasu")
            .and_then(|a| Animation::read(&disc.rom, a.offset, skeleton.joints.len()));
        let mut sink = crate::disc_gfx::run_display_lists(&disc.rom, &skeleton.lists());
        let vertices = sink.parts.iter().map(|p| p.vertices.clone()).collect();
        crate::skeleton::place_on_joints(
            &mut sink,
            &skeleton,
            &skeleton.world_matrices(anim.as_ref(), 1.0),
        );
        hand = crate::disc_town::sink_meshes_scaled(
            &disc,
            &sink,
            None,
            false,
            &Segments::default(),
            1.0,
            (&mut meshes, &mut materials, &mut images),
        );
        if let Some(animation) = anim {
            for (handle, _) in &hand {
                if let Some(mut mesh) = meshes.get_mut(handle) {
                    mesh.asset_usage = RenderAssetUsages::default();
                }
            }
            let catch = disc
                .catalog
                .get("cKF_ba_r_hnd_catch")
                .and_then(|a| Animation::read(&disc.rom, a.offset, skeleton.joints.len()));
            rigs.push(UiRig {
                skeleton,
                animation,
                vertices,
                models: hand.clone(),
                frame: 1.0,
                speed: 0.275,
                catch,
                grabbing: false,
            });
        }
    }
    commands.insert_resource(PocketAssets {
        frame,
        preview,
        hand,
        rigs,
        patterns,
        cloth: disc
            .player_cloth
            .get(226 * 512..227 * 512)
            .zip(disc.player_cloth_palettes.get(226 * 32..227 * 32))
            .and_then(|(tex, pal)| {
                gx_texture::decode(tex, GxFormat::C4, 32, 32, Some((pal, TlutFormat::Rgb5A3)))
            })
            .unwrap_or_default(),
        scroll: 0.0,
        scroll_step: -1,
        slide: 300.0,
        slide_speed: 75.0,
        slide_ticks: 0.0,
        slide_flag: false,
        was_open: false,
        hand_at: slot_at(0) + Vec2::new(6.0, -6.0),
        icons: HashMap::new(),
        root: None,
        dynamic: None,
        white: images.add(image(vec![255; 4], 1, 1)),
    });
}

/// mTG_item_col_pos / line_pos, converted to 320x240 screen space.
fn slot_at(slot: usize) -> Vec2 {
    Vec2::new(
        160.0 + (-77.0 + 24.0 * (slot % 5) as f32) * 0.975,
        130.0 + 24.0 * (slot / 5) as f32,
    )
}

fn animate(
    time: Res<Time>,
    ui: Res<PocketsUi>,
    mut assets: ResMut<PocketAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut transforms: Query<&mut Transform>,
    hands: Query<Entity, With<PocketHand>>,
) {
    let ticks = time.delta_secs() * 60.0;
    if ticks <= 0.0 || assets.root.is_none() {
        return;
    }
    if assets.was_open != ui.open {
        assets.was_open = ui.open;
        assets.slide_speed = if ui.open { 75.0 } else { 1.0 };
        assets.slide_ticks = 0.0;
        assets.slide_flag = false;
    }
    // mSM_move_menu: 60Hz accumulator, alternate-tick speed changes.
    assets.slide_ticks += ticks;
    while assets.slide_ticks >= 1.0 {
        if !assets.slide_flag && ((ui.open && assets.slide <= 120.0) || !ui.open) {
            assets.slide_speed =
                (assets.slide_speed * if ui.open { 0.5 } else { 2.0 }).clamp(1.0, 75.0);
        }
        assets.slide_flag = !assets.slide_flag;
        assets.slide = (assets.slide + assets.slide_speed * if ui.open { -0.5 } else { 0.5 })
            .clamp(0.0, 300.0);
        assets.slide_ticks -= 1.0;
    }
    if let Some(root) = assets.root {
        if let Ok(mut t) = transforms.get_mut(root) {
            t.translation.x = assets.slide * 16.0;
        }
    }
    for rig in &mut assets.rigs {
        let grabbing = rig.catch.is_some() && ui.grabbed.is_some();
        if rig.grabbing != grabbing {
            rig.frame = 1.0;
            rig.grabbing = grabbing;
        }
        let anim = if grabbing {
            rig.catch.as_ref().unwrap_or(&rig.animation)
        } else {
            &rig.animation
        };
        let end = anim.frames as f32;
        rig.frame = if grabbing {
            (rig.frame + ticks * 0.45).min(end)
        } else {
            1.0 + (rig.frame - 1.0 + ticks * rig.speed).rem_euclid((end - 1.0).max(1.0))
        };
        let world = rig.skeleton.world_matrices(Some(anim), rig.frame);
        let slots = rig.skeleton.matrix_joints();
        for (base, (handle, _)) in rig.vertices.iter().zip(&rig.models) {
            let Some(mut mesh) = meshes.get_mut(handle) else {
                continue;
            };
            let matrix = |v: &rustimal_logic::graphics::vertex::DecodedVtx| {
                let slot = match v.mtx >> 24 {
                    0xA0 => v.mtx & 0xFFFFFF,
                    0x0D => (v.mtx & 0xFFFFFF) / 0x40,
                    _ => return Mat4::IDENTITY,
                };
                slots
                    .get(slot as usize)
                    .and_then(|&j| world.get(j))
                    .copied()
                    .unwrap_or(Mat4::IDENTITY)
            };
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_POSITION,
                base.iter()
                    .map(|v| matrix(v).transform_point3(Vec3::from(v.pos)).to_array())
                    .collect::<Vec<_>>(),
            );
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                base.iter()
                    .map(|v| {
                        matrix(v)
                            .transform_vector3(Vec3::from(v.normal))
                            .normalize_or_zero()
                            .to_array()
                    })
                    .collect::<Vec<_>>(),
            );
        }
    }
    let target = slot_at(ui.selected) + Vec2::new(6.0, -6.0);
    let hand_at = assets.hand_at.lerp(target, 1.0 - 0.75_f32.powf(ticks));
    assets.hand_at = hand_at;
    for hand in &hands {
        if let Ok(mut t) = transforms.get_mut(hand) {
            t.translation = screen(hand_at).with_z(180.0);
        }
    }
    // mSM_menu_control_move supplies the 0.3535/tick rate and integer
    // quarter-texel offsets. Philip requests horizontal leftward scrolling;
    // sample progressively rightward in the texture, without a Y offset.
    assets.scroll = (assets.scroll + 0.3535 * ticks).rem_euclid(1024.0);
    let step = assets.scroll as i32;
    if step == assets.scroll_step || assets.cloth.is_empty() {
        return;
    }
    assets.scroll_step = step;
    let phase = (step & 127) as f32 * 0.25;
    for pattern in &assets.patterns {
        let Some(mut img) = images.get_mut(&pattern.image) else {
            continue;
        };
        let Some(data) = img.data.as_mut() else {
            continue;
        };
        for (i, p) in data.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let x = i % pattern.width;
            let y = i / pattern.width;
            let sx = x as f32 + phase;
            let sy = y as f32;
            let ix = sx.floor() as usize;
            let iy = sy.floor() as usize;
            let fx = sx.fract();
            let fy = sy.fract();
            for (c, out) in p.iter_mut().take(3).enumerate() {
                let read =
                    |x: usize, y: usize| assets.cloth[((y % 32) * 32 + x % 32) * 4 + c] as f32;
                let a = read(ix, iy) * (1.0 - fx) + read(ix + 1, iy) * fx;
                let b = read(ix, iy + 1) * (1.0 - fx) + read(ix + 1, iy + 1) * fx;
                *out = (a * (1.0 - fy) + b * fy) as u8;
            }
            p[3] = pattern.mask[i * 4 + 3];
        }
    }
}

struct Paint<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    meshes: &'a mut Assets<Mesh>,
    materials: &'a mut Assets<StandardMaterial>,
    root: Entity,
}
impl Paint<'_, '_, '_> {
    fn put(&mut self, mesh: Mesh, tex: Handle<Image>, tint: Color, z: f32) {
        self.commands.spawn((
            Mesh3d(self.meshes.add(mesh)),
            MeshMaterial3d(self.materials.add(StandardMaterial {
                base_color: tint,
                base_color_texture: Some(tex),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, z),
            RenderLayers::layer(1),
            ChildOf(self.root),
        ));
    }
    fn text(&mut self, font: &MsgAssets, text: &str, at: Vec2, color: [u8; 4], z: f32) {
        self.put(
            text_mesh(text, at, &font.widths, color, usize::MAX),
            font.font.clone(),
            Color::WHITE,
            z,
        );
    }
}

fn draw(
    mut commands: Commands,
    disc: Res<GameDisc>,
    font: Res<MsgAssets>,
    ui: Res<PocketsUi>,
    inventory: Res<PlayerInventory>,
    focus: Res<crate::input::InputFocus>,
    mut assets: ResMut<PocketAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let visible = ui.open && *focus == crate::input::InputFocus::Pockets;
    if !visible {
        if !ui.open && assets.root.is_some() && assets.slide < 300.0 {
            return;
        }
        if let Some(root) = assets.root.take() {
            commands.entity(root).despawn();
        }
        assets.dynamic = None;
        return;
    }
    let fresh = assets.root.is_none();
    if fresh {
        assets.slide = 300.0;
        assets.was_open = false;
        let root = commands
            .spawn((
                Name::new("retail pockets"),
                Transform::from_xyz(300.0 * 16.0, 0.0, 0.0),
                Visibility::default(),
            ))
            .id();
        for (index, (mesh, material)) in assets.frame.iter().enumerate() {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                // Retail issues these blended layers in list order. Bevy
                // needs distinct depths; entity IDs change after reopening.
                Transform::from_xyz(0.0, 0.0, index as f32 * 0.1),
                RenderLayers::layer(1),
                ChildOf(root),
            ));
        }
        // The current player appearance in the upper-left preview well.
        // Orthographic adaptation of mIV_set_player's separate perspective view.
        for (mesh, material) in &assets.preview {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(screen(Vec2::new(83.0, 98.0)).with_z(80.0))
                    .with_scale(Vec3::splat(0.16)),
                RenderLayers::layer(1),
                ChildOf(root),
            ));
        }
        assets.root = Some(root);
    }
    if !fresh && !ui.is_changed() && !inventory.is_changed() {
        return;
    }
    if let Some(old) = assets.dynamic.take() {
        commands.entity(old).despawn();
    }
    let Some(root) = assets.root else {
        return;
    };
    let dynamic = commands
        .spawn((Transform::default(), Visibility::default(), ChildOf(root)))
        .id();
    assets.dynamic = Some(dynamic);
    let mut paint = Paint {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        root: dynamic,
    };
    for (slot, &item) in inventory.pockets.pockets.iter().enumerate() {
        if item == 0 || ui.grabbed == Some(slot) {
            continue;
        }
        let present = inventory.pockets.item_cond(slot) & 1 != 0;
        let key = (item, present);
        if let std::collections::hash_map::Entry::Vacant(entry) = assets.icons.entry(key) {
            if let Some(rgba) = icon(&disc, item, present) {
                entry.insert(images.add(image(rgba, 32, 32)));
            }
        }
        let at = slot_at(slot);
        if let Some(tex) = assets.icons.get(&key) {
            paint.put(
                quad_mesh(at - Vec2::splat(12.0), Vec2::splat(24.0)),
                tex.clone(),
                Color::WHITE,
                120.0,
            );
        } else {
            // A missing/unsupported icon must remain a selectable, named item.
            paint.text(
                &font,
                "?",
                at - Vec2::new(4.0, 8.0),
                [50, 60, 50, 255],
                120.0,
            );
        }
    }
    let selected = inventory.pockets.pockets[ui.selected];
    let at = slot_at(ui.selected);
    if let Some(from) = ui.grabbed {
        let item = inventory.pockets.pockets[from];
        let present = inventory.pockets.item_cond(from) & 1 != 0;
        if let Some(tex) = assets.icons.get(&(item, present)) {
            paint.put(
                quad_mesh(at - Vec2::splat(12.0), Vec2::splat(24.0)),
                tex.clone(),
                Color::WHITE,
                175.0,
            );
        }
    }
    if assets.hand.is_empty() {
        paint.put(
            quad_mesh(at - Vec2::new(22.0, 7.0), Vec2::splat(14.0)),
            font.cursor.clone(),
            Color::srgb_u8(0, 195, 185),
            130.0,
        );
    } else {
        // Original hand model and rotation (mHD_ROT_X/Z); orthographic
        // placement in the shared overlay replaces its separate 64px view.
        for (mesh, material) in &assets.hand {
            paint.commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                PocketHand,
                Transform::from_translation(screen(assets.hand_at).with_z(180.0))
                    .with_rotation(Quat::from_euler(
                        EulerRot::ZYX,
                        70.3125_f32.to_radians(),
                        0.0,
                        -135.0_f32.to_radians(),
                    ))
                    .with_scale(Vec3::splat(0.08)),
                RenderLayers::layer(1),
                ChildOf(dynamic),
            ));
        }
    }
    let labelled = ui
        .grabbed
        .map_or(selected, |from| inventory.pockets.pockets[from]);
    let label = if labelled == 0 {
        String::new()
    } else {
        crate::items::name(labelled)
    };
    let width = text_width(&label, &font.widths);
    if !label.is_empty() {
        paint.put(
            quad_mesh(
                Vec2::new(160.0 - width * 0.5 - 5.0, 195.0),
                Vec2::new(width + 10.0, 19.0),
            ),
            assets.white.clone(),
            Color::srgb_u8(255, 255, 210),
            140.0,
        );
        paint.text(
            &font,
            &label,
            Vec2::new(160.0 - width * 0.5, 196.0),
            [50, 60, 50, 255],
            141.0,
        );
    }
    let money = format!("{}", inventory.bells);
    paint.text(
        &font,
        &money,
        Vec2::new(176.5 - text_width(&money, &font.widths), 102.0),
        [255, 60, 0, 255],
        130.0,
    );
    if let Some(choice) = ui.menu {
        let options =
            crate::pockets::item_actions(selected, inventory.pockets.item_cond(ui.selected));
        let centre = Vec2::new(224.0, 166.0);
        for (mesh, material) in &font.choice_body {
            paint.commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(screen(centre).with_z(160.0))
                    .with_scale(Vec3::new(0.7, 0.7, 1.0)),
                RenderLayers::layer(1),
                ChildOf(dynamic),
            ));
        }
        for (i, label) in options.iter().enumerate() {
            let at = Vec2::new(205.0, 150.0 + i as f32 * 16.0);
            paint.text(
                &font,
                label,
                at,
                if i == choice {
                    [120, 50, 50, 255]
                } else {
                    [180, 150, 110, 255]
                },
                162.0,
            );
            if i == choice {
                paint.put(
                    quad_mesh(at - Vec2::new(16.0, 0.0), Vec2::splat(16.0)),
                    font.cursor.clone(),
                    Color::srgb_u8(0, 195, 185),
                    163.0,
                );
            }
        }
    }
}

pub struct PocketDiscPlugin;
impl Plugin for PocketDiscPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            setup
                .run_if(resource_exists::<MsgAssets>)
                .run_if(not(resource_exists::<PocketAssets>)),
        )
        .add_systems(
            Update,
            draw.after(crate::input::InteractSet::Ui)
                .run_if(resource_exists::<PocketAssets>),
        );
        app.add_systems(
            Update,
            animate.after(draw).run_if(resource_exists::<PocketAssets>),
        );
    }
}
