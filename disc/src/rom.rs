//! The two code/data blobs the game's assets live in: `main.dol` and the
//! decompressed `foresta.rel`.
//!
//! The asset table (rustimal-crossing `pc_assets.c`) addresses each asset
//! as a byte offset straight into one of these files. Pointers *inside*
//! asset data (display list → vertices, textures, sub-lists) are not stored
//! in the file: the REL keeps them in its relocation table, and DOL data
//! holds absolute addresses. This module resolves both to file offsets.

use std::collections::HashMap;
use std::io::{Read, Seek};

use crate::{be32, Disc, DiscError};

const DOL_SECTIONS: usize = 18;
const R_PPC_ADDR32: u8 = 1;
const R_DOLPHIN_NOP: u8 = 201;
const R_DOLPHIN_SECTION: u8 = 202;
const R_DOLPHIN_END: u8 = 203;

/// Which blob an asset or pointer target lives in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RomSource {
    Rel,
    Dol,
}

/// Where a pointer inside asset data leads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// Byte offset into the given file.
    File(RomSource, u32),
    /// Zero-filled memory (bss) or an address outside both files.
    NoData(u32),
}

#[derive(Clone, Copy, Debug)]
struct DolSection {
    file_offset: u32,
    address: u32,
    size: u32,
}

#[derive(Clone, Copy, Debug)]
struct RelSection {
    /// File offset of the section's data; 0 for bss.
    file_offset: u32,
    size: u32,
}

/// `main.dol` + decompressed `foresta.rel`, with pointer resolution.
pub struct RomImage {
    pub dol: Vec<u8>,
    pub rel: Vec<u8>,
    dol_sections: Vec<DolSection>,
    rel_id: u32,
    rel_sections: Vec<RelSection>,
    /// REL file offset of a 32-bit pointer field → where it points.
    rel_pointers: HashMap<u32, Target>,
}

impl RomImage {
    /// Reads `main.dol` and `foresta.rel.szs` from the disc and decodes both.
    pub fn from_disc<R: Read + Seek>(disc: &mut Disc<R>) -> Result<Self, DiscError> {
        let dol = disc.read_dol()?;
        let szs = disc.read_file("foresta.rel.szs")?;
        let rel = crate::decode_yaz0(&szs).ok_or(DiscError::Corrupt("foresta.rel.szs Yaz0"))?;
        Self::new(dol, rel)
    }

    pub fn new(dol: Vec<u8>, rel: Vec<u8>) -> Result<Self, DiscError> {
        let dol_sections = parse_dol(&dol)?;
        let (rel_id, rel_sections) = parse_rel_sections(&rel)?;
        let mut image = Self {
            dol,
            rel,
            dol_sections,
            rel_id,
            rel_sections,
            rel_pointers: HashMap::new(),
        };
        image.rel_pointers = image.parse_rel_relocations()?;
        Ok(image)
    }

    pub fn bytes(&self, src: RomSource) -> &[u8] {
        match src {
            RomSource::Rel => &self.rel,
            RomSource::Dol => &self.dol,
        }
    }

    /// An asset's raw (big-endian) bytes, as addressed by the asset table.
    pub fn slice(&self, src: RomSource, offset: u32, size: u32) -> Option<&[u8]> {
        let start = offset as usize;
        self.bytes(src)
            .get(start..start.checked_add(size as usize)?)
    }

    /// Resolves the 32-bit pointer stored at `offset` in `src`.
    /// Returns `None` if the field holds no pointer (null / not relocated).
    pub fn pointer_at(&self, src: RomSource, offset: u32) -> Option<Target> {
        match src {
            RomSource::Rel => self.rel_pointers.get(&offset).copied(),
            RomSource::Dol => {
                let addr = be32(self.dol.get(offset as usize..)?)?;
                (addr != 0).then(|| self.resolve_dol_addr(addr))
            }
        }
    }

    /// Number of REL data pointers found (for diagnostics).
    pub fn rel_pointer_count(&self) -> usize {
        self.rel_pointers.len()
    }

    fn resolve_dol_addr(&self, addr: u32) -> Target {
        self.dol_sections
            .iter()
            .find(|s| addr >= s.address && addr - s.address < s.size)
            .map_or(Target::NoData(addr), |s| {
                Target::File(RomSource::Dol, s.file_offset + (addr - s.address))
            })
    }

    fn resolve_rel(&self, section: u8, addend: u32) -> Target {
        match self.rel_sections.get(section as usize) {
            Some(s) if s.file_offset != 0 => Target::File(RomSource::Rel, s.file_offset + addend),
            _ => Target::NoData(addend),
        }
    }

