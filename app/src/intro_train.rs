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
/// `ALL_NPC_NUM + (SP_NPC_SLEEP_OBABA - SP_NPC_START)`, at unit (4, 4) less the (6, 24) her `ct` takes off.
const OBABA_NPC: u32 = 238 + 56;
const OBABA_AT: Vec2 = Vec2::new(174.0, 156.0);

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

/// `aNSO_sleep` / `aNSO_cramp`: the old woman nods (`kokkuri_d1`) two to four times, twitches once
/// (`kokkuri_d2`) and goes back to sleep. Holds the number of nods this round.
#[derive(Component)]
struct Dozing(u32);

/// The tunnel wall of the window scenery: the train starts in a tunnel and comes out of it when Rover
/// sits down (`Common_Set(sunlight_flag, TRUE)` in `aNGD_sitdown`).
#[derive(Component)]
struct TunnelWall;

/// The trees of the window scenery and their mesh's unscrolled texture coordinates (`TreeScrollx`: the
/// tile slides 5 quarter texels of the 128 wide texture per frame).
#[derive(Component)]
struct TreeScroll {
    mesh: Handle<Mesh>,
    uvs: Vec<[f32; 2]>,
}

/// Rover's sound id (`l_sp_actor_name`: 2, like Nook's) and name.
const ROVER_SOUND: u8 = 2;
/// `aNGD_approach`: Rover stops at z = 290.
const APPROACH_Z: f32 = 290.0;
/// `aNGD_set_walk_spd`: at most 1 game unit per frame.
const WALK_SPEED: f32 = 1.0;
/// `aNGD_sitdown` / `aNGD_sitdown2`: where Rover sits.
const SEAT: Vec2 = Vec2::new(100.0, 280.0);
/// `aNGD_move_ready` .. `aNGD_move_to_deck`: the walk to the door end of the car.
const STAND: Vec2 = Vec2::new(100.0, 300.0);
const AISLE: Vec2 = Vec2::new(140.0, 290.0);
const DECK: Vec2 = Vec2::new(140.0, 130.0);
/// `aNGD_set_walk_spd2`: at most 1.5 game units per frame.
const FAST_WALK: f32 = 1.5;
/// `aNGD_set_talk_info_keitai_on` / `_last_talk_start_wait`.
const PHONE_MESSAGE: usize = 0x2AD0;
const LAST_MESSAGE: usize = 0x2AD1;
/// The clips of `aNGD_set_animation` (`cKF_ba_r_npc_1_*`).
mod clip {
    pub const SIT: &str = "cKF_ba_r_npc_1_sitdown_d1";
    pub const SIT_WAIT: &str = "cKF_ba_r_npc_1_sitdown_wait_d1";
    pub const STAND_UP: &str = "cKF_ba_r_npc_1_standup_d1";
    pub const TO_DECK: &str = "cKF_ba_r_npc_1_to_deck_d1";
    pub const PHONE_ON: &str = "cKF_ba_r_npc_1_keitai_on1";
    pub const PHONE_TALK: &str = "cKF_ba_r_npc_1_keitai_talk1";
    pub const PHONE_OFF: &str = "cKF_ba_r_npc_1_keitai_off1";
    pub const OPEN_DOOR: &str = "cKF_ba_r_npc_1_open_d2";
}
/// Blend frames between clips.
const MORPH: f32 = 8.0;
/// The camera's height, in game units, while Rover stands and while he sits on the bench: a little
/// below his head, so the head clears the speech box (his head is about 33 units up standing and
/// about 50 on the bench).
const STANDING_HEAD: f32 = 25.0;
const SEATED_HEAD: f32 = 38.0;
/// How much of the gap to the goal the camera closes per 60 Hz frame: slow, so it glides.
const CAMERA_EASE: f32 = 0.035;
/// Frames Rover takes to slide to the bench while he sits down.
const SIT_FRAMES: f32 = 36.0;
/// `aNGD_talk_start_wait`'s first message ("Excuse me... do you have a second?").
const FIRST_MESSAGE: usize = 0x2AC6;
/// `mDemo_ORDER_NPC0` as the message codes raise it, and the index Rover's script waits on.
const ORDER_NPC0: u8 = 1;
const ORDER_STEP: u8 = 9;

/// The window Rover's next "open it" order (NPC0 index 9 = 1) brings up
/// (`aNGD_ACTION_*_MENU_OPEN_WAIT`).
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Entry {
    /// The time menu (`mSM_OVL_TIMEIN`).
    #[default]
    Time,
    Player,
    Country,
}

/// Where Rover's part of the scene has got to (`aNGD_ACTION_*`).
#[derive(Default)]
enum Step {
    #[default]
    Walk,
    Turn,
    /// A message is up; the question it ends on, if any.
    Talk(Option<(usize, MsgChoice)>),
    /// A window is open; its answer goes back into the script.
    Window(Entry),
    /// Rover sits down (`aNGD_sitdown`); then the message `next`.
    Sit {
        next: usize,
    },
    /// `aNGD_standup` .. `aNGD_move_to_deck`: stands, walks to the door end and steps out on the deck.
    Stand,
    Aisle,
    Door,
    Deck,
    /// `aNGD_keitai_on`: the phone comes out and Rover rings Nook (message 0x2AD0).
    PhoneOn,
    PhoneOff,
    OpenDoor,
    /// `aNGD_return_approach`: back to the player, then the last talk.
    Return,
    Finished,
}

