//! Runtime-disc fish/insect inventory pages from `m_inventory_ovl.c`.
use super::*;
use rustimal_logic::critter_collection as native;

pub(super) struct Frames {
    full: [Model; 2],
    parts: [Model; 2],
    inventory: Model,
    scroll: Vec<Handle<N64Material>>,
}

/// Native `mTG_TABLE_COLLECT` anchors, converted to the shared 320x240 UI.
pub(super) fn slot_at(index: usize) -> Vec2 {
    const X: [f32; 8] = [-89.0, -65.0, -41.0, -17.0, 8.0, 32.0, 57.0, 81.0];
    const Y: [f32; 5] = [42.0, 18.0, -6.0, -30.0, -54.0];
    let index = index.min(native::COUNT - 1);
    Vec2::new(160.0 + X[index % 8], 120.0 - Y[index / 8])
}

pub(super) fn load(
    disc: &GameDisc,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
    patterns: &mut Vec<PatternTile>,
    pattern_materials: &mut Assets<PocketPattern>,
) -> Frames {
    let mut model = |names: &[&str]| {
        build(
            disc,
            names,
            [0; 4],
            16.0,
            meshes,
            materials,
            images,
            patterns,
            pattern_materials,
        )
    };
    let full = ["sakana", "mushi"].map(|kind| {
        model(&[
            "inv_mwin_mode",
            &format!("inv_{kind}_scroll_mode"),
            &format!("inv_{kind}_model"),
        ])
    });
    let parts = ["sakana", "mushi"]
        .map(|kind| model(&["inv_mwin_mode", &format!("inv_{kind}_part_model")]));
    let inventory = model(&["inv_mwin_mode", "inv_mwin_model"]);
    let mut scroll = Vec::new();
    for (kind, models) in ["sakana", "mushi"].into_iter().zip(&full) {
        let Some(cloth) = disc.catalog.get(&format!("inv_{kind}_nuno_tex_rgb_ci4")) else {
            continue;
        };
        let entries: Vec<_> = [
            "inv_mwin_mode".to_string(),
            format!("inv_{kind}_scroll_mode"),
            format!("inv_{kind}_model"),
        ]
        .iter()
        .filter_map(|name| disc.catalog.get(name).map(|s| s.offset))
        .collect();
        let sink = run_display_lists_with_colors(&disc.rom, &entries, [255; 4], 255, [0; 4], &[]);
        for (part, (_, handle)) in sink.parts.iter().zip(models) {
            // Tile 0 alone is the 32x32 repeating cloth. Tile 1 holds the
            // frame coverage and must retain its original fixed UVs.
            if part.key.tex.as_ref().is_some_and(|texture| {
                texture.img_addr == cloth.offset
                    && texture.width == 32
                    && texture.height == 32
                    && texture.wrap_s == 1
                    && texture.wrap_t == 1
            }) {
                scroll.push(handle.clone());
            }
        }
    }
    Frames {
        full,
        parts,
        inventory,
        scroll,
    }
}

/// `mIV_set_base_frame_dl` moves tile 0 in integer quarter-texel steps.
/// `phase` is the shared menu texture position multiplied by 0.25.
pub(super) fn animate(frames: &Frames, phase: f32, materials: &mut Assets<N64Material>) {
    let offset = ((phase * 4.0).floor() * 0.25).rem_euclid(32.0) / 32.0;
    for handle in &frames.scroll {
        let Some(material) = materials.get(handle) else {
            continue;
        };
        if material.combiner.extra.y == offset && material.combiner.extra.z == offset {
            continue;
        }
        if let Some(mut material) = materials.get_mut(handle) {
            material.combiner.extra.y = offset;
            material.combiner.extra.z = offset;
        }
    }
}

