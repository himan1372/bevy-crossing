//! The new-game intro's arrival (`ac_intro_demo`: `aID_ACT_FIRST_SET` .. `aID_ACT_RIDE_OFF_PLAYER`): the
//! scene changes from the train to the town at the station, the station's train comes in slowing
//! (`mTRC_demo_init`, `m_train_control.c`) with the player standing in its passenger car, stops, and the
//! car's door opens. The camera holds still on the platform (`Camera2_request_main_demo_fromNowPos2`
//! at (2180, 820)).
//!
//! The train is `ac_train0` (the locomotive `cKF_bs_r_obj_train1_1`, the tender `obj_train1_2_model`)
//! pulling `ac_train1` (the passenger car `cKF_bs_r_obj_train1_3` with its `_open` / `_close`
//! animations). Then the player steps down, Porter the station master turns and greets them (force talk
//! 0x07DD), the train pulls out and the player walks one unit (`aID_walk_one_unit`) to the platform's end.
//! Not built yet: the engineer in the cab, the smoke and steam, the back car's spring, the train's
//! sounds, the get-off animation, and Nook's welcome that follows.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use bevy::prelude::*;
use rustimal_logic::train_control::{self as tc, Action, Event, Train};

use crate::dialogue::DialogueState;
use crate::game_disc::GameDisc;
use crate::input::InputFocus;
use crate::intro::Intro;
use crate::n64_material::N64Material;
use crate::player::Player;
use crate::prop::Prop;
use crate::scene_wipe::SceneWipe;
use crate::shop::ShopLocation;
use crate::skeleton::Animation;

/// Game units per cell.
const UNIT: f32 = 40.0;
/// The field's border block (16 cells) and the town's half size (80 x 96 cells): game units to world.
const BORDER: f32 = 16.0;
const HALF_X: f32 = 40.0;
const HALF_Z: f32 = 48.0;
/// The camera's centre (`aID_train_birth_wait_init`): the doorway, 35 units below the ground.
const CAMERA_CENTRE: Vec2 = Vec2::new(2180.0, 820.0);
/// `ac_train1`'s passenger offset (`aTR1_passenger_ctrl`: arg0 = 1).
const PASSENGER: Vec3 = Vec3::new(60.0, 20.0, 20.0);
/// The car and tender trail the locomotive by this much (`-125` each).
const COUPLING: f32 = 125.0;
/// The town camera (`player.rs`): 620 units out, pitched 45 degrees.
const CAMERA_DISTANCE: f32 = 620.0 / UNIT;
const CAMERA_PITCH: f32 = 45.0;

static MUSIC: AtomicBool = AtomicBool::new(false);

/// Whether the arrival's music (`BGM_INTRO_ARRIVE`) should play.
pub fn music_on() -> bool {
    MUSIC.load(Ordering::Relaxed)
}

