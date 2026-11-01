//! The intro's native manpu effects (`ef_shock`, `warau`, `ha`, `gimonhu`,
//! `kangaeru`, `hirameki_den/hikari`, `ase2`). Models remain on the user's disc.
//! Pooled mesh entities/materials avoid allocating assets on every reaction.
use std::collections::HashMap;

use bevy::camera::visibility::VisibilitySystems;
use bevy::prelude::*;
use bevy::transform::TransformSystems;

use crate::character::CharacterRig;
use crate::disc_gfx::Segments;
use crate::game_disc::GameDisc;
use crate::intro_reactions::DialogueReaction;
use crate::n64_material::N64Material;

const UNIT: f32 = 40.0;
// The original effect controller has 100 slots; inactive slots are reused.
const MAX_EFFECTS: usize = 100;
type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct EffectSet;

/// Combined with the ordinary ambience mixer, never submitted as a separate mix.
#[derive(Resource, Default)]
pub(crate) struct ReactionSounds(pub(crate) Vec<rustimal_logic::ambience::Source>);

struct Visual {
    parts: Vec<(Entity, Handle<N64Material>)>,
    shown: bool,
}

struct Effect {
    kind: i16,
    owner: Entity,
    generation: u32,
    age: f32,
    anchor: Vec3,
    side: f32,
    ending: Option<f32>,
    active: bool,
    visuals: Vec<Visual>,
}

#[derive(Resource, Default)]
struct Effects {
    models: HashMap<&'static str, Model>,
    slots: Vec<Effect>,
    seen: HashMap<Entity, (i16, u32)>,
    scene: Option<(crate::shop::ShopLocation, crate::house::HouseState)>,
}

fn model_names(kind: i16) -> &'static [&'static str] {
    match kind {
        0 => &["ef_shock01_00_modelT"],
        3 => &[
            "ef_warau01_00_modelT",
            "ef_warau01_01_modelT",
            "ef_warau01_02_modelT",
            "ef_warau01_03_modelT",
        ],
        4 => &["ef_ha01_00_modelT"],
        6 => &["ef_gimonhu01_00_modelT"],
        10 => &["ef_think_l_modelT", "ef_think_s_modelT"],
        27 => &["ef_hirameki01_den_modelT", "ef_hirameki01_hikari_modelT"],
        30 => &[
            "ef_ase02_00_modelT",
            "ef_ase02_01_modelT",
            "ef_ase02_02_modelT",
            "ef_ase02_03_modelT",
        ],
        _ => &[],
    }
}

fn lifetime(kind: i16) -> f32 {
    match kind {
        0 => 14.0,
        4 => 56.0,
        6 | 27 => 72.0,
        _ => f32::INFINITY,
    }
}

fn level(kind: i16) -> Option<u8> {
    match kind {
        3 => Some(15),
        10 => Some(0x58),
        30 => Some(0x29),
        _ => None,
    }
}

