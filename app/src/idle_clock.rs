//! m_banti: the clock/date window that fades in at the lower right while the
//! player stands still outdoors, drawn from the disc's clk_win models.
//!
//! [`banti`] holds the timing; this module feeds it the player's display
//! type (standing, moving, talking, net ready), the projected player
//! position and the clock, and draws banti_draw's pieces on the overlay
//! camera in the font projection (x16 units): weekday, blinking colon (odd
//! seconds), AM/PM, frame, and the date/time digits, each a three-joint cKF
//! flip whose top and bottom read the shown and next digit textures
//! (segments 8 and 9) with the flipping leaf (joint 2) hidden at rest.
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use rustimal_logic::banti::{self, Addressable, Banti, BantiInput, BantiTime};

use crate::disc_gfx::{run_display_lists_with_colors, Segments};
use crate::ecology::EquippedTool;
use crate::game_disc::GameDisc;
use crate::input::InputFocus;
use crate::locomotion::Locomotion;
use crate::n64_material::N64Material;
use crate::player::{Player, WorldCamera};
use crate::player_tools::NetAction;
use crate::skeleton::{Animation, Skeleton};

/// The overlay camera's layer (msg_window).
const LAYER: usize = 1;
/// mFont_SCALE_F: font-projection units per screen pixel.
const UNIT: f32 = 16.0;
/// Below the message window and menus.
const Z: f32 = 20.0;
/// set_pos_table of banti_draw_hiduke_sub / banti_draw_jikan_sub.
const DATE_POS: [Vec2; 2] = [Vec2::new(64.0, -70.0), Vec2::new(94.0, -70.0)];
const TIME_POS: [Vec2; 4] = [
    Vec2::new(60.0, -91.0),
    Vec2::new(76.0, -91.0),
    Vec2::new(98.0, -91.0),
    Vec2::new(113.0, -91.0),
];
const WEEKDAYS: [&str; 7] = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
/// The leaf joint drawn only while a digit flips.
const LEAF_JOINT: usize = 2;

struct Rig {
    skeleton: Skeleton,
    animation: Animation,
}

#[derive(Resource)]
struct ClockAssets {
    mode: u32,
    youbi: u32,
    maru: u32,
    maru2: u32,
    ampm: u32,
    date: Rig,
    time: Rig,
}

