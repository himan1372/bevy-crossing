//! GameCube (GX) texture decoding to plain RGBA8.
//!
//! GX textures are stored in tiles ("blocks") of a fixed size per format,
//! row-major by block, then row-major within a block. Colour-indexed
//! formats (C4/C8) look colours up in a palette (TLUT).

/// GX texture formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GxFormat {
    I4,
    I8,
    IA4,
    IA8,
    Rgb565,
    Rgb5A3,
    Rgba8,
    C4,
    C8,
    Cmpr,
}

/// GX palette entry formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TlutFormat {
    IA8,
    Rgb565,
    Rgb5A3,
}

impl GxFormat {
    /// Block width, height (texels) and bits per texel.
    fn block(self) -> (usize, usize, usize) {
        match self {
            Self::I4 | Self::C4 | Self::Cmpr => (8, 8, 4),
            Self::I8 | Self::IA4 | Self::C8 => (8, 4, 8),
            Self::IA8 | Self::Rgb565 | Self::Rgb5A3 => (4, 4, 16),
            Self::Rgba8 => (4, 4, 32),
        }
    }

    /// Bytes a `width` x `height` texture occupies (padded to whole blocks).
    pub fn byte_size(self, width: usize, height: usize) -> usize {
        let (bw, bh, bpp) = self.block();
        width.div_ceil(bw) * bw * height.div_ceil(bh) * bh * bpp / 8
    }
}

fn be16(b: &[u8], i: usize) -> u16 {
    u16::from_be_bytes([b[i], b[i + 1]])
}

fn rgb565(c: u16) -> [u8; 4] {
    let r = ((c >> 11) & 0x1F) as u8;
    let g = ((c >> 5) & 0x3F) as u8;
    let b = (c & 0x1F) as u8;
    [r << 3 | r >> 2, g << 2 | g >> 4, b << 3 | b >> 2, 255]
}

fn rgb5a3(c: u16) -> [u8; 4] {
    if c & 0x8000 != 0 {
        let r = ((c >> 10) & 0x1F) as u8;
        let g = ((c >> 5) & 0x1F) as u8;
        let b = (c & 0x1F) as u8;
        [r << 3 | r >> 2, g << 3 | g >> 2, b << 3 | b >> 2, 255]
    } else {
        let a = ((c >> 12) & 0x7) as u8;
        let r = ((c >> 8) & 0xF) as u8;
        let g = ((c >> 4) & 0xF) as u8;
        let b = (c & 0xF) as u8;
        [r * 0x11, g * 0x11, b * 0x11, a << 5 | a << 2 | a >> 1]
    }
}

fn ia8(c: u16) -> [u8; 4] {
    let (a, i) = ((c >> 8) as u8, c as u8);
    [i, i, i, a]
}

fn palette_color(tlut: &[u8], fmt: TlutFormat, index: usize) -> [u8; 4] {
    if index * 2 + 2 > tlut.len() {
        return [255, 0, 255, 255]; // missing palette entry: loud magenta
    }
    let c = be16(tlut, index * 2);
    match fmt {
        TlutFormat::IA8 => ia8(c),
        TlutFormat::Rgb565 => rgb565(c),
        TlutFormat::Rgb5A3 => rgb5a3(c),
    }
}

