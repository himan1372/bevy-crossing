//! Phase 2: E-key interaction — talk to villagers, pick up field items.
//!
//! The spatial query (nearest interactable in range + facing cone) is
//! Bevy-side game code. The logic crate's `interaction` module models
//! villager mood/patience ("feels"), not spatial queries, so it isn't
//! called here; talk-throttle mechanics are a later-phase concern.

use bevy::ecs::message::{Message, MessageWriter};
use bevy::prelude::*;

use crate::house::HouseState;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::items::{self, ItemId};
use crate::npc::Villager;
use crate::player::Player;
use crate::states::GameplaySet;
use rustimal_logic::inventory::{Inventory, EMPTY_NO};

/// The player's wallet and pockets. Pockets are the decomp-verified
/// 15-slot model from `rustimal_logic::inventory` (item IDs in
/// [`crate::items`]); bells are kept separately, like the retail wallet.
#[derive(Resource, Default)]
pub struct PlayerInventory {
    pub bells: u32,
    pub pockets: Inventory,
}

impl PlayerInventory {
    /// Put an item in the first free pocket. False when pockets are full.
    pub fn add(&mut self, item: ItemId) -> bool {
        self.pockets.put(item, 0).is_some()
    }

    /// How many pockets hold `item`.
    pub fn count(&self, item: ItemId) -> u32 {
        self.pockets.count_item(item)
    }

    /// How many pockets hold an item matching `pred`.
    pub fn count_where(&self, pred: impl Fn(ItemId) -> bool) -> u32 {
        self.pockets
            .pockets
            .iter()
            .filter(|&&it| it != EMPTY_NO && pred(it))
            .count() as u32
    }

    /// Remove one `item`. False if none is carried.
    pub fn remove(&mut self, item: ItemId) -> bool {
        self.remove_first_where(|it| it == item).is_some()
    }

    /// Remove the first item matching `pred`, returning it.
    pub fn remove_first_where(&mut self, pred: impl Fn(ItemId) -> bool) -> Option<ItemId> {
        let slot = self
            .pockets
            .pockets
            .iter()
            .position(|&it| it != EMPTY_NO && pred(it))?;
        let item = self.pockets.pockets[slot];
        self.pockets.pockets[slot] = EMPTY_NO;
        self.pockets.set_item_cond(slot, 0);
        Some(item)
    }

    /// Filled pockets.
    pub fn used(&self) -> usize {
        self.pockets
            .pockets
            .iter()
            .filter(|&&it| it != EMPTY_NO)
            .count()
    }
}

/// Fired when the player talks to a villager.
#[derive(Message)]
pub struct TalkEvent {
    pub villager: Entity,
}

/// Player's facing direction on the ground plane. Set by
/// [`crate::locomotion::step`] while moving, and by doors on teleport.
#[derive(Component)]
pub struct Facing {
    pub dir: Vec3,
}

impl Default for Facing {
    fn default() -> Self {
        Self {
            dir: Vec3::new(0.0, 0.0, -1.0),
        }
    }
}

/// A door panel in a wall running along x: centred on `center_x`,
/// `half_w` wide each way, with `face_z` the wall face the player stands at
/// and `into_z` (+1 or -1) the z direction from the player into the door.
pub struct DoorPanel {
    pub center_x: f32,
    pub half_w: f32,
    pub face_z: f32,
    pub into_z: f32,
}

/// How far behind the wall face the door is measured to, so a player pressed
/// right against the face still has a clear direction to it.
const DOOR_DEPTH: f32 = 0.1;

impl DoorPanel {
    /// Facing ray must reach the finite panel from outside the wall.
    /// Used for player-home entry next to other interactive objects:
    /// a broad facing cone can otherwise consume E aimed at a mailbox.
    pub fn aimed_at(&self, pos: Vec3, facing_dir: Vec3, range: f32) -> bool {
        let facing = Vec2::new(facing_dir.x, facing_dir.z).normalize_or_zero();
        if facing.y * self.into_z <= 0.001 || (pos.z - self.face_z) * self.into_z > 0.0 {
            return false;
        }
        let distance = (self.face_z - pos.z) / facing.y;
        distance >= 0.0
            && distance <= range
            && (pos.x + facing.x * distance - self.center_x).abs() <= self.half_w
    }

    /// True when a player at `pos` is within `range` of the panel and facing
    /// it (cosine at least `min_dot`). Measured on the ground plane to the
    /// nearest point of the panel, so standing anywhere along the door works
    /// and turning arcs (see `locomotion`) don't leave you "beside" it.
    pub fn faced_by(&self, pos: Vec3, facing_dir: Vec3, range: f32, min_dot: f32) -> bool {
        let x = pos
            .x
            .clamp(self.center_x - self.half_w, self.center_x + self.half_w);
        let z = self.face_z + self.into_z * DOOR_DEPTH;
        let to_door = Vec3::new(x - pos.x, 0.0, z - pos.z);
        let dist = to_door.length();
        if dist > range || dist < 0.001 {
            return false;
        }
        let facing = Vec3::new(facing_dir.x, 0.0, facing_dir.z).normalize_or_zero();
        facing.dot(to_door / dist) >= min_dot
    }
}

const INTERACT_RANGE_VILLAGER: f32 = 4.0;
/// How far ahead of the player the unit they pick up from lies.
const PICK_UP_AHEAD: f32 = 0.6;
/// Cosine threshold for the facing cone (~78 degrees).
const FACING_DOT: f32 = 0.2;

