//! Audited US television callbacks; all assets decoded from the disc.
use super::*;
use crate::disc_gfx::decode_texture;
use crate::n64_material::{texture_image, CombinerUniform};
use rustimal_logic::television::Television;

pub(crate) fn kind(profile: &str) -> Option<Television> {
    Some(match profile.strip_prefix("iam_")? {
        "sum_tv01" => Television::Tv01,
        "sum_tv02" => Television::Tv02,
        "sum_fruittv01" => Television::Fruit,
        "kon_snowtv" => Television::Snow,
        _ => return None,
    })
}

/// Symbol differences are explicit: the snowman uses face/body and07a..d,
/// unlike the other sets' on/off parts and front1..4. Neither model name
/// denotes a mutually exclusive power state; both parts are always drawn.
fn asset_names(profile: &str) -> Option<([String; 2], Vec<String>)> {
    let kind = kind(profile)?;
    let base = profile.strip_prefix("iam_")?;
    let (models, screens) = if matches!(kind, Television::Snow) {
        (
            ["face_model", "body_model"],
            ["07a", "07b", "07c", "07d"].as_slice(),
        )
    } else if matches!(kind, Television::Tv02) {
        (
            ["on_model", "off_model"],
            ["_front1", "_front2", "_front3"].as_slice(),
        )
    } else {
        (
            ["on_model", "off_model"],
            ["_front1", "_front2", "_front3", "_front4"].as_slice(),
        )
    };
    Some((
        models.map(|suffix| format!("int_{base}_{suffix}")),
        screens
            .iter()
            .map(|suffix| format!("int_{base}{suffix}_TA_tex_txt"))
            .collect(),
    ))
}

/// Placement and rendering use the same complete model/screen asset set.
pub(super) fn available(disc: &GameDisc, profile: &Profile) -> bool {
    let Some((models, screens)) = asset_names(&profile.name) else {
        return false;
    };
    models.iter().chain(&screens).all(|name| {
        disc.catalog.get(name).is_some_and(|asset| {
            disc.rom
                .slice(RomSource::Rel, asset.offset, asset.size)
                .is_some()
        })
    })
}

pub(crate) fn item_kind(disc: &GameDisc, index: u16) -> Option<Television> {
    kind(&read_profile(disc, u32::from(index))?.name)
}

#[derive(Component)]
pub(crate) struct TelevisionRig {
    pub(crate) kind: Television,
    pub(crate) on: bool,
    ticks: f32,
    screen: u8,
    // Only segment-8 materials change. Keep every frame handle alive and
    // replace textures rather than spawning new assets each animation tick.
    screens: Vec<(Handle<N64Material>, Vec<Handle<Image>>)>,
}

impl TelevisionRig {
    pub(crate) fn update(&mut self, on: bool, seconds: f32, materials: &mut Assets<N64Material>) {
        self.on = on;
        if on {
            self.ticks = self.kind.advance(self.ticks, seconds);
        }
        let screen = self.kind.screen(on, self.ticks);
        if self.screen == screen {
            return;
        }
        self.screen = screen;
        for (handle, frames) in &self.screens {
            if let Some(mut material) = materials.get_mut(handle) {
                material.texture = frames.get(usize::from(screen - 1)).cloned();
            }
        }
    }
}

#[allow(clippy::too_many_arguments)] // Existing furniture asset stores.
pub(super) fn spawn(
    commands: &mut Commands,
    disc: &GameDisc,
    parent: Entity,
    profile: &Profile,
    kind: Television,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) {
    let base = profile.name.trim_start_matches("iam_");
    let Some((models, screens)) = asset_names(&profile.name) else {
        return;
    };
    let lists: Option<Vec<_>> = models
        .iter()
        .map(|name| disc.catalog.get(name).map(|a| a.offset))
        .collect();
    let segments: Option<Vec<_>> = screens
        .iter()
        .map(|name| {
            let asset = disc.catalog.get(name)?;
            let bytes = disc.rom.slice(RomSource::Rel, asset.offset, asset.size)?;
            let mut segments = Segments::default();
            segments.set_segment(8, Arc::from(bytes));
            Some(segments)
        })
        .collect();
    let (Some(lists), Some(segments)) = (lists, segments) else {
        warn!("incomplete television assets for {base}");
        return;
    };
    let count = segments.len();
    let mut screens = Vec::new();
    for part in &run_display_lists(&disc.rom, &lists).parts {
        let frames: Vec<_> = part.key.tex.as_ref().map_or_else(Vec::new, |key| {
            let animated = key.img_addr >> 24 == 8;
            segments
                .iter()
                .take(if animated { count } else { 1 })
                .filter_map(|segments| {
                    decode_texture(&disc.rom, key, profile.palette, segments)
                        .map(|rgba| images.add(texture_image(rgba, key)))
                })
                .collect()
        });
        let material = materials.add(N64Material {
            combiner: CombinerUniform::from_key(&part.key, false, false),
            texture: frames.first().cloned(),
            env: default(),
            blend: false,
        });
        if frames.len() == count {
            screens.push((material.clone(), frames));
        }
        commands.spawn((
            Mesh3d(meshes.add(crate::n64_material::part_mesh(part, 1.0))),
            MeshMaterial3d(material),
            ChildOf(parent),
        ));
    }
    commands.entity(parent).insert(TelevisionRig {
        kind,
        on: false,
        ticks: 0.0,
        screen: 1,
        screens,
    });
}