/// Decodes `data` into `width * height` RGBA8 texels.
/// `tlut` is required for C4/C8 (ignored otherwise). Returns `None` if
/// `data` is too short.
pub fn decode(
    data: &[u8],
    fmt: GxFormat,
    width: usize,
    height: usize,
    tlut: Option<(&[u8], TlutFormat)>,
) -> Option<Vec<u8>> {
    if data.len() < fmt.byte_size(width, height) {
        return None;
    }
    let mut out = vec![0u8; width * height * 4];
    let (bw, bh, bpp) = fmt.block();
    let block_bytes = bw * bh * bpp / 8;
    let blocks_x = width.div_ceil(bw);
    let mut put = |x: usize, y: usize, px: [u8; 4]| {
        if x < width && y < height {
            let i = (y * width + x) * 4;
            out[i..i + 4].copy_from_slice(&px);
        }
    };
    let pal = |i: usize| match tlut {
        Some((t, f)) => palette_color(t, f, i),
        None => [255, 0, 255, 255],
    };

    for by in 0..height.div_ceil(bh) {
        for bx in 0..blocks_x {
            let blk = &data[(by * blocks_x + bx) * block_bytes..][..block_bytes];
            let (x0, y0) = (bx * bw, by * bh);
            if fmt == GxFormat::Cmpr {
                // Four 4x4 DXT1 sub-blocks in a 2x2 grid.
                for sub in 0..4 {
                    let s = &blk[sub * 8..sub * 8 + 8];
                    let (sx, sy) = (x0 + (sub % 2) * 4, y0 + (sub / 2) * 4);
                    let (c0, c1) = (be16(s, 0), be16(s, 2));
                    let (p0, p1) = (rgb565(c0), rgb565(c1));
                    let mix = |a: u8, b: u8, wa: u16, wb: u16| {
                        ((a as u16 * wa + b as u16 * wb) / (wa + wb)) as u8
                    };
                    let lerp = |wa, wb| {
                        [
                            mix(p0[0], p1[0], wa, wb),
                            mix(p0[1], p1[1], wa, wb),
                            mix(p0[2], p1[2], wa, wb),
                            255,
                        ]
                    };
                    let colors = if c0 > c1 {
                        [p0, p1, lerp(2, 1), lerp(1, 2)]
                    } else {
                        [p0, p1, lerp(1, 1), [0, 0, 0, 0]]
                    };
                    for row in 0..4 {
                        let bits = s[4 + row];
                        for col in 0..4 {
                            let idx = (bits >> (6 - col * 2)) & 3;
                            put(sx + col, sy + row, colors[idx as usize]);
                        }
                    }
                }
                continue;
            }
            for ty in 0..bh {
                for tx in 0..bw {
                    let t = ty * bw + tx;
                    let px = match fmt {
                        GxFormat::I4 | GxFormat::C4 => {
                            let v = (blk[t / 2] >> if t % 2 == 0 { 4 } else { 0 }) & 0xF;
                            if fmt == GxFormat::C4 {
                                pal(v as usize)
                            } else {
                                let i = v * 0x11;
                                [i, i, i, i]
                            }
                        }
                        GxFormat::I8 => [blk[t]; 4],
                        GxFormat::C8 => pal(blk[t] as usize),
                        GxFormat::IA4 => {
                            let (a, i) = ((blk[t] >> 4) * 0x11, (blk[t] & 0xF) * 0x11);
                            [i, i, i, a]
                        }
                        GxFormat::IA8 => ia8(be16(blk, t * 2)),
                        GxFormat::Rgb565 => rgb565(be16(blk, t * 2)),
                        GxFormat::Rgb5A3 => rgb5a3(be16(blk, t * 2)),
                        // AR pairs in the first 32 bytes, GB pairs in the next 32.
                        GxFormat::Rgba8 => [
                            blk[t * 2 + 1],
                            blk[32 + t * 2],
                            blk[32 + t * 2 + 1],
                            blk[t * 2],
                        ],
                        GxFormat::Cmpr => unreachable!("handled above"),
                    };
                    put(x0 + tx, y0 + ty, px);
                }
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_conversions() {
        assert_eq!(rgb565(0xFFFF), [255, 255, 255, 255]);
        assert_eq!(rgb565(0xF800), [255, 0, 0, 255]);
        assert_eq!(rgb5a3(0x83E0), [0, 255, 0, 255]); // opaque, green
        assert_eq!(rgb5a3(0x0F00), [255, 0, 0, 0]); // alpha 0, red
        assert_eq!(rgb5a3(0x7FFF), [255, 255, 255, 255]);
        assert_eq!(ia8(0x80FF), [255, 255, 255, 0x80]);
    }

    #[test]
    fn i8_block_order() {
        // 16x4 I8 = two 8x4 blocks side by side.
        let data: Vec<u8> = (0..64).collect();
        let px = decode(&data, GxFormat::I8, 16, 4, None).unwrap();
        let at = |x: usize, y: usize| px[(y * 16 + x) * 4];
        assert_eq!(at(0, 0), 0);
        assert_eq!(at(7, 0), 7);
        assert_eq!(at(0, 1), 8); // next row inside block 0
        assert_eq!(at(8, 0), 32); // first texel of block 1
    }

    #[test]
    fn c4_uses_palette_and_nibble_order() {
        let mut data = vec![0u8; 32];
        data[0] = 0x10; // texel (0,0) = 1, (1,0) = 0
        let tlut = [0x00, 0x00, 0x80, 0x1F]; // [transparent black, opaque blue]
        let px = decode(&data, GxFormat::C4, 8, 8, Some((&tlut, TlutFormat::Rgb5A3))).unwrap();
        assert_eq!(&px[0..4], &[0, 0, 255, 255]);
        assert_eq!(&px[4..8], &[0, 0, 0, 0]);
    }

    #[test]
    fn rgba8_split_halves() {
        let mut data = vec![0u8; 64];
        data[0..2].copy_from_slice(&[0x40, 0x10]); // A R
        data[32..34].copy_from_slice(&[0x20, 0x30]); // G B
        let px = decode(&data, GxFormat::Rgba8, 4, 4, None).unwrap();
        assert_eq!(&px[0..4], &[0x10, 0x20, 0x30, 0x40]);
    }

    #[test]
    fn cmpr_two_colour_block() {
        let mut data = vec![0u8; 32];
        data[0..4].copy_from_slice(&[0xF8, 0x00, 0x00, 0x1F]); // c0 red > c1 blue
        data[4] = 0b0001_1011; // row 0: idx 0,1,2,3
        let px = decode(&data, GxFormat::Cmpr, 8, 8, None).unwrap();
        assert_eq!(&px[0..4], &[255, 0, 0, 255]);
        assert_eq!(&px[4..8], &[0, 0, 255, 255]);
        assert_eq!(&px[8..12], &[170, 0, 85, 255]);
    }

    #[test]
    fn short_data_rejected() {
        assert!(decode(&[0; 31], GxFormat::C4, 8, 8, None).is_none());
        assert_eq!(GxFormat::C4.byte_size(64, 64), 0x800);
    }
}
