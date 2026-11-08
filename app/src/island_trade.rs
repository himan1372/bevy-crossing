//! The island resident's furniture trade and her room (`ac_quest_talk_island.c`, `m_npc.c`: `mNpc_*Island*`).
//!
//! Her room is one of the eighteen islanders' own layouts (`l_island_npc_best_fg_id`, in the NPC room bank): real
//! furniture (her *best* pieces) and reserved markers (`RSV_ISLAND_FTR0..15`) for pieces of a size she would take from a
//! visitor. Slot `n` is the n-th piece or marker in reading order (16 at most). A best piece appears once the
//! player has given its twin (`have` bit n); a marker shows the normal piece given for its size (`trade[slot]`: two 1x1,
//! one 1x2, one 2x2).
//!
//! Talking to her (`aQMgr_decide_ftr_msg_no`): the first talk of a visit always checks, later ones check unless the roll
//! sends her to the mail (the other checks need another player's memory and fail here). The checks, with `base[looks]
//! + n` (`0x35E0 0x35FB 0x382F 0x3844 0x3616 0x381A`): 0 a best piece is carried (`0x35E0`: "It's all yours." gives it to
//! her), 1 a normal piece (`0x35E1`: she pays a random 100-3000 Bells for it), 5 she tells what she dreams of, 7 she
//! recalls a piece the player gave, 8 a shirt is carried (`0x35E8`: she wears it). The messages' own orders carry
//! out the trade: `0x35E9` takes the best piece, `0x35EB` the shirt, `0x35EF` the normal piece and pays.
//!
//! Not built: the pieces left by other players and the "present" the resident displays (checks 2, 3, 4, 6), her
//! wearing the shirt, and her mail and player-action talks.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game_disc::GameDisc;
use crate::interaction::PlayerInventory;
use crate::items::ItemId;
use crate::messages::MsgContext;

/// `l_island_npc_best_fg_id[islander][0]`.
const BEST: [u16; 18] = [
    0x0250, 0x0346, 0x0372, 0x027C, 0x02EA, 0x0326, 0x0270, 0x01C0, 0x039A, 0x02F0, 0x02C4, 0x0224,
    0x02D4, 0x02A4, 0x038C, 0x01FE, 0x032E, 0x01E6,
];
/// `msg_table` of `aQMgr_decide_ftr_msg_no`, by looks.
const MSG: [u32; 6] = [0x35E0, 0x35FB, 0x382F, 0x3844, 0x3616, 0x381A];
const RSV0: u16 = 0xFEB3;
const RSV15: u16 = 0xFEC2;
/// `mNpc_ISLAND_FTR_NUM`.
const SLOTS: usize = 16;
const CLOTH: std::ops::Range<u16> = 0x2400..0x2500;

/// What the island keeps of the trades (`mNpc_Island_Ftr_c` and the memory's `have_bitfield`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct IslandRoom {
    /// `mISL_PLAYER_ACTION_*`: what the player did on the island this trip (`Island_c` memory `check`); the resident
    /// remarks on one of them. Cleared when a new trip starts.
    #[serde(default)]
    pub actions: u32,
    /// Bit n: slot n has been given to her.
    pub have: u16,
    /// The normal pieces given: two 1x1, one 1x2, one 2x2.
    pub trade: [u16; 4],
}

/// `mISL_PLAYER_ACTION_TRADE_CLOTH`: the player gave her a shirt.
pub(crate) const ACTION_TRADE_CLOTH: u32 = 1 << 30;

/// This visit's chat state: the first talk always checks for a trade (`mNpc_GetIslandCheckFtrMsg`).
#[derive(Resource, Default)]
pub(crate) struct IslandChat {
    pub checked: bool,
}

/// What a trade message needs: the pieces named in its text and what she pays.
#[derive(Clone, Debug, Default)]
pub(crate) struct Deal {
    /// `trade_items[0]`: what the player gives.
    pub give: ItemId,
    /// `mMsg_ITEM_STR0..3`.
    pub names: [ItemId; 4],
    pub pay: u32,
}

