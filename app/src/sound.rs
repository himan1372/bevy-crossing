//! Sound: the game's own sequence player (`rustimal_logic::nas_seq`)
//! fed from the disc, streamed to Bevy's audio output. The engine runs on
//! the audio thread at the sample rate and update size from `nas_seq`;
//! the game talks to it with the original's audio
//! commands (`rustimal_logic::AudioPort`).
//!
//! What plays so far: the effect player (sequence 242 on group 0, as
//! `Na_InitAudio`) and the town's hourly music (`Sou_BgmStart` for music
//! 1-24, fine weather's subtrack mutes) and trigger effects (`se`, as
//! `Sou_TrgStart`). Villager Animalese follows the typed dialogue through
//! `na_voice`; indoor music and special voice modes remain to do.

use std::sync::{Arc, Mutex};

use bevy::audio::{AddAudioSource, ChannelCount, Decodable, SampleRate, Source};
use bevy::prelude::*;
use rustimal_logic::nas_seq::{Engine, SAMPLES_PER_UPDATE, SAMPLE_RATE};
use rustimal_logic::{audio_group, AudioParam, AudioPort, BGM_MUTE_TABLE_FINE};

use crate::game_disc::GameDisc;
use crate::sound_rom::{self, DiscSound};
use crate::states::AppState;

/// Group roles (`game64.c_inc`).
const SE_GROUP: u8 = audio_group::SE as u8;
const BGM0_GROUP: u8 = audio_group::BGM0 as u8;
const BGM1_GROUP: u8 = audio_group::BGM1 as u8;
/// The effect player's sequence (`Na_InitAudio`).
const SE_SEQUENCE: u8 = 242;

/// The running engine, shared with the audio thread.
#[derive(Resource, Clone)]
pub struct Sound {
    engine: Arc<Mutex<Engine>>,
    /// `SEQ_TABLE`: music id -> sequence.
    seq_table: Arc<[u8]>,
}

impl Sound {
    /// Sends the game's audio commands to the engine.
    pub fn send(&self, cmds: &[AudioPort]) {
        if let Ok(mut e) = self.engine.lock() {
            for c in cmds {
                e.process(c);
            }
        }
    }
}

fn cmd(opcode: u8, arg0: u8, arg1: u8, arg2: u8, param: AudioParam) -> AudioPort {
    AudioPort {
        opcode,
        arg0,
        arg1,
        arg2,
        param,
    }
}

/// The engine as a Bevy audio asset: its decoder pulls samples from it.
#[derive(Asset, TypePath)]
struct NasStream {
    engine: Arc<Mutex<Engine>>,
}

struct NasSource {
    engine: Arc<Mutex<Engine>>,
    buf: Vec<f32>,
    at: usize,
}

impl Iterator for NasSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.at >= self.buf.len() {
            self.buf.iter_mut().for_each(|x| *x = 0.0);
            if let Ok(mut e) = self.engine.lock() {
                e.update(&mut self.buf);
            }
            self.at = 0;
        }
        let x = self.buf[self.at];
        self.at += 1;
        Some(x.clamp(-1.0, 1.0))
    }
}

