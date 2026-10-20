//! m_player_item_{fan,windmill}, m_player_main_swing_fan and original tool banks.
use crate::n64_material::N64Material;
use crate::{
    character::CharacterRig,
    ecology::EquippedTool,
    game_disc::GameDisc,
    input::{Action, Actions, InputFocus, InteractSet},
    skeleton::{Animation, Skeleton},
};
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::prelude::*;

struct Models {
    fan: Animation,
    wave: Animation,
    pin: Animation,
    spin: Animation,
    mask: Vec<u8>,
}
#[derive(Resource, Default)]
pub(crate) struct ToyAction {
    models: Option<Models>,
    item: Option<u16>,
    frame: f32,
    waving: bool,
    shown: bool,
    pub(crate) pin_level: f32,
}
impl ToyAction {
    pub(crate) fn phase(&self) -> Option<(u16, bool)> {
        self.item.filter(|_| self.shown).map(|i| (i, self.waving))
    }
    pub(crate) fn blocks_movement(&self) -> bool {
        self.shown && self.waving && self.frame < 8.0
    }
    pub(crate) fn pose(&self) -> Option<(&Animation, f32, &[u8])> {
        if !self.shown {
            return None;
        }
        let models = self.models.as_ref()?;
        let item = self.item?;
        Some((
            if item < 0x2254 {
                &models.pin
            } else if self.waving {
                &models.wave
            } else {
                &models.fan
            },
            if self.waving { self.frame } else { 1.0 },
            &models.mask,
        ))
    }
}
#[derive(Component)]
struct HeldToy {
    item: u16,
    skeleton: Option<Skeleton>,
    joints: Vec<Entity>,
    frame: f32,
    speed: f32,
    last: Option<Vec3>,
}

