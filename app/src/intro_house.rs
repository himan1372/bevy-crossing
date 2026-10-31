//! `ac_intro_demo` / `ac_npc_rcn_guide`: welcome, home tour and initial payment.
//! Original dialogue and choice branches remain on the disc. Ownership is
//! committed after the quest Bell bag is given, never by inspecting a door.
use std::sync::atomic::{AtomicU8, Ordering};

use bevy::prelude::*;

use crate::character::{NpcClip, NpcGait, NpcModel, PlayerLook};
use crate::dialogue::DialogueState;
use crate::game_disc::GameDisc;
use crate::house::HouseState;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{DoorPanel, Facing, PlayerInventory};
use crate::intro::Intro;
use crate::locomotion::Locomotion;
use crate::messages::MsgChoice;
use crate::msg_window::DemoOrders;
use crate::player::Player;
use crate::pockets::PocketsUi;
use crate::save::PlayerIdentity;
use crate::shop_nook::handover::{HandOver, Item, Party, Request};
use crate::town::Town;

const NOOK: u32 = 238 + 20;
const MONEY: u16 = 0x2100;
const RUN_SPEED: f32 = crate::npc::RUN_SPEED;
static MUSIC: AtomicU8 = AtomicU8::new(0);

pub(crate) fn music() -> Option<rustimal_logic::scene_music::Scene> {
    match MUSIC.load(Ordering::Relaxed) {
        1 => Some(rustimal_logic::scene_music::Scene::IntroGuide),
        2 => Some(rustimal_logic::scene_music::Scene::IntroSelectHouse),
        _ => None,
    }
}

#[derive(Component)]
pub(crate) struct NookGuide;

#[derive(Clone, Copy, Default, PartialEq)]
enum Phase {
    #[default]
    AwaitExit,
    Talk(After),
    Approach,
    Tour(u8),
    Choosing,
    Entering(usize),
    Inside(usize),
    Payment,
    Transfer,
    Depart(u8),
    Done,
}

#[derive(Clone, Copy, PartialEq)]
enum After {
    Call,
    Introduce,
    Explain,
    Inspect(usize),
    Offer(usize),
    Reminder,
    Paid,
}

#[derive(Resource, Default)]
struct HouseIntro {
    phase: Phase,
    guide: Option<Entity>,
    corner: Vec2,
    question: Option<MsgChoice>,
    message: usize,
    answer: String,
    selected: usize,
    inspect_request: Option<usize>,
    talk_request: bool,
}

fn turn(from: f32, target: f32, frames: f32) -> (f32, bool) {
    let diff = (target - from + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    let unit = std::f32::consts::TAU / 65536.0;
    let step = (diff.abs() * (1.0 - 0.5_f32.sqrt().powf(frames)))
        .clamp(50.0 * unit * frames, 2500.0 * unit * frames)
        .min(diff.abs());
    (from + step * diff.signum(), step >= diff.abs())
}

fn face(at: &mut Transform, to: Vec3, dt: f32) {
    let delta = (to - at.translation).with_y(0.0);
    if delta.length_squared() > 0.0001 {
        let fwd = at.rotation * Vec3::Z;
        let (yaw, _) = turn(fwd.x.atan2(fwd.z), delta.x.atan2(delta.z), dt * 60.0);
        at.rotation = Quat::from_rotation_y(yaw);
    }
}

fn walk(at: &mut Transform, goal: Vec2, speed: f32, dt: f32, town: &Town) -> bool {
    face(at, Vec3::new(goal.x, at.translation.y, goal.y), dt);
    advance(at, goal, speed, dt, town)
}

/// Movement only: the player's child rig is rotated separately from Facing.
fn advance(at: &mut Transform, goal: Vec2, speed: f32, dt: f32, town: &Town) -> bool {
    let delta = goal - Vec2::new(at.translation.x, at.translation.z);
    let step = speed * dt;
    let next = if delta.length() <= step {
        goal
    } else {
        Vec2::new(at.translation.x, at.translation.z) + delta.normalize_or_zero() * step
    };
    let test = at.translation.with_x(next.x);
    if town.can_step(at.translation, test) {
        at.translation.x = next.x;
    }
    let test = at.translation.with_z(next.y);
    if town.can_step(at.translation, test) {
        at.translation.z = next.y;
    }
    at.translation.y = town.ground_y(at.translation) + 1.0;
    (goal - Vec2::new(at.translation.x, at.translation.z)).length() < 0.01
}

fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    orders: &mut DemoOrders,
    flow: &mut HouseIntro,
    world: (&PlayerIdentity, &Town),
    message: usize,
    after: After,
) {
    let (identity, town) = world;
    let (Some(text), Some(guide)) = (disc.text.as_ref(), flow.guide) else {
        return;
    };
    let mut ctx = crate::intro_train::context(identity);
    ctx.talk_name = "Tom Nook".into();
    ctx.determination.clone_from(&flow.answer);
    ctx.free.resize(3, String::new());
    if let Some((ax, az, ..)) = town
        .retail
        .as_ref()
        .and_then(|r| r.buildings.iter().find(|b| b.item == 0x5804))
        .and_then(|b| Town::cell_indices_of(b.origin))
    {
        ctx.free[1] = ['A', 'B', 'C', 'D', 'E', 'F'][az.min(5)].to_string();
        ctx.free[2] = (ax + 1).to_string();
    }
    // 081E and the cancel reminder must stop before 07EC: the inventory and
    // hand-over run at NPC0[9]=1 before that next message is allowed to appear.
    let stop = if message == 0x07EB { 0x07EB } else { 0x081E };
    let (pages, choice) = text.talk_until(message, &ctx, &mut |_| 0, Some(stop));
    dialogue.begin_special(
        guide,
        "Tom Nook",
        (NOOK, 2),
        pages,
        choice.as_ref().map(|(_, c)| c.labels.clone()),
    );
    flow.question = choice.map(|(_, c)| c);
    flow.message = message;
    flow.phase = Phase::Talk(after);
    orders.clear(1, 9);
    info!("intro Nook message {message:#06x}");
}