impl Deal {
    /// Fills the message's blanks.
    pub(crate) fn apply(&self, ctx: &mut MsgContext) {
        ctx.items = vec![String::new(); 5];
        for (i, &item) in self.names.iter().enumerate() {
            ctx.items[i] = crate::items::name(item);
        }
        if ctx.free.len() <= 19 {
            ctx.free.resize(20, String::new());
        }
        ctx.free[19] = self.pay.to_string();
    }
}

/// The islander's record in the NPC room bank.
fn record(disc: &GameDisc, ordinal: usize) -> Option<[u16; 256]> {
    let id = *BEST.get(ordinal)?;
    rustimal_logic::parse_fg_records(&disc.npc_room_layouts)
        .ok()?
        .iter()
        .find(|r| r.fg_id == id)
        .map(|r| r.items)
}

fn is_ftr(item: ItemId) -> bool {
    crate::furniture::furniture_index(item).is_some()
}

/// `aMR_CorrespondFurniture`: the same piece whatever its turn.
fn same(a: ItemId, b: ItemId) -> bool {
    is_ftr(a) && crate::furniture::furniture_index(a) == crate::furniture::furniture_index(b)
}

/// The cells with a piece or a marker, in reading order, first sixteen.
fn slots(rec: &[u16; 256]) -> Vec<(usize, ItemId)> {
    rec.iter()
        .enumerate()
        .filter(|(_, &i)| is_ftr(i) || (RSV0..=RSV15).contains(&i))
        .map(|(c, &i)| (c, i))
        .take(SLOTS)
        .collect()
}

/// `aMR_GetFurnitureUnit` from the piece's footprint: 0 one cell, 1 two, 2 four.
fn unit(disc: &GameDisc, item: ItemId) -> Option<usize> {
    match crate::furniture::footprint(disc, item, (0, 0)).len() {
        1 => Some(0),
        2 => Some(1),
        4 => Some(2),
        _ => None,
    }
}

/// `aMR_FurnitureFg_to_FurnitureFgWithDirect`.
fn with_direction(item: ItemId, j: u16) -> ItemId {
    (item & !3) | (j & 3)
}

impl IslandRoom {
    /// `mNpc_CheckFtrIsIslandBestFtr`.
    fn is_best(rec: &[u16; 256], item: ItemId) -> bool {
        rec.iter().any(|&r| same(item, r))
    }

    /// `mNpc_GetIslandFtrIdx` for a best piece.
    fn best_idx(rec: &[u16; 256], item: ItemId) -> Option<usize> {
        slots(rec)
            .iter()
            .position(|&(_, r)| is_ftr(r) && same(item, r))
    }

    /// `mNpc_CheckIslandNpcRoomFtrItemNo_keep`: she can take this piece.
    fn keep(&self, disc: &GameDisc, rec: &[u16; 256], item: ItemId) -> bool {
        if !is_ftr(item) {
            return false;
        }
        if Self::is_best(rec, item) {
            return Self::best_idx(rec, item).is_some_and(|n| self.have & (1 << n) == 0);
        }
        if self.trade.iter().any(|&t| {
            t != 0
                && crate::furniture::furniture_index(t) == crate::furniture::furniture_index(item)
        }) {
            return false;
        }
        let rsv = match unit(disc, item) {
            Some(0) if self.trade[0] == 0 => Some(RSV0),
            Some(0) if self.trade[1] == 0 => Some(RSV0 + 4),
            Some(u) if u > 0 && self.trade[u + 1] == 0 => Some(RSV0 + (u as u16 + 1) * 4),
            _ => None,
        };
        rsv.is_some_and(|base| rec.iter().any(|&c| (base..base + 4).contains(&c)))
    }

