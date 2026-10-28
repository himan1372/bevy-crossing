//! The monthly raffle (fukubiki, the last day of the month): Nook in his
//! raffle coat (SP_NPC_SHOP_MASTERSP, ac_npc_shop_mastersp_talk.c_inc)
//! takes five of this month's lottery tickets from the pockets for a turn
//! of the drum.
//!
//! Talk: 0x10D1 asks; five tickets -> 0x10D4 and the draw (else 0x10D3).
//! The tickets go first (a loss still costs them); RANDOM(100) below 5 wins
//! the first prize, below 15 the second, below 35 the third, else nothing,
//! and a prize already won is a loss. A win: 0x10D9 / 0x10DA / 0x10DB, the
//! prize handed over wrapped (0x10E3, mPr_ITEM_COND_PRESENT), then 0x10DD
//! (another go?) or 0x10DC when out of tickets or prizes. A loss: 0x10D5,
//! then 0x10D7 (another go?) or 0x10D6.
//!
//! The prizes (Shop_c.lottery_items, mSP_MakeLotteryList) are three from
//! the lottery list, one never yet held first when there is one, made anew
//! each month. On the raffle day they stand on the fuku layout's furniture
//! spots. Nook waits at the drum (`base_pos[0]`, 160, 140); a win sends
//! him to that prize. The drum (`cKF_bs_r_obj_gara`) turns, and at tick 46
//! the ball (`act_fukutama`) rolls out.
use bevy::prelude::*;
use rustimal_logic::furniture::FurnitureSize;
use rustimal_logic::qrand::Qrand;
use rustimal_logic::special_delivery as sd;

use crate::dialogue::DialogueState;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::n64_material::N64Material;
use crate::player::Player;
use crate::shop::ShopLocation;
use crate::shop_nook::Shopkeeper;
use crate::states::GameplaySet;

/// The drum turning before the result line. The ball leaves at tick 46
/// (about 0.8 s); the line waits out the rest of the turn.
const SPIN_SECONDS: f32 = 3.0;
/// NA_SE_51: the raffle drum.
const DRUM_SE: u16 = 0x51;
/// Nook's Cranny drum (`garagara_set_table[0]`) and where he stands.
const DRUM_GAME: Vec3 = Vec3::new(160.0, 60.0, 190.0);
pub(crate) const NOOK_AT_DRUM: Vec2 = Vec2::new(160.0, 140.0);
/// Ball comes out when `anime_frame` reaches this (`aGaragara_RotateMove`).
const BALL_TICK: u32 = 46;

/// Whether today is the raffle (mSP_CheckFukubikiDay).
pub fn raffle_day() -> bool {
    let now = crate::clock::now();
    rustimal_logic::shop::tanuki_shop_status(now.year, now.month, now.day, false)
        == rustimal_logic::shop::TanukiShopStatus::Fukubiki
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// A question is up (0x10D1, 0x10D7 / 0x10DD).
    Ask,
    /// The drum turning; then the result line.
    Spin(f32),
    Finish,
}

#[derive(Resource, Default)]
pub(crate) struct Raffle {
    step: Step,
    /// 0-2 a prize tier, 3 no win.
    result: usize,
    /// Game-unit XZ where he picks the prize up. None: stand at the drum.
    fetch: Option<Vec2>,
}

/// Where Nook should be, in game units. None: follow the player.
#[derive(Resource, Default)]
pub(crate) struct RaffleNook {
    pub at: Option<Vec2>,
}

/// The drum turn in progress. `ticks` is 60 Hz frames since it started.
#[derive(Resource, Default)]
struct DrumClock {
    ticks: Option<u32>,
    ball_type: u8,
}

