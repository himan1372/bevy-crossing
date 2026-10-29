//! The new-game intro's first scene (`SCENE_START_DEMO`, `data/scene/start_demo1.c`): the train
//! interior (`rom_train_in_model`), Rover (`SP_NPC_GUIDE`) on the seat and the sleeping old woman
//! (`SP_NPC_SLEEP_OBABA`), with the player standing at (120, 0, 340).
use bevy::prelude::*;

use crate::dialogue::DialogueState;
use crate::game_disc::GameDisc;
use crate::indoor_room::IndoorRoom;
use crate::input::{Actions, InputFocus};
use crate::interaction::Facing;
use crate::intro::Intro;
use crate::locomotion::Locomotion;
use crate::messages::{MsgChoice, MsgContext};
use crate::n64_material::N64Material;
use crate::player::Player;
use crate::shop::ShopLocation;
use crate::states::GameplaySet;

pub const TRAIN_ORIGIN: Vec3 = Vec3::new(6000.0, 0.0, 0.0);
const UNIT: f32 = 40.0;
/// `START_DEMO1_player_data`.
const PLAYER_START: Vec2 = Vec2::new(120.0, 340.0);
/// `ALL_NPC_NUM + (SP_NPC_GUIDE - SP_NPC_START)`.
const ROVER_NPC: u32 = 238 + 4;

fn local(p: Vec2) -> Vec3 {
    TRAIN_ORIGIN + Vec3::new(p.x / UNIT, 0.0, p.y / UNIT)
}

#[derive(Resource, Default)]
struct TrainRoom {
    room: Option<IndoorRoom>,
    built: bool,
}

#[derive(Component)]
struct Rover;

/// Rover's sound id (`l_sp_actor_name`: 2, like Nook's) and name.
const ROVER_SOUND: u8 = 2;
/// `aNGD_approach`: Rover stops at z = 290.
const APPROACH_Z: f32 = 290.0;
/// `aNGD_set_walk_spd`: at most 1 game unit per frame.
const WALK_SPEED: f32 = 1.0;
/// `aNGD_talk_start_wait`'s first message ("Excuse me... do you have a second?").
const FIRST_MESSAGE: usize = 0x2AC6;
/// `mDemo_ORDER_NPC0` as the message codes raise it, and the index Rover's script waits on.
const ORDER_NPC0: u8 = 1;
const ORDER_STEP: u8 = 9;

/// Where Rover's part of the scene has got to (`aNGD_ACTION_*`).
#[derive(Default)]
enum Step {
    #[default]
    Walk,
    Turn,
    /// A message is up; the question it ends on, if any.
    Talk(Option<(usize, MsgChoice)>),
    /// The script has asked for the player's name (order 9 = 1).
    NameEntry,
}

#[derive(Resource, Default)]
struct Script {
    step: Step,
}

#[allow(clippy::too_many_arguments)]
fn build_room(
    mut commands: Commands,
    disc: Res<GameDisc>,
    intro: Res<Intro>,
    mut room: ResMut<TrainRoom>,
    mut next: ResMut<NextState<ShopLocation>>,
    mut player: Query<(&mut Transform, &mut Facing), With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    if room.built || *intro != Intro::Train {
        return;
    }
    let Ok((mut transform, mut facing)) = player.single_mut() else {
        return;
    };
    room.built = true;
    room.room = IndoorRoom::build(
        &mut commands,
        &disc,
        "train room",
        TRAIN_ORIGIN,
        ("rom_train_in_model", Some("rom_train_in_modelT")),
        (&mut meshes, &mut materials, &mut images),
    );
    if room.room.is_none() {
        warn!("train room missing from the disc");
        return;
    }
    // `start_demo1_actable`: Rover starts at unit (3, 3), the door end of the car.
    commands.spawn((
        Name::new("Rover"),
        Rover,
        crate::character::NpcGait::default(),
        crate::character::NpcModel(ROVER_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(local(Vec2::new(140.0, 140.0)) + Vec3::Y)
            .looking_to(Vec3::Z, Vec3::Y),
    ));
    transform.translation = local(PLAYER_START) + Vec3::Y;
    facing.dir = Vec3::NEG_Z;
    next.set(ShopLocation::Train);
    info!("intro: on the train");
}

fn movement(
    actions: Res<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    time: Res<Time>,
    room: Res<TrainRoom>,
    mut player: Query<(&mut Transform, &mut Facing, &mut Locomotion), With<Player>>,
) {
    if *state != ShopLocation::Train || !focus.is_world() {
        return;
    }
    let (Ok((mut transform, mut facing, mut loco)), Some(room)) =
        (player.single_mut(), room.room.as_ref())
    else {
        return;
    };
    room.walk(
        &actions,
        time.delta_secs(),
        (&mut transform, &mut facing, &mut loco),
    );
}

fn context(identity: &crate::save::PlayerIdentity) -> MsgContext {
    let now = crate::clock::now();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Rover".into(),
        town_name: identity.town.clone(),
        free: vec![String::new(); 20],
        items: vec![String::new(); 5],
        item_articles: vec![0; 5],
        year: now.year,
        month: now.month,
        day: now.day,
        hour: now.hour,
        minute: now.minute,
        weekday: now.weekday,
        ..default()
    }
}

