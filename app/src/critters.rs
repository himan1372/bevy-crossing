//! Fish and bugs drawn from the disc.
//!
//! Fish in the water are shadows (`ac_gyoei_draw.c_inc`,
//! `aGYO_actor_draw_gyoei`): `act_gyoei02_00_modelT` with two of the four
//! `act_gyoei02_*_int_i4` textures in segments 8/9, blended by the prim
//! LOD. `fwork0` counts 19 -> 0 at half a step per frame and picks the
//! texture pair (`aGYO_2tile_texture_idx`) and blend (`aGYO_prim_f`): the
//! shadow's wriggle. Alpha 120 (`aGYO_actor_ct` clip), scale
//! `aGYO_shadow_scale[size] * 0.02` with x at 0.4 (`aGTT_actor_init`),
//! turned to heading + 180 deg.
//!
//! A landed fish hangs from the hand as its own model
//! (`aGYO_actor_draw_fish`, `aUKI_catch`): `aGYO_displayList[type]`'s
//! frames at scale 0.01, billboarded, lowered by `aGYO_hosei_y`, swung
//! 13 degrees by sin of a counter, flapping to the type's fast or slow
//! pattern (`aGYO_anime_ptn`; display list = pattern x 0.5), then shrunk
//! away over 30 frames as the rod is put away (`gyo_scale`).
//!
//! Bugs (`ac_insect_draw.c_inc`) draw two of their four
//! `aINS_displayList` lists, frame `_1E0` (0/1) picking 0+1 or 2+3, at
//! scale 0.01, 2 units up, env colour white.

use bevy::prelude::*;
use disc::{RomSource, Target};

use crate::disc_gfx::Segments;
use crate::disc_town::{build_model_full, DrawColors};
use crate::ecology::{Bug, Fish};
use crate::game_disc::GameDisc;
use crate::n64_material::N64Material;
use crate::states::AppState;

type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;

/// `ANIME_1_TXT_SEG` / `ANIME_2_TXT_SEG`.
const SEG_ANIME_1: u8 = 8;
const SEG_ANIME_2: u8 = 9;
/// `ctrl->alpha` of every fish but the whale.
const SHADOW_ALPHA: u8 = 120;
/// Shadow steps (`fwork0` 0..=19).
const SHADOW_STEPS: usize = 20;
/// Game frames per second (`dt_num_60fps_frames`).
const FPS: f32 = 60.0;
/// Bug wing-beat frames per second: `_1E0` flips every few frames in the
/// insect move code; one flip each 3 frames reads like the game.
const BUG_FLAPS_PER_SEC: f32 = 10.0;

/// The disc models for fish and bugs.
#[derive(Resource)]
struct CritterModels {
    /// One shadow per `fwork0` step.
    shadow: Vec<Model>,
    /// `aGYO_shadow_scale` by `gyoei_type[type].size`, per fish type.
    shadow_scale: Vec<f32>,
    /// Per insect type: the two frames.
    bugs: Vec<Option<[Model; 2]>>,
    /// Per fish type: frames a, b (c is never reached: frame x 0.5).
    fish: Vec<Option<[Model; 2]>>,
    /// Per fish type: `aGYO_anime_ptn` is slow (2), else fast.
    fish_slow: Vec<bool>,
}

/// A fish was landed; `at` is the hand holding it (scene units).
#[derive(bevy::ecs::message::Message, Clone, Copy)]
pub struct FishCaught {
    pub species: rustimal_logic::species::FishType,
    pub at: Vec3,
}

/// A landed fish being shown: seconds left, pattern tick.
#[derive(Component)]
struct HeldFish {
    species: usize,
    left: f32,
    tick: f32,
}

/// Seconds a landed fish is held up, then frames it shrinks over.
const HOLD_SECS: f32 = 2.5;
const PUTAWAY_TICKS: f32 = 30.0;
/// `aGYR_frame_ptn1` / `ptn2` (fast, slow), one step per 60 Hz tick.
const FISH_FAST: [usize; 8] = [1, 1, 0, 0, 2, 2, 0, 0];
const FISH_SLOW: [usize; 16] = [1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2, 2, 0, 0, 0, 0];
/// `aGYO_hosei_y` (game units), per fish type.
const HOSEI_Y: [f32; 45] = [
    0.0, 0.0, 2.2, 2.2, 2.2, 0.0, 1.5, 2.2, 0.0, 2.2, 2.2, 2.2, 1.5, 0.0, 0.0, //
    0.0, 0.0, 0.0, 0.0, 2.2, 1.5, 2.2, 1.5, 0.0, 0.0, 2.2, 2.2, 0.0, 0.0, 0.0, //
    0.0, 2.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];

