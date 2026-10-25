//! mTG_set_win_field artwork. Popup anchors remain the port's current layout.
use super::*;

struct Shape {
    model: Model,
    min: Vec3,
    max: Vec3,
}

pub(super) struct TagModels {
    item: Shape,
    item_shadow: Shape,
    action: Shape,
    action_shadow: Shape,
    confirm: Shape,
    cursor: Shape,
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
        Some(Self { model, min, max })
    }

    fn fitted(&self, rect: Rect, z: f32) -> Transform {
        let scale = Vec3::new(
            rect.width() * 16.0 / (self.max.x - self.min.x),
            rect.height() * 16.0 / (self.max.y - self.min.y),
            1.0,
        );
        Transform::from_translation(
            screen(rect.center()).with_z(z) - (self.min + self.max) * 0.5 * scale,
        )
        .with_scale(scale)
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
        })
    }

    pub(super) fn frame(&self, paint: &mut Paint, rect: Rect, z: f32, item: bool, confirm: bool) {
        let (frame, shadow) = if item {
            (&self.item, &self.item_shadow)
        } else if confirm {
            (&self.confirm, &self.action_shadow)
        } else {
            (&self.action, &self.action_shadow)
        };
        let rect = if item { rect } else { Self::action_body(rect) };
        let transform = frame.fitted(rect, z);
        // Shadow vertices already carry the source offset: share the frame's transform.
        let mut shadow_transform = transform;
        shadow_transform.translation.z -= 0.2;
        shadow.draw(paint, shadow_transform, "retail tag shadow");
        frame.draw(
            paint,
            transform,
            if item {
                "retail item tag"
            } else {
                "retail action tag"
            },
        );
    }

    fn action_body(rect: Rect) -> Rect {
        // mTG_set_tag_win_scale_p(SELECT2), including transparent padding.
        let text = rect.size() - Vec2::new(24.0, 12.0);
        let fraction = Vec2::new((text.x - 31.5) / 42.0, (text.y - 28.0) / 42.0);
        let minimum = Vec2::new(0.5714286, 0.43243244);
        let body_scale = minimum + fraction * (Vec2::ONE - minimum);
        let size = Vec2::new(122.5, 129.5) * 0.875 * body_scale;
        Rect::from_center_size(rect.center(), size)
    }

    pub(super) fn action_shift(rect: Rect) -> f32 {
        // Keep tall popups clear of the port's name tag at y193.
        (Self::action_body(rect).max.y - 190.0).max(0.0)
    }

    pub(super) fn cursor(&self, paint: &mut Paint, at: Vec2, z: f32) {
        self.cursor.draw(
            paint,
            self.cursor
                .fitted(Rect::from_corners(at, at + Vec2::splat(8.0)), z),
            "retail tag cursor",
        );
    }
}
