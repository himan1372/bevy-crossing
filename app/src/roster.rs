//! The town's villagers, picked from the disc's villager tables the way a
//! new town picks them (`mNpc_DecideLivingNpcMax`): shuffle every villager
//! (`mNpc_MakeRandTable`), then take starters (`npc_grow_list` =
//! `mNpc_GROW_STARTER`) until there is one of each personality
//! (`npc_looks_table`). Our shuffle is seeded from the town seed, so a
//! town keeps the same villagers; the game saves its list instead.

use disc::RomSource;

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
        let mut rng = Rng(seed.wrapping_mul(0x2545_F491) | 1);
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

/// xorshift standing in for `fqrand()`.
struct Rng(u32);

impl Rng {
    /// Uniform in [0, 1).
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 >> 8) as f32 / (1u32 << 24) as f32
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

    #[test]
    fn names_trim_padding() {
        assert_eq!(decode_name(b"Bob     "), "Bob");
        assert_eq!(decode_name(b"Maple\0\0\0"), "Maple");
    }
}
