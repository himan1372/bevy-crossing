//! Debug model viewer for models on the player's disc.
//!
//! `--view <name>` loads `<name>_mat_model` + `<name>_gfx_model` if both
//! exist, otherwise the display list(s) named (comma-separated, in order).
//! `--pal <name>[:index]` supplies a palette for colour-indexed textures
//! whose palette is normally set by game code (e.g. seasonal tree colours).
//!
//! The model is shown on its own, away from the town, with its own camera
//! (the town camera is switched off). Left-drag orbits, scroll zooms,
//! `[` / `]` step through the palettes in the `--pal` asset.

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use disc::AssetCatalog;

use crate::disc_gfx::{decode_texture, run_display_lists, Segments};
use crate::game_disc::GameDisc;
use crate::n64_material::{part_mesh, texture_image, CombinerUniform, N64Material};
use crate::states::AppState;
use crate::town::CELL_SIZE;

/// Game models are drawn at 1/100 scale; one game cell is 40 world units.
const MODEL_SCALE: f32 = 0.01 / 40.0 * CELL_SIZE;
const PALETTE_BYTES: u32 = 32;
/// Where the model is shown: far from the town so nothing else is in view.
const VIEW_ORIGIN: Vec3 = Vec3::new(0.0, 0.0, -5000.0);
const ORBIT_SPEED: f32 = 0.008;
const ZOOM_SPEED: f32 = 0.1;

#[derive(Resource)]
struct ViewRequest {
    names: String,
    palette: Option<String>,
}

/// Live viewer state, created once the disc is ready.
#[derive(Resource)]
struct Viewer {
    names: String,
    lists: Vec<u32>,
    /// Palette asset: (name, REL offset of palette 0, palette count).
    palettes: Option<(String, u32, u32)>,
    palette_index: u32,
    vertex_colors: bool,
    rebuild: bool,
    framed: bool,
    triangles: usize,
    yaw: f32,
    pitch: f32,
    distance: f32,
    focus: Vec3,
}

#[derive(Component)]
struct ViewedModel;

#[derive(Component)]
struct ViewerCamera;

#[derive(Component)]
struct ViewerLabel;

fn arg_after(args: &[String], flag: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == flag).map(|w| w[1].clone())
}

/// Display-list REL offsets for a `--view` value.
fn resolve_lists(catalog: &AssetCatalog, names: &str) -> Result<Vec<u32>, String> {
    let (mat, gfx) = (format!("{names}_mat_model"), format!("{names}_gfx_model"));
    if let (Some(m), Some(g)) = (catalog.get(&mat), catalog.get(&gfx)) {
        return Ok(vec![m.offset, g.offset]);
    }
    names
        .split(',')
        .map(|n| {
            catalog
                .get(n)
                .map(|a| a.offset)
                .ok_or_else(|| format!("no asset named {n}"))
        })
        .collect()
}

/// `name` or `name:index` → (name, palette 0 offset, palette count, index).
fn resolve_palette(catalog: &AssetCatalog, spec: &str) -> Result<(String, u32, u32, u32), String> {
    let (name, index) = match spec.split_once(':') {
        Some((n, i)) => (
            n,
            i.parse::<u32>()
                .map_err(|_| format!("bad palette index {i}"))?,
        ),
        None => (spec, 0),
    };
    let asset = catalog
        .get(name)
        .ok_or_else(|| format!("no asset named {name}"))?;
    let count = (asset.size / PALETTE_BYTES).max(1);
    if index >= count {
        return Err(format!("{name} has {count} palettes"));
    }
    Ok((name.to_string(), asset.offset, count, index))
}

