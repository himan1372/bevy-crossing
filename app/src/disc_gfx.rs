//! Runs the game's own display lists (from the player's disc) through the
//! translated `emu64` interpreter (`logic::graphics`) and collects the
//! triangles it emits.
//!
//! Addressing: the interpreter works on flat addresses. Here a flat address
//! is a byte offset into the decompressed `foresta.rel` (segment 0, base 0;
//! the REL is under 16 MiB so offsets fit the 24-bit segment field).
//! Pointers inside display lists aren't stored in the REL file itself; they
//! come from its relocation table, so every `Gfx` is patched as it's read.
//! Pointers into `main.dol` and runtime segments (1-15, set by game code)
//! are left unresolved for now; the interpreter skips those commands.

use std::collections::{HashMap, VecDeque};

use disc::gx_texture::{self, GxFormat, TlutFormat};
use disc::{RomImage, RomSource, Target};
use rustimal_logic::graphics::command::{dl_param, geo, op, Gfx};
use rustimal_logic::graphics::interpreter::{EmittedTri, Emu64, GfxMemory, GxBackend};
use rustimal_logic::graphics::state::{GbiState, Rgba, NUM_TLUTS};
use rustimal_logic::graphics::texture::GxTexFormat;
use rustimal_logic::graphics::vertex::{DecodedVtx, Vtx};

/// Flat address the interpreter can't resolve (outside the REL).
const UNRESOLVED: u32 = 0xF000_0000;
/// Tag for REL .bss offsets (runtime buffers), above every real segment.
pub const BSS_TAG: u32 = 0xE000_0000;
/// Stop runaway lists (no real list comes close).
const MAX_LIST_LEN: usize = 16 * 1024;
const GFX_SIZE: u32 = 8;
const VTX_SIZE: usize = 16;

/// `GfxMemory` over the decompressed REL.
pub struct RelGfx<'a> {
    rom: &'a RomImage,
}

impl<'a> RelGfx<'a> {
    pub fn new(rom: &'a RomImage) -> Self {
        Self { rom }
    }

    fn rel(&self) -> &[u8] {
        self.rom.bytes(RomSource::Rel)
    }

