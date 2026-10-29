//! The town's villagers, picked from the disc's villager tables the way a
//! new town picks them (`mNpc_DecideLivingNpcMax`): shuffle every villager
//! (`mNpc_MakeRandTable`), then take starters (`npc_grow_list` =
//! `mNpc_GROW_STARTER`) until there is one of each personality
//! (`npc_looks_table`). Our shuffle is seeded from the town seed, so a
//! town keeps the same villagers; the game saves its list instead.

use disc::RomSource;
use rustimal_logic::qrand::Qrand;

use crate::game_disc::GameDisc;

/// `NPC_NUM`: villagers in the tables.
const NPC_NUM: usize = 236;
/// `mNpc_LOOKS_NUM`: personalities, and so starting villagers.
pub const PERSONALITIES: usize = 6;
/// `mNpc_GROW_STARTER`.
const GROW_STARTER: i8 = 0;
/// Characters per name in `npc_name_str_table.bin`.
const NAME_LEN: usize = 8;

/// A villager from the disc.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RosterEntry {
    pub name: String,
    /// Personality (`mNpc_LOOKS_*`).
    pub looks: u8,
    /// NPC id - `NPC_START`: index into the draw and default tables.
    pub index: u32,
}

/// The disc's villager tables.
pub struct VillagerTables {
    pub names: Vec<String>,
    pub looks: Vec<u8>,
    pub grow: Vec<i8>,
}

impl VillagerTables {
    /// Reads names, personalities and grow permissions.
    pub fn from_disc(disc: &GameDisc) -> Option<Self> {
        if disc.npc_names.is_empty() {
            return None;
        }
        let names = disc.npc_names.chunks(NAME_LEN).map(decode_name).collect();
        let table = |name: &str| {
            let a = disc.catalog.get(name)?;
            disc.rom.slice(RomSource::Rel, a.offset, NPC_NUM as u32)
        };
        Some(Self {
            names,
            looks: table("npc_looks_table")?.to_vec(),
            grow: table("npc_grow_list")?.iter().map(|&b| b as i8).collect(),
        })
    }

    /// The starting villagers for a town seeded with `seed`.
    pub fn starters(&self, seed: u32) -> Vec<RosterEntry> {
        let mut rng = Rng::new(seed.wrapping_mul(0x2545_F491) | 1);
        let order = rand_table(NPC_NUM, NPC_NUM, &mut rng);
        let mut seen = 0u8;
        let mut out = Vec::with_capacity(PERSONALITIES);
        for idx in order {
            if out.len() == PERSONALITIES {
                break;
            }
            if self.grow.get(idx) != Some(&GROW_STARTER) {
                continue;
            }
            let Some(&looks) = self.looks.get(idx) else {
                continue;
            };
            if looks as usize >= PERSONALITIES || seen & (1 << looks) != 0 {
                continue;
            }
            seen |= 1 << looks;
            out.push(RosterEntry {
                name: self.names.get(idx).cloned().unwrap_or_default(),
                looks,
                index: idx as u32,
            });
        }
        out
    }
}

/// `ANIMAL_NUM_MAX`: most villagers a town holds.
pub const RESIDENTS_MAX: usize = 15;
/// `mNpc_GROW_MOVE_IN`.
const GROW_MOVE_IN: i8 = 1;

/// `mNpc_GetLooks2Sex`: normal, peppy and snooty villagers are female.
pub(crate) fn looks_female(looks: u8) -> bool {
    matches!(looks, 0 | 1 | 5)
}

