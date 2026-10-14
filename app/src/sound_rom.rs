//! The game's sound data from the disc: the audio tables in the DOL
//! (`AudioseqHeaderStart`, `AudiobankHeaderStart`, `AudiowaveHeaderStart`,
//! `AudiodataHeaderStart`, `AudiomapHeaderStart`) and the data they index
//! in `audiorom.img`. `rustimal_logic::nas_bank` reads banks and decodes
//! samples.

// Read at disc load; the player that uses it is audio step 2.
#![allow(dead_code)]

use disc::{AssetCatalog, Disc, RomImage, RomSource};
use rustimal_logic::nas_bank::{self, ArcEntry, Bank, BankInfo, Sample, MEDIUM_CART, MEDIUM_DISK};

/// `SEQUENCE_TABLE`, `BANK_TABLE`, `WAVE_TABLE` in the data table.
const SEQ: usize = 0;
const BANK: usize = 1;
const WAVE: usize = 2;

pub struct SoundRom {
    pub sequences: Vec<ArcEntry>,
    pub banks: Vec<ArcEntry>,
    pub waves: Vec<ArcEntry>,
    /// Where each table's data starts in `audiorom.img`.
    base: [u32; 3],
    /// `AudiomapHeaderStart`: the banks each sequence uses.
    pub map: Vec<u8>,
    rom: Vec<u8>,
}

impl SoundRom {
    pub fn read<R: std::io::Read + std::io::Seek>(
        disc: &mut Disc<R>,
        rom: &RomImage,
        catalog: &AssetCatalog,
    ) -> Option<Self> {
        let table = |name: &str| {
            let a = catalog.get(name)?;
            nas_bank::parse_table(rom.slice(a.source, a.offset, a.size)?)
        };
        let data = table("AudiodataHeaderStart")?;
        let map = catalog.get("AudiomapHeaderStart")?;
        Some(SoundRom {
            sequences: table("AudioseqHeaderStart")?,
            banks: table("AudiobankHeaderStart")?,
            waves: table("AudiowaveHeaderStart")?,
            base: [
                data.get(SEQ)?.addr,
                data.get(BANK)?.addr,
                data.get(WAVE)?.addr,
            ],
            map: rom.slice(RomSource::Dol, map.offset, map.size)?.to_vec(),
            rom: disc.read_file("audiorom.img").ok()?,
        })
    }

    /// An entry's bytes (`MEDIUM_CART` entries are offsets into the table's
    /// data in `audiorom.img`).
    fn bytes(&self, table: usize, e: &ArcEntry) -> Option<&[u8]> {
        let at = if e.medium == MEDIUM_CART {
            self.base[table] + e.addr
        } else {
            e.addr
        } as usize;
        self.rom.get(at..at + e.size as usize)
    }

    /// Sound bank `id`, parsed.
    pub fn bank(&self, id: usize) -> Option<(BankInfo, Bank)> {
        let (_, e) = nas_bank::resolve(&self.banks, id)?;
        // The counts are on the entry asked for (a reference keeps its own).
        let info = BankInfo::of(self.banks.get(id)?);
        let info = if info.instruments + info.drums + info.sfx == 0 {
            BankInfo::of(&e)
        } else {
            info
        };
        Some((info, Bank::parse(self.bytes(BANK, &e)?, &info)?))
    }

    /// A sample of a bank, as 16-bit PCM (`__WaveTouch`: `MEDIUM_RAM`
    /// samples sit in the bank's first sample bank, `MEDIUM_DISK` ones in
    /// its second).
    pub fn pcm(&self, info: &BankInfo, s: &Sample) -> Option<Vec<i16>> {
        let wave = if s.medium == MEDIUM_DISK {
            info.wave_banks[1]
        } else {
            info.wave_banks[0]
        };
        let (_, e) = nas_bank::resolve(&self.waves, wave as usize)?;
        let bank = self.bytes(WAVE, &e)?;
        let at = s.offset as usize;
        Some(nas_bank::decode(s, bank.get(at..at + s.size as usize)?))
    }

    /// Sequence `id`'s bytes.
    pub fn sequence(&self, id: usize) -> Option<&[u8]> {
        let (_, e) = nas_bank::resolve(&self.sequences, id)?;
        self.bytes(SEQ, &e)
    }
}

