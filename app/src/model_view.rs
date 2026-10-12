//! Debug model viewer: draws one model from the player's disc in the town.
//!
//! `--view <name>` loads `<name>_mat_model` + `<name>_gfx_model` if both
//! exist, otherwise the display list(s) named (comma-separated).
//! `--pal <name>[:index]` supplies a palette for colour-indexed textures
//! whose palette is normally set by game code (e.g. seasonal tree colours).
//! The model appears two cells in front of the player.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use disc::AssetCatalog;

use crate::disc_gfx::{decode_texture, run_display_lists, Part, TexKey};
use crate::game_disc::GameDisc;
use crate::player::Player;
use crate::states::AppState;
use crate::town::CELL_SIZE;

/// Game models are drawn at 1/100 scale; one game cell is 40 world units.
const MODEL_SCALE: f32 = 0.01 / 40.0 * CELL_SIZE;
/// N64 texture coordinates are s10.5 fixed point.
const ST_ONE: f32 = 32.0;
const PALETTE_BYTES: u32 = 32;

#[derive(Resource)]
struct ViewRequest {
    names: String,
    palette: Option<String>,
}

#[derive(Component)]
struct ViewedModel;

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

/// `name` or `name:index` → REL offset of that 16-colour palette.
fn resolve_palette(catalog: &AssetCatalog, spec: &str) -> Result<u32, String> {
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
    if (index + 1) * PALETTE_BYTES > asset.size.max(PALETTE_BYTES) {
        return Err(format!("{name} has no palette {index}"));
    }
    Ok(asset.offset + index * PALETTE_BYTES)
}

fn part_mesh(part: &Part, key: Option<&TexKey>) -> Mesh {
    let (w, h) = key.map_or((1.0, 1.0), |k| (k.width as f32, k.height as f32));
    let v = &part.vertices;
    let positions: Vec<[f32; 3]> = v.iter().map(|v| v.pos.map(|c| c * MODEL_SCALE)).collect();
    let uvs: Vec<[f32; 2]> = v
        .iter()
        .map(|v| [v.st[0] / ST_ONE / w, v.st[1] / ST_ONE / h])
        .collect();
    let colors: Vec<[f32; 4]> = v
        .iter()
        .map(|v| match part.lit {
            true => [1.0; 4],
            false => v.color.map(|c| c as f32 / 255.0),
        })
        .collect();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.compute_flat_normals();
    mesh
}

fn address_mode(gx_wrap: u8) -> ImageAddressMode {
    match gx_wrap {
        1 => ImageAddressMode::Repeat,
        2 => ImageAddressMode::MirrorRepeat,
        _ => ImageAddressMode::ClampToEdge,
    }
}

fn texture_image(rgba: Vec<u8>, key: &TexKey) -> Image {
    let size = Extent3d {
        width: key.width as u32,
        height: key.height as u32,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new(
        size,
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: address_mode(key.wrap_s),
        address_mode_v: address_mode(key.wrap_t),
        ..ImageSamplerDescriptor::linear()
    });
    image
}

fn spawn_view(
    mut commands: Commands,
    request: Res<ViewRequest>,
    disc: Option<Res<GameDisc>>,
    player: Query<&Transform, With<Player>>,
    existing: Query<(), With<ViewedModel>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(player) = player.single() else { return };
    if !existing.is_empty() {
        return;
    }
    let Some(disc) = disc else {
        warn!("--view needs the game disc; nothing to show");
        commands.remove_resource::<ViewRequest>();
        return;
    };
    commands.remove_resource::<ViewRequest>();
    let lists = match resolve_lists(&disc.catalog, &request.names) {
        Ok(l) => l,
        Err(e) => return warn!("--view {}: {e}", request.names),
    };
    let palette = match request
        .palette
        .as_deref()
        .map(|p| resolve_palette(&disc.catalog, p))
    {
        Some(Err(e)) => return warn!("--pal: {e}"),
        Some(Ok(p)) => Some(p),
        None => None,
    };
    let sink = run_display_lists(&disc.rom, &lists);
    info!(
        "--view {}: {} triangles in {} parts",
        request.names,
        sink.triangle_count(),
        sink.parts.len()
    );

    let at = player.translation + player.forward() * 2.0 * CELL_SIZE;
    let root = commands
        .spawn((
            ViewedModel,
            Name::new(format!("view {}", request.names)),
            Transform::from_translation(at),
            Visibility::default(),
        ))
        .id();
    for (key, part) in &sink.parts {
        let texture = key.as_ref().and_then(|k| {
            let rgba = decode_texture(&disc.rom, k, palette);
            if rgba.is_none() {
                warn!("can't decode texture {k:?}");
            }
            rgba.map(|rgba| images.add(texture_image(rgba, k)))
        });
        let material = materials.add(StandardMaterial {
            base_color_texture: texture,
            alpha_mode: AlphaMode::Mask(0.5),
            double_sided: true,
            cull_mode: None,
            perceptual_roughness: 1.0,
            ..default()
        });
        commands.spawn((
            Mesh3d(meshes.add(part_mesh(part, key.as_ref()))),
            MeshMaterial3d(material),
            ChildOf(root),
        ));
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
            spawn_view.run_if(in_state(AppState::InGame).and_then(resource_exists::<ViewRequest>)),
        );
    }
}
