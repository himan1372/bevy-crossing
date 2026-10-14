//! The game's sound data (`jaudio_NES`, the Zelda 64 "Nas" engine): the
//! audio tables (`ArcHeader`), sound banks (`voicetable`, `perctable`,
//! `percvoicetable`, `smzwavetable`) and sample decoding (VADPCM, as the
//! N64 SDK's reference decoder). Everything is big-endian bytes from the
//! disc: the tables live in the DOL, the data in `audiorom.img`.

/// `MEDIUM_*`.
pub const MEDIUM_RAM: u8 = 0;
pub const MEDIUM_DISK: u8 = 1;
pub const MEDIUM_CART: u8 = 2;

/// Sample codecs.
pub const CODEC_ADPCM: u8 = 0;
pub const CODEC_S8: u8 = 1;
pub const CODEC_S16_INMEMORY: u8 = 2;
pub const CODEC_SMALL_ADPCM: u8 = 3;
pub const CODEC_S16: u8 = 5;

fn be16(b: &[u8], at: usize) -> Option<i16> {
    Some(i16::from_be_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn be32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

/// `ArcEntry`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArcEntry {
    pub addr: u32,
    pub size: u32,
    pub medium: u8,
    pub cache: u8,
    pub param: [i16; 3],
}

/// `ArcHeader` + entries: the sequence, bank, wave or data table.
pub fn parse_table(b: &[u8]) -> Option<Vec<ArcEntry>> {
    let n = be16(b, 0)?.max(0) as usize;
    (0..n)
        .map(|i| {
            let at = 0x10 + i * 0x10;
            Some(ArcEntry {
                addr: be32(b, at)?,
                size: be32(b, at + 4)?,
                medium: *b.get(at + 8)?,
                cache: *b.get(at + 9)?,
                param: [be16(b, at + 10)?, be16(b, at + 12)?, be16(b, at + 14)?],
            })
        })
        .collect()
}

/// An entry of size 0 stands for another (`addr` is its index), as the
/// engine's `__Link_BankNum` follows.
pub fn resolve(table: &[ArcEntry], id: usize) -> Option<(usize, ArcEntry)> {
    let e = *table.get(id)?;
    if e.size == 0 {
        let real = e.addr as usize;
        Some((real, *table.get(real)?))
    } else {
        Some((id, e))
    }
}

/// `__SetVlute`: what a bank entry says about its contents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BankInfo {
    /// Sample banks for `MEDIUM_RAM` and `MEDIUM_DISK` samples.
    pub wave_banks: [u8; 2],
    pub instruments: usize,
    pub drums: usize,
    pub sfx: usize,
}

impl BankInfo {
    pub fn of(e: &ArcEntry) -> Self {
        BankInfo {
            wave_banks: [(e.param[0] >> 8) as u8, e.param[0] as u8],
            instruments: ((e.param[1] >> 8) & 0xFF) as usize,
            drums: (e.param[1] & 0xFF) as usize,
            sfx: e.param[2].max(0) as usize,
        }
    }
}

/// `adpcmloop`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Loop {
    pub start: u32,
    pub end: u32,
    pub count: u32,
    pub state: [i16; 16],
}

/// `smzwavetable`: a sample's description (offsets as on the disc).
#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub codec: u8,
    pub medium: u8,
    pub size: u32,
    /// Offset into the sample bank its medium picks.
    pub offset: u32,
    pub looping: Loop,
    /// `adpcmbook`: order, then `[predictor][order][8]` coefficients.
    pub order: usize,
    pub book: Vec<i16>,
}

/// `wtstr`: a sample and its tuning (playback rate over 32 kHz).
#[derive(Clone, Debug, PartialEq)]
pub struct Tuned {
    pub sample: Sample,
    pub tuning: f32,
}

/// `voicetable`: up to three samples split by key.
#[derive(Clone, Debug, PartialEq)]
pub struct Instrument {
    pub range: [u8; 2],
    pub release: u8,
    pub envelope: u32,
    pub low: Option<Tuned>,
    pub normal: Option<Tuned>,
    pub high: Option<Tuned>,
}

/// `perctable`.
#[derive(Clone, Debug, PartialEq)]
pub struct Drum {
    pub release: u8,
    pub pan: u8,
    pub sample: Option<Tuned>,
    pub envelope: u32,
}

