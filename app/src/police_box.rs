//! The police box (SCENE_POLICE_BOX): its door, the disc room
//! (police_indoor), Booker at the desk and the lost and found.
//!
//! ac_police_box_move: walking in within sqrt(1100) game units of (x + 24,
//! z + 40) facing between north and west enters at (200, 380); leaving puts
//! the player at (x + 60, z + 60) facing south-east. Booker (SP_NPC_POLICE2)
//! starts at unit (4, 6) and follows the player about the room, greets a player who came through the door
//! (0x784 / 0x785 by whether anything is kept) and otherwise answers with
//! 0x786 / 0x787 / 0x77D. The lost and found (mPB, 20 items) lies on the
//! floor at the room's RSV_POLICE_ITEM units; A facing one makes Booker
//! offer it (0x77E): yes takes it if the pockets have room (else 0x781).
//! A new lost and found holds one common furniture item and two shirts
//! (mPB_police_box_init); each new day it may gain one more
//! (mPB_force_set_keep_item). Copper's post outside is in copper.rs.
use bevy::prelude::*;

use crate::dialogue::DialogueState;
use crate::game_disc::GameDisc;
use crate::indoor_room::IndoorRoom;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::locomotion::Locomotion;
use crate::messages::MsgContext;
use crate::n64_material::N64Material;
use crate::player::Player;
use crate::shop::{ShopLocation, ShopReturnPos};
use crate::states::GameplaySet;
use crate::town::Town;

pub const POLICE_ORIGIN: Vec3 = Vec3::new(-3000.0, 0.0, 3000.0);
const UNIT: f32 = 40.0;
/// `POLICE_STATION` layout item (the kouban building).
const POLICE_STATION: u16 = 0x580C;
/// FG_TYPE_POLICE_INDOOR.
const FG_POLICE_INDOOR: u16 = 0x00CE;
/// RSV_POLICE_ITEM_0..: lost-and-found spots in the room's units.
const RSV_POLICE_ITEM_0: u16 = 0xF128;
/// mPB_POLICE_BOX_ITEM_STORAGE_COUNT.
pub const KEEP_COUNT: usize = 20;
/// police_box door data: (200, 0, 380), facing north.
const ENTRY: Vec2 = Vec2::new(200.0, 380.0);
/// mvactor police_box: Booker at unit (4, 6).
const BOOKER: Vec2 = Vec2::new(4.0 * 40.0 + 20.0, 6.0 * 40.0 + 20.0);
/// ALL_NPC_NUM + (SP_NPC_POLICE2 - SP_NPC_START); l_sp_actor_name sound 3.
const BOOKER_NPC: u32 = 238 + 16;
const BOOKER_SOUND: u8 = 3;
/// Seconds inside before Booker greets (the door demo).
const ARRIVAL: f32 = 0.6;

/// `PoliceBox_c.keep_items`.
#[derive(Resource, Default, Clone, Debug)]
pub struct LostAndFound(pub [u16; KEEP_COUNT]);

impl LostAndFound {
    pub fn count(&self) -> usize {
        self.0.iter().filter(|&&i| i != 0).count()
    }

    /// mPB_keep_item: items and furniture only, after the others; when
    /// full the oldest goes. (The box is compacted first, as leaving it does,
    /// mPB_copy_itemBuf.)
    pub fn keep(&mut self, item: u16) {
        if !matches!(item >> 12, 1..=3) {
            return;
        }
        let mut kept: Vec<u16> = self.0.iter().copied().filter(|&i| i != 0).collect();
        if kept.len() >= KEEP_COUNT {
            kept.remove(0);
        }
        kept.push(item);
        self.0 = [0; KEEP_COUNT];
        for (slot, i) in self.0.iter_mut().zip(kept) {
            *slot = i;
        }
        info!("lost and found keeps {item:#06x}");
    }
}

/// The game day the lost and found last grew (m_all_grow once a day).
#[derive(Resource, Default)]
pub struct LostGrowDay(pub Option<i64>);

