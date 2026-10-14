//! The game's sound data from the disc: the audio tables in the DOL
//! (`AudioseqHeaderStart`, `AudiobankHeaderStart`, `AudiowaveHeaderStart`,
//! `AudiodataHeaderStart`, `AudiomapHeaderStart`) and the data they index
//! in `audiorom.img`. `rustimal_logic::nas_bank` reads banks and decodes
//! samples.

// Read at disc load; the player that uses it is audio step 2.
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use disc::{AssetCatalog, Disc, RomImage, RomSource, Target};
use rustimal_logic::nas_bank::{self, ArcEntry, Bank, BankInfo, Sample, MEDIUM_CART, MEDIUM_DISK};
use rustimal_logic::nas_seq::{self, Pcm, SoundData, Tables};

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

    /// Bank `id`'s raw bytes (its envelopes are read from them).
    fn bank_bytes(&self, id: usize) -> Option<&[u8]> {
        let (_, e) = nas_bank::resolve(&self.banks, id)?;
        self.bytes(BANK, &e)
    }

    /// Sequence `id`'s bytes.
    pub fn sequence(&self, id: usize) -> Option<&[u8]> {
        let (_, e) = nas_bank::resolve(&self.sequences, id)?;
        self.bytes(SEQ, &e)
    }
}

/// The engine's tables from the DOL (`audiotable.c`, `audioconst.c`).
pub fn tables(rom: &RomImage, catalog: &AssetCatalog) -> Option<Tables> {
    let bytes = |name: &str| {
        let a = catalog.get(name)?;
        rom.slice(a.source, a.offset, a.size)
    };
    let f32s = |b: &[u8]| -> Vec<f32> {
        b.as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_be_bytes(*c))
            .collect()
    };
    let arr = |v: Vec<f32>| -> Option<[f32; 256]> { v.get(..256)?.try_into().ok() };
    let pitch: [f32; 128] = f32s(bytes("PITCHTABLE")?).get(..128)?.try_into().ok()?;
    let stereo: [f32; 128] = f32s(bytes("StereoLeft")?).get(..128)?.try_into().ok()?;
    let wavemem = catalog.get("WAVEMEM_TABLE")?;
    let Some(Target::File(src, at)) = rom.pointer_at(wavemem.source, wavemem.offset + 2 * 4) else {
        return None;
    };
    let curve = rom.slice(src, at, 128)?;
    let vibrato: [i16; 64] =
        std::array::from_fn(|i| i16::from_be_bytes([curve[i * 2], curve[i * 2 + 1]]));
    let env = bytes("DEFAULT_ENV")?;
    Some(Tables {
        pitch,
        pcent: arr(f32s(bytes("PCENTTABLE")?))?,
        pcent2: arr(f32s(bytes("PCENTTABLE2")?))?,
        velocities: bytes("DEFAULT_VTABLE")?.get(..16)?.try_into().ok()?,
        gate_times: bytes("DEFAULT_GTABLE")?.get(..16)?.try_into().ok()?,
        default_env: nas_seq::read_envelope(env, 0),
        stereo,
        vibrato,
        main_script: bytes("BDB_SEQDATA")?.to_vec(),
    })
}

/// The disc's sound data for the engine: banks are turned into playable
/// ones (decoded samples, envelopes) the first time they are asked for.
pub struct DiscSound {
    rom: Arc<SoundRom>,
    banks: Mutex<HashMap<usize, Option<Arc<nas_seq::Bank>>>>,
    pcm: Mutex<HashMap<(u8, u8, u32), Arc<Pcm>>>,
}

impl DiscSound {
    pub fn new(rom: Arc<SoundRom>) -> Self {
        DiscSound {
            rom,
            banks: Mutex::new(HashMap::new()),
            pcm: Mutex::new(HashMap::new()),
        }
    }

    fn tuned(&self, info: &BankInfo, t: &nas_bank::Tuned) -> Option<nas_seq::Tuned> {
        let s = &t.sample;
        let wave = if s.medium == MEDIUM_DISK {
            info.wave_banks[1]
        } else {
            info.wave_banks[0]
        };
        let key = (wave, s.codec, s.offset);
        let cached = self.pcm.lock().ok()?.get(&key).cloned();
        let pcm = match cached {
            Some(p) => p,
            None => {
                let data = self.rom.pcm(info, s)?;
                // A sample that doesn't loop plays to its end.
                let end = if s.looping.end == 0 {
                    data.len() as u32
                } else {
                    s.looping.end
                };
                let p = Arc::new(Pcm {
                    data,
                    loop_start: s.looping.start,
                    loop_end: end,
                    loop_count: s.looping.count,
                });
                self.pcm.lock().ok()?.insert(key, p.clone());
                p
            }
        };
        Some(nas_seq::Tuned {
            pcm,
            tuning: t.tuning,
        })
    }

