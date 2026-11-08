//! Gracie (`SP_NPC_DESIGNER`), the fashion designer with the car-wash game.
//!
//! She is out while the original scheduler says `mEv_EVENT_DESIGNER` is active (06:00 on her day until 05:59 the next
//! morning; `event_schedule` with the special-event calendar), on free ground like the other visitors.
//!
//! Talking (`ac_ev_designer_talk.c_inc`, kernel `event_visitors::gracie_msg`): the message table depends on whether
//! the shirt the player wears is on the event cloth list (`wear_flag`). After the game this visit she says
//! `21 + RANDOM(3)`; a player who has her gift says `17` (an event piece) or `18`; otherwise `8` when the event was
//! completed already, else `RANDOM(4)` (+4 for a girl): the offer, which ends in the pocket check: message `19` when a
//! pocket is free, and the game starts: press the action button 100 times in 18 seconds (`aEDSN_game_end_wait`). Done with more than 6
//! seconds left is result 0 (a piece of the event cloth list, not one given before), in the last 6 seconds result 1 (the
//! common cloth list), out of time result 2 (nothing). Then the forced talk `25` and `12` / `14` / `16`; a piece is
//! handed over into the first free pocket and recorded with the event's save (`mEv_designer_c`, kept as
//! [`DesignerEvent`]); `mEv_SPECL_DESIGNER_COMPLETE` goes on when the game is over.
//!
//! Her car (`ac_s_car.c`, `DESIGNER_CAR`) stands two cells north and one west of her
//! (`aEDSN_think_init_proc`: she is placed at the car + (1, 2) cells). It is `s_car_DL_model`, drawn with the primitive
//! colour `remaining * (150, 120, 100) + done * 255` per channel (`aSCR_set_prim_col`, `done` being the presses over
//! 100), so it turns from dull to white as it is washed; the wash stays on it while it is there.
//!
//! Not built: the car's ground offsets and shadow, its sparkles at 100 presses (`eEC_EFFECT_CAR_BLIGHT`), the player's
//! wash-car animation, the camera and the wash speed; a plain counter on screen stands in for the game's view.

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::event_visitors as native;
use rustimal_logic::inventory::EMPTY_NO;
use serde::{Deserialize, Serialize};

use crate::character::{NpcClip, NpcGait, NpcModel, PlayerLook};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::messages::{MsgChoice, MsgContext};
use crate::n64_material::N64Material;
use crate::player::Player;
use crate::save::LoadedSave;
use crate::shop::ShopLocation;
use crate::shop_nook::handover::{HandOver, Item, Party, Request};
use crate::states::GameplaySet;
use crate::town::Town;

/// ALL_NPC_NUM + (SP_NPC_DESIGNER - SP_NPC_START).
const GRACIE_NPC: u32 = 238 + 2;
/// `l_sp_actor_name` sound id of `SP_NPC_DESIGNER`.
const GRACIE_SOUND: u8 = 2;
/// `mSP_LIST_EVENT`: the cloth list of the best prize.
const EVENT_LIST: usize = 3;
/// `designer->wash_time = 1080` ticks of 60 Hz.
const WASH_SECS: f32 = 1080.0 / 60.0;
/// `wash_time <= 360`: finishing in the last six seconds is the lesser prize.
const SLOW_SECS: f32 = 360.0 / 60.0;
/// `tmp_data_p->button_count >= 100`.
const PRESSES: u32 = 100;
/// `ITM_CLOTH000`.
const SHIRT_START: u16 = 0x2400;

/// The event's save area (`mEv_designer_c`) for the one player: the pieces given, and whether the game was played
/// (`mEv_SPECL_DESIGNER_COMPLETE`). It belongs to one special event (`rolled`: when the calendar rolled it).
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignerEvent {
    /// `dates[SPECIAL0]` and `dates[SPECIAL1]` of the event this belongs to.
    pub rolled: (u16, u16),
    /// `gifted_cloths[..used]`.
    pub gifted: Vec<u16>,
    pub complete: bool,
}

#[derive(Component)]
struct Gracie {
    /// The game day she came out for.
    day: i64,
}

#[derive(Component)]
struct WashText;