/// mPB_force_set_keep_item, once on a new day: with five or fewer kept, a
/// 50-50 chance of one more: goods 86% (furniture 36, stationery 23,
/// clothing 30, carpet 6, wallpaper 5), a tool or sapling 5%, a flower bag
/// 5%, an umbrella 4%.
fn grow_lost_and_found(
    mut lost: ResMut<LostAndFound>,
    mut day: ResMut<LostGrowDay>,
    disc: Option<Res<GameDisc>>,
) {
    let today = crate::clock::now().day_number;
    if disc.is_none() || day.0 == Some(today) {
        return;
    }
    let first = day.0.is_none();
    day.0 = Some(today);
    if first {
        return;
    }
    let mut seed = (today as u32).wrapping_mul(2_654_435_761) ^ 0x9e37_79b9;
    let mut below = |n: u32| {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        seed % n.max(1)
    };
    if lost.count() > 5 || below(2) == 0 {
        return;
    }
    let roll = below(100);
    let item = if roll <= 85 {
        use crate::goods::Kind;
        let goods = below(100);
        let kind = [
            (35, Kind::Furniture),
            (58, Kind::Paper),
            (88, Kind::Cloth),
            (94, Kind::Carpet),
        ]
        .iter()
        .find(|(top, _)| goods <= *top)
        .map_or(Kind::Wallpaper, |(_, k)| *k);
        let now = crate::goods::roll_now();
        crate::goods::select(&now, kind, 1, &[], &mut below)
            .first()
            .copied()
            .unwrap_or(0)
    } else if roll <= 90 {
        // ITM_NET, AXE, SHOVEL, ROD, SAPLING, CEDAR_SAPLING.
        [0x2200, 0x2201, 0x2202, 0x2203, 0x2900, 0x2901][below(6) as usize]
    } else if roll <= 95 {
        // ITM_WHITE_PANSY_BAG ..
        0x2902 + below(8) as u16
    } else {
        // mSP_RandomUmbSelect: one of the 32 shop umbrellas.
        0x2204 + below(32) as u16
    };
    if item != 0 {
        lost.keep(item);
    }
}

#[derive(Resource)]
struct PoliceBuilding {
    trigger: Vec3,
    exit: Vec3,
}

#[derive(Resource, Default)]
struct PoliceRoom {
    built: bool,
    room: Option<IndoorRoom>,
    /// Room-local unit and lost-and-found slot of each RSV_POLICE_ITEM.
    spots: Vec<((usize, usize), usize)>,
}

/// aPOL2's state: his speed (game units a frame) and the zone he heads for.
#[derive(Component, Default)]
struct Booker {
    speed: f32,
    next_zone: Option<u8>,
}

/// aPOL2_get_zone: the room in a 4 x 5 grid (units x 0-1 / 2-4 / 5-7 / 8-,
/// z 2..6 clamped).
fn booker_zone(p: Vec2) -> u8 {
    let ux = (p.x / UNIT).floor() as i32;
    let uz = ((p.y / UNIT).floor() as i32).clamp(2, 6);
    let x = match ux {
        8.. => 3,
        5..=7 => 2,
        2..=4 => 1,
        _ => 0,
    };
    ((uz - 2) * 4 + x) as u8
}

/// aPOL2_get_next_zone: the next zone on the way from `src` to `dst`
/// (`coin` is RANDOM(2)).
fn booker_next_zone(dst: u8, src: u8, coin: u8) -> u8 {
    let (sx, sz, dx, dz) = (src % 4, src / 4, dst % 4, dst / 4);
    let move_z = || -> u8 {
        if sx == 0 || sx == 3 {
            if sz < dz {
                sx + (sz + 1) * 4
            } else {
                sx + (sz - 1) * 4
            }
        } else if sx == dx {
            if sx == 1 {
                sx + sz * 4 - 1
            } else {
                sx + sz * 4 + 1
            }
        } else if sx < dx {
            sx + sz * 4 + 1
        } else {
            sx + sz * 4 - 1
        }
    };
    if sz == 1 || sz == 3 {
        if sz != dz {
            move_z()
        } else if sx == dx {
            dst
        } else {
            sx + (sz + coin) * 4
        }
    } else if dz.abs_diff(sz) > 1 {
        move_z()
    } else if sx == dx {
        dst
    } else if sx < dx {
        sx + sz * 4 + 1
    } else {
        sx + sz * 4 - 1
    }
}

