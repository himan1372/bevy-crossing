//! The post office (SCENE_POST_OFFICE): its door, the disc room
//! (grd_post_office), the clerk at the counter and loan repayment.
//!
//! ac_post_office_move: A within 19 game units of (x - 40, z + 50), facing
//! between +x and -z, enters at (160, 300); leaving returns to (x - 64,
//! z + 64). Pelly works the counter by day and Phyllis from 19:00 to 07:00
//! (bg_post_item); she stands at unit (4, 2), 20 units left. Her talk follows
//! ac_npc_post_girl: a greeting that asks the business, repay (while a loan
//! is owed) through m_repay_ovl, its result line, then the question again.
//! Only the services the port has are offered (repay and leaving); letters
//! and the card machine are separate tasks.
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use rustimal_logic::post_office::{self as native, Business, RepayEvent, RepayInput, RepayMenu};

use crate::dialogue::DialogueState;
use crate::disc_gfx::{run_display_lists_with_colors, Segments};
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::locomotion::{self, Locomotion};
use crate::messages::MsgContext;
use crate::n64_material::N64Material;
use crate::player::Player;
use crate::shop::{ShopLocation, ShopReturnPos};
use crate::states::GameplaySet;
use crate::town::Town;

/// Where the interior is built, away from the town and the other rooms.
pub const POST_ORIGIN: Vec3 = Vec3::new(-3000.0, 0.0, -3000.0);
/// Game units per cell.
const UNIT: f32 = 40.0;
/// post_office door data: the player arrives at (160, 0, 300).
const ENTRY: Vec2 = Vec2::new(160.0, 300.0);
/// mvactor post_office: the clerk at unit (4, 2), moved 20 units left.
const CLERK: Vec2 = Vec2::new(4.0 * 40.0 + 20.0 - 20.0, 2.0 * 40.0 + 20.0);
/// `POST_OFFICE` layout item.
const POST_OFFICE: u16 = 0x5808;
/// npc_draw_data_tbl: ALL_NPC_NUM + (SP_NPC_POST_GIRL(2) - SP_NPC_START).
const PELLY: u32 = 238 + 3;
const PHYLLIS: u32 = 238 + 18;
/// l_sp_actor_name: both sisters use sound id 4.
const CLERK_SOUND: u8 = 4;

/// Phyllis is on duty now.
pub fn night_clerk() -> bool {
    native::night_clerk(crate::clock::now().day_secs)
}

#[derive(Resource)]
struct PostBuilding {
    /// aPOFF door check point (world, ground height).
    trigger: Vec3,
    /// Where leaving puts the player.
    exit: Vec3,
}

#[derive(Resource, Default)]
struct PostRoom {
    built: bool,
    room: Option<crate::indoor_room::IndoorRoom>,
}

/// The repay window (mSM_OVL_REPAY) is open: it owns the controls.
#[derive(Resource, Default)]
pub(crate) struct RepayOpen(pub bool);

#[derive(Component)]
struct Clerk {
    night: bool,
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// The question is up; `offered[i]` is the original answer index.
    Menu {
        status: u8,
        offered: Vec<usize>,
        next: Vec<Option<usize>>,
    },
    /// The line before the repay window.
    RepayPrompt,
    Repay {
        menu: RepayMenu,
        ui: Option<Entity>,
    },
    /// The result line; then the question again.
    Result,
    /// The pockets are open to pick a letter to post.
    Mailing,
    /// A question about letters is up (0x8E3 / 0x8BD), by its message.
    Asked(usize),
    /// A line about the letter; then aPG_office_space_check.
    ThenSpaceCheck,
    Finish,
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
    context: MsgContext,
}

fn local(p: Vec2) -> Vec3 {
    POST_ORIGIN + Vec3::new(p.x / UNIT, 0.0, p.y / UNIT)
}

fn locate(town: Res<Town>, mut commands: Commands) {
    let Some(b) = town
        .retail
        .as_ref()
        .and_then(|r| r.buildings.iter().find(|b| b.item == POST_OFFICE))
    else {
        return;
    };
    let at = |dx: f32, dz: f32| {
        let p = b.origin + Vec3::new(dx, 0.0, dz) / UNIT;
        p.with_y(town.ground_y(p))
    };
    commands.insert_resource(PostBuilding {
        trigger: at(-40.0, 50.0),
        exit: at(-64.0, 64.0),
    });
    info!("post office door at {:?}", at(-40.0, 50.0));
}