fn lock(flow: Res<HouseIntro>, mut focus: ResMut<InputFocus>) {
    if matches!(
        flow.phase,
        Phase::Approach | Phase::Tour(_) | Phase::Depart(_)
    ) {
        *focus = InputFocus::Transition;
    }
}

fn choose_input(
    mut flow: ResMut<HouseIntro>,
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    town: Res<Town>,
    player: Query<(&Transform, &Facing), With<Player>>,
    nook: Query<&Transform, With<NookGuide>>,
) {
    if flow.phase != Phase::Choosing || !focus.is_world() || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    for slot in 0..4 {
        let Some(door) = crate::house::retail_home_door(&town, slot) else {
            continue;
        };
        let back = Quat::from_rotation_y(-door.door_yaw);
        let panel = DoorPanel {
            center_x: 0.0,
            half_w: 0.6,
            face_z: -0.8,
            into_z: -1.0,
        };
        if panel.aimed_at(
            back * (at.translation - door.door_pos),
            back * facing.dir,
            1.25,
        ) {
            actions.consume(Action::Interact);
            flow.inspect_request = Some(slot);
            return;
        }
    }
    if let Ok(nook) = nook.single() {
        let delta = (nook.translation - at.translation).with_y(0.0);
        if delta.length() < 2.5 && delta.normalize_or_zero().dot(facing.dir) > 0.3 {
            actions.consume(Action::Interact);
            flow.talk_request = true;
        }
    }
}