/// mSP_MakeLotteryList, once a month.
pub(crate) fn lineup(mut catalog: ResMut<crate::catalog::Catalog>) {
    let month = crate::clock::now().month;
    if catalog.lottery_month == month
        || crate::goods::list(crate::goods::Kind::Furniture, 5).is_empty()
    {
        return;
    }
    // The month fixes the lineup, so it keeps its own stream rather than the global one.
    let mut stream = Qrand::new(0x2F6B_7A11 ^ u32::from(month));
    let mut rand = |n: usize| stream.random(n.max(1) as i32) as usize;
    let list = crate::goods::list(crate::goods::Kind::Furniture, 5);
    let unheld: Vec<u16> = list
        .iter()
        .copied()
        .filter(|&i| !catalog.collection.has(i))
        .collect();
    let first = (!unheld.is_empty()).then(|| unheld[rand(unheld.len())]);
    let mut rest = Vec::new();
    let mut guard = 0;
    while rest.len() < 3 && guard < 200 {
        guard += 1;
        let item = list[rand(list.len())];
        if Some(item) != first && !rest.contains(&item) {
            rest.push(item);
        }
    }
    catalog.lottery = sd::make_lottery_lineup(first, &rest);
    catalog.lottery_month = month;
    info!("raffle prizes: {:x?}", catalog.lottery);
}

/// check_lottery_ticket: five of this month's tickets in normal condition.
fn has_tickets(inventory: &PlayerInventory, month: u8) -> bool {
    let mut count = 0;
    for (slot, &item) in inventory.pockets.pockets.iter().enumerate() {
        if inventory.pockets.item_cond(slot) == 0
            && sd::is_ticket_item(item)
            && sd::ticket_month(item) == month
        {
            count += sd::ticket_count(item);
        }
    }
    count >= sd::LOTTERY_REQ_TICKETS
}

/// uketori_lottery_ticket.
fn take_tickets(inventory: &mut PlayerInventory, month: u8) {
    let mut req = i16::from(sd::LOTTERY_REQ_TICKETS);
    for slot in 0..inventory.pockets.pockets.len() {
        if req <= 0 {
            break;
        }
        let item = inventory.pockets.pockets[slot];
        if inventory.pockets.item_cond(slot) != 0
            || !sd::is_ticket_item(item)
            || sd::ticket_month(item) != month
        {
            continue;
        }
        let count = i16::from(sd::ticket_count(item));
        inventory.pockets.pockets[slot] = if count <= req {
            0
        } else {
            sd::ticket_get_item(month, (count - req) as u8)
        };
        req -= count;
    }
}

/// Several lines in one window; true when the last asks a question.
fn say_all(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    msgs: &[usize],
    ctx: &crate::messages::MsgContext,
) -> bool {
    let Some(text) = disc.text.as_ref() else {
        return false;
    };
    let mut pages = Vec::new();
    let mut labels = None;
    for &msg in msgs {
        let (more, choice) = text.talk(msg, ctx, &mut |_| 0);
        pages.extend(more);
        labels = choice.map(|c| c.labels);
        info!("raffle message {msg:#06x}");
    }
    let asks = labels.is_some();
    dialogue.begin_external(actor, pages, labels);
    asks
}

fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    msg: usize,
    ctx: &crate::messages::MsgContext,
) -> bool {
    let Some(text) = disc.text.as_ref() else {
        return false;
    };
    let (pages, choice) = text.talk(msg, ctx, &mut |_| 0);
    let asks = choice.is_some();
    dialogue.begin_external(actor, pages, choice.map(|c| c.labels));
    info!("raffle message {msg:#06x}");
    asks
}