/// aPOL2 movement: he follows the player about the room, zone by zone,
/// walking (max 1, +0.1 a frame) when near and running (max 4, +0.4)
/// beyond about 70 units, stopping within 50; more than 90 degrees off he
/// turns on the spot first (11.25 degrees a frame). He stays put to talk.
fn booker_follow(
    time: Res<Time>,
    state: Res<State<ShopLocation>>,
    focus: Res<InputFocus>,
    room: Res<PoliceRoom>,
    player: Query<&Transform, (With<Player>, Without<Booker>)>,
    mut booker: Query<(&mut Transform, &mut Booker, &mut crate::character::NpcGait)>,
) {
    if *state != ShopLocation::PoliceBox {
        return;
    }
    let (Ok(at), Ok((mut transform, mut booker, mut gait)), Some(room)) =
        (player.single(), booker.single_mut(), room.room.as_ref())
    else {
        return;
    };
    let ticks = time.delta_secs() * 60.0;
    let local = |p: Vec3| Vec2::new(p.x - POLICE_ORIGIN.x, p.z - POLICE_ORIGIN.z) * UNIT;
    let (me, pl) = (local(transform.translation), local(at.translation));
    let to_player = pl - me;
    let dist2 = to_player.length_squared();
    let yaw = {
        let f = transform.rotation * Vec3::Z;
        f.x.atan2(f.z)
    };
    let step = 11.25f32.to_radians() * ticks;
    let chase = |from: f32, to: f32| {
        let d = (to - from + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        from + d.clamp(-step, step)
    };
    let face_player = to_player.x.atan2(to_player.y);
    // aPOL2_decide_next_move_act.
    let (max, accel) = if !focus.is_world() || dist2 < 50.0 * 50.0 - 50.0 {
        (0.0, 0.0)
    } else if dist2 < 70.0 * 70.0 + 100.0 {
        (1.0, 0.1)
    } else {
        (4.0, 0.4)
    };
    let (now_zone, pl_zone) = (booker_zone(me), booker_zone(pl));
    let target = if max == 0.0 {
        booker.next_zone = None;
        face_player
    } else if pl_zone != now_zone {
        let coin = (time.elapsed_secs() * 7.0) as u8 & 1;
        let next = *booker
            .next_zone
            .get_or_insert_with(|| booker_next_zone(pl_zone, now_zone, coin));
        // aPOL2_search_player2: the next zone's centre.
        const POS_X: [f32; 4] = [60.0, 140.0, 260.0, 340.0];
        const POS_Z: [f32; 5] = [100.0, 140.0, 180.0, 220.0, 300.0];
        let centre = Vec2::new(POS_X[(next % 4) as usize], POS_Z[(next / 4) as usize]);
        if (centre - me).length_squared() < 14.0 * 14.0 + 4.0 {
            booker.next_zone = Some(booker_next_zone(pl_zone, now_zone, coin));
        }
        let d = centre - me;
        d.x.atan2(d.y)
    } else {
        booker.next_zone = None;
        face_player
    };
    let off = (target - yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    let new_yaw = chase(yaw, target);
    transform.rotation = Quat::from_rotation_y(new_yaw);
    // aPOL2_ACT_TURN: on the spot while more than 90 degrees off.
    let (max, accel) = if off.abs() > std::f32::consts::FRAC_PI_2 {
        (0.0, 0.0)
    } else {
        (max, accel)
    };
    booker.speed = if booker.speed < max {
        (booker.speed + accel * ticks).min(max)
    } else {
        (booker.speed - (if max == 0.0 { 0.8 } else { 0.2 }) * ticks).max(max)
    };
    let dir = Vec2::new(new_yaw.sin(), new_yaw.cos());
    let next = me + dir * booker.speed * ticks;
    // Bumping a counter (ClObj_DID_COLLIDE): wait.
    if booker.speed > 0.0 && room.free(next / UNIT) {
        transform.translation.x = POLICE_ORIGIN.x + next.x / UNIT;
        transform.translation.z = POLICE_ORIGIN.z + next.y / UNIT;
    } else {
        booker.speed = 0.0;
    }
    gait.0 = match booker.speed {
        s if s <= 0.05 => 0,
        s if s <= 1.5 => 1,
        _ => 2,
    };
}

#[derive(Component)]
struct KeptItemModel;

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// Booker asked whether to hand over slot `n`.
    Offer(usize),
    Finish,
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
    context: MsgContext,
    /// `_99C`: Booker already greeted this visit.
    greeted: bool,
    inside_for: Option<f32>,
}

fn local(p: Vec2) -> Vec3 {
    POLICE_ORIGIN + Vec3::new(p.x / UNIT, 0.0, p.y / UNIT)
}

fn locate(town: Res<Town>, mut commands: Commands) {
    let Some(b) = town
        .retail
        .as_ref()
        .and_then(|r| r.buildings.iter().find(|b| b.item == POLICE_STATION))
    else {
        return;
    };
    let at = |dx: f32, dz: f32| {
        let p = b.origin + Vec3::new(dx, 0.0, dz) / UNIT;
        p.with_y(town.ground_y(p))
    };
    commands.insert_resource(PoliceBuilding {
        trigger: at(24.0, 40.0),
        exit: at(60.0, 60.0),
    });
    info!("police box door at {:?}", at(24.0, 40.0));
}

/// mPB_police_box_init for a lost and found never filled.
fn init_lost_and_found(
    mut lost: ResMut<LostAndFound>,
    mut grow: ResMut<LostGrowDay>,
    loaded: Res<crate::save::LoadedSave>,
    disc: Option<Res<GameDisc>>,
    mut done: Local<bool>,
) {
    if *done || disc.is_none() {
        return;
    }
    *done = true;
    grow.0 = loaded.0.as_ref().and_then(|s| s.police_grow_day);
    if let Some(saved) = loaded.0.as_ref().and_then(|s| s.police_box) {
        lost.0 = saved;
        return;
    }
    let mut seed = crate::clock::now().day_secs ^ 0x5bd1_e995;
    let mut below = |n: u32| {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        seed % n.max(1)
    };
    let roll = crate::goods::roll_now();
    let mut items = crate::goods::select(&roll, crate::goods::Kind::Furniture, 1, &[], &mut below);
    items.extend(crate::goods::select(
        &roll,
        crate::goods::Kind::Cloth,
        2,
        &[],
        &mut below,
    ));
    lost.0 = [0; KEEP_COUNT];
    for (slot, item) in lost.0.iter_mut().zip(items) {
        *slot = item;
    }
    info!("lost and found starts with {:x?}", &lost.0[..3]);
}

#[allow(clippy::too_many_arguments)]
fn build_room(
    mut commands: Commands,
    disc: Res<GameDisc>,
    mut room: ResMut<PoliceRoom>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    if room.built {
        return;
    }
    room.built = true;
    room.room = IndoorRoom::build(
        &mut commands,
        &disc,
        "police box room",
        POLICE_ORIGIN,
        ("police_indoor_model", Some("police_indoor_modelT")),
        (&mut meshes, &mut materials, &mut images),
    );
    if room.room.is_none() {
        warn!("police box room missing from the disc");
        return;
    }
    let layouts = crate::disc_town::read_fg_layouts(&disc.fgdata);
    if let Some(units) = layouts.get(&FG_POLICE_INDOOR) {
        room.spots = units
            .iter()
            .enumerate()
            .filter_map(|(i, &item)| {
                let slot = item.checked_sub(RSV_POLICE_ITEM_0)? as usize;
                (slot < KEEP_COUNT).then_some(((i % 16, i / 16), slot))
            })
            .collect();
    }
    info!("police box: {} lost-and-found spots", room.spots.len());
    commands.spawn((
        Name::new("Booker"),
        Booker::default(),
        crate::character::NpcGait::default(),
        crate::character::NpcModel(BOOKER_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(local(BOOKER) + Vec3::Y).looking_to(Vec3::NEG_Z, Vec3::Y),
    ));
}

/// bg_police_item: the kept items on their floor units.
#[allow(clippy::too_many_arguments)]
fn draw_items(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    room: Res<PoliceRoom>,
    lost: Res<LostAndFound>,
    drawn: Query<Entity, With<KeptItemModel>>,
    (mut meshes, mut materials, mut images): (
        ResMut<Assets<Mesh>>,
        ResMut<Assets<N64Material>>,
        ResMut<Assets<Image>>,
    ),
) {
    let Some(disc) = disc else { return };
    if !(lost.is_changed() || room.is_changed()) || room.spots.is_empty() {
        return;
    }
    for e in &drawn {
        commands.entity(e).despawn();
    }
    for &((x, z), slot) in &room.spots {
        let item = lost.0[slot];
        if item == 0 {
            continue;
        }
        let Some(model) = crate::field_items::item_meshes(
            &disc,
            item,
            (&mut meshes, &mut materials, &mut images),
        ) else {
            continue;
        };
        // pos_table: the unit's centre, on whatever surface is there.
        let spot = Vec2::new(x as f32 + 0.5, z as f32 + 0.5);
        let top = room
            .room
            .as_ref()
            .and_then(|r| r.height_at(spot))
            .unwrap_or(0.0);
        let at = local(spot * 40.0) + Vec3::Y * (top + 1.0 / UNIT);
        let root = commands
            .spawn((
                KeptItemModel,
                Name::new(format!("lost item {item:04x}")),
                Transform::from_translation(at),
                Visibility::default(),
            ))
            .id();
        for (mesh, material) in model {
            commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), ChildOf(root)));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn enter(
    mut commands: Commands,
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    tool: Res<crate::ecology::EquippedTool>,
    building: Option<Res<PoliceBuilding>>,
    mut next: ResMut<NextState<ShopLocation>>,
    mut player: Query<(&mut Transform, &mut Facing, &Locomotion), With<Player>>,
    mut talk: ResMut<Talk>,
) {
    if *state != ShopLocation::Town
        || *house.get() != crate::house::HouseState::Town
        || !focus.is_world()
    {
        return;
    }
    let (Some(building), Ok((mut transform, mut facing, loco))) = (building, player.single_mut())
    else {
        return;
    };
    // aPBOX_check_player: dist^2 < 1100 game units, facing between north
    // and west (yaw 0x8000..0xC000), walking in (E there works too).
    let d = (transform.translation - building.trigger).with_y(0.0) * UNIT;
    let yaw = facing.dir.x.atan2(facing.dir.z).to_degrees();
    let facing_ok = yaw > -180.0 && yaw < -90.0;
    let pressed =
        actions.just_pressed(Action::Interact) && *tool == crate::ecology::EquippedTool::None;
    let walking = loco.state() != rustimal_logic::player_move::LocomotionState::Wait;
    if d.length_squared() >= 1100.0 || !facing_ok || !(walking || pressed) {
        return;
    }
    if pressed {
        actions.consume(Action::Interact);
    }
    commands.insert_resource(ShopReturnPos(building.exit.with_y(transform.translation.y)));
    transform.translation = local(ENTRY) + Vec3::Y;
    facing.dir = Vec3::NEG_Z;
    talk.greeted = false;
    talk.inside_for = Some(0.0);
    crate::sound::door_sound(None, true);
    next.set(ShopLocation::PoliceBox);
    info!("entered the police box");
}

/// Walking south into the door (or E there) leaves for town.
fn leave(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    mut next: ResMut<NextState<ShopLocation>>,
    return_pos: Option<Res<ShopReturnPos>>,
    mut player: Query<(&mut Transform, &mut Facing), With<Player>>,
    mut lost: ResMut<LostAndFound>,
) {
    if *state != ShopLocation::PoliceBox || !focus.is_world() {
        return;
    }
    let Ok((mut transform, mut facing)) = player.single_mut() else {
        return;
    };
    let near = (transform.translation - local(ENTRY)).with_y(0.0).length() < 0.6;
    let into = actions.pressed(Action::MoveDown) || actions.just_pressed(Action::Interact);
    if !near || facing.dir.z < 0.7 || !into {
        return;
    }
    actions.consume(Action::Interact);
    if let Some(r) = return_pos {
        transform.translation = r.0;
    }
    facing.dir = Vec3::new(1.0, 0.0, 1.0).normalize();
    // mPB_copy_itemBuf: the kept items close up on the way out.
    let kept: Vec<u16> = lost.0.iter().copied().filter(|&i| i != 0).collect();
    lost.0 = [0; KEEP_COUNT];
    for (slot, item) in lost.0.iter_mut().zip(kept) {
        *slot = item;
    }
    crate::sound::door_sound(None, false);
    next.set(ShopLocation::Town);
    info!("left the police box");
}

fn movement(
    actions: Res<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    time: Res<Time>,
    room: Res<PoliceRoom>,
    mut player: Query<(&mut Transform, &mut Facing, &mut Locomotion), With<Player>>,
    booker: Query<&Transform, (With<Booker>, Without<Player>)>,
) {
    if *state != ShopLocation::PoliceBox || !focus.is_world() {
        return;
    }
    let (Ok((mut transform, mut facing, mut loco)), Some(room)) =
        (player.single_mut(), room.room.as_ref())
    else {
        return;
    };
    let before = transform.translation;
    room.walk(
        &actions,
        time.delta_secs(),
        (&mut transform, &mut facing, &mut loco),
    );
    // Booker is solid: no stepping closer once bodies touch.
    if let Ok(b) = booker.single() {
        let gap = |p: Vec3| (p - b.translation).with_y(0.0).length();
        if gap(transform.translation) < 0.7 && gap(transform.translation) < gap(before) {
            transform.translation = before;
        }
    }
}

fn context(identity: &crate::save::PlayerIdentity) -> MsgContext {
    let now = crate::clock::now();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Booker".into(),
        free: vec![String::new(); 20],
        items: vec![String::new(); 5],
        item_articles: vec![0; 5],
        month: now.month,
        day: now.day,
        hour: now.hour,
        minute: now.minute,
        weekday: now.weekday,
        ..default()
    }
}

fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    booker: Entity,
    ctx: &MsgContext,
    msg: usize,
) -> bool {
    let Some(text) = disc.text.as_ref() else {
        return false;
    };
    let (pages, choice) = text.talk(msg, ctx, &mut |_| 0);
    dialogue.begin_special(
        booker,
        "Booker",
        (BOOKER_NPC, BOOKER_SOUND),
        pages,
        choice.as_ref().map(|c| c.labels.clone()),
    );
    info!("police box message {msg:#06x}");
    choice.is_some()
}

