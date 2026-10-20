//! The museum: its door, the lobby and four exhibit rooms from the disc,
//! Blathers and donating, with the collection saved.
//!
//! ac_museum: walking in within sqrt(1100) game units of (x, z + 100),
//! facing north (yaw 0x6000..0xA000), enters the lobby at (240, 440);
//! leaving returns to (x, z + 120) facing south. Each room's FG layout holds
//! its doors (DOOR_START + n leads from the lobby to painting, fossil,
//! insect and fish; DOOR_START in a room leads back; EXIT_DOOR leaves),
//! entered by walking into them, arriving where the scene door data says.
//! Blathers (SP_NPC_CURATOR) dozes 06:00-18:00. His question (0x2F50) offers
//! donating (the pockets, picking an item), hearing about donating
//! (0x2F5F, 0x2F60) or leaving; his answer to an item is
//! rustimal_logic::museum::Collection::offer. The fossil room shows each
//! donated fossil piece where its plinth stood (mMmd_museum_fossil_data);
//! the gallery hangs each donated painting or its covered frame
//! (aMP_art_data_table). Fish tanks and insect cases are not drawn yet.
use bevy::prelude::*;
use rustimal_logic::museum::{self as native, Collection};

use crate::dialogue::DialogueState;
use crate::disc_gfx::Segments;
use crate::game_disc::GameDisc;
use crate::indoor_room::IndoorRoom;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::locomotion::Locomotion;
use crate::messages::MsgContext;
use crate::n64_material::N64Material;
use crate::player::Player;
use crate::pockets::PocketsUi;
use crate::shop::{ShopLocation, ShopReturnPos};
use crate::states::GameplaySet;
use crate::town::Town;

const UNIT: f32 = 40.0;
/// `MUSEUM` layout item.
const MUSEUM: u16 = 0x584A;
/// Rooms are built side by side from here.
const ORIGIN: Vec3 = Vec3::new(0.0, 0.0, 3000.0);
const ROOM_SPACING: f32 = 40.0;
/// DOOR_START / EXIT_DOOR items in the rooms' FG layouts.
const DOOR_START: u16 = 0x4000;
const EXIT_DOOR: u16 = 0x4080;
/// SP_NPC_CURATOR: npc_draw_data index and l_sp_actor_name sound id.
const BLATHERS_NPC: u32 = 238 + 109;
const BLATHERS_SOUND: u8 = 2;
/// Where Blathers stands in the lobby, game units (at his counter).
const BLATHERS: Vec2 = Vec2::new(260.0, 260.0);

/// One room: model, translucent model, FG type, where you arrive from the
/// lobby, and where the lobby puts you when you come back.
struct RoomDef {
    name: &'static str,
    model: (&'static str, Option<&'static str>),
    fg: u16,
    arrive: Vec2,
    back: Vec2,
}

/// Lobby, then the rooms in DOOR_START order (painting, fossil, insect, fish).
const ROOMS: [RoomDef; 5] = [
    RoomDef {
        name: "museum lobby",
        model: ("rom_museum1_model", Some("rom_museum1_modelT")),
        fg: 0x017F,
        arrive: Vec2::new(240.0, 440.0),
        back: Vec2::ZERO,
    },
    RoomDef {
        name: "museum gallery",
        model: ("rom_museum3_model", Some("rom_museum3_modelT")),
        fg: 0x0180,
        arrive: Vec2::new(280.0, 480.0),
        back: Vec2::new(160.0, 120.0),
    },
    RoomDef {
        name: "museum fossil room",
        model: ("rom_museum2_model", Some("rom_museum2_modelT")),
        fg: 0x0181,
        arrive: Vec2::new(280.0, 480.0),
        back: Vec2::new(320.0, 120.0),
    },
    RoomDef {
        name: "museum insect room",
        model: ("rom_museum4_model", None),
        fg: 0x0182,
        arrive: Vec2::new(520.0, 560.0),
        back: Vec2::new(120.0, 280.0),
    },
    RoomDef {
        name: "museum fish room",
        model: ("rom_museum5_model", Some("rom_museum5_modelT")),
        fg: 0x0183,
        arrive: Vec2::new(120.0, 560.0),
        back: Vec2::new(360.0, 280.0),
    },
];

