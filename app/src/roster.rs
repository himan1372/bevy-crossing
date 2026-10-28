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
        self.residents.push(entry.clone());
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
        residents,
    };
    let rank = || crate::field_rank::rank_at_start(disc.as_deref()?, seed.0, save);
    if let Some(new) = roster.grow(&tables, crate::clock::now().day_number, seed.0, rank) {
        bevy::prelude::info!("{} moved in", new.name);
    }
    commands.insert_resource(roster);
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
        assert_eq!(
            t.grow[new.index as usize], GROW_STARTER,
            "only villagers allowed to move in"
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
}
