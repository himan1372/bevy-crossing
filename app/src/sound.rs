//! Sound: the game's own sequence player (`rustimal_logic::nas_seq`)
//! fed from the disc, streamed to Bevy's audio output. The engine runs on
//! the audio thread, 200 samples an update, so it keeps the original's
//! 240 updates a second; the game talks to it with the original's audio
//! commands (`rustimal_logic::AudioPort`).
//!
//! What plays so far: the effect player (sequence 242 on group 0, as
//! `Na_InitAudio`) and the town's hourly music (`Sou_BgmStart` for music
//! 1-24, fine weather's subtrack mutes) and trigger effects (`se`, as
//! `Sou_TrgStart`). Indoor music and voices come next.

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
                (hourly_music, play_effects).run_if(in_state(AppState::InGame)),
            );
    }
}