#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    time: Res<Time>,
    location: Res<State<ShopLocation>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), With<Player>>,
    nook: Query<(Entity, &Transform, &Shopkeeper)>,
    mut raffle: ResMut<Raffle>,
    mut clock: ResMut<DrumClock>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut catalog: ResMut<crate::catalog::Catalog>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        clock.ticks = None;
        return;
    };
    if *location != ShopLocation::Inside || !raffle_day() {
        clock.ticks = None;
        return;
    }
    if matches!(raffle.step, Step::Idle) {
        clock.ticks = None;
        raffle.fetch = None;
    }
    let Some((actor, nook_at, _)) = nook.iter().find(|(_, _, k)| k.twin.is_none()) else {
        return;
    };
    let month = crate::clock::now().month;
    let mut ctx = crate::messages::MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Tom Nook".into(),
        free: vec![String::new(); 20],
        items: vec![String::new(); 5],
        item_articles: vec![0; 5],
        ..default()
    };
    let draw = |raffle: &mut Raffle,
                inventory: &mut PlayerInventory,
                catalog: &crate::catalog::Catalog,
                rand: &mut Qrand| {
        // fukubiki_before_process.
        take_tickets(inventory, month);
        let roll = sd::lottery_roll(rand.random(100) as u8) as usize;
        raffle.result = if roll < 3 && !sd::prize_available(catalog.lottery[roll]) {
            3
        } else {
            roll
        };
        raffle.fetch = None;
        raffle.step = Step::Spin(0.0);
        crate::sound::se(DRUM_SE);
        info!("raffle draw: {}", raffle.result);
    };

    if let Step::Spin(t) = raffle.step {
        {
            // 0x10D4 stays up while the drum turns; the result follows on in
            // the same window (the original continues each line into the
            // next as Nook fetches the prize).
            let t = t + time.delta_secs();
            let add = ((time.delta_secs() * 60.0).round() as u32).max(1);
            clock.ticks = Some(clock.ticks.unwrap_or(0).saturating_add(add));
            clock.ball_type = raffle.result.min(3) as u8;
            if t < SPIN_SECONDS {
                raffle.step = Step::Spin(t);
                return;
            }
            *focus = InputFocus::Dialogue;
            let mut msgs = Vec::new();
            if raffle.result < 3 {
                // to_atari / aSHM_give_player: wrapped, the slot marked sold.
                let prize = catalog.lottery[raffle.result];
                raffle.fetch = Some(prize_stand(prize, raffle.result, Some(&disc)));
                ctx.free[0] = crate::items::name(prize);
                if let Some(slot) = inventory.pockets.pockets.iter().position(|&i| i == 0) {
                    inventory.pockets.pockets[slot] = prize;
                    inventory.pockets.set_item_cond(slot, 1);
                }
                catalog.lottery[raffle.result] = sd::RSV_SHOP_SOLD_FTR;
                info!("raffle prize {prize:#06x}");
                msgs.push(0x10D9 + raffle.result);
                msgs.push(0x10E3);
                let prizes_left = catalog.lottery.iter().any(|&p| sd::prize_available(p));
                msgs.push(if prizes_left && has_tickets(&inventory, month) {
                    0x10DD
                } else {
                    0x10DC
                });
            } else {
                msgs.push(0x10D5);
                msgs.push(if has_tickets(&inventory, month) {
                    0x10D7
                } else {
                    0x10D6
                });
            }
            raffle.step = if say_all(&disc, &mut dialogue, actor, &msgs, &ctx) {
                Step::Ask
            } else {
                Step::Finish
            };
            return;
        }
    }

    if let Some(choice) = dialogue.external_result.take() {
        *focus = InputFocus::Dialogue;
        match (std::mem::take(&mut raffle.step), choice) {
            // aSHM_talk_try / aSHM_talk_retry.
            (Step::Ask, 0) if has_tickets(&inventory, month) => {
                say(&disc, &mut dialogue, actor, 0x10D4, &ctx);
                draw(&mut raffle, &mut inventory, &catalog, &mut rand.0);
                clock.ticks = Some(0);
                clock.ball_type = raffle.result.min(3) as u8;
            }
            (Step::Ask, 0) => {
                say(&disc, &mut dialogue, actor, 0x10D3, &ctx);
                raffle.step = Step::Finish;
            }
            _ => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
            }
        }
        return;
    }

    if !matches!(raffle.step, Step::Idle | Step::Finish)
        || !focus.is_world()
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    let delta = (nook_at.translation - at.translation).with_y(0.0);
    if delta.length() > 1.8 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    raffle.step = if say(&disc, &mut dialogue, actor, 0x10D1, &ctx) {
        Step::Ask
    } else {
        Step::Finish
    };
}

/// `aSHM_set_base_position`: stand in front of prize `tier` (0 first).
fn prize_stand(item: u16, tier: usize, disc: Option<&GameDisc>) -> Vec2 {
    let (mut x, mut z) = [(160.0_f32, 138.0), (240.0, 138.0), (80.0, 138.0)][tier.min(2)];
    let size = disc.and_then(|d| {
        crate::furniture::furniture_index(item).and_then(|index| {
            crate::furniture::gameplay_profile(d, index as u16).map(|p| p.shape.size_class())
        })
    });
    // aSHM_ftr_revise_x / _z. A 1x1 shifts half a unit left and one unit
    // forward; anything but a 2x2 shifts one unit forward.
    if size != Some(FurnitureSize::Size2x2) {
        z -= 40.0;
    }
    if size == Some(FurnitureSize::Size1x1) {
        x -= 20.0;
    }
    Vec2::new(x, z)
}

