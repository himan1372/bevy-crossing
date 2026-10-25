//! Original tag artwork, native placement and retained popup motion.
use super::*;

mod layout;
mod presentation;
pub(super) use presentation::{animate, Presentation};

struct Shape {
    model: Model,
}

pub(super) struct TagModels {
    item: Shape,
    item_shadow: Shape,
    action: Shape,
    action_shadow: Shape,
    confirm: Shape,
    cursor: Shape,
    arrows: [Shape; 2],
}

impl Shape {
    fn load(
        disc: &GameDisc,
        name: &str,
        prim: [u8; 4],
        env: [u8; 4],
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<N64Material>,
        images: &mut Assets<Image>,
    ) -> Option<Self> {
        let symbol = |name: &str| {
            disc.catalog
                .get(name)
                .filter(|s| s.source == RomSource::Rel)
                .map(|s| s.offset)
        };
        let sink = run_display_lists_with_colors(
            &disc.rom,
            &[symbol("sen_item2_DL_mode")?, symbol(name)?],
            prim,
            255,
            env,
            &[],
        );
        // Cache before Bevy extracts CPU mesh data to the render world.
        let (mut min, mut max) = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
        for vertex in sink.parts.iter().flat_map(|p| &p.vertices) {
            let at = Vec3::from(vertex.pos) * 16.0;
            min = min.min(at);
            max = max.max(at);
        }
        if !min.is_finite() || !max.is_finite() || max.x <= min.x || max.y <= min.y {
            return None;
        }
        let segments = Segments::default();
        let model = crate::disc_town::sink_meshes_scaled(
            disc,
            &sink,
            None,
            true,
            &segments,
            16.0,
            (meshes, materials, images),
        );
        // Source I4 shadow intensity supplies coverage for its alpha combiner.
        for (part, (_, handle)) in sink.parts.iter().zip(&model) {
            let Some(key) = part.key.tex.as_ref() else {
                continue;
            };
            if key.format != rustimal_logic::graphics::texture::GxTexFormat::I4 {
                continue;
            }
            if let Some(mut rgba) = crate::disc_gfx::decode_texture(&disc.rom, key, None, &segments)
            {
                for pixel in rgba.as_chunks_mut::<4>().0 {
                    pixel[3] = pixel[0];
                }
                if let Some(mut material) = materials.get_mut(handle) {
                    material.texture =
                        Some(images.add(crate::n64_material::texture_image(rgba, key)));
                }
            }
        }
        Some(Self { model })
    }

    fn draw(&self, paint: &mut Paint, transform: Transform, name: &str) {
        for (i, (mesh, material)) in self.model.iter().enumerate() {
            let mut transform = transform;
            transform.translation.z += i as f32 * 0.01;
            paint.commands.spawn((
                Name::new(name.to_string()),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                transform,
                RenderLayers::layer(1),
                ChildOf(paint.root),
            ));
        }
    }
}

impl TagModels {
    pub(super) fn load(
        disc: &GameDisc,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<N64Material>,
        images: &mut Assets<Image>,
    ) -> Option<Self> {
        let mut load =
            |name, prim, env| Shape::load(disc, name, prim, env, meshes, materials, images);
        Some(Self {
            // mTG_set_win_field: ordinary item draw0, action draw1, confirm draw2.
            item: load(
                "sen_itemw_wakuT_model",
                [255, 245, 140, 255],
                [215, 165, 60, 150],
            )?,
            item_shadow: load("sen_itemw_kage_model", [255; 4], [0; 4])?,
            action: load(
                "sen_win_wakuT_model",
                [190, 235, 245, 255],
                [120, 120, 200, 150],
            )?,
            action_shadow: load("sen_win_kageT_model", [255; 4], [0; 4])?,
            confirm: load(
                "sen_win_wakuT_model",
                [205, 245, 255, 255],
                [110, 110, 200, 150],
            )?,
            cursor: load("sen_win_cursor_model", [215, 30, 215, 255], [0; 4])?,
            arrows: [
                load("sen_itemw_yajirushi_model", [215, 165, 60, 255], [0; 4])?,
                load("sen_itemw_yajirushi2T_model", [215, 165, 60, 255], [0; 4])?,
            ],
        })
    }

    fn layout(&self, paint: &mut Paint, layout: layout::Layout, kind: usize) {
        let (frame, shadow) = if kind == 0 {
            (&self.item, &self.item_shadow)
        } else if kind == 2 {
            (&self.confirm, &self.action_shadow)
        } else {
            (&self.action, &self.action_shadow)
        };
        let transform = Transform::from_translation((layout.body * 16.0).extend(0.1))
            .with_scale((layout.body_scale * 0.875).extend(1.0));
        // Shadow vertices already carry the source offset: share the frame's transform.
        let mut shadow_transform = transform;
        shadow_transform.translation.z -= 0.2;
        shadow.draw(paint, shadow_transform, "retail tag shadow");
        frame.draw(
            paint,
            transform,
            if kind == 0 {
                "retail item tag"
            } else {
                "retail action tag"
            },
        );
        if layout.arrow != 0 {
            self.arrows[usize::from(layout.arrow - 1)].draw(
                paint,
                Transform::from_xyz(0.0, 0.0, 0.05),
                "retail tag arrow",
            );
        }
    }
}
