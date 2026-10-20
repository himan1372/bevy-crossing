//! Retail inventory frame/icons: m_inventory_ovl, m_submenu_ovl and m_tag_ovl.
//! All graphics and icon pointer tables are read from the user's disc.
use std::collections::HashMap;

use bevy::asset::{embedded_asset, RenderAssetUsages};
use bevy::camera::visibility::RenderLayers;
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat};
use bevy::shader::ShaderRef;
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
    releasing: bool,
}

struct PatternTile {
    source: Handle<N64Material>,
    material: Handle<PocketPattern>,
}

/// One continuous screen-space cloth tile, independently masked per part.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct PocketPattern {
    #[uniform(0)]
    offset: Vec4,
    #[texture(1)]
    #[sampler(2)]
    cloth: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    mask: Handle<Image>,
}
impl PocketPattern {
    pub(crate) fn scroll(&mut self, slide: f32, phase: f32) {
        self.offset = Vec4::new(slide * 16.0, phase, 0.0, 0.0);
    }
    pub(crate) fn new(cloth: Handle<Image>, mask: Handle<Image>) -> Self {
        Self {
            offset: Vec4::ZERO,
            cloth,
            mask,
        }
    }
}
impl Material for PocketPattern {
    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_crossing/pockets_pattern.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

#[derive(Component)]
struct PocketHand;
#[derive(Component)]
struct PocketCarry(Vec2, f32);

#[derive(Resource)]
pub(crate) struct PocketAssets {
    frame: Model,
    hand: Model,
    portrait: Handle<Image>,
    hand_image: Handle<Image>,
    envelopes: [Option<Handle<Image>>; 2],
    rigs: Vec<UiRig>,
    patterns: Vec<PatternTile>,
    scroll: f32,
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
impl PocketAssets {
    pub(crate) fn hand_image(&self) -> Handle<Image> {
        self.hand_image.clone()
    }
    /// aMR_SetReserveFurniture waits for the submenu to finish closing.
    pub(crate) fn world_ready(&self) -> bool {
        !self.was_open && self.slide >= 300.0
    }
    pub(crate) fn hand_busy(&self) -> bool {
        self.rigs.iter().any(|r| {
            r.releasing
                || (r.grabbing && r.catch.as_ref().is_some_and(|a| r.frame < a.frames as f32))
        })
    }
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
    pattern_materials: &mut Assets<PocketPattern>,
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
            let cloth_key = crate::disc_gfx::TexKey {
                img_addr: 0,
                format: rustimal_logic::graphics::texture::GxTexFormat::C4,
                width: 32,
                height: 32,
                wrap_s: 1,
                wrap_t: 1,
                tlut_addr: 0,
                n64: None,
                tlut_rgba5551: false,
            };
            let cloth = images.add(crate::n64_material::texture_image(cloth, &cloth_key));
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
                let mask = images.add(crate::n64_material::texture_image(mask, key));
                patterns.push(PatternTile {
                    source: material.clone(),
                    material: pattern_materials.add(PocketPattern {
                        offset: Vec4::ZERO,
                        cloth: cloth.clone(),
                        mask,
                    }),
                });
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
    mut pattern_materials: ResMut<Assets<PocketPattern>>,
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
        &mut pattern_materials,
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
                &mut pattern_materials,
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
        &mut pattern_materials,
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
                releasing: false,
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
                releasing: false,
            });
        }
    }
    // mSM_change_view's fixed ambient and directional light.
    for (_, handle) in preview.iter().chain(&hand) {
        if let Some(mut mat) = materials.get_mut(handle) {
            mat.combiner.flags |= crate::n64_material::FLAG_MENU_LIGHTING;
            mat.combiner.flags &= !crate::n64_material::FLAG_FOG;
            mat.env = crate::n64_material::EnvUniform {
                ambient: Vec4::new(105.0 / 255.0, 90.0 / 255.0, 90.0 / 255.0, 1.0),
                sun_dir: Vec3::new(0.0, 1.0, 1.0).normalize().extend(0.0),
                sun_color: Vec4::new(1.0, 1.0, 245.0 / 255.0, 1.0),
                moon_color: Vec4::ZERO,
                ..default()
            };
        }
    }
    let portrait = make_view(&mut commands, &mut images, &preview, 2, true);
    let hand_image = make_view(&mut commands, &mut images, &hand, 3, false);
    commands.insert_resource(PocketAssets {
        portrait,
        hand_image,
        envelopes: [false, true]
            .map(|read| envelope(&disc, read).map(|rgba| images.add(image(rgba, 32, 32)))),
        frame,
        hand,
        rigs,
        patterns,
        scroll: 0.0,
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

fn make_view(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    model: &Model,
    layer: usize,
    player: bool,
) -> Handle<Image> {
    let size = if player { 512 } else { 128 };
    let image = images.add(Image::new_target_texture(
        size,
        size,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    ));
    let transform = if player {
        Transform::from_scale(Vec3::splat(0.01))
    } else {
        Transform::from_xyz(0.0, 0.0, 77.0)
            .with_scale(Vec3::splat(0.0004))
            .with_rotation(Quat::from_euler(
                EulerRot::ZYX,
                70.3125_f32.to_radians(),
                0.0,
                -135.0_f32.to_radians(),
            ))
    };
    for (mesh, material) in model {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            transform,
            RenderLayers::layer(layer),
        ));
    }
    let (eye, look, fov, near, far) = if player {
        let angle = 12.65625_f32.to_radians();
        (
            Vec3::new(0.0, 25.0 + 330.0 * angle.sin(), 330.0 * angle.cos()),
            Vec3::new(0.0, 25.0, 0.0),
            20.0_f32,
            100.0,
            800.0,
        )
    } else {
        (Vec3::new(0.0, 0.0, 80.0), Vec3::ZERO, 35.0, 1.0, 2000.0)
    };
    commands.spawn((
        Name::new(if player {
            "pockets portrait camera"
        } else {
            "pockets hand camera"
        }),
        Camera3d::default(),
        Camera {
            order: -2,
            is_active: false,
            clear_color: Color::NONE.into(),
            ..default()
        },
        RenderTarget::Image(image.clone().into()),
        Projection::Perspective(PerspectiveProjection {
            fov: fov.to_radians(),
            aspect_ratio: 1.0,
            near,
            far,
            ..default()
        }),
        Transform::from_translation(eye).looking_at(look, Vec3::Y),
        RenderLayers::layer(layer),
        PocketView { hand: !player },
    ));
    image
}