    /// `mNpc_SetIslandGetFtr`.
    pub(crate) fn get(&mut self, disc: &GameDisc, ordinal: usize, item: ItemId) {
        let Some(rec) = record(disc, ordinal) else {
            return;
        };
        if !is_ftr(item) {
            return;
        }
        if Self::is_best(&rec, item) {
            if let Some(n) = Self::best_idx(&rec, item) {
                self.have |= 1 << n;
            }
            return;
        }
        let Some(u) = unit(disc, item) else { return };
        let (v0, v1) = if u == 0 {
            (RSV0, Some(RSV0 + 4))
        } else {
            (RSV0 + (u as u16 + 1) * 4, None)
        };
        for (n, (_, r)) in slots(&rec).into_iter().enumerate() {
            if !(RSV0..=RSV15).contains(&r) || self.have & (1 << n) != 0 {
                continue;
            }
            let hit = (0..4u16).find_map(|j| {
                if r == v0 + j {
                    Some((0usize, j))
                } else if v1.is_some_and(|v1| r == v1 + j) {
                    Some((1, j))
                } else {
                    None
                }
            });
            if let Some((variant, j)) = hit {
                self.have |= 1 << n;
                let slot = if variant == 0 {
                    (v0 - RSV0) / 4
                } else {
                    (v1.unwrap_or(v0) - RSV0) / 4
                };
                self.trade[usize::from(slot).min(3)] = with_direction(item, j);
                return;
            }
        }
    }

    /// `mNpc_GetRandomBestFtr`: a best piece she does not have yet.
    fn random_best(&self, rec: &[u16; 256], rng: &mut dyn FnMut(u32) -> u32) -> Option<ItemId> {
        let open: Vec<ItemId> = slots(rec)
            .iter()
            .enumerate()
            .filter(|(n, (_, r))| is_ftr(*r) && self.have & (1 << n) == 0)
            .map(|(_, &(_, r))| with_direction(r, 0))
            .collect();
        (!open.is_empty()).then(|| open[rng(open.len() as u32) as usize])
    }

    /// `mNpc_GetPlayerBestFtr`: a best piece the player gave.
    fn player_best(&self, rec: &[u16; 256], rng: &mut dyn FnMut(u32) -> u32) -> Option<ItemId> {
        let given: Vec<ItemId> = slots(rec)
            .iter()
            .enumerate()
            .filter(|(n, (_, r))| is_ftr(*r) && self.have & (1 << n) != 0)
            .map(|(_, &(_, r))| with_direction(r, 0))
            .collect();
        (!given.is_empty()).then(|| given[rng(given.len() as u32) as usize])
    }

    /// `mNpc_GetPlayerFtr`: any piece (best or normal) the player gave.
    fn player_ftr(&self, rec: &[u16; 256], rng: &mut dyn FnMut(u32) -> u32) -> Option<ItemId> {
        let mut given: Vec<ItemId> = Vec::new();
        for (n, (_, r)) in slots(rec).into_iter().enumerate() {
            if self.have & (1 << n) == 0 {
                continue;
            }
            if is_ftr(r) {
                given.push(with_direction(r, 0));
            } else if (RSV0..=RSV15).contains(&r) {
                let t = self.trade[usize::from((r - RSV0) / 4).min(3)];
                if t != 0 {
                    given.push(with_direction(t, 0));
                }
            }
        }
        (!given.is_empty()).then(|| given[rng(given.len() as u32) as usize])
    }

    /// The pieces her room shows (`mNpc_ChangeIslandRoom`): (cell, piece).
    pub(crate) fn shown(&self, disc: &GameDisc, ordinal: usize) -> Vec<(usize, ItemId)> {
        let Some(rec) = record(disc, ordinal) else {
            return Vec::new();
        };
        let mut out: Vec<(usize, ItemId)> = Vec::new();
        for (n, (cell, r)) in slots(&rec).into_iter().enumerate() {
            if self.have & (1 << n) == 0 {
                continue;
            }
            if is_ftr(r) {
                out.push((cell, r));
            } else if (RSV0..=RSV15).contains(&r) {
                let t = self.trade[usize::from((r - RSV0) / 4).min(3)];
                if t != 0 {
                    out.push((cell, t));
                }
            }
        }
        out
    }
}