    /// The `Gfx` at a REL offset, with its address word relocated.
    fn gfx_at(&self, offset: u32) -> Option<Gfx> {
        let raw = self.rom.slice(RomSource::Rel, offset, GFX_SIZE)?;
        let w0 = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]);
        let w1 = match self.rom.pointer_at(RomSource::Rel, offset + 4) {
            Some(Target::File(RomSource::Rel, target)) => target,
            // Runtime memory (bss) the game fills, e.g. the ground texture
            // buffer: keep the offset so `Segments` can map it.
            Some(Target::NoData(addend)) if addend < 0x0100_0000 => BSS_TAG | addend,
            Some(_) => UNRESOLVED,
            None => u32::from_be_bytes([raw[4], raw[5], raw[6], raw[7]]),
        };
        Some(Gfx::new(w0, w1))
    }

    /// Reads the list at `entry` and every list it calls, keyed by offset.
    pub fn lists(&self, entry: u32) -> HashMap<u32, Vec<Gfx>> {
        let mut lists = HashMap::new();
        let mut queue = VecDeque::from([entry]);
        while let Some(start) = queue.pop_front() {
            if lists.contains_key(&start) {
                continue;
            }
            let mut list = Vec::new();
            let mut at = start;
            while list.len() < MAX_LIST_LEN {
                let Some(gfx) = self.gfx_at(at) else { break };
                list.push(gfx);
                at += GFX_SIZE;
                match gfx.opcode() {
                    op::G_ENDDL => break,
                    op::G_DL if gfx.addr() < UNRESOLVED && gfx.addr() != 0 => {
                        match gfx.param() {
                            dl_param::G_DL_PUSH => queue.push_back(gfx.addr()),
                            dl_param::G_DL_NOPUSH => {
                                queue.push_back(gfx.addr());
                                break; // branch: nothing after it runs
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            lists.insert(start, list);
        }
        lists
    }
}

impl GfxMemory for RelGfx<'_> {
    fn vertices(&self, flat_addr: u32, n: usize) -> Option<Vec<Vtx>> {
        if flat_addr == 0 {
            return None;
        }
        let start = flat_addr as usize;
        let raw = self.rel().get(start..start + n * VTX_SIZE)?;
        Some(
            raw.as_chunks::<VTX_SIZE>()
                .0
                .iter()
                .map(|b| read_vtx(b))
                .collect(),
        )
    }

    fn bytes(&self, flat_addr: u32, n: usize) -> Option<Vec<u8>> {
        if flat_addr == 0 {
            return None;
        }
        let start = flat_addr as usize;
        self.rel().get(start..start + n).map(<[u8]>::to_vec)
    }
}

/// N64 `Vtx`, big-endian: x y z flag (s16 each), s t (s16), then
/// r g b a, which doubles as nx ny nz a for lit geometry.
fn read_vtx(b: &[u8]) -> Vtx {
    let s16 = |i: usize| i16::from_be_bytes([b[i], b[i + 1]]);
    Vtx {
        x: s16(0),
        y: s16(2),
        z: s16(4),
        flag: b[7],
        s: s16(8),
        t: s16(10),
        nx: b[12] as i8,
        ny: b[13] as i8,
        nz: b[14] as i8,
        r: b[12],
        g: b[13],
        b: b[14],
        a: b[15],
    }
}

/// What a group of triangles is drawn with: tile 0's texture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TexKey {
    pub img_addr: u32,
    pub format: GxTexFormat,
    pub width: u16,
    pub height: u16,
    pub wrap_s: u8,
    pub wrap_t: u8,
    /// REL offset of a palette loaded by the list itself (0 = none).
    pub tlut_addr: u32,
}

/// Everything a group of triangles is drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PartKey {
    pub tex: Option<TexKey>,
    /// `G_SETCOMBINE` words (low 24 bits of w0, w1); (0, 0) = never set.
    pub combine: (u32, u32),
    pub prim: [u8; 4],
    pub prim_lod: u8,
    pub env: [u8; 4],
    /// `G_LIGHTING`: vertex colour bytes are normals, shade is lit.
    pub lit: bool,
    /// Two-cycle combiner (`G_CYC_2CYCLE`).
    pub two_cycle: bool,
}

/// Triangles sharing one [`PartKey`].
pub struct Part {
    pub key: PartKey,
    pub vertices: Vec<DecodedVtx>,
}

/// Collects emitted triangles, grouped by render state.
#[derive(Default)]
pub struct TriangleSink {
    pub parts: Vec<Part>,
    current: Option<PartKey>,
    pub gx_lists_skipped: usize,
    /// The interpreter hit its command budget (looping/malformed list).
    pub aborted: bool,
}

impl TriangleSink {
    pub fn triangle_count(&self) -> usize {
        self.parts.iter().map(|p| p.vertices.len() / 3).sum()
    }

    fn part(&mut self) -> &mut Part {
        let Some(key) = self.current else {
            unreachable!("set_state runs before every emit");
        };
        let i = match self.parts.iter().position(|p| p.key == key) {
            Some(i) => i,
            None => {
                self.parts.push(Part {
                    key,
                    vertices: Vec::new(),
                });
                self.parts.len() - 1
            }
        };
        &mut self.parts[i]
    }
}

impl GxBackend for TriangleSink {
    fn set_state(&mut self, state: &GbiState) {
        let tile = &state.tiles[0];
        // Image addresses stay segmented here; `Segments` resolves them
        // when the texture is decoded.
        let textured = tile.img_addr != 0 && tile.img_addr < UNRESOLVED;
        let tex = textured.then(|| TexKey {
            img_addr: tile.img_addr,
            format: tile.gx_format,
            width: tile.width,
            height: tile.height,
            wrap_s: tile.wrap_s,
            wrap_t: tile.wrap_t,
            tlut_addr: state.tluts[tile.tlut as usize % NUM_TLUTS].addr,
        });
        let rgba = |c: Rgba| [c.r, c.g, c.b, c.a];
        self.current = Some(PartKey {
            tex,
            combine: (state.combine_l & 0x00FF_FFFF, state.combine_h),
            prim: rgba(state.prim_color),
            prim_lod: state.prim_lod,
            env: rgba(state.env_color),
            lit: state.geometry_mode & geo::G_LIGHTING != 0,
            two_cycle: state.othermode.is_2cycle(),
        });
    }

    fn emit_triangle(&mut self, tri: EmittedTri) {
        self.part().vertices.extend_from_slice(&tri.v);
    }

    fn emit_quad(&mut self, v: [DecodedVtx; 4]) {
        self.part()
            .vertices
            .extend_from_slice(&[v[0], v[1], v[2], v[0], v[2], v[3]]);
    }

    fn emit_texrect(&mut self, _ulx: u16, _uly: u16, _lrx: u16, _lry: u16) {}

    fn gx_call_display_list(&mut self, _bytes: &[u8]) {
        self.gx_lists_skipped += 1;
    }

    fn cull_display_list(&mut self, _v0: u8, _vn: u8) -> bool {
        false
    }
}

/// Interprets display lists at REL offsets `entries`, in order, sharing
/// render state (e.g. a model's `_mat_model` list, then its `_gfx_model`).
pub fn run_display_lists(rom: &RomImage, entries: &[u32]) -> TriangleSink {
    let mem = RelGfx::new(rom);
    let mut emu = Emu64::new();
    for seg in 1..16 {
        emu.segments.set(seg, UNRESOLVED);
    }
    // Game code usually sets the primitive colour before calling a model;
    // start from white so models that rely on that aren't drawn black.
    emu.state.prim_color = Rgba {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    let mut sink = TriangleSink::default();
    for &entry in entries {
        let lists = mem.lists(entry);
        emu.taskstart(&mem, &mut sink, &lists, entry);
        sink.aborted |= emu.task_aborted;
    }
    sink
}

/// Runtime memory game code would have filled, rebuilt from REL data:
/// .bss buffers (e.g. the season's ground textures). Segment 0 is the REL
/// itself; other segments aren't modelled yet.
#[derive(Default, Clone)]
pub struct Segments {
    /// (bss offset, length, REL offset).
    bss: Vec<(u32, u32, u32)>,
}

impl Segments {
    /// Fills the .bss buffer at `bss_offset` with REL data at `rel`.
    pub fn set_bss(&mut self, bss_offset: u32, len: u32, rel: u32) {
        self.bss.push((bss_offset, len, rel));
    }

    /// Segmented (or bss-tagged) address → REL offset.
    pub fn resolve(&self, addr: u32) -> Option<u32> {
        if addr & 0xF000_0000 == BSS_TAG {
            let off = addr & 0x0FFF_FFFF;
            return self
                .bss
                .iter()
                .find(|&&(start, len, _)| off >= start && off - start < len)
                .map(|&(start, _, rel)| rel + (off - start));
        }
        (addr >> 24 == 0).then_some(addr)
    }
}

/// Decodes a part's texture to RGBA8. `palette` (REL offset) is used for
/// colour-indexed textures when the lists didn't load one themselves.
pub fn decode_texture(
    rom: &RomImage,
    key: &TexKey,
    palette: Option<u32>,
    segments: &Segments,
) -> Option<Vec<u8>> {
    let fmt = match key.format {
        GxTexFormat::I4 => GxFormat::I4,
        GxTexFormat::I8 => GxFormat::I8,
        GxTexFormat::IA4 => GxFormat::IA4,
        GxTexFormat::IA8 => GxFormat::IA8,
        GxTexFormat::RGB565 => GxFormat::Rgb565,
        GxTexFormat::RGB5A3 => GxFormat::Rgb5A3,
        GxTexFormat::RGBA8 => GxFormat::Rgba8,
        GxTexFormat::C4 => GxFormat::C4,
        GxTexFormat::C8 => GxFormat::C8,
        GxTexFormat::CMPR => GxFormat::Cmpr,
        _ => return None,
    };
    let (w, h) = (key.width as usize, key.height as usize);
    let rel = rom.bytes(RomSource::Rel);
    let data = rel.get(segments.resolve(key.img_addr)? as usize..)?;
    // A palette the lists loaded wins over the caller's. Disc palettes
    // checked so far (ground, `_dol`) are RGB5A3.
    let pal_addr = match key.tlut_addr {
        a if a != 0 && a < UNRESOLVED => segments.resolve(a).or(palette),
        _ => palette,
    };
    // Palettes hold at most 256 entries; the decoder bounds-checks.
    let tlut = pal_addr
        .and_then(|a| rel.get(a as usize..(a as usize + 512).min(rel.len())))
        .map(|t| (t, TlutFormat::Rgb5A3));
    gx_texture::decode(data, fmt, w, h, tlut)
}

#[cfg(test)]
mod tests {
    use super::*;
    use disc::{AssetCatalog, Disc};

    /// Needs the player's disc (absolute path in BEVY_CROSSING_ISO):
    /// `cargo test -p bevy-crossing disc_gfx -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn real_disc_models_emit_triangles() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut disc = Disc::open_path(std::path::Path::new(&path)).expect("open disc");
        let rom = RomImage::from_disc(&mut disc).expect("rom");
        let cat = AssetCatalog::builtin(&rom);
        let asset = |n: &str| cat.get(n).expect(n).offset;
        let sink = run_display_lists(
            &rom,
            &[
                asset("obj_f_tree2T_mat_model"),
                asset("obj_f_tree2T_gfx_model"),
            ],
        );
        assert_eq!(sink.triangle_count(), 7);
        let key = sink.parts[0].key.tex.expect("tree is textured");
        assert_eq!(key.img_addr, asset("obj_f_tree_young_tex"));
        assert_eq!(
            (key.width, key.height, key.format),
            (64, 64, GxTexFormat::C4)
        );
        let pal = asset("mFM_obj_tree_01_pal_dol");
        let rgba = decode_texture(&rom, &key, Some(pal), &Segments::default()).expect("decodes");
        assert_eq!(rgba.len(), 64 * 64 * 4);
    }
}