#[allow(clippy::too_many_arguments)] // One owner commits scene, dialogue and inventory transitions.
fn run(
    mut commands: Commands,
    time: Res<Time>,
    disc: Option<Res<GameDisc>>,
    town: Res<Town>,
    mut flow: ResMut<HouseIntro>,
    mut intro: ResMut<Intro>,
    arrival: Res<crate::intro_arrival::Arrival>,
    (state, mut next, wipe): (
        Res<State<HouseState>>,
        ResMut<NextState<HouseState>>,
        Res<crate::scene_wipe::SceneWipe>,
    ),
    (mut identity, mut inventory, mut home, mut catalog, look): (
        ResMut<PlayerIdentity>,
        ResMut<PlayerInventory>,
        ResMut<crate::home::PlayerHouse>,
        ResMut<crate::catalog::Catalog>,
        Res<PlayerLook>,
    ),
    (mut dialogue, mut orders, mut pockets, mut handover): (
        ResMut<DialogueState>,
        ResMut<DemoOrders>,
        ResMut<PocketsUi>,
        ResMut<HandOver>,
    ),
    mut nook: Query<
        (Entity, &mut Transform, &mut NpcGait, &mut Visibility),
        (With<NookGuide>, Without<Player>),
    >,
    mut player: Query<
        (&mut Transform, &mut Facing, &mut Locomotion),
        (With<Player>, Without<NookGuide>),
    >,
) {
    let Ok((mut at, mut facing, mut loco)) = player.single_mut() else {
        return;
    };
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        if matches!(*intro, Intro::Arrival | Intro::Houses) {
            crate::intro::complete_intro(&mut identity, &mut inventory);
            *intro = Intro::Off;
            flow.phase = Phase::Done;
            if let Some(guide) = flow.guide.take() {
                commands.entity(guide).try_despawn();
            }
            pockets.giving = None;
            MUSIC.store(0, Ordering::Relaxed);
        }
        return;
    };
    // Loading an already-played save abandons the unfinished tour and its actor.
    if *intro == Intro::Houses && identity.intro_done {
        if let Some(guide) = flow.guide.take() {
            commands.entity(guide).try_despawn();
        }
        flow.phase = Phase::Done;
        pockets.giving = None;
        MUSIC.store(0, Ordering::Relaxed);
        *intro = Intro::Off;
        return;
    }
    let dt = time.delta_secs();
    if flow.phase == Phase::AwaitExit && *intro == Intro::Arrival && arrival.ready_for_welcome() {
        let fast = std::env::var_os("BEVY_CROSSING_INTRO_HOUSES").is_some();
        if fast {
            at.translation = Vec3::new(2.0, 0.0, 970.0 / 40.0 - 63.6);
            at.translation.y = town.ground_y(at.translation) + 1.0;
        }
        if at.translation.z < 970.0 / 40.0 - 63.6 || !wipe.clear() {
            return;
        }
        let Some((ax, az, ..)) = town
            .retail
            .as_ref()
            .and_then(|r| r.player_homes.iter().flatten().next())
            .copied()
        else {
            crate::intro::complete_intro(&mut identity, &mut inventory);
            *intro = Intro::Off;
            return;
        };
        let corner = Town::cell_to_world(ax, az, 0, 0) - Vec3::splat(0.5);
        flow.corner = Vec2::new(corner.x, corner.z);
        let mut spawn = Town::cell_to_world(ax, az.saturating_sub(1), 8, 15);
        spawn.y = town.ground_y(spawn) + 1.0;
        let guide = commands
            .spawn((
                Name::new("intro Tom Nook"),
                NookGuide,
                NpcModel(NOOK),
                NpcClip::default(),
                NpcGait::default(),
                Transform::from_translation(spawn),
                Visibility::default(),
            ))
            .id();
        flow.guide = Some(guide);
        *intro = Intro::Houses;
        *loco = Locomotion::default();
        MUSIC.store(1, Ordering::Relaxed);
        say(
            &disc,
            &mut dialogue,
            &mut orders,
            &mut flow,
            (&identity, &town),
            0x07DE,
            After::Call,
        );
        return;
    }
    if *intro != Intro::Houses {
        return;
    }
    let Ok((guide, mut nat, mut gait, mut visibility)) = nook.single_mut() else {
        return;
    };
    if !matches!(flow.phase, Phase::Inside(_) | Phase::Entering(_)) {
        *visibility = Visibility::Inherited;
    }
    match flow.phase {
        Phase::Talk(after) => {
            gait.0 = 0;
            *loco = Locomotion::default();
            face(&mut nat, at.translation, dt);
            let delta = (nat.translation - at.translation).with_y(0.0);
            let (yaw, _) = turn(
                facing.dir.x.atan2(facing.dir.z),
                delta.x.atan2(delta.z),
                dt * 60.0,
            );
            facing.dir = Quat::from_rotation_y(yaw) * Vec3::Z;
            if matches!(after, After::Offer(_)) && orders.get(1, 9) == 1 {
                orders.clear(1, 9);
                *dialogue = DialogueState::default();
                *pockets = PocketsUi {
                    open: true,
                    giving: Some(MONEY),
                    selected: inventory.pockets.find_item_with_cond(MONEY, 2).unwrap_or(0),
                    ..default()
                };
                flow.phase = Phase::Payment;
                info!("intro: select the starting Bell bag");
                return;
            }
            let Some(answer) = dialogue.external_result.take() else {
                return;
            };
            if let Some(question) = flow.question.take() {
                let chosen = answer.min(question.labels.len().saturating_sub(1));
                flow.answer = question.labels.get(chosen).cloned().unwrap_or_default();
                if let Some(message) = question.next.get(chosen).copied().flatten() {
                    say(
                        &disc,
                        &mut dialogue,
                        &mut orders,
                        &mut flow,
                        (&identity, &town),
                        message,
                        after,
                    );
                    return;
                }
            }
            *dialogue = DialogueState::default();
            match after {
                After::Call => flow.phase = Phase::Approach,
                After::Introduce => flow.phase = Phase::Tour(0),
                After::Explain | After::Reminder | After::Offer(_) => {
                    MUSIC.store(2, Ordering::Relaxed);
                    flow.phase = Phase::Choosing;
                    info!("intro: choosing a house");
                }
                After::Inspect(slot) => {
                    crate::house::enter_intro_home(&mut commands, &mut at, &mut facing);
                    next.set(HouseState::Inside);
                    flow.phase = Phase::Entering(slot);
                }
                // aNRG_exit_turn_init: leave through the nearer end of the
                // acre (north of game Z 1540, otherwise south). Crossing the
                // whole acre from a southern home runs into the notice board.
                After::Paid => {
                    flow.phase = Phase::Depart(if nat.translation.z < flow.corner.y + 6.5 {
                        0
                    } else {
                        2
                    });
                }
            }
        }
        Phase::Approach => {
            gait.0 = 2;
            *loco = Locomotion::default();
            let goal = Vec2::new(at.translation.x, at.translation.z + 70.0 / 40.0);
            if walk(&mut nat, goal, RUN_SPEED, dt, &town) {
                gait.0 = 0;
                say(
                    &disc,
                    &mut dialogue,
                    &mut orders,
                    &mut flow,
                    (&identity, &town),
                    0x07DF,
                    After::Introduce,
                );
            }
        }
        Phase::Tour(path) => {
            let goal = flow.corner + Vec2::new(8.0, if path == 0 { 0.5 } else { 5.5 });
            gait.0 = 2;
            let done = walk(&mut nat, goal, RUN_SPEED, dt, &town);
            let delta = (nat.translation - at.translation).with_y(0.0);
            let speed = RUN_SPEED * if delta.length() < 2.0 { 0.6 } else { 1.0 };
            if delta.length() > 1.0 {
                let target = Vec2::new(nat.translation.x, nat.translation.z);
                let target = target - Vec2::new(delta.x, delta.z).normalize_or_zero() * 1.0;
                let before = at.translation;
                advance(&mut at, target, speed, dt, &town);
                at.rotation = Quat::IDENTITY;
                let (yaw, _) = turn(
                    facing.dir.x.atan2(facing.dir.z),
                    delta.x.atan2(delta.z),
                    dt * 60.0,
                );
                facing.dir = Quat::from_rotation_y(yaw) * Vec3::Z;
                if at.translation != before {
                    loco.0.state = rustimal_logic::player_move::LocomotionState::Walk;
                    loco.0.speed = speed / crate::locomotion::FULL_TILT_SPEED;
                } else {
                    *loco = Locomotion::default();
                }
            } else {
                *loco = Locomotion::default();
            }
            if done {
                if path == 0 {
                    flow.phase = Phase::Tour(1);
                } else {
                    gait.0 = 0;
                    *loco = Locomotion::default();
                    say(
                        &disc,
                        &mut dialogue,
                        &mut orders,
                        &mut flow,
                        (&identity, &town),
                        0x07E1,
                        After::Explain,
                    );
                    info!("intro: Nook showed the houses");
                }
            }
        }
        Phase::Choosing => {
            if let Some(slot) = flow.inspect_request.take() {
                flow.selected = slot;
                say(
                    &disc,
                    &mut dialogue,
                    &mut orders,
                    &mut flow,
                    (&identity, &town),
                    0x07E4,
                    After::Inspect(slot),
                );
                info!("intro: inspect house {slot}");
            } else if flow.talk_request {
                flow.talk_request = false;
                say(
                    &disc,
                    &mut dialogue,
                    &mut orders,
                    &mut flow,
                    (&identity, &town),
                    0x0820,
                    After::Reminder,
                );
            } else {
                let p = Vec2::new(at.translation.x, at.translation.z);
                let clamped = p.clamp(
                    flow.corner + Vec2::splat(0.45),
                    flow.corner + Vec2::splat(15.55),
                );
                if p != clamped {
                    at.translation.x = clamped.x;
                    at.translation.z = clamped.y;
                    *loco = Locomotion::default();
                    say(
                        &disc,
                        &mut dialogue,
                        &mut orders,
                        &mut flow,
                        (&identity, &town),
                        0x07E2,
                        After::Reminder,
                    );
                }
            }
        }
        Phase::Entering(slot) => {
            if *state == HouseState::Inside && wipe.clear() {
                *visibility = Visibility::Hidden;
                MUSIC.store(0, Ordering::Relaxed);
                flow.phase = Phase::Inside(slot);
            }
        }
        Phase::Inside(slot) => {
            *visibility = Visibility::Hidden;
            if *state == HouseState::Town && wipe.clear() {
                let door = crate::house::retail_home_door(&town, slot);
                let outward = door
                    .as_ref()
                    .map_or(Vec3::Z, |d| Quat::from_rotation_y(d.door_yaw) * Vec3::Z);
                nat.translation = at.translation + outward * 1.75;
                nat.translation.y = town.ground_y(nat.translation) + 1.0;
                *visibility = Visibility::Inherited;
                MUSIC.store(2, Ordering::Relaxed);
                say(
                    &disc,
                    &mut dialogue,
                    &mut orders,
                    &mut flow,
                    (&identity, &town),
                    0x07E6,
                    After::Offer(slot),
                );
            }
        }
        Phase::Payment => {
            if pockets.open {
                return;
            }
            let paid = pockets.sell_slot.take().is_some_and(|slot| {
                rustimal_logic::intro::take_starting_bells(&mut inventory.pockets, slot)
            });
            pockets.giving = None;
            if paid {
                handover.start(Request {
                    item: Item::Money,
                    giver: Party::Player,
                    taker: Party::Npc(guide),
                });
                flow.phase = Phase::Transfer;
                info!("intro: starting Bell bag handed to Nook");
            } else {
                let selected = flow.selected;
                say(
                    &disc,
                    &mut dialogue,
                    &mut orders,
                    &mut flow,
                    (&identity, &town),
                    0x07EB,
                    After::Offer(selected),
                );
            }
        }
        Phase::Transfer => {
            if !handover.busy() {
                orders.set(1, 1, 3);
                say(
                    &disc,
                    &mut dialogue,
                    &mut orders,
                    &mut flow,
                    (&identity, &town),
                    0x07EC,
                    After::Paid,
                );
            }
        }
        Phase::Depart(path) => {
            *loco = Locomotion::default();
            gait.0 = 2;
            // aNRG_exit: game Z 1300/1220 or 1900/1980, X 2240.
            let z = [0.5, -1.5, 15.5, 17.5][usize::from(path)];
            let goal = flow.corner + Vec2::new(8.0, z);
            if walk(&mut nat, goal, RUN_SPEED, dt, &town) {
                if path.is_multiple_of(2) {
                    flow.phase = Phase::Depart(path + 1);
                } else {
                    crate::home::select(&mut home, flow.selected);
                    crate::intro::complete_intro(&mut identity, &mut inventory);
                    catalog.collection.set(0x2400 + look.cloth as u16);
                    if let Some(bytes) = disc
                        .catalog
                        .get("l_mHm_player_room_default_data")
                        .and_then(|s| disc.rom.slice(s.source, s.offset, 16))
                    {
                        catalog.collection.set(0x2600 + u16::from(bytes[1]));
                        catalog.collection.set(0x2700 + u16::from(bytes[0]));
                        catalog
                            .collection
                            .set(u16::from_be_bytes([bytes[12], bytes[13]]));
                    }
                    commands.entity(guide).despawn();
                    flow.guide = None;
                    flow.phase = Phase::Done;
                    *intro = Intro::Off;
                    MUSIC.store(0, Ordering::Relaxed);
                    info!(
                        "intro: house {} chosen; the part-time job starts",
                        flow.selected
                    );
                }
            }
        }
        Phase::AwaitExit | Phase::Done => {}
    }
}

pub(crate) struct IntroHousePlugin;
impl Plugin for IntroHousePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HouseIntro>()
            .add_systems(PreUpdate, lock.after(crate::input::update_focus))
            .add_systems(
                Update,
                choose_input
                    .in_set(InteractSet::Doors)
                    .in_set(crate::states::GameplaySet),
            )
            .add_systems(
                Update,
                run.after(choose_input)
                    .after(crate::locomotion::MovementSet)
                    .before(crate::character::CharacterAnimationSet)
                    .in_set(crate::states::GameplaySet),
            );
    }
}