/// E key: talk to the nearest faced villager, else pick up the field item
/// underfoot or just ahead.
///
/// When a fishing rod or bug net is equipped (see `ecology::EquippedTool`),
/// this system stands down and lets the ecology plugin handle the E press.
/// Last in the Interact order ([`InteractSet::Talk`]): doors, the mailbox and
/// tools get the press first.
pub fn interact(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<HouseState>>,
    shop_loc: Res<State<crate::shop::ShopLocation>>,
    tool: Res<crate::ecology::EquippedTool>,
    mut talk_events: MessageWriter<TalkEvent>,
    player_query: Query<(&Transform, &Facing), With<Player>>,
    // Villagers at home can't be talked to.
    villagers: Query<
        (Entity, &Villager, &Transform),
        (Without<Player>, Without<crate::npc::Indoors>),
    >,
    field: Option<ResMut<crate::field_items::FieldItems>>,
    mut inventory: ResMut<PlayerInventory>,
) {
    // Inside the house or shop, E belongs to the doors.
    if *state != HouseState::Town || *shop_loc != crate::shop::ShopLocation::Town {
        return;
    }
    if !actions.just_pressed(Action::Interact) || !focus.is_world() {
        return;
    }
    // A held tool claims the E press — the ecology plugin handles it.
    if *tool != crate::ecology::EquippedTool::None {
        return;
    }
    let Ok((player_transform, facing)) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    // 1. Nearest villager in range inside the facing cone.
    let mut best: Option<(Entity, f32)> = None;
    for (entity, _villager, v_transform) in villagers.iter() {
        let to_v = v_transform.translation - player_pos;
        let dist = to_v.length();
        if dist > INTERACT_RANGE_VILLAGER || dist < 0.001 {
            continue;
        }
        if facing.dir.dot(to_v / dist) < FACING_DOT {
            continue;
        }
        if best.is_none_or(|(_, d)| dist < d) {
            best = Some((entity, dist));
        }
    }
    if let Some((entity, _)) = best {
        if actions.consume(Action::Interact) {
            talk_events.write(TalkEvent { villager: entity });
        }
        return;
    }

    // 2. A field item underfoot, else on the unit ahead.
    let Some(mut field) = field else { return };
    let ahead = player_pos + facing.dir * PICK_UP_AHEAD;
    let Some((acre, cell, item)) = [player_pos, ahead]
        .into_iter()
        .filter_map(|p| field.at(p))
        .find(|&(_, _, item)| crate::field_items::can_pick_up(item))
    else {
        return;
    };
    if !actions.consume(Action::Interact) {
        return;
    }
    if !inventory.add(item) {
        // Leave it on the ground; the HUD shows the pockets are full.
        println!("Pockets full; left {}", items::name(item));
        return;
    }
    field.acres[acre][cell] = rustimal_logic::field_items::EMPTY_NO;
    crate::sound::se_at(crate::sound::se_id::ITEM_GET, player_pos);
    println!(
        "Picked up {} ({}/15 pockets)",
        items::name(item),
        inventory.used()
    );
}

/// F7 (debug): talk to the nearest villager outdoors, however far away,
/// so dialogue can be checked without walking up to someone.
fn debug_talk(
    actions: Res<Actions>,
    focus: Res<InputFocus>,
    mut talk_events: MessageWriter<TalkEvent>,
    player: Query<&Transform, With<Player>>,
    villagers: Query<(Entity, &Transform), (With<Villager>, Without<crate::npc::Indoors>)>,
) {
    if !actions.just_pressed(Action::DebugTalk) || !focus.is_world() {
        return;
    }
    let Ok(p) = player.single() else { return };
    let nearest = villagers.iter().min_by(|a, b| {
        let da = a.1.translation.distance_squared(p.translation);
        let db = b.1.translation.distance_squared(p.translation);
        da.total_cmp(&db)
    });
    if let Some((villager, _)) = nearest {
        talk_events.write(TalkEvent { villager });
    }
}

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerInventory::default())
            .add_message::<TalkEvent>()
            .add_systems(
                Update,
                (interact, debug_talk)
                    .in_set(InteractSet::Talk)
                    .in_set(GameplaySet),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An interior door on a south wall at z = 4.8, player inside (north).
    fn interior_door() -> DoorPanel {
        DoorPanel {
            center_x: 0.0,
            half_w: 0.7,
            face_z: 4.8,
            into_z: 1.0,
        }
    }

    #[test]
    fn door_after_turning_arc_still_counts() {
        // Turning 180° to face the door arcs the player ~1.3 to the side.
        assert!(interior_door().faced_by(Vec3::new(-1.3, 1.0, 4.2), Vec3::Z, 2.6, 0.5));
    }

    #[test]
    fn door_flush_against_face_counts() {
        assert!(interior_door().faced_by(Vec3::new(0.0, 1.0, 4.8), Vec3::Z, 2.6, 0.5));
    }

    #[test]
    fn door_rejects_facing_away_or_far() {
        assert!(!interior_door().faced_by(Vec3::new(0.0, 1.0, 4.2), -Vec3::Z, 2.6, 0.5));
        assert!(!interior_door().faced_by(Vec3::new(0.0, 1.0, 0.0), Vec3::Z, 2.6, 0.5));
    }
}