/// Shadow wriggle state (`fwork0`).
#[derive(Component)]
pub struct ShadowStep(f32);

/// Wing-beat frame of a bug.
#[derive(Component)]
pub struct BugFrame(f32, usize);

/// The model parts under a dressed critter.
#[derive(Component)]
struct Parts(Vec<Entity>);

fn rel_bytes(disc: &GameDisc, at: u32, len: u32) -> Option<&[u8]> {
    disc.rom.slice(RomSource::Rel, at, len)
}

fn rel_ptr(disc: &GameDisc, at: u32) -> Option<u32> {
    match disc.rom.pointer_at(RomSource::Rel, at) {
        Some(Target::File(RomSource::Rel, p)) => Some(p),
        _ => None,
    }
}

fn load_models(
    disc: &GameDisc,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) -> Option<CritterModels> {
    let cat = &disc.catalog;
    // Shadow: textures, pairs, blends.
    let textures: Vec<std::sync::Arc<[u8]>> = {
        let table = cat.get("aGYO_texture_table")?;
        (0..4)
            .map(|i| {
                let p = rel_ptr(disc, table.offset + i * 4)?;
                Some(rel_bytes(disc, p, 0x100)?.into())
            })
            .collect::<Option<_>>()?
    };
    let pairs = rel_bytes(disc, cat.get("aGYO_2tile_texture_idx")?.offset, 40)?;
    let blend = rel_bytes(disc, cat.get("aGYO_prim_f")?.offset, 20)?;
    let shadow_list = cat.get("act_gyoei02_00_modelT")?.offset;
    let mut shadow = Vec::with_capacity(SHADOW_STEPS);
    for t in 0..SHADOW_STEPS {
        let mut segs = Segments::default();
        segs.set_segment(SEG_ANIME_1, textures[pairs[t * 2] as usize & 3].clone());
        segs.set_segment(SEG_ANIME_2, textures[pairs[t * 2 + 1] as usize & 3].clone());
        let colors = DrawColors {
            prim: [0, 0, 0, SHADOW_ALPHA],
            prim_lod: blend[t],
            env: [0; 4],
        };
        shadow.push(build_model_full(
            disc,
            &[shadow_list],
            None,
            colors,
            true,
            (&segs, &[]),
            (&mut *meshes, &mut *materials, &mut *images),
        ));
    }
    // Shadow scale per fish type.
    let scales = rel_bytes(disc, cat.get("aGYO_shadow_scale")?.offset, 32)?;
    let types = cat.get("gyoei_type")?;
    let shadow_scale = (0..types.size / 8)
        .map(|i| {
            let e = rel_bytes(disc, types.offset + i * 8, 2).unwrap_or(&[0, 0]);
            let size = (i16::from_be_bytes([e[0], e[1]]).clamp(0, 7) as usize) * 4;
            f32::from_be_bytes(scales[size..size + 4].try_into().unwrap_or([0; 4]))
        })
        .collect();
    // Bugs.
    let table = cat.get("aINS_displayList")?;
    let white = DrawColors {
        prim: [255; 4],
        prim_lod: 255,
        env: [255; 4],
    };
    let bugs = (0..table.size / 4)
        .map(|i| {
            let lists = rel_ptr(disc, table.offset + i * 4)?;
            let dl: Vec<u32> = (0..4)
                .map(|k| rel_ptr(disc, lists + k * 4))
                .collect::<Option<_>>()?;
            let mut frame = |a: u32, b: u32| {
                build_model_full(
                    disc,
                    &[a, b],
                    None,
                    white,
                    true,
                    (&Segments::default(), &[]),
                    (&mut *meshes, &mut *materials, &mut *images),
                )
            };
            Some([frame(dl[0], dl[1]), frame(dl[2], dl[3])])
        })
        .collect();
    // Landed fish: aGYO_displayList -> aGYO_DL_c { a, b, c }.
    let fish = cat
        .get("aGYO_displayList")
        .map(|table| {
            (0..table.size / 4)
                .map(|i| {
                    let dl = rel_ptr(disc, table.offset + i * 4)?;
                    let mut frame = |k: u32| {
                        let list = rel_ptr(disc, dl + k * 4)?;
                        Some(build_model_full(
                            disc,
                            &[list],
                            None,
                            white,
                            true,
                            (&Segments::default(), &[]),
                            (&mut *meshes, &mut *materials, &mut *images),
                        ))
                    };
                    Some([frame(0)?, frame(1)?])
                })
                .collect()
        })
        .unwrap_or_default();
    let fish_slow = cat
        .get("aGYO_anime_ptn")
        .and_then(|t| rel_bytes(disc, t.offset, t.size))
        .map(|b| {
            b.as_chunks::<4>()
                .0
                .iter()
                .map(|w| *w == [0, 0, 0, 2])
                .collect()
        })
        .unwrap_or_default();
    Some(CritterModels {
        shadow,
        shadow_scale,
        bugs,
        fish,
        fish_slow,
    })
}