fn hide(effect: &mut Effect, commands: &mut Commands) {
    effect.active = false;
    for visual in &mut effect.visuals {
        if visual.shown {
            for (entity, _) in &visual.parts {
                commands.entity(*entity).insert(Visibility::Hidden);
            }
            visual.shown = false;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn make_visuals(
    kind: i16,
    disc: &GameDisc,
    models: &mut HashMap<&'static str, Model>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) -> Vec<Visual> {
    model_names(kind)
        .iter()
        .map(|&name| {
            let model = models.entry(name).or_insert_with(|| {
                let Some(list) = disc.catalog.get(name) else {
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
                    (&mut *meshes, &mut *materials, &mut *images),
                )
            });
            let parts = model
                .iter()
                .filter_map(|(mesh, original)| {
                    let material = materials.get(original)?.clone();
                    let material = materials.add(material);
                    let entity = commands
                        .spawn((
                            Name::new(format!("intro reaction {name}")),
                            Mesh3d(mesh.clone()),
                            MeshMaterial3d(material.clone()),
                            Transform::default(),
                            Visibility::Hidden,
                        ))
                        .id();
                    Some((entity, material))
                })
                .collect();
            Visual {
                parts,
                shown: false,
            }
        })
        .collect()
}

struct Draw {
    offset: Vec3,
    scale: Vec3,
    angle: f32,
    prim: Vec4,
    env: Option<Vec4>,
    lod: f32,
}

impl Draw {
    fn new(scale: f32) -> Self {
        Self {
            offset: Vec3::ZERO,
            scale: Vec3::splat(scale),
            angle: 0.0,
            prim: Vec4::ONE,
            env: None,
            lod: 128.0 / 255.0,
        }
    }
}

fn adjust(t: f32, start: f32, end: f32, a: f32, b: f32) -> f32 {
    a + (b - a) * ((t - start) / (end - start)).clamp(0.0, 1.0)
}

/// Native display tables and per-effect transforms. Offsets are billboard-space
/// game units; animation ages are 60-Hz ticks, independent of render frequency.
fn draw(effect: &Effect, variant: usize) -> Option<Draw> {
    let t = effect.age;
    let mut out = Draw::new(1.0);
    match effect.kind {
        0 => {
            const SCALE: [f32; 7] = [0.019, 0.02375, 0.0285, 0.026125, 0.02375, 0.021375, 0.019];
            const COLORS: [[f32; 4]; 7] = [
                [255., 0., 0., 200.],
                [255., 128., 0., 227.],
                [255., 255., 0., 255.],
                [255., 191., 0., 191.],
                [255., 128., 0., 128.],
                [255., 63., 0., 63.],
                [255., 0., 0., 0.],
            ];
            let k = (t * 0.5).min(6.0);
            let i = k as usize;
            let j = (i + 1).min(6);
            out.scale = Vec3::splat(SCALE[i] + (SCALE[j] - SCALE[i]) * k.fract());
            out.prim =
                Vec4::from_array(COLORS[i]).lerp(Vec4::from_array(COLORS[j]), k.fract()) / 255.0;
            out.offset = Vec3::new(0.0, 4.0, 6.0);
        }
        3 => {
            let frame = (t.rem_euclid(24.0) * 0.5) as usize;
            if !(2..10).contains(&frame) || (frame - 2) / 2 != variant {
                return None;
            }
            out.scale = Vec3::splat(0.0045);
            // The model's own Gfx overrides the caller's primitive/environment state.
            out.prim.w = 200.0 / 255.0;
            out.env = Some(Vec4::new(0.0, 0.0, 200.0 / 255.0, 1.0));
            out.lod = 1.0;
        }
        4 => {
            out.scale = Vec3::splat(0.0067);
            out.offset = Vec3::new(effect.side * 16.0, 12.0, 0.0);
            out.angle = -effect.side * std::f32::consts::FRAC_PI_4;
            out.prim.w = adjust(t, 24.0, 56.0, 1.0, 0.0);
        }
        6 => {
            const SCALE: [Vec2; 7] = [
                Vec2::new(0.5, 0.5),
                Vec2::new(0.5, 1.2),
                Vec2::new(0.5, 2.0),
                Vec2::new(1.2, 1.4),
                Vec2::new(2.0, 0.7),
                Vec2::new(1.5, 0.8),
                Vec2::ONE,
            ];
            let k = (t * 0.5).min(6.0);
            let i = k as usize;
            let scale = SCALE[i].lerp(SCALE[(i + 1).min(6)], k.fract()) * 0.008;
            out.scale = scale.extend(0.008);
            out.prim.w = adjust(t, 64.0, 72.0, 1.0, 0.0);
        }
        10 => {
            // eKG's two counter-rotating thought clouds, including its 7-tick exit.
            let scale = if effect.ending.is_some() || t >= 36.0 {
                0.008
            } else {
                adjust(t, 0.0, 7.0, 0.0, 0.008)
            };
            out.scale = Vec3::new(
                scale,
                scale,
                if t < 36.0 && effect.ending.is_none() {
                    1.0
                } else {
                    0.8
                },
            );
            out.offset = Vec3::new(0.0, -6.0, 30.0)
                + Vec3::new(
                    if variant == 0 { -475.0 } else { 475.0 },
                    if variant == 0 { 950.0 } else { 0.0 },
                    0.0,
                ) * out.scale;
            // Retail advances these angles for ten of each 25 updates.
            let cycle = t.floor() as u32;
            let steps = (cycle / 25) * 10 + (cycle % 25).min(10);
            // Native binary-angle increments truncate to 455/-682. Crossing
            // zero after a complete turn resets both counters together.
            let step = (steps % 145) as i32;
            let angle = (step * if variant == 0 { 455 } else { -682 }) as i16;
            out.angle = f32::from(angle) * std::f32::consts::TAU / 65536.0;
            out.prim.w = effect
                .ending
                .map_or(1.0, |age| adjust(age, 0.0, 6.0, 1.0, 0.0));
            out.lod = 1.0;
        }
        27 if variant == 0 => {
            out.scale = Vec3::splat(0.007);
            out.prim.w = adjust(t, 64.0, 72.0, 1.0, 0.0);
            if t < 4.0 {
                let k = (t * 0.5).min(1.0);
                let gb = 100.0 / 255.0 + (155.0 / 255.0) * k;
                out.prim = Vec4::new(k, gb, gb, out.prim.w);
                out.env = Some(Vec4::new(k, k, 0.0, 1.0));
            } else {
                out.prim.z = 100.0 / 255.0;
                out.env = Some(Vec4::new(100.0 / 255.0, 50.0 / 255.0, 0.0, 1.0));
            }
            out.lod = 1.0;
        }
        27 => {
            if t >= 12.0 {
                return None;
            }
            out.scale = Vec3::splat(adjust(t, 0.0, 12.0, 0.014, 0.0175));
            out.prim = Vec4::new(
                1.0,
                1.0,
                100.0 / 255.0,
                if t < 4.0 {
                    t * 50.0 / 255.0
                } else {
                    adjust(t, 4.0, 12.0, 1.0, 0.0)
                },
            );
            out.lod = 1.0;
        }
        30 => {
            if ((t as usize & 12) >> 2) != variant {
                return None;
            }
            out.scale = Vec3::splat(0.006);
            out.offset.z = 30.0;
        }
        _ => return None,
    }
    Some(out)
}

fn paint(
    effect: &mut Effect,
    billboard: Quat,
    commands: &mut Commands,
    materials: &mut Assets<N64Material>,
) {
    for index in 0..effect.visuals.len() {
        let drawing = effect.active.then(|| draw(effect, index)).flatten();
        let visual = &mut effect.visuals[index];
        if visual.shown != drawing.is_some() {
            visual.shown = drawing.is_some();
            for (entity, _) in &visual.parts {
                commands.entity(*entity).insert(if visual.shown {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
            }
        }
        let Some(drawing) = drawing else { continue };
        let transform = Transform {
            translation: effect.anchor + billboard * drawing.offset / UNIT,
            rotation: billboard * Quat::from_rotation_z(drawing.angle),
            scale: drawing.scale / UNIT,
        };
        for (entity, material) in &visual.parts {
            // These unparented meshes render after propagation; keep both transforms current.
            commands
                .entity(*entity)
                .insert((transform, GlobalTransform::from(transform)));
            // ASE2's display lists set their own distinct LOD fractions (128 then 255).
            let lod = if effect.kind == 30 {
                materials
                    .get(material)
                    .map_or(drawing.lod, |m| m.combiner.extra.x)
            } else {
                drawing.lod
            };
            let changed = materials.get(material).is_some_and(|m| {
                m.combiner.prim != drawing.prim
                    || m.combiner.extra.x != lod
                    || drawing.env.is_some_and(|env| m.combiner.env != env)
            });
            if changed {
                if let Some(mut material) = materials.get_mut(material) {
                    material.combiner.prim = drawing.prim;
                    material.combiner.extra.x = lod;
                    if let Some(env) = drawing.env {
                        material.combiner.env = env;
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn update(
    mut commands: Commands,
    time: Res<Time>,
    disc: Option<Res<GameDisc>>,
    actors: Query<(Entity, &DialogueReaction, &GlobalTransform)>,
    rigs: Query<(&CharacterRig, &ChildOf)>,
    globals: Query<&GlobalTransform>,
    cameras: Query<&GlobalTransform, With<crate::player::WorldCamera>>,
    shop: Res<State<crate::shop::ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut loads: MessageReader<crate::sound::ResetAudio>,
    mut effects: ResMut<Effects>,
    mut sounds: ResMut<ReactionSounds>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let scene = (*shop.get(), *house.get());
    let loaded = loads.read().count() != 0;
    if loaded || effects.scene.is_some_and(|previous| previous != scene) {
        for effect in &mut effects.slots {
            hide(effect, &mut commands);
        }
        effects.seen.clear();
        sounds.0.clear();
    }
    effects.scene = Some(scene);
    if loaded {
        return;
    }
    if actors.is_empty() && !effects.slots.iter().any(|effect| effect.active) {
        return;
    }
    let Some(disc) = disc else { return };
    let Ok(camera) = cameras.single() else { return };
    let billboard = camera.rotation();
    let frames = time.delta_secs() * 60.0;
    sounds.0.clear();

    for effect in &mut effects.slots {
        if !effect.active {
            continue;
        }
        let continuing = actors
            .get(effect.owner)
            .ok()
            .and_then(|(_, reaction, _)| reaction.effect)
            == Some((effect.kind, effect.generation));
        effect.age += frames;
        if level(effect.kind).is_some() && !continuing {
            if effect.kind == 10 {
                *effect.ending.get_or_insert(0.0) += frames;
            } else {
                hide(effect, &mut commands);
            }
        }
        if effect.age >= lifetime(effect.kind)
            || effect.ending.is_some_and(|age| age >= 7.0)
            || globals.get(effect.owner).is_err()
        {
            hide(effect, &mut commands);
        }
    }
    effects.seen.retain(|owner, _| {
        actors
            .get(*owner)
            .ok()
            .is_some_and(|(_, r, _)| r.effect.is_some())
    });
    for (owner, reaction, actor) in &actors {
        let Some((kind, generation)) = reaction.effect else {
            continue;
        };
        if model_names(kind).is_empty() || effects.seen.get(&owner) == Some(&(kind, generation)) {
            continue;
        }
        effects.seen.insert(owner, (kind, generation));
        let Some(anchor) = rigs
            .iter()
            .find(|(_, parent)| parent.parent() == owner)
            .and_then(|(rig, _)| rig.feel_joint())
            .and_then(|joint| globals.get(joint).ok())
        else {
            effects.seen.remove(&owner);
            continue;
        };
        let position = anchor.translation();
        let yaw = actor.rotation().to_euler(EulerRot::YXZ).0;
        let camera_yaw = billboard.to_euler(EulerRot::YXZ).0;
        let side = if (yaw - camera_yaw - std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            >= std::f32::consts::PI
        {
            -1.0
        } else {
            1.0
        };
        let position = position
            + match kind {
                // eGM_init deliberately applies only rotated x and y, not z.
                6 => Vec3::new(yaw.sin() * 7.0, 15.0, 0.0) / UNIT,
                10 => Vec3::Y * 21.0 / UNIT,
                27 => Vec3::Y * 24.0 / UNIT,
                _ => Vec3::ZERO,
            };
        let index = if let Some(index) = effects
            .slots
            .iter()
            .position(|e| !e.active && e.kind == kind)
        {
            index
        } else {
            if effects.slots.len() >= MAX_EFFECTS {
                continue;
            }
            let visuals = make_visuals(
                kind,
                &disc,
                &mut effects.models,
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut images,
            );
            let index = effects.slots.len();
            effects.slots.push(Effect {
                kind,
                owner,
                generation,
                age: 0.0,
                anchor: position,
                side,
                ending: None,
                active: false,
                visuals,
            });
            index
        };
        let effect = &mut effects.slots[index];
        effect.owner = owner;
        effect.generation = generation;
        effect.age = 0.0;
        effect.anchor = position;
        effect.side = side;
        effect.ending = None;
        effect.active = true;
        if let Some(sound) = match kind {
            0 => Some(0x2D),
            4 => Some(0x2C),
            6 => Some(0x2F),
            27 => Some(0x2E),
            _ => None,
        } {
            crate::sound::se_at(sound, position);
        }
    }
    for (index, effect) in effects.slots.iter_mut().enumerate() {
        if !effect.active {
            continue;
        }
        if effect.ending.is_none() {
            if let Some(id) = level(effect.kind) {
                sounds.0.push(rustimal_logic::ambience::Source {
                    key: u64::MAX - 1000 - index as u64,
                    id,
                    position: (effect.anchor * UNIT).to_array(),
                });
            }
        }
        paint(effect, billboard, &mut commands, &mut materials);
    }
}

pub(crate) fn register(app: &mut App) {
    app.init_resource::<Effects>()
        .init_resource::<ReactionSounds>()
        .add_systems(
            PostUpdate,
            update
                .in_set(EffectSet)
                .after(TransformSystems::Propagate)
                .before(VisibilitySystems::VisibilityPropagate)
                .before(VisibilitySystems::CalculateBounds)
                .before(VisibilitySystems::CheckVisibility)
                .run_if(in_state(crate::states::PauseState::Running)),
        );
}