/// mMmd_museum_fossil_data: unit (x, z), plinth furniture index and turn.
const FOSSILS: [((i32, i32), u16, u16); native::FOSSIL_NUM] = [
    ((5, 2), TRIKERA, 2),
    ((1, 2), TRIKERA, 2),
    ((3, 2), TRIKERA, 2),
    ((9, 2), TREX, 0),
    ((13, 2), TREX, 0),
    ((11, 2), TREX, 0),
    ((5, 10), BRONT, 2),
    ((1, 10), BRONT, 2),
    ((3, 10), BRONT, 2),
    ((10, 6), STEGO_A, 1),
    ((13, 5), STEGO_B, 0),
    ((11, 5), STEGO_B, 0),
    ((6, 6), PTERA, 0),
    ((6, 5), PTERA, 0),
    ((6, 7), PTERA, 0),
    ((9, 10), HUTABA, 0),
    ((11, 10), HUTABA, 0),
    ((13, 10), HUTABA, 0),
    ((2, 7), MAMMOTH, 1),
    ((2, 5), MAMMOTH, 1),
    ((6, 8), SMALL, 0),
    ((7, 8), SMALL, 0),
    ((8, 8), SMALL, 0),
    ((9, 8), SMALL, 0),
    ((10, 8), SMALL, 0),
];
/// FTR_DIN_*_DUMMY furniture indices (m_ftr_def order).
const TRIKERA: u16 = 991;
const TREX: u16 = 992;
const BRONT: u16 = 993;
const PTERA: u16 = 994;
const HUTABA: u16 = 995;
const MAMMOTH: u16 = 996;
const STEGO_A: u16 = 997;
const STEGO_B: u16 = 998;
const SMALL: u16 = 999;

/// aMP_art_data_table: (model number, covered frame number, unit x, z).
const ART: [(u8, u8, f32, f32); native::ART_NUM] = [
    (1, 1, 7.0, 1.0),
    (2, 3, 1.0, 9.0),
    (3, 1, 11.0, 5.0),
    (4, 4, 9.0, 1.0),
    (5, 2, 5.0, 9.0),
    (6, 5, 3.0, 1.0),
    (7, 7, 3.0, 9.0),
    (8, 6, 1.0, 5.0),
    (9, 4, 3.0, 5.0),
    (10, 6, 11.0, 9.0),
    (11, 7, 11.0, 1.0),
    (12, 4, 13.0, 5.0),
    (13, 8, 13.0, 9.0),
    (14, 8, 9.0, 5.0),
    (15, 8, 5.0, 1.0),
];

/// The saved collection.
#[derive(Resource, Default, Clone, Debug)]
pub struct MuseumCollection(pub Collection);

/// The collection in the save: each exhibit's donor (0 none, 1-4 player).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct MuseumSave {
    pub fossil: Vec<u8>,
    pub art: Vec<u8>,
    pub insect: Vec<u8>,
    pub fish: Vec<u8>,
}

impl MuseumCollection {
    pub fn save(&self) -> MuseumSave {
        MuseumSave {
            fossil: self.0.fossil.to_vec(),
            art: self.0.art.to_vec(),
            insect: self.0.insect.to_vec(),
            fish: self.0.fish.to_vec(),
        }
    }

    pub fn restore(&mut self, saved: Option<&MuseumSave>) {
        let mut c = Collection::default();
        if let Some(s) = saved {
            let fill = |dst: &mut [u8], src: &[u8]| {
                for (d, s) in dst.iter_mut().zip(src) {
                    *d = *s;
                }
            };
            fill(&mut c.fossil, &s.fossil);
            fill(&mut c.art, &s.art);
            fill(&mut c.insect, &s.insect);
            fill(&mut c.fish, &s.fish);
        }
        self.0 = c;
    }
}

