//! Player-room furniture transactions. Numeric foreground layers are saved;
//! meshes, profiles and textures continue to come from the owner's disc.
//!
//! Source: ac_my_room's actor construction/KeepItem2Fg, ac_my_room_move's
//! placement/pickup, and ac_my_room_msg_ctrl's three-item storage choices.
//! Placement/pickup/storage commit immediately; movement commits its numeric
//! destination before playing the original clips. Birth and pickup have
//! temporary disc-backed presentation, including supported storage doors.
use std::collections::HashMap;

use bevy::prelude::*;
use disc::{RomSource, Target};
use rustimal_logic::furniture::{
    self as ftr, Direction, FurnitureProfile, MyRoomRuntime, RoomLayer, SetType, ShapeType,
};
use serde::{Deserialize, Serialize};

use crate::dialogue::DialogueState;
use crate::game_disc::GameDisc;
use crate::house::{CurrentRoom, HouseState, InteriorObstacles, INTERIOR_ORIGIN};
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::player::Player;
use crate::pockets::PocketsUi;
use crate::states::GameplaySet;

mod loose;
mod loose_presentation;
mod movement;
mod music;
mod music_grid;
mod presentation;
mod storage;

/// Four 16x16 source layers, including owner-cell storage save-back.
/// An absent field means an old save's default room; an empty room is explicit.
#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "RoomSaveData")]
pub(crate) struct RoomSave {
    layers: [Vec<u16>; 4],
    switches: [u64; 4],
    music_box: [u32; 2],
}

#[derive(Deserialize)]
struct RoomSaveData {
    layers: [Vec<u16>; 4],
    switches: [u64; 4],
    #[serde(default)]
    music_box: [u32; 2],
}

impl TryFrom<RoomSaveData> for RoomSave {
    type Error = &'static str;
    fn try_from(data: RoomSaveData) -> Result<Self, Self::Error> {
        if data.layers.iter().any(|l| l.len() != 256) {
            return Err("home furniture layers must each contain 256 cells");
        }
        Ok(Self {
            layers: data.layers,
            switches: data.switches,
            music_box: data.music_box,
        })
    }
}

pub(crate) enum PocketRequest {
    Place {
        slot: usize,
        item: u16,
    },
    Store {
        owner: usize,
        slot: usize,
        item: u16,
    },
    Register {
        owner: usize,
        slot: usize,
        item: u16,
    },
}

#[derive(Default)]
enum StorageFlow {
    #[default]
    Idle,
    Opening(usize),
    Choose(usize),
    Take {
        owner: usize,
        slots: Vec<usize>,
    },
    Pockets,
    Notice,
    Music(music::Flow),
}

#[derive(Resource)]
pub(crate) struct PlayerRoom {
    music_grid: Option<music_grid::Grid>,
    music_box: [u32; 2],
    loose_angles: loose::Angles,
    movement: movement::Movement,
    presentation: presentation::Presentation,
    loose_presentation: loose_presentation::Presentation,
    storage: storage::Storage,
    layers: [RoomLayer; 4],
    runtime: MyRoomRuntime,
    profiles: HashMap<u16, FurnitureProfile>,
    original: Option<RoomSave>,
    initialized: bool,
    dirty: bool,
    flow: StorageFlow,
    /// Refreshed before pockets can open; never true in a villager's room.
    pub(crate) available: bool,
}

impl Default for PlayerRoom {
    fn default() -> Self {
        Self {
            music_grid: None,
            music_box: [0; 2],
            loose_angles: [[0; 16]; 16],
            movement: movement::Movement::default(),
            presentation: presentation::Presentation::default(),
            loose_presentation: loose_presentation::Presentation::default(),
            storage: storage::Storage::default(),
            layers: std::array::from_fn(|_| RoomLayer::default()),
            runtime: MyRoomRuntime::new(32), // aMR_GetSceneFurnitureMax: small room
            profiles: HashMap::new(),
            original: None,
            initialized: false,
            dirty: true,
            flow: StorageFlow::Idle,
            available: false,
        }
    }
}

fn layer_views(layers: &mut [RoomLayer; 4]) -> [Option<&mut RoomLayer>; 4] {
    let [a, b, c, d] = layers;
    [Some(a), Some(b), Some(c), Some(d)]
}

impl PlayerRoom {
    pub(crate) fn manipulating(&self) -> bool {
        self.music_grid.is_some()
            || self.presentation.busy()
            || self.loose_presentation.busy()
            || self.storage.busy()
            || self.movement.grip.is_some()
            || self.movement.active.is_some()
    }

