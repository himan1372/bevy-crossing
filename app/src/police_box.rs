//! The police box (SCENE_POLICE_BOX): its door, the disc room
//! (police_indoor), Booker at the desk and the lost and found.
//!
//! ac_police_box_move: walking in within sqrt(1100) game units of (x + 24,
//! z + 40) facing between north and west enters at (200, 380); leaving puts
//! the player at (x + 60, z + 60) facing south-east. Booker (SP_NPC_POLICE2)
//! stands at unit (4, 6), greets a player who came through the door
//! (0x784 / 0x785 by whether anything is kept) and otherwise answers with
//! 0x786 / 0x787 / 0x77D. The lost and found (mPB, 20 items) lies on the
//! floor at the room's RSV_POLICE_ITEM units; A facing one makes Booker
//! offer it (0x77E): yes takes it if the pockets have room (else 0x781).
//! A new lost and found holds one common furniture item and two shirts
//! (mPB_police_box_init). Copper's post outside is not modelled yet.
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

#[derive(Component)]
struct Booker;

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
    loaded: Res<crate::save::LoadedSave>,
    disc: Option<Res<GameDisc>>,
    mut done: Local<bool>,
) {
    if *done || disc.is_none() {
        return;
    }
    *done = true;
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
        Booker,
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
) {
    if *state != ShopLocation::PoliceBox || !focus.is_world() {
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
            .add_systems(Startup, locate.after(crate::town::generate_town))
            .add_systems(
                Update,
                (
                    init_lost_and_found,
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
                movement
                    .in_set(crate::locomotion::MovementSet)
                    .in_set(GameplaySet),
            );
    }
}