/// Turns the `--view` request into a [`Viewer`] once the disc is loaded,
/// and switches to the viewer's own camera.
fn start_viewer(mut commands: Commands, request: Res<ViewRequest>, disc: Option<Res<GameDisc>>) {
    commands.remove_resource::<ViewRequest>();
    let Some(disc) = disc else {
        warn!("--view needs the game disc; nothing to show");
        return;
    };
    let lists = match resolve_lists(&disc.catalog, &request.names) {
        Ok(l) => l,
        Err(e) => return warn!("--view {}: {e}", request.names),
    };
    let palette = request
        .palette
        .as_deref()
        .map(|p| resolve_palette(&disc.catalog, p));
    let (palettes, palette_index) = match palette {
        Some(Err(e)) => return warn!("--pal: {e}"),
        Some(Ok((name, base, count, index))) => (Some((name, base, count)), index),
        None => (None, 0),
    };
    commands.insert_resource(Viewer {
        names: request.names.clone(),
        lists,
        palettes,
        palette_index,
        vertex_colors: true,
        rebuild: true,
        framed: false,
        triangles: 0,
        yaw: 0.0,
        pitch: 0.3,
        distance: 3.0,
        focus: VIEW_ORIGIN,
    });
    commands.spawn((
        ViewerCamera,
        Camera3d::default(),
        Camera {
            order: 1,
            ..default()
        },
        Transform::default(),
    ));
    commands.spawn((
        ViewerLabel,
        Text::default(),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

fn rebuild_model(
    mut commands: Commands,
    mut viewer: ResMut<Viewer>,
    disc: Res<GameDisc>,
    old: Query<Entity, With<ViewedModel>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !viewer.rebuild {
        return;
    }
    viewer.rebuild = false;
    for e in &old {
        commands.entity(e).despawn();
    }
    let palette = viewer
        .palettes
        .as_ref()
        .map(|(_, base, _)| base + viewer.palette_index * PALETTE_BYTES);
    let sink = run_display_lists(&disc.rom, &viewer.lists);
    viewer.triangles = sink.triangle_count();
    if sink.aborted {
        warn!(
            "--view {}: display list never ended; drawing what it emitted",
            viewer.names
        );
    }

    // Frame the camera on the model's bounds the first time.
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for v in sink.parts.iter().flat_map(|p| &p.vertices) {
        let p = Vec3::from(v.pos) * MODEL_SCALE;
        (lo, hi) = (lo.min(p), hi.max(p));
    }
    if viewer.triangles > 0 && !viewer.framed {
        viewer.framed = true;
        viewer.focus = VIEW_ORIGIN + (lo + hi) / 2.0;
        viewer.distance = (hi - lo).length().max(0.5) * 1.5;
    }

    let root = commands
        .spawn((
            ViewedModel,
            Transform::from_translation(VIEW_ORIGIN),
            Visibility::default(),
        ))
        .id();
    for part in &sink.parts {
        let k = &part.key;
        debug!(
            "part: {} tris, combine {:06x} {:08x}, prim {:?}, env {:?}, lit {}, 2cyc {}, tex {:?}",
            part.vertices.len() / 3,
            k.combine.0,
            k.combine.1,
            k.prim,
            k.env,
            k.lit,
            k.two_cycle,
            k.tex.map(|t| t.img_addr)
        );
        let texture = part.key.tex.as_ref().and_then(|k| {
            let rgba = decode_texture(&disc.rom, k, palette, &Segments::default());
            if rgba.is_none() {
                warn!("can't decode texture {k:?}");
            }
            rgba.map(|rgba| images.add(texture_image(rgba, k)))
        });
        let material = materials.add(N64Material {
            combiner: CombinerUniform::from_key(&part.key, !viewer.vertex_colors, false),
            texture,
            blend: false,
        });
        commands.spawn((
            Mesh3d(meshes.add(part_mesh(part, MODEL_SCALE))),
            MeshMaterial3d(material),
            ChildOf(root),
        ));
    }
}

fn viewer_controls(
    mut viewer: ResMut<Viewer>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
) {
    if buttons.pressed(MouseButton::Left) {
        viewer.yaw -= motion.delta.x * ORBIT_SPEED;
        viewer.pitch = (viewer.pitch + motion.delta.y * ORBIT_SPEED).clamp(-1.5, 1.5);
    }
    if scroll.delta.y != 0.0 {
        let step = 1.0 - scroll.delta.y.signum() * ZOOM_SPEED;
        viewer.distance = (viewer.distance * step).max(0.05);
    }
    let count = viewer.palettes.as_ref().map_or(1, |(_, _, c)| *c);
    if keys.just_pressed(KeyCode::BracketRight) {
        viewer.palette_index = (viewer.palette_index + 1) % count;
        viewer.rebuild = true;
    }
    if keys.just_pressed(KeyCode::KeyV) {
        viewer.vertex_colors = !viewer.vertex_colors;
        viewer.rebuild = true;
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        viewer.palette_index = (viewer.palette_index + count - 1) % count;
        viewer.rebuild = true;
    }
}

fn update_viewer_camera(
    viewer: Res<Viewer>,
    mut viewer_cam: Query<&mut Transform, With<ViewerCamera>>,
    mut others: Query<&mut Camera, (Without<ViewerCamera>, With<Camera3d>)>,
    mut label: Query<&mut Text, With<ViewerLabel>>,
) {
    for mut cam in &mut others {
        cam.is_active = false;
    }
    if let Ok(mut t) = viewer_cam.single_mut() {
        let dir = Quat::from_euler(EulerRot::YXZ, viewer.yaw, -viewer.pitch, 0.0) * Vec3::Z;
        *t = Transform::from_translation(viewer.focus + dir * viewer.distance)
            .looking_at(viewer.focus, Vec3::Y);
    }
    if let Ok(mut text) = label.single_mut() {
        let pal = match &viewer.palettes {
            Some((name, _, count)) => {
                format!("{name} {}/{count}  ([ ] to change)", viewer.palette_index)
            }
            None => "none (--pal <name>)".to_string(),
        };
        text.0 = format!(
            "MODEL VIEWER  {}\n{} triangles   palette: {pal}\nleft-drag: orbit   scroll: zoom   V: shade {}",
            viewer.names,
            viewer.triangles,
            if viewer.vertex_colors { "on" } else { "off" }
        );
    }
}

pub struct ModelViewPlugin;

impl Plugin for ModelViewPlugin {
    fn build(&self, app: &mut App) {
        let args: Vec<String> = std::env::args().collect();
        let Some(names) = arg_after(&args, "--view") else {
            return;
        };
        app.insert_resource(ViewRequest {
            names,
            palette: arg_after(&args, "--pal"),
        })
        .add_systems(
            Update,
            (
                start_viewer
                    .run_if(in_state(AppState::InGame).and_then(resource_exists::<ViewRequest>)),
                (viewer_controls, rebuild_model, update_viewer_camera)
                    .chain()
                    .run_if(resource_exists::<Viewer>),
            ),
        );
    }
}
