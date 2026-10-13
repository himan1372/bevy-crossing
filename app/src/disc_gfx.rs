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
                let Some(mut gfx) = self.gfx_at(at) else {
                    break;
                };
                if gfx.opcode() == op::G_DL && gfx.addr() >> 24 != 0 {
                    // A call into a runtime segment (e.g. the ocean's wave
                    // list in segment 12) that game code would fill. The
                    // interpreter ends the task on a missing list, so
                    // skip the call instead.
                    gfx = Gfx::new(0, 0);
                }
                list.push(gfx);
                at += GFX_SIZE;
                if matches!(gfx.opcode(), op::G_TRIN | op::G_TRIN_INDEPEND) {
                    // The face data continues in the following words; don't
                    // read it as commands (an index byte can look like
                    // G_ENDDL and cut the list short).
                    for _ in 1..trin_words(gfx, |i| self.gfx_at(at + (i as u32 - 1) * GFX_SIZE)) {
                        let Some(data) = self.gfx_at(at) else { break };
                        list.push(data);
                        at += GFX_SIZE;
                    }
                    continue;
                }
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

/// How many `Gfx` words a `G_TRIN` run occupies (itself included), the
/// way `Emu64::h_trin` consumes them: the first word carries 3 faces, each
/// later word 3 (7-bit indices, bit 0 of w1 set) or 4 (5-bit indices).
/// `word(i)` returns the run's i-th word.
fn trin_words(first: Gfx, word: impl Fn(usize) -> Option<Gfx>) -> usize {
    let mut faces = (((first.w0 >> 17) & 0x7F) + 1) as usize;
    let mut words = 0;
    while faces > 0 {
        let per_word = if words == 0 {
            3
        } else {
            match word(words) {
                Some(g) if g.w1 & 1 == 1 => 3,
                Some(_) => 4,
                None => break,
            }
        };
        faces = faces.saturating_sub(per_word);
        words += 1;
    }
    words.max(1)
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
    /// The highest upward-facing surface over model point (`x`, `z`), as
    /// a stand-in for a room's collision heights.
    pub fn height_at(&self, x: f32, z: f32) -> Option<f32> {
        let mut best: Option<f32> = None;
        for tri in self
            .parts
            .iter()
            .flat_map(|p| p.vertices.as_chunks::<3>().0)
        {
            let [a, b, c] = [tri[0].pos, tri[1].pos, tri[2].pos];
            let d = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2]);
            if d.abs() < 1e-6 {
                continue;
            }
            let u = ((b[2] - c[2]) * (x - c[0]) + (c[0] - b[0]) * (z - c[2])) / d;
            let v = ((c[2] - a[2]) * (x - c[0]) + (a[0] - c[0]) * (z - c[2])) / d;
            let w = 1.0 - u - v;
            if u < 0.0 || v < 0.0 || w < 0.0 {
                continue;
            }
            let y = u * a[1] + v * b[1] + w * c[1];
            best = Some(best.map_or(y, |h| h.max(y)));
        }
        best
    }

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

/// `DecodedVtx::mtx` of a vertex loaded by the `i`-th entry list before
/// any G_MTX: `LIST_TAG | i`, so callers can place each list (e.g. on its
/// skeleton joint).
pub const LIST_TAG: u32 = 0xA000_0000;

/// Geometry mode of the game's model setup list:
/// G_ZBUFFER | G_SHADE | G_CULL_BACK | G_FOG | G_LIGHTING | G_SHADING_SMOOTH.
const SETUP_GEOMETRY_MODE: u32 = geo::G_ZBUFFER
    | geo::G_SHADE
    | geo::G_CULL_BACK
    | geo::G_FOG
    | geo::G_LIGHTING
    | geo::G_SHADING_SMOOTH;

/// G_SETCOMBINE words (low 24 bits of w0, w1) for TEXEL0 * SHADE (alpha
/// TEXEL0), then COMBINED * PRIMITIVE (alpha too).
const SETUP_COMBINE: (u32, u32) = (0x12_7E03, 0xFF0F_F3FF);

