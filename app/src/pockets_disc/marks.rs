//! mSM_set_dl_item: the original pulsing green mark, above the item icon.
use super::*;

pub(super) fn load(
    disc: &GameDisc,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) -> Model {
    let (Some(mode), Some(mark)) = (
        disc.catalog.get("inv_item_mode"),
        disc.catalog.get("inv_mark_model"),
    ) else {
        return Vec::new();
    };
    let sink = run_display_lists_with_colors(
        &disc.rom,
        &[mode.offset, mark.offset],
        [0, 95, 0, 255],
        0,
        [0, 210, 0, 255],
        &[],
    );
    crate::disc_town::sink_meshes_scaled(
        disc,
        &sink,
        None,
        true,
        &Segments::default(),
        16.0,
        (meshes, materials, images),
    )
}

pub(super) fn pulse(model: &Model, ticks: f32, materials: &mut Assets<N64Material>) {
    let t = (ticks.rem_euclid(40.0) / 40.0 * 2.0 - 1.0).abs();
    let t = 1.0 - t;
    for (_, handle) in model {
        if let Some(mut material) = materials.get_mut(handle) {
            material.combiner.prim = Vec4::new(0.0, (t * 90.0 + 95.0) / 255.0, 0.0, 1.0);
            material.combiner.env = Vec4::new(
                t * 60.0 / 255.0,
                (t * 45.0 + 210.0) / 255.0,
                t * 60.0 / 255.0,
                1.0,
            );
        }
    }
}

pub(super) fn draw(model: &Model, paint: &mut Paint, at: Vec2) {
    for (mesh, material) in model {
        paint.commands.spawn((
            Name::new("pocket sale mark"),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(screen(at).with_z(122.0)),
            RenderLayers::layer(1),
            ChildOf(paint.root),
        ));
    }
}