fn setup_models(
    mut commands: Commands,
    disc: Res<GameDisc>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    match load_models(&disc, &mut meshes, &mut materials, &mut images) {
        Some(m) => commands.insert_resource(m),
        None => warn!("fish/bug models not found on the disc; keeping stand-ins"),
    }
}

fn spawn_parts(commands: &mut Commands, root: Entity, model: &Model) -> Vec<Entity> {
    model
        .iter()
        .map(|(mesh, material)| {
            commands
                .spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    ChildOf(root),
                ))
                .id()
        })
        .collect()
}

/// Swaps a stand-in critter's sphere for its disc model.
fn dress_critters(
    mut commands: Commands,
    models: Res<CritterModels>,
    fish: Query<(Entity, &Fish), Without<Parts>>,
    bugs: Query<(Entity, &Bug), Without<Parts>>,
) {
    for (e, f) in &fish {
        let scale = models
            .shadow_scale
            .get(f.species as usize)
            .copied()
            .unwrap_or(0.5)
            * 2.0;
        let parts = spawn_parts(&mut commands, e, &models.shadow[SHADOW_STEPS - 1]);
        commands
            .entity(e)
            .remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>)>()
            .insert((
                Parts(parts),
                Name::new(format!("fish {:?}", f.species)),
                ShadowStep(19.0),
                FishScale(Vec3::new(scale * 0.4, scale, scale)),
            ));
    }
    for (e, b) in &bugs {
        let Some(Some(frames)) = models.bugs.get(b.insect as usize) else {
            continue;
        };
        let parts = spawn_parts(&mut commands, e, &frames[0]);
        commands
            .entity(e)
            .remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>)>()
            .insert((
                Parts(parts),
                Name::new(format!("bug {:?}", b.insect)),
                BugFrame(0.0, 0),
            ));
    }
}

/// The shadow's size (`scale.x * 0.4`, `scale.y`, `scale.z`).
#[derive(Component)]
struct FishScale(Vec3);

/// Shadow wriggle and heading; bug wing beats.
fn animate_critters(
    time: Res<Time>,
    models: Res<CritterModels>,
    mut fish: Query<(&Fish, &mut ShadowStep, &FishScale, &Parts, &mut Transform)>,
    mut bugs: Query<(&Bug, &mut BugFrame, &Parts, &mut Transform), Without<Fish>>,
    mut part_q: Query<(&mut Mesh3d, &mut MeshMaterial3d<N64Material>)>,
) {
    let dt = time.delta_secs() * FPS;
    for (f, mut step, scale, parts, mut t) in &mut fish {
        // aGYO_position_move
        if step.0 <= 0.0 {
            step.0 += 19.0;
        } else {
            step.0 -= 0.5 * dt;
        }
        let i = (step.0.max(0.0) as usize).min(SHADOW_STEPS - 1);
        set_parts(&mut part_q, parts, &models.shadow[i]);
        let heading = f.dir.x.atan2(f.dir.z);
        t.rotation = Quat::from_rotation_y(heading + std::f32::consts::PI);
        t.scale = scale.0;
    }
    for (b, mut frame, parts, mut t) in &mut bugs {
        let Some(Some(frames)) = models.bugs.get(b.insect as usize) else {
            continue;
        };
        frame.0 += time.delta_secs() * BUG_FLAPS_PER_SEC;
        let k = b.ai.as_ref().map_or(frame.0 as usize % 2, |ai| ai.frame());
        if k != frame.1 {
            frame.1 = k;
            set_parts(&mut part_q, parts, &frames[k]);
        }
        if let Some(ai) = &b.ai {
            // aINS_actor_draw_sub: RotateX(pitch), RotateY(heading), RotateZ
            // (a cicada stood up on its trunk, a bagworm twisting).
            let r = |a: i16| a as f32 * std::f32::consts::PI / 32768.0;
            t.rotation = Quat::from_rotation_x(r(ai.pitch))
                * Quat::from_rotation_y(r(ai.facing))
                * Quat::from_rotation_z(r(ai.roll));
            // Pondskater stretch, bagworm squash, mole cricket pulse.
            t.scale = Vec3::from(ai.scale_xyz());
            continue;
        }
        let to = b.target - t.translation;
        if to.xz().length_squared() > 1e-4 {
            t.rotation = Quat::from_rotation_y(to.x.atan2(to.z));
        }
    }
}