    /// Walks the import table; keeps every `R_PPC_ADDR32` (data pointer).
    /// Code relocations (ADDR16_HA/LO, REL24) are skipped: we never run code.
    fn parse_rel_relocations(&self) -> Result<HashMap<u32, Target>, DiscError> {
        let rel = &self.rel;
        let corrupt = || DiscError::Corrupt("REL relocations");
        let imp_offset = be32(&rel[0x28..]).ok_or_else(corrupt)? as usize;
        let imp_size = be32(&rel[0x2C..]).ok_or_else(corrupt)? as usize;
        let imports = rel
            .get(imp_offset..imp_offset + imp_size)
            .ok_or_else(corrupt)?;

        let mut pointers = HashMap::new();
        for imp in imports.as_chunks::<8>().0 {
            let module = be32(&imp[..]).ok_or_else(corrupt)?;
            let mut at = be32(&imp[4..]).ok_or_else(corrupt)? as usize;
            let mut section_base: Option<u32> = None;
            loop {
                let entry = rel.get(at..at + 8).ok_or_else(corrupt)?;
                at += 8;
                let delta = u32::from(u16::from_be_bytes([entry[0], entry[1]]));
                let (kind, section) = (entry[2], entry[3]);
                let addend = be32(&entry[4..]).ok_or_else(corrupt)?;
                match kind {
                    R_DOLPHIN_END => break,
                    R_DOLPHIN_SECTION => {
                        let s = self
                            .rel_sections
                            .get(section as usize)
                            .ok_or_else(corrupt)?;
                        section_base = Some(s.file_offset);
                    }
                    R_DOLPHIN_NOP => {
                        section_base = section_base.map(|b| b + delta);
                    }
                    _ => {
                        let base = section_base.ok_or_else(corrupt)? + delta;
                        section_base = Some(base);
                        if kind == R_PPC_ADDR32 && base != 0 {
                            let target = if module == self.rel_id {
                                self.resolve_rel(section, addend)
                            } else if module == 0 {
                                self.resolve_dol_addr(addend)
                            } else {
                                Target::NoData(addend)
                            };
                            pointers.insert(base, target);
                        }
                    }
                }
            }
        }
        Ok(pointers)
    }
}

fn parse_dol(dol: &[u8]) -> Result<Vec<DolSection>, DiscError> {
    if dol.len() < 0xE4 {
        return Err(DiscError::Corrupt("DOL header"));
    }
    let word = |at: usize| be32(&dol[at..]).unwrap_or(0);
    Ok((0..DOL_SECTIONS)
        .map(|i| DolSection {
            file_offset: word(i * 4),
            address: word(0x48 + i * 4),
            size: word(0x90 + i * 4),
        })
        .filter(|s| s.size != 0)
        .collect())
}