fn game_to_world(g: Vec3, layout: crate::shop::layout::Layout) -> Vec3 {
    // Match the shop shell's (4, 1, 4) centre, including its floor height.
    let corner = crate::shop::SHOP_ORIGIN - layout.centre();
    corner + (g + layout.raffle_shift()) / 40.0
}

/// The drum's posed footprint, including thin parts that room-height sampling
/// can miss. Use the existing shop collision radius around these world bounds.
pub(crate) fn obstacle_bounds(
    disc: &GameDisc,
    layout: crate::shop::layout::Layout,
) -> Option<(Vec2, Vec2)> {
    let skeleton =
        crate::skeleton::Skeleton::read(&disc.rom, disc.catalog.get("cKF_bs_r_obj_gara")?.offset)?;
    let animation = crate::skeleton::Animation::read(
        &disc.rom,
        disc.catalog.get("cKF_ba_r_obj_gara")?.offset,
        skeleton.joints.len(),
    )?;
    let mut sink = crate::disc_gfx::run_display_lists(&disc.rom, &skeleton.lists());
    let pose = skeleton.world_matrices(Some(&animation), 1.0);
    crate::skeleton::place_on_joints(&mut sink, &skeleton, &pose);
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for vertex in sink.parts.iter().flat_map(|part| &part.vertices) {
        let point = game_to_world(DRUM_GAME + Vec3::from(vertex.pos) * 0.01, layout).xz();
        min = min.min(point);
        max = max.max(point);
    }
    (min.is_finite() && max.is_finite()).then_some((min, max))
}

/// On the raffle day Nook stays at the drum, or at the prize he is fetching.
pub(crate) fn stage(
    location: Res<State<ShopLocation>>,
    layout: Res<crate::shop::layout::Layout>,
    raffle: Res<Raffle>,
    mut nook: ResMut<RaffleNook>,
) {
    if *location != ShopLocation::Inside || !raffle_day() {
        nook.at = None;
        return;
    }
    nook.at = Some(raffle.fetch.unwrap_or(NOOK_AT_DRUM) + layout.raffle_shift().xz());
}

#[derive(Component)]
struct DrumRoot {
    applied: u32,
    playing: bool,
}

#[derive(Component)]
struct RaffleBall;

struct BallSim {
    made: bool,
    simulated: u32,
    pos: Vec3,
    on_ground: bool,
    old_on_ground: bool,
    hit_wall: bool,
    hit_wall_angle: f32,
    vel_y: f32,
    acc_y: f32,
    max_vel_y: f32,
    vel_xz: f32,
    angle: f32,
    status: u8,
    shown: Option<u8>,
    parts: Vec<Entity>,
}

impl Default for BallSim {
    fn default() -> Self {
        Self {
            made: false,
            simulated: 0,
            pos: Vec3::ZERO,
            on_ground: false,
            old_on_ground: false,
            hit_wall: false,
            hit_wall_angle: 0.0,
            vel_y: 0.0,
            acc_y: -0.15,
            max_vel_y: -1.0,
            vel_xz: 0.0,
            angle: 0.0,
            status: 0,
            shown: None,
            parts: Vec::new(),
        }
    }
}

/// `aGRBL_make` then one 60 Hz step of `aGRBL_move`.
fn ball_make(sim: &mut BallSim, rand: &mut Qrand) {
    let src = DRUM_GAME;
    sim.pos = src + Vec3::new(10.0, 10.0, 0.0);
    sim.on_ground = false;
    sim.old_on_ground = false;
    sim.hit_wall = false;
    sim.vel_y = -0.15 - rand.fqrand() * 0.1;
    sim.acc_y = -0.15;
    sim.max_vel_y = -1.0;
    sim.vel_xz = 0.225 + rand.fqrand() * 0.1;
    let span = 22.5_f32.to_radians();
    sim.angle = std::f32::consts::FRAC_PI_2 + rand.fqrand() * span - span * 0.5;
    sim.status = 1;
}