impl Source for NasSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        ChannelCount::new(2).unwrap()
    }

    fn sample_rate(&self) -> SampleRate {
        SampleRate::new(SAMPLE_RATE).unwrap()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl Decodable for NasStream {
    type Decoder = NasSource;

    fn decoder(&self) -> NasSource {
        NasSource {
            engine: self.engine.clone(),
            buf: vec![0.0; SAMPLES_PER_UPDATE * 2],
            at: SAMPLES_PER_UPDATE * 2,
        }
    }
}

/// Builds the engine from the disc and starts the effect player.
fn start_sound(
    mut commands: Commands,
    disc: Res<GameDisc>,
    mut streams: ResMut<Assets<NasStream>>,
) {
    let Some(rom) = disc.sound.clone() else {
        warn!("no sound: the disc's sound data wasn't read");
        return;
    };
    let Some(tables) = sound_rom::tables(&disc.rom, &disc.catalog) else {
        warn!("no sound: the engine's tables weren't found");
        return;
    };
    let seq_table: Arc<[u8]> = disc
        .catalog
        .get("SEQ_TABLE")
        .and_then(|a| disc.rom.slice(a.source, a.offset, a.size))
        .map(Arc::from)
        .unwrap_or_else(|| Arc::from(Vec::new()));
    let engine = Arc::new(Mutex::new(Engine::new(
        Arc::new(DiscSound::new(rom)),
        tables,
    )));
    let sound = Sound { engine, seq_table };
    sound.send(&[cmd(0x82, SE_GROUP, SE_SEQUENCE, 0, AudioParam::S32(0))]);
    let handle = streams.add(NasStream {
        engine: sound.engine.clone(),
    });
    commands.spawn((Name::new("sound engine"), AudioPlayer::<NasStream>(handle)));
    commands.insert_resource(sound);
    commands.insert_resource(Bgm::default());
    info!("sound engine started");
}

/// `NA_SE_*` ids the game uses so far (`audio_defs.h`).
pub mod se_id {
    pub const PAGE_OKURI: u16 = 0x0B;
    pub const MENU_PAUSE: u16 = 0x30;
    pub const ITEM_GET: u16 = 0x40;
    pub const TOOL_FURI: u16 = 0x5A;
    pub const AMI_HIT: u16 = 0x5C;
}

static SE_QUEUE: Mutex<Vec<u16>> = Mutex::new(Vec::new());
/// Typed characters to voice: (speaker's spec, a, b, c phonemes).
static VOICE_QUEUE: Mutex<Vec<(u8, u8, u8, u8)>> = Mutex::new(Vec::new());

/// Voices one typed character (`sAdo_VoiceSe` from the message window):
/// the phonemes of the character and the two after it.
pub fn voice(spec: u8, [a, b, c]: [u8; 3]) {
    if let Ok(mut q) = VOICE_QUEUE.lock() {
        q.push((spec, a, b, c));
    }
}

/// `Na_SpecChange` + `Na_VoiceSe` + `Sou_VoiceMake` for queued characters.
fn play_voices(
    sound: Option<Res<Sound>>,
    mut voice: Local<rustimal_logic::na_voice::Voice>,
    mut seed: Local<u32>,
) {
    let queued: Vec<_> = VOICE_QUEUE
        .lock()
        .map(|mut q| std::mem::take(&mut *q))
        .unwrap_or_default();
    let Some(sound) = sound else { return };
    for request in queued {
        play_voice(&sound, &mut voice, &mut seed, request);
    }
}

/// Shared by live typing and the disc-backed voice comparison probe.
fn play_voice(
    sound: &Sound,
    voice: &mut rustimal_logic::na_voice::Voice,
    seed: &mut u32,
    (spec, a, b, c): (u8, u8, u8, u8),
) {
    use rustimal_logic::na_voice::VoiceOut;
    let voice_group = audio_group::VOICE as u8;
    if let Some(seq) = voice.spec_change(spec) {
        sound.send(&[
            cmd(0x83, voice_group, 0, 0, AudioParam::S32(0)),
            cmd(0x82, voice_group, seq, 0, AudioParam::S32(0)),
        ]);
    }
    let mut rand = || {
        *seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
        (*seed >> 16) as u8
    };
    for out in voice.voice_se(a, b, c, 32, &mut rand) {
        match out {
            VoiceOut::Bebe => se(rustimal_logic::na_voice::SE_BEBE),
            VoiceOut::Start { slot, phoneme } => {
                let v = voice.slots[slot as usize];
                sound.send(&[
                    cmd(0x01, voice_group, slot, 0, AudioParam::F32(v.volume())),
                    cmd(0x04, voice_group, slot, 0, AudioParam::F32(v.pitch())),
                    cmd(0x03, voice_group, slot, 0, AudioParam::S8(64)),
                    cmd(0x06, voice_group, slot, 0, AudioParam::S8(phoneme as i8)),
                ]);
            }
        }
    }
}

/// Plays a trigger effect (`sAdo_SysTrgStart`), from anywhere.
pub fn se(id: u16) {
    if let Ok(mut q) = SE_QUEUE.lock() {
        q.push(id);
    }
}

/// `Sou_TrgStart`, simplified: effects take the six trigger subtracks in
/// turn; each gets full volume, normal pitch and centre pan, then its id
/// in ports 0 (low byte) and 1 (high nibble) for sequence 242 to play.
fn play_effects(sound: Option<Res<Sound>>, mut slot: Local<u8>) {
    let ids: Vec<u16> = SE_QUEUE
        .lock()
        .map(|mut q| std::mem::take(&mut *q))
        .unwrap_or_default();
    let Some(sound) = sound else { return };
    for id in ids {
        let s = *slot;
        *slot = (s + 1) % 6;
        sound.send(&[
            cmd(0x01, SE_GROUP, s, 0, AudioParam::F32(1.0)),
            cmd(0x04, SE_GROUP, s, 0, AudioParam::F32(1.0)),
            cmd(0x03, SE_GROUP, s, 0, AudioParam::S8(64)),
            cmd(
                0x06,
                SE_GROUP,
                s,
                0,
                AudioParam::S8((id & 0xFF) as u8 as i8),
            ),
            cmd(
                0x06,
                SE_GROUP,
                s,
                1,
                AudioParam::S8(((id >> 8) & 0xF) as i8),
            ),
        ]);
    }
}

/// `sou_now_bgm_num` and `sou_now_bgm_handle`.
#[derive(Resource)]
struct Bgm {
    playing: u8,
    handle: u8,
}

impl Default for Bgm {
    fn default() -> Self {
        Bgm {
            playing: 0,
            handle: BGM1_GROUP,
        }
    }
}

/// The town's music for the hour (music id = hour + 1), switched as
/// `Sou_BgmStart` does: the other group fades the old song out while the
/// new one starts with fine weather's mutes.
fn hourly_music(sound: Option<Res<Sound>>, bgm: Option<ResMut<Bgm>>) {
    let (Some(sound), Some(mut bgm)) = (sound, bgm) else {
        return;
    };
    let id = crate::clock::now().hour + 1;
    if bgm.playing == id {
        return;
    }
    let Some(&seq) = sound.seq_table.get(id as usize) else {
        return;
    };
    let old = bgm.handle;
    let new = if old == BGM0_GROUP {
        BGM1_GROUP
    } else {
        BGM0_GROUP
    };
    let fade = if bgm.playing == 0 { 0 } else { 60 };
    let mask = BGM_MUTE_TABLE_FINE[(id - 1) as usize];
    sound.send(&[
        cmd(0x83, old, 0, 0, AudioParam::S32(fade)),
        cmd(0x83, new, 0, 0, AudioParam::S32(0)),
        // Sou_BgmTenkiConv (fine): mute the hour's masked subtracks.
        cmd(0x90, new, 0, 0, AudioParam::U16(0xFFFF)),
        cmd(0x08, new, 0xFF, 0, AudioParam::S8(0)),
        cmd(0x90, new, 0, 0, AudioParam::U16(mask)),
        cmd(0x08, new, 0xFF, 0, AudioParam::S8(1)),
        cmd(0x82, new, seq, 0, AudioParam::S32(0)),
    ]);
    info!("music {id} (sequence {seq}) on group {new}");
    bgm.playing = id;
    bgm.handle = new;
}

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<NasStream>()
            .add_systems(
                OnEnter(AppState::InGame),
                start_sound.run_if(resource_exists::<GameDisc>),
            )
            .add_systems(
                Update,
                (hourly_music, play_effects, play_voices).run_if(in_state(AppState::InGame)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exercises the same command path as typed dialogue, including
    /// switching away from a voice and back. Disc-derived WAVs stay in temp.
    #[test]
    #[ignore = "requires BEVY_CROSSING_ISO"]
    fn probe_villager_voice_types() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut disc = disc::Disc::open_path(std::path::Path::new(&path)).unwrap();
        let rom = disc::RomImage::from_disc(&mut disc).unwrap();
        let catalog = disc::AssetCatalog::builtin(&rom);
        let data = sound_rom::SoundRom::read(&mut disc, &rom, &catalog).unwrap();
        let tables = sound_rom::tables(&rom, &catalog).unwrap();
        let sound = Sound {
            engine: Arc::new(Mutex::new(Engine::new(
                Arc::new(DiscSound::new(Arc::new(data))),
                tables,
            ))),
            seq_table: Arc::from(Vec::new()),
        };
        let out = std::env::temp_dir().join("bevy-crossing-voices");
        std::fs::create_dir_all(&out).unwrap();
        let phonemes: Vec<_> = b"Hello there! How are you doing today?"
            .iter()
            .copied()
            .map(rustimal_logic::na_voice::phoneme_of)
            .collect();
        let mut voice = rustimal_logic::na_voice::Voice::default();
        let mut seed = 0;
        let mut clips = Vec::new();
        for (pass, looks) in [2, 4, 0, 2].into_iter().enumerate() {
            let spec = rustimal_logic::na_voice::villager_spec(looks);
            let (sequence, _) = rustimal_logic::na_voice::voice_sequence(spec).unwrap();
            let mut clip = Vec::new();
            for (i, &a) in phonemes.iter().enumerate() {
                let b = phonemes.get(i + 1).copied().unwrap_or(0xFF);
                let c = phonemes.get(i + 2).copied().unwrap_or(0xFF);
                play_voice(&sound, &mut voice, &mut seed, (spec, a, b, c));
                // Normal typing: 30 characters per second.
                for _ in 0..8 {
                    let mut buf = vec![0.0; SAMPLES_PER_UPDATE * 2];
                    sound.engine.lock().unwrap().update(&mut buf);
                    clip.extend(buf);
                }
            }
            for _ in 0..120 {
                let mut buf = vec![0.0; SAMPLES_PER_UPDATE * 2];
                sound.engine.lock().unwrap().update(&mut buf);
                clip.extend(buf);
            }
            let rms = (clip.iter().map(|x| x * x).sum::<f32>() / clip.len() as f32).sqrt();
            let peak = clip.iter().fold(0.0_f32, |m, x| m.max(x.abs()));
            let file = out.join(format!("{pass}_spec{spec}_seq{sequence}.wav"));
            std::fs::write(&file, sound_rom::wav_stereo(&clip, SAMPLE_RATE)).unwrap();
            println!(
                "spec {spec}, sequence {sequence}: rms {rms:.5}, peak {peak:.5}, {}",
                file.display()
            );
            assert!(rms > 0.001, "voice spec {spec} is silent");
            assert!(clip.iter().all(|x| x.is_finite()));
            clips.push(clip);
        }
        for (a, b) in [(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)] {
            let difference = clips[a]
                .iter()
                .zip(&clips[b])
                .map(|(x, y)| (x - y).abs())
                .sum::<f32>()
                / clips[a].len() as f32;
            println!("passes {a}/{b}: mean difference {difference:.5}");
            assert!(
                difference > 0.001,
                "different voice types produced the same output"
            );
        }
    }
}