/// REL header (v1–v3): id at 0x00, section count 0x0C, section table 0x10.
fn parse_rel_sections(rel: &[u8]) -> Result<(u32, Vec<RelSection>), DiscError> {
    let corrupt = || DiscError::Corrupt("REL header");
    if rel.len() < 0x40 {
        return Err(corrupt());
    }
    let id = be32(rel).ok_or_else(corrupt)?;
    let count = be32(&rel[0x0C..]).ok_or_else(corrupt)? as usize;
    let table = be32(&rel[0x10..]).ok_or_else(corrupt)? as usize;
    let bytes = rel.get(table..table + count * 8).ok_or_else(corrupt)?;
    let sections = bytes
        .as_chunks::<8>()
        .0
        .iter()
        .map(|s| RelSection {
            file_offset: be32(&s[..]).unwrap_or(0) & !1, // bit 0 = executable flag
            size: be32(&s[4..]).unwrap_or(0),
        })
        .collect::<Vec<_>>();
    if sections
        .iter()
        .any(|s| s.file_offset != 0 && (s.file_offset + s.size) as usize > rel.len())
    {
        return Err(corrupt());
    }
    Ok((id, sections))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(buf: &mut [u8], at: usize, v: u32) {
        buf[at..at + 4].copy_from_slice(&v.to_be_bytes());
    }

    /// DOL with one data section: file 0x100..0x140 loaded at 0x8000_3000.
    /// Holds a pointer at file 0x104 → 0x8000_3010.
    fn synthetic_dol() -> Vec<u8> {
        let mut dol = vec![0u8; 0x140];
        put(&mut dol, 0x1C, 0x100); // data0 file offset
        put(&mut dol, 0x48 + 7 * 4, 0x8000_3000);
        put(&mut dol, 0x90 + 7 * 4, 0x40);
        put(&mut dol, 0x104, 0x8000_3010);
        dol
    }

    /// REL id 1: section 1 = text (exec flag) at 0x80, section 2 = data at
    /// 0xA0, section 3 = bss. Data holds two pointers: one into its own data
    /// (+0x10), one into the DOL (0x8000_3020).
    fn synthetic_rel() -> Vec<u8> {
        let mut rel = vec![0u8; 0x200];
        put(&mut rel, 0x00, 1);
        put(&mut rel, 0x0C, 4);
        put(&mut rel, 0x10, 0x40);
        put(&mut rel, 0x48, 0x80 | 1); // section 1: text
        put(&mut rel, 0x4C, 0x20);
        put(&mut rel, 0x50, 0xA0); // section 2: data
        put(&mut rel, 0x54, 0x40);
        put(&mut rel, 0x5C, 0x100); // section 3: bss (no file data)

        // Imports at 0x100: module 1 relocs at 0x120, module 0 at 0x160.
        put(&mut rel, 0x28, 0x100);
        put(&mut rel, 0x2C, 16);
        put(&mut rel, 0x100, 1);
        put(&mut rel, 0x104, 0x120);
        put(&mut rel, 0x108, 0);
        put(&mut rel, 0x10C, 0x160);

        let reloc = |rel: &mut Vec<u8>, at: usize, delta: u16, kind: u8, sec: u8, add: u32| {
            rel[at..at + 2].copy_from_slice(&delta.to_be_bytes());
            rel[at + 2] = kind;
            rel[at + 3] = sec;
            put(rel, at + 4, add);
        };
        // Module 1: in section 2, +0x4 → (sec 2, +0x10); a code reloc; +0x8 → bss.
        reloc(&mut rel, 0x120, 0, R_DOLPHIN_SECTION, 2, 0);
        reloc(&mut rel, 0x128, 4, R_PPC_ADDR32, 2, 0x10);
        reloc(&mut rel, 0x130, 2, 4, 2, 0); // ADDR16_LO: skipped
        reloc(&mut rel, 0x138, 2, R_PPC_ADDR32, 3, 0x8);
        reloc(&mut rel, 0x140, 0, R_DOLPHIN_END, 0, 0);
        // Module 0 (DOL): in section 2, +0x20 → 0x8000_3020; NOP advances.
        reloc(&mut rel, 0x160, 0, R_DOLPHIN_SECTION, 2, 0);
        reloc(&mut rel, 0x168, 0x10, R_DOLPHIN_NOP, 0, 0);
        reloc(&mut rel, 0x170, 0x10, R_PPC_ADDR32, 0, 0x8000_3020);
        reloc(&mut rel, 0x178, 0, R_DOLPHIN_END, 0, 0);
        rel
    }

    #[test]
    fn resolves_rel_and_dol_pointers() {
        let rom = RomImage::new(synthetic_dol(), synthetic_rel()).unwrap();
        assert_eq!(rom.rel_pointer_count(), 3);
        let data = 0xA0;
        assert_eq!(
            rom.pointer_at(RomSource::Rel, data + 4),
            Some(Target::File(RomSource::Rel, data + 0x10))
        );
        assert_eq!(
            rom.pointer_at(RomSource::Rel, data + 8),
            Some(Target::NoData(8))
        );
        assert_eq!(
            rom.pointer_at(RomSource::Rel, data + 0x20),
            Some(Target::File(RomSource::Dol, 0x120))
        );
        assert_eq!(rom.pointer_at(RomSource::Rel, data), None);
        assert_eq!(
            rom.pointer_at(RomSource::Dol, 0x104),
            Some(Target::File(RomSource::Dol, 0x110))
        );
        assert_eq!(rom.pointer_at(RomSource::Dol, 0x108), None);
        assert_eq!(
            rom.slice(RomSource::Dol, 0x104, 4),
            Some(&[0x80, 0, 0x30, 0x10][..])
        );
        assert_eq!(rom.slice(RomSource::Rel, 0x1FF, 2), None);
    }

    #[test]
    fn rejects_bad_headers() {
        assert!(RomImage::new(vec![0; 8], synthetic_rel()).is_err());
        assert!(RomImage::new(synthetic_dol(), vec![0; 8]).is_err());
        let mut rel = synthetic_rel();
        put(&mut rel, 0x54, 0x10_000); // data section runs past the file
        assert!(RomImage::new(synthetic_dol(), rel).is_err());
    }
}