fn ball_step(sim: &mut BallSim) {
    if sim.status == 1 {
        sim.status = 2;
        ball_integrate(sim);
        ball_bounds(sim);
        return;
    }
    if sim.status != 2 {
        return;
    }
    if sim.hit_wall {
        // aGRBL_KorokoroMove: outgoing angle is the wall angle minus 180.
        sim.angle = sim.hit_wall_angle - std::f32::consts::PI;
        sim.vel_xz *= 0.985;
    }
    if sim.on_ground {
        if sim.old_on_ground {
            sim.vel_xz *= 0.99;
        } else {
            sim.vel_y *= -0.4;
        }
    }
    ball_integrate(sim);
    ball_bounds(sim);
}

fn ball_integrate(sim: &mut BallSim) {
    sim.vel_y += sim.acc_y;
    if sim.vel_y < sim.max_vel_y {
        sim.vel_y = sim.max_vel_y;
    }
    sim.pos.y += sim.vel_y;
    sim.pos.x += sim.vel_xz * sim.angle.sin();
    sim.pos.z += sim.vel_xz * sim.angle.cos();
}

fn ball_bounds(sim: &mut BallSim) {
    let src = DRUM_GAME;
    sim.old_on_ground = sim.on_ground;
    sim.on_ground = false;
    if sim.pos.y - 2.0 < src.y {
        sim.pos.y = src.y + 2.0;
        sim.on_ground = true;
    }
    sim.hit_wall = false;
    if sim.pos.x > src.x + 20.0 {
        sim.pos.x = src.x + 20.0;
        sim.hit_wall = true;
        sim.hit_wall_angle = -std::f32::consts::FRAC_PI_2;
    }
    if sim.pos.x < src.x - 20.0 {
        sim.pos.x = src.x - 20.0;
        sim.hit_wall = true;
        sim.hit_wall_angle = std::f32::consts::FRAC_PI_2;
    }
    if sim.pos.z > src.z + 7.0 {
        sim.pos.z = src.z + 7.0;
        sim.hit_wall = true;
        sim.hit_wall_angle = -std::f32::consts::PI;
    }
    if sim.pos.z < src.z - 7.0 {
        sim.pos.z = src.z - 7.0;
        sim.hit_wall = true;
        sim.hit_wall_angle = 0.0;
    }
}

fn symbol_bytes(disc: &GameDisc, name: &str) -> Option<std::sync::Arc<[u8]>> {
    let asset = disc.catalog.get(name)?;
    disc.rom
        .slice(disc::RomSource::Rel, asset.offset, asset.size)
        .map(std::sync::Arc::from)
}

fn load_ball_frames(
    disc: &GameDisc,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) -> Vec<Vec<(Handle<Mesh>, Handle<N64Material>)>> {
    let Some(model) = disc.catalog.get("act_fukutama_modelT") else {
        return Vec::new();
    };
    let colors = crate::disc_town::DrawColors {
        prim: [255; 4],
        prim_lod: 255,
        env: [255; 4],
    };
    let mut frames = Vec::with_capacity(4);
    for i in 1..=4 {
        let Some(bytes) = symbol_bytes(disc, &format!("act_fukutama{i}_tex")) else {
            return Vec::new();
        };
        let mut segs = crate::disc_gfx::Segments::default();
        segs.set_segment(8, bytes);
        frames.push(crate::disc_town::build_model_full(
            disc,
            &[model.offset],
            None,
            colors,
            true,
            (&segs, &[]),
            (&mut *meshes, &mut *materials, &mut *images),
        ));
    }
    frames
}