    pub(crate) fn animation(&self) -> Option<(&crate::skeleton::Animation, f32, u8)> {
        self.presentation
            .animation()
            .or_else(|| self.loose_presentation.animation())
            .or_else(|| self.storage.animation())
            .or_else(|| self.movement.animation())
    }
    pub(crate) fn snapshot(&self) -> Option<RoomSave> {
        if !self.initialized {
            return self.original.clone();
        }
        let mut layers = self.layers.clone();
        for actor in self.runtime.actors.iter().flatten() {
            // Save-back clears successfully written slots. Clone the actor so
            // F5 never empties the live wardrobe.
            ftr::keep_items_to_fg(&mut actor.clone(), &mut layer_views(&mut layers));
        }
        Some(RoomSave {
            layers: std::array::from_fn(|i| layers[i].items.iter().flatten().copied().collect()),
            switches: std::array::from_fn(|i| layers[i].ftr_switch),
            music_box: self.music_box,
        })
    }

    pub(crate) fn restore(&mut self, saved: Option<&RoomSave>, disc: Option<&GameDisc>) {
        self.music_grid = None;
        self.loose_angles = [[0; 16]; 16];
        self.movement.reset();
        self.presentation.reset();
        self.loose_presentation.reset();
        self.storage.reset();
        self.layers = std::array::from_fn(|_| RoomLayer::default());
        self.original = saved.cloned();
        self.music_box = saved.map_or([0; 2], |s| s.music_box);
        self.initialized = saved.is_some() || disc.is_some();
        self.flow = StorageFlow::Idle;
        if let Some(saved) = saved {
            for (i, layer) in self.layers.iter_mut().enumerate() {
                for (cell, &item) in layer.items.iter_mut().flatten().zip(&saved.layers[i]) {
                    *cell = item;
                }
                layer.ftr_switch = saved.switches[i];
            }
        } else if let Some(disc) = disc {
            // mHm_SetDefaultPlayerRoomData. Resolve original numeric IDs from
            // the runtime tables instead of the previous hardcoded mesh names.
            if let Some(bytes) = disc
                .catalog
                .get("l_mHm_player_room_default_data")
                .and_then(|s| disc.rom.slice(s.source, s.offset, 16))
            {
                let x = i32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
                let z = i32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
                self.layers[0].set(x, z, u16::from_be_bytes([bytes[12], bytes[13]]));
            }
            if let (Some(table), Some(box_profile)) = (
                disc.catalog.get("furniture_quality"),
                disc.catalog.get("iam_nog_mikanbox"),
            ) {
                let index = (0..table.size/4).find(|i| matches!(disc.rom.pointer_at(RomSource::Rel,table.offset+i*4),Some(Target::File(RomSource::Rel,p)) if p==box_profile.offset));
                if let Some(index) = index {
                    self.layers[0].set(
                        1,
                        1,
                        ftr::ftr_idx_2_item_no(index as u16, Direction::South),
                    );
                }
            }
            // mHm_SetDefaultPlayerRoomData: diary on the orange box. Only new
            // default rooms receive it; saved/explicitly empty rooms stay exact.
            self.layers[1].set(1, 1, 0x2B00);
        }
        self.rebuild();
    }

    fn rebuild(&mut self) {
        self.runtime = MyRoomRuntime::new(32);
        let profiles = &self.profiles;
        let mut bank = |idx, _| profiles.get(&idx).map(|p| p.shape);
        ftr::make_furniture_actors_from_layers(
            [Some(&self.layers[0]), None],
            &mut self.runtime,
            &mut bank,
        );
        // A stored furniture leaf in layer 1 belongs to its layer-0 wardrobe;
        // it is not another placed actor. Keep the real layer for restoration.
        let mut visible_secondary = self.layers[1].clone();
        for z in 0..16 {
            for x in 0..16 {
                if self.runtime.occupancy[0][z][x]
                    .and_then(|id| self.runtime.actors[id as usize].as_ref())
                    .and_then(|a| profiles.get(&a.name))
                    .is_some_and(|p| ftr::is_storage(p.interaction_type))
                {
                    visible_secondary.items[z][x] = 0;
                }
            }
        }
        ftr::make_furniture_actors_from_layers(
            [None, Some(&visible_secondary)],
            &mut self.runtime,
            &mut bank,
        );
        ftr::restore_stored_items_from_fg(
            &mut self.runtime,
            &mut layer_views(&mut self.layers),
            &|idx| {
                profiles
                    .get(&idx)
                    .is_some_and(|p| ftr::is_storage(p.interaction_type))
            },
        );
        self.dirty = true;
    }