fn set_parts(
    part_q: &mut Query<(&mut Mesh3d, &mut MeshMaterial3d<N64Material>)>,
    parts: &Parts,
    model: &Model,
) {
    for (e, (mesh, material)) in parts.0.iter().zip(model) {
        if let Ok((mut m, mut mat)) = part_q.get_mut(*e) {
            if m.0 != *mesh {
                m.0 = mesh.clone();
            }
            if mat.0 != *material {
                mat.0 = material.clone();
            }
        }
    }
}

/// Holds a landed fish up over the player.
fn show_caught(
    mut commands: Commands,
    mut caught: bevy::ecs::message::MessageReader<FishCaught>,
    models: Res<CritterModels>,
) {
    for c in caught.read() {
        let species = c.species as usize;
        let Some(Some(frames)) = models.fish.get(species) else {
            continue;
        };
        let root = commands
            .spawn((
                Name::new("landed fish"),
                HeldFish {
                    species,
                    left: HOLD_SECS,
                    tick: 0.0,
                },
                Transform::from_translation(
                    c.at - Vec3::Y * HOSEI_Y.get(species).copied().unwrap_or(0.0) / 40.0,
                ),
                Visibility::default(),
            ))
            .id();
        let parts = spawn_parts(&mut commands, root, &frames[0]);
        commands.entity(root).insert(Parts(parts));
    }
}

/// Faces the camera (`billboard_matrix`), wriggles, then goes.
fn animate_held(
    mut commands: Commands,
    time: Res<Time>,
    models: Res<CritterModels>,
    camera: Query<
        &GlobalTransform,
        (
            With<Camera3d>,
            Without<crate::msg_window::OverlayCamera>,
            Without<HeldFish>,
        ),
    >,
    mut held: Query<(Entity, &mut HeldFish, &Parts, &mut Transform)>,
    mut part_q: Query<(&mut Mesh3d, &mut MeshMaterial3d<N64Material>)>,
) {
    let cam = camera.iter().next().map(|c| c.translation());
    for (e, mut h, parts, mut t) in &mut held {
        h.left -= time.delta_secs();
        if h.left <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        h.tick += time.delta_secs() * FPS;
        // Swing: 13 degrees x sin of a counter stepping 0xE38 (fast) or
        // 0xAAA (slow) a tick.
        let slow = models.fish_slow.get(h.species).copied().unwrap_or(false);
        let step = if slow { 0xAAA } else { 0xE38 } as f32;
        let swing =
            (13.0f32.to_radians()) * (h.tick * step * std::f32::consts::TAU / 65536.0).sin();
        if let Some(cam) = cam {
            let to = cam - t.translation;
            t.rotation = Quat::from_rotation_y(to.x.atan2(to.z)) * Quat::from_rotation_y(swing);
        }
        let shrink = (h.left * FPS / PUTAWAY_TICKS).min(1.0);
        t.scale = Vec3::splat(shrink);
        let tick = h.tick as usize;
        let frame = if slow {
            FISH_SLOW[tick % FISH_SLOW.len()]
        } else {
            FISH_FAST[tick % FISH_FAST.len()]
        } / 2;
        if let Some(Some(frames)) = models.fish.get(h.species) {
            set_parts(&mut part_q, parts, &frames[frame]);
        }
    }
}

pub struct CrittersPlugin;

impl Plugin for CrittersPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FishCaught>()
            .add_systems(
                OnEnter(AppState::InGame),
                setup_models.run_if(resource_exists::<GameDisc>),
            )
            .add_systems(
                Update,
                (dress_critters, animate_critters, show_caught, animate_held)
                    .chain()
                    .run_if(in_state(AppState::InGame).and_then(resource_exists::<CritterModels>)),
            );
    }
}
