//! ac_yaz_candle fYC_dw: body and texture-edge flame, segment9, two ticks/frame.
use super::*;
use crate::disc_gfx::decode_texture;
use crate::n64_material::{texture_image, CombinerUniform};

const MODELS: [&str; 2] = ["int_yaz_candle_body_model", "int_yaz_candle_fire_model"];
const FRAMES: [&str; 2] = [
    "int_yaz_candle_fire1_tex_txt",
    "int_yaz_candle_fire2_tex_txt",
];

pub(super) fn available(disc: &GameDisc) -> bool {
    MODELS.iter().chain(&FRAMES).all(|name| {
        disc.catalog
            .get(name)
            .is_some_and(|a| disc.rom.slice(RomSource::Rel, a.offset, a.size).is_some())
    })
}

#[derive(Component)]
pub(crate) struct Candle {
    ticks: f32,
    frame: usize,
    flames: Vec<(Handle<N64Material>, [Handle<Image>; 2])>,
}

pub(super) fn spawn(
    commands: &mut Commands,
    disc: &GameDisc,
    parent: Entity,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) {
    if !available(disc) {
        warn!("incomplete festive candle assets");
        return;
    }
    let lists: Vec<_> = MODELS
        .iter()
        .filter_map(|name| disc.catalog.get(name).map(|a| a.offset))
        .collect();
    let segments: Vec<_> = FRAMES
        .iter()
        .filter_map(|name| {
            let asset = disc.catalog.get(name)?;
            let mut segs = Segments::default();
            segs.set_segment(
                9,
                Arc::from(disc.rom.slice(RomSource::Rel, asset.offset, asset.size)?),
            );
            Some(segs)
        })
        .collect();
    let mut flames = Vec::new();
    for part in &run_display_lists(&disc.rom, &lists).parts {
        let animated = part
            .key
            .tex
            .as_ref()
            .is_some_and(|key| key.img_addr >> 24 == 9);
        let textures: Vec<_> = part.key.tex.as_ref().map_or_else(Vec::new, |key| {
            segments
                .iter()
                .take(if animated { 2 } else { 1 })
                .filter_map(|segs| {
                    decode_texture(&disc.rom, key, None, segs)
                        .map(|rgba| images.add(texture_image(rgba, key)))
                })
                .collect()
        });
        // Both lists use POLY_OPA_DISP and texture-edge alpha in fYC_dw.
        let material = materials.add(N64Material {
            combiner: CombinerUniform::from_key(&part.key, false, false),
            texture: textures.first().cloned(),
            texture1: None,
            env: default(),
            blend: false,
        });
        if animated {
            if let Ok(frames) = textures.try_into() {
                flames.push((material.clone(), frames));
            }
        }
        commands.spawn((
            Mesh3d(meshes.add(crate::n64_material::part_mesh(part, 1.0))),
            MeshMaterial3d(material),
            ChildOf(parent),
        ));
    }
    commands.entity(parent).insert(Candle {
        ticks: 0.0,
        frame: 0,
        flames,
    });
}

pub(crate) fn update(
    time: Res<Time>,
    mut candles: Query<&mut Candle>,
    mut materials: ResMut<Assets<N64Material>>,
) {
    for mut candle in &mut candles {
        candle.ticks = (candle.ticks + time.delta_secs() * 60.0) % 4.0;
        let frame = usize::from(candle.ticks >= 2.0);
        if candle.frame == frame {
            continue;
        }
        candle.frame = frame;
        for (handle, frames) in &candle.flames {
            if let Some(mut material) = materials.get_mut(handle) {
                material.texture = Some(frames[frame].clone());
            }
        }
    }
}