/// A mono 16-bit WAV file.
pub fn wav(pcm: &[i16], rate: u32) -> Vec<u8> {
    let data = pcm.len() as u32 * 2;
    let mut w = Vec::with_capacity(44 + data as usize);
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + data).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&rate.to_le_bytes());
    w.extend_from_slice(&(rate * 2).to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&data.to_le_bytes());
    for s in pcm {
        w.extend_from_slice(&s.to_le_bytes());
    }
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Probe (needs the disc): every bank parses and its samples decode to
    /// sane levels; a few sound effects go to the temp folder to listen to.
    #[test]
    #[ignore]
    fn probe_sound_rom() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut disc = Disc::open_path(std::path::Path::new(&path)).unwrap();
        let rom = RomImage::from_disc(&mut disc).unwrap();
        let catalog = AssetCatalog::builtin(&rom);
        let s = SoundRom::read(&mut disc, &rom, &catalog).expect("sound tables");
        println!(
            "{} sequences, {} banks, {} sample banks, audiorom {} bytes, data bases {:x?}",
            s.sequences.len(),
            s.banks.len(),
            s.waves.len(),
            s.rom.len(),
            s.base
        );
        let (mut banks, mut samples, mut silent, mut failed) = (0, 0, 0, 0);
        let mut ratios: Vec<f64> = Vec::new();
        for id in 0..s.banks.len() {
            let Some((info, bank)) = s.bank(id) else {
                println!("bank {id} failed to parse ({:?})", s.banks[id]);
                continue;
            };
            banks += 1;
            let tuned = bank
                .instruments
                .iter()
                .flatten()
                .flat_map(|i| [&i.low, &i.normal, &i.high])
                .chain(bank.drums.iter().flatten().map(|d| &d.sample))
                .chain(bank.sfx.iter().map(Some).map(|t| t.unwrap()))
                .flatten();
            for t in tuned {
                match s.pcm(&info, &t.sample) {
                    Some(p) if p.iter().any(|&x| x.unsigned_abs() > 256) => {
                        samples += 1;
                        let amp = p.iter().map(|&x| (x as f64).abs()).sum::<f64>();
                        let step = p
                            .windows(2)
                            .map(|w| (w[1] as f64 - w[0] as f64).abs())
                            .sum::<f64>();
                        ratios.push(step / amp.max(1.0));
                    }
                    Some(_) => silent += 1,
                    None => failed += 1,
                }
            }
            if id < 4 {
                println!(
                    "bank {id}: {} instruments, {} drums, {} sfx, wave banks {:?}",
                    bank.instruments.iter().flatten().count(),
                    bank.drums.iter().flatten().count(),
                    bank.sfx.iter().flatten().count(),
                    info.wave_banks
                );
            }
        }
        ratios.sort_by(f64::total_cmp);
        let q = |f: f64| ratios[((ratios.len() - 1) as f64 * f) as usize];
        println!(
            "step/amplitude: 10% {:.2}, median {:.2}, 90% {:.2} (white noise ~1.1)",
            q(0.1),
            q(0.5),
            q(0.9)
        );
        println!("{banks} banks parsed; samples: {samples} with sound, {silent} near-silent, {failed} unreadable");
        let out = std::env::temp_dir().join("bevy-crossing-sfx");
        std::fs::create_dir_all(&out).unwrap();
        if let Some((info, bank)) = s.bank(0) {
            for (i, ins) in bank.instruments.iter().enumerate() {
                let Some(t) = ins.as_ref().and_then(|i| i.normal.as_ref()) else {
                    continue;
                };
                if let Some(p) = s.pcm(&info, &t.sample) {
                    // Smoothness: real sound moves little sample to sample.
                    let amp =
                        p.iter().map(|&x| (x as f64).abs()).sum::<f64>() / p.len().max(1) as f64;
                    let step = p
                        .windows(2)
                        .map(|w| (w[1] as f64 - w[0] as f64).abs())
                        .sum::<f64>()
                        / p.len().max(2) as f64;
                    if i < 12 {
                        println!(
                            "bank 0 instrument {i}: {} samples, codec {}, tuning {:.3}, mean |x| {amp:.0}, mean step {step:.0}",
                            p.len(),
                            t.sample.codec,
                            t.tuning
                        );
                    }
                    let rate = (32000.0 * t.tuning) as u32;
                    std::fs::write(out.join(format!("bank0_inst{i:03}.wav")), wav(&p, rate))
                        .unwrap();
                }
            }
        }
        println!("wrote bank 0's instruments to {}", out.display());
        assert!(banks > 0 && samples > 0);
    }
}