/// Her car: the display list's materials at each wash level (0..=5, 20% steps), the level shown, and the game day.
#[derive(Component)]
struct GracieCar {
    day: i64,
    level: usize,
    /// `variants[level][part]`.
    variants: Vec<Vec<Handle<N64Material>>>,
    /// The parts' entities, in the same order.
    parts: Vec<Entity>,
}

/// `aSCR_set_prim_col` for a wash of `percent` (0..=100).
fn car_colour(percent: f32) -> [u8; 4] {
    let done = percent / 100.0;
    let remaining = 1.0 - done;
    [
        (remaining * 150.0 + done * 255.0) as u8,
        (remaining * 120.0 + done * 255.0) as u8,
        (remaining * 100.0 + done * 255.0) as u8,
        255,
    ]
}

/// The washing levels of the car (20% steps).
const CAR_LEVELS: usize = 6;

/// Startup: the saved event area.
fn apply_loaded(loaded: Res<LoadedSave>, mut event: ResMut<DesignerEvent>) {
    if let Some(save) = loaded.0.as_ref() {
        *event = save.designer_event.clone().unwrap_or_default();
    }
}

/// Brings her out for her day and sends her off when the event ends.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    items: Option<Res<crate::field_items::FieldItems>>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Gracie)>,
    cars: Query<(Entity, &GracieCar)>,
    disc: Option<Res<GameDisc>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
    mut flow: ResMut<Talk>,
) {
    let now = crate::clock::now();
    let out = today.is_active(ev::DESIGNER, now.hour)
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    // After midnight she is still the previous day's visitor.
    let day = if now.hour < 6 {
        now.day_number - 1
    } else {
        now.day_number
    };
    for (entity, g) in &existing {
        if !out || g.day != day {
            commands.entity(entity).despawn();
        }
    }
    for (entity, car) in &cars {
        if !out || car.day != day {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    let Some(items) = items else { return };
    let Some(at) = crate::joan::spot(&town, &items, day) else {
        return;
    };
    // The car stands on a lot of clear ground (`make_FG_somewhere_lot4sale`): the nearest spot to the pick where the
    // car's 3 x 3 cells and her own are clear.
    let clear = |p: Vec3| town.is_clear_ground(p) && !town.in_retail_obstacle(p);
    let fits = |g: Vec3| {
        let car = g + Vec3::new(-1.0, 0.0, -2.0);
        clear(g)
            && (-1..=1)
                .all(|dz| (-1..=1).all(|dx| clear(car + Vec3::new(dx as f32, 0.0, dz as f32))))
    };
    let mut best: Option<(f32, Vec3)> = None;
    for dz in -10i32..=10 {
        for dx in -10i32..=10 {
            let g = at + Vec3::new(dx as f32, 0.0, dz as f32);
            let d = (g - at).length();
            if fits(g) && best.is_none_or(|(b, _)| d < b) {
                best = Some((d, g));
            }
        }
    }
    let at = best.map_or(at, |(_, g)| g.with_y(town.ground_y(g)));
    commands.spawn((
        Name::new("Gracie"),
        Gracie { day },
        NpcClip::default(),
        NpcGait::default(),
        NpcModel(GRACIE_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y),
    ));
    // `designer->complete_flag` belongs to the actor.
    flow.complete_flag = false;
    flow.car_presses = 0;
    info!("Gracie is out at {at:?}");
    // Her car, two cells north and one west of her.
    let Some(disc) = disc else { return };
    let Some(model) = disc.catalog.get("s_car_DL_model") else {
        return;
    };
    let lists = [model.offset];
    let car_at = at + Vec3::new(-1.0, 0.0, -2.0);
    let car_at = car_at.with_y(town.ground_y(car_at));
    let mut variants = Vec::with_capacity(CAR_LEVELS);
    let mut shapes: Vec<Handle<Mesh>> = Vec::new();
    for level in 0..CAR_LEVELS {
        let colours = crate::disc_town::DrawColors {
            prim: car_colour(level as f32 * 20.0),
            prim_lod: 128,
            env: [0; 4],
        };
        let built = crate::disc_town::build_model_full(
            &disc,
            &lists,
            None,
            colours,
            false,
            (&crate::disc_gfx::Segments::default(), &[]),
            (&mut meshes, &mut materials, &mut images),
        );
        if level == 0 {
            shapes = built.iter().map(|(mesh, _)| mesh.clone()).collect();
        }
        variants.push(
            built
                .into_iter()
                .map(|(_, material)| material)
                .collect::<Vec<_>>(),
        );
    }
    if shapes.is_empty() {
        return;
    }
    let root = commands
        .spawn((
            Name::new("Gracie's car"),
            Transform::from_translation(car_at),
            Visibility::default(),
        ))
        .id();
    let parts: Vec<Entity> = shapes
        .into_iter()
        .zip(variants[0].iter())
        .map(|(mesh, material)| {
            commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material.clone()),
                    ChildOf(root),
                ))
                .id()
        })
        .collect();
    commands.entity(root).insert(GracieCar {
        day,
        level: 0,
        variants,
        parts,
    });
    info!("Gracie's car stands at {car_at:?}");
}

