//! ac_radio's effect32, arg0=1: silent KONPU notes from the user's disc.
use bevy::camera::visibility::VisibilitySystems;
use bevy::prelude::*;
use bevy::transform::TransformSystems;
use rustimal_logic::{
    game_math::{cos_s, sin_s},
    radio,
};

use crate::{disc_gfx::Segments, game_disc::GameDisc, n64_material::N64Material};

const UNIT: f32 = 40.0;
const PRIM: [[u8; 3]; 5] = [
    [255, 255, 0],
    [255, 150, 255],
    [0, 255, 0],
    [100, 120, 255],
    [255, 100, 50],
];
const ENV: [[u8; 3]; 5] = [
    [100, 50, 0],
    [150, 0, 150],
    [0, 70, 0],
    [0, 0, 200],
    [150, 0, 0],
];

#[derive(Component, Default)]
pub(super) struct Emitter {
    counter: i16,
    fraction: f32,
}

struct Note {
    position: Vec3,
    velocity: Vec3,
    age: u8,
    phase: i16,
    roll: f32,
    model: usize,
    color: usize,
}

impl Note {
    fn new(position: Vec3, random: &mut rustimal_logic::qrand::Qrand) -> Self {
        let model = random.random(3) as usize;
        let color = random.random(5) as usize;
        let angle = |degrees: f32| (degrees * 65536.0 / 360.0) as i16;
        let roll = angle(random.fqrand() * 60.0 - 30.0);
        let x = angle(random.fqrand2() * 30.0);
        let z = angle(random.fqrand2() * 30.0);
        let speed = (random.fqrand() * 4.0 + 8.0) * 0.5;
        let velocity = Vec3::new(
            speed * sin_s(z),
            speed * cos_s(z) * cos_s(x),
            speed * cos_s(z) * sin_s(x),
        );
        Self {
            position,
            velocity: Quat::from_rotation_y(135f32.to_radians())
                * Quat::from_rotation_x(25f32.to_radians())
                * velocity,
            age: 0,
            phase: 0,
            roll: f32::from(roll) * std::f32::consts::TAU / 65536.0,
            model,
            color,
        }
    }

    fn step(&mut self) {
        self.age += 1;
        self.phase = self.phase.wrapping_add(2047);
        self.position += self.velocity / UNIT;
        self.velocity *= 0.8f32.sqrt();
    }

    fn scale(&self) -> Vec3 {
        let age = f32::from(self.age);
        let adjust = |end: f32, a: f32, b: f32| a + (b - a) * (age / end).clamp(0.0, 1.0);
        let scale = adjust(18.0, 0.00156, 0.0078);
        let max = adjust(30.0, 1.35, 0.85);
        let min = adjust(30.0, 0.05, 0.55);
        let sine = sin_s(self.phase);
        Vec3::new(
            min + (sine + 1.0) * 0.5 * (max - min),
            min + (1.0 - sine) * 0.5 * (max - min),
            1.0,
        ) * scale
            / UNIT
    }
}

type Part = (Entity, Handle<N64Material>);

struct Slot {
    note: Option<Note>,
    visuals: [Vec<Part>; 3],
}

#[derive(Resource, Default)]
struct Notes {
    owner: Option<Entity>,
    slots: Vec<Slot>,
}

fn hide(notes: &mut Notes, commands: &mut Commands) {
    for slot in &mut notes.slots {
        slot.note = None;
        for (entity, _) in slot.visuals.iter().flatten() {
            commands.entity(*entity).insert(Visibility::Hidden);
        }
    }
    notes.owner = None;
}

// Assets/entities are allocated once: at most three slots, each with three variants.
fn build(
    notes: &mut Notes,
    disc: &GameDisc,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) {
    let models: [Vec<(Handle<Mesh>, Handle<N64Material>)>; 3] = std::array::from_fn(|variant| {
        let name = format!("ef_onpu01_{variant:02}_modelT");
        let Some(list) = disc.catalog.get(&name) else {
            return Vec::new();
        };
        let sink = crate::disc_gfx::run_display_lists(&disc.rom, &[list.offset]);
        crate::disc_town::sink_meshes_scaled(
            disc,
            &sink,
            None,
            true,
            &Segments::default(),
            1.0,
            (meshes, materials, images),
        )
    });
    for slot in 0..3 {
        notes.slots.push(Slot {
            note: None,
            visuals: std::array::from_fn(|variant| {
                models[variant]
                    .iter()
                    .filter_map(|(mesh, original)| {
                        let material = materials.get(original)?.clone();
                        let material = materials.add(material);
                        let entity = commands
                            .spawn((
                                Name::new(format!("radio note {slot} variant {variant}")),
                                Mesh3d(mesh.clone()),
                                MeshMaterial3d(material.clone()),
                                Transform::default(),
                                Visibility::Hidden,
                            ))
                            .id();
                        Some((entity, material))
                    })
                    .collect()
            }),
        });
    }
}