/// Interprets display lists at REL offsets `entries`, in order, sharing
/// render state (e.g. a model's `_mat_model` list, then its `_gfx_model`).
pub fn run_display_lists(rom: &RomImage, entries: &[u32]) -> TriangleSink {
    run_display_lists_with_prim(rom, entries, [255; 4], 255)
}

/// As [`run_display_lists`], starting from the primitive colour and LOD
/// fraction game code would set before calling these lists (e.g. a house
/// actor tinting its light model).
pub fn run_display_lists_with_prim(
    rom: &RomImage,
    entries: &[u32],
    prim: [u8; 4],
    prim_lod: u8,
) -> TriangleSink {
    run_lists(rom, entries, prim, prim_lod, [0; 4])
}

/// As [`run_display_lists_with_prim`], also with the environment colour
/// the draw code sets first (`gDPSetEnvColor`).
pub fn run_display_lists_with_colors(
    rom: &RomImage,
    entries: &[u32],
    prim: [u8; 4],
    prim_lod: u8,
    env: [u8; 4],
) -> TriangleSink {
    run_lists(rom, entries, prim, prim_lod, env)
}

/// The wet-sand colour field drawing puts in segment C for beach acres
/// (`aFD_SetBeachColorOpaSegment`): `beach_env_color` swings between
/// (144,128,96) and (186,164,124) with the waves; this is its midpoint.
const BEACH_ENV_COLOR: [u8; 4] = [165, 146, 110, 255];

/// Runs an acre's ground lists. Field drawing sets the environment colour
/// (segment C) before them; the lists' beach combiners blend toward it.
pub fn run_ground_lists(rom: &RomImage, entries: &[u32]) -> TriangleSink {
    run_lists(rom, entries, [255; 4], 255, BEACH_ENV_COLOR)
}

/// Runs lists after game code has loaded palettes into TLUT slots
/// (`gDPLoadTLUT_Dolphin(slot, ...)`), e.g. a villager's body palette in
/// slot 15: `tluts` = (slot, REL offset).
pub fn run_display_lists_with_tluts(
    rom: &RomImage,
    entries: &[u32],
    tluts: &[(usize, u32)],
) -> TriangleSink {
    run_lists_full(rom, entries, [255; 4], 255, [0; 4], tluts)
}

fn run_lists(
    rom: &RomImage,
    entries: &[u32],
    prim: [u8; 4],
    prim_lod: u8,
    env: [u8; 4],
) -> TriangleSink {
    run_lists_full(rom, entries, prim, prim_lod, env, &[])
}

fn run_lists_full(
    rom: &RomImage,
    entries: &[u32],
    prim: [u8; 4],
    prim_lod: u8,
    env: [u8; 4],
    tluts: &[(usize, u32)],
) -> TriangleSink {
    let mem = RelGfx::new(rom);
    let mut emu = Emu64::new();
    for seg in 1..16 {
        emu.segments.set(seg, UNRESOLVED);
    }
    // Game code usually sets the primitive colour before calling a model;
    // callers pass what it would be (white for most models).
    emu.state.prim_color = Rgba {
        r: prim[0],
        g: prim[1],
        b: prim[2],
        a: prim[3],
    };
    emu.state.prim_lod = prim_lod;
    emu.state.env_color = Rgba {
        r: env[0],
        g: env[1],
        b: env[2],
        a: env[3],
    };
    // The render setup the game draws models after (`m_rcp.c`
    // `_texture_z_light_fog_prim` = `z_gsCPModeSet_Data[2]`): lighting on,
    // combiner G_CC_MODULATEIDECALA then G_CC_MODULATEIA_PRIM2. Lists that
    // don't set their own mode (e.g. trees) inherit it.
    emu.state.geometry_mode = SETUP_GEOMETRY_MODE;
    emu.state.combine_l = SETUP_COMBINE.0;
    emu.state.combine_h = SETUP_COMBINE.1;
    for &(slot, addr) in tluts {
        if let Some(t) = emu.state.tluts.get_mut(slot) {
            t.addr = addr;
            t.count = 16;
        }
    }
    let mut sink = TriangleSink::default();
    for (i, &entry) in entries.iter().enumerate() {
        // Vertices remember which list loaded them (see `LIST_TAG`) until
        // a list loads a matrix of its own.
        emu.cur_mtx = LIST_TAG | i as u32;
        let lists = mem.lists(entry);
        emu.taskstart(&mem, &mut sink, &lists, entry);
        sink.aborted |= emu.task_aborted;
    }
    sink
}