#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    time: Res<Time>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    mut player: Query<(&Transform, &Facing, &mut Locomotion), With<Player>>,
    booker: Query<(Entity, &Transform), With<Booker>>,
    room: Res<PoliceRoom>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut lost: ResMut<LostAndFound>,
    identity: Res<crate::save::PlayerIdentity>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::PoliceBox {
        return;
    }
    let Ok((booker, booker_at)) = booker.single() else {
        return;
    };

    if let Some(choice) = dialogue.external_result.take() {
        *focus = InputFocus::Dialogue;
        match std::mem::take(&mut flow.step) {
            Step::Offer(slot) if choice == 0 => {
                // aPOL2_check_answer: into the pockets, or 0x781 when full.
                let item = lost.0[slot];
                if item != 0 && inventory.add(item) {
                    lost.0[slot] = 0;
                    crate::sound::se(crate::sound::se_id::MENU_EXIT);
                    *dialogue = DialogueState::default();
                    *focus = InputFocus::World;
                    info!("lost and found: took {item:#06x}");
                } else {
                    say(&disc, &mut dialogue, booker, &flow.context, 0x0781);
                    flow.step = Step::Finish;
                }
            }
            _ => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
            }
        }
        return;
    }

    // aPOL2_set_force_talk_info_message_ctrl2: greet a player who came in.
    if let Some(since) = flow.inside_for.as_mut() {
        let was = *since;
        *since += time.delta_secs();
        if was < ARRIVAL && *since >= ARRIVAL && matches!(flow.step, Step::Idle) && focus.is_world()
        {
            flow.context = context(&identity);
            if let Ok((_, _, mut loco)) = player.single_mut() {
                *loco = Locomotion::default();
            }
            *focus = InputFocus::Dialogue;
            let msg = if lost.count() == 0 { 0x0784 } else { 0x0785 };
            say(&disc, &mut dialogue, booker, &flow.context, msg);
            flow.greeted = true;
            flow.step = Step::Finish;
            return;
        }
    }

    if !matches!(flow.step, Step::Idle | Step::Finish)
        || !focus.is_world()
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let Ok((at, facing, _)) = player.single() else {
        return;
    };
    flow.context = context(&identity);
    // A facing a kept item: Booker offers it (0x77E).
    let front = at.translation + facing.dir.normalize_or_zero() * 0.75 - POLICE_ORIGIN;
    let unit = (front.x.floor() as i32, front.z.floor() as i32);
    if let Some(&(_, slot)) = room
        .spots
        .iter()
        .find(|((x, z), slot)| (*x as i32, *z as i32) == unit && lost.0[*slot] != 0)
    {
        actions.consume(Action::Interact);
        *focus = InputFocus::Dialogue;
        let item = lost.0[slot];
        flow.context.items[0] = crate::items::name(item);
        flow.context.item_articles[0] = crate::shop_nook::item_article(&disc, item);
        if say(&disc, &mut dialogue, booker, &flow.context, 0x077E) {
            flow.step = Step::Offer(slot);
        } else {
            flow.step = Step::Finish;
        }
        return;
    }
    // aPOL2_set_norm_talk_info_message_ctrl.
    let delta = (booker_at.translation - at.translation).with_y(0.0);
    if delta.length() > 2.0 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    let msg = match (lost.count(), flow.greeted) {
        (0, false) => 0x0786,
        (0, true) => 0x0787,
        _ => 0x077D,
    };
    say(&disc, &mut dialogue, booker, &flow.context, msg);
    flow.step = Step::Finish;
}

pub struct PoliceBoxPlugin;

impl Plugin for PoliceBoxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PoliceRoom>()
            .init_resource::<Talk>()
            .init_resource::<LostAndFound>()
            .init_resource::<LostGrowDay>()
            .add_systems(Startup, locate.after(crate::town::generate_town))
            .add_systems(
                Update,
                (
                    init_lost_and_found,
                    grow_lost_and_found,
                    build_room.run_if(resource_exists::<GameDisc>),
                    draw_items,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (enter, leave)
                    .in_set(InteractSet::Doors)
                    .in_set(GameplaySet),
            )
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet))
            .add_systems(
                Update,
                (movement, booker_follow)
                    .in_set(crate::locomotion::MovementSet)
                    .in_set(GameplaySet),
            );
    }
}