/// Build the disc room once the disc is loaded.
fn build_room(
    mut commands: Commands,
    disc: Res<GameDisc>,
    mut room: ResMut<PostRoom>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    if room.built {
        return;
    }
    room.built = true;
    room.room = crate::indoor_room::IndoorRoom::build(
        &mut commands,
        &disc,
        "post office room",
        POST_ORIGIN,
        ("grd_post_office_model", Some("grd_post_office_modelT")),
        (&mut meshes, &mut materials, &mut images),
    );
    if room.room.is_none() {
        warn!("post office room missing from the disc");
    }
}

fn spawn_clerk(commands: &mut Commands, meshes: &mut Assets<Mesh>, night: bool) {
    let at = local(CLERK) + Vec3::Y;
    commands.spawn((
        Name::new(if night { "Phyllis" } else { "Pelly" }),
        Clerk { night },
        crate::character::NpcModel(if night { PHYLLIS } else { PELLY }),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at).looking_to(Vec3::NEG_Z, Vec3::Y),
    ));
}

#[allow(clippy::too_many_arguments)]
fn enter(
    mut commands: Commands,
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    tool: Res<crate::ecology::EquippedTool>,
    building: Option<Res<PostBuilding>>,
    mut next: ResMut<NextState<ShopLocation>>,
    mut player: Query<(&mut Transform, &mut Facing), With<Player>>,
    clerks: Query<(Entity, &Clerk)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if *state != ShopLocation::Town
        || *house.get() != crate::house::HouseState::Town
        || !focus.is_world()
        || *tool != crate::ecology::EquippedTool::None
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let (Some(building), Ok((mut transform, mut facing))) = (building, player.single_mut()) else {
        return;
    };
    // aPOFF: dist^2 < 360 game units of the door point, facing between +x
    // and -z (yaw 0x4000..0x8000).
    let d = (transform.translation - building.trigger).with_y(0.0) * UNIT;
    let yaw = facing.dir.x.atan2(facing.dir.z).to_degrees();
    if d.length_squared() >= 360.0 || !(90.0..180.0).contains(&yaw) {
        return;
    }
    if !actions.consume(Action::Interact) {
        return;
    }
    commands.insert_resource(ShopReturnPos(building.exit.with_y(transform.translation.y)));
    transform.translation = local(ENTRY) + Vec3::Y;
    facing.dir = Vec3::NEG_Z;
    // The clerk of the hour (bg_post_item sets the type on entry).
    let night = night_clerk();
    for (e, clerk) in &clerks {
        if clerk.night != night {
            commands.entity(e).despawn();
        }
    }
    if !clerks.iter().any(|(_, c)| c.night == night) {
        spawn_clerk(&mut commands, &mut meshes, night);
    }
    crate::sound::door_sound(None, true);
    next.set(ShopLocation::PostOffice);
    info!(
        "entered the post office ({})",
        if night { "Phyllis" } else { "Pelly" }
    );
}

/// E at the door (the south wall by the entry): back to town.
fn leave(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    mut next: ResMut<NextState<ShopLocation>>,
    return_pos: Option<Res<ShopReturnPos>>,
    mut player: Query<(&mut Transform, &mut Facing), With<Player>>,
    mut post: ResMut<crate::post::Post>,
) {
    if *state != ShopLocation::PostOffice
        || !focus.is_world()
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let Ok((mut transform, mut facing)) = player.single_mut() else {
        return;
    };
    let door = local(ENTRY);
    let near = (transform.translation - door).with_y(0.0).length() < 1.0;
    if !near || facing.dir.z < 0.5 || !actions.consume(Action::Interact) {
        return;
    }
    if let Some(r) = return_pos {
        transform.translation = r.0;
    }
    facing.dir = Vec3::new(-1.0, 0.0, 1.0).normalize();
    // aPG_actor_dt: a full desk sends Pete out straight away.
    if post.desk_full() {
        post.force = true;
    }
    crate::sound::door_sound(None, false);
    next.set(ShopLocation::Town);
    info!("left the post office");
}