    fn place(
        &mut self,
        item: u16,
        pos: Vec3,
        facing: Vec3,
        disc: &GameDisc,
    ) -> Result<usize, &'static str> {
        let item = ftr::clothing_to_furniture(item).unwrap_or(item);
        let (idx, _) = ftr::ftr_item_no_2_idx_dir(item).ok_or("That isn't furniture.")?;
        let p = self
            .profiles
            .get(&idx)
            .ok_or("That furniture isn't available yet.")?;
        if !crate::furniture::can_render(disc, idx) {
            return Err("That furniture isn't available yet.");
        }
        let slot = self
            .runtime
            .search_free_actor()
            .ok_or("There's no room for any more furniture.")?;
        if !ftr::weight_possible(self.runtime.count_furniture(), self.runtime.list_size) {
            return Err("There's no room for any more furniture.");
        }
        let cell = player_cell(pos);
        let angle = facing_angle(facing);
        let base = ftr::player_facing_to_angle_idx(angle) as u8;
        let rotation = direction(base.wrapping_add(ftr::special_orientation_offset(idx) as u8) & 3);
        // The inherited judge_place_furniture models a straight five-cell
        // search. Retail instead checks adjacent units at angular offsets
        // 0,+2,-2,+4,-4 in its 16-direction table. Use that source search here.
        let mut ground = self.layers[0].clone();
        let mut occupied = [[false; 16]; 16];
        for (z, row) in occupied.iter_mut().enumerate() {
            for (x, taken) in row.iter_mut().enumerate() {
                *taken = !(1..=4).contains(&x)
                    || !(1..=4).contains(&z)
                    || self.runtime.occupancy[0][z][x].is_some();
            }
        }
        // Keep the player's collision circle clear, including across a cell
        // boundary; this also prevents placement from trapping the player.
        let local = pos - corner();
        for z in 1..=4 {
            for x in 1..=4 {
                if local.x + 0.4 > x as f32
                    && local.x - 0.4 < (x + 1) as f32
                    && local.z + 0.4 > z as f32
                    && local.z - 0.4 < (z + 1) as f32
                {
                    ground.items[z][x] = 0xFFFF;
                }
            }
        }
        let rotations: Vec<Direction> = if p.shape.size_class() == ftr::FurnitureSize::Size1x2 {
            std::iter::once(rotation)
                .chain((0..4).map(direction))
                .collect()
        } else {
            vec![rotation]
        };
        let mut chosen = None;
        'search: for rotation in rotations {
            for n in 0..5 {
                let (x, z) = look_at(cell, angle, n);
                if !(1..=4).contains(&x) || !(1..=4).contains(&z) {
                    continue;
                }
                if p.set_type == SetType::OnSurface
                    && p.shape == ShapeType::TypeA
                    && self.layers[1].get(x, z) == Some(0)
                    && self.runtime.occupancy[1][z as usize][x as usize].is_none()
                {
                    let below = self.runtime.occupancy[0][z as usize][x as usize]
                        .and_then(|id| self.runtime.actors[id as usize].as_ref());
                    if below
                        .and_then(|a| self.profiles.get(&a.name))
                        .is_some_and(|p| {
                            p.set_type == SetType::Surface && !ftr::is_storage(p.interaction_type)
                        })
                    {
                        chosen = Some((x, z, rotation, 1));
                        break 'search;
                    }
                }
                let shape = rotated(p.shape, rotation);
                if ftr::check_place_situation(shape, x, z, &ground, &occupied) {
                    chosen = Some((x, z, rotation, 0));
                    break 'search;
                }
            }
        }
        let (x, z, rotation, layer) = chosen.ok_or("There's no room to put that here.")?;
        let shape = rotated(p.shape, rotation);
        if !self.runtime.occupy(layer, shape, x, z, slot as u16) {
            return Err("There's no room to put that here.");
        }
        ftr::set_furniture_to_fg(&mut self.layers[layer], idx, shape, rotation, x, z, true);
        if let Some(bit) = RoomLayer::switch_bit_index(x, z) {
            self.layers[layer].ftr_switch &= !(1u64 << bit);
        }
        let mut actor = ftr::FurnitureActor::new(idx, shape, layer, x, z);
        actor.original_shape_type = p.shape;
        actor.angle_y = rotation as u8 as f32 * 90.0;
        actor.angle_y_target = actor.angle_y;
        actor.s_angle_y = ((rotation as u16) << 14) as i16;
        self.runtime.actors[slot] = Some(actor);
        music::placed(self, slot);
        self.dirty = true;
        info!("placed furniture {item:#06x} at ({x},{z}), layer {layer}");
        Ok(slot)
    }

    fn pickup(&mut self, slot: usize, inventory: &mut PlayerInventory) -> Result<(), &'static str> {
        let Some(actor) = self.runtime.actors.get(slot).and_then(Option::as_ref) else {
            return Ok(());
        };
        if actor.layer == 0
            && actor
                .shape_type
                .unit_offsets()
                .iter()
                .take(actor.shape_type.unit_count())
                .any(|&(dx, dz)| {
                    self.layers[1].get(actor.cell_x + dx, actor.cell_z + dz) != Some(0)
                })
        {
            return Err("Take the things off the top first.");
        }
        let furniture_item = ftr::ftr_idx_2_item_no(actor.name, Direction::South);
        let mut received =
            vec![ftr::clothing_from_furniture(furniture_item).unwrap_or(furniture_item)];
        // A music player's selected title refers to the shared collection;
        // it is not another owned copy to return with the furniture leaf.
        received.extend(
            actor
                .items
                .iter()
                .copied()
                .enumerate()
                .filter_map(|(i, item)| {
                    (item != 0
                        && !(i == 0
                            && music::is_player(self, slot)
                            && music::registered(self, item)))
                    .then_some(item)
                }),
        );
        let mut pockets = inventory.pockets.clone();
        for item in received {
            if pockets.put(item, 0).is_none() {
                return Err("Your pockets are full.");
            }
        }
        ftr::set_furniture_to_fg(
            &mut self.layers[actor.layer],
            actor.name,
            actor.shape_type,
            Direction::South,
            actor.cell_x,
            actor.cell_z,
            false,
        );
        if let Some(bit) = RoomLayer::switch_bit_index(actor.cell_x, actor.cell_z) {
            self.layers[actor.layer].ftr_switch &= !(1u64 << bit);
        }
        self.runtime.release(slot as u16);
        self.runtime.actors[slot] = None;
        inventory.pockets = pockets;
        self.dirty = true;
        Ok(())
    }
}