/// Who lives in town and the move-in bookkeeping, kept in the save.
#[derive(bevy::prelude::Resource, Clone, Debug, Default)]
pub struct TownRoster {
    /// In move-in order; resident `i` lives at reserved home spot `i`.
    pub residents: Vec<RosterEntry>,
    /// Villagers who have lived here (`npc_used_tbl`): not picked again
    /// until everyone of a personality has been.
    pub appeared: Vec<u32>,
    /// Villagers the player has talked to (`mNpc_CheckFriendAllAnimal`).
    pub met: Vec<u32>,
    /// Game day of the last move-in check that let someone in.
    pub last_grow_day: Option<i64>,
    /// The home slot of each resident (`animals[i]`, parallel to `residents`): a villager who
    /// leaves keeps the others where they are. Empty means resident `i` is in slot `i`.
    pub slots: Vec<usize>,
    /// `force_remove_date`: the last move-in or removal (`mNpc_RenewRemoveHistory`).
    pub remove_day: Option<i64>,
    /// `remove_animal_idx`: the slot of the villager who is thinking of moving.
    pub leaver: Option<usize>,
}

impl VillagerTables {
    /// The roster entry for table index `idx`.
    pub fn entry(&self, idx: u32) -> Option<RosterEntry> {
        Some(RosterEntry {
            name: self.names.get(idx as usize)?.clone(),
            looks: *self.looks.get(idx as usize)?,
            index: idx,
        })
    }

    /// May join a town by moving in (`mNpc_GROW_STARTER` / `_MOVE_IN`).
    fn can_move_in(&self, idx: usize) -> bool {
        matches!(
            self.grow.get(idx),
            Some(&GROW_STARTER) | Some(&GROW_MOVE_IN)
        )
    }
}

impl TownRoster {
    /// `mNpc_CheckGrow` then `mNpc_Grow`: on a new day, with room in
    /// town, once the player has talked to everyone living here, and with
    /// the field-rank chance, a new villager moves in. Returns them.
    /// `rank` gives the town's field rank (`mFAs_GetFieldRank`) and is only
    /// asked for once the cheaper gates pass.
    /// The home slot of resident `i`.
    pub fn slot(&self, i: usize) -> usize {
        self.slots.get(i).copied().unwrap_or(i)
    }

    /// Every resident's slot, `residents.len()` of them.
    pub fn resident_slots(&self) -> Vec<usize> {
        (0..self.residents.len()).map(|i| self.slot(i)).collect()
    }

    /// Whether table index `npc_index` is the villager thinking of moving.
    pub fn is_leaver(&self, npc_index: u32) -> bool {
        self.leaver.is_some_and(|slot| {
            self.residents
                .iter()
                .position(|r| r.index == npc_index)
                .is_some_and(|i| self.slot(i) == slot)
        })
    }

    /// `mNpc_SetRemoveAnimalNo`: names who is thinking of moving, if nobody is and the town has
    /// more than five villagers. `ignored` is a slot just talked out of it.
    pub fn choose_leaver(&mut self, ignored: Option<usize>, seed: u32, day: i64) {
        if self.leaver.is_some() {
            return;
        }
        let mut rng = Rng::new((seed ^ (day as u32).wrapping_mul(0x27D4_EB2F)) | 1);
        let mut town: Vec<(usize, bool)> = (0..self.residents.len())
            .map(|i| (self.slot(i), self.met.contains(&self.residents[i].index)))
            .collect();
        town.sort_by_key(|&(slot, _)| slot);
        self.leaver = rustimal_logic::force_remove::pick_leaver(&town, ignored, &mut |n| {
            ((rng.next() * n as f32) as u32).min(n - 1)
        });
    }

    /// `aQMgr_order_cancel_remove`: talked out of it; someone else is chosen.
    pub fn cancel_leaver(&mut self, seed: u32, day: i64) {
        let old = self.leaver.take();
        self.choose_leaver(old, seed, day);
    }

    /// The first slot nobody lives in (`mNpc_UseFreeAnimalInfo`).
    fn free_slot(&self) -> usize {
        let used = self.resident_slots();
        (0..RESIDENTS_MAX)
            .find(|s| !used.contains(s))
            .unwrap_or(used.len())
    }