fn movement(
    actions: Res<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<ShopLocation>>,
    time: Res<Time>,
    room: Res<PostRoom>,
    mut player: Query<(&mut Transform, &mut Facing, &mut Locomotion), With<Player>>,
) {
    if *state != ShopLocation::PostOffice || !focus.is_world() {
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

fn clerk_info(night: bool) -> (u8, &'static str, u32) {
    if night {
        (1, "Phyllis", PHYLLIS)
    } else {
        (0, "Pelly", PELLY)
    }
}

/// Show `msg`; returns its choice branches.
fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    clerk: Entity,
    night: bool,
    ctx: &MsgContext,
    msg: usize,
) -> Option<crate::messages::MsgChoice> {
    let text = disc.text.as_ref()?;
    let (_, name, npc) = clerk_info(night);
    let (pages, choice) = text.talk(msg, ctx, &mut |_| 0);
    dialogue.begin_special(
        clerk,
        name,
        (npc, CLERK_SOUND),
        pages,
        choice.as_ref().map(|c| c.labels.clone()),
    );
    info!("post office message {msg:#06x}");
    choice
}

fn context(identity: &crate::save::PlayerIdentity, night: bool, loan: u32) -> MsgContext {
    let now = crate::clock::now();
    let (millions, rest) = native::loan_strings(loan);
    let mut free = vec![String::new(); 20];
    free[1] = millions;
    free[2] = rest;
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: clerk_info(night).1.into(),
        free,
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

/// aPG_set_post_status for the port: the first job is over, no savings
/// account yet; the desk is full with five letters waiting.
fn status(loan: u32, desk_full: bool) -> u8 {
    native::status(desk_full, loan, true, false)
}

/// Ask the business question: only working answers are shown.
fn ask(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    clerk: Entity,
    night: bool,
    talk: &mut Talk,
    msg: usize,
    (loan, desk_full): (u32, bool),
) {
    let status = status(loan, desk_full);
    let choice = say(disc, dialogue, clerk, night, &talk.context, msg);
    let Some(choice) = choice else {
        talk.step = Step::Finish;
        return;
    };
    let offered: Vec<usize> = (0..choice.labels.len())
        .filter(|&i| {
            matches!(
                native::business(status, i),
                Business::SendLetter | Business::DeskFull | Business::Repay | Business::Quit
            )
        })
        .collect();
    dialogue.set_actor_choices(offered.iter().map(|&i| choice.labels[i].clone()).collect());
    talk.step = Step::Menu {
        status,
        offered,
        next: choice.next,
    };
}

#[allow(clippy::too_many_arguments)]
fn talk(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), With<Player>>,
    clerks: Query<(Entity, &Transform, &Clerk)>,
    mut flow: ResMut<Talk>,
    mut repay_open: ResMut<RepayOpen>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    identity: Res<crate::save::PlayerIdentity>,
    font: Option<Res<crate::msg_window::MsgAssets>>,
    (mut pockets, mut mailbox, mut post, mut collection, mut villagers, mut memory): (
        ResMut<crate::pockets::PocketsUi>,
        ResMut<crate::mail::Mailbox>,
        ResMut<crate::post::Post>,
        ResMut<crate::museum::MuseumCollection>,
        Query<(&crate::npc::Villager, &mut crate::dialogue::VillagerTalk)>,
        ResMut<crate::villager_memory::VillagerMemory>,
    ),
    (mut meshes, mut materials, mut n64, mut images): (
        ResMut<Assets<Mesh>>,
        ResMut<Assets<StandardMaterial>>,
        ResMut<Assets<N64Material>>,
        ResMut<Assets<Image>>,
    ),
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::PostOffice {
        return;
    }
    let Some((clerk, clerk_at, info)) = clerks.iter().next() else {
        return;
    };
    let night = info.night;
    let (index, _, _) = clerk_info(night);
    let clerk_msg = |n: usize| n + index as usize;

    let repaying = matches!(flow.step, Step::Repay { .. });
    if repay_open.0 != repaying {
        repay_open.0 = repaying;
    }
    // The repay window owns input while it is open.
    if let Step::Repay { menu, ui } = &mut flow.step {
        let press = if actions.consume(Action::Cancel) || actions.consume(Action::Back) {
            Some(RepayInput::B)
        } else if actions.consume(Action::Confirm) {
            Some(RepayInput::Start)
        } else if actions.consume(Action::Interact) || actions.consume(Action::Advance) {
            Some(RepayInput::A)
        } else if actions.consume(Action::MoveLeft) {
            Some(RepayInput::Left)
        } else if actions.consume(Action::MoveRight) {
            Some(RepayInput::Right)
        } else if actions.consume(Action::MoveUp) || actions.consume(Action::MenuUp) {
            Some(RepayInput::Up)
        } else if actions.consume(Action::MoveDown) || actions.consume(Action::MenuDown) {
            Some(RepayInput::Down)
        } else {
            None
        };
        let mut redraw = ui.is_none();
        if let Some(press) = press {
            let event = menu.press(press);
            match event {
                RepayEvent::None => {}
                RepayEvent::Cursor => {
                    crate::sound::se(crate::sound::se_id::CURSOR);
                    redraw = true;
                }
                RepayEvent::Changed => {
                    crate::sound::se(0x426);
                    redraw = true;
                }
                RepayEvent::Limit => crate::sound::se(0x1003),
                RepayEvent::Cancel | RepayEvent::Confirm => {
                    crate::sound::se(crate::sound::se_id::MENU_EXIT);
                    let before = menu.loan_at_open;
                    if event == RepayEvent::Confirm {
                        let wallet = inventory.bells;
                        if let Some((loan, wallet)) = menu.commit(wallet, &mut inventory.pockets) {
                            inventory.loan = loan;
                            inventory.bells = wallet;
                            info!("repaid: loan {before} -> {loan}, wallet {wallet}");
                        }
                    }
                    if let Some(e) = ui.take() {
                        commands.entity(e).despawn();
                    }
                    let after = status(inventory.loan, post.desk_full());
                    flow.context = context(&identity, night, inventory.loan);
                    let msg = clerk_msg(native::repay_result(before, inventory.loan, after));
                    *focus = InputFocus::Dialogue;
                    say(&disc, &mut dialogue, clerk, night, &flow.context, msg);
                    flow.step = Step::Result;
                    return;
                }
            }
        }
        if redraw {
            if let Some(font) = font.as_deref() {
                if let Some(e) = ui.take() {
                    commands.entity(e).despawn();
                }
                *ui = Some(draw_repay(
                    &mut commands,
                    &disc,
                    font,
                    menu,
                    (&mut meshes, &mut materials, &mut n64, &mut images),
                ));
            }
        }
        return;
    }

    let open_mailing =
        |pockets: &mut crate::pockets::PocketsUi, focus: &mut InputFocus, d: &mut DialogueState| {
            // aPG_receive_menu_close_wait_init: mSM_IV_OPEN_SEND_MAIL.
            *pockets = crate::pockets::PocketsUi {
                open: true,
                mailing: true,
                mail_slot: Some(0),
                ..default()
            };
            *d = DialogueState::default();
            *focus = InputFocus::Pockets;
            crate::sound::se(crate::sound::se_id::MENU_PAUSE);
        };

    // aPG_receive_menu_close_wait: the letter picked (or not).
    if matches!(flow.step, Step::Mailing) {
        if pockets.open {
            return;
        }
        pockets.mailing = false;
        *focus = InputFocus::Dialogue;
        let Some(index) = pockets
            .mail_pick
            .take()
            .filter(|&i| i < mailbox.letters.len())
        else {
            let asked = say(
                &disc,
                &mut dialogue,
                clerk,
                night,
                &flow.context,
                clerk_msg(0x8E3),
            );
            flow.step = if asked.is_some() {
                Step::Asked(0x8E3)
            } else {
                Step::Finish
            };
            return;
        };
        // aPG_check_destination: a villager who has moved away can't get it.
        let to = mailbox.letters[index].to.clone();
        let looks = match &to {
            Some(crate::mail::Recipient::Villager(name)) => villagers
                .iter()
                .find(|(v, _)| v.name == name)
                .map(|(v, _)| Some(v.looks)),
            Some(crate::mail::Recipient::Museum) => Some(None),
            None => None,
        };
        let msg = if let Some(looks) = looks {
            let letter = mailbox.letters.remove(index);
            let name = letter.to.as_ref().map(|t| t.name().to_string());
            if let Some(crate::mail::Recipient::Villager(n)) = &letter.to {
                memory.meet(n);
            }
            if let Some(delta) = post.receive(letter, looks, &mut collection.0) {
                for (v, mut talk) in &mut villagers {
                    if Some(v.name) == name.as_deref() {
                        talk.friendship += delta;
                    }
                }
            }
            0x8B9
        } else {
            0x8E1
        };
        let asked = say(
            &disc,
            &mut dialogue,
            clerk,
            night,
            &flow.context,
            clerk_msg(msg),
        );
        flow.step = asked.map_or(Step::ThenSpaceCheck, |_| Step::Asked(clerk_msg(msg)));
        return;
    }

    // aPG_office_space_check, once the line about the letter is read.
    if matches!(flow.step, Step::ThenSpaceCheck) {
        if dialogue.active {
            return;
        }
        *focus = InputFocus::Dialogue;
        if post.desk_full() {
            say(
                &disc,
                &mut dialogue,
                clerk,
                night,
                &flow.context,
                clerk_msg(0x8BB),
            );
            flow.step = Step::Finish;
        } else {
            let asked = say(
                &disc,
                &mut dialogue,
                clerk,
                night,
                &flow.context,
                clerk_msg(0x8BD),
            );
            flow.step = asked.map_or(Step::Finish, |_| Step::Asked(0x8BD));
        }
        return;
    }

    if let Some(choice) = dialogue.external_result.take() {
        *focus = InputFocus::Dialogue;
        match std::mem::take(&mut flow.step) {
            // aPG_check_receive_after / aPG_loop_check: another letter?
            Step::Asked(0x8E3 | 0x8BD) if choice == 0 => {
                open_mailing(&mut pockets, &mut focus, &mut dialogue);
                flow.step = Step::Mailing;
            }
            Step::Asked(0x8BD) => {
                say(
                    &disc,
                    &mut dialogue,
                    clerk,
                    night,
                    &flow.context,
                    clerk_msg(0x8B3),
                );
                flow.step = Step::Finish;
            }
            Step::Menu {
                status,
                offered,
                next,
            } => {
                let original = offered.get(choice).copied().unwrap_or(usize::MAX);
                match native::business(status, original) {
                    // aPG_ACTION_21: the pockets, to pick a letter.
                    Business::SendLetter => {
                        open_mailing(&mut pockets, &mut focus, &mut dialogue);
                        flow.step = Step::Mailing;
                    }
                    Business::Repay => {
                        let msg = next
                            .get(original)
                            .copied()
                            .flatten()
                            .unwrap_or(native::repay_prompt(index));
                        say(&disc, &mut dialogue, clerk, night, &flow.context, msg);
                        flow.step = Step::RepayPrompt;
                    }
                    _ => {
                        if let Some(msg) = next.get(original).copied().flatten() {
                            say(&disc, &mut dialogue, clerk, night, &flow.context, msg);
                            flow.step = Step::Finish;
                        } else {
                            *dialogue = DialogueState::default();
                            *focus = InputFocus::World;
                        }
                    }
                }
            }
            Step::RepayPrompt => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::Shop;
                flow.step = Step::Repay {
                    menu: RepayMenu::open(inventory.bells, inventory.loan, &inventory.pockets),
                    ui: None,
                };
            }
            Step::Result => {
                // aPG_repay_after: the question again.
                flow.context = context(&identity, night, inventory.loan);
                let msg = clerk_msg(native::after_repay(status(
                    inventory.loan,
                    post.desk_full(),
                )));
                ask(
                    &disc,
                    &mut dialogue,
                    clerk,
                    night,
                    &mut flow,
                    msg,
                    (inventory.loan, post.desk_full()),
                );
            }
            _ => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
            }
        }
        return;
    }

    if !matches!(flow.step, Step::Idle | Step::Finish)
        || !focus.is_world()
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    // talk_distance 85 game units, across the counter.
    let delta = (clerk_at.translation - at.translation).with_y(0.0);
    if delta.length() * UNIT > 85.0 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    flow.context = context(&identity, night, inventory.loan);
    let msg = native::greeting(status(inventory.loan, post.desk_full()), index);
    ask(
        &disc,
        &mut dialogue,
        clerk,
        night,
        &mut flow,
        msg,
        (inventory.loan, post.desk_full()),
    );
}