/// The pocket pieces that pass a test, as (pocket item) in order.
fn carried(inventory: &PlayerInventory, pass: &dyn Fn(ItemId) -> bool) -> Vec<ItemId> {
    inventory
        .pockets
        .pockets
        .iter()
        .copied()
        .filter(|&it| it != 0 && pass(it))
        .collect()
}

/// `aQMgr_decide_ftr_msg_no` for the one player: the message and what it needs, or none.
pub(crate) fn decide(
    disc: &GameDisc,
    ordinal: usize,
    looks: usize,
    room: &IslandRoom,
    inventory: &PlayerInventory,
    chat: &mut IslandChat,
    rng: &mut dyn FnMut(u32) -> u32,
) -> Option<(u32, Deal)> {
    let rec = record(disc, ordinal)?;
    let best = carried(inventory, &|it| {
        IslandRoom::is_best(&rec, it) && room.keep(disc, &rec, it)
    });
    let normal = carried(inventory, &|it| {
        is_ftr(it) && !IslandRoom::is_best(&rec, it) && room.keep(disc, &rec, it)
    });
    let cloth = carried(inventory, &|it| CLOTH.contains(&it));
    let pick = |list: &[ItemId], rng: &mut dyn FnMut(u32) -> u32| {
        (!list.is_empty()).then(|| list[rng(list.len() as u32) as usize])
    };
    // The checks that can pass with one player.
    let check = |n: u32, rng: &mut dyn FnMut(u32) -> u32| -> Option<Deal> {
        let mut deal = Deal {
            pay: 100 + (rng(2900) / 10) * 10,
            ..Deal::default()
        };
        match n {
            0 => deal.give = pick(&best, rng)?,
            1 => deal.give = pick(&normal, rng)?,
            5 => {
                deal.names[2] = room.random_best(&rec, rng)?;
            }
            7 => {
                deal.names[2] = room.player_best(&rec, rng)?;
            }
            8 => deal.give = pick(&cloth, rng)?,
            _ => return None,
        }
        // `aQMgr_order_decide_trade_18..21`: the names the text can use.
        if deal.give != 0 {
            deal.names[0] = deal.give;
        }
        if n != 8 && deal.names[2] == 0 {
            deal.names[2] = room.random_best(&rec, rng).unwrap_or(0);
        }
        deal.names[3] = room.player_ftr(&rec, rng).unwrap_or(0);
        Some(deal)
    };
    let mut n = if chat.checked { rng(5) } else { 0 };
    chat.checked = true;
    let mut found = check(n, rng);
    if found.is_none() {
        n = 5 + rng(4);
        found = check(n, rng);
        if found.is_some() && n == 5 {
            if let Some(d) = check(0, rng) {
                n = 0;
                found = Some(d);
            }
        }
    }
    found.map(|d| (MSG[looks.min(5)] + n, d))
}

/// `aQMgr_decide_player_msg_no`: the remark on one of the player's actions this trip (the base by looks plus the
/// action's bit), trying three actions from a random one.
pub(crate) fn player_msg(
    room: &IslandRoom,
    looks: usize,
    rng: &mut dyn FnMut(u32) -> u32,
) -> Option<u32> {
    const BASE: [u32; 6] = [0x3937, 0x3974, 0x37BA, 0x378A, 0x39B1, 0x37EA];
    let mut action = rng(32);
    for _ in 0..3 {
        if room.actions & (1 << action) != 0 {
            return Some(BASE[looks.min(5)] + action);
        }
        action = (action + 1) % 32;
    }
    None
}