    /// `mNpc_GetGoodbyAnimalIdx`: the slot of the villager the player knows least, never the one
    /// who is thinking of moving. `known(entry)` is (met, letter sent).
    fn goodbye_slot(
        &self,
        seed: u32,
        day: i64,
        known: impl Fn(&RosterEntry) -> (bool, bool),
    ) -> Option<usize> {
        use rustimal_logic::force_remove as fr;
        let mut order: Vec<usize> = (0..self.residents.len()).collect();
        order.sort_by_key(|&i| self.slot(i));
        let candidates: Vec<fr::Candidate> = order
            .iter()
            .filter(|&&i| self.leaver != Some(self.slot(i)))
            .map(|&i| {
                let entry = &self.residents[i];
                let (met, letter) = known(entry);
                fr::Candidate {
                    slot: self.slot(i),
                    memories: u32::from(met),
                    letters: u32::from(met && letter),
                    lands: u32::from(met),
                    same_looks: self
                        .residents
                        .iter()
                        .filter(|r| r.looks == entry.looks)
                        .count() as u32,
                }
            })
            .collect();
        let mut rng = Rng::new((seed ^ (day as u32).wrapping_mul(0x85EB_CA6B)) | 1);
        fr::goodbye_slot(&candidates, &mut |n| {
            ((rng.next() * n as f32) as u32).min(n - 1)
        })
    }

    /// `mNpc_GetRemoveAnimal`, moving out: the villager thinking of moving leaves with the player.
    /// Their memories go with them and nobody is thinking of moving until the next pick.
    pub fn take_leaver(&mut self) -> Option<RosterEntry> {
        let slot = self.leaver.take()?;
        let i = (0..self.residents.len()).find(|&i| self.slot(i) == slot)?;
        self.slots = self.resident_slots();
        self.slots.remove(i);
        let gone = self.residents.remove(i);
        self.met.retain(|&m| m != gone.index);
        Some(gone)
    }

    /// `mNpc_GetRemoveAnimal`, moving in: a villager arriving from another town. A full town
    /// loses its goodbye villager (or slot 0) in their place. The summer camper and the last
    /// villager to leave (`last_removed_animal_id`) are not tracked here yet.
    pub fn receive(
        &mut self,
        incoming: RosterEntry,
        day: i64,
        seed: u32,
        known: impl Fn(&RosterEntry) -> (bool, bool),
    ) -> Arrival {
        use rustimal_logic::villager_transfer::{may_move_in, move_in_slot, MoveInSlot};
        let town: Vec<u32> = self.residents.iter().map(|r| r.index).collect();
        if !may_move_in(incoming.index, false, &town, None, None) {
            return Arrival::Refused;
        }
        let free = (self.residents.len() < RESIDENTS_MAX).then(|| self.free_slot());
        let goodbye = match free {
            Some(_) => None,
            None => self.goodbye_slot(seed, day, known),
        };
        self.appeared.push(incoming.index);
        self.met.retain(|&m| m != incoming.index);
        self.remove_day = Some(day);
        match move_in_slot(free, goodbye) {
            MoveInSlot::Free(slot) => {
                self.slots = self.resident_slots();
                self.slots.push(slot);
                self.residents.push(incoming);
                Arrival::MovedIn
            }
            MoveInSlot::Replace(slot) => {
                let i = (0..self.residents.len())
                    .find(|&i| self.slot(i) == slot)
                    .unwrap_or(0);
                if self.leaver == Some(self.slot(i)) {
                    self.leaver = None;
                }
                Arrival::Replaced(std::mem::replace(&mut self.residents[i], incoming))
            }
        }
    }

    /// `mNpc_ForceRemove`: with a full town and no move-in or removal for ten days, the villager
    /// the player knows least leaves. `known(index)` says whether the player has met them and
    /// whether they have been sent a letter. Returns who left.
    pub fn force_remove(
        &mut self,
        day: i64,
        seed: u32,
        known: impl Fn(&RosterEntry) -> (bool, bool),
    ) -> Option<RosterEntry> {
        use rustimal_logic::force_remove as fr;
        if !fr::due(self.residents.len(), self.remove_day, day) {
            return None;
        }
        let slot = self.goodbye_slot(seed, day, known)?;
        let i = (0..self.residents.len()).find(|&i| self.slot(i) == slot)?;
        if self.leaver == Some(slot) {
            self.leaver = None;
        }
        self.slots = self.resident_slots();
        self.slots.remove(i);
        let gone = self.residents.remove(i);
        self.remove_day = Some(day);
        Some(gone)
    }

