//! The town map (`mSM_CHECK_OPEN_MAP`: the X button while `Common(map_flag)` is set).
//!
//! With the disc this is the retail overlay (`m_map_ovl.c`): the window, the acre pictures by
//! block type, the cursor with its glide and pulse, the label panel of the selected acre, the
//! villagers' house marks and the town name, all drawn from the disc's display lists. Without
//! a disc (or a retail layout) a plain text grid stands in. The flag and when it is given are
//! the source's: off from the new-game intro, handed over by the guide after the third job,
//! and by Officer Copper when it is missing.

use std::collections::HashMap;
use std::sync::Arc;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use disc::{RomSource, Target};
use rustimal_logic::random_field::{BLOCK_X_NUM, FIELD_STEP1};
use rustimal_logic::town_gen::{Feature, ACRE_DEPTH, ACRE_WIDTH};
use rustimal_logic::town_map::{
    self as map, Cursor, HousePos, Label, ACRES, ACRES_X, BLOCK_SIZE, CURSOR_G, CURSOR_SCALE,
    HOUSE_OFFSET_X, HOUSE_OFFSET_Y,
};

use crate::disc_gfx::{run_display_lists_with_segment_lists, Segments};
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus};
use crate::msg_window::{screen, text_mesh, text_width, MsgAssets};
use crate::n64_material::N64Material;
use crate::player::Player;
use crate::town::{Town, HOUSE_SIZE};
use rustimal_logic::menu_slide::Slide;

type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;

/// Overlay units per screen pixel (`msg_window`'s `SCALE`), and the models' own scale.
const PX: f32 = 16.0;
/// Where the acre pictures start (`mMP_set_map_dl`), pixels from the screen centre.
const MAP_ORIGIN: Vec2 = Vec2::new(11.7, 45.7);
/// `mMP_set_label_dl`: building names and the residents' name rows.
const BUILDING_COLOR: [u8; 4] = [120, 95, 205, 255];
const RESIDENT_COLOR: [u8; 4] = [255, 75, 40, 255];
const FREE_COLOR: [u8; 4] = [165, 145, 140, 255];
/// Retail repeat of a held direction (`mSM_make_trigger_data`).
const REPEAT_FIRST: f32 = 25.0;
const REPEAT_NEXT: f32 = 3.0;
/// The window's colour list for a map opened with X (`data0 == 1`) and from a map board
/// (`data0 == 0`, `mMP_set_base_dl`).
const COLOR_LIST: [&str; 2] = ["kan_win_color0_mode", "kan_win_color1_mode"];
/// The town name's colour by `data0` (`mMP_set_dl`'s `land_color`).
const LAND_COLORS: [[u8; 4]; 2] = [[255, 0, 255, 255], [60, 60, 255, 255]];
/// Map signboards (`MAP_BOARD0` / `MAP_BOARD1`).
const MAP_BOARDS: [u16; 2] = [0x000C, 0x000D];

#[derive(Resource, Default)]
pub(crate) struct MapUi {
    pub open: bool,
    view: Option<View>,
}

/// A resident listed in an acre's label.
struct Resident {
    name: String,
    /// False for an empty player slot ("free").
    known: bool,
    layer: usize,
    idx: usize,
}

struct AcreLabel {
    label: Label,
    residents: Vec<Resident>,
}

/// What the open map holds.
struct View {
    /// Opened at a map signboard (`data0 == 0`) rather than with X: another colour, and X does
    /// not close it.
    board: bool,
    cursor: Cursor,
    slide: Slide,
    /// The block type behind each acre picture (`mMP_set_map_texture_pal`).
    pictures: [u8; ACRES],
    labels: Vec<Option<AcreLabel>>,
    /// Glide ticks left to run before the cursor takes input again (`mMP_move_Wait`).
    gliding: bool,
    glide_ticks: f32,
    repeat: Option<(Action, f32)>,
}

#[derive(Component)]
struct MapPanel;
#[derive(Component)]
struct MapText;

fn setup(mut commands: Commands) {
    commands
        .spawn((
            MapPanel,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(30.0),
                right: Val::Percent(30.0),
                top: Val::Percent(18.0),
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.09, 0.04, 0.95)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                MapText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.95, 0.75)),
            ));
        });
}