/// One sound bank (a soundfont), parsed from its bytes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bank {
    pub instruments: Vec<Option<Instrument>>,
    pub drums: Vec<Option<Drum>>,
    /// `percvoicetable`: what sound-effect layers play.
    pub sfx: Vec<Option<Tuned>>,
}

fn sample_at(b: &[u8], at: usize) -> Option<Sample> {
    let bits = be32(b, at)?;
    let size = bits & 0xFF_FFFF;
    let codec = ((bits >> 28) & 7) as u8;
    let medium = ((bits >> 26) & 3) as u8;
    let offset = be32(b, at + 4)?;
    let lp = be32(b, at + 8)? as usize;
    let bk = be32(b, at + 12)? as usize;
    let count = be32(b, lp + 8)?;
    let mut state = [0i16; 16];
    if count != 0 {
        for (i, s) in state.iter_mut().enumerate() {
            *s = be16(b, lp + 16 + i * 2)?;
        }
    }
    let looping = Loop {
        start: be32(b, lp)?,
        end: be32(b, lp + 4)?,
        count,
        state,
    };
    let order = be32(b, bk)? as usize;
    let npred = be32(b, bk + 4)? as usize;
    if order > 8 || npred > 16 {
        return None;
    }
    let book = (0..order * npred * 8)
        .map(|i| be16(b, bk + 8 + i * 2))
        .collect::<Option<Vec<_>>>()?;
    Some(Sample {
        codec,
        medium,
        size,
        offset,
        looping,
        order,
        book,
    })
}

fn tuned_at(b: &[u8], at: usize) -> Option<Tuned> {
    let wt = be32(b, at)? as usize;
    if wt == 0 {
        return None;
    }
    Some(Tuned {
        sample: sample_at(b, wt)?,
        tuning: f32::from_bits(be32(b, at + 4)?),
    })
}

impl Bank {
    /// A bank's bytes: offsets to the drum and sfx lists, then one offset
    /// per instrument (all from the bank's start).
    pub fn parse(b: &[u8], info: &BankInfo) -> Option<Bank> {
        let drums_at = be32(b, 0)? as usize;
        let sfx_at = be32(b, 4)? as usize;
        let instruments = (0..info.instruments)
            .map(|i| {
                let at = be32(b, 8 + i * 4)? as usize;
                if at == 0 {
                    return Some(None);
                }
                Some(Some(Instrument {
                    range: [*b.get(at + 1)?, *b.get(at + 2)?],
                    release: *b.get(at + 3)?,
                    envelope: be32(b, at + 4)?,
                    low: tuned_at(b, at + 8),
                    normal: tuned_at(b, at + 0x10),
                    high: tuned_at(b, at + 0x18),
                }))
            })
            .collect::<Option<Vec<_>>>()?;
        let drums = if drums_at == 0 {
            Vec::new()
        } else {
            (0..info.drums)
                .map(|i| {
                    let at = be32(b, drums_at + i * 4)? as usize;
                    if at == 0 {
                        return Some(None);
                    }
                    Some(Some(Drum {
                        release: *b.get(at)?,
                        pan: *b.get(at + 1)?,
                        sample: tuned_at(b, at + 4),
                        envelope: be32(b, at + 0xC)?,
                    }))
                })
                .collect::<Option<Vec<_>>>()?
        };
        let sfx = if sfx_at == 0 {
            Vec::new()
        } else {
            (0..info.sfx).map(|i| tuned_at(b, sfx_at + i * 8)).collect()
        };
        Some(Bank {
            instruments,
            drums,
            sfx,
        })
    }
}

/// The SDK decoder's per-predictor table: for output `k`, the weights on
/// `[order previous outputs, 8 innovations]`.
fn coef_table(book: &[i16], order: usize) -> Vec<[[i32; 16]; 8]> {
    let npred = book.len() / (order * 8);
    (0..npred)
        .map(|p| {
            let mut t = [[0i32; 16]; 8];
            for j in 0..order {
                for (k, row) in t.iter_mut().enumerate() {
                    row[j] = book[p * order * 8 + j * 8 + k] as i32;
                }
            }
            for k in 1..8 {
                t[k][order] = t[k - 1][order - 1];
            }
            t[0][order] = 1 << 11;
            for k in 1..8 {
                for j in k..8 {
                    t[j][k + order] = t[j - k][order];
                }
            }
            t
        })
        .collect()
}

