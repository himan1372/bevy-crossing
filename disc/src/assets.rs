//! Asset catalog: named data objects in `foresta.rel` / `main.dol`.
//!
//! Names, sections and sizes come from the ac-decomp symbol maps, converted
//! by `tools/gen_symbols.py` into `disc/data/*.sym` (metadata only, no game
//! bytes). They're resolved against a [`RomImage`] into file offsets.

use std::collections::HashMap;

use crate::rom::{RomImage, RomSource, Target};

const FORESTA_SYM: &str = include_str!("../data/foresta.sym");
const DOL_SYM: &str = include_str!("../data/dol.sym");

/// REL section indices in `foresta.rel` (dtk layout: 1 .text, 2 .ctors,
/// 3 .dtors, 4 .rodata, 5 .data, 6 .bss).
const REL_RODATA: u8 = 4;
const REL_DATA: u8 = 5;

/// A named object with its location in the ROM image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Asset<'a> {
    pub name: &'a str,
    pub source: RomSource,
    pub offset: u32,
    pub size: u32,
}

/// All named data objects that have bytes in the ROM image.
pub struct AssetCatalog<'a> {
    /// Sorted by (source, offset) for reverse lookup.
    assets: Vec<Asset<'a>>,
    /// Name → indices into `assets` (static names can repeat).
    by_name: HashMap<&'a str, Vec<usize>>,
}

impl AssetCatalog<'static> {
    /// The catalog for Animal Crossing NTSC-U, from the built-in symbol maps.
    pub fn builtin(rom: &RomImage) -> Self {
        Self::from_sym(rom, FORESTA_SYM, DOL_SYM)
    }
}

impl<'a> AssetCatalog<'a> {
    /// Builds a catalog from `.sym` text (see `tools/gen_symbols.py`).
    /// Malformed lines and objects without file data (bss) are skipped.
    pub fn from_sym(rom: &RomImage, foresta: &'a str, dol: &'a str) -> Self {
        let rel = parse(foresta).filter_map(|(name, sec, value, size)| {
            let section = match sec {
                "d" => REL_DATA,
                "r" => REL_RODATA,
                _ => return None,
            };
            located(name, rom.resolve_rel(section, value), size)
        });
        let dol = parse(dol).filter_map(|(name, sec, value, size)| {
            (sec != "b")
                .then(|| located(name, rom.resolve_dol_addr(value), size))
                .flatten()
        });

        let mut assets: Vec<Asset> = rel.chain(dol).collect();
        assets.sort_by_key(|a| (a.source == RomSource::Dol, a.offset));
        let mut by_name: HashMap<&str, Vec<usize>> = HashMap::with_capacity(assets.len());
        for (i, a) in assets.iter().enumerate() {
            by_name.entry(a.name).or_default().push(i);
        }
        Self { assets, by_name }
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    /// First object with this name (REL before DOL, lowest offset first).
    pub fn get(&self, name: &str) -> Option<Asset<'a>> {
        self.all(name).next()
    }

    /// Every object with this name (file-local statics can share a name).
    pub fn all<'s>(&'s self, name: &str) -> impl Iterator<Item = Asset<'a>> + 's {
        self.by_name
            .get(name)
            .into_iter()
            .flatten()
            .map(|&i| self.assets[i])
    }

    /// The object containing `offset` in `source`, e.g. a pointer target.
    pub fn containing(&self, source: RomSource, offset: u32) -> Option<Asset<'a>> {
        let key = (source == RomSource::Dol, offset);
        let i = self
            .assets
            .partition_point(|a| (a.source == RomSource::Dol, a.offset) <= key);
        let a = *self.assets.get(i.checked_sub(1)?)?;
        (a.source == source && offset - a.offset < a.size.max(1)).then_some(a)
    }
}

fn located(name: &str, target: Target, size: u32) -> Option<Asset<'_>> {
    match target {
        Target::File(source, offset) => Some(Asset {
            name,
            source,
            offset,
            size,
        }),
        Target::NoData(_) => None,
    }
}

/// Yields (name, section char, value, size) from `.sym` text.
fn parse(text: &str) -> impl Iterator<Item = (&str, &str, u32, u32)> {
    text.lines().filter_map(|line| {
        let mut f = line.split('\t');
        let (name, sec) = (f.next()?, f.next()?);
        let value = u32::from_str_radix(f.next()?, 16).ok()?;
        let size = u32::from_str_radix(f.next()?, 16).ok()?;
        Some((name, sec, value, size))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(buf: &mut [u8], at: usize, v: u32) {
        buf[at..at + 4].copy_from_slice(&v.to_be_bytes());
    }

    /// REL with .rodata (sec 4) at 0x100 and .data (sec 5) at 0x200;
    /// DOL with one section at file 0x100 loaded at 0x8000_3000.
    fn rom() -> RomImage {
        let mut rel = vec![0u8; 0x300];
        put(&mut rel, 0x0C, 7);
        put(&mut rel, 0x10, 0x40);
        put(&mut rel, 0x40 + 4 * 8, 0x100);
        put(&mut rel, 0x44 + 4 * 8, 0x80);
        put(&mut rel, 0x40 + 5 * 8, 0x200);
        put(&mut rel, 0x44 + 5 * 8, 0x80);
        put(&mut rel, 0x28, 0x2F0); // empty import table
        let mut dol = vec![0u8; 0x140];
        put(&mut dol, 0x1C, 0x100);
        put(&mut dol, 0x48 + 7 * 4, 0x8000_3000);
        put(&mut dol, 0x90 + 7 * 4, 0x40);
        RomImage::new(dol, rel).expect("synthetic rom")
    }

    const FORESTA: &str = "tree_v\td\t10\t20\ntree_tex\tr\t0\t40\nshared\td\t40\t8\nshared\td\t50\t8\nbss_thing\tb\t0\t10\nbad line\n";
    const DOL: &str = "dol_tbl\td\t80003010\t10\ndol_bss\tb\t80004000\t10\n";

    #[test]
    fn resolves_and_looks_up() {
        let rom = rom();
        let cat = AssetCatalog::from_sym(&rom, FORESTA, DOL);
        assert_eq!(cat.len(), 5);
        let v = cat.get("tree_v").unwrap();
        assert_eq!((v.source, v.offset, v.size), (RomSource::Rel, 0x210, 0x20));
        assert_eq!(cat.get("tree_tex").unwrap().offset, 0x100);
        assert_eq!(cat.all("shared").count(), 2);
        assert!(cat.get("bss_thing").is_none());
        assert!(cat.get("dol_bss").is_none());
        let d = cat.get("dol_tbl").unwrap();
        assert_eq!((d.source, d.offset), (RomSource::Dol, 0x110));
    }

    #[test]
    fn reverse_lookup() {
        let rom = rom();
        let cat = AssetCatalog::from_sym(&rom, FORESTA, DOL);
        assert_eq!(
            cat.containing(RomSource::Rel, 0x21F).unwrap().name,
            "tree_v"
        );
        assert!(cat.containing(RomSource::Rel, 0x230).is_none());
        assert!(cat.containing(RomSource::Rel, 0x0).is_none());
        assert_eq!(
            cat.containing(RomSource::Dol, 0x115).unwrap().name,
            "dol_tbl"
        );
        assert!(cat.containing(RomSource::Dol, 0x210).is_none());
    }

    #[test]
    fn builtin_tables_parse() {
        assert!(parse(FORESTA_SYM).count() > 30_000);
        assert!(parse(DOL_SYM).count() > 1_000);
    }
}