/// Starts message `msg` (and what follows it up to a question) and returns the question.
fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    rover: Entity,
    ctx: &MsgContext,
    msg: usize,
) -> Option<(usize, MsgChoice)> {
    let text = disc.text.as_ref()?;
    let (pages, choice) = text.talk_asking(msg, ctx, &mut |_| 0);
    dialogue.begin_special(
        rover,
        "Rover",
        (ROVER_NPC, ROVER_SOUND),
        pages,
        choice.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("train message {msg:#06x}");
    choice
}

/// `aNGD_enter` .. `aNGD_talk_start_wait` .. the questions: Rover walks to the player, turns,
/// and talks; each answer leads to the message the source's choice codes name.
#[allow(clippy::too_many_arguments)]
fn rover(
    disc: Res<GameDisc>,
    state: Res<State<ShopLocation>>,
    time: Res<Time>,
    identity: Res<crate::save::PlayerIdentity>,
    mut focus: ResMut<InputFocus>,
    mut script: ResMut<Script>,
    mut dialogue: ResMut<DialogueState>,
    mut orders: ResMut<crate::msg_window::DemoOrders>,
    mut rover: Query<(Entity, &mut Transform, &mut crate::character::NpcGait), With<Rover>>,
    mut player: Query<(&Transform, &mut Facing), (With<Player>, Without<Rover>)>,
) {
    if *state != ShopLocation::Train {
        return;
    }
    let (Ok((entity, mut at, mut gait)), Ok((player_at, mut facing))) =
        (rover.single_mut(), player.single_mut())
    else {
        return;
    };
    let frames = time.delta_secs() * 60.0;
    match &mut script.step {
        Step::Walk => {
            at.translation.z += WALK_SPEED * frames / UNIT;
            gait.0 = 1;
            if (at.translation.z - TRAIN_ORIGIN.z) * UNIT >= APPROACH_Z {
                at.translation.z = TRAIN_ORIGIN.z + APPROACH_Z / UNIT;
                gait.0 = 0;
                script.step = Step::Turn;
            }
        }
        Step::Turn => {
            // `chase_angle(.., 0x400)` per frame towards the player.
            let to = (player_at.translation - at.translation).with_y(0.0);
            let want = to.x.atan2(to.z);
            let yaw = (at.rotation * Vec3::Z).x.atan2((at.rotation * Vec3::Z).z);
            let diff = (want - yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            let step = 0x400 as f32 / 65536.0 * std::f32::consts::TAU * frames;
            if diff.abs() <= step {
                at.rotation = Quat::from_rotation_y(want);
                facing.dir = -to.normalize_or_zero();
                *focus = InputFocus::Dialogue;
                let choice = say(
                    &disc,
                    &mut dialogue,
                    entity,
                    &context(&identity),
                    FIRST_MESSAGE,
                );
                script.step = Step::Talk(choice);
            } else {
                at.rotation = Quat::from_rotation_y(yaw + diff.signum() * step);
            }
        }
        Step::Talk(choice) => {
            for (kind, index, value) in orders.take_raised() {
                if (kind, index) == (ORDER_NPC0, ORDER_STEP) && value == 1 {
                    info!("the script asks for the player's name");
                    script.step = Step::NameEntry;
                    return;
                }
            }
            let Some(answer) = dialogue.external_result.take() else {
                return;
            };
            let next = choice
                .as_ref()
                .and_then(|(_, c)| c.next.get(answer).copied().flatten());
            match next {
                Some(msg) => {
                    let choice = say(&disc, &mut dialogue, entity, &context(&identity), msg);
                    script.step = Step::Talk(choice);
                }
                None => {
                    *dialogue = DialogueState::default();
                    *focus = InputFocus::World;
                }
            }
        }
        Step::NameEntry => {}
    }
}

pub struct IntroTrainPlugin;

impl Plugin for IntroTrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrainRoom>()
            .init_resource::<Script>()
            .add_systems(
                Update,
                build_room
                    .run_if(resource_exists::<GameDisc>)
                    .run_if(resource_exists::<Intro>)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                movement
                    .in_set(crate::locomotion::MovementSet)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                rover
                    .in_set(crate::input::InteractSet::Talk)
                    .in_set(GameplaySet),
            );
    }
}