/// `inner_product` with the SDK's rounding toward minus infinity.
fn inner(len: usize, a: &[i32; 16], b: &[i32; 16]) -> i32 {
    let out: i64 = (0..len).map(|j| a[j] as i64 * b[j] as i64).sum();
    let d = out >> 11;
    d.clamp(i16::MIN as i64, i16::MAX as i64) as i32
}

/// Decodes a sample's bytes (from its offset, `size` long) to 16-bit PCM.
pub fn decode(s: &Sample, data: &[u8]) -> Vec<i16> {
    match s.codec {
        CODEC_ADPCM | CODEC_SMALL_ADPCM if s.order > 0 && !s.book.is_empty() => {
            let small = s.codec == CODEC_SMALL_ADPCM;
            let frame = if small { 5 } else { 9 };
            let table = coef_table(&s.book, s.order);
            let order = s.order;
            let mut out = Vec::with_capacity(data.len() / frame * 16);
            let mut prev = [0i32; 16];
            for f in data.chunks_exact(frame) {
                let scale = 1i32 << (f[0] >> 4);
                let Some(t) = table.get((f[0] & 0xF) as usize) else {
                    break;
                };
                let mut ix = [0i32; 16];
                for (i, v) in ix.iter_mut().enumerate() {
                    let n = if small {
                        let b = f[1 + i / 4] >> (6 - 2 * (i % 4)) & 3;
                        if b >= 2 {
                            b as i32 - 4
                        } else {
                            b as i32
                        }
                    } else {
                        let b = (f[1 + i / 2] >> if i % 2 == 0 { 4 } else { 0 }) & 0xF;
                        if b >= 8 {
                            b as i32 - 16
                        } else {
                            b as i32
                        }
                    };
                    *v = n * scale;
                }
                let mut cur = [0i32; 16];
                for half in 0..2 {
                    let mut v = [0i32; 16];
                    for i in 0..order {
                        v[i] = if half == 0 {
                            prev[16 - order + i]
                        } else {
                            cur[8 - order + i]
                        };
                    }
                    for i in 0..8 {
                        v[order + i] = ix[half * 8 + i];
                    }
                    for i in 0..8 {
                        cur[half * 8 + i] = inner(order + 8, &t[i], &v);
                    }
                }
                out.extend(cur.iter().map(|&x| x as i16));
                prev = cur;
            }
            out
        }
        CODEC_S8 => data.iter().map(|&b| ((b as i8) as i16) << 8).collect(),
        CODEC_S16 | CODEC_S16_INMEMORY => data
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| i16::from_be_bytes(*c))
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_entries_and_references() {
        let mut b = vec![0u8; 0x30];
        b[1] = 2;
        b[0x10..0x14].copy_from_slice(&0x100u32.to_be_bytes());
        b[0x14..0x18].copy_from_slice(&0x40u32.to_be_bytes());
        b[0x18] = MEDIUM_CART;
        b[0x20..0x24].copy_from_slice(&0u32.to_be_bytes()); // size 0: see entry 0
        let t = parse_table(&b).unwrap();
        assert_eq!(t.len(), 2);
        assert_eq!(resolve(&t, 1).unwrap().0, 0);
    }

    #[test]
    fn adpcm_with_a_zero_book_is_just_the_scaled_nibbles() {
        // Order 2, one predictor, all-zero coefficients: output = innovation.
        let s = Sample {
            codec: CODEC_ADPCM,
            medium: MEDIUM_CART,
            size: 9,
            offset: 0,
            looping: Loop::default(),
            order: 2,
            book: vec![0; 16],
        };
        // scale 2^2, nibbles 1, -1, 7, -8, ...
        let frame = [0x20, 0x1F, 0x78, 0, 0, 0, 0, 0, 0];
        let pcm = decode(&s, &frame);
        assert_eq!(&pcm[..4], &[4, -4, 28, -32]);
        assert_eq!(pcm.len(), 16);
    }
}
