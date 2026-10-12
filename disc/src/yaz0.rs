//! Yaz0 (`.szs`) decompression.

use crate::be32;

/// Decodes a Yaz0 stream. Returns `None` on a bad header or truncated data.
pub fn decode(src: &[u8]) -> Option<Vec<u8>> {
    if src.len() < 16 || src.get(..4)? != b"Yaz0" {
        return None;
    }
    let out_len = be32(&src[4..])? as usize;
    let mut dst = Vec::new();
    dst.try_reserve_exact(out_len).ok()?;
    let mut sp = 16;
    while dst.len() < out_len {
        let flags = *src.get(sp)?;
        sp += 1;
        for bit in (0..8).rev() {
            if dst.len() >= out_len {
                break;
            }
            let first = *src.get(sp)?;
            sp += 1;
            if flags & (1 << bit) != 0 {
                dst.push(first);
                continue;
            }
            let second = *src.get(sp)?;
            sp += 1;
            let distance = ((first as usize & 0x0F) << 8 | second as usize) + 1;
            let length = match first >> 4 {
                0 => {
                    let n = *src.get(sp)? as usize + 0x12;
                    sp += 1;
                    n
                }
                n => n as usize + 2,
            };
            let start = dst.len().checked_sub(distance)?;
            for i in 0..length.min(out_len - dst.len()) {
                dst.push(dst[start + i]);
            }
        }
    }
    Some(dst)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Small greedy Yaz0 encoder, only for building test data.
    pub(crate) fn encode(data: &[u8]) -> Vec<u8> {
        let mut out = b"Yaz0".to_vec();
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        out.extend_from_slice(&[0; 8]);
        let mut pos = 0;
        while pos < data.len() {
            let flag_at = out.len();
            out.push(0);
            for bit in (0..8).rev() {
                if pos >= data.len() {
                    break;
                }
                let (mut best_len, mut best_dist) = (0, 0);
                for dist in 1..=pos.min(0x1000) {
                    let mut len = 0;
                    while len < 0x111
                        && pos + len < data.len()
                        && data[pos + len] == data[pos - dist + len]
                    {
                        len += 1;
                    }
                    if len > best_len {
                        (best_len, best_dist) = (len, dist);
                    }
                }
                if best_len < 3 {
                    out[flag_at] |= 1 << bit;
                    out.push(data[pos]);
                    pos += 1;
                    continue;
                }
                let d = best_dist - 1;
                if best_len >= 0x12 {
                    out.extend_from_slice(&[(d >> 8) as u8, d as u8, (best_len - 0x12) as u8]);
                } else {
                    out.extend_from_slice(&[(((best_len - 2) << 4) | (d >> 8)) as u8, d as u8]);
                }
                pos += best_len;
            }
        }
        out
    }

    #[test]
    fn round_trips() {
        let mut data = b"abcabcabcabc hello hello hello ".to_vec();
        data.extend(std::iter::repeat_n(7u8, 300)); // long run: 3-byte form
        data.extend((0..2000u32).map(|i| (i * 31 % 251) as u8));
        let packed = encode(&data);
        assert!(packed.len() < data.len());
        assert_eq!(decode(&packed).unwrap(), data);
    }

    #[test]
    fn empty_and_literal_only() {
        assert_eq!(decode(&encode(&[])).unwrap(), Vec::<u8>::new());
        assert_eq!(decode(&encode(b"xy")).unwrap(), b"xy");
    }

    #[test]
    fn rejects_bad_input() {
        assert!(decode(b"nope").is_none());
        assert!(decode(b"Yaz1\0\0\0\x04\0\0\0\0\0\0\0\0\xff").is_none());
        let mut packed = encode(b"hello hello hello");
        packed.truncate(packed.len() - 1);
        assert!(decode(&packed).is_none());
        // Back-reference before the start of the output.
        assert!(decode(b"Yaz0\0\0\0\x04\0\0\0\0\0\0\0\0\x00\x10\x05").is_none());
    }
}