fn draw_model(commands: &mut Commands, root: Entity, assets: &PocketAssets, model: &Model, z: f32) {
    for (index, (mesh, material)) in model.iter().enumerate() {
        let mut entity = commands.spawn((
            Mesh3d(mesh.clone()),
            Transform::from_xyz(0.0, 0.0, z + index as f32 * 0.1),
            RenderLayers::layer(1),
            ChildOf(root),
        ));
        if let Some(pattern) = assets.patterns.iter().find(|p| p.source == *material) {
            entity.insert(MeshMaterial3d(pattern.material.clone()));
        } else {
            entity.insert(MeshMaterial3d(material.clone()));
        }
    }
}

/// Native inactive page parts expose the other two header tabs.
/// Call once beneath the ordinary inventory frame when creating its root.
pub(super) fn draw_back_pages(
    commands: &mut Commands,
    root: Entity,
    assets: &PocketAssets,
    page: usize,
) {
    let frames = &assets.collection_frames;
    let (back, middle) = match page {
        native::PAGE_INVENTORY => (&frames.parts[1], &frames.parts[0]),
        native::PAGE_FISH => (&frames.parts[1], &frames.inventory),
        native::PAGE_INSECTS => (&frames.parts[0], &frames.inventory),
        _ => return,
    };
    draw_model(commands, root, assets, back, -20.0);
    draw_model(commands, root, assets, middle, -10.0);
}

/// These pages inspect discoveries; they never take ownership of pocket items.
#[allow(clippy::too_many_arguments)] // Existing disc paint context and shared caches.
pub(super) fn draw(
    paint: &mut Paint,
    disc: &GameDisc,
    font: &MsgAssets,
    assets: &mut PocketAssets,
    ui: &PocketsUi,
    catalog: &crate::catalog::Catalog,
    images: &mut Assets<Image>,
) {
    let Some(frame) = ui
        .page
        .checked_sub(1)
        .and_then(|p| assets.collection_frames.full.get(p))
    else {
        return;
    };
    draw_back_pages(paint.commands, paint.root, assets, ui.page);
    draw_model(paint.commands, paint.root, assets, frame, 0.0);
    for index in 0..native::COUNT {
        let Some(item) = native::item(ui.page, index) else {
            continue;
        };
        if !native::has(&catalog.collection, item) {
            continue;
        }
        let key = (item, false, 1);
        if let std::collections::hash_map::Entry::Vacant(entry) = assets.icons.entry(key) {
            if let Some(rgba) = icon_frame(disc, item, false, 1) {
                entry.insert(images.add(image(rgba, 32, 32)));
            }
        }
        if let Some(texture) = assets.icons.get(&key) {
            paint.put(
                quad_mesh(slot_at(index) - Vec2::splat(12.0), Vec2::splat(24.0)),
                texture.clone(),
                Color::WHITE,
                100.0,
            );
        }
    }
    let at = slot_at(ui.collection_selected);
    if assets.hand.is_empty() {
        paint.put(
            quad_mesh(at - Vec2::new(22.0, 7.0), Vec2::splat(14.0)),
            font.cursor.clone(),
            Color::srgb_u8(0, 195, 185),
            130.0,
        );
    } else {
        let hand = paint.put(
            quad_mesh(Vec2::splat(-16.0), Vec2::splat(32.0)),
            assets.hand_image.clone(),
            Color::WHITE,
            180.0,
        );
        paint.commands.entity(hand).insert(PocketHand);
    }
    let label = native::item(ui.page, ui.collection_selected)
        .filter(|&item| native::has(&catalog.collection, item))
        .map(crate::items::name)
        .unwrap_or_default();
    // Tags outlive the dynamic icon root, as they do on the inventory page.
    let root = assets.root.unwrap_or(paint.root);
    let PocketAssets {
        tags, tag_state, ..
    } = assets;
    if let Some(models) = tags {
        // Name tags use the existing native artwork and retained animation.
        tag_state.show(
            models,
            paint,
            font,
            root,
            at,
            &label,
            &[],
            None,
            false,
            None,
        );
    } else if !label.is_empty() {
        let width = text_width(&label, &font.widths);
        paint.text(
            font,
            &label,
            Vec2::new(160.0 - width * 0.5, 196.0),
            [50, 60, 50, 255],
            141.0,
        );
    }
}