/// mRP_set_frame_dl + mRP_set_character_dl at the window's rest position.
fn draw_repay(
    commands: &mut Commands,
    disc: &GameDisc,
    font: &crate::msg_window::MsgAssets,
    menu: &RepayMenu,
    (meshes, materials, n64, images): (
        &mut Assets<Mesh>,
        &mut Assets<StandardMaterial>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> Entity {
    let root = commands
        .spawn((
            Name::new("repay window"),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    // fkm_win_mode and fkm_win_model follow fkm_win_sw10T_model.
    if let Some(sw10) = disc.catalog.get("fkm_win_sw10T_model") {
        let mode = sw10.offset + 0x30;
        let model = mode + 0x40;
        let opcode = |o: u32| {
            disc.rom
                .slice(disc::RomSource::Rel, o, 1)
                .and_then(|b| b.first().copied())
        };
        if opcode(mode) == Some(0xD9) && opcode(model) == Some(0xDE) {
            let sink = run_display_lists_with_colors(
                &disc.rom,
                &[mode, model],
                [255; 4],
                255,
                [255; 4],
                &[],
            );
            let frame = commands
                .spawn((
                    Transform::from_xyz(0.0, 0.0, 139.0).with_scale(Vec3::new(16.0, 16.0, 1.0)),
                    Visibility::Inherited,
                    ChildOf(root),
                ))
                .id();
            for (i, (mesh, material)) in crate::disc_town::sink_meshes_scaled(
                disc,
                &sink,
                None,
                true,
                &Segments::default(),
                1.0,
                (meshes, n64, images),
            )
            .into_iter()
            .enumerate()
            {
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    Transform::from_xyz(0.0, 0.0, i as f32 * 0.01),
                    RenderLayers::layer(1),
                    ChildOf(frame),
                ));
            }
        }
    }
    let mut paint = crate::pockets_disc::Paint {
        commands,
        meshes,
        materials,
        root,
    };
    let scaled = |text: &str, scale: u8| {
        use crate::messages::markup;
        format!("{}{}{text}", markup::LINE_SCALE, markup::arg(scale))
    };
    let width = |text: &str| crate::msg_window::text_width(text, &font.widths);
    let right =
        |paint: &mut crate::pockets_disc::Paint, text: &str, x: f32, y: f32, color: [u8; 4]| {
            let t = scaled(text, 28);
            let w = width(&t);
            paint.text(font, &t, Vec2::new(x - w, y), color, 141.0);
        };
    paint.text(
        font,
        &scaled("Your Loan", 28),
        Vec2::new(153.0, 64.0),
        [255; 4],
        141.0,
    );
    right(
        &mut paint,
        &group(menu.money),
        206.0,
        91.0,
        [100, 40, 255, 255],
    );
    // The amount: six zero-filled digits with a comma, the cursor's in red.
    let amount = format!("{:03},{:03}", menu.amount / 1000, menu.amount % 1000);
    let selected = if menu.cursor >= 3 {
        menu.cursor + 1
    } else {
        menu.cursor
    };
    let mut x = 206.0 - width(&amount);
    for (i, ch) in amount.chars().enumerate() {
        let color = if i == selected && menu.cursor < native::CURSOR_OK {
            [215, 0, 0, 255]
        } else {
            [70, 120, 245, 255]
        };
        let s = ch.to_string();
        paint.text(font, &s, Vec2::new(x, 112.0), color, 141.0);
        x += width(&s);
    }
    let ok = if menu.cursor >= native::CURSOR_OK {
        [215, 0, 0, 255]
    } else {
        [70, 120, 245, 255]
    };
    paint.text(font, &scaled("OK", 28), Vec2::new(207.0, 128.0), ok, 141.0);
    right(
        &mut paint,
        &group(menu.loan),
        209.0,
        155.0,
        [40, 185, 110, 255],
    );
    root
}

fn group(v: u32) -> String {
    if v >= 1000 {
        format!("{},{:03}", v / 1000, v % 1000)
    } else {
        v.to_string()
    }
}

pub struct PostOfficePlugin;

impl Plugin for PostOfficePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PostRoom>()
            .init_resource::<Talk>()
            .init_resource::<RepayOpen>()
            .add_systems(Startup, locate.after(crate::town::generate_town))
            .add_systems(Update, build_room.run_if(resource_exists::<GameDisc>))
            .add_systems(
                Update,
                (enter, leave)
                    .in_set(InteractSet::Doors)
                    .in_set(GameplaySet),
            )
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet))
            .add_systems(
                Update,
                movement.in_set(locomotion::MovementSet).in_set(GameplaySet),
            );
    }
}