    pub fn grow(
        &mut self,
        tables: &VillagerTables,
        day: i64,
        seed: u32,
        rank: impl FnOnce() -> Option<u8>,
    ) -> Option<RosterEntry> {
        if self.residents.len() >= RESIDENTS_MAX {
            return None;
        }
        let Some(last) = self.last_grow_day else {
            // First check: start the clock (`last_grow_time` was clear).
            self.last_grow_day = Some(day);
            return None;
        };
        let mut rng = Rng::new((seed ^ (day as u32).wrapping_mul(0x9E37_79B9)) | 1);
        let met_all = self.residents.iter().all(|r| self.met.contains(&r.index));
        if day <= last || !met_all {
            return None;
        }
        // `mNpc_CheckGrowFieldRank`: RANDOM(100) < npc_grow_prob[rank].
        let roll = (rng.next() * 100.0) as u8;
        if !rank()
            .is_some_and(|rank| rustimal_logic::check_grow_field_rank(usize::from(rank), roll))
        {
            return None;
        }
        self.last_grow_day = Some(day);
        let looks = self.min_looks(tables, &mut rng)?;
        let idx = self.pick(tables, looks, &mut rng)?;
        let entry = tables.entry(idx as u32)?;
        self.appeared.push(entry.index);
        let slot = self.free_slot();
        self.slots = self.resident_slots();
        self.slots.push(slot);
        self.residents.push(entry.clone());
        self.remove_day = Some(day);
        Some(entry)
    }

    fn living(&self, idx: usize) -> bool {
        self.residents.iter().any(|r| r.index as usize == idx)
    }

    /// `mNpc_GetMinLooks` (+ the sex tie-break in `mNpc_Grow`): the
    /// personality with the fewest residents among those with someone
    /// left to move in.
    fn min_looks(&mut self, tables: &VillagerTables, rng: &mut Rng) -> Option<u8> {
        let waiting = |roster: &Self, looks: u8| {
            (0..NPC_NUM).any(|i| {
                tables.looks.get(i) == Some(&looks)
                    && tables.can_move_in(i)
                    && !roster.appeared.contains(&(i as u32))
                    && !roster.living(i)
            })
        };
        let count =
            |roster: &Self, looks: u8| roster.residents.iter().filter(|r| r.looks == looks).count();
        let mut tied: Vec<u8> = Vec::new();
        let mut best = usize::MAX;
        for looks in 0..PERSONALITIES as u8 {
            if !waiting(self, looks) {
                continue;
            }
            let n = count(self, looks);
            if n < best {
                (best, tied) = (n, vec![looks]);
            } else if n == best {
                tied.push(looks);
            }
        }
        if tied.is_empty() {
            // Everyone has been here before: start over (residents stay).
            self.appeared
                .retain(|i| self.residents.iter().any(|r| r.index == *i));
            tied = (0..PERSONALITIES as u8).collect();
        }
        if tied.len() > 1 {
            // `mNpc_GetMinSex` (as written: male when females < males).
            let females = self
                .residents
                .iter()
                .filter(|r| looks_female(r.looks))
                .count();
            let males = self.residents.len() - females;
            let want_female = females >= males;
            let same: Vec<u8> = tied
                .iter()
                .copied()
                .filter(|&l| looks_female(l) == want_female)
                .collect();
            if !same.is_empty() && same.len() != tied.len() {
                tied = same;
            }
        }
        let pick = ((rng.next() * tied.len() as f32) as usize).min(tied.len() - 1);
        Some(tied[pick])
    }