#[allow(clippy::too_many_arguments)] // One source of truth for held item, ownership, input and reload cleanup.
fn advance(
    time: Res<Time>,
    actions: Res<Actions>,
    focus: Res<InputFocus>,
    mut tool: ResMut<EquippedTool>,
    inventory: Res<crate::interaction::PlayerInventory>,
    mut state: ResMut<ToyAction>,
    mut reset: MessageReader<crate::sound::ResetAudio>,
    player: Query<&Transform, With<crate::player::Player>>,
) {
    if reset.read().count() > 0 && matches!(*tool, EquippedTool::Fan(_) | EquippedTool::Pinwheel(_))
    {
        *tool = EquippedTool::None;
    }
    let item = match *tool {
        EquippedTool::Fan(i) | EquippedTool::Pinwheel(i) => Some(i),
        _ => None,
    };
    if item.is_some_and(|item| {
        !inventory
            .pockets
            .pockets
            .iter()
            .enumerate()
            .any(|(i, &v)| v == item && inventory.pockets.item_cond(i) == 0)
    }) {
        *tool = EquippedTool::None;
        state.item = None;
    } else if state.item != item {
        state.item = item;
        state.waving = false;
        state.frame = 1.0;
    }
    state.shown = state.item.is_some() && matches!(*focus, InputFocus::World | InputFocus::Paused);
    if !state.shown {
        state.waving = false;
        return;
    }
    if *focus == InputFocus::Paused {
        return;
    }
    if !matches!(*tool, EquippedTool::Fan(_)) {
        state.waving = false;
        return;
    }
    let Some(end) = state.models.as_ref().map(|m| m.wave.frames as f32) else {
        return;
    };
    if !state.waving && actions.pressed(Action::Interact) {
        state.waving = true;
        state.frame = 1.0;
    }
    if state.waving {
        let old = state.frame;
        state.frame += time.delta_secs() * 30.0;
        if old < 1.5 && state.frame >= 1.5 {
            if let Ok(player) = player.single() {
                crate::sound::se_at(0x167, player.translation);
            }
        }
        if state.frame >= 8.0 && actions.move_dir() != Vec3::ZERO {
            state.waving = false;
        } else if state.frame >= end {
            if actions.pressed(Action::Interact) {
                state.frame = 1.0;
            } else {
                state.waving = false;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)] // Same original Gfx/cKF/skin asset stores used by the existing net.
fn spawn(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    mut state: ResMut<ToyAction>,
    rigs: Query<(&ChildOf, &CharacterRig), Added<CharacterRig>>,
    players: Query<(), With<crate::player::Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
    mut binds: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    let Some(disc) = disc else { return };
    for (owner, rig) in &rigs {
        if !players.contains(owner.parent()) {
            continue;
        }
        let Some(hand) = rig.right_hand() else {
            continue;
        };
        let read = |name: &str, n| {
            disc.catalog
                .get(name)
                .and_then(|s| Animation::read(&disc.rom, s.offset, n))
        };
        let (Some(fan), Some(wave), Some(pin), Some(spin)) = (
            read("cKF_ba_r_ply_1_utiwa_wait1", rig.joint_count()),
            read("cKF_ba_r_ply_1_utiwa_d1", rig.joint_count()),
            read("cKF_ba_r_ply_1_kaza1", rig.joint_count()),
            read("cKF_ba_r_tol_kaza1_wait", 3),
        ) else {
            warn!("original toy clips unavailable");
            continue;
        };
        let width = rig.joint_count() + 1;
        let mask = disc
            .catalog
            .get("BOY_part_data")
            .and_then(|s| {
                disc.rom.slice(
                    disc::RomSource::Rel,
                    s.offset + 4 * width as u32,
                    width as u32,
                )
            })
            .map(|b| b[1..].to_vec())
            .unwrap_or_default();
        for item in 0x224C..=0x225B {
            let root = commands
                .spawn((
                    Name::new(format!("held toy {item:04x}")),
                    Transform::default(),
                    Visibility::Hidden,
                    ChildOf(hand),
                ))
                .id();
            let (skeleton, joints) = if item < 0x2254 {
                let name = format!("cKF_bs_r_tol_kaza{}", item - 0x224C + 1);
                let Some(sk) = disc
                    .catalog
                    .get(&name)
                    .and_then(|s| Skeleton::read(&disc.rom, s.offset))
                else {
                    commands.entity(root).despawn();
                    continue;
                };
                let mut joints = Vec::new();
                for (i, local) in spin.pose(1.0).local_transforms(&sk).into_iter().enumerate() {
                    let parent = sk.parents[i].map_or(root, |p| joints[p]);
                    joints.push(
                        commands
                            .spawn((local, Visibility::default(), ChildOf(parent)))
                            .id(),
                    );
                }
                let inverse = binds.add(SkinnedMeshInverseBindposes::from(vec![
                    Mat4::IDENTITY;
                    joints.len()
                ]));
                let slots = sk.matrix_joints();
                for &j in &slots {
                    let Some(list) = sk.joints[j].model else {
                        continue;
                    };
                    let sink = crate::disc_gfx::run_display_lists(&disc.rom, &[list]);
                    for part in &sink.parts {
                        let texture = part.key.tex.as_ref().and_then(|key| {
                            crate::disc_gfx::decode_texture(
                                &disc.rom,
                                key,
                                None,
                                &crate::disc_gfx::Segments::default(),
                            )
                            .map(|rgba| images.add(crate::n64_material::texture_image(rgba, key)))
                        });
                        let material = materials.add(N64Material {
                            combiner: crate::n64_material::CombinerUniform::from_key(
                                &part.key, false, false,
                            ),
                            texture,
                            texture1: None,
                            env: default(),
                            blend: false,
                        });
                        commands.spawn((
                            Mesh3d(
                                meshes.add(crate::character::skinned_part_mesh(part, j, &slots)),
                            ),
                            MeshMaterial3d(material),
                            SkinnedMesh {
                                inverse_bindposes: inverse.clone(),
                                joints: joints.clone(),
                            },
                            Transform::default(),
                            ChildOf(root),
                            bevy::camera::visibility::NoFrustumCulling,
                        ));
                    }
                }
                (Some(sk), joints)
            } else {
                if let Some(list) = disc
                    .catalog
                    .get(&format!("tol_fan{}_model", item - 0x2254 + 1))
                {
                    let sink = crate::disc_gfx::run_display_lists(&disc.rom, &[list.offset]);
                    for (mesh, material) in crate::disc_town::sink_meshes_scaled(
                        &disc,
                        &sink,
                        None,
                        false,
                        &crate::disc_gfx::Segments::default(),
                        1.0,
                        (&mut meshes, &mut materials, &mut images),
                    ) {
                        commands.spawn((
                            Mesh3d(mesh),
                            MeshMaterial3d(material),
                            Transform::default(),
                            ChildOf(root),
                        ));
                    }
                }
                (None, Vec::new())
            };
            commands.entity(root).insert(HeldToy {
                item,
                skeleton,
                joints,
                frame: 1.0,
                speed: 0.0,
                last: None,
            });
        }
        state.models = Some(Models {
            fan,
            wave,
            pin,
            spin,
            mask,
        });
    }
}

fn animate(
    time: Res<Time>,
    mut state: ResMut<ToyAction>,
    focus: Res<InputFocus>,
    mut toys: Query<(&mut HeldToy, &mut Visibility, &GlobalTransform)>,
    mut transforms: Query<&mut Transform>,
    globals: Query<&GlobalTransform, Without<HeldToy>>,
) {
    state.pin_level = 0.0;
    for (mut toy, mut visibility, global) in &mut toys {
        let shown = state.shown && state.item == Some(toy.item);
        *visibility = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if !shown {
            toy.last = None;
            toy.speed = 0.0;
            continue;
        }
        let Some(models) = &state.models else {
            continue;
        };
        let Some(sk) = toy.skeleton.as_ref() else {
            continue;
        };
        let ticks = time.delta_secs() * 60.0;
        let fan = toy
            .joints
            .get(2)
            .and_then(|e| globals.get(*e).ok())
            .unwrap_or(global);
        let position = fan.translation();
        let delta = toy.last.map_or(Vec3::ZERO, |p| position - p);
        let forward = fan.affine().transform_vector3(Vec3::Z).normalize_or_zero();
        let target = if ticks > 0.0 && delta.length() < 2.0 {
            8.0 * delta.dot(forward) * 40.0 / ticks
        } else {
            0.0
        };
        if *focus != InputFocus::Paused && ticks > 0.0 {
            let rate = 0.005 * target.abs();
            let change = (target - toy.speed)
                * (1.0 - (1.0 - rate.min(0.99) - 0.02).max(0.0).sqrt()).mul_add(ticks, 0.0);
            let limit = (0.3 + 0.03 * target.abs()) * 0.5 * ticks;
            let minimum = (0.1 + 0.005 * target.abs()) * 0.5 * ticks;
            let step = change
                .abs()
                .clamp(minimum, limit)
                .min((target - toy.speed).abs())
                .copysign(target - toy.speed);
            let speed = toy.speed + step;
            let frame = 1.0
                + (toy.frame - 1.0 + speed * ticks)
                    .rem_euclid((models.spin.frames as f32 - 1.0).max(1.0));
            let locals = models.spin.pose(frame).local_transforms(sk);
            for (&joint, local) in toy.joints.iter().zip(locals) {
                if let Ok(mut transform) = transforms.get_mut(joint) {
                    *transform = local;
                }
            }
            toy.frame = frame;
            toy.speed = speed;
        }
        toy.last = Some(position);
        if *focus != InputFocus::Paused {
            state.pin_level = (toy.speed.abs() / 44.0).min(1.0);
        }
    }
}

pub(crate) struct PlayerToysPlugin;
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ToyVisualSet;
impl Plugin for PlayerToysPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToyAction>()
            .add_systems(
                Update,
                advance
                    .after(InteractSet::Tools)
                    .after(crate::save::LoadSet)
                    .before(crate::locomotion::MovementSet)
                    .before(crate::character::CharacterAnimationSet)
                    .in_set(crate::states::GameplaySet),
            )
            .add_systems(
                Update,
                spawn
                    .after(crate::character::CharacterAnimationSet)
                    .run_if(in_state(crate::states::AppState::InGame)),
            )
            .add_systems(
                PostUpdate,
                animate
                    .in_set(ToyVisualSet)
                    .after(bevy::transform::TransformSystems::Propagate)
                    .run_if(in_state(crate::states::AppState::InGame)),
            );
    }
}
