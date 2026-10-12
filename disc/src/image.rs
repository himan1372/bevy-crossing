//! GameCube disc images: ISO/GCM (raw) and CISO (block-mapped), plus the
//! disc header, DOL location and FST (file table).

use std::fs::{self, File};
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::be32;

/// Animal Crossing, NTSC-U. The only game ID accepted for now (disc-setup).
pub const GAME_ID_NTSC_U: &[u8; 6] = b"GAFE01";

const CISO_HEADER_SIZE: usize = 0x8000;
const CISO_MAP_OFFSET: usize = 8;
const GC_MAGIC: u32 = 0xC233_9F3D;
const MAX_FST_SIZE: u32 = 128 * 1024 * 1024;
const MAX_DIR_DEPTH: usize = 32;

/// CISO block size and logical -> physical block map.
type CisoMap = (u32, Vec<Option<u32>>);

#[derive(Debug, thiserror::Error)]
pub enum DiscError {
    #[error("disc read failed: {0}")]
    Io(#[from] io::Error),
    /// Not a GameCube disc (bad magic or CISO header).
    #[error("not a GameCube disc image")]
    NotGameCube,
    /// A GameCube disc, but not one we support.
    #[error("wrong game {} (need Animal Crossing NTSC-U, GAFE01)", String::from_utf8_lossy(.0))]
    WrongGame([u8; 6]),
    /// Header, DOL or FST data points outside the image or is malformed.
    #[error("disc image is damaged: {0}")]
    Corrupt(&'static str),
    #[error("file not on disc: {0}")]
    FileNotFound(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FstFile {
    /// Path inside the disc, `/`-separated, no leading slash.
    pub path: String,
    pub disc_offset: u32,
    pub size: u32,
}

/// An open disc image. Generic over the reader so tests can use memory.
pub struct Disc<R> {
    reader: R,
    /// CISO block size and logical â†’ physical block map; `None` for raw images.
    ciso: Option<CisoMap>,
    game_id: [u8; 6],
    dol_offset: u32,
    dol_size: u32,
    files: Vec<FstFile>,
}

impl Disc<BufReader<File>> {
    pub fn open_path(path: &Path) -> Result<Self, DiscError> {
        Self::open(BufReader::new(File::open(path)?))
    }
}

impl<R: Read + Seek> Disc<R> {
    /// Opens an image, checks it is Animal Crossing (GAFE01), and reads the FST.
    pub fn open(mut reader: R) -> Result<Self, DiscError> {
        let ciso = read_ciso_map(&mut reader)?;
        let mut disc = Self {
            reader,
            ciso,
            game_id: [0; 6],
            dol_offset: 0,
            dol_size: 0,
            files: Vec::new(),
        };

        let mut header = [0u8; 0x20];
        disc.read_at(0, &mut header)
            .map_err(|_| DiscError::NotGameCube)?;
        if be32(&header[0x1C..]) != Some(GC_MAGIC) {
            return Err(DiscError::NotGameCube);
        }
        disc.game_id.copy_from_slice(&header[..6]);
        if &disc.game_id != GAME_ID_NTSC_U {
            return Err(DiscError::WrongGame(disc.game_id));
        }

        let mut word = [0u8; 4];
        disc.read_at(0x420, &mut word)?;
        disc.dol_offset = be32(&word).unwrap_or(0);
        disc.dol_size = disc.dol_size_from_header()?;
        disc.files = disc.read_fst()?;
        Ok(disc)
    }

    pub fn game_id(&self) -> &[u8; 6] {
        &self.game_id
    }

    pub fn files(&self) -> &[FstFile] {
        &self.files
    }

    /// Case-insensitive lookup; a leading `/` is ignored.
    pub fn find(&self, path: &str) -> Option<&FstFile> {
        let path = path.trim_start_matches('/');
        self.files
            .iter()
            .find(|f| f.path.eq_ignore_ascii_case(path))
    }

    pub fn read_file(&mut self, path: &str) -> Result<Vec<u8>, DiscError> {
        let entry = self
            .find(path)
            .cloned()
            .ok_or_else(|| DiscError::FileNotFound(path.to_owned()))?;
        let mut out = vec![0; entry.size as usize];
        self.read_at(entry.disc_offset, &mut out)?;
        Ok(out)
    }

    /// The main executable (`main.dol`), which holds part of the game's assets.
    pub fn read_dol(&mut self) -> Result<Vec<u8>, DiscError> {
        let mut out = vec![0; self.dol_size as usize];
        self.read_at(self.dol_offset, &mut out)?;
        Ok(out)
    }

    /// Reads `out.len()` bytes at a logical disc offset.
    pub fn read_at(&mut self, offset: u32, out: &mut [u8]) -> io::Result<()> {
        let Some((block_size, map)) = &self.ciso else {
            self.reader.seek(SeekFrom::Start(offset.into()))?;
            return self.reader.read_exact(out);
        };
        let block_size = u64::from(*block_size);
        let mut logical = u64::from(offset);
        let mut written = 0;
        while written < out.len() {
            let within = logical % block_size;
            let chunk = ((block_size - within) as usize).min(out.len() - written);
            let dest = &mut out[written..written + chunk];
            match map.get((logical / block_size) as usize).copied().flatten() {
                None => dest.fill(0),
                Some(physical) => {
                    let at = CISO_HEADER_SIZE as u64 + u64::from(physical) * block_size + within;
                    self.reader.seek(SeekFrom::Start(at))?;
                    self.reader.read_exact(dest)?;
                }
            }
            logical += chunk as u64;
            written += chunk;
        }
        Ok(())
    }

    /// DOL size = furthest end of its 7 text + 11 data sections.
    fn dol_size_from_header(&mut self) -> Result<u32, DiscError> {
        let mut h = [0u8; 0xE4];
        self.read_at(self.dol_offset, &mut h)
            .map_err(|_| DiscError::Corrupt("DOL header"))?;
        let mut end = 0u32;
        for i in 0..18 {
            let off = be32(&h[i * 4..]).unwrap_or(0);
            let size = be32(&h[0x90 + i * 4..]).unwrap_or(0);
            end = end.max(
                off.checked_add(size)
                    .ok_or(DiscError::Corrupt("DOL section"))?,
            );
        }
        Ok(end)
    }

    fn read_fst(&mut self) -> Result<Vec<FstFile>, DiscError> {
        let mut head = [0u8; 8];
        self.read_at(0x424, &mut head)?;
        let (fst_offset, fst_size) = (be32(&head).unwrap_or(0), be32(&head[4..]).unwrap_or(0));
        if !(12..=MAX_FST_SIZE).contains(&fst_size) {
            return Err(DiscError::Corrupt("FST size"));
        }
        let mut fst = vec![0u8; fst_size as usize];
        self.read_at(fst_offset, &mut fst)
            .map_err(|_| DiscError::Corrupt("FST outside image"))?;

        // Root entry: its "size" field is the total entry count.
        let count = be32(&fst[8..]).unwrap_or(0) as usize;
        if count == 0 || count.saturating_mul(12) > fst.len() {
            return Err(DiscError::Corrupt("FST entry count"));
        }
        let strings = &fst[count * 12..];
        let name_at = |off: usize| -> Result<String, DiscError> {
            let bytes = strings.get(off..).ok_or(DiscError::Corrupt("FST name"))?;
            let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
            Ok(String::from_utf8_lossy(&bytes[..end]).into_owned())
        };

        // Stack of (end index, dir name); the root has no name.
        let mut dirs: Vec<(usize, String)> = vec![(count, String::new())];
        let mut files = Vec::new();
        for index in 1..count {
            while dirs.last().is_some_and(|(end, _)| index >= *end) {
                dirs.pop();
            }
            let node = &fst[index * 12..index * 12 + 12];
            let name = name_at(
                usize::from(node[1]) << 16 | usize::from(node[2]) << 8 | usize::from(node[3]),
            )?;
            let (field_a, field_b) = (be32(&node[4..]).unwrap_or(0), be32(&node[8..]).unwrap_or(0));
            if node[0] == 1 {
                let end = field_b as usize;
                if end <= index || end > count || dirs.len() > MAX_DIR_DEPTH {
                    return Err(DiscError::Corrupt("FST directory"));
                }
                dirs.push((end, name));
            } else {
                let mut path: Vec<&str> = dirs.iter().skip(1).map(|(_, d)| d.as_str()).collect();
                path.push(&name);
                files.push(FstFile {
                    path: path.join("/"),
                    disc_offset: field_a,
                    size: field_b,
                });
            }
        }
        Ok(files)
    }
}

/// Reads the CISO header if present, then rewinds. `None` = raw ISO/GCM.
fn read_ciso_map<R: Read + Seek>(reader: &mut R) -> Result<Option<CisoMap>, DiscError> {
    let mut header = vec![0u8; CISO_HEADER_SIZE];
    let is_ciso = reader.read_exact(&mut header).is_ok() && &header[..4] == b"CISO";
    reader.seek(SeekFrom::Start(0))?;
    if !is_ciso {
        return Ok(None);
    }
    let block_size = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
    if block_size == 0 {
        return Err(DiscError::NotGameCube);
    }
    let mut physical = 0u32;
    let map = header[CISO_MAP_OFFSET..]
        .iter()
        .map(|&present| {
            (present != 0).then(|| {
                physical += 1;
                physical - 1
            })
        })
        .collect();
    Ok(Some((block_size, map)))
}

/// Finds a disc image (`.iso`, `.gcm`, `.ciso`) in `rom/` next to the
/// executable, then `rom/` in the working directory.
pub fn find_image() -> Option<PathBuf> {
    let exe_rom = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("rom")));
    exe_rom
        .into_iter()
        .chain([PathBuf::from("rom")])
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flat_map(|entries| entries.flatten().map(|e| e.path()))
        .find(|p| {
            p.is_file()
                && p.extension().and_then(|e| e.to_str()).is_some_and(|e| {
                    matches!(e.to_ascii_lowercase().as_str(), "iso" | "gcm" | "ciso")
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Builds a tiny synthetic GCM: header, a 2-section DOL, and an FST with
    /// `opening.bnr`, `foresta.rel.szs` and `dir/sub/a.bin`.
    fn synthetic_gcm(game_id: &[u8; 6]) -> Vec<u8> {
        let mut img = vec![0u8; 0x4000];
        img[..6].copy_from_slice(game_id);
        img[0x1C..0x20].copy_from_slice(&GC_MAGIC.to_be_bytes());
        let put = |img: &mut Vec<u8>, at: usize, v: u32| {
            img[at..at + 4].copy_from_slice(&v.to_be_bytes())
        };

        // DOL at 0x1000: text0 at file offset 0x100 size 0x20, data0 at 0x200 size 0x40.
        let dol = 0x1000;
        put(&mut img, 0x420, dol as u32);
        put(&mut img, dol, 0x100);
        put(&mut img, dol + 0x90, 0x20);
        put(&mut img, dol + 0x1C, 0x200);
        put(&mut img, dol + 0xAC, 0x40);

        // FST at 0x2000. Entries: root, opening.bnr, foresta.rel.szs, dir/, sub/, a.bin.
        let names = b"opening.bnr\0foresta.rel.szs\0dir\0sub\0a.bin\0";
        let entries: [(u8, u32, u32, u32); 6] = [
            (1, 0, 0, 6),
            (0, 0, 0x3000, 4),
            (0, 12, 0x3010, 3),
            (1, 28, 0, 6),
            (1, 32, 3, 6),
            (0, 36, 0x3020, 5),
        ];
        let fst = 0x2000;
        for (i, (kind, name, a, b)) in entries.iter().enumerate() {
            let at = fst + i * 12;
            put(&mut img, at, (u32::from(*kind) << 24) | name);
            put(&mut img, at + 4, *a);
            put(&mut img, at + 8, *b);
        }
        let strings = fst + entries.len() * 12;
        img[strings..strings + names.len()].copy_from_slice(names);
        put(&mut img, 0x424, fst as u32);
        put(&mut img, 0x428, (entries.len() * 12 + names.len()) as u32);

        img[0x3000..0x3004].copy_from_slice(b"BNR1");
        img[0x3010..0x3013].copy_from_slice(b"REL");
        img[0x3020..0x3025].copy_from_slice(b"hello");
        img
    }

    /// Wraps a raw image as CISO with the given block size; all-zero blocks are omitted.
    fn to_ciso(raw: &[u8], block: usize) -> Vec<u8> {
        let mut out = vec![0u8; CISO_HEADER_SIZE];
        out[..4].copy_from_slice(b"CISO");
        out[4..8].copy_from_slice(&(block as u32).to_le_bytes());
        for (i, chunk) in raw.chunks(block).enumerate() {
            if chunk.iter().any(|&b| b != 0) {
                out[CISO_MAP_OFFSET + i] = 1;
                out.extend_from_slice(chunk);
                out.resize(out.len() + block - chunk.len(), 0);
            }
        }
        out
    }

    fn check(mut disc: Disc<Cursor<Vec<u8>>>) {
        assert_eq!(disc.game_id(), GAME_ID_NTSC_U);
        let paths: Vec<_> = disc.files().iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, ["opening.bnr", "foresta.rel.szs", "dir/sub/a.bin"]);
        assert_eq!(disc.read_file("/DIR/sub/A.bin").unwrap(), b"hello");
        assert_eq!(disc.read_file("foresta.rel.szs").unwrap(), b"REL");
        assert!(matches!(
            disc.read_file("nope"),
            Err(DiscError::FileNotFound(_))
        ));
        let dol = disc.read_dol().unwrap();
        assert_eq!(dol.len(), 0x240);
        assert_eq!(&dol[..4], &0x100u32.to_be_bytes());
    }

    #[test]
    fn reads_raw_gcm() {
        check(Disc::open(Cursor::new(synthetic_gcm(GAME_ID_NTSC_U))).unwrap());
    }

    #[test]
    fn reads_ciso_with_sparse_blocks() {
        let ciso = to_ciso(&synthetic_gcm(GAME_ID_NTSC_U), 0x800);
        check(Disc::open(Cursor::new(ciso)).unwrap());
    }

    #[test]
    fn rejects_other_games_and_non_discs() {
        let err = Disc::open(Cursor::new(synthetic_gcm(b"GAFP01")))
            .err()
            .unwrap();
        assert!(matches!(err, DiscError::WrongGame(id) if &id == b"GAFP01"));
        assert!(matches!(
            Disc::open(Cursor::new(vec![0u8; 0x4000])),
            Err(DiscError::NotGameCube)
        ));
        assert!(matches!(
            Disc::open(Cursor::new(vec![1u8; 16])),
            Err(DiscError::NotGameCube)
        ));
    }

    #[test]
    fn rejects_corrupt_fst() {
        let mut img = synthetic_gcm(GAME_ID_NTSC_U);
        img[0x2000 + 8..0x2000 + 12].copy_from_slice(&0xFFFFu32.to_be_bytes());
        assert!(matches!(
            Disc::open(Cursor::new(img)),
            Err(DiscError::Corrupt(_))
        ));
    }

    /// Needs the player's own disc; set BEVY_CROSSING_ISO to an absolute path, then
    /// `cargo test -p disc -- --ignored`.
    #[test]
    #[ignore]
    fn real_disc() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut disc = Disc::open_path(Path::new(&path)).unwrap();
        let rel = disc.read_file("foresta.rel.szs").unwrap();
        let rel = crate::decode_yaz0(&rel).expect("foresta.rel.szs should be Yaz0");
        assert!(rel.len() > 1024 * 1024);
        let dol = disc.read_dol().unwrap();
        eprintln!(
            "files={} rel={} dol={}",
            disc.files().len(),
            rel.len(),
            dol.len()
        );
        // AC keeps most code in foresta.rel; main.dol is small but non-trivial.
        assert!(dol.len() > 64 * 1024);

        for name in ["forest_1st.arc", "forest_2nd.arc"] {
            let arc = crate::Rarc::parse(disc.read_file(name).unwrap()).expect(name);
            eprintln!(
                "{name}: {} files, fgdata={:?}",
                arc.files().len(),
                arc.file("fgdata.bin").map(|f| f.len())
            );
        }
        let rom = crate::RomImage::new(dol, rel).unwrap();
        let n = rom.rel_pointer_count();
        eprintln!("rel data pointers={n}");
        assert!(n > 1000);

        // Offsets cross-checked against rustimal-crossing pc_assets.c.
        let cat = crate::AssetCatalog::builtin(&rom);
        eprintln!("catalog={}", cat.len());
        let font = cat.get("FONT_nes_tex_font1").unwrap();
        assert_eq!(
            (font.source, font.offset, font.size),
            (crate::RomSource::Rel, 0x4E1A40, 0x6000)
        );
        let koi = cat.get("obj_e_koinobori_b_pal").unwrap();
        assert_eq!((koi.source, koi.offset), (crate::RomSource::Rel, 0x355760));
    }
}