    /// `mNpc_GrowLooksNpcIdx`: a random villager of `looks` who isn't here,
    /// hasn't been, and may move in.
    fn pick(&self, tables: &VillagerTables, looks: u8, rng: &mut Rng) -> Option<usize> {
        let candidates: Vec<usize> = (0..NPC_NUM)
            .filter(|&i| {
                tables.looks.get(i) == Some(&looks)
                    && tables.can_move_in(i)
                    && !self.living(i)
                    && !self.appeared.contains(&(i as u32))
            })
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let k = ((rng.next() * candidates.len() as f32) as usize).min(candidates.len() - 1);
        Some(candidates[k])
    }
}

/// The reserved home spot each resident gets (`mNpc_SetNpcHome`): a random
/// order over the `count` spots (`mNpc_MakeRandTable(.., 30)`), fixed by
/// the town seed so residents keep their homes as others move in.
pub fn home_spot_order(count: usize, seed: u32) -> Vec<usize> {
    if count == 0 {
        return Vec::new();
    }
    let mut rng = Rng::new(seed.wrapping_mul(0x85EB_CA6B) | 1);
    rand_table(count, 30, &mut rng)
}

/// A name: printable characters up to the padding.
fn decode_name(raw: &[u8]) -> String {
    raw.iter()
        .take_while(|&&c| c != 0)
        .filter(|c| c.is_ascii_graphic() || **c == b' ')
        .map(|&c| c as char)
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// `fqrand()` on a stream derived from the town seed (and day), so a town's
/// residents and move-in picks do not reroll when the save is reloaded.
struct Rng(Qrand);

impl Rng {
    fn new(seed: u32) -> Self {
        Self(Qrand::new(seed))
    }

    /// Uniform in [0, 1).
    fn next(&mut self) -> f32 {
        self.0.fqrand()
    }
}

/// `mNpc_MakeRandTable`: 0..count, then `swaps` random pair swaps.
fn rand_table(count: usize, swaps: usize, rng: &mut Rng) -> Vec<usize> {
    let mut table: Vec<usize> = (0..count).collect();
    for _ in 0..swaps {
        let a = ((rng.next() * count as f32) as usize).min(count - 1);
        let b = ((rng.next() * count as f32) as usize).min(count - 1);
        table.swap(a, b);
    }
    table
}

/// Startup, before the town is built: the residents from the save (or a
/// new town's starters), then the day's move-in check.
fn init_roster(
    mut commands: bevy::prelude::Commands,
    disc: Option<bevy::prelude::Res<GameDisc>>,
    loaded: bevy::prelude::Res<crate::save::LoadedSave>,
    seed: bevy::prelude::Res<crate::town::TownSeed>,
) {
    let Some(tables) = disc.as_deref().and_then(VillagerTables::from_disc) else {
        return;
    };
    let save = loaded.0.as_ref();
    let residents: Vec<RosterEntry> = match save.and_then(|s| s.villagers.as_ref()) {
        Some(ids) => ids.iter().filter_map(|&i| tables.entry(i)).collect(),
        None => tables.starters(seed.0),
    };
    let mut roster = TownRoster {
        appeared: save
            .map(|s| s.villagers_appeared.clone())
            .filter(|a| !a.is_empty())
            .unwrap_or_else(|| residents.iter().map(|r| r.index).collect()),
        met: save.map(|s| s.villagers_met.clone()).unwrap_or_default(),
        last_grow_day: save.and_then(|s| s.last_move_in_day),
        slots: save
            .map(|s| s.villager_slots.clone())
            .filter(|slots| slots.len() == residents.len())
            .unwrap_or_default(),
        remove_day: save.and_then(|s| s.last_remove_day),
        leaver: save.and_then(|s| s.remove_slot),
        residents,
    };
    let day = crate::clock::now().day_number;
    // `mSDI_StartInitAfter`: `mNpc_ForceRemove` comes before `mNpc_Grow`.
    let letters: &[String] = save
        .and_then(|s| s.post.as_ref())
        .map_or(&[], |p| p.sent_to.as_slice());
    let met = roster.met.clone();
    let mut goodbyes = Vec::new();
    let known = |e: &RosterEntry| (met.contains(&e.index), letters.contains(&e.name));
    if let Some(gone) = roster.force_remove(day, seed.0, known) {
        bevy::prelude::info!("{} moved out", gone.name);
        goodbyes.push(gone);
    }
    let rank = || crate::field_rank::rank_at_start(disc.as_deref()?, seed.0, save);
    if let Some(new) = roster.grow(&tables, crate::clock::now().day_number, seed.0, rank) {
        bevy::prelude::info!("{} moved in", new.name);
    }
    if let Ok(mode) = std::env::var("BEVY_CROSSING_TRANSFER") {
        debug_transfer(
            &mode,
            &mut roster,
            &tables,
            (day, seed.0),
            known,
            &mut goodbyes,
        );
    }
    commands.insert_resource(PendingGoodbyes(goodbyes));
    // `mSDI_StartInitAfter`: someone is picked to be thinking of moving.
    roster.choose_leaver(None, seed.0, day);
    commands.insert_resource(roster);
}

/// The file `BEVY_CROSSING_TRANSFER` passes a villager through: the memory card's foreigner file.
const TRANSFER_FILE: &str = "villager-transfer.txt";

/// QA aid (`BEVY_CROSSING_TRANSFER=out|in`): the port has no memory card or other towns, so
/// `out` packs the villager thinking of moving into `villager-transfer.txt` and `in` takes the
/// villager in it into this town (`mNpc_GetRemoveAnimal`, moving out / moving in).
fn debug_transfer(
    mode: &str,
    roster: &mut TownRoster,
    tables: &VillagerTables,
    (day, seed): (i64, u32),
    known: impl Fn(&RosterEntry) -> (bool, bool),
    goodbyes: &mut Vec<RosterEntry>,
) {
    use bevy::prelude::{info, warn};
    match mode {
        "out" => match roster.take_leaver() {
            Some(gone) => {
                info!("transfer out: {} (index {})", gone.name, gone.index);
                if let Err(e) = std::fs::write(TRANSFER_FILE, gone.index.to_string()) {
                    warn!("transfer file: {e}");
                }
            }
            None => info!("transfer out: nobody is thinking of moving"),
        },
        "in" => {
            let incoming = std::fs::read_to_string(TRANSFER_FILE)
                .ok()
                .and_then(|text| text.trim().parse::<u32>().ok())
                .and_then(|index| tables.entry(index));
            let Some(incoming) = incoming else {
                warn!("transfer in: no villager in {TRANSFER_FILE}");
                return;
            };
            let name = incoming.name.clone();
            match roster.receive(incoming, day, seed, known) {
                Arrival::Refused => info!("transfer in: {name} refused"),
                Arrival::MovedIn => info!("transfer in: {name} moved in"),
                Arrival::Replaced(gone) => {
                    info!("transfer in: {name} moved in, {} left", gone.name);
                    goodbyes.push(gone);
                }
            }
        }
        other => warn!("BEVY_CROSSING_TRANSFER={other}: expected out or in"),
    }
}

/// What `TownRoster::receive` did with an arriving villager.
#[derive(Debug, PartialEq, Eq)]
pub enum Arrival {
    Refused,
    MovedIn,
    /// The town was full: this villager left in their place.
    Replaced(RosterEntry),
}

/// Villagers who moved out at this start and whose goodbye letters are still to be sent
/// (`l_mnpc_goodby_mail`).
#[derive(bevy::prelude::Resource, Default)]
pub struct PendingGoodbyes(pub Vec<RosterEntry>);

/// `aQMgr_control_animal_info`: the quest order 2 of "would you miss me?" (0x1200) talks the
/// villager out of moving, and someone else is picked (`aQMgr_order_cancel_remove`).
fn cancel_moving(
    mut orders: bevy::prelude::ResMut<crate::msg_window::DemoOrders>,
    roster: Option<bevy::prelude::ResMut<TownRoster>>,
    seed: Option<bevy::prelude::Res<crate::town::TownSeed>>,
) {
    let queued = orders.take_quest();
    let (Some(mut roster), Some(seed)) = (roster, seed) else {
        return;
    };
    for (index, value) in queued {
        if (index, value) == (1, 2) {
            roster.cancel_leaver(seed.0, crate::clock::now().day_number);
            bevy::prelude::info!("talked out of moving; now {:?}", roster.leaver);
        }
    }
}

pub struct RosterPlugin;

impl bevy::prelude::Plugin for RosterPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        use bevy::prelude::IntoScheduleConfigs;
        app.add_systems(
            bevy::prelude::Startup,
            init_roster
                .after(crate::game_disc::start_disc_load)
                .before(crate::town::generate_town),
        )
        .add_systems(
            bevy::prelude::Update,
            cancel_moving.in_set(crate::states::GameplaySet),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tables() -> VillagerTables {
        // 12 villagers: two of each personality; odd ones aren't starters.
        VillagerTables {
            names: (0..NPC_NUM).map(|i| format!("V{i}")).collect(),
            looks: (0..NPC_NUM)
                .map(|i| (i / 2 % PERSONALITIES) as u8)
                .collect(),
            grow: (0..NPC_NUM)
                .map(|i| if i % 2 == 0 { 0 } else { 1 })
                .collect(),
        }
    }

    #[test]
    fn one_starter_per_personality() {
        let t = tables();
        let picked = t.starters(42);
        assert_eq!(picked.len(), PERSONALITIES);
        let mut looks: Vec<u8> = picked.iter().map(|e| e.looks).collect();
        looks.sort_unstable();
        assert_eq!(looks, vec![0, 1, 2, 3, 4, 5]);
        assert!(picked.iter().all(|e| e.index % 2 == 0), "starters only");
        assert_eq!(t.starters(42), picked, "same seed, same town");
    }

    fn town(t: &VillagerTables) -> TownRoster {
        let residents = t.starters(7);
        TownRoster {
            appeared: residents.iter().map(|r| r.index).collect(),
            residents,
            ..TownRoster::default()
        }
    }

    #[test]
    fn nobody_moves_in_until_everyone_is_met() {
        let t = tables();
        let mut r = town(&t);
        assert!(
            r.grow(&t, 100, 1, || Some(3)).is_none(),
            "first check only starts the clock"
        );
        for day in 101..200 {
            assert!(r.grow(&t, day, 1, || Some(3)).is_none());
        }
        assert_eq!(r.residents.len(), PERSONALITIES);
    }

    #[test]
    fn a_new_villager_moves_in_once_a_day() {
        let t = tables();
        let mut r = town(&t);
        r.met = r.residents.iter().map(|e| e.index).collect();
        r.last_grow_day = Some(100);
        assert!(r.grow(&t, 100, 1, || Some(3)).is_none(), "same day");
        let new = (101..200)
            .find_map(|day| r.grow(&t, day, 1, || Some(3)))
            .expect("someone moves in");
        assert_eq!(r.residents.len(), PERSONALITIES + 1);
        assert!(r.appeared.contains(&new.index));
        assert!(
            t.can_move_in(new.index as usize),
            "only villagers allowed to move in (starter or move-in)"
        );
        let first: Vec<u32> = t.starters(7).iter().map(|e| e.index).collect();
        assert!(
            !first.contains(&new.index),
            "not someone already living here"
        );
    }

    #[test]
    fn home_order_is_stable() {
        let a = home_spot_order(10, 5);
        assert_eq!(a, home_spot_order(10, 5));
        let mut sorted = a.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn names_trim_padding() {
        assert_eq!(decode_name(b"Bob     "), "Bob");
        assert_eq!(decode_name(b"Maple\0\0\0"), "Maple");
    }

    #[test]
    fn the_villager_thinking_of_moving_can_be_talked_out_of_it() {
        use bevy::prelude::*;
        let t = tables();
        let mut r = town(&t);
        r.residents.extend((6..8).filter_map(|i| t.entry(i)));
        r.met = r.residents.iter().map(|e| e.index).collect();
        r.choose_leaver(None, 3, 100);
        let first = r
            .leaver
            .expect("eight villagers: someone is thinking of moving");
        assert!(r.is_leaver(r.residents[first].index));
        let mut app = App::new();
        app.insert_resource(r)
            .insert_resource(crate::town::TownSeed(3))
            .init_resource::<crate::msg_window::DemoOrders>()
            .add_systems(Update, cancel_moving);
        // The "would you miss me?" answer's quest order (index 1, value 2).
        app.world_mut()
            .resource_mut::<crate::msg_window::DemoOrders>()
            .set(4, 1, 2);
        app.update();
        let after = app.world().resource::<TownRoster>().leaver;
        assert!(
            after.is_some() && after != Some(first),
            "{first:?} -> {after:?}"
        );
        // An unrelated quest order changes nothing.
        app.world_mut()
            .resource_mut::<crate::msg_window::DemoOrders>()
            .set(4, 5, 0x66);
        app.update();
        assert_eq!(app.world().resource::<TownRoster>().leaver, after);
    }

    fn full_town(t: &VillagerTables) -> TownRoster {
        let residents: Vec<RosterEntry> = (0..RESIDENTS_MAX as u32)
            .filter_map(|i| t.entry(i))
            .collect();
        TownRoster {
            appeared: residents.iter().map(|r| r.index).collect(),
            residents,
            ..TownRoster::default()
        }
    }

    #[test]
    fn a_full_town_loses_the_villager_the_player_knows_least() {
        let t = tables();
        let mut r = full_town(&t);
        r.remove_day = Some(100);
        assert!(
            r.force_remove(105, 1, |_| (false, false)).is_none(),
            "too soon"
        );
        // Everyone has been met but one.
        let gone = r
            .force_remove(111, 1, |e| (e.index != 4, false))
            .expect("someone leaves");
        assert_eq!(gone.index, 4);
        assert_eq!(r.residents.len(), RESIDENTS_MAX - 1);
        assert_eq!(r.remove_day, Some(111));
        // The others keep their slots; the empty one is the next to be used.
        assert!(!r.resident_slots().contains(&4));
        assert_eq!(r.free_slot(), 4);
        assert_eq!(r.slot(4), 5, "the resident after the gap keeps slot 5");
    }

    #[test]
    fn the_villager_thinking_of_moving_leaves_with_the_player() {
        let t = tables();
        let mut r = full_town(&t);
        r.met = r.residents.iter().map(|e| e.index).collect();
        r.leaver = Some(6);
        let gone = r.take_leaver().expect("slot 6 leaves");
        assert_eq!(gone.index, 6);
        assert_eq!((r.leaver, r.residents.len()), (None, RESIDENTS_MAX - 1));
        assert!(!r.met.contains(&6), "their memories go with them");
        assert!(r.take_leaver().is_none());
        // Back in a town with room they take the free slot; the same town refuses a resident.
        assert_eq!(
            r.receive(gone.clone(), 50, 1, |_| (true, false)),
            Arrival::MovedIn
        );
        assert_eq!(r.residents.len(), RESIDENTS_MAX);
        assert_eq!(r.remove_day, Some(50));
        assert_eq!(r.receive(gone, 50, 1, |_| (true, false)), Arrival::Refused);
    }

    #[test]
    fn a_full_town_swaps_the_goodbye_villager_for_the_newcomer() {
        let t = tables();
        let mut r = full_town(&t);
        // Everyone is known but index 4; the villager thinking of moving (slot 2) is never chosen.
        r.leaver = Some(2);
        let newcomer = t.entry(RESIDENTS_MAX as u32 + 1).expect("a 16th villager");
        let known = |e: &RosterEntry| (e.index != 4, false);
        match r.receive(newcomer.clone(), 60, 1, known) {
            Arrival::Replaced(gone) => assert_eq!(gone.index, 4),
            other => panic!("{other:?}"),
        }
        assert_eq!(r.residents.len(), RESIDENTS_MAX);
        assert_eq!(r.residents[4], newcomer, "the newcomer takes slot 4");
        assert_eq!(r.leaver, Some(2));
    }
}