/// A new special event starts the event area empty (`mEv_reserve_save_area`).
fn reset_for_new_event(shop: Option<Res<crate::shop::ShopData>>, mut event: ResMut<DesignerEvent>) {
    let Some((ev::DESIGNER, slots)) = shop.and_then(|s| s.events.special_event()) else {
        return;
    };
    let rolled = (slots[0], slots[1]);
    if event.rolled != rolled {
        *event = DesignerEvent {
            rolled,
            ..default()
        };
    }
}

/// What follows a line.
#[derive(Clone, Copy, Debug)]
enum After {
    Close,
    /// The pocket check, then message 19 and the game.
    Check,
    /// The game starts.
    Game,
    /// The forced talk is over: the prize for this result.
    Result(u8),
    /// She hands the piece over.
    Give(u16),
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// The offer is up (`next`: each answer's follow-up line).
    Offer(Vec<Option<usize>>),
    /// A line is up.
    Said(After),
    /// The car-wash game runs.
    Washing { left: f32, presses: u32 },
    /// The game is over (result 0..2): the forced talk follows.
    Washed(u8),
    /// The piece is crossing.
    Hand(u16),
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
    /// `designer->complete_flag`: the game was played this visit.
    complete_flag: bool,
    /// `designer->wear_flag`.
    wear_flag: u8,
    /// `designer_common->button_presses`: the wash so far, kept for the car's colour while it stands.
    car_presses: u32,
}

fn context(identity: &crate::save::PlayerIdentity, present: Option<u16>) -> MsgContext {
    let now = crate::clock::now();
    let mut items = vec![String::new(); 5];
    if let Some(item) = present {
        items[0] = crate::items::name(item);
    }
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Gracie".into(),
        free: vec![String::new(); 20],
        items,
        item_articles: vec![0; 5],
        town_name: identity.town.clone(),
        year: now.year,
        month: now.month,
        day: now.day,
        hour: now.hour,
        minute: now.minute,
        weekday: now.weekday,
        ..default()
    }
}