fn symbol(feature: u8) -> char {
    match feature {
        f if f == Feature::Station as u8 => 'T',
        f if f == Feature::Shop as u8 => 'S',
        f if f == Feature::PostOffice as u8 => 'P',
        f if f == Feature::PlayerHouse as u8 => 'H',
        f if f == Feature::WishingWell as u8 => 'W',
        f if f == Feature::PoliceStation as u8 => 'C',
        f if f == Feature::Museum as u8 => 'M',
        f if f == Feature::Tailor as u8 => 'A',
        f if f == Feature::Dock as u8 => 'D',
        _ => '.',
    }
}

/// The plain map as text: one row per acre row, two characters per acre (the building, then
/// `*` where the player stands).
fn render(town: &Town, here: Option<(usize, usize)>) -> String {
    let mut out = String::from("Town map\n\n    ");
    for x in 0..ACRE_WIDTH {
        out.push_str(&format!("{}  ", x + 1));
    }
    out.push('\n');
    for z in 0..ACRE_DEPTH {
        out.push_str(&format!("{}   ", ['A', 'B', 'C', 'D', 'E', 'F'][z.min(5)]));
        for x in 0..ACRE_WIDTH {
            let feature = town.plan.acres[z * ACRE_WIDTH + x].feature;
            out.push(symbol(feature));
            out.push(if here == Some((x, z)) { '*' } else { ' ' });
            out.push(' ');
        }
        out.push('\n');
    }
    out.push_str(
        "\nT station  S shop  P post office  C police  M museum\nA tailor  W well  H your house  D dock  * you\n",
    );
    out
}

/// A REL pointer field.
fn pointer(disc: &GameDisc, at: u32) -> Option<u32> {
    match disc.rom.pointer_at(RomSource::Rel, at)? {
        Target::File(RomSource::Rel, p) => Some(p),
        _ => None,
    }
}

/// What the overlay reads from the disc once.
struct Tables {
    pals: Vec<u8>,
    plus_bridge: Vec<u8>,
    houses: Vec<HousePos>,
}

fn read_tables(disc: &GameDisc) -> Option<Tables> {
    let bytes = |name: &str| {
        let a = disc.catalog.get(name)?;
        disc.rom
            .slice(RomSource::Rel, a.offset, a.size)
            .map(<[u8]>::to_vec)
    };
    Some(Tables {
        pals: bytes("l_map_pal")?,
        plus_bridge: bytes("pluss_bridge")?,
        houses: map::parse_house_pos(&bytes("mMP_house_pos_list")?),
    })
}

/// The retail layout's acre combination (`fg_id`) at an acre.
fn fg_id(town: &Town, ax: usize, az: usize) -> Option<u16> {
    let retail = town.retail.as_ref()?;
    let combi = retail.field.combi[(az + 1) * BLOCK_X_NUM + ax + 1].combi;
    Some(retail.combos.get(usize::from(combi))?.fg_id)
}

