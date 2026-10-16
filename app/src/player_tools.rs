//! First normal-net action slice, from m_player_main_{swing,stop}_net and
//! m_player_item_net. Runtime cKF rig, attached to right-hand joint 20.
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::prelude::*;

use crate::character::CharacterRig;
use crate::disc_gfx::{decode_texture, run_display_lists, Segments};
use crate::ecology::EquippedTool;
use crate::game_disc::GameDisc;
use crate::n64_material::{texture_image, CombinerUniform, N64Material};
use crate::skeleton::{Animation, Skeleton};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Idle,
    Swing,
    Recover,
}

#[derive(Resource, Default)]
pub struct NetAction {
    phase: Phase,
    frame: f32,
    contact: bool,
    pub player_frame: f32,
    pub held: bool,
}

impl NetAction {
    pub fn busy(&self) -> bool {
        self.phase != Phase::Idle
    }
    pub fn begin(&mut self) -> bool {
        if self.busy() {
            return false;
        }
        self.phase = Phase::Swing;
        self.frame = 1.0;
        self.player_frame = 1.0;
        self.contact = false;
        debug!("net action: swing");
        true
    }
    pub fn take_contact(&mut self) -> bool {
        std::mem::take(&mut self.contact)
    }
    pub fn cancel(&mut self) {
        if self.busy() {
            debug!("net action: cancelled");
        }
        *self = Self::default();
    }
}

#[derive(Resource)]
pub(crate) struct NetModels {
    skeleton: Skeleton,
    idle: Animation,
    swing: Animation,
    recovery: Animation,
    pub player_swing: Animation,
    pub player_hold: Animation,
    pub hold_mask: Vec<u8>,
}

#[derive(Component)]
struct HeldNet {
    joints: Vec<Entity>,
    flex: Entity,
    angle: [i16; 3],
    idle_frame: f32,
    last_yaw: f32,
    phase: Phase,
    morph: f32,
}

