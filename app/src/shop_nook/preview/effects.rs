//! ef_kigae / ef_kigae_light: the original spiral of white change sparkles.
use bevy::prelude::*;

use super::Preview;
use crate::character::CharacterRig;
use crate::game_disc::GameDisc;
use crate::n64_material::{texture_image, CombinerUniform, N64Material};
use crate::player::{Player, WorldCamera};

struct Spiral {
    position: Vec3,
    angle: i16,
    tick: u8,
}

struct Sparkle {
    position: Vec3,
    velocity: f32,
    age: u8,
}

#[derive(Resource, Default)]
pub(crate) struct Effects {
    gesture: u64,
    fired: bool,
    ticks: f32,
    spirals: Vec<Spiral>,
    particles: Vec<Sparkle>,
    sprites: Vec<Entity>,
    model: Option<Vec<(Handle<Mesh>, Handle<N64Material>)>>,
    rng: u32,
}

impl Effects {
    fn clear(&mut self, commands: &mut Commands) {
        self.spirals.clear();
        self.particles.clear();
        self.ticks = 0.0;
        self.fired = false;
        for entity in self.sprites.drain(..) {
            commands.entity(entity).try_despawn();
        }
    }

    fn jitter(&mut self) -> f32 {
        let mut x = if self.rng == 0 { 0x2545_F491 } else { self.rng };
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x as f64 / (u32::MAX as f64 + 1.0)) as f32 * 5.0 - 2.5
    }

    fn step(&mut self) {
        for particle in &mut self.particles {
            particle.age += 1;
            particle.velocity -= 0.025 / 40.0;
            particle.position.y += particle.velocity;
        }
        self.particles.retain(|p| p.age < 30);
        for i in 0..self.spirals.len() {
            let spiral = &mut self.spirals[i];
            spiral.angle = spiral.angle.wrapping_sub(1638); // DEG2SHORT(9)
            spiral.tick += 1;
            if !spiral.tick.is_multiple_of(2) {
                continue;
            }
            let theta = f32::from(spiral.angle) * std::f32::consts::TAU / 65536.0;
            // Source mv runs before timer--, so tick2 observes timer59.
            let timer = f32::from(61 - spiral.tick);
            let offset = Vec3::new(
                theta.sin() * 22.0,
                10.0 - 35.0 * timer / 60.0,
                theta.cos() * 22.0,
            );
            let position = spiral.position + offset / 40.0;
            let jitter = Vec3::new(self.jitter(), self.jitter(), self.jitter()) / 40.0;
            self.particles.push(Sparkle {
                position: position + jitter,
                velocity: 0.0,
                age: 0,
            });
        }
        self.spirals.retain(|s| s.tick < 60);
    }
}

pub(crate) fn reset(mut commands: Commands, mut effects: ResMut<Effects>) {
    effects.clear(&mut commands);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update(
    mut commands: Commands,
    time: Res<Time>,
    location: Res<State<crate::shop::ShopLocation>>,
    preview: Res<Preview>,
    disc: Option<Res<GameDisc>>,
    players: Query<&Transform, With<Player>>,
    rigs: Query<(&CharacterRig, &ChildOf, &Transform)>,
    globals: Query<&GlobalTransform>,
    cameras: Query<&Transform, With<WorldCamera>>,
    mut loads: MessageReader<crate::sound::ResetAudio>,
    mut effects: ResMut<Effects>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let loaded = loads.read().count() != 0;
    if loaded || *location != crate::shop::ShopLocation::Inside {
        effects.clear(&mut commands);
        return;
    }
    if preview.shirt() {
        if effects.gesture != preview.gesture {
            effects.gesture = preview.gesture;
            effects.fired = false;
        }
        if preview.frame >= 16.0 && !effects.fired {
            effects.fired = true;
            if let Some((rig, root, player)) = rigs.iter().find_map(|(rig, parent, root)| {
                Some((rig, root, players.get(parent.parent()).ok()?))
            }) {
                // The C trigger reads feel_pos from the previous draw callback.
                if let Some(at) = rig.feel_joint().and_then(|j| globals.get(j).ok()) {
                    let (yaw, _, _) = root.rotation.to_euler(EulerRot::YXZ);
                    let angle = (yaw * 65536.0 / std::f32::consts::TAU) as i32 as i16;
                    effects.spirals.push(Spiral {
                        position: at.translation(),
                        angle: angle.wrapping_add(16384),
                        tick: 0,
                    });
                }
                crate::sound::se_at(crate::sound::se_id::WEAR, player.translation);
                info!(
                    "shop wear effect gesture={} restoring={} frame={:.2} sound=0x11c",
                    preview.gesture, preview.restoring, preview.frame
                );
            }
        }
    }
    effects.ticks = (effects.ticks + time.delta_secs() * 60.0).min(4.0);
    while effects.ticks >= 1.0 {
        effects.ticks -= 1.0;
        effects.step();
    }
    if !effects.particles.is_empty() && effects.model.is_none() {
        effects.model = disc.as_ref().and_then(|disc| {
            // ef_flash and ef_kigae_light use the same legacy IA8 sparkle.
            crate::furniture::gong::flash_parts(disc, 255)?
                .iter()
                .map(|part| {
                    let key = part.key.tex.as_ref()?;
                    let rgba = crate::furniture::gong::flash_rgba(disc, part)?;
                    let texture = images.add(texture_image(rgba, key));
                    let material = materials.add(N64Material {
                        combiner: CombinerUniform::from_key(&part.key, false, true),
                        texture: Some(texture),
                        texture1: None,
                        env: default(),
                        blend: true,
                    });
                    Some((
                        meshes.add(crate::n64_material::part_mesh(part, 1.0)),
                        material,
                    ))
                })
                .collect()
        });
    }
    let count = if effects.model.is_some() {
        effects.particles.len()
    } else {
        0
    };
    while effects.sprites.len() > count {
        if let Some(e) = effects.sprites.pop() {
            commands.entity(e).try_despawn();
        }
    }
    while effects.sprites.len() < count {
        let root = commands
            .spawn((
                Name::new("shop change sparkle"),
                Transform::default(),
                Visibility::default(),
            ))
            .id();
        for (mesh, material) in effects.model.as_ref().into_iter().flatten() {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                ChildOf(root),
            ));
        }
        effects.sprites.push(root);
    }
    let rotation = cameras.single().map_or(Quat::IDENTITY, |c| c.rotation);
    for (&entity, particle) in effects.sprites.iter().zip(&effects.particles) {
        let age = f32::from(particle.age);
        let scale = if age < 10.0 {
            0.009 * age / 10.0
        } else {
            0.009 * (30.0 - age) / 20.0
        };
        commands.entity(entity).insert(Transform {
            translation: particle.position,
            rotation,
            scale: Vec3::splat(scale / 40.0),
        });
    }
}
