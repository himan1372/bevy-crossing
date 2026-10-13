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
}

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
    Some(CritterModels {
        shadow,
        shadow_scale,
        bugs,
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
        let k = frame.0 as usize % 2;
        if k != frame.1 {
            frame.1 = k;
            set_parts(&mut part_q, parts, &frames[k]);
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

pub struct CrittersPlugin;

impl Plugin for CrittersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            setup_models.run_if(resource_exists::<GameDisc>),
        )
        .add_systems(
            Update,
            (dress_critters, animate_critters)
                .chain()
                .run_if(in_state(AppState::InGame).and_then(resource_exists::<CritterModels>)),
        );
    }
}