// One owner advances native ticks, manages pooled assets and clears reload/scene state.
#[allow(clippy::too_many_arguments)]
fn update(
    mut commands: Commands,
    time: Res<Time>,
    app: Res<State<crate::states::AppState>>,
    pause: Option<Res<State<crate::states::PauseState>>>,
    mut loads: MessageReader<crate::sound::ResetAudio>,
    mut emitters: Query<(Entity, &Transform, &mut Emitter)>,
    player: Query<&Transform, With<crate::player::Player>>,
    cameras: Query<&GlobalTransform, With<crate::player::WorldCamera>>,
    disc: Option<Res<GameDisc>>,
    mut random: ResMut<crate::game_rand::GameRand>,
    mut notes: ResMut<Notes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let loaded = loads.read().count() != 0;
    if loaded || *app.get() != crate::states::AppState::InGame || emitters.is_empty() {
        hide(&mut notes, &mut commands);
        for (_, _, mut emitter) in &mut emitters {
            *emitter = Emitter::default();
        }
        return;
    }
    if pause.is_none_or(|state| *state.get() != crate::states::PauseState::Running) {
        return;
    }
    let (Some(disc), Ok(camera)) = (disc, cameras.single()) else {
        return;
    };
    let Ok((owner, at, mut emitter)) = emitters.single_mut() else {
        return;
    };
    let block = |position| {
        crate::town::Town::cell_indices_of(position).map(|(x, z, _, _)| (x as i32, z as i32))
    };
    let nearby = player
        .single()
        .ok()
        .and_then(|player| {
            Some(!radio::radio_despawns(
                block(at.translation)?,
                block(player.translation)?,
                true,
            ))
        })
        .unwrap_or(false);
    if !nearby {
        hide(&mut notes, &mut commands);
        *emitter = Emitter::default();
        return;
    }
    if notes.owner != Some(owner) {
        hide(&mut notes, &mut commands);
        notes.owner = Some(owner);
    }
    if notes.slots.is_empty() {
        build(
            &mut notes,
            &disc,
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut images,
        );
    }
    emitter.fraction += time.delta_secs() * 60.0;
    while emitter.fraction >= 1.0 {
        emitter.fraction -= 1.0;
        for slot in &mut notes.slots {
            if let Some(note) = &mut slot.note {
                note.step();
                if note.age >= 72 {
                    slot.note = None;
                }
            }
        }
        if radio::effect_ticks(&mut emitter.counter, 1) != 0 {
            if let Some(slot) = notes.slots.iter_mut().find(|slot| slot.note.is_none()) {
                slot.note = Some(Note::new(
                    at.translation + Vec3::from_array(radio::RADIO_NOTE_OFFSET) / UNIT,
                    &mut random.0,
                ));
            }
        }
    }
    let billboard = camera.rotation();
    for slot in &notes.slots {
        for (variant, parts) in slot.visuals.iter().enumerate() {
            let note = slot.note.as_ref().filter(|note| note.model == variant);
            for (entity, material) in parts {
                commands.entity(*entity).insert(if note.is_some() {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
                let Some(note) = note else { continue };
                let transform = Transform {
                    translation: note.position,
                    rotation: billboard * Quat::from_rotation_z(note.roll),
                    scale: note.scale(),
                };
                commands
                    .entity(*entity)
                    .insert((transform, GlobalTransform::from(transform)));
                if let Some(mut material) = materials.get_mut(material) {
                    let rgb = PRIM[note.color].map(|c| f32::from(c) / 255.0);
                    let alpha = ((72.0 - f32::from(note.age)) / 12.0).clamp(0.0, 1.0);
                    material.combiner.prim =
                        Vec4::new(rgb[0], rgb[1], rgb[2], (alpha * 255.0).floor() / 255.0);
                    // Models01/02 override the caller's environment in their own lists.
                    if variant == 0 {
                        let env = ENV[note.color].map(|c| f32::from(c) / 255.0);
                        material.combiner.env = Vec4::new(env[0], env[1], env[2], 1.0);
                    }
                    material.combiner.extra.x = 1.0;
                }
            }
        }
    }
}

pub(super) fn register(app: &mut App) {
    app.init_resource::<Notes>().add_systems(
        PostUpdate,
        update
            .after(TransformSystems::Propagate)
            .before(VisibilitySystems::VisibilityPropagate)
            .before(VisibilitySystems::CalculateBounds)
            .before(VisibilitySystems::CheckVisibility),
    );
}