/// Game units on the field (x, z) to the town's world position.
fn world(x: f32, z: f32) -> Vec2 {
    Vec2::new(x / UNIT - BORDER - HALF_X, z / UNIT - BORDER - HALF_Z)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Stage {
    #[default]
    Idle,
    /// Fading out of the train scene.
    Leaving,
    /// The train comes in and stops.
    Riding,
    /// The door is open; the player waits a moment, then steps off.
    Open {
        seconds: f32,
    },
    /// `mPlayer_INDEX_DEMO_GETOFF_TRAIN`: stepping down onto the platform, frames so far.
    GetOff {
        frames: f32,
    },
    /// Porter turns to the player (`aSTM_THINK_GET_OFF_WAIT`), then speaks (force talk 0x07DD).
    Turn,
    Talk,
    /// `aID_walk_one_unit_init`: the player walks to (2220, 840).
    Walk,
}

/// `aID_PLR_START`: where the player steps down to, and where they walk to.
const OFF_POS: Vec2 = Vec2::new(2180.0, 820.0);
const WALK_POS: Vec2 = Vec2::new(2220.0, 840.0);
/// `aID_walk_one_unit_init`'s speed, game units a frame.
const WALK_SPEED: f32 = 2.5;
/// `aID_go_out_of_station`: past this z the arrival music fades (`aSTM_talk_wait`).
const OUT_Z: f32 = 970.0;
/// `NPC_STATION_MASTER`'s model (`ALL_NPC_NUM + SP_NPC_STATION_MASTER - SP_NPC_START`), voice 2.
const PORTER_NPC: u32 = 238 + 14;
const PORTER_SOUND: u8 = 2;
/// Force talk 0 of `aSTM_set_force_talk_info`.
const PORTER_GREETING: usize = 0x07DD;

/// Porter, the station master.
#[derive(Component)]
struct Porter;

/// The arrival's state.
#[derive(Resource, Default)]
pub(crate) struct Arrival {
    stage: Stage,
    train: Option<Train>,
    /// Where the player stood in the car.
    from: Vec3,
    /// The track's height (`mCoBG_GetBgY_OnlyCenter_FromWpos2`) plus the rails.
    y: f32,
}

impl Arrival {
    /// The train scene is over: leave it.
    pub(crate) fn begin(&mut self) {
        if self.stage == Stage::Idle {
            self.stage = Stage::Leaving;
        }
    }
}

/// The three parts of the train.
#[derive(Component)]
struct Locomotive {
    prop: Prop,
    anim: Option<Arc<Animation>>,
    frame: f32,
}

#[derive(Component)]
struct Tender;

/// `aTR1_setupAction`: the car's door.
#[derive(Component)]
struct Car {
    prop: Prop,
    open: Option<Arc<Animation>>,
    close: Option<Arc<Animation>>,
    /// Which animation plays, its frame and speed (0.5 while the door moves).
    closing: bool,
    frame: f32,
    speed: f32,
    last_action: Action,
    posed: bool,
}

/// The track's centre, in world units, for the train at field x `x` (game units).
fn track(x: f32, y: f32) -> Vec3 {
    let at = world(x, tc::TRACK_Z);
    Vec3::new(at.x, y, at.y)
}

/// `aSTR_PAL_TRAIN1_A1` / `_A2`: the palette the actors put in segment 8.
fn palette(disc: &GameDisc, index: u32, winter: bool) -> Option<u32> {
    crate::disc_town::structure_palette(disc, index, winter)
}

/// Spawns the train (`mTRC_trainSet`): locomotive, tender, car.
fn spawn_train(
    commands: &mut Commands,
    disc: &GameDisc,
    y: f32,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) {
    let scale = crate::disc_town::OBJECT_SCALE;
    // `world.angle.y = 90 degrees`: the train faces +x.
    let facing = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let engine_palette = palette(disc, 57, false);
    let car_palette = palette(disc, 58, false);
    let at = |x: f32| {
        Transform::from_translation(track(x, y))
            .with_rotation(facing)
            .with_scale(Vec3::splat(scale))
    };
    // The locomotive.
    let loco = commands
        .spawn((
            Name::new("train locomotive"),
            at(tc::DEMO_X),
            Visibility::default(),
        ))
        .id();
    if let Some((_, prop)) = Prop::spawn(
        commands,
        disc,
        "cKF_bs_r_obj_train1_1",
        loco,
        Transform::default(),
        engine_palette,
        (&mut *meshes, &mut *materials, &mut *images),
    ) {
        let anim = prop.animation(disc, "cKF_ba_r_obj_train1_1");
        commands.entity(loco).insert(Locomotive {
            prop,
            anim,
            frame: 1.0,
        });
    }
    // The tender, a plain model.
    let tender = commands
        .spawn((
            Name::new("train tender"),
            Tender,
            at(tc::DEMO_X - COUPLING),
            Visibility::default(),
        ))
        .id();
    if let Some(list) = disc.catalog.get("obj_train1_2_model") {
        let sink = crate::disc_gfx::run_display_lists(&disc.rom, &[list.offset]);
        for (mesh, material) in crate::disc_town::sink_meshes_scaled(
            disc,
            &sink,
            engine_palette,
            false,
            &crate::disc_gfx::Segments::default(),
            1.0,
            (&mut *meshes, &mut *materials, &mut *images),
        ) {
            commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), ChildOf(tender)));
        }
    }
    // The passenger car.
    let car = commands
        .spawn((
            Name::new("train car"),
            at(tc::DEMO_X - 2.0 * COUPLING),
            Visibility::default(),
        ))
        .id();
    if let Some((_, prop)) = Prop::spawn(
        commands,
        disc,
        "cKF_bs_r_obj_train1_3",
        car,
        Transform::default(),
        car_palette,
        (&mut *meshes, &mut *materials, &mut *images),
    ) {
        let open = prop.animation(disc, "cKF_ba_r_obj_train1_3_open");
        let close = prop.animation(disc, "cKF_ba_r_obj_train1_3_close");
        commands.entity(car).insert(Car {
            prop,
            open,
            close,
            closing: false,
            frame: 1.0,
            speed: 0.0,
            last_action: Action::BeginSlowdown,
            posed: false,
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn leave(
    mut commands: Commands,
    mut arrival: ResMut<Arrival>,
    mut intro: ResMut<Intro>,
    mut wipe: ResMut<SceneWipe>,
    mut next: ResMut<NextState<ShopLocation>>,
    disc: Option<Res<GameDisc>>,
    town: Res<crate::town::Town>,
    mut player: Query<(&mut Transform, &mut crate::interaction::Facing), With<Player>>,
    (mut meshes, mut materials, mut images): (
        ResMut<Assets<Mesh>>,
        ResMut<Assets<N64Material>>,
        ResMut<Assets<Image>>,
    ),
) {
    if arrival.stage != Stage::Leaving {
        return;
    }
    // The train scene fades out; the change is made under the black.
    if !wipe.black() {
        wipe.hold();
        return;
    }
    let Some(disc) = disc else { return };
    let Ok((mut at, mut facing)) = player.single_mut() else {
        return;
    };
    // The track's ground and the rails (the train sits 20 units above it).
    let ground_at = |x: f32| {
        let w = world(x, tc::TRACK_Z);
        town.ground_y(Vec3::new(w.x, 0.0, w.y))
    };
    arrival.y = ground_at(tc::DEMO_X) + 20.0 / UNIT;
    arrival.train = Some(Train::demo());
    spawn_train(
        &mut commands,
        &disc,
        arrival.y,
        (&mut meshes, &mut materials, &mut images),
    );
    let car = track(tc::DEMO_X - 2.0 * COUPLING, arrival.y);
    at.translation = car + PASSENGER / UNIT;
    arrival.from = at.translation;
    facing.dir = Vec3::Z;
    // Porter waits by the station, looking at the train (`aSTM_think_init_proc`: -90 degrees).
    if let Some(b) = town.retail.as_ref().and_then(|r| {
        r.buildings
            .iter()
            .find(|b| b.item == crate::disc_town::STATION)
    }) {
        let off = world(OFF_POS.x, OFF_POS.y);
        let p = Vec3::new(off.x - 2.0, 0.0, off.y);
        let home = p.with_y(town.ground_y(p));
        info!(
            "Porter at {home:?} (the station building's origin is {:?})",
            b.origin
        );
        commands.spawn((
            Name::new("Porter"),
            Porter,
            crate::character::NpcClip::default(),
            crate::character::NpcModel(PORTER_NPC),
            Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
            Transform::from_translation(home + Vec3::Y).with_rotation(Quat::from_rotation_y(0.0)),
        ));
    }
    next.set(ShopLocation::Town);
    *intro = Intro::Arrival;
    MUSIC.store(true, Ordering::Relaxed);
    // `Common_Get(transition).wipe_type = WIPE_TYPE_CIRCLE_LEFT`: the town opens as a circle.
    wipe.circle_next(false);
    wipe.release();
    arrival.stage = Stage::Riding;
    info!("intro: arriving at the station");
}

/// `mTRC_trainControl` and the actors that follow it.
#[allow(clippy::too_many_arguments)]
fn run_train(
    time: Res<Time>,
    mut arrival: ResMut<Arrival>,
    mut locomotive: Query<(&mut Transform, &mut Locomotive), (Without<Tender>, Without<Car>)>,
    mut tender: Query<&mut Transform, (With<Tender>, Without<Locomotive>, Without<Car>)>,
    mut car: Query<(&mut Transform, &mut Car), (Without<Locomotive>, Without<Tender>)>,
    mut transforms: Query<
        &mut Transform,
        (
            Without<Locomotive>,
            Without<Tender>,
            Without<Car>,
            Without<Player>,
        ),
    >,
    mut player: Query<
        &mut Transform,
        (
            With<Player>,
            Without<Locomotive>,
            Without<Tender>,
            Without<Car>,
        ),
    >,
) {
    let Some(train) = arrival.train.as_mut() else {
        return;
    };
    let dt = time.delta_secs() * 60.0;
    let event = train.step(dt);
    let (x, speed, action) = (train.x, train.speed, train.action);
    let y = arrival.y;
    let mut event_seen = event;
    if let Ok((mut at, mut loco)) = locomotive.single_mut() {
        at.translation = track(x, y);
        // `aTR0_actor_move`: the wheels turn at a tenth of the speed per unit, at most 0.5 frames a tick.
        let rate = (speed / 40.0 * 10.0).min(0.5);
        loco.frame += rate * dt;
        while loco.frame >= 25.0 {
            loco.frame -= 24.0;
        }
        loco.prop
            .pose(loco.anim.as_deref(), loco.frame, &mut transforms);
    }
    if let Ok(mut at) = tender.single_mut() {
        at.translation = track(x - COUPLING, y);
    }
    if let Ok((mut at, mut car)) = car.single_mut() {
        at.translation = track(x - 2.0 * COUPLING, y);
        // `aTR1_setupAction`: the door follows the train's action.
        if car.last_action != action {
            car.last_action = action;
            let (closing, speed) = match action.number() {
                4 => (false, 0.5),
                5 => (true, 0.0),
                6 => (true, 0.5),
                _ => (false, 0.0),
            };
            car.closing = closing;
            car.speed = speed;
            car.frame = 1.0;
            if action.number() == 4 || action.number() == 6 {
                crate::sound::se(43);
            }
            car.posed = false;
        }
        let end = if car.closing { 32.0 } else { 24.0 };
        if car.speed > 0.0 {
            car.frame = (car.frame + car.speed * dt).min(end);
            car.posed = false;
        }
        if !car.posed {
            car.posed = true;
            let anim = if car.closing {
                car.close.clone()
            } else {
                car.open.clone()
            };
            car.prop.pose(anim.as_deref(), car.frame, &mut transforms);
        }
    }
    // The passenger stands in the car (`aTR1_passenger_ctrl`).
    if matches!(arrival.stage, Stage::Riding | Stage::Open { .. }) {
        if let Ok(mut at) = player.single_mut() {
            at.translation = track(x - 2.0 * COUPLING, y) + PASSENGER / UNIT;
        }
    }
    if event_seen.take() == Some(Event::Stopped) {
        info!("intro: the train has stopped");
    }
    if action == Action::WaitStopped && arrival.stage == Stage::Riding {
        arrival.stage = Stage::Open { seconds: 0.0 };
    }
}

/// The camera holds still on the doorway (`Camera2_request_main_demo_fromNowPos2`).
fn camera(
    arrival: Res<Arrival>,
    town: Res<crate::town::Town>,
    mut cameras: Query<
        &mut Transform,
        (
            With<crate::player::WorldCamera>,
            Without<crate::msg_window::OverlayCamera>,
        ),
    >,
) {
    if !on_platform(arrival.stage) {
        return;
    }
    let Ok(mut at) = cameras.single_mut() else {
        return;
    };
    let w = world(CAMERA_CENTRE.x, CAMERA_CENTRE.y);
    let ground = town.ground_y(Vec3::new(w.x, 0.0, w.y));
    let centre = Vec3::new(w.x, ground - 35.0 / UNIT, w.y);
    let pitch = CAMERA_PITCH.to_radians();
    at.translation = centre + Vec3::new(0.0, pitch.sin(), pitch.cos()) * CAMERA_DISTANCE;
    at.look_at(centre, Vec3::Y);
}

/// Nothing takes input while the train comes in.
fn lock(arrival: Res<Arrival>, mut focus: ResMut<InputFocus>) {
    if matches!(
        arrival.stage,
        Stage::Riding | Stage::Open { .. } | Stage::GetOff { .. } | Stage::Turn | Stage::Walk
    ) {
        *focus = InputFocus::Transition;
    }
}

/// Whether the player is still in the arrival's cut scene (the camera holds, the train is shown).
fn on_platform(stage: Stage) -> bool {
    !matches!(stage, Stage::Idle | Stage::Leaving)
}

/// Door open, stepping off, Porter's greeting, the walk (`aID_ACT_RIDE_OFF_PLAYER` .. `WALK_ONE_UNIT`).
/// Until Nook's welcome is built the game then goes on as a played one (`finish_intro`).
#[allow(clippy::too_many_arguments)]
fn scene(
    time: Res<Time>,
    disc: Option<Res<GameDisc>>,
    town: Res<crate::town::Town>,
    mut arrival: ResMut<Arrival>,
    mut intro: ResMut<Intro>,
    mut dialogue: ResMut<DialogueState>,
    mut identity: ResMut<crate::save::PlayerIdentity>,
    mut inventory: ResMut<crate::interaction::PlayerInventory>,
    mut player: Query<(&mut Transform, &mut crate::interaction::Facing), With<Player>>,
    mut porter: Query<(Entity, &mut Transform), (With<Porter>, Without<Player>)>,
) {
    let frames = time.delta_secs() * 60.0;
    let Ok((mut at, mut facing)) = player.single_mut() else {
        return;
    };
    let ground = |p: Vec2| {
        let w = world(p.x, p.y);
        town.ground_y(Vec3::new(w.x, 0.0, w.y))
    };
    match arrival.stage {
        // The door has been open a moment.
        Stage::Open { seconds } => {
            let seconds = seconds + time.delta_secs();
            arrival.stage = if seconds < 1.0 {
                Stage::Open { seconds }
            } else {
                Stage::GetOff { frames: 0.0 }
            };
        }
        Stage::GetOff { frames: done } => {
            let done = done + frames;
            let t = (done / 45.0).min(1.0);
            let off = world(OFF_POS.x, OFF_POS.y);
            let to = Vec3::new(off.x, ground(OFF_POS), off.y);
            at.translation = arrival.from.lerp(to, t);
            facing.dir = (to - arrival.from).with_y(0.0).normalize_or_zero();
            arrival.stage = if t >= 1.0 {
                Stage::Turn
            } else {
                Stage::GetOff { frames: done }
            };
        }
        Stage::Turn => {
            let Ok((porter, mut pat)) = porter.single_mut() else {
                // No station master (no station building): carry on.
                arrival.stage = Stage::Walk;
                return;
            };
            // `chase_angle(.., 0x400)` per frame towards the player.
            let to = (at.translation - pat.translation).with_y(0.0);
            let want = to.x.atan2(to.z);
            let fwd = pat.rotation * Vec3::Z;
            let yaw = fwd.x.atan2(fwd.z);
            let diff = (want - yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            let step = 0x400 as f32 / 65536.0 * std::f32::consts::TAU * frames;
            if diff.abs() > step {
                pat.rotation = Quat::from_rotation_y(yaw + diff.signum() * step);
                return;
            }
            pat.rotation = Quat::from_rotation_y(want);
            facing.dir = -to.normalize_or_zero();
            if let Some(text) = disc.as_ref().and_then(|d| d.text.as_ref()) {
                let mut ctx = crate::intro_train::context(&identity);
                ctx.talk_name = "Porter".into();
                let (pages, choice) = text.talk_asking(PORTER_GREETING, &ctx, &mut |_| 0);
                dialogue.begin_special(
                    porter,
                    "Porter",
                    (PORTER_NPC, PORTER_SOUND),
                    pages,
                    choice.as_ref().map(|(_, c)| c.labels.clone()),
                );
            }
            arrival.stage = Stage::Talk;
        }
        Stage::Talk => {
            if !dialogue.active {
                // `train_control_state` changes: the train may go.
                if let Some(train) = arrival.train.as_mut() {
                    train.depart = true;
                }
                arrival.stage = Stage::Walk;
            }
        }
        Stage::Walk => {
            let goal = world(WALK_POS.x, WALK_POS.y);
            let here = Vec2::new(at.translation.x, at.translation.z);
            let to = goal - here;
            let step = WALK_SPEED * 60.0 / UNIT * time.delta_secs();
            facing.dir = Vec3::new(to.x, 0.0, to.y).normalize_or_zero();
            if to.length() <= step {
                at.translation = Vec3::new(goal.x, ground(WALK_POS), goal.y);
                arrival.stage = Stage::Idle;
                *intro = Intro::Off;
                crate::intro::finish_intro(&mut identity, &mut inventory);
                info!("intro: arrived; the game goes on");
            } else {
                let next = here + to.normalize() * step;
                at.translation = Vec3::new(next.x, ground(WALK_POS), next.y);
            }
        }
        Stage::Idle | Stage::Leaving | Stage::Riding => {}
    }
}

/// The train that has pulled out goes (`mTRC` state 4), and the arrival music fades once the player
/// has left the station (`aSTM_talk_wait`).
fn tidy(
    mut arrival: ResMut<Arrival>,
    mut commands: Commands,
    parts: Query<Entity, Or<(With<Locomotive>, With<Tender>, With<Car>)>>,
    player: Query<&Transform, With<Player>>,
) {
    if arrival.train.is_some_and(|t| t.action == Action::None) {
        arrival.train = None;
        for entity in &parts {
            commands.entity(entity).despawn();
        }
    }
    if arrival.stage == Stage::Idle && music_on() {
        if let Ok(at) = player.single() {
            if at.translation.z >= world(0.0, OUT_Z).y {
                MUSIC.store(false, Ordering::Relaxed);
            }
        }
    }
}

pub struct IntroArrivalPlugin;

impl Plugin for IntroArrivalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Arrival>()
            .add_systems(PreUpdate, lock.after(crate::input::update_focus))
            .add_systems(
                Update,
                (leave, run_train, scene, tidy)
                    .chain()
                    .in_set(crate::states::GameplaySet),
            )
            .add_systems(
                Update,
                camera
                    .after(crate::player::camera_follow)
                    .in_set(crate::states::GameplaySet),
            );
    }
}