#[derive(Resource, Default)]
struct Script {
    step: Step,
    entry: Entry,
    /// The message the current chain started with.
    current: usize,
    /// The script raised its "open the window" order during this message.
    ask: bool,
    /// `guide->answer_flags`: which of the four rude answers were given (`aNGD_check_talk_msg_no`).
    flags: u8,
    /// `Now_Private->gender`, once Rover has worked it out.
    male: Option<bool>,
    /// Rover is on the bench: the camera comes down to his level.
    sat: bool,
    /// `camera_tilt_goal_angle` / `camera_tilt_add`: the camera leans out past the bench to see him
    /// at the door end (`aNGD_move_to_door`, `aNGD_open_door`).
    tilt_goal: f32,
    tilt_add: f32,
    /// Rover sliding to the bench: where he started (game units) and how far along (0 to 1).
    slide: Option<(Vec2, f32)>,
}

/// `aNGD_msg_win_open_wait`: the message that follows each window.
const AFTER_WINDOW: [usize; 3] = [0x2AE5, 0x2ACA, 0x2AE4];
/// `aNGD_sex_select_after`.
const COUNTRY_ASK: usize = 0x2AD8;
/// The messages whose closing order (9 = 1) asks for a window: the name (0x2AC8, 0x2AC9, 0x2AE2,
/// 0x2AE3), the time (0x2AE1, 0x2AE6) and the destination (0x2AD8), or the apology 0x2ADF / 0x2AE0
/// that goes on to the destination. 0x2AD6 / 0x2AD7 raise 1 / 2 for the gender instead.
const ASKS: [usize; 9] = [
    0x2AC8,
    0x2AC9,
    0x2AE2,
    0x2AE3,
    0x2AE1,
    0x2AE6,
    COUNTRY_ASK,
    0x2ADF,
    0x2AE0,
];

/// The text of an eight-byte name field.
fn name_text(bytes: &[u8; 8]) -> String {
    let len = bytes
        .iter()
        .rposition(|&c| c != rustimal_logic::editor_keyboard::CHAR_SPACE)
        .map_or(0, |i| i + 1);
    bytes[..len]
        .iter()
        .map(|&c| crate::messages::char_of(c).unwrap_or('?'))
        .collect()
}

/// `rom_train_in_modelT` is two translucent parts, the window glass (`rom_train_glass_tex`, aqua) and the
/// ceiling lamps' glow (`rom_train_light_tex`). Drawn the way the room draws them the glass is a pale
/// aqua sheet that washes the scenery out to white, which the retail windows are not, so only the lamps'
/// glow is drawn.
fn spawn_ceiling_lights(
    commands: &mut Commands,
    disc: &GameDisc,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) {
    let Some(list) = disc.catalog.get("rom_train_in_modelT") else {
        return;
    };
    let mut sink = crate::disc_gfx::run_ground_lists(&disc.rom, &[list.offset]);
    if sink.parts.len() > 1 {
        sink.parts.remove(0);
    }
    // As `IndoorRoom::build`: the room's 16x ground units, its floor tier at model height 640.
    let scale = crate::town::CELL_SIZE / 16.0 / UNIT;
    let root = commands
        .spawn((
            Name::new("train lamp glow"),
            Transform::from_translation(TRAIN_ORIGIN - Vec3::Y * 640.0 * scale)
                .with_scale(Vec3::splat(scale)),
            Visibility::default(),
        ))
        .id();
    for (mesh, material) in crate::disc_town::sink_meshes_scaled(
        disc,
        &sink,
        None,
        true,
        &crate::disc_gfx::Segments::default(),
        1.0,
        (meshes, materials, images),
    ) {
        commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), ChildOf(root)));
    }
}

/// `TRAIN_DOOR` in the train layouts (`fgdata` 0xC9 / 0xCA): unit (3, 2).
const DOOR_AT: Vec2 = Vec2::new(140.0, 100.0);

/// `ac_train_door.c`: the door at the end of the car, `cKF_ba_r_obj_romtrain_door` held at its first
/// frame until it is told to open; it then plays through at half speed and closes again.
#[derive(Component)]
struct TrainDoor {
    prop: crate::prop::Prop,
    anim: Option<std::sync::Arc<crate::skeleton::Animation>>,
    frame: f32,
    speed: f32,
}

/// Seconds until the door's `open_flag` is raised (Rover reaches the frame that opens it).
#[derive(Resource, Default)]
struct DoorCue(Option<f32>);