    fn build(&self, id: usize) -> Option<nas_seq::Bank> {
        let (info, bank) = self.rom.bank(id)?;
        let bytes = self.rom.bank_bytes(id)?;
        let env = |at: u32| nas_seq::read_envelope(bytes, at as usize);
        let tuned = |t: &Option<nas_bank::Tuned>| t.as_ref().and_then(|t| self.tuned(&info, t));
        Some(nas_seq::Bank {
            instruments: bank
                .instruments
                .iter()
                .map(|i| {
                    i.as_ref().map(|i| nas_seq::Instrument {
                        range: i.range,
                        decay: i.release,
                        envelope: env(i.envelope),
                        low: tuned(&i.low),
                        normal: tuned(&i.normal),
                        high: tuned(&i.high),
                    })
                })
                .collect(),
            drums: bank
                .drums
                .iter()
                .map(|d| {
                    let d = d.as_ref()?;
                    Some(nas_seq::Drum {
                        decay: d.release,
                        pan: d.pan,
                        sample: tuned(&d.sample)?,
                        envelope: env(d.envelope),
                    })
                })
                .collect(),
            sfx: bank.sfx.iter().map(tuned).collect(),
        })
    }
}

impl SoundData for DiscSound {
    fn sequence(&self, id: usize) -> Option<Vec<u8>> {
        self.rom.sequence(id).map(<[u8]>::to_vec)
    }

    fn bank(&self, id: usize) -> Option<Arc<nas_seq::Bank>> {
        if let Some(b) = self.banks.lock().ok()?.get(&id) {
            return b.clone();
        }
        let b = self.build(id).map(Arc::new);
        self.banks.lock().ok()?.insert(id, b.clone());
        b
    }

    fn map(&self) -> &[u8] {
        &self.rom.map
    }
}

/// A stereo 16-bit WAV file from interleaved floats.
pub fn wav_stereo(pcm: &[f32], rate: u32) -> Vec<u8> {
    let data = pcm.len() as u32 * 2;
    let mut w = Vec::with_capacity(44 + data as usize);
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + data).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&rate.to_le_bytes());
    w.extend_from_slice(&(rate * 4).to_le_bytes());
    w.extend_from_slice(&4u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&data.to_le_bytes());
    for s in pcm {
        w.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
    }
    w
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
        // The tempo commands (0xDD) at the start of a few songs' scripts.
        for id in [86usize, 95] {
            let q = s.sequence(id).unwrap();
            let tempos: Vec<u8> = q
                .windows(2)
                .take(64)
                .filter(|w| w[0] == 0xDD)
                .map(|w| w[1])
                .collect();
            println!(
                "seq {id}: first bytes {:02x?}, tempo commands {tempos:?}",
                &q[..24.min(q.len())]
            );
        }
        // Render a few sequences through the player.
        let t = tables(&rom, &catalog).expect("engine tables");
        let data = Arc::new(DiscSound::new(Arc::new(s)));
        for id in [1usize, 2, 3, 10, 20, 40] {
            let mut e = nas_seq::Engine::new(data.clone(), t.clone());
            if !e.start_sequence(0, id) {
                println!("seq {id}: could not start");
                continue;
            }
            let updates = 240 * 15;
            let mut all = Vec::with_capacity(updates * nas_seq::SAMPLES_PER_UPDATE * 2);
            for _ in 0..updates {
                let mut buf = vec![0.0f32; nas_seq::SAMPLES_PER_UPDATE * 2];
                e.update(&mut buf);
                all.extend_from_slice(&buf);
            }
            let rms = (all.iter().map(|x| x * x).sum::<f32>() / all.len() as f32).sqrt();
            let peak = all.iter().fold(0.0f32, |m, x| m.max(x.abs()));
            println!(
                "seq {id}: rms {rms:.3}, peak {peak:.2}, still playing {}",
                e.playing(0)
            );
            std::fs::write(
                out.join(format!("seq{id:03}.wav")),
                wav_stereo(&all, nas_seq::SAMPLE_RATE),
            )
            .unwrap();
        }
        assert!(banks > 0 && samples > 0);
    }
}