#[derive(Component)]
struct PocketView {
    hand: bool,
}

/// mTG_item_col_pos / line_pos, matching the inv_mwin pocket-quad centers.
/// Both frame and icons share this orthographic view: applying mSM's separate
/// item-view X correction here would shift icons away from their circles.
fn slot_at(slot: usize) -> Vec2 {
    Vec2::new(
        83.0 + 24.0 * (slot % 5) as f32,
        130.0 + 24.0 * (slot / 5) as f32,
    )
}

fn mail_at(slot: usize) -> Vec2 {
    Vec2::new(
        215.0 + 24.0 * (slot % 2) as f32,
        82.0 + 24.0 * (slot / 2) as f32,
    )
}

fn envelope(disc: &GameDisc, read: bool) -> Option<Vec<u8>> {
    let suffix = if read { "mtegami2" } else { "mtegami" };
    let tex = disc.catalog.get(&format!("inv_mwin_{suffix}_tex"))?;
    let pal = disc.catalog.get(&format!("inv_mwin_{suffix}_pal"))?;
    gx_texture::decode(
        disc.rom.slice(tex.source, tex.offset, tex.size)?,
        GxFormat::C4,
        32,
        32,
        Some((
            disc.rom.slice(pal.source, pal.offset, pal.size)?,
            TlutFormat::Rgb5A3,
        )),
    )
}

fn selected_mail(ui: &PocketsUi, mail: &crate::mail::MailUi) -> Option<usize> {
    if mail.listing() {
        Some(mail.selected)
    } else {
        ui.mail_slot
    }
}

/// mHD_hand_pos_get + mHD_hand_ovl_draw, CLOSE_KEEP offsets (10,2).
/// The carried icon is displaced from the pocket center, exposing its rim.
fn held_item_offset() -> Vec2 {
    let (sin_z, cos_z) = 70.3125_f32.to_radians().sin_cos();
    let cos_x = (-135.0_f32).to_radians().cos();
    let target_x = -5.0 + 10.0 * cos_z - 2.0 * cos_x * sin_z + 4.0 - 2.5;
    let target_y = 12.0 + 10.0 * sin_z + 2.0 * cos_x * cos_z - 4.0 + 2.5;
    Vec2::new(
        target_x - 6.9 * cos_z - 7.0 * cos_x * sin_z,
        -(target_y - 6.9 * sin_z + 7.0 * cos_x * cos_z - 10.0),
    )
}