/// The drum on the counter, and the ball that rolls out at tick 46.
#[allow(clippy::too_many_arguments)]
fn present(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    view: (Res<State<ShopLocation>>, Res<crate::shop::layout::Layout>),
    clock: Res<DrumClock>,
    mut frames: Local<Vec<Vec<(Handle<Mesh>, Handle<N64Material>)>>>,
    mut sim: Local<BallSim>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut drum_made: Local<bool>,
    mut ball_made: Local<bool>,
    mut rigs: Query<(Entity, &mut crate::disc_rig::DiscRig, &mut DrumRoot)>,
    mut balls: Query<(Entity, &mut Transform, &mut Visibility), With<RaffleBall>>,
    camera: Query<
        &GlobalTransform,
        (
            With<Camera3d>,
            Without<crate::msg_window::OverlayCamera>,
            Without<RaffleBall>,
            Without<DrumRoot>,
        ),
    >,
    mut joints: Query<&mut Transform, (Without<DrumRoot>, Without<RaffleBall>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (location, layout) = view;
    let inside = disc.is_some() && *location == ShopLocation::Inside && raffle_day();
    if !inside {
        for (e, ..) in &rigs {
            commands.entity(e).despawn();
        }
        for (e, ..) in &balls {
            commands.entity(e).despawn();
        }
        *drum_made = false;
        *ball_made = false;
        sim.made = false;
        return;
    }
    let disc = disc.unwrap();
    if !*drum_made {
        let root = commands
            .spawn((
                Name::new("raffle drum"),
                DrumRoot {
                    applied: 0,
                    playing: false,
                },
                Transform::from_translation(game_to_world(DRUM_GAME, *layout))
                    .with_scale(Vec3::splat(crate::shop_display::GOODS_SCALE)),
                Visibility::default(),
            ))
            .id();
        if let Some(rig) = crate::disc_rig::spawn(
            &mut commands,
            &disc,
            root,
            ("cKF_bs_r_obj_gara", "cKF_ba_r_obj_gara"),
            (&mut meshes, &mut materials, &mut images),
        ) {
            commands.entity(root).insert(rig);
            info!("raffle drum at {:?}", DRUM_GAME);
        }
        *drum_made = true;
    }
    if let Ok((_, mut rig, mut root)) = rigs.single_mut() {
        match clock.ticks {
            Some(ticks) => {
                if ticks < root.applied || !root.playing {
                    rig.rewind();
                    root.applied = 0;
                    root.playing = true;
                }
                let delta = ticks.saturating_sub(root.applied);
                if delta > 0 {
                    rig.step_stop(0.5 * delta as f32, &mut joints);
                    root.applied = ticks;
                }
            }
            None => root.playing = false,
        }
    }
    if frames.is_empty() {
        *frames = load_ball_frames(&disc, &mut meshes, &mut materials, &mut images);
    }
    if !*ball_made && !frames.is_empty() {
        *ball_made = true;
        commands.spawn((
            Name::new("raffle ball"),
            RaffleBall,
            // build_model_full already converts the ball's vertices to world units.
            Transform::default(),
            Visibility::Hidden,
        ));
    }
    let Ok((ball_e, mut ball_tf, mut vis)) = balls.single_mut() else {
        return;
    };
    let show = clock.ticks.is_some_and(|t| t >= BALL_TICK);
    if !show {
        sim.made = false;
        sim.shown = None;
        for e in sim.parts.drain(..) {
            commands.entity(e).despawn();
        }
        *vis = Visibility::Hidden;
        return;
    }
    let ticks = clock.ticks.unwrap_or(BALL_TICK);
    if !sim.made {
        ball_make(&mut sim, &mut rand.0);
        sim.simulated = BALL_TICK;
        sim.made = true;
        sim.shown = None;
    }
    let mut n = ticks.saturating_sub(sim.simulated);
    while n > 0 {
        ball_step(&mut sim);
        n -= 1;
    }
    sim.simulated = ticks;
    if sim.shown != Some(clock.ball_type) {
        for e in sim.parts.drain(..) {
            commands.entity(e).despawn();
        }
        if let Some(parts) = frames.get(usize::from(clock.ball_type)) {
            for (mesh, mat) in parts {
                sim.parts.push(
                    commands
                        .spawn((
                            Mesh3d(mesh.clone()),
                            MeshMaterial3d(mat.clone()),
                            ChildOf(ball_e),
                        ))
                        .id(),
                );
            }
        }
        sim.shown = Some(clock.ball_type);
    }
    ball_tf.translation = game_to_world(sim.pos, *layout);
    if let Some(cam) = camera.iter().next() {
        let to = cam.translation() - ball_tf.translation;
        ball_tf.rotation = Quat::from_rotation_y(to.x.atan2(to.z));
    }
    *vis = Visibility::Inherited;
}

pub struct RafflePlugin;

impl Plugin for RafflePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Raffle>()
            .init_resource::<RaffleNook>()
            .init_resource::<DrumClock>()
            .add_systems(Update, lineup.in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet))
            .add_systems(Update, stage.after(talk).in_set(GameplaySet))
            .add_systems(Update, present.after(talk).in_set(GameplaySet));
    }
}