/// `mMP_set_house_data` / `mMP_set_field_data`: who is listed in which acre.
fn labels_for(
    town: &Town,
    tables: &Tables,
    identity: &crate::save::PlayerIdentity,
    villagers: &Query<&crate::npc::Villager>,
) -> Vec<Option<AcreLabel>> {
    let mut labels: Vec<Option<AcreLabel>> = (0..ACRES).map(|_| None).collect();
    let Some(retail) = town.retail.as_ref() else {
        return labels;
    };
    // The players' houses stand in one fixed acre (`label_info[1][2]`).
    let mut players = vec![Resident {
        name: identity.player.clone(),
        known: true,
        layer: 0,
        idx: 0,
    }];
    while players.len() < 4 {
        players.push(Resident {
            name: "free".into(),
            known: false,
            layer: 0,
            idx: 0,
        });
    }
    labels[ACRES_X + 2] = Some(AcreLabel {
        label: Label::Player,
        residents: players,
    });
    for villager in villagers.iter() {
        let door = villager.door();
        let Some((ax, az, _, _)) = Town::cell_indices_of(door) else {
            continue;
        };
        // The house unit is the door's, a house and a half back.
        let centre = Vec3::new(door.x, door.y, door.z - HOUSE_SIZE / 2.0 - 0.5);
        let Some((_, _, cx, cz)) = Town::cell_indices_of(centre) else {
            continue;
        };
        let at = (az + 1) * BLOCK_X_NUM + ax + 1;
        let idx = fg_id(town, ax, az).map_or(0, |fg| {
            map::house_index(&tables.houses, fg, cx as u8, cz as u8 + HOUSE_UT_Z_BIAS)
        });
        let tier = retail.field.combi[at].height.saturating_sub(FIELD_STEP1);
        let slot = az * ACRES_X + ax;
        let entry = labels[slot].get_or_insert(AcreLabel {
            label: Label::Npc,
            residents: Vec::new(),
        });
        if entry.label == Label::Npc && entry.residents.len() < 4 {
            entry.residents.push(Resident {
                name: villager.name.to_string(),
                known: true,
                layer: map::house_layer(map::tier_height(tier), retail.field.three_step),
                idx: usize::from(idx),
            });
        }
    }
    for entry in labels.iter_mut().flatten() {
        if entry.label == Label::Npc {
            entry.residents.sort_by_key(|r| r.layer);
        }
    }
    for (slot, entry) in labels.iter_mut().enumerate() {
        if entry.is_none() {
            let (ax, az) = (slot % ACRES_X, slot / ACRES_X);
            let t = retail.field.blocks[(az + 1) * BLOCK_X_NUM + ax + 1];
            *entry = map::field_label(t).map(|label| AcreLabel {
                label,
                residents: Vec::new(),
            });
        }
    }
    labels
}

/// How a villager's house unit maps onto the source's `home_info.ut_z` (the layout tables name
/// the unit in front of the house).
const HOUSE_UT_Z_BIAS: u8 = 1;

/// Facing a map signboard (field items 0x000C / 0x000D).
fn facing_map_board(
    player: &Query<(&Transform, &crate::interaction::Facing), With<Player>>,
    items: Option<&crate::field_items::FieldItems>,
) -> bool {
    let (Some(items), Ok((at, facing))) = (items, player.single()) else {
        return false;
    };
    let ahead = at.translation + facing.dir * 0.6;
    let Some((ax, az, cx, cz)) = Town::cell_indices_of(ahead) else {
        return false;
    };
    items
        .acres
        .get(az * ACRE_WIDTH + ax)
        .is_some_and(|acre| MAP_BOARDS.contains(&acre[cz * 16 + cx]))
}

