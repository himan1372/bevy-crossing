//! JSystem RARC archives (`forest_1st.arc`, `forest_2nd.arc`), which hold
//! runtime resources such as `fgdata.bin` (town object layouts).
//!
//! Layout: a 0x20-byte header ("RARC", sizes, data offset relative to
//! 0x20), an info block at 0x20 (node and entry tables, string table),
//! then the file data. Files are listed flat by name; directories are
//! skipped. A Yaz0-compressed archive is decompressed first.

use crate::be32;

/// One file in the archive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RarcFile {
    pub name: String,
    pub offset: usize,
    pub size: usize,
}

/// A parsed archive (owns the decompressed bytes).
pub struct Rarc {
    data: Vec<u8>,
    files: Vec<RarcFile>,
}

const ENTRY_SIZE: usize = 0x14;
const ENTRY_DIR: u16 = 0x0200;

fn be16(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

impl Rarc {
    /// Parses an archive, decompressing Yaz0 if needed.
    pub fn parse(bytes: Vec<u8>) -> Option<Self> {
        let data = if bytes.starts_with(b"Yaz0") {
            crate::decode_yaz0(&bytes)?
        } else {
            bytes
        };
        if data.get(..4)? != b"RARC" {
            return None;
        }
        let data_start = 0x20 + be32(data.get(0x0C..)?)? as usize;
        let info = 0x20;
        let entry_count = be32(data.get(info + 0x08..)?)? as usize;
        let entries = info + be32(data.get(info + 0x0C..)?)? as usize;
        let strings = info + be32(data.get(info + 0x14..)?)? as usize;
        let mut files = Vec::new();
        for i in 0..entry_count {
            let e = entries + i * ENTRY_SIZE;
            let kind = be16(&data, e + 4)?;
            if kind & ENTRY_DIR != 0 {
                continue;
            }
            let name_at = strings + be16(&data, e + 6)? as usize;
            let name_bytes = data.get(name_at..)?;
            let end = name_bytes.iter().position(|&b| b == 0)?;
            let offset = data_start + be32(data.get(e + 8..)?)? as usize;
            let size = be32(data.get(e + 12..)?)? as usize;
            if offset + size > data.len() {
                return None;
            }
            files.push(RarcFile {
                name: String::from_utf8_lossy(&name_bytes[..end]).into_owned(),
                offset,
                size,
            });
        }
        Some(Self { data, files })
    }

    pub fn files(&self) -> &[RarcFile] {
        &self.files
    }

    /// A file's bytes by name (case-insensitive).
    pub fn file(&self, name: &str) -> Option<&[u8]> {
        let f = self
            .files
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(name))?;
        Some(&self.data[f.offset..f.offset + f.size])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny archive: root dir entries ".", ".." and one file "a.bin".
    fn synthetic() -> Vec<u8> {
        let mut d = vec![0u8; 0x100];
        d[..4].copy_from_slice(b"RARC");
        let put32 =
            |d: &mut Vec<u8>, at: usize, v: u32| d[at..at + 4].copy_from_slice(&v.to_be_bytes());
        let put16 =
            |d: &mut Vec<u8>, at: usize, v: u16| d[at..at + 2].copy_from_slice(&v.to_be_bytes());
        put32(&mut d, 0x0C, 0xC0); // data at 0x20 + 0xC0 = 0xE0
        put32(&mut d, 0x28, 3); // entries
        put32(&mut d, 0x2C, 0x20); // entry table at 0x40
        put32(&mut d, 0x34, 0x80); // strings at 0xA0
                                   // entry 0: file "a.bin" at data+0x4, size 5
        put16(&mut d, 0x44, 0x1100);
        put16(&mut d, 0x46, 0);
        put32(&mut d, 0x48, 4);
        put32(&mut d, 0x4C, 5);
        // entries 1, 2: "." and ".." directories
        for i in 1..3 {
            put16(&mut d, 0x40 + i * 0x14 + 4, 0x0200);
            put16(&mut d, 0x40 + i * 0x14 + 6, 6);
        }
        d[0xA0..0xA8].copy_from_slice(b"a.bin\0.\0");
        d[0xE4..0xE9].copy_from_slice(b"hello");
        d
    }

    #[test]
    fn lists_files_and_reads_them() {
        let arc = Rarc::parse(synthetic()).unwrap();
        assert_eq!(arc.files().len(), 1);
        assert_eq!(arc.file("A.BIN"), Some(&b"hello"[..]));
        assert!(arc.file("missing").is_none());
    }

    #[test]
    fn rejects_non_archives() {
        assert!(Rarc::parse(b"nope".to_vec()).is_none());
        let mut bad = synthetic();
        bad[0x4C..0x50].copy_from_slice(&0xFFFFu32.to_be_bytes()); // size past end
        assert!(Rarc::parse(bad).is_none());
    }
}
