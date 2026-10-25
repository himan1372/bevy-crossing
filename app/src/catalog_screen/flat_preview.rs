//! mCL_item_draw's flat previews, using its original models and segment map.
use std::collections::HashMap;

use disc::Target;

use super::*;
use crate::disc_gfx::Segments;

#[derive(Resource, Default)]
pub(super) struct Cache(HashMap<u16, Model>);

#[derive(Component)]
pub(super) struct FlatPreview;

#[derive(Component)]
pub(super) struct MusicMotion {
    ticks: f32,
}

fn symbol(disc: &GameDisc, name: &str) -> Option<u32> {
    disc.catalog
        .get(name)
        .filter(|s| s.source == RomSource::Rel)
        .map(|s| s.offset)
}

fn paper_pointer(disc: &GameDisc, name: &str, index: u16) -> Option<u32> {
    match disc
        .rom
        .pointer_at(RomSource::Rel, symbol(disc, name)? + u32::from(index) * 4)?
    {
        Target::File(RomSource::Rel, offset) => Some(offset),
        _ => None,
    }
}

fn load(
    disc: &GameDisc,
    item: u16,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) -> Option<Model> {
    let mut lists = vec![symbol(disc, "mCL_lat_letter_mode")?];
    let mut segments = Segments::default();
    match item {
        0x2000..=0x203f => {
            lists.push(paper_pointer(disc, "paper_disp_model", item - 0x2000)?);
            if let Some(ruling) = paper_pointer(disc, "paper_disp_sen_model", item - 0x2000) {
                lists.extend([symbol(disc, "mCL_lat_letter_sen_mode")?, ruling]);
            }
        }
        0x2600..=0x2646 | 0x2700..=0x2746 => {
            let wall = item >= 0x2700;
            let (bank, base, block, palette, count, model) = if wall {
                (
                    &disc.player_room_walls,
                    0x2700,
                    0x1020,
                    10,
                    2,
                    "mCL_rom_myhome1_wall_model",
                )
            } else {
                // Catalog floor palette is D; the room shell uses C.
                (
                    &disc.player_room_floors,
                    0x2600,
                    0x2020,
                    13,
                    4,
                    "mCL_rom_myhome1_floor_model",
                )
            };
            let start = usize::from(item - base) * block;
            let bytes = bank.get(start..start + block)?;
            segments.set_segment(palette, bytes[..0x20].into());
            for i in 0..count {
                let start = 0x20 + i * 0x800;
                segments.set_segment(8 + i as u8, bytes[start..start + 0x800].into());
            }
            lists.push(symbol(disc, model)?);
        }
        0x2a00 => lists.push(symbol(disc, "mCL_music_model")?),
        _ => return None,
    }
    let sink = crate::disc_gfx::run_display_lists(&disc.rom, &lists);
    if sink.parts.is_empty() {
        return None;
    }
    let model = crate::disc_town::sink_meshes_scaled(
        disc,
        &sink,
        None,
        true,
        &segments,
        MODEL_SCALE,
        (meshes, materials, images),
    );
    // As in mail_disc: I4 stationery ruling uses intensity as coverage.
    if (0x2000..=0x203f).contains(&item) {
        for (part, (_, handle)) in sink.parts.iter().zip(&model) {
            let Some(key) = part.key.tex.as_ref() else {
                continue;
            };
            if key.format != rustimal_logic::graphics::texture::GxTexFormat::I4 {
                continue;
            }
            if let Some(mut rgba) = crate::disc_gfx::decode_texture(&disc.rom, key, None, &segments)
            {
                for p in rgba.as_chunks_mut::<4>().0 {
                    p[3] = p[0];
                }
                if let Some(mut material) = materials.get_mut(handle) {
                    material.texture =
                        Some(images.add(crate::n64_material::texture_image(rgba, key)));
                }
            }
        }
    }
    Some(model)
}

pub(super) fn show(
    mut commands: Commands,
    ui: Res<CatalogUi>,
    disc: Option<Res<GameDisc>>,
    roots: Query<(Entity, Option<&Children>), With<CatalogRoot>>,
    previews: Query<(), With<FlatPreview>>,
    mut cache: ResMut<Cache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(disc) = disc else { return };
    let Ok((root, children)) = roots.single() else {
        return;
    };
    if children.is_some_and(|kids| kids.iter().any(|e| previews.contains(e))) {
        return;
    }
    let Some(&item) = ui.lists[ui.page].get((ui.top[ui.page] + ui.row[ui.page]) as usize) else {
        return;
    };
    let (key, scale, y) = match ui.page {
        1 | 2 => (item, 0.54, 7.0),
        // Catalog papers use the fourth stack range (20C0..20FF).
        // mCL_paper_init selects the unique design modulo64.
        5 => (0x2000 + (item - 0x2000) % 64, 0.28, 4.0),
        8 => (0x2a00, 0.55, 7.0),
        _ => return,
    };
    let model = cache.0.entry(key).or_insert_with(|| {
        load(&disc, key, &mut meshes, &mut materials, &mut images).unwrap_or_default()
    });
    let piece = commands
        .spawn((
            Name::new(format!("catalog flat preview {item:#06x}")),
            FlatPreview,
            Transform::from_xyz(-85.0 * MODEL_SCALE, y * MODEL_SCALE, 2.5)
                .with_scale(Vec3::new(scale, scale, 1.0)),
            Visibility::default(),
            RenderLayers::layer(LAYER),
            ChildOf(root),
        ))
        .id();
    if ui.page == 8 {
        commands.entity(piece).insert(MusicMotion { ticks: 60.0 });
    }
    paint(
        &mut commands,
        model,
        Vec3::ZERO,
        &mut 0.0,
        &RenderLayers::layer(LAYER),
        piece,
    );
}

pub(super) fn animate(time: Res<Time>, mut notes: Query<(&mut MusicMotion, &mut Transform)>) {
    for (mut motion, mut transform) in &mut notes {
        motion.ticks = (motion.ticks - time.delta_secs() * 60.0).rem_euclid(65536.0);
        // Source sin_s/cos_s angles: 3deg=0x222, 6deg=0x444 per 60Hz tick.
        let angle = motion.ticks * 546.0 * std::f32::consts::TAU / 65536.0;
        transform.translation.x = (-85.0 + angle.sin() * 6.0) * MODEL_SCALE;
        transform.translation.y = (7.0 + (angle * 2.0).sin() * 6.0) * MODEL_SCALE;
        transform.rotation = Quat::from_rotation_z(angle.cos() * std::f32::consts::TAU / 16.0);
    }
}