fn spawn_door(
    commands: &mut Commands,
    disc: &GameDisc,
    assets: (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) {
    let scale = crate::disc_town::OBJECT_SCALE;
    let holder = commands
        .spawn((
            Name::new("train door"),
            Transform::from_translation(local(DOOR_AT)).with_scale(Vec3::splat(scale)),
            Visibility::default(),
        ))
        .id();
    let Some((_, prop)) = crate::prop::Prop::spawn(
        commands,
        disc,
        "cKF_bs_r_obj_romtrain_door",
        holder,
        Transform::default(),
        assets,
    ) else {
        warn!("the train door is not on the disc");
        return;
    };
    let anim = prop.animation(disc, "cKF_ba_r_obj_romtrain_door");
    commands.entity(holder).insert((
        TrainDoor {
            prop,
            anim,
            frame: 1.0,
            speed: 0.0,
        },
        Posed::default(),
    ));
}

/// `Train_Window_Actor_draw`: what the windows look at, drawn at the field origin in 0.05 scale: the
/// tunnel wall, the sky, the clouds, the trees and the shine on the glass. The trees' palette is the
/// date's (`aTrainWindow_GetTreePalletIdx`, segment 10).
fn spawn_scenery(
    commands: &mut Commands,
    disc: &GameDisc,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) {
    let now = crate::clock::now();
    let mut segments = crate::disc_gfx::Segments::default();
    if let Some(table) = disc.catalog.get("aTrainWindow_tree_pal_table") {
        let index = rustimal_logic::intro::tree_palette(now.month, now.day);
        let at = table.offset + (index * 32) as u32;
        if let Some(bytes) = disc.rom.slice(disc::RomSource::Rel, at, 32) {
            segments.set_segment(10, bytes.into());
        }
    }
    // `aTrainWindow_SetLightPrimColorDetail`: the scenery is tinted by the room's ambient plus the sun's
    // colour for the time of day, with the alpha and LOD fraction the draw sets for each list.
    let secs = now.day_secs;
    let light = crate::environment::LightTable::from_disc(disc).map(|table| table.at(secs));
    let tint = |offset: [f32; 3], alpha: f32| -> [u8; 4] {
        let base = light.map_or([255.0; 3], |l| {
            [0, 1, 2].map(|i| l.ambient[i] + l.sun_color[i])
        });
        let c = |i: usize| (base[i] + offset[i]).clamp(0.0, 255.0) as u8;
        [c(0), c(1), c(2), alpha.clamp(0.0, 255.0) as u8]
    };
    // `aTW_GetNowAlpha` and `lod_factor`: the scenery fades with the daylight.
    let day = f32::from(secs > 14_400 && secs < 72_000);
    let xlu = if (14_400..72_000).contains(&secs) {
        if secs < 43_200 {
            255.0 * (secs - 14_400) as f32 / 28_800.0
        } else {
            255.0 * (1.0 - (secs - 43_200) as f32 / 28_800.0)
        }
    } else if secs < 14_400 {
        200.0 * (1.0 - (0.5 + secs as f32 / 28_800.0))
    } else {
        200.0 * ((secs - 72_000) as f32 / 28_800.0)
    };
    let glass_lod = if day == 0.0 {
        0.0
    } else if secs >= 43_200 {
        160.0 - 160.0 * (secs - 43_200) as f32 / 28_800.0
    } else {
        160.0 * (secs - 14_400) as f32 / 28_800.0
    };
    let root = commands
        .spawn((
            Name::new("train window scenery"),
            Transform::from_translation(TRAIN_ORIGIN).with_scale(Vec3::splat(0.05 / UNIT)),
            Visibility::default(),
        ))
        .id();
    // The opaque lists, then the translucent ones (`NOW_POLY_OPA_DISP` / `NOW_POLY_XLU_DISP`), each with
    // the colour, alpha and LOD fraction `Train_Window_Actor_draw` sets before it.
    let lists: [(&str, bool, [u8; 4], u8); 5] = [
        (
            "rom_train_out_tunnel_model",
            false,
            tint([0.0; 3], 255.0),
            43,
        ),
        (
            "rom_train_out_bgsky_model",
            false,
            tint([0.0; 3], 255.0),
            127,
        ),
        (
            "rom_train_out_bgcloud_modelT",
            true,
            tint([0.0; 3], xlu),
            127,
        ),
        (
            "rom_train_out_bgtree_modelT",
            true,
            tint([-80.0, -70.0, -160.0], xlu),
            27,
        ),
        (
            "rom_train_out_shineglass_modelT",
            true,
            tint([0.0; 3], xlu),
            glass_lod as u8,
        ),
    ];
    for (name, blend, prim, lod) in lists {
        {
            let Some(list) = disc.catalog.get(name) else {
                continue;
            };
            let sink =
                crate::disc_gfx::run_display_lists_with_prim(&disc.rom, &[list.offset], prim, lod);
            for (mesh, material) in crate::disc_town::sink_meshes_scaled(
                disc,
                &sink,
                None,
                blend,
                &segments,
                1.0,
                (&mut *meshes, &mut *materials, &mut *images),
            ) {
                // The scene has no distance fog (the town's would wash the backdrop out to white).
                if let Some(mut m) = materials.get_mut(&material) {
                    m.combiner.flags &= !crate::n64_material::FLAG_FOG;
                }
                if name.contains("bgsky") {
                    // The sky texture runs blue, lilac, yellow, cream, then a dark horizon. Through the
                    // low camera's windows the cream and the dark band took over, so only the part down
                    // to the yellow is stretched over the backdrop: blue at the top, yellow below.
                    if let Some(mut sky) = meshes.get_mut(&mesh) {
                        if let Some(bevy::mesh::VertexAttributeValues::Float32x2(uvs)) =
                            sky.attribute_mut(Mesh::ATTRIBUTE_UV_0)
                        {
                            for uv in uvs.iter_mut() {
                                uv[1] *= 0.58;
                            }
                        }
                    }
                }
                let uvs = meshes
                    .get(&mesh)
                    .and_then(|m| match m.attribute(Mesh::ATTRIBUTE_UV_0) {
                        Some(bevy::mesh::VertexAttributeValues::Float32x2(uvs)) => {
                            Some(uvs.clone())
                        }
                        _ => None,
                    });
                let mut entity = commands.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material),
                    ChildOf(root),
                ));
                if name.contains("tunnel") {
                    entity.insert(TunnelWall);
                } else if name.contains("bgtree") {
                    if let Some(uvs) = uvs {
                        // Its texture coordinates move every frame, so the main world keeps them.
                        if let Some(mut tree) = meshes.get_mut(&mesh) {
                            tree.asset_usage = bevy::asset::RenderAssetUsages::default();
                        }
                        entity.insert(TreeScroll { mesh, uvs });
                    }
                }
            }
        }
    }
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
        ("rom_train_in_model", None),
        (&mut meshes, &mut materials, &mut images),
    );
    if room.room.is_none() {
        warn!("train room missing from the disc");
        return;
    }
    spawn_ceiling_lights(
        &mut commands,
        &disc,
        (&mut meshes, &mut materials, &mut images),
    );
    spawn_scenery(
        &mut commands,
        &disc,
        (&mut meshes, &mut materials, &mut images),
    );
    spawn_door(
        &mut commands,
        &disc,
        (&mut meshes, &mut materials, &mut images),
    );
    // `start_demo1_actable`: Rover starts at unit (3, 3), the door end of the car.
    commands.spawn((
        Name::new("Rover"),
        Rover,
        crate::character::NpcGait::default(),
        crate::character::NpcClip::default(),
        crate::character::NpcModel(ROVER_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        // The models face +Z at yaw 0: Rover comes in walking south.
        Transform::from_translation(local(Vec2::new(140.0, 140.0)) + Vec3::Y),
    ));
    // `start_demo1_actable`: the old woman dozes on the other bench (`aNSO_set_animation`: kokkuri).
    let mut nod = crate::character::NpcClip::default();
    nod.start("cKF_ba_r_npc_1_kokkuri_d1", false, 0.0);
    commands.spawn((
        Name::new("sleeping old woman"),
        Dozing(3),
        crate::character::NpcGait::default(),
        nod,
        crate::character::NpcModel(OBABA_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(local(OBABA_AT) + Vec3::Y),
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

/// Turns Rover towards a point and moves him `speed` game units per frame along it.
fn walk_to(at: &mut Transform, to: Vec2, speed: f32, frames: f32) {
    let here = Vec2::new(
        (at.translation.x - TRAIN_ORIGIN.x) * UNIT,
        (at.translation.z - TRAIN_ORIGIN.z) * UNIT,
    );
    let dir = (to - here).normalize_or_zero();
    if dir != Vec2::ZERO {
        at.rotation = Quat::from_rotation_y(dir.x.atan2(dir.y));
        at.translation.x += dir.x * speed * frames / UNIT;
        at.translation.z += dir.y * speed * frames / UNIT;
    }
}

/// Where Rover stands, in game units on the car's floor.
fn rover_units(at: &Transform) -> Vec2 {
    Vec2::new(
        (at.translation.x - TRAIN_ORIGIN.x) * UNIT,
        (at.translation.z - TRAIN_ORIGIN.z) * UNIT,
    )
}

fn place(at: &mut Transform, p: Vec2) {
    at.translation.x = TRAIN_ORIGIN.x + p.x / UNIT;
    at.translation.z = TRAIN_ORIGIN.z + p.y / UNIT;
}

/// Starts message `msg` (and what follows it up to a question) and returns the question.
fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    script: &mut Script,
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
    script.current = msg;
    script.ask = false;
    // Which window a later order opens.
    match msg {
        0x2AC6 => script.entry = Entry::Time,
        0x2AC7 => script.entry = Entry::Player,
        COUNTRY_ASK => script.entry = Entry::Country,
        _ => {}
    }
    // `aNGD_check_talk_msg_no`: the answers that shape the player's face.
    script.flags |= match msg {
        0x2AC9 => 1 << 3,
        0x2ACD => 1 << 2,
        0x2ACF => 1 << 1,
        0x2AD3 => 1,
        _ => 0,
    };
    info!("train message {msg:#06x}");
    choice
}

/// `aNGD_enter` .. `aNGD_talk_start_wait` .. the questions: Rover walks to the player, turns,
/// and talks; each answer leads to the message the source's choice codes name, and an order
/// (index 9 = 1) at the end of a message opens the next window.
#[allow(clippy::too_many_arguments)]
fn rover(
    disc: Res<GameDisc>,
    state: Res<State<ShopLocation>>,
    time: Res<Time>,
    mut identity: ResMut<crate::save::PlayerIdentity>,
    mut focus: ResMut<InputFocus>,
    mut script: ResMut<Script>,
    mut dialogue: ResMut<DialogueState>,
    mut orders: ResMut<crate::msg_window::DemoOrders>,
    mut names: ResMut<crate::name_entry::NameEntry>,
    mut times: ResMut<crate::time_window::TimeWindow>,
    mut look: ResMut<crate::character::PlayerLook>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut rover: Query<
        (
            Entity,
            &mut Transform,
            &mut crate::character::NpcGait,
            &mut crate::character::NpcClip,
        ),
        With<Rover>,
    >,
    mut player: Query<(&Transform, &mut Facing), (With<Player>, Without<Rover>)>,
) {
    if *state != ShopLocation::Train {
        return;
    }
    let (Ok((entity, mut at, mut gait, mut clip)), Ok((player_at, mut facing))) =
        (rover.single_mut(), player.single_mut())
    else {
        return;
    };
    let frames = time.delta_secs() * 60.0;
    let script = &mut *script;
    // Sitting down: he slides to the bench and turns to face the car instead of jumping there.
    if let Some((from, t)) = script.slide {
        let t = (t + frames / SIT_FRAMES).min(1.0);
        let eased = t * t * (3.0 - 2.0 * t);
        place(&mut at, from.lerp(SEAT, eased));
        let yaw = (at.rotation * Vec3::Z).x.atan2((at.rotation * Vec3::Z).z);
        let turn = (0.0 - yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        at.rotation = Quat::from_rotation_y(yaw + turn * eased.max(0.05) * 0.25);
        script.slide = if t >= 1.0 {
            at.rotation = Quat::from_rotation_y(0.0);
            None
        } else {
            Some((from, t))
        };
    }
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
                let ctx = context(&identity);
                let choice = say(&disc, &mut dialogue, script, entity, &ctx, FIRST_MESSAGE);
                script.step = Step::Talk(choice);
            } else {
                at.rotation = Quat::from_rotation_y(yaw + diff.signum() * step);
            }
        }
        Step::Talk(choice) => {
            // The order value, not the raised queue: other talk scripts drain that one.
            let value = orders.get(ORDER_NPC0, ORDER_STEP);
            if value != 0 {
                orders.clear(ORDER_NPC0, ORDER_STEP);
                if script.current == LAST_MESSAGE {
                    // `aNGD_sitdown2_start_wait`: Rover sits down while the last talk goes on.
                    script.slide = Some((rover_units(&at), 0.0));
                    clip.start(clip::SIT, true, MORPH);
                    script.sat = true;
                } else if value == 1 && ASKS.contains(&script.current) {
                    script.ask = true;
                }
            }
            let Some(answer) = dialogue.external_result.take() else {
                return;
            };
            let asked = choice.as_ref().map(|(msg, _)| *msg);
            let next = choice
                .as_ref()
                .and_then(|(_, c)| c.next.get(answer).copied().flatten());
            // `aNGD_sex_select_wait2`: "cool" / "cute" and whether Rover's guess was right.
            if let (Some(msg @ (0x2AD6 | 0x2AD7)), true) = (asked, answer < 2) {
                script.male = Some(rustimal_logic::intro::is_male(msg == 0x2AD6, answer == 0));
                info!("gender: male = {:?}", script.male);
            }
            let ctx = context(&identity);
            if script.ask {
                *dialogue = DialogueState::default();
                match script.current {
                    // `aNGD_sex_select_after`: after the apology, on to the destination.
                    0x2ADF | 0x2AE0 => {
                        *focus = InputFocus::Dialogue;
                        let choice = say(&disc, &mut dialogue, script, entity, &ctx, COUNTRY_ASK);
                        script.step = Step::Talk(choice);
                    }
                    _ => {
                        let window = script.entry;
                        info!("the script opens a window");
                        match window {
                            Entry::Player => names.begin(crate::name_entry::Kind::Player),
                            Entry::Country => names.begin(crate::name_entry::Kind::Country),
                            Entry::Time => times.begin(crate::clock::now()),
                        }
                        script.step = Step::Window(window);
                    }
                }
                return;
            }
            match next {
                Some(msg @ (0x2AC8 | 0x2AC9)) if script.current == 0x2AC7 => {
                    // `aNGD_sitdown`: the next message waits for Rover to sit down.
                    *dialogue = DialogueState::default();
                    script.slide = Some((rover_units(&at), 0.0));
                    gait.0 = 0;
                    clip.start(clip::SIT, true, MORPH);
                    script.sat = true;
                    script.step = Step::Sit { next: msg };
                }
                Some(msg) => {
                    let choice = say(&disc, &mut dialogue, script, entity, &ctx, msg);
                    script.step = Step::Talk(choice);
                }
                None => {
                    info!("message chain {:#06x} ends", script.current);
                    *dialogue = DialogueState::default();
                    match script.current {
                        // The questions are over: Rover goes to ring Nook.
                        0x2ACE | 0x2ACF => {
                            clip.start(clip::STAND_UP, true, MORPH);
                            script.sat = false;
                            script.step = Step::Stand;
                        }
                        PHONE_MESSAGE => {
                            clip.start(clip::PHONE_OFF, true, MORPH);
                            script.step = Step::PhoneOff;
                        }
                        // The last talk is over: the train ride is done.
                        _ => {
                            // `aNGD_scene_change_wait_init`: the face and the first shirt.
                            let male = script.male.unwrap_or(true);
                            look.female = !male;
                            look.face = rustimal_logic::intro::face_type(
                                male,
                                script.flags,
                                rand.0.random(8) as usize,
                            );
                            look.cloth = rustimal_logic::intro::starting_cloth(
                                male,
                                rand.0.random(8) as usize,
                            );
                            info!(
                                "the player is a {}: face {}, shirt {}",
                                if male { "boy" } else { "girl" },
                                look.face,
                                look.cloth
                            );
                            *focus = InputFocus::World;
                            script.step = Step::Finished;
                            info!("intro: the train scene ends");
                        }
                    }
                }
            }
        }
        Step::Window(window) => {
            let window = *window;
            // `mTI_window_close`: the clock is set to what the player chose (`lbRTC_SetTime`).
            let name = if window == Entry::Time {
                let Some(set) = times.take() else { return };
                crate::clock::set_game_time(
                    set.year(),
                    set.month(),
                    set.day(),
                    set.hour(),
                    set.minute(),
                );
                info!(
                    "clock set to {}-{:02}-{:02} {:02}:{:02}",
                    set.year(),
                    set.month(),
                    set.day(),
                    set.hour(),
                    set.minute()
                );
                None
            } else {
                let Some(bytes) = names.take() else { return };
                Some(bytes)
            };
            if let Some(bytes) = name {
                let text = name_text(&bytes);
                match window {
                    Entry::Player => identity.player = text,
                    Entry::Country => identity.town = text,
                    Entry::Time => {}
                }
                info!("named: {} / {}", identity.player, identity.town);
            }
            *focus = InputFocus::Dialogue;
            let ctx = context(&identity);
            let msg = AFTER_WINDOW[window as usize];
            let choice = say(&disc, &mut dialogue, script, entity, &ctx, msg);
            script.step = Step::Talk(choice);
        }
        Step::Sit { next, .. } => {
            if clip.done && script.slide.is_none() {
                let next = *next;
                clip.start(clip::SIT_WAIT, false, MORPH);
                *focus = InputFocus::Dialogue;
                let ctx = context(&identity);
                let choice = say(&disc, &mut dialogue, script, entity, &ctx, next);
                script.step = Step::Talk(choice);
            }
        }
        Step::Stand => {
            if clip.done {
                place(&mut at, STAND);
                clip.stop(MORPH);
                script.step = Step::Aisle;
            }
        }
        Step::Aisle => {
            gait.0 = 1;
            walk_to(&mut at, AISLE, FAST_WALK, frames);
            if (at.translation.x - TRAIN_ORIGIN.x) * UNIT > AISLE.x {
                script.step = Step::Door;
            }
        }
        Step::Door => {
            walk_to(&mut at, DECK, FAST_WALK, frames);
            if (at.translation.z - TRAIN_ORIGIN.z) * UNIT < 140.0 {
                script.tilt_goal = std::f32::consts::FRAC_PI_2;
                script.tilt_add = 2.8125f32.to_radians();
            }
            if (at.translation.z - TRAIN_ORIGIN.z) * UNIT < DECK.y {
                gait.0 = 0;
                place(&mut at, DECK);
                clip.start(clip::TO_DECK, true, MORPH);
                script.step = Step::Deck;
            }
        }
        Step::Deck => {
            if clip.done {
                clip.start(clip::PHONE_ON, true, MORPH);
                *focus = InputFocus::Dialogue;
                let ctx = context(&identity);
                say(&disc, &mut dialogue, script, entity, &ctx, PHONE_MESSAGE);
                script.step = Step::PhoneOn;
            }
        }
        Step::PhoneOn => {
            if clip.done {
                // The message goes on while Rover talks; its end is handled as a chain end.
                clip.start(clip::PHONE_TALK, false, MORPH);
                script.step = Step::Talk(None);
            }
        }
        Step::PhoneOff => {
            if clip.done {
                clip.start(clip::OPEN_DOOR, true, MORPH);
                script.tilt_goal = 0.0;
                script.tilt_add = 8.4375f32.to_radians();
                script.step = Step::OpenDoor;
            }
        }
        Step::OpenDoor => {
            // `aNGD_open_door`: he turns his back on the door while it opens.
            at.rotation = Quat::from_rotation_y(std::f32::consts::PI);
            if clip.done {
                clip.stop(MORPH);
                script.step = Step::Return;
            }
        }
        Step::Return => {
            gait.0 = 1;
            walk_to(&mut at, Vec2::new(DECK.x, 300.0), FAST_WALK, frames);
            if (at.translation.z - TRAIN_ORIGIN.z) * UNIT > APPROACH_Z {
                gait.0 = 0;
                at.rotation = Quat::from_rotation_y(0.0);
                *focus = InputFocus::Dialogue;
                let ctx = context(&identity);
                let choice = say(&disc, &mut dialogue, script, entity, &ctx, LAST_MESSAGE);
                script.step = Step::Talk(choice);
            }
        }
        Step::Finished => {}
    }
}

/// The camera of the scene (`aNGD_set_camera`, `Camera2_request_main_lock`): it sits behind the player
/// at (100, 80, 400) with a 40 degree lens, looks down the car at (90, 80, 280) and, once Rover talks,
/// comes round to his head over 40 frames and stays there; the train's sway shakes it a little.
#[derive(Resource)]
struct TrainCamera {
    sway: rustimal_logic::intro::Sway,
    ticks: f32,
    offset: (f32, f32),
    /// `camera_morph_counter`: frames left before the camera is locked on Rover.
    morph: f32,
    locked: bool,
    /// `camera_tilt_cur_angle`.
    tilt: f32,
    /// The camera's height and where it looks, eased towards Rover's head.
    height: f32,
    center: Vec3,
    /// The lens has been put back for the town.
    restored: bool,
}

impl Default for TrainCamera {
    fn default() -> Self {
        Self {
            sway: Default::default(),
            ticks: 0.0,
            offset: (0.0, 0.0),
            morph: rustimal_logic::intro::CAMERA_MORPH,
            locked: false,
            tilt: 0.0,
            height: STANDING_HEAD,
            center: Vec3::new(90.0, STANDING_HEAD, 280.0),
            restored: false,
        }
    }
}

/// The lens the town uses (`player.rs`): 20 degrees, 200 to 1600 units.
fn town_lens() -> Projection {
    Projection::Perspective(PerspectiveProjection {
        fov: 20.0f32.to_radians(),
        near: 200.0 / UNIT,
        far: 1600.0 / UNIT,
        ..default()
    })
}

#[allow(clippy::too_many_arguments)]
fn train_camera(
    time: Res<Time>,
    state: Res<State<ShopLocation>>,
    script: Res<Script>,
    dialogue: Res<DialogueState>,
    mut cam: ResMut<TrainCamera>,
    rover: Query<&Transform, With<Rover>>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut camera: Query<
        (&mut Transform, &mut Projection, &mut Camera),
        (
            With<crate::player::WorldCamera>,
            Without<Rover>,
            Without<crate::msg_window::OverlayCamera>,
        ),
    >,
) {
    if *state != ShopLocation::Train {
        return;
    }
    let Ok((mut at, mut lens, mut view)) = camera.single_mut() else {
        return;
    };
    if matches!(script.step, Step::Finished) {
        // The scene is over: the town's camera takes over again.
        if !cam.restored {
            cam.restored = true;
            *lens = town_lens();
            view.viewport = None;
        }
        return;
    }
    // The retail picture is 4:3: on a wide window the car's open end shows as a void at the sides, so the
    // scene is drawn in a centred 4:3 viewport.
    if let Ok(window) = window.single() {
        let (w, h) = (window.physical_width(), window.physical_height());
        let wide = (h * 4 / 3).min(w);
        view.viewport = Some(bevy::camera::Viewport {
            physical_position: UVec2::new((w - wide) / 2, 0),
            physical_size: UVec2::new(wide, h),
            ..default()
        });
    }
    let frames = time.delta_secs() * 60.0;
    cam.ticks += frames;
    while cam.ticks >= 1.0 {
        cam.ticks -= 1.0;
        cam.offset = cam.sway.tick();
        if dialogue.active && !cam.locked {
            cam.morph -= 1.0;
            if cam.morph <= 0.0 {
                cam.locked = true;
            }
        }
    }
    let units = |p: Vec3| p * UNIT;
    let (sway_x, sway_y) = cam.offset;
    // Level with Rover's head: lower once he sits, eased so it never jumps.
    let goal_height = if script.sat {
        SEATED_HEAD
    } else {
        STANDING_HEAD
    };
    let ease = 1.0 - (1.0 - CAMERA_EASE).powf(frames);
    cam.height += (goal_height - cam.height) * ease;
    let height = cam.height;
    let rest = Vec3::new(90.0, height, 280.0);
    let goal = match rover.single() {
        Ok(rover) if cam.locked => {
            let p = units(rover.translation - TRAIN_ORIGIN);
            Vec3::new(p.x, height, p.z)
        }
        Ok(rover) if dialogue.active => {
            let p = units(rover.translation - TRAIN_ORIGIN);
            let r = (rustimal_logic::intro::CAMERA_MORPH - cam.morph)
                / rustimal_logic::intro::CAMERA_MORPH;
            let blend = rustimal_logic::intro::camera_blend(r);
            rest + (Vec3::new(p.x, height, p.z) - rest) * blend
        }
        _ => rest,
    };
    let moved = (goal - cam.center) * ease;
    cam.center += moved;
    let center = cam.center + Vec3::new(sway_x, sway_y, 0.0);
    let step = (script.tilt_goal - cam.tilt).clamp(-script.tilt_add, script.tilt_add) * frames;
    cam.tilt += step;
    let lean = cam.tilt.sin();
    // `eye.x = moveX + sin * 20 + 100`, `eye.y = moveY - sin * 5 + height`.
    let eye = Vec3::new(
        100.0 + sway_x + lean * 20.0,
        height + sway_y - lean * 5.0,
        400.0,
    );
    let to_world = |p: Vec3| TRAIN_ORIGIN + p / UNIT;
    at.translation = to_world(eye);
    at.look_at(to_world(center), Vec3::Y);
    *lens = Projection::Perspective(PerspectiveProjection {
        fov: 40.0f32.to_radians(),
        near: 60.0 / UNIT,
        far: 800.0 / UNIT,
        ..default()
    });
}

/// The player is not drawn on the train (the camera sits behind them).
fn hide_player(
    state: Res<State<ShopLocation>>,
    script: Res<Script>,
    mut player: Query<&mut Visibility, With<Player>>,
    mut hidden: Local<bool>,
) {
    let want = *state == ShopLocation::Train && !matches!(script.step, Step::Finished);
    if want == *hidden {
        return;
    }
    if let Ok(mut visibility) = player.single_mut() {
        *visibility = if want {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        *hidden = want;
    }
}

/// The train leaves the tunnel once Rover has sat down: the wall goes after a second and a half.
fn tunnel_exit(
    time: Res<Time>,
    script: Res<Script>,
    mut since: Local<Option<f32>>,
    mut walls: Query<&mut Visibility, With<TunnelWall>>,
) {
    if script.sat && since.is_none() {
        *since = Some(0.0);
    }
    let Some(t) = since.as_mut() else { return };
    *t += time.delta_secs();
    if *t >= 1.5 {
        for mut visibility in &mut walls {
            *visibility = Visibility::Hidden;
        }
    }
}

/// `Train_Window_Actor_move`: the trees' tile slides along the window.
fn scroll_trees(
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    trees: Query<&TreeScroll>,
    mut quarter_texels: Local<f32>,
) {
    *quarter_texels += 5.0 * time.delta_secs() * 60.0;
    let shift = *quarter_texels / 4.0 / 128.0;
    for tree in &trees {
        let Some(mut mesh) = meshes.get_mut(&tree.mesh) else {
            continue;
        };
        let moved: Vec<[f32; 2]> = tree.uvs.iter().map(|uv| [uv[0] - shift, uv[1]]).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, moved);
    }
}

fn doze(
    time: Res<Time>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut woman: Query<(&mut Dozing, &mut crate::character::NpcClip)>,
    mut twitching: Local<bool>,
) {
    let _ = time;
    for (mut dozing, mut clip) in &mut woman {
        if *twitching {
            if clip.done {
                *twitching = false;
                dozing.0 = 2 + rand.0.random(3) as u32;
                clip.start("cKF_ba_r_npc_1_kokkuri_d1", false, 8.0);
            }
        } else if clip.loops >= dozing.0 {
            *twitching = true;
            clip.start("cKF_ba_r_npc_1_kokkuri_d2", true, 8.0);
        }
    }
}

/// Raises the door's cue when Rover reaches the frame of his clip that opens it
/// (`aNGD_move_to_deck`: frame 9, `aNGD_open_door`: frame 22, at 0.5 frames per tick).
fn door_cues(script: Res<Script>, mut cue: ResMut<DoorCue>, mut last: Local<u8>) {
    let now = match script.step {
        Step::Deck => 1,
        Step::OpenDoor => 2,
        _ => 0,
    };
    if now != *last {
        *last = now;
        match now {
            1 => cue.0 = Some(9.0 / 0.5 / 60.0),
            2 => cue.0 = Some(22.0 / 0.5 / 60.0),
            _ => {}
        }
    }
}

/// `aTRD_actor_move`: the door plays through once at half speed when told to, with its two sounds.
fn door_animate(
    time: Res<Time>,
    mut cue: ResMut<DoorCue>,
    mut doors: Query<(&mut TrainDoor, &mut Posed)>,
    mut transforms: Query<&mut Transform>,
) {
    if let Some(left) = cue.0.as_mut() {
        *left -= time.delta_secs();
        if *left <= 0.0 {
            cue.0 = None;
            for (mut door, _) in &mut doors {
                door.speed = 0.5;
            }
        }
    }
    let ticks = time.delta_secs() * 60.0;
    for (mut door, mut posed) in &mut doors {
        let before = door.frame;
        if door.speed > 0.0 {
            door.frame += door.speed * ticks;
            // `NA_SE_TRAINDOOR0` / `1` at frames 2 and 27.
            for (frame, se) in [(2.0, 0x412), (27.0, 0x413)] {
                if before < frame && door.frame >= frame {
                    crate::sound::se(se);
                }
            }
            if door.frame >= 51.0 {
                door.frame = 1.0;
                door.speed = 0.0;
            }
        } else if posed.0 {
            continue;
        }
        posed.0 = true;
        door.prop
            .pose(door.anim.as_deref(), door.frame, &mut transforms);
    }
}

/// Whether a prop has been posed at least once.
#[derive(Component, Default)]
struct Posed(bool);

/// `ac_t_keitai.c`: the phone in Rover's right hand, `cKF_ba_r_tol_keitai_1_keitai_on1` / `_off1`
/// at half speed; it grows out of the hand over frames 10 to 20 and shrinks away over 33 to 41.
#[derive(Component)]
struct Phone {
    prop: crate::prop::Prop,
    on: Option<std::sync::Arc<crate::skeleton::Animation>>,
    off: Option<std::sync::Arc<crate::skeleton::Animation>>,
    putting_away: bool,
    frame: f32,
}

#[allow(clippy::too_many_arguments)]
fn phone_cues(
    mut commands: Commands,
    script: Res<Script>,
    disc: Option<Res<GameDisc>>,
    rover: Query<Entity, With<Rover>>,
    rigs: Query<(&ChildOf, &crate::character::CharacterRig)>,
    mut phones: Query<&mut Phone>,
    mut last: Local<u8>,
    (mut meshes, mut materials, mut images): (
        ResMut<Assets<Mesh>>,
        ResMut<Assets<N64Material>>,
        ResMut<Assets<Image>>,
    ),
) {
    let now = match script.step {
        Step::PhoneOn => 1,
        Step::PhoneOff => 2,
        _ => 0,
    };
    if now == *last {
        return;
    }
    *last = now;
    match now {
        1 => {
            let (Some(disc), Ok(rover)) = (disc, rover.single()) else {
                return;
            };
            let Some(hand) = rigs
                .iter()
                .find(|(parent, _)| parent.parent() == rover)
                .and_then(|(_, rig)| rig.right_hand())
            else {
                return;
            };
            let Some((root, prop)) = crate::prop::Prop::spawn(
                &mut commands,
                &disc,
                "cKF_bs_r_tol_keitai_1",
                hand,
                Transform::from_scale(Vec3::ZERO),
                (&mut meshes, &mut materials, &mut images),
            ) else {
                return;
            };
            let on = prop.animation(&disc, "cKF_ba_r_tol_keitai_1_keitai_on1");
            let off = prop.animation(&disc, "cKF_ba_r_tol_keitai_1_keitai_off1");
            commands.entity(root).insert((
                Phone {
                    prop,
                    on,
                    off,
                    putting_away: false,
                    frame: 1.0,
                },
                Posed::default(),
            ));
        }
        2 => {
            for mut phone in &mut phones {
                phone.putting_away = true;
                phone.frame = 1.0;
            }
        }
        _ => {}
    }
}

fn phone_animate(
    mut commands: Commands,
    time: Res<Time>,
    mut phones: Query<(Entity, &mut Phone)>,
    mut transforms: Query<&mut Transform>,
) {
    let ticks = time.delta_secs() * 60.0;
    for (entity, mut phone) in &mut phones {
        phone.frame += 0.5 * ticks;
        let (anim, last, from, to, grow) = if phone.putting_away {
            (phone.off.clone(), 61.0, 33.0, 41.0, false)
        } else {
            (phone.on.clone(), 68.0, 10.0, 20.0, true)
        };
        phone.frame = phone.frame.min(last);
        let t = ((phone.frame - from) / (to - from)).clamp(0.0, 1.0);
        let scale = if grow { t } else { 1.0 - t };
        if let Ok(mut transform) = transforms.get_mut(entity) {
            transform.scale = Vec3::splat(scale);
        }
        phone
            .prop
            .pose(anim.as_deref(), phone.frame, &mut transforms);
        if phone.putting_away && phone.frame >= last {
            commands.entity(entity).despawn();
        }
    }
}

pub struct IntroTrainPlugin;

impl Plugin for IntroTrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrainRoom>()
            .init_resource::<DoorCue>()
            .init_resource::<Script>()
            .init_resource::<TrainCamera>()
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
            )
            .add_systems(Update, hide_player.in_set(GameplaySet))
            .add_systems(
                Update,
                (tunnel_exit, scroll_trees, doze).in_set(GameplaySet),
            )
            .add_systems(
                Update,
                (door_cues, door_animate, phone_cues, phone_animate)
                    .chain()
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                train_camera
                    .after(crate::player::camera_follow)
                    .in_set(GameplaySet),
            );
    }
}
