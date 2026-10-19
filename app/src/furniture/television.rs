//! Three audited US television callbacks; all assets decoded from the disc.
use super::*;
use crate::disc_gfx::decode_texture;
use crate::n64_material::{texture_image, CombinerUniform};
use rustimal_logic::television::Television;

pub(crate) fn kind(profile: &str) -> Option<Television> {
    Some(match profile.strip_prefix("iam_")? {
        "sum_tv01" => Television::Tv01,
        "sum_tv02" => Television::Tv02,
        "sum_fruittv01" => Television::Fruit,
        _ => return None,
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
    let lists: Vec<_> = ["on_model", "off_model"]
        .into_iter()
        .filter_map(|suffix| {
            disc.catalog
                .get(&format!("int_{base}_{suffix}"))
                .map(|a| a.offset)
        })
        .collect();
    let count = if matches!(kind, Television::Tv02) {
        3
    } else {
        4
    };
    let segments: Vec<_> = (1..=count)
        .filter_map(|frame| {
            let asset = disc
                .catalog
                .get(&format!("int_{base}_front{frame}_TA_tex_txt"))?;
            let bytes = disc.rom.slice(RomSource::Rel, asset.offset, asset.size)?;
            let mut segments = Segments::default();
            segments.set_segment(8, Arc::from(bytes));
            Some(segments)
        })
        .collect();
    if segments.len() != count {
        warn!("incomplete television screen assets for {base}");
        return;
    }
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