#[derive(Resource)]
struct IdleClock {
    banti: Banti,
    ticks: f32,
    root: Option<Entity>,
    /// What the built pieces show; a change rebuilds them.
    built: Option<Shown>,
    /// Blinking colon pieces.
    colon: Vec<Entity>,
    /// Per digit: joint entities.
    digits: Vec<Vec<Entity>>,
    materials: Vec<Handle<N64Material>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Shown {
    displayed: BantiTime,
    next: BantiTime,
    left: bool,
}

fn rtc() -> (BantiTime, bool) {
    let now = crate::clock::now();
    let time = BantiTime {
        month: now.month,
        day: now.day,
        weekday: now.weekday,
        hour: now.hour,
        min: now.minute,
    };
    (time, (now.day_secs % 60) % 2 == 1)
}

fn load(disc: &GameDisc) -> Option<ClockAssets> {
    let list = |name: &str| disc.catalog.get(name).map(|a| a.offset);
    let rig = |bs: &str, ba: &str| -> Option<Rig> {
        let skeleton = Skeleton::read(&disc.rom, list(bs)?)?;
        let animation = Animation::read(&disc.rom, list(ba)?, skeleton.joints.len())?;
        Some(Rig {
            skeleton,
            animation,
        })
    };
    Some(ClockAssets {
        mode: list("clk_win_mode")?,
        youbi: list("clk_win_youbiT_model")?,
        maru: list("clk_win_maruT_model")?,
        maru2: list("clk_win_maru2T_model")?,
        ampm: list("clk_win_ampmT_model")?,
        date: rig("cKF_bs_r_clk_hiniti", "cKF_ba_r_clk_hiniti")?,
        time: rig("cKF_bs_r_clk_jikan", "cKF_ba_r_clk_jikan")?,
    })
}

fn setup(mut commands: Commands, disc: Res<GameDisc>) {
    let Some(assets) = load(&disc) else {
        warn!("idle clock: clk_win assets missing");
        commands.insert_resource(IdleClockUnavailable);
        return;
    };
    commands.insert_resource(assets);
    commands.insert_resource(IdleClock {
        banti: Banti::new(rtc().0),
        ticks: 0.0,
        root: None,
        built: None,
        colon: Vec::new(),
        digits: Vec::new(),
        materials: Vec::new(),
    });
}

#[derive(Resource)]
struct IdleClockUnavailable;

fn segment(disc: &GameDisc, name: &str) -> Option<std::sync::Arc<[u8]>> {
    let a = disc.catalog.get(name)?;
    let bytes = disc.rom.slice(disc::RomSource::Rel, a.offset, a.size)?;
    Some(bytes.into())
}

/// The player's main state as `mPlayer_ADDRESSABLE_*`
/// (Player_actor_Set_address_able_display).
fn addressable(
    focus: InputFocus,
    loco: Option<&Locomotion>,
    tool: EquippedTool,
    net: &NetAction,
) -> Addressable {
    use rustimal_logic::player_move::LocomotionState;
    match focus {
        InputFocus::Dialogue | InputFocus::Shop | InputFocus::Quest | InputFocus::Mail => {
            return Addressable::FalseTalking
        }
        _ => {}
    }
    if net.busy() {
        // SWING_NET / STOP_NET.
        return Addressable::True;
    }
    let standing = loco.is_none_or(|l| l.state() == LocomotionState::Wait);
    match (standing, tool) {
        // READY_NET; READY_ROD counts as movement.
        (true, EquippedTool::Net) => Addressable::FalseReadyNet,
        (true, EquippedTool::Rod) => Addressable::FalseMovement,
        (true, _) => Addressable::True,
        (false, _) => Addressable::FalseMovement,
    }
}

#[allow(clippy::too_many_arguments)]
fn update(
    mut commands: Commands,
    time: Res<Time>,
    disc: Res<GameDisc>,
    assets: Res<ClockAssets>,
    mut clock: ResMut<IdleClock>,
    focus: Res<InputFocus>,
    tool: Res<EquippedTool>,
    net: Res<NetAction>,
    house: Option<Res<State<crate::house::HouseState>>>,
    shop: Option<Res<State<crate::shop::ShopLocation>>>,
    player: Query<(&GlobalTransform, Option<&Locomotion>), With<Player>>,
    camera: Query<(&Camera, &GlobalTransform), With<WorldCamera>>,
    mut transforms: Query<&mut Transform>,
    mut visibility: Query<&mut Visibility>,
    (mut meshes, mut materials, mut images): (
        ResMut<Assets<Mesh>>,
        ResMut<Assets<N64Material>>,
        ResMut<Assets<Image>>,
    ),
) {
    // mFI_FIELDTYPE_FG: town only.
    let outdoors = house.is_some_and(|h| *h.get() == crate::house::HouseState::Town)
        && shop.is_none_or(|s| *s.get() == crate::shop::ShopLocation::Town);
    let Ok((at, loco)) = player.single() else {
        return;
    };
    let screen = camera
        .single()
        .ok()
        .and_then(|(cam, cam_at)| {
            let p = cam.world_to_viewport(cam_at, at.translation()).ok()?;
            let size = cam.logical_viewport_size()?;
            Some((p.x * 320.0 / size.x, p.y * 240.0 / size.y))
        })
        .unwrap_or((160.0, 120.0));
    let (now, odd_second) = rtc();
    let input = BantiInput {
        addressable: addressable(*focus, loco, *tool, &net),
        player_screen: screen,
        submenu_idle: !matches!(*focus, InputFocus::Pockets | InputFocus::Paused),
        demo: false,
        now,
    };
    clock.ticks += time.delta_secs() * 60.0;
    let ticks = (clock.ticks as u32).min(4);
    clock.ticks = (clock.ticks - ticks as f32).min(1.0);
    for _ in 0..ticks {
        clock.banti.tick(&input);
    }

    let b = clock.banti;
    let shown = outdoors && b.alpha > 0.01;
    let want = Shown {
        displayed: b.displayed,
        next: b.next,
        left: b.disp_left,
    };
    if shown && clock.built != Some(want) {
        if let Some(root) = clock.root.take() {
            commands.entity(root).despawn();
        }
        build(
            &mut commands,
            &disc,
            &assets,
            &mut clock,
            want,
            (&mut meshes, &mut materials, &mut images),
        );
        clock.built = Some(want);
    }
    let Some(root) = clock.root else { return };
    if let Ok(mut v) = visibility.get_mut(root) {
        *v = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if !shown {
        return;
    }
    // Every piece's prim and env alpha is the window alpha.
    let alpha = (b.alpha * 255.0) as u8 as f32 / 255.0;
    for handle in &clock.materials {
        if let Some(mut m) = materials.get_mut(handle) {
            m.combiner.prim.w = alpha;
            m.combiner.env.w = alpha;
        }
    }
    for &e in &clock.colon {
        if let Ok(mut v) = visibility.get_mut(e) {
            *v = if odd_second {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
    for (index, joints) in clock.digits.iter().enumerate() {
        let flip = b.flips[index];
        let rig = if index < 2 {
            &assets.date
        } else {
            &assets.time
        };
        let pose = rig
            .animation
            .pose(flip.frame)
            .local_transforms(&rig.skeleton);
        for (j, (&joint, local)) in joints.iter().zip(&pose).enumerate() {
            if let Ok(mut t) = transforms.get_mut(joint) {
                *t = *local;
            }
            if j == LEAF_JOINT {
                if let Ok(mut v) = visibility.get_mut(joint) {
                    *v = if flip.playing {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                }
            }
        }
    }
}

/// Spawn banti_draw's pieces for `shown`.
fn build(
    commands: &mut Commands,
    disc: &GameDisc,
    assets: &ClockAssets,
    clock: &mut IdleClock,
    shown: Shown,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) {
    let left = if shown.left { -184.0 } else { 0.0 };
    let root = commands
        .spawn((
            Name::new("idle clock"),
            Transform::from_xyz(left * UNIT, 0.0, Z).with_scale(Vec3::new(UNIT, UNIT, 1.0)),
            Visibility::Hidden,
        ))
        .id();
    clock.root = Some(root);
    clock.colon.clear();
    clock.digits.clear();
    clock.materials.clear();
    let mut depth = 0.0;
    let mut piece = |commands: &mut Commands,
                     parent: Entity,
                     list: u32,
                     prim: [u8; 3],
                     env: [u8; 3],
                     segs: &Segments,
                     materials_out: &mut Vec<Handle<N64Material>>,
                     meshes: &mut Assets<Mesh>,
                     materials: &mut Assets<N64Material>,
                     images: &mut Assets<Image>|
     -> Entity {
        let sink = run_display_lists_with_colors(
            &disc.rom,
            &[assets.mode, list],
            [prim[0], prim[1], prim[2], 255],
            255,
            [env[0], env[1], env[2], 255],
            &[],
        );
        let holder = commands
            .spawn((
                Transform::from_xyz(0.0, 0.0, depth),
                Visibility::Inherited,
                ChildOf(parent),
            ))
            .id();
        depth += 0.01;
        for (mesh, material) in crate::disc_town::sink_meshes_scaled(
            disc,
            &sink,
            None,
            true,
            segs,
            1.0,
            (meshes, materials, images),
        ) {
            materials_out.push(material.clone());
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                RenderLayers::layer(LAYER),
                ChildOf(holder),
            ));
        }
        holder
    };
    let d = shown.displayed;
    let mut mats = Vec::new();
    // Weekday: Sunday's ink is red.
    let mut segs = Segments::default();
    if let Some(tex) = segment(
        disc,
        &format!("clk_win_{}_tex_rgb_ia8", WEEKDAYS[d.weekday as usize % 7]),
    ) {
        segs.set_segment(9, tex);
    }
    let youbi_env = if d.weekday == 0 {
        [165, 30, 30]
    } else {
        [80, 40, 0]
    };
    piece(
        commands,
        root,
        assets.youbi,
        [255, 255, 225],
        youbi_env,
        &segs,
        &mut mats,
        meshes,
        materials,
        images,
    );
    let colon = piece(
        commands,
        root,
        assets.maru2,
        [215, 120, 0],
        [70, 50, 50],
        &Segments::default(),
        &mut mats,
        meshes,
        materials,
        images,
    );
    clock.colon.push(colon);
    let mut segs = Segments::default();
    let ampm = if d.hour < 12 {
        "clk_win_am_tex_rgb_ia8"
    } else {
        "clk_win_pm_tex_rgb_ia8"
    };
    if let Some(tex) = segment(disc, ampm) {
        segs.set_segment(10, tex);
    }
    piece(
        commands,
        root,
        assets.ampm,
        [255, 255, 120],
        [70, 10, 10],
        &segs,
        &mut mats,
        meshes,
        materials,
        images,
    );
    piece(
        commands,
        root,
        assets.maru,
        [255, 255, 0],
        [70, 50, 50],
        &Segments::default(),
        &mut mats,
        meshes,
        materials,
        images,
    );

    // Date then time digits: (rig, shown texture, next texture, position).
    let n = shown.next;
    let date_tex = |v: u8| format!("clk_win_suuji{}_TA_tex_txt", v.clamp(1, 31));
    let time_tex = |v: u8, hide_zero: bool| {
        if hide_zero && v == 0 {
            "clk_win_jikan_TA_tex_txt".to_string()
        } else {
            format!("clk_win_jikan{}_TA_tex_txt", v % 10)
        }
    };
    let (h0, h1) = (banti::hour12(d.hour), banti::hour12(n.hour));
    let digits: [(bool, String, String, Vec2, [u8; 3], [u8; 3]); 6] = [
        (
            true,
            date_tex(d.month),
            date_tex(n.month),
            DATE_POS[0],
            [235, 255, 120],
            [80, 40, 40],
        ),
        (
            true,
            date_tex(d.day),
            date_tex(n.day),
            DATE_POS[1],
            [235, 255, 120],
            [80, 40, 40],
        ),
        (
            false,
            time_tex(h0 / 10, true),
            time_tex(h1 / 10, true),
            TIME_POS[0],
            [255; 3],
            [60, 25, 10],
        ),
        (
            false,
            time_tex(h0 % 10, false),
            time_tex(h1 % 10, false),
            TIME_POS[1],
            [255; 3],
            [60, 25, 10],
        ),
        (
            false,
            time_tex(d.min / 10, false),
            time_tex(n.min / 10, false),
            TIME_POS[2],
            [255; 3],
            [60, 25, 10],
        ),
        (
            false,
            time_tex(d.min % 10, false),
            time_tex(n.min % 10, false),
            TIME_POS[3],
            [255; 3],
            [60, 25, 10],
        ),
    ];
    for (date, now_tex, next_tex, pos, prim, env) in digits {
        let rig = if date { &assets.date } else { &assets.time };
        let mut segs = Segments::default();
        if let Some(tex) = segment(disc, &now_tex) {
            segs.set_segment(8, tex);
        }
        if let Some(tex) = segment(disc, &next_tex) {
            segs.set_segment(9, tex);
        }
        let anchor = commands
            .spawn((
                Transform::from_xyz(pos.x, pos.y, 0.0),
                Visibility::Inherited,
                ChildOf(root),
            ))
            .id();
        let pose = rig.animation.pose(1.0).local_transforms(&rig.skeleton);
        let mut joints: Vec<Entity> = Vec::with_capacity(rig.skeleton.joints.len());
        for (i, joint) in rig.skeleton.joints.iter().enumerate() {
            let parent = rig.skeleton.parents[i].map_or(anchor, |p| joints[p]);
            let e = commands
                .spawn((pose[i], Visibility::Inherited, ChildOf(parent)))
                .id();
            if let Some(list) = joint.model {
                piece(
                    commands, e, list, prim, env, &segs, &mut mats, meshes, materials, images,
                );
            }
            joints.push(e);
        }
        clock.digits.push(joints);
    }
    clock.materials = mats;
}

pub struct IdleClockPlugin;

impl Plugin for IdleClockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                setup.run_if(
                    resource_exists::<GameDisc>
                        .and_then(not(resource_exists::<ClockAssets>))
                        .and_then(not(resource_exists::<IdleClockUnavailable>)),
                ),
                update
                    .run_if(resource_exists::<ClockAssets>.and_then(resource_exists::<IdleClock>)),
            )
                .chain(),
        );
    }
}