/// Runtime memory game code would have filled, rebuilt from REL data:
/// .bss buffers (e.g. the season's ground textures), and segments a draw
/// routine points at other data (e.g. the player's face and clothing from
/// `forest_1st.arc`). Segment 0 is the REL itself.
#[derive(Default, Clone)]
pub struct Segments {
    /// (bss offset, length, REL offset).
    bss: Vec<(u32, u32, u32)>,
    /// (segment, bytes) the draw code sets with `gSPSegment`.
    external: Vec<(u8, std::sync::Arc<[u8]>)>,
}

impl Segments {
    /// Fills the .bss buffer at `bss_offset` with REL data at `rel`.
    pub fn set_bss(&mut self, bss_offset: u32, len: u32, rel: u32) {
        self.bss.push((bss_offset, len, rel));
    }

    /// Points segment `seg` at `bytes` (`gSPSegment(seg, bytes)`).
    pub fn set_segment(&mut self, seg: u8, bytes: std::sync::Arc<[u8]>) {
        self.external.retain(|(s, _)| *s != seg);
        self.external.push((seg, bytes));
    }

    /// The bytes at a segmented, bss-tagged or REL address.
    pub fn bytes<'a>(&'a self, rel: &'a [u8], addr: u32) -> Option<&'a [u8]> {
        let seg = (addr >> 24) as u8;
        if let Some((_, b)) = self.external.iter().find(|(s, _)| *s == seg) {
            return b.get((addr & 0x00FF_FFFF) as usize..);
        }
        rel.get(self.resolve(addr)? as usize..)
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
    let data = segments.bytes(rel, key.img_addr)?;
    // A palette the lists loaded wins over the caller's. Disc palettes
    // checked so far (ground, `_dol`) are RGB5A3.
    let loaded = match key.tlut_addr {
        a if a != 0 && a < UNRESOLVED => segments.bytes(rel, a),
        _ => None,
    };
    let fallback = palette.and_then(|a| rel.get(a as usize..));
    // Palettes hold at most 256 entries; the decoder bounds-checks.
    let tlut = loaded
        .or(fallback)
        .map(|t| (&t[..t.len().min(512)], TlutFormat::Rgb5A3));
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

        // Calls into runtime segments are skipped, not fatal: the ocean
        // template calls segment 12 before its own geometry.
        assert!(run_display_lists(&rom, &[asset("grd_s_o_1_model")]).triangle_count() > 0);
        // Ground templates must cover their whole acre (G_TRIN data words
        // used to be misread as commands, leaving holes).
        let mut lists = vec![asset("grd_s_c1_r1_1_model")];
        lists.push(asset("grd_s_c1_r1_1_modelT"));
        let sink = run_display_lists(&rom, &lists);
        let area: f32 = sink
            .parts
            .iter()
            .flat_map(|p| p.vertices.chunks(3))
            .map(|t| {
                let (a, b, c) = (t[0].pos, t[1].pos, t[2].pos);
                ((b[0] - a[0]) * (c[2] - a[2]) - (c[0] - a[0]) * (b[2] - a[2])).abs() / 2.0
            })
            .sum();
        assert!(
            area / (10240.0 * 10240.0) > 0.97,
            "coverage {}",
            area / (10240.0 * 10240.0)
        );
    }
}