fn load_collection(loaded: Res<crate::save::LoadedSave>, mut collection: ResMut<MuseumCollection>) {
    collection.restore(loaded.0.as_ref().and_then(|s| s.museum.as_ref()));
}

#[derive(Resource)]
struct MuseumBuilding {
    trigger: Vec3,
    exit: Vec3,
}

#[derive(Resource, Default)]
pub(crate) struct Museum {
    built: bool,
    pub(crate) rooms: Vec<Option<IndoorRoom>>,
    /// Each room's FG layout (doors).
    layouts: Vec<[u16; 256]>,
    /// The room shown.
    room: usize,
}

pub(crate) fn room_origin(room: usize) -> Vec3 {
    ORIGIN + Vec3::X * ROOM_SPACING * room as f32
}

fn at(room: usize, p: Vec2) -> Vec3 {
    room_origin(room) + Vec3::new(p.x / UNIT, 0.0, p.y / UNIT)
}

#[derive(Component)]
struct Blathers {
    doze: Doze,
}

/// aCR_ACTION_WAIT / SLEEP_WAIT / SLEEP.
#[derive(Clone, Copy, PartialEq)]
enum Doze {
    Wait,
    /// Standing a while (aCR_SLEEP_WAIT_TIMER, 360 frames) before nodding off.
    SleepWait(f32),
    /// Asleep (wait_nemu1).
    Sleep,
}

/// aCR_SLEEP_WAIT_TIMER.
const SLEEP_WAIT_FRAMES: f32 = 360.0;

#[derive(Component)]
struct Exhibit;

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// A question is up; the message asking it (`mMsg_Get_msg_num`).
    Asked(usize),
    /// A line before the pockets open (0x2F62).
    ThenPockets,
    /// The pockets are open to pick an item.
    Picking,
    Finish,
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
    context: MsgContext,
    /// aNPC_SPNPC_BIT_CURATOR: talked since coming in from town.
    talked: bool,
}

fn locate(town: Res<Town>, mut commands: Commands) {
    let Some(b) = town
        .retail
        .as_ref()
        .and_then(|r| r.buildings.iter().find(|b| b.item == MUSEUM))
    else {
        return;
    };
    let pos = |dx: f32, dz: f32| {
        let p = b.origin + Vec3::new(dx, 0.0, dz) / UNIT;
        p.with_y(town.ground_y(p))
    };
    commands.insert_resource(MuseumBuilding {
        trigger: pos(0.0, 100.0),
        exit: pos(0.0, 120.0),
    });
    info!("museum door at {:?}", pos(0.0, 100.0));
}

fn build(
    mut commands: Commands,
    disc: Res<GameDisc>,
    mut museum: ResMut<Museum>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    if museum.built {
        return;
    }
    museum.built = true;
    let layouts = crate::disc_town::read_fg_layouts(&disc.fgdata);
    // The insect garden takes the season's ground textures.
    let ground = crate::disc_town::ground_segments(&disc.catalog, crate::disc_town::current_term());
    for (i, def) in ROOMS.iter().enumerate() {
        let segs = if i == 3 {
            ground.clone()
        } else {
            Segments::default()
        };
        let room = IndoorRoom::build_with(
            &mut commands,
            &disc,
            def.name,
            room_origin(i),
            def.model,
            &segs,
            (&mut meshes, &mut materials, &mut images),
        );
        if room.is_none() {
            warn!("{} missing from the disc", def.name);
        }
        // The garden beds, rocks and pond rise a little: keep off them.
        let mut room = room;
        if let (3, Some(room)) = (i, room.as_mut()) {
            let raised = (0..16)
                .flat_map(|x| (0..16).map(move |z| (x, z)))
                .filter(|&(x, z)| {
                    room.height_at(Vec2::new(x as f32 + 0.5, z as f32 + 0.5))
                        .is_some_and(|h| h.abs() > 0.05)
                })
                .collect();
            room.set_furniture(raised);
        }
        museum.rooms.push(room);
        // Ef_Room_Sunshine_Minsect: the insect room's upper garden layer.
        if i == 3 {
            let _ = IndoorRoom::build_with(
                &mut commands,
                &disc,
                "museum insect garden",
                room_origin(i),
                ("rom_museum4_ue_model", None),
                &segs,
                (&mut meshes, &mut materials, &mut images),
            );
        }
        museum
            .layouts
            .push(layouts.get(&def.fg).copied().unwrap_or([0; 256]));
    }
    commands.spawn((
        Name::new("Blathers"),
        Blathers {
            doze: Doze::SleepWait(SLEEP_WAIT_FRAMES),
        },
        crate::character::NpcClip::default(),
        crate::character::NpcModel(BLATHERS_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at(0, BLATHERS) + Vec3::Y).looking_to(Vec3::NEG_Z, Vec3::Y),
    ));
}