/// A new trip clears what the player did last time (`mISL_ClearNowPlayerAction` when the boat leaves).
fn clear_actions(
    chat: Res<IslandChat>,
    mut identity: ResMut<crate::save::PlayerIdentity>,
    mut was_checked: Local<bool>,
) {
    // `kappn.rs` re-arms the first-talk check at a trip's start.
    if *was_checked && !chat.checked {
        identity.island_room.actions = 0;
    }
    *was_checked = chat.checked;
}

/// A message's trade orders (`SET_DEMO_ORDER` quest kind, index 3 = `aQMgr_order_trade`, then the value): carries
/// them out as the message is shown.
pub(crate) fn orders(
    deal: &Deal,
    pages: &[String],
    disc: &GameDisc,
    ordinal: usize,
    room: &mut IslandRoom,
    inventory: &mut PlayerInventory,
) {
    use crate::messages::markup::{parse, Item};
    for page in pages {
        for item in parse(page) {
            let Item::Order(4, 3, value) = item else {
                continue;
            };
            match value {
                // `aQMgr_order_move_trade_23` / `_13`: the player gives the piece (and for 13 she pays).
                23 | 13 | 24 => {
                    if deal.give != 0 && inventory.remove(deal.give) {
                        if value != 24 {
                            room.get(disc, ordinal, deal.give);
                        } else {
                            // `aQMgr_order_move_trade_24`: a shirt changes hands.
                            room.actions |= ACTION_TRADE_CLOTH;
                        }
                        info!("The island resident takes {:#06x}", deal.give);
                    }
                }
                // `aQMgr_order_move_trade_22`: she pays.
                22 => {
                    inventory.bells = (inventory.bells + deal.pay).min(99_999);
                    info!("The island resident pays {} Bells", deal.pay);
                }
                _ => {}
            }
        }
    }
}

/// Her room's furniture, to respawn when the trades change.
#[derive(Component)]
pub(crate) struct IslandFurniture;

/// Where her room stands (set when the villagers' rooms are built).
#[derive(Resource, Clone, Copy)]
pub(crate) struct IslandRoomSite(pub Vec3);

/// Respawns her room's furniture when the trades change (and once at the start).
#[allow(clippy::too_many_arguments)]
fn refresh_room(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    identity: Res<crate::save::PlayerIdentity>,
    island: Option<Res<crate::npc::IslandNpc>>,
    site: Option<Res<IslandRoomSite>>,
    mut obstacles: ResMut<crate::house::InteriorObstacles>,
    old: Query<Entity, With<IslandFurniture>>,
    mut last: Local<Option<IslandRoom>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<crate::n64_material::N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (Some(disc), Some(island), Some(site)) = (disc, island, site) else {
        return;
    };
    if last.as_ref() == Some(&identity.island_room) {
        return;
    }
    *last = Some(identity.island_room.clone());
    for e in &old {
        commands.entity(e).despawn();
    }
    let origin = site.0;
    let corner = origin - crate::interior::VILLAGER_ROOM_CENTRE.with_y(0.0);
    let mut boxes = Vec::new();
    for (cell, item) in identity.island_room.shown(&disc, island.1) {
        let ut = ((cell % 16) as i32, (cell / 16) as i32);
        if let Some(entity) = crate::furniture::spawn_furniture(
            &mut commands,
            &disc,
            item,
            ut,
            corner,
            0.0,
            &mut meshes,
            &mut materials,
            &mut images,
        ) {
            commands.entity(entity).insert(IslandFurniture);
        }
        for c in crate::furniture::footprint(&disc, item, ut) {
            boxes.push(crate::interior::RoomFurniture {
                centre: corner + Vec3::new(c.0 as f32 + 0.5, 0.0, c.1 as f32 + 0.5) - origin,
                half: Vec2::splat(0.5),
            });
        }
    }
    obstacles.replace_room(origin, &boxes);
    info!("The island resident's room shows {} pieces", boxes.len());
}

pub struct IslandTradePlugin;

impl Plugin for IslandTradePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IslandChat>()
            .add_systems(Update, (refresh_room, clear_actions));
    }
}