fn direction(n: u8) -> Direction {
    match n & 3 {
        1 => Direction::East,
        2 => Direction::North,
        3 => Direction::West,
        _ => Direction::South,
    }
}
fn rotated(shape: ShapeType, dir: Direction) -> ShapeType {
    if shape == ShapeType::TypeB0 {
        [
            ShapeType::TypeB0,
            ShapeType::TypeB90,
            ShapeType::TypeB180,
            ShapeType::TypeB270,
        ][dir as usize]
    } else {
        shape
    }
}
fn corner() -> Vec3 {
    INTERIOR_ORIGIN - Vec3::new(3.0, 0.0, 3.0)
}
fn player_cell(pos: Vec3) -> (i32, i32) {
    let p = pos - corner();
    (p.x.floor() as i32, p.z.floor() as i32)
}
fn facing_angle(facing: Vec3) -> u16 {
    ((facing.x.atan2(facing.z).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU * 65536.0)
        as u32
        & 0xFFFF) as u16
}
fn look_at((x, z): (i32, i32), angle: u16, candidate: usize) -> (i32, i32) {
    const UNITS: [(i32, i32); 16] = [
        (0, 1),
        (1, 1),
        (1, 1),
        (1, 0),
        (1, 0),
        (1, -1),
        (1, -1),
        (0, -1),
        (0, -1),
        (-1, -1),
        (-1, -1),
        (-1, 0),
        (-1, 0),
        (-1, 1),
        (-1, 1),
        (0, 1),
    ];
    const HIGH: [i32; 5] = [0, 2, -2, 4, -4];
    let (dx, dz) = UNITS[(((angle >> 12) as i32 + HIGH[candidate]) & 15) as usize];
    (x + dx, z + dz)
}

/// mSM_check_item_for_furniture; conditions are not stored by furniture.
pub(crate) fn can_store(item: u16, condition: u32) -> bool {
    condition == 0
        && item >> 12 == 2
        && item != 0x2530
        && !(0x2523..=0x252F).contains(&item)
        && !(0x2300..=0x23FF).contains(&item)
        && !(0x2D00..=0x2DFF).contains(&item)
        && !(0x2F00..=0x2FFF).contains(&item)
}

pub(crate) fn can_place_in_room(item: u16) -> bool {
    crate::items::is_furniture(item)
        || ftr::clothing_to_furniture(item).is_some()
        || loose::supports(item)
}

fn setup(
    mut room: ResMut<PlayerRoom>,
    disc: Option<Res<GameDisc>>,
    loaded: Res<crate::save::LoadedSave>,
) {
    if let Some(disc) = disc.as_deref() {
        if let Some(table) = disc.catalog.get("furniture_quality") {
            for idx in 0..table.size / 4 {
                if let Some(p) = crate::furniture::gameplay_profile(disc, idx as u16) {
                    room.profiles.insert(idx as u16, p);
                }
            }
        }
    }
    room.restore(
        loaded.0.as_ref().and_then(|s| s.home_furniture.as_ref()),
        disc.as_deref(),
    );
    room.movement.load(disc.as_deref());
    room.presentation.load(disc.as_deref());
    room.loose_presentation.load(disc.as_deref());
    room.storage.load(disc.as_deref());
    info!(
        "player furniture: {} profiles, {} placed actors",
        room.profiles.len(),
        room.runtime.count_furniture()
    );
}

fn notice(
    dialogue: &mut DialogueState,
    room: &mut PlayerRoom,
    focus: &mut InputFocus,
    message: &str,
) {
    dialogue.begin_furniture(message.into(), None);
    room.flow = StorageFlow::Notice;
    *focus = InputFocus::Dialogue;
}

fn storage_choices(
    dialogue: &mut DialogueState,
    room: &mut PlayerRoom,
    focus: &mut InputFocus,
    owner: usize,
) {
    dialogue.begin_furniture(
        "What would you like to do?".into(),
        Some(vec![
            "Put something in".into(),
            "Take something out".into(),
            "Quit".into(),
        ]),
    );
    room.flow = StorageFlow::Choose(owner);
    *focus = InputFocus::Dialogue;
}