fn animate(
    time: Res<Time>,
    ui: Res<PocketsUi>,
    mail: Res<crate::mail::MailUi>,
    mut assets: ResMut<PocketAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pattern_materials: ResMut<Assets<PocketPattern>>,
    mut transforms: Query<&mut Transform>,
    hands: Query<Entity, With<PocketHand>>,
    carries: Query<(Entity, &PocketCarry)>,
    mut cameras: Query<(&mut Camera, &PocketView)>,
    room: Res<crate::player_room::PlayerRoom>,
) {
    for (mut camera, view) in &mut cameras {
        camera.is_active = assets.root.is_some() || (view.hand && room.music_menu_open());
    }
    let ticks = time.delta_secs() * 60.0;
    if ticks <= 0.0 || assets.root.is_none() {
        return;
    }
    let open = (ui.open && !mail.open) || mail.listing();
    if assets.was_open != open {
        assets.was_open = open;
        assets.slide_speed = if open { 75.0 } else { 1.0 };
        assets.slide_ticks = 0.0;
        assets.slide_flag = false;
    }
    // mSM_move_menu: 60Hz accumulator, alternate-tick speed changes.
    assets.slide_ticks += ticks;
    while assets.slide_ticks >= 1.0 {
        if !assets.slide_flag && ((open && assets.slide <= 120.0) || !open) {
            assets.slide_speed =
                (assets.slide_speed * if open { 0.5 } else { 2.0 }).clamp(1.0, 75.0);
        }
        assets.slide_flag = !assets.slide_flag;
        assets.slide =
            (assets.slide + assets.slide_speed * if open { -0.5 } else { 0.5 }).clamp(0.0, 300.0);
        assets.slide_ticks -= 1.0;
    }
    if let Some(root) = assets.root {
        if let Ok(mut t) = transforms.get_mut(root) {
            t.translation.x = assets.slide * 16.0;
        }
    }
    for rig in &mut assets.rigs {
        let grabbing = rig.catch.is_some() && (ui.grabbed.is_some() || mail.grabbed.is_some());
        if rig.grabbing != grabbing {
            rig.releasing = !grabbing;
            if grabbing {
                rig.frame = 1.0;
            }
            rig.grabbing = grabbing;
        }
        let anim = if grabbing || rig.releasing {
            rig.catch.as_ref().unwrap_or(&rig.animation)
        } else {
            &rig.animation
        };
        let end = anim.frames as f32;
        rig.frame = if rig.releasing {
            (rig.frame - ticks * 0.45).max(1.0)
        } else if grabbing {
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
        if rig.releasing && rig.frame <= 1.0 {
            rig.releasing = false;
        }
    }
    let target = selected_mail(&ui, &mail).map_or_else(|| slot_at(ui.selected), mail_at)
        + Vec2::new(6.0, -6.0);
    let delta = target - assets.hand_at;
    let distance = delta.length();
    // mHD_hand_position_move/add_calc: ease, max6/min0.5 pixels/tick.
    let step = (distance * (1.0 - 0.5_f32.sqrt().powf(ticks)))
        .clamp(0.5 * ticks, 6.0 * ticks)
        .min(distance);
    let hand_at = assets.hand_at + delta.normalize_or_zero() * step;
    assets.hand_at = hand_at;
    for hand in &hands {
        if let Ok(mut t) = transforms.get_mut(hand) {
            t.translation = Vec3::new(hand_at.x * 16.0, -hand_at.y * 16.0, 180.0);
        }
    }
    for (entity, carry) in &carries {
        if let Ok(mut t) = transforms.get_mut(entity) {
            let delta = hand_at - Vec2::new(6.0, -6.0) + held_item_offset() - carry.0;
            t.translation = Vec3::new(delta.x * 16.0, -delta.y * 16.0, carry.1);
        }
    }
    // mSM_menu_control_move supplies the 0.3535/tick rate and integer
    // quarter-texel offsets. Sample down/right so the cloth moves up/left.
    assets.scroll = (assets.scroll + 0.3535 * ticks).rem_euclid(1024.0);
    let phase = assets.scroll * 0.25;
    for pattern in &assets.patterns {
        if let Some(mut material) = pattern_materials.get_mut(&pattern.material) {
            material.offset = Vec4::new(assets.slide * 16.0, phase, 0.0, 0.0);
        }
    }
}

pub(crate) struct Paint<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<StandardMaterial>,
    pub root: Entity,
}
impl Paint<'_, '_, '_> {
    pub(crate) fn put(&mut self, mesh: Mesh, tex: Handle<Image>, tint: Color, z: f32) -> Entity {
        self.commands
            .spawn((
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
            ))
            .id()
    }
    pub(crate) fn text(&mut self, font: &MsgAssets, text: &str, at: Vec2, color: [u8; 4], z: f32) {
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
    identity: Res<crate::save::PlayerIdentity>,
    mail: Res<crate::mail::MailUi>,
    mailbox: Res<crate::mail::Mailbox>,
    focus: Res<crate::input::InputFocus>,
    mut assets: ResMut<PocketAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let open = (ui.open && !mail.open) || mail.listing();
    let visible = open
        && matches!(
            *focus,
            crate::input::InputFocus::Pockets | crate::input::InputFocus::Mail
        );
    if !visible {
        if !open && !mail.open && assets.root.is_some() && assets.slide < 300.0 {
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
            let mut entity = commands.spawn((
                Mesh3d(mesh.clone()),
                // Retail issues these blended layers in list order. Bevy
                // needs distinct depths; entity IDs change after reopening.
                Transform::from_xyz(0.0, 0.0, index as f32 * 0.1),
                RenderLayers::layer(1),
                ChildOf(root),
            ));
            if let Some(pattern) = assets.patterns.iter().find(|p| p.source == *material) {
                entity.insert(MeshMaterial3d(pattern.material.clone()));
            } else {
                entity.insert(MeshMaterial3d(material.clone()));
            }
        }
        // The current player appearance in the upper-left preview well.
        // Composite mIV_set_player's separate perspective view.
        commands.spawn((
            Mesh3d(meshes.add(quad_mesh(Vec2::new(20.0, 9.0), Vec2::splat(128.0)))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(assets.portrait.clone()),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 80.0),
            RenderLayers::layer(1),
            ChildOf(root),
        ));
        assets.root = Some(root);
    }
    if !fresh
        && !ui.is_changed()
        && !inventory.is_changed()
        && !identity.is_changed()
        && !mail.is_changed()
        && !mailbox.is_changed()
    {
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
    for (slot, index) in mailbox.slots().into_iter().enumerate() {
        if mail.grabbed == Some(slot) {
            continue;
        }
        let Some(index) = index else {
            continue;
        };
        let letter = &mailbox.letters[index];
        if let Some(tex) = &assets.envelopes[usize::from(letter.read)] {
            paint.put(
                quad_mesh(mail_at(slot) - Vec2::splat(12.0), Vec2::splat(24.0)),
                tex.clone(),
                Color::WHITE,
                120.0,
            );
        }
    }
    let mail_slot = selected_mail(&ui, &mail);
    let at = mail_slot.map_or_else(|| slot_at(ui.selected), mail_at);
    if let Some(letter) = mail
        .grabbed
        .and_then(|slot| mailbox.letter_index(slot))
        .and_then(|i| mailbox.letters.get(i))
    {
        if let Some(tex) = &assets.envelopes[usize::from(letter.read)] {
            let shadow = paint.put(
                quad_mesh(at + Vec2::splat(4.0) - Vec2::splat(12.0), Vec2::splat(24.0)),
                tex.clone(),
                Color::srgba(0.0, 0.0, 0.0, 0.3),
                174.0,
            );
            paint.commands.entity(shadow).insert(PocketCarry(at, 174.0));
            let carry = paint.put(
                quad_mesh(at - Vec2::splat(12.0), Vec2::splat(24.0)),
                tex.clone(),
                Color::WHITE,
                175.0,
            );
            paint.commands.entity(carry).insert(PocketCarry(at, 175.0));
        }
    }
    if let Some(from) = ui.grabbed {
        let item = inventory.pockets.pockets[from];
        let present = inventory.pockets.item_cond(from) & 1 != 0;
        if let Some(tex) = assets.icons.get(&(item, present)) {
            let shadow = paint.put(
                quad_mesh(at + Vec2::splat(4.0) - Vec2::splat(12.0), Vec2::splat(24.0)),
                tex.clone(),
                Color::srgba(0.0, 0.0, 0.0, 0.3),
                174.0,
            );
            paint.commands.entity(shadow).insert(PocketCarry(at, 174.0));
            let carry = paint.put(
                quad_mesh(at - Vec2::splat(12.0), Vec2::splat(24.0)),
                tex.clone(),
                Color::WHITE,
                175.0,
            );
            paint.commands.entity(carry).insert(PocketCarry(at, 175.0));
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
        // Composite the separately rendered source hand view at its eased target.
        let hand = paint.put(
            quad_mesh(Vec2::splat(-16.0), Vec2::splat(32.0)),
            assets.hand_image.clone(),
            Color::WHITE,
            180.0,
        );
        paint.commands.entity(hand).insert(PocketHand);
    }
    let labelled = ui
        .grabbed
        .map_or(selected, |from| inventory.pockets.pockets[from]);
    let label = if let Some(slot) = mail.grabbed.or(mail_slot) {
        mailbox
            .letter_index(slot)
            .and_then(|index| mailbox.letters.get(index))
            .map(|l| match &l.to {
                Some(to) => format!("To {}", to.name()),
                None => format!("From {}", l.sender),
            })
            .unwrap_or_default()
    } else if labelled == 0 {
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
    // mIV_SetLineStrings_centering: town then player, source scale/colors.
    for (name, origin, scale, color) in [
        (
            &identity.town,
            Vec2::new(105.0, 52.0),
            28,
            [60, 80, 110, 255],
        ),
        (
            &identity.player,
            Vec2::new(111.0, 64.0),
            30,
            [70, 70, 100, 255],
        ),
    ] {
        let name: String = name.chars().take(8).collect();
        let text = scaled_text(&name, scale);
        let at = origin + Vec2::new((90.0 - text_width(&text, &font.widths)).max(0.0) * 0.5, 0.0);
        paint.text(&font, &text, at, color, 130.0);
    }
    // mIV_set_money uses 0.875 scale and the same right edge.
    let money = scaled_text(&inventory.bells.to_string(), 28);
    paint.text(
        &font,
        &money,
        Vec2::new(176.5 - text_width(&money, &font.widths), 102.0),
        [255, 60, 0, 255],
        130.0,
    );
    if let Some(choice) = mail.menu.or(ui.menu) {
        let options = if mail.menu.is_some() {
            mail.action_options().to_vec()
        } else {
            ui.actions(selected, inventory.pockets.item_cond(ui.selected))
        };
        if mail.discard {
            paint.text(
                &font,
                &scaled_text("Throw it out?", 28),
                Vec2::new(194.0, 128.0),
                [220, 30, 220, 255],
                162.0,
            );
        }
        let centre = Vec2::new(224.0, 166.0);
        let compact = mail.menu.is_some()
            || ui.selling
            || ui.donating
            || ui.mailing
            || ui.storing.is_some()
            || ui.home;
        let (menu_min, menu_max) = font.choice_bounds;
        for (mesh, material) in &font.choice_body {
            let transform = if compact && menu_min.is_finite() && menu_max.is_finite() {
                let width = options
                    .iter()
                    .map(|s| text_width(s, &font.widths))
                    .chain(
                        mail.discard
                            .then(|| text_width("Throw it out?", &font.widths)),
                    )
                    .fold(0.0, f32::max)
                    * 0.875
                    + 24.0;
                let height =
                    options.len() as f32 * 14.0 + 12.0 + if mail.discard { 22.0 } else { 0.0 };
                let scale = Vec3::new(
                    width * 16.0 / (menu_max.x - menu_min.x).max(1.0),
                    height * 16.0 / (menu_max.y - menu_min.y).max(1.0),
                    1.0,
                );
                let center = Vec2::new(
                    186.0 + width * 0.5,
                    (if mail.discard { 122.0 } else { 144.0 }) + height * 0.5,
                );
                Transform::from_translation(
                    screen(center).with_z(160.0) - (menu_min + menu_max) * 0.5 * scale,
                )
                .with_scale(scale)
            } else {
                Transform::from_translation(screen(centre).with_z(160.0))
                    .with_scale(Vec3::new(0.7, 0.7, 1.0))
            };
            paint.commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                transform,
                RenderLayers::layer(1),
                ChildOf(dynamic),
            ));
        }
        for (i, label) in options.iter().enumerate() {
            let at = Vec2::new(205.0, 150.0 + i as f32 * if compact { 14.0 } else { 16.0 });
            let label = if compact {
                scaled_text(label, 28)
            } else {
                (*label).to_string()
            };
            paint.text(
                &font,
                &label,
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

fn scaled_text(text: &str, scale: u8) -> String {
    use crate::messages::markup;
    format!("{}{}{text}", markup::LINE_SCALE, markup::arg(scale))
}

pub struct PocketDiscPlugin;
impl Plugin for PocketDiscPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "pockets_pattern.wgsl");
        app.add_plugins(MaterialPlugin::<PocketPattern>::default())
            .add_systems(
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