/// Shows disc message `msg` (absolute) in Gracie's voice.
fn say_abs(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &MsgContext,
    msg: usize,
) -> Option<MsgChoice> {
    let text = disc.text.as_ref()?;
    let (pages, asked) = text.talk_asking(msg, ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "Gracie",
        (GRACIE_NPC, GRACIE_SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Gracie message {msg:#06x}");
    asked.map(|(_, c)| c)
}

/// She turns to face the player while they talk.
fn face(
    time: Res<Time>,
    flow: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<Gracie>)>,
    mut gracie: Query<&mut Transform, With<Gracie>>,
) {
    let (Ok(at), Ok(mut transform)) = (player.single(), gracie.single_mut()) else {
        return;
    };
    if matches!(flow.step, Step::Idle) {
        return;
    }
    let to = (at.translation - transform.translation).with_y(0.0);
    if to.length() > 0.01 {
        let want = Quat::from_rotation_y(to.x.atan2(to.z));
        transform.rotation = transform
            .rotation
            .slerp(want, (time.delta_secs() * 8.0).min(1.0));
    }
}

/// The player at the car while the wash runs (`Player_actor_setup_main_Wash_car`): where they stood, the wash clip in turn
/// (`wash1..5`, four runs each) and the clip's speed (`designer->clean_speed`, which a press raises and time lowers).
#[derive(Default)]
struct WashPose {
    from: Option<(Vec3, Vec3)>,
    clip: usize,
    runs_seen: u32,
    speed: f32,
}

const WASH_CLIPS: [&str; 5] = [
    "cKF_ba_r_ply_1_wash1",
    "cKF_ba_r_ply_1_wash2",
    "cKF_ba_r_ply_1_wash3",
    "cKF_ba_r_ply_1_wash4",
    "cKF_ba_r_ply_1_wash5",
];

/// `aEDSN_game_end_wait`: the wash, with the counter on screen.
#[allow(clippy::too_many_arguments)]
fn wash(
    mut commands: Commands,
    time: Res<Time>,
    mut actions: ResMut<Actions>,
    mut focus: ResMut<InputFocus>,
    mut flow: ResMut<Talk>,
    mut text: Query<(Entity, &mut Text), With<WashText>>,
    mut player: Query<(Entity, &mut Transform, &mut Facing, Option<&mut NpcClip>), With<Player>>,
    cars: Query<&Transform, (With<GracieCar>, Without<Player>)>,
    mut pose: Local<WashPose>,
) {
    let Step::Washing { left, presses } = &mut flow.step else {
        for (entity, _) in &text {
            commands.entity(entity).despawn();
        }
        // `Player_actor_settle_main_Wash_car`: back where they stood.
        if let Some((at, dir)) = pose.from.take() {
            if let Ok((_, mut transform, mut facing, clip)) = player.single_mut() {
                transform.translation = at;
                facing.dir = dir;
                if let Some(mut clip) = clip {
                    clip.stop(8.0);
                }
            }
        }
        return;
    };
    *focus = InputFocus::Dialogue;
    // `chkTrigger(BUTTON_A)`: each press counts.
    let pressed = actions.just_pressed(Action::Interact);
    if pressed {
        actions.consume(Action::Interact);
        *presses += 1;
    }
    // The wash clip: the player stands at the car, facing south, and scrubs (`aEDSN_game_end_wait`: a press adds 0.61
    // to the speed up to 2.25, it falls 0.07 a 60 Hz tick).
    if let (Ok((entity, mut transform, mut facing, clip)), Ok(car)) =
        (player.single_mut(), cars.single())
    {
        if pose.from.is_none() {
            pose.from = Some((transform.translation, facing.dir));
            pose.clip = 0;
            pose.runs_seen = 0;
            pose.speed = 0.0;
        }
        transform.translation = car.translation;
        facing.dir = Vec3::Z;
        pose.speed = if pressed {
            (pose.speed + 0.61).min(2.25)
        } else {
            (pose.speed - 0.07 * 60.0 * time.delta_secs()).max(0.0)
        };
        if clip.is_none() {
            commands.entity(entity).insert(NpcClip::default());
        }
        if let Some(mut clip) = clip {
            // Four runs of a clip, then the next (`counter >= 3`).
            if clip.name != Some(WASH_CLIPS[pose.clip]) {
                clip.start(WASH_CLIPS[pose.clip], false, 0.0);
                pose.runs_seen = 0;
            } else if clip.loops >= 4 {
                pose.clip = (pose.clip + 1) % WASH_CLIPS.len();
                clip.start(WASH_CLIPS[pose.clip], false, 0.0);
                pose.runs_seen = 0;
            }
            clip.speed = Some(pose.speed * 0.5);
        }
    }
    *left -= time.delta_secs();
    let line = format!(
        "Press E to wash the car!  {presses}/{PRESSES}   {:.0} s",
        left.max(0.0).ceil()
    );
    if let Ok((_, mut shown)) = text.single_mut() {
        shown.0 = line;
    } else {
        commands.spawn((
            WashText,
            Text::new(line),
            TextFont {
                font_size: FontSize::Px(36.0),
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(90.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                ..default()
            },
        ));
    }
    let shown = *presses;
    let result = if *presses >= PRESSES {
        // `designer->wash_time <= 360 ? 1 : 0`.
        Some(if *left <= SLOW_SECS { 1 } else { 0 })
    } else if *left <= 0.0 {
        Some(2)
    } else {
        None
    };
    if let Some(result) = result {
        info!("Gracie: the wash is over, result {result} ({presses} presses, {left:.1} s left)");
        flow.step = Step::Washed(result);
    }
    flow.car_presses = shown.min(PRESSES);
}

/// `aSCR_set_prim_col`: the car's colour follows the wash.
fn car_colour_update(
    flow: Res<Talk>,
    mut cars: Query<&mut GracieCar>,
    mut parts: Query<&mut MeshMaterial3d<N64Material>>,
) {
    for mut car in &mut cars {
        let level = ((flow.car_presses as f32 / 20.0).round() as usize).min(CAR_LEVELS - 1);
        if level == car.level {
            continue;
        }
        car.level = level;
        for (part, material) in car.parts.iter().zip(&car.variants[level]) {
            if let Ok(mut shown) = parts.get_mut(*part) {
                shown.0 = material.clone();
            }
        }
    }
}

/// The talk, as `aEDSN_set_norm_talk_info`, the inventory check, `aEDSN_set_force_talk_info` and the result steps.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), With<Player>>,
    gracie: Query<(Entity, &Transform), With<Gracie>>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    mut event: ResMut<DesignerEvent>,
    look: Res<PlayerLook>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, actor_at)) = gracie.single() else {
        return;
    };
    let msg_idx = |flow: &Talk, idx: u8| usize::from(native::gracie_msg(flow.wear_flag, idx));
    let ctx = context(&identity, None);
    let end = |dialogue: &mut DialogueState, focus: &mut InputFocus, flow: &mut Talk| {
        *dialogue = DialogueState::default();
        *focus = InputFocus::World;
        flow.step = Step::Idle;
    };

    // The game is over: the forced talk (`aEDSN_set_force_talk_info`, message 25).
    if let Step::Washed(result) = flow.step {
        *focus = InputFocus::Dialogue;
        flow.complete_flag = true;
        let m = msg_idx(&flow, 25);
        say_abs(&disc, &mut dialogue, actor, &ctx, m);
        flow.step = Step::Said(After::Result(result));
        return;
    }
    if matches!(flow.step, Step::Washing { .. }) {
        return;
    }

    // The piece is crossing.
    if let Step::Hand(present) = flow.step {
        *focus = InputFocus::Dialogue;
        if handover.busy() {
            return;
        }
        // `mPr_SetFreePossessionItem`, then the record (`aEDSN_demo_end_wait_talk_proc`).
        inventory.add(present);
        event.gifted.push(present);
        end(&mut dialogue, &mut focus, &mut flow);
        return;
    }

    // Only her own answers; other talkers in town take theirs.
    let answer = if dialogue.speaker == Some(actor) {
        dialogue.external_result.take()
    } else {
        None
    };
    if let Some(choice) = answer {
        *focus = InputFocus::Dialogue;
        match std::mem::take(&mut flow.step) {
            Step::Offer(next) => {
                // Yes goes on (the answer's own line, then the pocket check); No ends.
                let follow = next.get(choice).copied().flatten();
                if choice == 0 {
                    match follow {
                        Some(m) => {
                            say_abs(&disc, &mut dialogue, actor, &ctx, m);
                            flow.step = Step::Said(After::Check);
                        }
                        None => {
                            // No line of its own: straight to the pocket check.
                            if inventory.pockets.pockets.contains(&EMPTY_NO) {
                                let m = msg_idx(&flow, 19);
                                say_abs(&disc, &mut dialogue, actor, &ctx, m);
                                flow.step = Step::Said(After::Game);
                            } else {
                                end(&mut dialogue, &mut focus, &mut flow);
                            }
                        }
                    }
                } else if let Some(m) = follow {
                    say_abs(&disc, &mut dialogue, actor, &ctx, m);
                    flow.step = Step::Said(After::Close);
                } else {
                    end(&mut dialogue, &mut focus, &mut flow);
                }
            }
            Step::Said(After::Check) => {
                // `aEDSN_inventory_check_talk_proc`: with a free pocket, message 19 and the game.
                if inventory.pockets.pockets.contains(&EMPTY_NO) {
                    let m = msg_idx(&flow, 19);
                    say_abs(&disc, &mut dialogue, actor, &ctx, m);
                    flow.step = Step::Said(After::Game);
                } else {
                    end(&mut dialogue, &mut focus, &mut flow);
                }
            }
            Step::Said(After::Game) => {
                // `aEDSN_game_end_wait_init`.
                *dialogue = DialogueState::default();
                flow.step = Step::Washing {
                    left: WASH_SECS,
                    presses: 0,
                };
            }
            Step::Said(After::Result(result)) => {
                // `aEDSN_check_result_talk_proc`: the prize for the result.
                let list = |i: usize| crate::goods::list(crate::goods::Kind::Cloth, i);
                let present = match result {
                    0 => {
                        let fresh: Vec<u16> = list(EVENT_LIST)
                            .into_iter()
                            .filter(|c| !event.gifted.contains(c))
                            .collect();
                        (!fresh.is_empty())
                            .then(|| fresh[rand.0.random(fresh.len() as i32) as usize])
                    }
                    1 => crate::goods::select_common(
                        crate::goods::Kind::Cloth,
                        &event.gifted,
                        &mut |n| rand.0.random(n as i32) as u32,
                    ),
                    _ => None,
                };
                event.complete = true;
                info!("Gracie: result {result}, prize {present:?}");
                let m = msg_idx(&flow, native::gracie_result_msg_idx(result));
                let ctx = context(&identity, present);
                say_abs(&disc, &mut dialogue, actor, &ctx, m);
                flow.step = match present {
                    Some(p) => Step::Said(After::Give(p)),
                    None => Step::Said(After::Close),
                };
            }
            Step::Said(After::Give(present)) => {
                // `aEDSN_demo_start_wait_talk_proc`.
                *dialogue = DialogueState::default();
                handover.start(Request {
                    item: Item::Goods(present),
                    giver: Party::Npc(actor),
                    taker: Party::Player,
                });
                flow.step = Step::Hand(present);
            }
            Step::Said(After::Close)
            | Step::Idle
            | Step::Washing { .. }
            | Step::Washed(_)
            | Step::Hand(_) => {
                end(&mut dialogue, &mut focus, &mut flow);
            }
        }
        return;
    }
    if !matches!(flow.step, Step::Idle)
        || !focus.is_world()
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    let delta = (actor_at.translation - at.translation).with_y(0.0);
    if delta.length() > 1.6 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    // `aEDSN_set_norm_talk_info`: the shirt she sees, then what to say.
    let worn = SHIRT_START + look.cloth as u16;
    flow.wear_flag =
        u8::from(crate::goods::list(crate::goods::Kind::Cloth, EVENT_LIST).contains(&worn));
    let (idx, offer) = if flow.complete_flag {
        (21 + rand.0.random(3) as u8, false)
    } else if let Some(&gift) = event.gifted.first() {
        let event_piece = crate::goods::list(crate::goods::Kind::Cloth, EVENT_LIST).contains(&gift);
        (if event_piece { 17 } else { 18 }, false)
    } else if event.complete {
        (8, true)
    } else {
        (
            rand.0.random(4) as u8 + if look.female { 4 } else { 0 },
            true,
        )
    };
    let m = msg_idx(&flow, idx);
    let asked = say_abs(&disc, &mut dialogue, actor, &ctx, m);
    flow.step = match (offer, asked) {
        (true, Some(choice)) => Step::Offer(choice.next),
        // The offer messages set no question: when they end (`aEDSN_inventory_check_talk_proc`) the pockets are checked.
        (true, None) => Step::Said(After::Check),
        _ => Step::Said(After::Close),
    };
}

pub struct GraciePlugin;

impl Plugin for GraciePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .init_resource::<DesignerEvent>()
            .add_systems(Startup, apply_loaded)
            .add_systems(
                Update,
                (
                    reset_for_new_event,
                    visit.after(crate::event_today::refresh),
                )
                    .chain()
                    .in_set(GameplaySet),
            )
            .add_systems(Update, (face, wash, car_colour_update).in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