#[allow(clippy::too_many_arguments)]
fn input(
    mut ui: ResMut<MapUi>,
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    identity: Res<crate::save::PlayerIdentity>,
    town: Res<Town>,
    disc: Option<Res<GameDisc>>,
    font: Option<Res<MsgAssets>>,
    time: Res<Time>,
    state: Res<State<crate::shop::ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    villagers: Query<&crate::npc::Villager>,
    player: Query<(&Transform, &crate::interaction::Facing), With<Player>>,
    items: Option<Res<crate::field_items::FieldItems>>,
    tool: Res<crate::ecology::EquippedTool>,
    mut panel: Query<&mut Visibility, With<MapPanel>>,
    mut text: Query<&mut Text, With<MapText>>,
) {
    let ticks = time.delta_secs() * 60.0;
    let mut plain = false;
    let mut from_board = false;
    if ui.open {
        if let Some(view) = ui.view.as_mut() {
            let mut closed = false;
            if view.slide.advance(ticks) {
                closed = true;
            } else if view.slide.settled() {
                view.cursor.pulse(ticks);
                if view.gliding {
                    view.glide_ticks += ticks;
                    while view.glide_ticks >= 1.0 {
                        view.glide_ticks -= 1.0;
                        if view.cursor.glide() {
                            view.gliding = false;
                            view.glide_ticks = 0.0;
                            break;
                        }
                    }
                } else {
                    // Every close key is consumed, so none reaches the world.
                    let mut closing = false;
                    for action in [
                        Action::Cancel,
                        Action::Back,
                        Action::Map,
                        Action::Confirm,
                        Action::Advance,
                        Action::Interact,
                    ] {
                        let pressed = actions.consume(action);
                        // `mMP_move_Play`: X closes it only when X opened it.
                        closing |= pressed && (action != Action::Map || !view.board);
                    }
                    if closing {
                        view.slide.close();
                        crate::sound::se(0x17D);
                    } else {
                        let direction = [
                            (Action::MoveLeft, (-1, 0)),
                            (Action::MoveRight, (1, 0)),
                            (Action::MoveUp, (0, -1)),
                            (Action::MenuUp, (0, -1)),
                            (Action::MoveDown, (0, 1)),
                            (Action::MenuDown, (0, 1)),
                        ]
                        .into_iter()
                        .find(|(a, _)| actions.pressed(*a));
                        let step = match direction {
                            Some((action, step)) => {
                                let fire = match &mut view.repeat {
                                    Some((held, timer)) if *held == action => {
                                        *timer -= ticks;
                                        if *timer <= 0.0 {
                                            *timer = REPEAT_NEXT;
                                            true
                                        } else {
                                            false
                                        }
                                    }
                                    _ => {
                                        view.repeat = Some((action, REPEAT_FIRST));
                                        true
                                    }
                                };
                                fire.then_some(step)
                            }
                            None => {
                                view.repeat = None;
                                None
                            }
                        };
                        if let Some((dx, dz)) = step {
                            if view.cursor.step(dx, dz) {
                                view.gliding = true;
                                view.glide_ticks = 0.0;
                                crate::sound::se(crate::sound::se_id::CURSOR);
                            }
                        }
                    }
                }
            }
            if closed {
                ui.open = false;
                ui.view = None;
            }
        } else if actions.consume(Action::Cancel)
            || actions.consume(Action::Back)
            || actions.consume(Action::Map)
            || actions.consume(Action::Confirm)
        {
            ui.open = false;
            crate::sound::se(crate::sound::se_id::MENU_EXIT);
        } else {
            plain = true;
        }
    } else if focus.is_world()
        && *state.get() == crate::shop::ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town
        && {
            // `mSM_submenu_ctrl`: X with the map in hand, or the action button facing a map
            // signboard (which needs no map).
            let board = *tool == crate::ecology::EquippedTool::None
                && actions.just_pressed(Action::Interact)
                && facing_map_board(&player, items.as_deref());
            if board {
                actions.consume(Action::Interact);
                from_board = true;
                true
            } else {
                identity.map_flag && actions.consume(Action::Map)
            }
        }
    {
        ui.open = true;
        let here = player
            .single()
            .ok()
            .and_then(|(t, _)| Town::cell_indices_of(t.translation))
            .map(|(ax, az, _, _)| (ax, az));
        let overlay = disc
            .as_deref()
            .zip(font.as_deref())
            .filter(|_| town.retail.is_some())
            .and_then(|(disc, _)| read_tables(disc))
            .zip(here);
        if let Some((tables, (ax, az))) = overlay {
            let retail = town.retail.as_ref().expect("checked above");
            let mut blocks = [map::FLAT; rustimal_logic::random_field::BLOCK_TOTAL_NUM];
            blocks.copy_from_slice(&retail.field.blocks);
            let pictures = map::acre_block_types(&blocks, None, &tables.plus_bridge);
            ui.view = Some(View {
                board: from_board,
                cursor: Cursor::new(ax, az),
                slide: Slide::new(),
                pictures,
                labels: labels_for(&town, &tables, &identity, &villagers),
                gliding: false,
                glide_ticks: 0.0,
                repeat: None,
            });
            crate::sound::se(0x17C);
        } else {
            crate::sound::se(crate::sound::se_id::CHOICE_OPEN);
            if let Ok(mut text) = text.single_mut() {
                text.0 = render(&town, here);
            }
            plain = true;
        }
    }
    if let Ok(mut visibility) = panel.single_mut() {
        *visibility = if ui.open && (plain || ui.view.is_none()) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Disc models built on demand, and the entities of the drawn map.
#[derive(Default)]
struct Drawn {
    models: HashMap<String, Model>,
    font_material: Option<Handle<StandardMaterial>>,
    /// Which of [`COLOR_LIST`] the cached models were built with.
    color: usize,
    tables: Option<Arc<Tables>>,
    root: Option<Entity>,
    body: Option<Entity>,
    cursor: Option<Entity>,
    cursor_model: Model,
    key: Option<(usize, usize)>,
    meshes: Vec<Handle<Mesh>>,
}

/// The bytes of a disc table entry as a segment.
fn rel_segment(disc: &GameDisc, at: u32, len: usize) -> Arc<[u8]> {
    let rel = disc.rom.bytes(RomSource::Rel);
    rel.get(at as usize..(at as usize + len).min(rel.len()))
        .unwrap_or_default()
        .into()
}

fn build(
    disc: &GameDisc,
    names: &[&str],
    color: &str,
    waku: &str,
    segments: &Segments,
    prim: [u8; 4],
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> Model {
    let lists: Vec<_> = names
        .iter()
        .filter_map(|n| disc.catalog.get(n).map(|a| a.offset))
        .collect();
    // `mMP_set_base_dl` puts the window's colour list in segment B, `mMP_set_win_dl` the label
    // area's list in segment A.
    let dl_segments: Vec<(u8, u32)> = [(0xB, color), (0xA, waku)]
        .into_iter()
        .filter_map(|(seg, name)| Some((seg, disc.catalog.get(name)?.offset)))
        .collect();
    let sink = run_display_lists_with_segment_lists(&disc.rom, &lists, prim, &dl_segments);
    crate::disc_town::sink_meshes_scaled(
        disc,
        &sink,
        None,
        true,
        segments,
        PX,
        (meshes, materials, images),
    )
}

fn label_model(label: Label) -> &'static str {
    match label {
        Label::Npc => "kan_win_npcT_1_model",
        Label::Player => "kan_win_playerT_model",
        Label::Shop => "kan_win_omiseT_model",
        Label::Police => "kan_win_koubanT_model",
        Label::Post => "kan_win_yuuT_model",
        Label::Shrine => "kan_win_yashiroT_model",
        Label::Station => "kan_win_ekiT_model",
        Label::Junk => "kan_win_gomiT_model",
        Label::Museum => "kan_win_mu_model",
        Label::Needle => "kan_win_ta_model",
        Label::Port => "kan_win_funeT_model",
    }
}

fn npc_icon(layer: usize) -> &'static str {
    [
        "kan_win_npcT_1_model",
        "kan_win_npcT_2_model",
        "kan_win_npcT_3_model",
    ][layer.min(2)]
}

fn npc_mark(layer: usize) -> &'static str {
    [
        "kan_win_npc2T_1_model",
        "kan_win_npc2T_2_model",
        "kan_win_npc2T_3_model",
    ][layer.min(2)]
}

