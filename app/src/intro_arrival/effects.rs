//! ac_train0's original chimney smoke and cylinder steam, rendered from disc lists.
use bevy::prelude::*;

use crate::disc_gfx::Segments;
use crate::game_disc::GameDisc;
use crate::n64_material::N64Material;

type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;

struct Puff {
    steam: bool,
    age: u8,
    position: Vec3,
    velocity: Vec3,
    entity: Option<Entity>,
    materials: Vec<Handle<N64Material>>,
}

#[derive(Resource, Default)]
pub(super) struct Effects {
    particles: Vec<Puff>,
    smoke: Option<Model>,
    steam: Vec<Model>,
    prepared: bool,
}

impl Effects {
    pub(super) fn clear(&mut self, commands: &mut Commands, materials: &mut Assets<N64Material>) {
        for puff in self.particles.drain(..) {
            retire(puff, commands, materials);
        }
    }

    /// Run alongside the controller at 60 Hz; already emitted puffs stay in world space.
    pub(super) fn tick(
        &mut self,
        at: Vec3,
        cues: &rustimal_logic::train_control::Cues,
        random: &mut crate::game_rand::GameRand,
        commands: &mut Commands,
        materials: &mut Assets<N64Material>,
    ) {
        let mut i = 0;
        while i < self.particles.len() {
            let puff = &mut self.particles[i];
            if puff.steam {
                puff.velocity.y += 0.125 / super::UNIT;
                puff.position += puff.velocity;
                puff.velocity *= 0.8_f32.sqrt();
            } else {
                let age = f32::from(puff.age);
                puff.position.y += (2.2 + (0.5 - 2.2) * (age / 20.0).min(1.0)) / super::UNIT;
            }
            puff.age += 1;
            if puff.age >= if puff.steam { 30 } else { 80 } {
                retire(self.particles.swap_remove(i), commands, materials);
            } else {
                i += 1;
            }
        }
        if cues.smoke {
            self.particles.push(Puff {
                steam: false,
                age: 0,
                position: at
                    + Vec3::new(
                        36.0 + random.0.fqrand() * 5.0 - 2.5,
                        110.0,
                        random.0.fqrand() * 5.0 - 2.5,
                    ) / super::UNIT,
                velocity: Vec3::ZERO,
                entity: None,
                materials: Vec::new(),
            });
        }
        if let Some(index) = cues.steam {
            let angle = [-0x3000_i16, 0x0400][index] as f32 * std::f32::consts::TAU / 65536.0;
            let speed = random.0.fqrand() + 0.5;
            self.particles.push(Puff {
                steam: true,
                age: 0,
                position: at + Vec3::new([15.0, 22.0][index], 21.0, 42.0) / super::UNIT,
                velocity: Vec3::new(
                    angle.sin() * speed,
                    -random.0.fqrand() * 3.0 - 1.5,
                    angle.cos() * speed,
                ) / super::UNIT,
                entity: None,
                materials: Vec::new(),
            });
        }
    }
}

fn retire(puff: Puff, commands: &mut Commands, materials: &mut Assets<N64Material>) {
    if let Some(entity) = puff.entity {
        commands.entity(entity).try_despawn();
    }
    for material in puff.materials {
        materials.remove(material.id());
    }
}

fn model(
    disc: &GameDisc,
    name: &str,
    textures: Option<(usize, usize)>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) -> Option<Model> {
    let list = disc.catalog.get(name)?;
    let sink = crate::disc_gfx::run_display_lists(&disc.rom, &[list.offset]);
    let mut segments = Segments::default();
    if let Some((a, b)) = textures {
        for (segment, index) in [(8, a), (9, b)] {
            let texture = disc.catalog.get(&format!("ef_dust01_{index}"))?;
            let bytes = disc
                .rom
                .slice(texture.source, texture.offset, texture.size)?;
            segments.set_segment(segment, bytes.into());
        }
    }
    let parts = crate::disc_town::sink_meshes_scaled(
        disc,
        &sink,
        None,
        true,
        &segments,
        1.0,
        (meshes, materials, images),
    );
    (!parts.is_empty()).then_some(parts)
}

pub(super) fn draw(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    mut effects: ResMut<Effects>,
    cameras: Query<&Transform, With<crate::player::WorldCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    if effects.particles.is_empty() {
        return;
    }
    let Some(disc) = disc else { return };
    if !effects.prepared {
        effects.prepared = true;
        effects.smoke = model(
            &disc,
            "ef_kisha_kemuri01_modelT",
            None,
            &mut meshes,
            &mut materials,
            &mut images,
        );
        effects.steam = [(0, 1), (2, 1), (2, 3), (3, 3)]
            .into_iter()
            .map(|pair| {
                model(
                    &disc,
                    "ef_dust01_modelT",
                    Some(pair),
                    &mut meshes,
                    &mut materials,
                    &mut images,
                )
            })
            .collect::<Option<Vec<_>>>()
            .unwrap_or_default();
        info!(
            "intro train effects: smoke={} steam variants={}",
            effects.smoke.is_some(),
            effects.steam.len()
        );
    }
    let rotation = cameras
        .single()
        .map_or(Quat::IDENTITY, |camera| camera.rotation);
    let effects = &mut *effects;
    for puff in &mut effects.particles {
        let age = f32::from(puff.age);
        let k = (age * 0.5).min(14.0);
        let frame = k as usize;
        let variant = match frame {
            0..=3 => 0,
            4..=7 => 1,
            8..=11 => 2,
            _ => 3,
        };
        let model = if puff.steam {
            effects.steam.get(variant)
        } else {
            effects.smoke.as_ref()
        };
        let Some(model) = model else { continue };
        let entity = *puff.entity.get_or_insert_with(|| {
            let root = commands
                .spawn((
                    Name::new(if puff.steam {
                        "train steam"
                    } else {
                        "train smoke"
                    }),
                    Transform::default(),
                    Visibility::default(),
                ))
                .id();
            for (mesh, original) in model {
                let Some(material) = materials.get(original).cloned() else {
                    continue;
                };
                let material = materials.add(material);
                commands.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    ChildOf(root),
                ));
                puff.materials.push(material);
            }
            root
        });
        let scale = if puff.steam {
            0.005 + 0.015 * age / 30.0
        } else {
            0.003 + 0.024 * (age / 40.0).min(1.0)
        };
        commands.entity(entity).insert(Transform {
            translation: puff.position,
            rotation,
            scale: Vec3::splat(scale / super::UNIT),
        });
        const LOD: [u8; 15] = [
            0, 64, 128, 192, 255, 192, 128, 64, 0, 64, 128, 192, 255, 255, 255,
        ];
        let lod = if puff.steam {
            f32::from(LOD[frame])
                + (f32::from(LOD[(frame + 1).min(14)]) - f32::from(LOD[frame])) * k.fract()
        } else {
            128.0
        };
        let color = if puff.steam {
            Vec4::ONE
        } else {
            Vec4::new(
                30.0 / 255.0,
                30.0 / 255.0,
                30.0 / 255.0,
                (200.0 - 196.0 * ((age - 40.0) / 40.0).clamp(0.0, 1.0)) / 255.0,
            )
        };
        for (handle, (_, original)) in puff.materials.iter().zip(model) {
            let textures = materials
                .get(original)
                .map(|m| (m.texture.clone(), m.texture1.clone()));
            if let Some(mut material) = materials.get_mut(handle) {
                material.combiner.prim = color;
                material.combiner.extra.x = lod / 255.0;
                if let Some((a, b)) = textures {
                    material.texture = a;
                    material.texture1 = b;
                }
            }
        }
    }
}