/// The fossil plinths/pieces and the paintings, rebuilt when the
/// collection changes (mMmd_MakeMuseumDisplayData / aMP draw).
#[allow(clippy::too_many_arguments)]
fn exhibits(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    mut museum: ResMut<Museum>,
    collection: Res<MuseumCollection>,
    shown: Query<Entity, With<Exhibit>>,
    (mut meshes, mut materials, mut images): (
        ResMut<Assets<Mesh>>,
        ResMut<Assets<N64Material>>,
        ResMut<Assets<Image>>,
    ),
) {
    let Some(disc) = disc else { return };
    if !museum.built || !(collection.is_changed() || museum.is_changed()) {
        return;
    }
    for e in &shown {
        commands.entity(e).despawn();
    }
    // Fossils: the piece itself (FTR_DIN_TRIKERA_HEAD + index) or the plinth.
    let corner = room_origin(2);
    let mut solid = Vec::new();
    for (i, &((x, z), dummy, turn)) in FOSSILS.iter().enumerate() {
        let item = if collection.0.fossil[i] != 0 {
            native::FOSSIL_FIRST + i as u16 * 4 + turn
        } else {
            0x1000 + dummy * 4 + turn
        };
        solid.extend(crate::furniture::footprint(&disc, item, (x, z)));
        if let Some(e) = crate::furniture::spawn_furniture(
            &mut commands,
            &disc,
            item,
            (x, z),
            corner,
            0.0,
            &mut meshes,
            &mut materials,
            &mut images,
        ) {
            commands.entity(e).insert(Exhibit);
        }
    }
    // Plinths and fossils are solid (their FG units).
    if let Some(Some(room)) = museum.bypass_change_detection().rooms.get_mut(2) {
        room.set_furniture(solid);
    }
    // Paintings: the art or its covered frame, at y 40.
    for (i, &(art, dummy, x, z)) in ART.iter().enumerate() {
        let name = if collection.0.art[i] != 0 {
            format!("obj_art{art:02}_model")
        } else {
            format!("obj_art_dummy{dummy:02}_model")
        };
        let Some(list) = disc.catalog.get(&name).map(|a| a.offset) else {
            continue;
        };
        let root = commands
            .spawn((
                Exhibit,
                Name::new(name),
                Transform::from_translation(
                    at(1, Vec2::new(x * 40.0 + 20.0, z * 40.0 + 20.0)) + Vec3::Y,
                )
                .with_scale(Vec3::splat(0.01 / UNIT)),
                Visibility::default(),
            ))
            .id();
        let sink = crate::disc_gfx::run_display_lists(&disc.rom, &[list]);
        crate::interior::spawn_sink(
            &mut commands,
            &disc,
            root,
            &sink,
            None,
            &crate::disc_gfx::Segments::default(),
            false,
            &mut meshes,
            &mut materials,
            &mut images,
        );
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
    building: Option<Res<MuseumBuilding>>,
    mut next: ResMut<NextState<ShopLocation>>,
    mut player: Query<(&mut Transform, &mut Facing, &Locomotion), With<Player>>,
    (mut museum, mut talk): (ResMut<Museum>, ResMut<Talk>),
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
    // aMsm_check_player: dist^2 < 1100 game units of (x, z + 100), facing
    // north (between 135 and 225 degrees), walking in (E there too).
    let d = (transform.translation - building.trigger).with_y(0.0) * UNIT;
    let yaw = facing.dir.x.atan2(facing.dir.z).to_degrees();
    let pressed =
        actions.just_pressed(Action::Interact) && *tool == crate::ecology::EquippedTool::None;
    let walking = loco.state() != rustimal_logic::player_move::LocomotionState::Wait;
    if d.length_squared() >= 1100.0 || yaw.abs() <= 135.0 || !(walking || pressed) {
        return;
    }
    if pressed {
        actions.consume(Action::Interact);
    }
    commands.insert_resource(ShopReturnPos(building.exit.with_y(transform.translation.y)));
    museum.room = 0;
    transform.translation = at(0, ROOMS[0].arrive) + Vec3::Y;
    facing.dir = Vec3::NEG_Z;
    // Coming in from town clears his first-talk flag (aCR_actor_ct).
    talk.talked = false;
    crate::sound::door_sound(None, true);
    next.set(ShopLocation::Museum);
    info!("entered the museum");
}

/// Walking into a door item: another room, or back to town.
fn doors(
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    mut next: ResMut<NextState<ShopLocation>>,
    return_pos: Option<Res<ShopReturnPos>>,
    mut museum: ResMut<Museum>,
    mut player: Query<(&mut Transform, &mut Facing, &Locomotion), With<Player>>,
) {
    if *state != ShopLocation::Museum || !focus.is_world() {
        return;
    }
    let Ok((mut transform, mut facing, loco)) = player.single_mut() else {
        return;
    };
    if loco.state() == rustimal_logic::player_move::LocomotionState::Wait {
        return;
    }
    let room = museum.room;
    let front = transform.translation + facing.dir.normalize_or_zero() * 0.5 - room_origin(room);
    let (x, z) = (front.x.floor(), front.z.floor());
    if !(0.0..16.0).contains(&x) || !(0.0..16.0).contains(&z) {
        return;
    }
    let item = museum
        .layouts
        .get(room)
        .map_or(0, |l| l[z as usize * 16 + x as usize]);
    let to = match item {
        EXIT_DOOR => {
            if let Some(r) = return_pos {
                transform.translation = r.0;
            }
            facing.dir = Vec3::Z;
            crate::sound::door_sound(None, false);
            next.set(ShopLocation::Town);
            info!("left the museum");
            return;
        }
        d if (DOOR_START..DOOR_START + 4).contains(&d) => {
            if room == 0 {
                (d - DOOR_START) as usize + 1
            } else {
                0
            }
        }
        _ => return,
    };
    let arrive = if to == 0 {
        ROOMS[room].back
    } else {
        ROOMS[to].arrive
    };
    museum.room = to;
    transform.translation = at(to, arrive) + Vec3::Y;
    // Face away from the door just used.
    facing.dir = -facing.dir;
    crate::sound::door_sound(None, true);
    info!("museum room {room} -> {to}");
}

fn movement(
    actions: Res<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    time: Res<Time>,
    museum: Res<Museum>,
    mut player: Query<(&mut Transform, &mut Facing, &mut Locomotion), With<Player>>,
    owl: Query<&Transform, (With<Blathers>, Without<Player>)>,
) {
    if *state != ShopLocation::Museum || !focus.is_world() {
        return;
    }
    let (Ok((mut transform, mut facing, mut loco)), Some(Some(room))) =
        (player.single_mut(), museum.rooms.get(museum.room))
    else {
        return;
    };
    let before = transform.translation;
    room.walk(
        &actions,
        time.delta_secs(),
        (&mut transform, &mut facing, &mut loco),
    );
    // Blathers is solid: no stepping closer once bodies touch.
    if let (0, Ok(b)) = (museum.room, owl.single()) {
        let gap = |p: Vec3| (p - b.translation).with_y(0.0).length();
        if gap(transform.translation) < 0.75 && gap(transform.translation) < gap(before) {
            transform.translation = before;
        }
    }
}

fn context(identity: &crate::save::PlayerIdentity) -> MsgContext {
    let now = crate::clock::now();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Blathers".into(),
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

/// Shows `msgs` one after another in one window (mMsg_Set_continue_msg_num)
/// and returns the message asking the question it ends on, if any.
fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &MsgContext,
    msgs: &[usize],
) -> Option<usize> {
    let text = disc.text.as_ref()?;
    let mut pages = Vec::new();
    let mut asked = None;
    for &msg in msgs {
        let (more, choice) = text.talk_asking(msg, ctx, &mut |_| 0);
        pages.extend(more);
        asked = choice;
        info!("museum message {msg:#06x}");
    }
    dialogue.begin_special(
        actor,
        "Blathers",
        (BLATHERS_NPC, BLATHERS_SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    asked.map(|(no, _)| no)
}

/// Where a choice leads by the message script (`MsgChoice::next`).
fn choice_target(disc: &GameDisc, ctx: &MsgContext, asked: usize, choice: usize) -> Option<usize> {
    let text = disc.text.as_ref()?;
    let (_, c) = text.talk_asking(asked, ctx, &mut |_| 0);
    c.and_then(|(_, c)| c.next.get(choice).copied().flatten())
}

/// aCR_think_main_proc / aCR_sleep_wait: by day he stands a while, then
/// sleeps; by night he is awake.
fn doze(
    time: Res<Time>,
    state: Res<State<ShopLocation>>,
    dialogue: Res<DialogueState>,
    mut owl: Query<(Entity, &mut Blathers, &mut crate::character::NpcClip)>,
) {
    let Ok((entity, mut owl, mut clip)) = owl.single_mut() else {
        return;
    };
    let talking = dialogue.active && dialogue.speaker == Some(entity);
    if talking || (state.is_changed() && *state.get() == ShopLocation::Museum) {
        // aCR_talk_init, or the actor made afresh: WAIT.
        owl.doze = Doze::Wait;
    }
    let night = !native::asleep(crate::clock::now().day_secs);
    owl.doze = match owl.doze {
        _ if night => Doze::Wait,
        Doze::Wait if !talking => Doze::SleepWait(SLEEP_WAIT_FRAMES),
        Doze::SleepWait(t) => {
            let t = t - time.delta_secs() * 60.0;
            if t <= 0.0 {
                Doze::Sleep
            } else {
                Doze::SleepWait(t)
            }
        }
        d => d,
    };
    let want = (owl.doze == Doze::Sleep).then_some("cKF_ba_r_npc_1_wait_nemu1");
    if clip.name != want {
        match want {
            Some(name) => clip.start(name, false, 8.0),
            None => clip.stop(8.0),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    museum: Res<Museum>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), With<Player>>,
    blathers: Query<(Entity, &Transform, &Blathers)>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut pockets: ResMut<PocketsUi>,
    mut inventory: ResMut<PlayerInventory>,
    mut collection: ResMut<MuseumCollection>,
    identity: Res<crate::save::PlayerIdentity>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Museum {
        return;
    }
    let Ok((actor, actor_at, owl)) = blathers.single() else {
        return;
    };
    let open_pockets =
        |pockets: &mut PocketsUi, focus: &mut InputFocus, dialogue: &mut DialogueState| {
            *pockets = PocketsUi {
                open: true,
                donating: true,
                ..default()
            };
            *dialogue = DialogueState::default();
            *focus = InputFocus::Pockets;
            crate::sound::se(crate::sound::se_id::MENU_PAUSE);
        };
    let asked_or_finish = |asked: Option<usize>| asked.map_or(Step::Finish, Step::Asked);

    // aCR_menu_open_wait: his line read, the pockets open.
    if matches!(flow.step, Step::ThenPockets) {
        if !dialogue.active {
            open_pockets(&mut pockets, &mut focus, &mut dialogue);
            flow.step = Step::Picking;
        }
        return;
    }

    // aCR_msg_win_open_wait: the pockets closed after picking (or not).
    if matches!(flow.step, Step::Picking) {
        if pockets.open {
            return;
        }
        pockets.donating = false;
        *focus = InputFocus::Dialogue;
        let msgs = match pockets.sell_slot.take() {
            Some(slot) => {
                let item = inventory.pockets.pockets[slot];
                flow.context.items[0] = crate::items::name(item);
                flow.context.item_articles[0] = crate::shop_nook::item_article(&disc, item);
                let pick = (crate::clock::now().day_secs ^ slot as u32).wrapping_mul(2654435761);
                let answer = collection.0.offer(item, 1, pick >> 16);
                let mut msgs = vec![answer.msg];
                if answer.accepted {
                    inventory.pockets.pockets[slot] = 0;
                    inventory.pockets.set_item_cond(slot, 0);
                    msgs.extend(collection.0.after_donation(item));
                    info!("donated {item:#06x} to the museum");
                }
                msgs
            }
            None => vec![0x2F63],
        };
        let asked = say(&disc, &mut dialogue, actor, &flow.context, &msgs);
        flow.step = asked_or_finish(asked);
        return;
    }

    if let Some(choice) = dialogue.external_result.take() {
        *focus = InputFocus::Dialogue;
        let Step::Asked(asked) = std::mem::take(&mut flow.step) else {
            *dialogue = DialogueState::default();
            *focus = InputFocus::World;
            return;
        };
        let target = choice_target(&disc, &flow.context, asked, choice);
        let after_talk = collection.0.after_talk();
        // aCR_chk_request / chk_decide_to_donate(2) / chk_continue_to_donate.
        let next: Option<(Vec<usize>, bool)> = match (asked, choice) {
            (0x2F50 | 0x2F60 | 0x2F66 | 0x2F6F | 0x2F73, 0) => Some((vec![0x2F62], true)),
            (0x2F50, 2) | (0x2F66 | 0x2F6F, _) => Some((vec![after_talk], false)),
            _ => target.map(|t| (vec![t], false)),
        };
        match next {
            Some((msgs, pockets_after)) => {
                let asked = say(&disc, &mut dialogue, actor, &flow.context, &msgs);
                flow.step = if pockets_after {
                    Step::ThenPockets
                } else {
                    asked_or_finish(asked)
                };
            }
            None => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
            }
        }
        return;
    }

    if museum.room != 0
        || !matches!(flow.step, Step::Idle | Step::Finish)
        || !focus.is_world()
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let Ok((at_, facing)) = player.single() else {
        return;
    };
    let delta = (actor_at.translation - at_.translation).with_y(0.0);
    if delta.length() > 2.0 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    flow.context = context(&identity);
    let complete = collection.0.count() == native::TOTAL;
    let msg = native::greeting(
        complete,
        !flow.talked,
        crate::clock::now().day_secs,
        matches!(owl.doze, Doze::SleepWait(_)),
    );
    flow.talked = true;
    // aCR_set_talk_info: a local player gets his question unless done.
    let asked = say(&disc, &mut dialogue, actor, &flow.context, &[msg]);
    flow.step = if complete {
        Step::Finish
    } else {
        asked_or_finish(asked)
    };
}

pub struct MuseumPlugin;

impl Plugin for MuseumPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Museum>()
            .init_resource::<Talk>()
            .init_resource::<MuseumCollection>()
            .add_systems(
                Startup,
                (locate.after(crate::town::generate_town), load_collection),
            )
            .add_systems(
                Update,
                (build.run_if(resource_exists::<GameDisc>), exhibits).chain(),
            )
            .add_systems(
                Update,
                (enter, doors)
                    .in_set(InteractSet::Doors)
                    .in_set(GameplaySet),
            )
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet))
            .add_systems(Update, doze.in_set(GameplaySet))
            .add_systems(
                Update,
                movement
                    .in_set(crate::locomotion::MovementSet)
                    .in_set(GameplaySet),
            );
    }
}