// Inventory, actor storage and UI ownership commit in one system to avoid
// partial transfers if a menu closes, the pockets fill, or the scene changes.
#[allow(clippy::too_many_arguments)]
fn interact(
    mut room: ResMut<PlayerRoom>,
    disc: Option<Res<GameDisc>>,
    current: Res<CurrentRoom>,
    state: Res<State<HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), With<Player>>,
    tool: Res<crate::ecology::EquippedTool>,
    mut inventory: ResMut<PlayerInventory>,
    mut ui: ResMut<PocketsUi>,
    mut dialogue: ResMut<DialogueState>,
    rigs: Query<(&RoomPiece, &crate::furniture::storage::StorageRig)>,
) {
    room.available = disc.is_some() && crate::house::is_player_room(&current, &state);
    if room.music_grid.is_some() {
        return;
    }
    let request = ui.room_request.take();
    if !room.available {
        ui.home = false;
        ui.storing = None;
        ui.registering = false;
        room.flow = StorageFlow::Idle;
        room.storage.reset();
        return;
    }
    let Some(disc) = disc else { return };
    let Ok((transform, facing)) = player.single() else {
        return;
    };
    if let Some(request) = request {
        let result = match request {
            PocketRequest::Register { owner, slot, item } => {
                music::register(&mut room, &mut inventory, owner, slot, item)
            }
            PocketRequest::Place { slot, item }
                if inventory.pockets.pockets.get(slot) == Some(&item)
                    && inventory.pockets.item_cond(slot) == 0 =>
            {
                let result = if loose::supports(item) {
                    loose::place(&mut room, item, transform.translation, facing.dir, &disc).map(
                        |piece| {
                            let at = loose::position(&room, piece);
                            room.loose_presentation
                                .drop(piece, at, transform.translation);
                            None
                        },
                    )
                } else {
                    room.place(item, transform.translation, facing.dir, &disc)
                        .map(Some)
                };
                if result.is_ok() {
                    inventory.pockets.pockets[slot] = 0;
                    inventory.pockets.set_item_cond(slot, 0);
                }
                result.map(|slot| {
                    if let Some((slot, actor)) = slot.and_then(|slot| {
                        room.runtime.actors[slot]
                            .as_ref()
                            .cloned()
                            .map(|a| (slot, a))
                    }) {
                        let at = presentation::actor_center(&room, &actor);
                        room.presentation.birth(
                            slot,
                            ftr::clothing_to_furniture(item).unwrap_or(item),
                            at,
                            transform.translation,
                            facing.dir,
                        );
                    }
                })
            }
            PocketRequest::Store { owner, slot, item }
                if inventory.pockets.pockets.get(slot) == Some(&item)
                    && can_store(item, inventory.pockets.item_cond(slot)) =>
            {
                if let Some(actor) = room.runtime.actors.get_mut(owner).and_then(Option::as_mut) {
                    if let Some(empty) = actor
                        .items
                        .iter_mut()
                        .take(3 - actor.layer)
                        .find(|i| **i == 0)
                    {
                        *empty = item;
                        inventory.pockets.pockets[slot] = 0;
                        inventory.pockets.set_item_cond(slot, 0);
                        Ok(())
                    } else {
                        Err("There's no more room in there.")
                    }
                } else {
                    Err("That furniture is no longer there.")
                }
            }
            _ => Err("That item is no longer available."),
        };
        ui.storing = None;
        ui.registering = false;
        room.flow = StorageFlow::Idle;
        if let Err(message) = result {
            notice(&mut dialogue, &mut room, &mut focus, message);
        } else {
            *focus = if room.presentation.busy()
                || room.loose_presentation.busy()
                || room.storage.busy()
            {
                InputFocus::Furniture
            } else {
                InputFocus::World
            };
            crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
        }
        return;
    }
    if let StorageFlow::Pockets = room.flow {
        if !ui.open {
            ui.storing = None;
            ui.registering = false;
            room.flow = StorageFlow::Idle;
            *focus = if room.storage.busy() {
                InputFocus::Furniture
            } else {
                InputFocus::World
            };
        }
        return;
    }
    if let StorageFlow::Opening(owner) = room.flow {
        if room.storage.ready() {
            storage_choices(&mut dialogue, &mut room, &mut focus, owner);
        }
        return;
    }
    if !matches!(room.flow, StorageFlow::Idle) {
        let Some(choice) = dialogue.external_result.take() else {
            return;
        };
        let flow = std::mem::take(&mut room.flow);
        *focus = InputFocus::World;
        match flow {
            StorageFlow::Music(flow) => music::answer(
                &mut room,
                &mut inventory,
                &mut ui,
                &mut dialogue,
                &mut focus,
                flow,
                choice,
            ),
            StorageFlow::Choose(owner) if choice == 0 => {
                let full = room
                    .runtime
                    .actors
                    .get(owner)
                    .and_then(Option::as_ref)
                    .is_none_or(|a| a.items.iter().take(3 - a.layer).all(|i| *i != 0));
                if full {
                    notice(
                        &mut dialogue,
                        &mut room,
                        &mut focus,
                        "There's no more room in there.",
                    );
                    return;
                }
                ui.open = true;
                ui.selling = false;
                ui.storing = Some(owner);
                ui.registering = false;
                ui.menu = None;
                ui.grabbed = None;
                ui.mail_slot = None;
                room.flow = StorageFlow::Pockets;
                *focus = InputFocus::Pockets;
            }
            StorageFlow::Choose(owner) if choice == 1 => {
                let slots: Vec<usize> = room
                    .runtime
                    .actors
                    .get(owner)
                    .and_then(Option::as_ref)
                    .map(|a| {
                        a.items
                            .iter()
                            .enumerate()
                            .filter_map(|(i, item)| (*item != 0).then_some(i))
                            .collect()
                    })
                    .unwrap_or_default();
                if slots.is_empty() {
                    notice(
                        &mut dialogue,
                        &mut room,
                        &mut focus,
                        "There's nothing in there.",
                    );
                    return;
                }
                let mut names: Vec<String> = slots
                    .iter()
                    .map(|&i| {
                        crate::items::name(
                            room.runtime.actors[owner]
                                .as_ref()
                                .map_or(0, |a| a.items[i]),
                        )
                    })
                    .collect();
                names.push("Quit".into());
                dialogue.begin_furniture("What would you like to take out?".into(), Some(names));
                room.flow = StorageFlow::Take { owner, slots };
                *focus = InputFocus::Dialogue;
            }
            StorageFlow::Take { owner, slots } if choice < slots.len() => {
                if let Some(actor) = room.runtime.actors.get_mut(owner).and_then(Option::as_mut) {
                    let item = actor.items[slots[choice]];
                    if item != 0 && inventory.add(item) {
                        actor.items[slots[choice]] = 0;
                    } else {
                        notice(
                            &mut dialogue,
                            &mut room,
                            &mut focus,
                            "Your pockets are full.",
                        );
                    }
                }
            }
            _ => {}
        }
        if room.storage.busy() && matches!(room.flow, StorageFlow::Idle) {
            *focus = InputFocus::Furniture;
        }
        return;
    }
    if focus.is_world() && actions.consume(Action::DebugToys) {
        let mut pockets = inventory.pockets.clone();
        let message = if pockets.put(0x224C, 0).is_some() && pockets.put(0x2254, 0).is_some() {
            inventory.pockets = pockets;
            "Fan and pinwheel added.\nI: select toy > Hold.\nE: fan; walk/run: spin; 3: put away."
        } else {
            "You need two empty pockets."
        };
        notice(&mut dialogue, &mut room, &mut focus, message);
        return;
    }
    if focus.is_world() && actions.consume(Action::DebugMusic) {
        let message = if inventory.add(0x2A23) {
            "K.K. Love Song added.\nFace a music player; E: open.\nChoose Register a song."
        } else {
            "Your pockets are full."
        };
        notice(&mut dialogue, &mut room, &mut focus, message);
        return;
    }
    let grant = if focus.is_world() && actions.consume(Action::DebugDrawers) {
        Some(true)
    } else if focus.is_world() && actions.consume(Action::DebugFurniture) {
        Some(false)
    } else {
        None
    };
    if let Some(drawers) = grant {
        let message = match crate::furniture::playtest_storage(&disc, drawers) {
            Some(item) if inventory.add(item) => {
                if drawers {
                    "Drawers added.\nI: select leaf > Place in room.\nFace its front; E: open."
                } else {
                    "Blue cabinet added.\nI: select it > Place in room.\nFace its front; E: open."
                }
            }
            Some(_) => "Your pockets are full.\nFree a pocket and try again.",
            None => "Storage furniture unavailable.\nCheck the selected disc.",
        };
        notice(&mut dialogue, &mut room, &mut focus, message);
        return;
    }
    if !focus.is_world() || *tool != crate::ecology::EquippedTool::None {
        return;
    }
    if !actions.just_pressed(Action::Interact) && !actions.just_pressed(Action::Back) {
        return;
    }
    // Source pickup range is 50 units on layer 0, 56 on layer 1.
    if actions.just_pressed(Action::Back) {
        if let Some(piece) = loose::target(&room, transform.translation, facing.dir) {
            actions.consume(Action::Back);
            if inventory.add(piece.item) {
                let at = loose::position(&room, piece);
                let yaw = if piece.layer == 1
                    && crate::shop_display::goods_rotates_on_surface(&disc, piece.item)
                {
                    room.loose_angles[piece.z as usize][piece.x as usize] as f32
                        * std::f32::consts::FRAC_PI_2
                } else {
                    0.0
                };
                room.layers[piece.layer].set(piece.x, piece.z, 0);
                if piece.layer == 1 {
                    room.loose_angles[piece.z as usize][piece.x as usize] = 0;
                }
                room.dirty = true;
                room.loose_presentation.pickup(piece, at, yaw);
                if room.loose_presentation.busy() {
                    *focus = InputFocus::Furniture;
                }
            } else {
                notice(
                    &mut dialogue,
                    &mut room,
                    &mut focus,
                    "Your pockets are full.",
                );
            }
            return;
        }
    }
    let target = room
        .runtime
        .actors
        .iter()
        .enumerate()
        .filter_map(|(slot, a)| {
            let a = a.as_ref()?;
            let distance = a
                .shape_type
                .unit_offsets()
                .iter()
                .take(a.shape_type.unit_count())
                .filter_map(|&(dx, dz)| {
                    let center = corner()
                        + Vec3::new(
                            (a.cell_x + dx) as f32 + 0.5,
                            0.0,
                            (a.cell_z + dz) as f32 + 0.5,
                        );
                    let delta = (center - transform.translation).with_y(0.0);
                    let forward = delta.dot(facing.dir);
                    (forward > 0.0
                        && forward <= 1.4
                        && (delta - facing.dir * forward).length() <= 0.55
                        && delta.length() <= if a.layer == 0 { 1.25 } else { 1.4 })
                    .then_some(delta.length_squared())
                })
                .min_by(f32::total_cmp)?;
            Some((slot, a.layer, distance))
        })
        .min_by(|a, b| b.1.cmp(&a.1).then_with(|| a.2.total_cmp(&b.2)))
        .map(|t| t.0);
    let Some(slot) = target else { return };
    if actions.consume(Action::Back) {
        let old = room.runtime.actors[slot].as_ref().cloned();
        if let Err(message) = room.pickup(slot, &mut inventory) {
            notice(&mut dialogue, &mut room, &mut focus, message);
        } else if let Some(old) = old {
            let at = presentation::actor_center(&room, &old);
            room.presentation
                .pickup(slot, ftr::ftr_idx_2_item_no(old.name, Direction::South), at);
            *focus = InputFocus::Furniture;
        }
    } else if music::is_player(&room, slot) && actions.consume(Action::Interact) {
        if let Some(actor) = room.runtime.actors[slot].as_ref() {
            let front = Quat::from_rotation_y(actor.angle_y.to_radians()) * Vec3::Z;
            let at = presentation::actor_center(&room, actor);
            if facing.dir.dot(front) > -0.9 || (transform.translation - at).dot(front) <= 0.0 {
                return;
            }
        }
        music::begin(&mut room, &mut dialogue, &mut focus, slot);
    } else if room.runtime.actors[slot]
        .as_ref()
        .and_then(|a| room.profiles.get(&a.name))
        .is_some_and(|p| {
            p.interaction_type
                & (ftr::interaction::STORAGE_DRAWERS
                    | ftr::interaction::STORAGE_WARDROBE
                    | ftr::interaction::STORAGE_CLOSET)
                != 0
        })
        && actions.consume(Action::Interact)
    {
        if let Some((_, rig)) = rigs.iter().find(|(piece, _)| piece.slot == slot) {
            let actor = room.runtime.actors[slot].as_ref().cloned();
            if let Some(actor) = actor {
                let front = Quat::from_rotation_y(actor.angle_y.to_radians()) * Vec3::Z;
                let at = presentation::actor_center(&room, &actor);
                // Original storage requires front contact. Side/back
                // attempts leave storage and inventory untouched.
                if facing.dir.dot(front) > -0.9 || (transform.translation - at).dot(front) <= 0.0 {
                    return;
                }
                let interaction = room
                    .profiles
                    .get(&actor.name)
                    .map_or(0, |p| p.interaction_type);
                let kind = if interaction & ftr::interaction::STORAGE_DRAWERS != 0 {
                    if !ftr::drawer_front_clear(
                        actor.shape_type,
                        (actor.cell_x, actor.cell_z),
                        direction((actor.angle_y / 90.0).round() as u8),
                        &room.layers[0],
                    ) {
                        notice(
                            &mut dialogue,
                            &mut room,
                            &mut focus,
                            "Something is blocking\nthe drawers.",
                        );
                        return;
                    }
                    storage::Kind::Drawers
                } else if interaction & ftr::interaction::STORAGE_WARDROBE != 0 {
                    storage::Kind::Wardrobe
                } else {
                    storage::Kind::Closet
                };
                if room.storage.begin(slot, kind, rig.end, at, -front) {
                    room.flow = StorageFlow::Opening(slot);
                    *focus = InputFocus::Furniture;
                    return;
                }
            }
        }
        storage_choices(&mut dialogue, &mut room, &mut focus, slot);
    }
}