/// Source speeds are 0.5 frames per 60-Hz tick. STOP_NET holds the player
/// frame and returns to wait only when SWING_WAIT (item index11) is ready.
pub(crate) fn advance_net(
    time: Res<Time>,
    tool: Res<EquippedTool>,
    focus: Res<crate::input::InputFocus>,
    house: Res<State<crate::house::HouseState>>,
    shop: Res<State<crate::shop::ShopLocation>>,
    models: Option<Res<NetModels>>,
    mut action: ResMut<NetAction>,
) {
    if *tool != EquippedTool::Net
        || *house != crate::house::HouseState::Town
        || *shop != crate::shop::ShopLocation::Town
        || !matches!(
            *focus,
            crate::input::InputFocus::World | crate::input::InputFocus::Paused
        )
    {
        action.cancel();
        return;
    }
    action.held = true;
    let (swing_end, stop_end) = models.as_ref().map_or((10.0, 21.0), |m| {
        (m.player_swing.frames as f32, m.recovery.frames as f32)
    });
    let delta = time.delta_secs() * 30.0;
    match action.phase {
        Phase::Swing => {
            let previous = action.frame;
            action.frame += delta;
            action.player_frame = action.frame.min(swing_end);
            if previous <= 6.0 && action.frame > 6.0 {
                action.contact = true;
            }
            if rustimal_logic::net_swing_outcome(0, action.frame >= swing_end, false)
                == rustimal_logic::NetSwingOutcome::Stop
            {
                action.frame = 1.0 + (action.frame - swing_end);
                action.phase = Phase::Recover;
                debug!("net action: recovery");
            }
        }
        Phase::Recover => {
            action.frame += delta;
            if action.frame >= stop_end {
                action.phase = Phase::Idle;
                debug!("net action: ready");
            }
        }
        Phase::Idle => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_net(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    rigs: Query<(&ChildOf, &CharacterRig), Added<CharacterRig>>,
    players: Query<(), With<crate::player::Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
    mut binds: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    let Some(disc) = disc else {
        return;
    };
    for (owner, rig) in &rigs {
        if !players.contains(owner.parent()) {
            continue;
        }
        let Some(hand) = rig.right_hand() else {
            continue;
        };
        let Some(skeleton) = disc
            .catalog
            .get("cKF_bs_r_tol_net_1")
            .and_then(|a| Skeleton::read(&disc.rom, a.offset))
        else {
            continue;
        };
        let anim = |name: &str, n| {
            disc.catalog
                .get(name)
                .and_then(|a| Animation::read(&disc.rom, a.offset, n))
        };
        let (Some(idle), Some(swing), Some(recovery), Some(player_swing), Some(player_hold)) = (
            anim("cKF_ba_r_tol_net_1_kamae_main_m1", skeleton.joints.len()),
            anim("cKF_ba_r_tol_net_1_net_swing1", skeleton.joints.len()),
            anim("cKF_ba_r_tol_net_1_swing_wait1", skeleton.joints.len()),
            anim("cKF_ba_r_ply_1_net_swing1", rig.joint_count()),
            anim("cKF_ba_r_ply_1_net1", rig.joint_count()),
        ) else {
            warn!("normal net animations unavailable");
            continue;
        };
        let root = commands
            .spawn((
                Name::new("held normal net"),
                Transform::default(),
                Visibility::Hidden,
                ChildOf(hand),
            ))
            .id();
        let locals = idle.pose(1.0).local_transforms(&skeleton);
        let mut joints = Vec::new();
        let flex = commands
            .spawn((
                Name::new("net procedural bend"),
                Transform::default(),
                Visibility::default(),
            ))
            .id();
        for (j, local) in locals.iter().enumerate() {
            let parent =
                skeleton.parents[j].map_or(root, |p| if p == 3 { flex } else { joints[p] });
            joints.push(
                commands
                    .spawn((
                        Name::new(format!("net joint {j}")),
                        *local,
                        Visibility::default(),
                        ChildOf(parent),
                    ))
                    .id(),
            );
            if j == 3 {
                commands.entity(flex).insert(ChildOf(joints[j]));
            }
        }
        let inverse = binds.add(SkinnedMeshInverseBindposes::from(vec![
            Mat4::IDENTITY;
            joints.len()
        ]));
        let slots = skeleton.matrix_joints();
        for &j in &slots {
            let Some(list) = skeleton.joints[j].model else {
                continue;
            };
            let sink = run_display_lists(&disc.rom, &[list]);
            for part in &sink.parts {
                let texture = part.key.tex.as_ref().and_then(|key| {
                    decode_texture(&disc.rom, key, None, &Segments::default())
                        .map(|rgba| images.add(texture_image(rgba, key)))
                });
                let material = materials.add(N64Material {
                    combiner: CombinerUniform::from_key(&part.key, false, false),
                    texture,
                    env: default(),
                    blend: false,
                });
                commands.spawn((
                    Mesh3d(meshes.add(crate::character::skinned_part_mesh(part, j, &slots))),
                    MeshMaterial3d(material),
                    SkinnedMesh {
                        inverse_bindposes: inverse.clone(),
                        joints: joints.clone(),
                    },
                    ChildOf(root),
                ));
            }
        }
        commands.entity(root).insert(HeldNet {
            joints,
            flex,
            angle: [0, 182, -7281],
            idle_frame: 1.0,
            last_yaw: 0.0,
            phase: Phase::Idle,
            morph: 0.0,
        });
        info!(
            "normal net attached to player right hand; swing {} / recovery {} frames",
            player_swing.frames, recovery.frames
        );
        // mPlayer_PART_TABLE_NET=3, 27 entries per table; the first byte
        // controls translation, each subsequent byte selects joint i.
        let table_len = rig.joint_count() + 1;
        let hold_mask = disc
            .catalog
            .get("BOY_part_data")
            .and_then(|a| {
                disc.rom.slice(
                    disc::RomSource::Rel,
                    a.offset + 3 * table_len as u32,
                    table_len as u32,
                )
            })
            .map(|b| b[1..].to_vec())
            .unwrap_or_default();
        commands.insert_resource(NetModels {
            skeleton,
            idle,
            swing,
            recovery,
            player_swing,
            player_hold,
            hold_mask,
        });
    }
}

fn animate_net(
    time: Res<Time>,
    tool: Res<EquippedTool>,
    action: Res<NetAction>,
    house: Res<State<crate::house::HouseState>>,
    shop: Res<State<crate::shop::ShopLocation>>,
    models: Option<Res<NetModels>>,
    player: Query<
        (&crate::locomotion::Locomotion, &crate::interaction::Facing),
        With<crate::player::Player>,
    >,
    mut nets: Query<(&mut HeldNet, &mut Visibility)>,
    mut transforms: Query<&mut Transform>,
) {
    let (Some(models), Ok((loco, facing))) = (models, player.single()) else {
        return;
    };
    let ticks = time.delta_secs() * 60.0;
    let shown = *tool == EquippedTool::Net
        && *house == crate::house::HouseState::Town
        && *shop == crate::shop::ShopLocation::Town;
    for (mut net, mut visibility) in &mut nets {
        *visibility = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if !shown {
            net.angle = [0, 182, -7281];
            net.phase = Phase::Idle;
            net.morph = 0.0;
            continue;
        }
        let yaw = facing.dir.x.atan2(facing.dir.z);
        let turn = rustimal_logic::player_move::wrap_deg((yaw - net.last_yaw).to_degrees())
            * 65536.0
            / 360.0
            / ticks.max(1e-3);
        net.last_yaw = yaw;
        let target = match action.phase {
            Phase::Swing if action.player_frame < 5.0 => [0, 364, -11832],
            Phase::Swing | Phase::Recover => [0; 3],
            Phase::Idle => match loco.state() {
                rustimal_logic::player_move::LocomotionState::Walk => {
                    rustimal_logic::net_walk_angle(loco.anim_speed(), turn)
                }
                rustimal_logic::player_move::LocomotionState::Run
                | rustimal_logic::player_move::LocomotionState::Dash => [0; 3],
                _ => [0, 182, -7281],
            },
        };
        for (angle, target) in net.angle.iter_mut().zip(target) {
            *angle = rustimal_logic::chase_net_angle(*angle, target, ticks);
        }
        if net.phase != action.phase {
            net.phase = action.phase;
            net.morph = 5.0;
        }
        let (anim, frame) = match action.phase {
            Phase::Swing => (&models.swing, action.frame),
            Phase::Recover => (&models.recovery, action.frame),
            Phase::Idle => {
                net.idle_frame = 1.0
                    + (net.idle_frame - 1.0 + ticks * 0.5)
                        .rem_euclid((models.idle.frames - 1).max(1) as f32);
                (&models.idle, net.idle_frame)
            }
        };
        let locals = anim.pose(frame).local_transforms(&models.skeleton);
        // The source callback rotates AFTER dummy joint3, independently
        // of keyframe morphing. Its separate node affects descendants4/5.
        if let Ok(mut dummy) = transforms.get_mut(net.flex) {
            let a = net.angle.map(|v| v as f32 * std::f32::consts::PI / 32768.0);
            dummy.rotation = Quat::from_euler(EulerRot::ZYX, a[2], a[1], a[0]);
        }
        let blend = if net.morph > 0.0 {
            (ticks * 0.5 / net.morph).min(1.0)
        } else {
            1.0
        };
        for (&joint, local) in net.joints.iter().zip(locals) {
            if let Ok(mut t) = transforms.get_mut(joint) {
                t.translation = t.translation.lerp(local.translation, blend);
                t.rotation = t.rotation.slerp(local.rotation, blend);
                t.scale = local.scale;
            }
        }
        net.morph = (net.morph - ticks * 0.5).max(0.0);
    }
}

pub struct PlayerToolsPlugin;
fn cancel_net(mut action: ResMut<NetAction>) {
    action.cancel();
}
impl Plugin for PlayerToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetAction>()
            .add_systems(OnExit(crate::states::AppState::InGame), cancel_net)
            .add_systems(
                Update,
                advance_net
                    .after(crate::input::InteractSet::Tools)
                    .after(crate::save::LoadSet)
                    .before(crate::locomotion::MovementSet)
                    .run_if(in_state(crate::states::AppState::InGame)),
            )
            .add_systems(
                Update,
                (spawn_net, animate_net)
                    .chain()
                    .after(crate::character::CharacterAnimationSet)
                    .run_if(in_state(crate::states::AppState::InGame)),
            );
    }
}