/// A pixel position (y up, from the screen centre) as overlay units.
fn at(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x * PX, y * PX, z)
}

/// A disc model, built the first time it is asked for.
#[allow(clippy::too_many_arguments)]
fn cached(
    drawn: &mut Drawn,
    disc: &GameDisc,
    color: usize,
    gpu: (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
    name: String,
    names: &[&str],
    waku: &str,
    segments: Segments,
    prim: [u8; 4],
) -> Model {
    drawn
        .models
        .entry(name)
        .or_insert_with(|| build(disc, names, COLOR_LIST[color], waku, &segments, prim, gpu))
        .clone()
}

/// A line of text from its top-left corner (screen pixels), scaled about that corner.
#[allow(clippy::too_many_arguments)]
fn put_text(
    commands: &mut Commands,
    drawn: &mut Drawn,
    meshes: &mut Assets<Mesh>,
    font: &MsgAssets,
    material: &Handle<StandardMaterial>,
    parent: Entity,
    s: &str,
    origin: Vec2,
    color: [u8; 4],
    scale: Vec2,
) {
    let Some(mesh) = text_mesh(s, Vec2::ZERO, &font.widths, color, usize::MAX) else {
        return;
    };
    let mesh = meshes.add(mesh);
    drawn.meshes.push(mesh.clone());
    let corner = screen(Vec2::ZERO);
    let position = screen(origin) - Vec3::new(scale.x * corner.x, scale.y * corner.y, 0.0);
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(position.with_z(6.0)).with_scale(scale.extend(1.0)),
        RenderLayers::layer(1),
        ChildOf(parent),
    ));
}

