//! aNSC_show_item_check: presentation only; never changes saved appearance.
use bevy::prelude::*;
use std::collections::HashMap;

use crate::disc_gfx::{decode_texture, PartKey};
use crate::game_disc::GameDisc;
use crate::n64_material::{texture_image, N64Material};
use crate::skeleton::{Animation, Skeleton};

pub(super) mod effects;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Surface {
    Shirt,
    Wall,
    Floor,
}

impl Surface {
    pub fn of(item: u16) -> Option<Self> {
        match item {
            0x2400..=0x24FE => Some(Self::Shirt),
            0x2600..=0x2646 => Some(Self::Floor),
            0x2700..=0x2746 => Some(Self::Wall),
            _ => None,
        }
    }
}

#[derive(Component)]
pub(crate) struct ShopShell;

/// Original handles stay alive while a decoded preview is in use.
#[derive(Component)]
pub(crate) struct PreviewTexture {
    surface: Surface,
    key: PartKey,
    original: (Option<Handle<Image>>, Option<Handle<Image>>),
    cache: HashMap<u16, (Option<Handle<Image>>, Option<Handle<Image>>)>,
    applied: Option<u16>,
}

impl PreviewTexture {
    pub fn new(surface: Surface, key: PartKey, material: &N64Material) -> Self {
        Self {
            surface,
            key,
            original: (material.texture.clone(), material.texture1.clone()),
            cache: default(),
            applied: None,
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct Preview {
    pub active: Option<(usize, u16)>,
    clip: Option<Animation>,
    frame: f32,
    restoring: bool,
    changing: bool,
    gesture: u64,
}

impl Preview {
    pub fn start(&mut self, disc: &GameDisc, index: usize, item: u16) -> bool {
        let Some(surface) = Surface::of(item) else {
            return false;
        };
        if surface == Surface::Shirt {
            if self.clip.is_none() {
                self.clip = (|| {
                    let skeleton = disc.catalog.get("cKF_bs_r_boy_1")?;
                    let skeleton = Skeleton::read(&disc.rom, skeleton.offset)?;
                    let clip = disc.catalog.get("cKF_ba_r_ply_1_menu_change1")?;
                    Animation::read(&disc.rom, clip.offset, skeleton.joints.len())
                })();
            }
            if self.clip.is_none() {
                return false;
            }
            self.frame = 1.0;
            self.changing = true;
            self.gesture = self.gesture.wrapping_add(1);
        }
        self.restoring = false;
        self.active = Some((index, item));
        true
    }

    pub fn clear(&mut self) {
        self.active = None;
        self.changing = false;
        self.restoring = false;
    }

    pub fn shirt(&self) -> bool {
        self.active
            .is_some_and(|(_, item)| Surface::of(item) == Some(Surface::Shirt))
    }

    pub fn restore_shirt(&mut self) {
        self.restoring = true;
        self.changing = true;
        self.frame = 1.0;
        self.gesture = self.gesture.wrapping_add(1);
    }

    pub fn changing(&self) -> bool {
        self.changing
    }

    pub fn animation(&self) -> Option<(&Animation, f32)> {
        (self.shirt() && self.changing)
            .then(|| self.clip.as_ref().map(|a| (a, self.frame)))
            .flatten()
    }

    fn texture_item(&self, surface: Surface) -> Option<u16> {
        let (_, item) = self.active?;
        if Surface::of(item) != Some(surface) {
            return None;
        }
        if surface == Surface::Shirt && ((self.frame < 36.0) != self.restoring) {
            return None;
        }
        Some(item)
    }

    pub fn camera(&self, layout: crate::shop::layout::Layout) -> Option<(Vec3, Vec3)> {
        let (_, item) = self.active?;
        let (mut center, mut eye) = match Surface::of(item)? {
            Surface::Wall => (
                Vec3::new(160.0, 85.0, 180.0),
                Vec3::new(160.0, 235.0, 900.0),
            ),
            Surface::Floor => (
                Vec3::new(160.0, 60.0, 285.0),
                Vec3::new(160.0, 475.0, 840.0),
            ),
            Surface::Shirt => return None,
        };
        // aNSC_request_show_camera selects the second native table for DSUPER.
        if layout.tier == 3 {
            center.z += 80.0;
            eye.z += 80.0;
            if Surface::of(item) == Some(Surface::Wall) {
                eye.y += 50.0;
            }
        }
        let corner = crate::shop::SHOP_ORIGIN - layout.centre();
        Some((corner + center / 40.0, corner + eye / 40.0))
    }
}

pub(super) fn advance(
    time: Res<Time>,
    location: Res<State<crate::shop::ShopLocation>>,
    shop: Res<crate::shop::ShopData>,
    mut preview: ResMut<Preview>,
) {
    if *location != crate::shop::ShopLocation::Inside
        || preview.active.is_some_and(|(index, item)| {
            !shop
                .stock
                .get(index)
                .is_some_and(|s| s.item == item && !s.sold)
        })
    {
        preview.clear();
    }
    if preview.changing {
        let end = preview.clip.as_ref().map_or(1.0, |a| f32::from(a.frames));
        preview.frame = (preview.frame + time.delta_secs() * 60.0).min(end);
        preview.changing = preview.frame < end;
    }
}

pub(super) fn textures(
    disc: Option<Res<GameDisc>>,
    preview: Res<Preview>,
    shells: Query<(), With<ShopShell>>,
    mut parts: Query<(&ChildOf, &MeshMaterial3d<N64Material>, &mut PreviewTexture)>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(disc) = disc else { return };
    for (parent, handle, mut part) in &mut parts {
        if part.surface != Surface::Shirt && !shells.contains(parent.parent()) {
            continue;
        }
        let wanted = preview.texture_item(part.surface);
        if wanted == part.applied {
            continue;
        }
        let Some(mut material) = materials.get_mut(&handle.0) else {
            continue;
        };
        let pair = if let Some(item) = wanted {
            if !part.cache.contains_key(&item) {
                let index = usize::from(item & 255);
                let segs = match part.surface {
                    Surface::Shirt => crate::character::player_segments(
                        &disc,
                        crate::character::PlayerLook {
                            face: 0,
                            cloth: index,
                        },
                    ),
                    Surface::Floor => crate::interior::floor_segments(&disc, index),
                    Surface::Wall => crate::interior::wall_segments(&disc, index),
                };
                let mut replacement = material.clone();
                replacement.texture = part.key.tex.as_ref().and_then(|k| {
                    decode_texture(&disc.rom, k, None, &segs)
                        .map(|rgba| images.add(texture_image(rgba, k)))
                });
                // Failed decoding must leave the original material intact.
                if part.key.tex.is_some() && replacement.texture.is_none() {
                    continue;
                }
                replacement.texture1 = None;
                crate::n64_material::attach_texture1(
                    &mut replacement,
                    &disc.rom,
                    &part.key,
                    None,
                    &segs,
                    &mut images,
                );
                part.cache
                    .insert(item, (replacement.texture, replacement.texture1));
            }
            part.cache[&item].clone()
        } else {
            part.original.clone()
        };
        material.texture = pair.0;
        material.texture1 = pair.1;
        part.applied = wanted;
    }
}

#[derive(Resource, Default)]
pub(super) struct Mannequin {
    hidden: Option<(Entity, Visibility)>,
    shown: Option<Entity>,
    model: Option<Vec<(Handle<Mesh>, Handle<N64Material>)>>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn mannequin(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    preview: Res<Preview>,
    mut displays: Query<(Entity, &crate::shop_display::ShopDisplay, &mut Visibility)>,
    mut state: ResMut<Mannequin>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let target = preview
        .active
        .filter(|_| preview.shirt())
        .and_then(|(index, _)| {
            displays
                .iter()
                .find(|(_, d, _)| d.stock_index == index)
                .map(|(e, d, _)| (e, d.position))
        });
    if state.hidden.map(|(e, _)| e) == target.map(|(e, _)| e) {
        return;
    }
    if let Some((old, visibility)) = state.hidden.take() {
        if let Ok((_, _, mut v)) = displays.get_mut(old) {
            *v = visibility;
        }
    }
    if let Some(old) = state.shown.take() {
        commands.entity(old).try_despawn();
    }
    let (Some((entity, position)), Some(disc)) = (target, disc) else {
        return;
    };
    if state.model.is_none() {
        let Some(list) = disc.catalog.get("obj_shop_manekin_model") else {
            return;
        };
        let (Some(tex), Some(pal)) = (
            disc.player_cloth.get(255 * 0x200..256 * 0x200),
            disc.player_cloth_palettes.get(255 * 0x20..256 * 0x20),
        ) else {
            return;
        };
        let mut segs = crate::disc_gfx::Segments::default();
        segs.set_segment(8, tex.into());
        segs.set_segment(9, pal.into());
        let sink = crate::disc_gfx::run_display_lists(&disc.rom, &[list.offset]);
        state.model = Some(crate::disc_town::sink_meshes_scaled(
            &disc,
            &sink,
            None,
            false,
            &segs,
            0.01 / 40.0,
            (&mut meshes, &mut materials, &mut images),
        ));
    }
    let root = commands
        .spawn((
            Name::new("shop try-on mannequin"),
            Transform::from_translation(position),
            Visibility::default(),
        ))
        .id();
    for (mesh, material) in state.model.as_ref().into_iter().flatten() {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            ChildOf(root),
        ));
    }
    if let Ok((_, _, mut visibility)) = displays.get_mut(entity) {
        state.hidden = Some((entity, *visibility));
        *visibility = Visibility::Hidden;
        state.shown = Some(root);
    }
}

/// Camera2 main_inter: 28 ticks, Hermite slopes .6 and .3, then reverse.
#[derive(Default)]
pub(crate) struct CameraBlend {
    from: Option<(Vec3, Vec3)>,
    goal: Option<(Vec3, Vec3)>,
    tick: f32,
}

impl CameraBlend {
    pub fn view(
        &mut self,
        goal: Option<(Vec3, Vec3)>,
        normal: (Vec3, Vec3),
        ticks: f32,
        inside: bool,
    ) -> (Vec3, Vec3) {
        if !inside {
            *self = Self::default();
            return normal;
        }
        if let Some(goal) = goal {
            self.from.get_or_insert(normal);
            self.goal = Some(goal);
            self.tick = (self.tick + ticks).min(28.0);
        } else {
            self.tick = (self.tick - ticks).max(0.0);
        }
        let (Some(from), Some(to)) = (self.from, self.goal) else {
            return normal;
        };
        if self.tick == 0.0 {
            *self = Self::default();
            return normal;
        }
        let t = self.tick / 28.0;
        let h = -2.0 * t * t * t
            + 3.0 * t * t
            + (t + t * t * t - 2.0 * t * t) * 0.6
            + (t * t * t - t * t) * 0.3;
        (from.0.lerp(to.0, h), from.1.lerp(to.1, h))
    }
}