#[derive(Component, Clone, Copy, PartialEq)]
struct RoomPiece {
    slot: usize,
    item: u16,
    x: i32,
    z: i32,
    layer: usize,
}

#[allow(clippy::too_many_arguments)]
fn refresh(
    mut commands: Commands,
    mut room: ResMut<PlayerRoom>,
    disc: Option<Res<GameDisc>>,
    mut existing: Query<(Entity, &mut RoomPiece, &mut Transform)>,
    loose_pieces: Query<Entity, With<loose::Piece>>,
    mut obstacles: ResMut<InteriorObstacles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<crate::n64_material::N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !room.dirty {
        return;
    }
    room.dirty = false;
    let Some(disc) = disc else { return };
    loose::refresh(
        &mut commands,
        &room,
        &disc,
        &loose_pieces,
        (&mut meshes, &mut materials, &mut images),
    );
    let wanted: Vec<RoomPiece> = room
        .runtime
        .actors
        .iter()
        .enumerate()
        .filter_map(|(slot, a)| {
            let a = a.as_ref()?;
            Some(RoomPiece {
                slot,
                item: ftr::ftr_idx_2_item_no(a.name, direction((a.angle_y / 90.0).round() as u8)),
                x: a.cell_x,
                z: a.cell_z,
                layer: a.layer,
            })
        })
        .collect();
    for (entity, mut piece, mut transform) in &mut existing {
        if let Some(next) = wanted.iter().find(|p| {
            p.slot == piece.slot && p.item >> 2 == piece.item >> 2 && p.layer == piece.layer
        }) {
            *piece = *next;
            transform.translation.y = if piece.layer == 1 {
                crate::furniture::top_height(
                    &disc,
                    room.runtime.occupancy[0][piece.z as usize][piece.x as usize]
                        .and_then(|id| room.runtime.actors[id as usize].as_ref())
                        .map_or(0, |a| ftr::ftr_idx_2_item_no(a.name, Direction::South)),
                )
            } else {
                0.0
            };
        } else {
            if room.presentation.picking_up(piece.slot) {
                let long = crate::furniture::furniture_index(piece.item)
                    .and_then(|idx| room.profiles.get(&(idx as u16)))
                    .is_some_and(|p| p.shape.unit_count() == 2);
                commands
                    .entity(entity)
                    .remove::<RoomPiece>()
                    .insert(presentation::RetiringPiece {
                        base: *transform,
                        long,
                    });
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
    for piece in wanted {
        if existing.iter().any(|(_, p, _)| *p == piece) {
            continue;
        }
        let lift = if piece.layer == 1 {
            room.runtime.occupancy[0][piece.z as usize][piece.x as usize]
                .and_then(|id| room.runtime.actors[id as usize].as_ref())
                .and_then(|a| room.profiles.get(&a.name))
                .map_or(0.0, |p| p.height / 40.0)
        } else {
            0.0
        };
        if let Some(entity) = crate::furniture::spawn_furniture(
            &mut commands,
            &disc,
            piece.item,
            (piece.x, piece.z),
            corner(),
            lift,
            &mut meshes,
            &mut materials,
            &mut images,
        ) {
            commands.entity(entity).insert(piece);
        }
    }
    let mut collision = Vec::new();
    for a in room.runtime.actors.iter().flatten() {
        if a.layer != 0
            || room
                .profiles
                .get(&a.name)
                .is_some_and(|p| p.interaction_type & ftr::interaction::NO_COLLISION != 0)
        {
            continue;
        }
        for &(dx, dz) in a
            .shape_type
            .unit_offsets()
            .iter()
            .take(a.shape_type.unit_count())
        {
            collision.push(crate::interior::RoomFurniture {
                centre: Vec3::new(
                    (a.cell_x + dx) as f32 - 2.5,
                    0.0,
                    (a.cell_z + dz) as f32 - 2.5,
                ),
                half: Vec2::splat(0.45),
            });
        }
    }
    obstacles.replace_player(&collision);
}

pub(crate) struct PlayerRoomPlugin;
impl Plugin for PlayerRoomPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerRoom>()
            .add_plugins(music_grid::MusicGridPlugin)
            .add_systems(Startup, setup.after(crate::house::build_interior))
            .add_systems(
                Update,
                storage::advance
                    .after(crate::save::LoadSet)
                    .before(movement::advance)
                    .in_set(GameplaySet),
            )
            .add_systems(Update, storage::render.after(refresh).in_set(GameplaySet))
            .add_systems(
                Update,
                loose_presentation::advance
                    .after(crate::save::LoadSet)
                    .before(movement::advance)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                loose_presentation::render
                    .after(movement::animate_roots)
                    .after(crate::character::CharacterAnimationSet)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                presentation::advance
                    .after(crate::save::LoadSet)
                    .before(movement::advance)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                presentation::render
                    .after(movement::animate_roots)
                    .after(crate::character::CharacterAnimationSet)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                movement::advance
                    .before(InteractSet::Doors)
                    .before(crate::locomotion::MovementSet)
                    .before(crate::character::CharacterAnimationSet)
                    .after(crate::save::LoadSet)
                    .in_set(GameplaySet),
            )
            .add_systems(
                Update,
                movement::animate_roots.after(refresh).in_set(GameplaySet),
            )
            .add_systems(
                Update,
                interact
                    .in_set(InteractSet::Objects)
                    .in_set(GameplaySet)
                    .before(crate::character::CharacterAnimationSet)
                    .after(crate::save::LoadSet),
            )
            .add_systems(
                Update,
                refresh
                    .after(InteractSet::Ui)
                    .after(crate::save::LoadSet)
                    .after(interact)
                    .in_set(GameplaySet),
            );
    }
}