#[allow(clippy::too_many_arguments)]
fn draw(
    mut commands: Commands,
    ui: Res<MapUi>,
    disc: Option<Res<GameDisc>>,
    font: Option<Res<MsgAssets>>,
    identity: Res<crate::save::PlayerIdentity>,
    mut drawn: Local<Drawn>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut n64: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
    mut standard: ResMut<Assets<StandardMaterial>>,
    mut transforms: Query<&mut Transform>,
) {
    let (Some(view), Some(disc), Some(font)) = (ui.view.as_ref(), disc, font) else {
        for entity in [drawn.root.take()].into_iter().flatten() {
            commands.entity(entity).despawn();
            drawn.body = None;
            drawn.cursor = None;
            drawn.key = None;
        }
        return;
    };
    let color = usize::from(!view.board);
    if drawn.color != color {
        drawn.color = color;
        drawn.models.clear();
        drawn.key = None;
    }
    let tables = drawn
        .tables
        .get_or_insert_with(|| Arc::new(read_tables(&disc).expect("read when the map opened")))
        .clone();
    let root = *drawn.root.get_or_insert_with(|| {
        commands
            .spawn((
                Name::new("original town map"),
                Transform::default(),
                Visibility::default(),
                RenderLayers::layer(1),
            ))
            .id()
    });
    if let Ok(mut transform) = transforms.get_mut(root) {
        transform.translation = at(0.0, view.slide.pos, 0.0);
    }
    let key = (view.cursor.bx, view.cursor.bz);
    // The cursor follows the glide and pulses every frame.
    if drawn.cursor_model.is_empty() {
        let model = build(
            &disc,
            &["kan_win_mode", "kan_win_cursorT_model"],
            COLOR_LIST[1],
            "",
            &Segments::default(),
            [255, 0, 255, 255],
            (&mut meshes, &mut n64, &mut images),
        );
        drawn.cursor_model = model;
    }
    let frame = view.cursor.pulse_frame();
    for (_, handle) in &drawn.cursor_model {
        if let Some(mut material) = n64.get_mut(handle) {
            material.combiner.prim = Vec4::new(1.0, f32::from(CURSOR_G[frame]) / 255.0, 1.0, 1.0);
        }
    }
    let (cx, cz) = view.cursor.position();
    let cursor_at = at(cx + MAP_ORIGIN.x, cz + MAP_ORIGIN.y, 5.0);
    let cursor_scale = Vec3::new(CURSOR_SCALE[frame], CURSOR_SCALE[frame], 1.0);
    if let Some(cursor) = drawn.cursor {
        if let Ok(mut transform) = transforms.get_mut(cursor) {
            *transform = Transform::from_translation(cursor_at).with_scale(cursor_scale);
        }
    } else {
        let cursor = commands
            .spawn((
                Name::new("map cursor"),
                Transform::from_translation(cursor_at).with_scale(cursor_scale),
                Visibility::default(),
                RenderLayers::layer(1),
                ChildOf(root),
            ))
            .id();
        for (i, (mesh, material)) in drawn.cursor_model.iter().enumerate() {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.0, 0.0, i as f32 * 0.02),
                RenderLayers::layer(1),
                ChildOf(cursor),
            ));
        }
        drawn.cursor = Some(cursor);
    }
    if drawn.key == Some(key) && drawn.body.is_some() {
        return;
    }
    drawn.key = Some(key);
    if let Some(body) = drawn.body.take() {
        commands.entity(body).despawn();
    }
    for handle in drawn.meshes.drain(..) {
        meshes.remove(handle.id());
    }
    let body = commands
        .spawn((
            Name::new("map body"),
            Transform::default(),
            Visibility::default(),
            RenderLayers::layer(1),
            ChildOf(root),
        ))
        .id();
    drawn.body = Some(body);
    let place = |commands: &mut Commands, model: &Model, position: Vec3| {
        for (i, (mesh, material)) in model.iter().enumerate() {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(position + Vec3::Z * (i as f32 * 0.02)),
                RenderLayers::layer(1),
                ChildOf(body),
            ));
        }
    };
    macro_rules! model_for {
        ($name:expr, $names:expr, $waku:expr, $segments:expr, $prim:expr $(,)?) => {
            cached(
                &mut drawn,
                &disc,
                color,
                (&mut meshes, &mut n64, &mut images),
                $name,
                $names,
                $waku,
                $segments,
                $prim,
            )
        };
    }
    // The window.
    let frame_model = model_for!(
        "frame".into(),
        &["kan_win_model"],
        "",
        Segments::default(),
        [255; 4],
    );
    place(&mut commands, &frame_model, Vec3::ZERO);
    // The acre pictures (`mMP_set_map_dl`).
    let table = disc
        .catalog
        .get("l_map_texture")
        .map(|a| a.offset)
        .unwrap_or(0);
    for (slot, &block_type) in view.pictures.iter().enumerate() {
        let (bx, bz) = (slot % ACRES_X, slot / ACRES_X);
        let Some(texture) = pointer(&disc, table + 4 * u32::from(block_type)) else {
            continue;
        };
        let palette = usize::from(
            tables
                .pals
                .get(usize::from(block_type))
                .copied()
                .unwrap_or(0),
        )
        .min(1);
        let model = model_for!(
            format!("acre {texture:x} {palette}"),
            &["kan_tizu_mode", "kan_tizu_model"],
            "",
            {
                let mut segments = Segments::default();
                segments.set_segment(8, rel_segment(&disc, texture, 0x200));
                if let Some(pal) = disc
                    .catalog
                    .get(["kan_tizu1_pal", "kan_tizu2_pal"][palette])
                {
                    segments.set_segment(9, rel_segment(&disc, pal.offset, 0x20));
                }
                segments
            },
            [255; 4],
        );
        place(
            &mut commands,
            &model,
            at(
                MAP_ORIGIN.x + bx as f32 * BLOCK_SIZE,
                MAP_ORIGIN.y - bz as f32 * BLOCK_SIZE,
                1.0,
            ),
        );
    }
    // The selected acre's coordinates and its label panel (`mMP_set_win_dl`).
    let selected = view.labels[view.cursor.bz * ACRES_X + view.cursor.bx].as_ref();
    let count = match selected {
        Some(l) if matches!(l.label, Label::Npc | Label::Player) => l.residents.len(),
        Some(_) => 2,
        None => 0,
    };
    let waku = match count {
        4 => "kan_waku_w4T_model",
        3 => "kan_waku_w3T_model",
        1 => "kan_win_wakuT_model",
        0 => "kan_waku_w1T_model",
        _ => "kan_waku_w2T_model",
    };
    let win = model_for!(
        format!("win {} {} {waku}", view.cursor.bx, view.cursor.bz),
        &["kan_win_model2"],
        waku,
        {
            let mut segments = Segments::default();
            for (seg, table_name, index) in [
                (8, "kan_win_evw_anime_1_tex_table", view.cursor.bx),
                (9, "kan_win_evw_anime_2_tex_table", view.cursor.bz),
            ] {
                if let Some(t) = disc.catalog.get(table_name) {
                    if let Some(p) = pointer(&disc, t.offset + 4 * index as u32) {
                        segments.set_segment(seg, rel_segment(&disc, p, 0x100));
                    }
                }
            }
            segments
        },
        [255; 4],
    );
    place(&mut commands, &win, at(0.0, 0.0, 2.0));
    // Label icons of the selected acre (`mMP_set_label_top_dl`).
    if let Some(selected) = selected {
        let ((ox, oy), words) = selected.label.layout();
        let icons = match selected.label {
            Label::Npc | Label::Player => selected.residents.len(),
            _ => 1,
        };
        for i in 0..icons {
            let layer = selected.residents.get(i).map_or(0, |r| r.layer);
            let name = if selected.label == Label::Npc && layer != 0 {
                npc_icon(layer)
            } else {
                label_model(selected.label)
            };
            let model = model_for!(
                format!("icon {name}"),
                &["kan_win_mode", name],
                "",
                Segments::default(),
                [255; 4],
            );
            place(&mut commands, &model, at(ox, oy - 12.0 * i as f32, 3.0));
        }
        // Names.
        let font_material = drawn
            .font_material
            .get_or_insert_with(|| {
                standard.add(StandardMaterial {
                    base_color_texture: Some(font.font.clone()),
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    cull_mode: None,
                    ..default()
                })
            })
            .clone();
        macro_rules! text {
            ($s:expr, $origin:expr, $color:expr, $scale:expr $(,)?) => {
                put_text(
                    &mut commands,
                    &mut drawn,
                    &mut meshes,
                    &font,
                    &font_material,
                    body,
                    $s,
                    $origin,
                    $color,
                    $scale,
                )
            };
        }
        let mut residents = selected.residents.iter();
        for word in words {
            match word.text {
                Some(s) => text!(
                    s,
                    Vec2::new(160.0 + word.x, 120.0 - word.y),
                    BUILDING_COLOR,
                    Vec2::splat(0.75),
                ),
                None => {
                    let mut y = 120.0 - word.y;
                    for resident in residents.by_ref() {
                        let width = text_width(&resident.name, &font.widths).max(1.0);
                        let sx = (54.0 / width).min(0.75);
                        text!(
                            &resident.name,
                            Vec2::new(160.0 + word.x, y),
                            if resident.known {
                                RESIDENT_COLOR
                            } else {
                                FREE_COLOR
                            },
                            Vec2::new(sx, 0.75),
                        );
                        y += 12.0;
                    }
                }
            }
        }
        // The town name.
        text!(
            &identity.town,
            Vec2::new(-136.0 + 24.0 + 160.0, 120.0 - (102.0 - 29.0)),
            LAND_COLORS[color],
            Vec2::ONE,
        );
    } else {
        let font_material = drawn
            .font_material
            .get_or_insert_with(|| {
                standard.add(StandardMaterial {
                    base_color_texture: Some(font.font.clone()),
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    cull_mode: None,
                    ..default()
                })
            })
            .clone();
        if let Some(mesh) = text_mesh(
            &identity.town,
            Vec2::new(-136.0 + 24.0 + 160.0, 120.0 - (102.0 - 29.0)),
            &font.widths,
            LAND_COLORS[color],
            usize::MAX,
        ) {
            let mesh = meshes.add(mesh);
            drawn.meshes.push(mesh.clone());
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(font_material),
                Transform::from_xyz(0.0, 0.0, 6.0),
                RenderLayers::layer(1),
                ChildOf(body),
            ));
        }
    }
    // The villagers' house marks and the current-acre mark (`mMP_set_house_dl`).
    for (slot, label) in view.labels.iter().enumerate() {
        let Some(label) = label.as_ref().filter(|l| l.label == Label::Npc) else {
            continue;
        };
        let (bx, bz) = (slot % ACRES_X, slot / ACRES_X);
        for resident in &label.residents {
            let model = model_for!(
                format!("mark {}", npc_mark(resident.layer)),
                &["kan_win_mode", npc_mark(resident.layer)],
                "",
                Segments::default(),
                [255; 4],
            );
            place(
                &mut commands,
                &model,
                at(
                    MAP_ORIGIN.x - BLOCK_SIZE / 2.0
                        + bx as f32 * BLOCK_SIZE
                        + HOUSE_OFFSET_X[resident.idx % 3],
                    MAP_ORIGIN.y + BLOCK_SIZE / 2.0 - bz as f32 * BLOCK_SIZE
                        + HOUSE_OFFSET_Y[resident.idx / 3],
                    4.0,
                ),
            );
        }
    }
    let here = model_for!(
        "here".into(),
        &["kan_win_mode", "kan_win_genzaiT_model"],
        "",
        Segments::default(),
        [255; 4],
    );
    let (px, pz) = view.cursor.player;
    place(
        &mut commands,
        &here,
        at(
            MAP_ORIGIN.x - BLOCK_SIZE / 2.0 + px as f32 * BLOCK_SIZE + 8.0,
            MAP_ORIGIN.y - BLOCK_SIZE / 2.0 - pz as f32 * BLOCK_SIZE + 9.0,
            4.0,
        ),
    );
}

pub struct TownMapPlugin;

impl Plugin for TownMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapUi>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    input
                        .in_set(crate::input::InteractSet::Ui)
                        .in_set(crate::states::GameplaySet),
                    draw.after(input),
                ),
            );
    }
}
