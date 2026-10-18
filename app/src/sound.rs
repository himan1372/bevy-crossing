//! Sound: the game's own sequence player (`rustimal_logic::nas_seq`)
//! fed from the disc, streamed to Bevy's audio output. The engine runs on
//! the audio thread at the sample rate and update size from `nas_seq`;
//! the game talks to it with the original's audio
//! commands (`rustimal_logic::AudioPort`).
//!
//! What plays so far: the effect player (sequence 242 on group 0, as
//! `Na_InitAudio`) and the town's hourly music (`Sou_BgmStart` for music
//! 1-24, fine weather's subtrack mutes), shop themes and trigger effects (`se`, as
//! `Sou_TrgStart`). Villager Animalese follows the typed dialogue through
//! `na_voice`; player houses are quiet without a furniture player. Special
//! voice modes and weather simulation remain to do.

use std::sync::{Arc, Mutex};

use bevy::audio::{AddAudioSource, ChannelCount, Decodable, SampleRate, Source};
use bevy::prelude::*;
use rustimal_logic::nas_seq::{Engine, SAMPLES_PER_UPDATE, SAMPLE_RATE};
use rustimal_logic::scene_music as music_policy;
use rustimal_logic::{audio_group, AudioParam, AudioPort};

use crate::game_disc::GameDisc;
use crate::sound_rom::{self, DiscSound};
use crate::states::AppState;

/// Group roles (`game64.c_inc`).
const SE_GROUP: u8 = audio_group::SE as u8;
const BGM0_GROUP: u8 = audio_group::BGM0 as u8;
const BGM1_GROUP: u8 = audio_group::BGM1 as u8;
/// The effect player's sequence (`Na_InitAudio`).
const SE_SEQUENCE: u8 = 242;

#[derive(Component)]
struct SoundStream;

/// Successful quick-load clears transient sounds before submitting this frame.
#[derive(Message)]
pub(crate) struct ResetAudio;

/// The running engine, shared with the audio thread.
#[derive(Resource, Clone)]
pub struct Sound {
    engine: Arc<Mutex<Engine>>,
    /// `SEQ_TABLE`: music id -> sequence.
    seq_table: Arc<[u8]>,
}

impl Sound {
    pub(crate) fn ambience(
        &self,
        levels: [Option<rustimal_logic::ambience::Level>; rustimal_logic::ambience::VOICES],
    ) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.set_ambience(levels);
        }
    }

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
    commands.spawn((
        SoundStream,
        Name::new("sound engine"),
        AudioPlayer::<NasStream>(handle),
    ));
    commands.insert_resource(sound);
    commands.insert_resource(Bgm::default());
    if std::env::var_os("BEVY_CROSSING_AUDIO_DIAGNOSTICS").is_some_and(|v| v == "1") {
        commands.insert_resource(AudioDiagnostics::default());
    }
    info!("sound engine started");
}

/// `NA_SE_*` ids the game uses so far (`audio_defs.h`).
pub mod se_id {
    pub const CURSOR: u16 = 0x01;
    // mTG_move_cancel: inventory/catalog exits use NA_SE_31.
    pub const MENU_EXIT: u16 = 0x31;
    pub const PANEL_OPEN: u16 = 0x8004;
    pub const PANEL_CLOSE: u16 = 0x8005;
    pub const CHOICE_OPEN: u16 = 0x0C;
    pub const CHOICE_CONFIRM: u16 = 0x0D;
    // Retail SE_REGISTER is MONO(NA_SE_REGISTER), using subtrack 14.
    pub const REGISTER: u16 = 0x1050;
    pub const MAILBOX_FLAG_UP: u16 = 0x433;
    pub const MAILBOX_FLAG_DOWN: u16 = 0x434;
    pub const PAGE_OKURI: u16 = 0x0B;
    pub const MENU_PAUSE: u16 = 0x30;
    pub const ITEM_GET: u16 = 0x40;
    pub const TOOL_FURI: u16 = 0x5A;
    pub const NET_CATCH: u16 = 0x5D;
    pub const POCKET_RUSTLE: u16 = 0x69;
    pub const ROD_CAST: u16 = 0x109;
    pub const ROD_BACK: u16 = 0x10A;
    pub const BOBBER_LAND: u16 = 0x10B;
    pub const BOBBER_OUT: u16 = 0x10C;
    pub const FISH_LIFT: u16 = 0x11A;
    pub const ROD_AIR: u16 = 0x445;
}

#[derive(Clone, Copy)]
enum EffectOrigin {
    Center,
    World { position: Vec3, attenuate: bool },
}

struct EffectRequest {
    id: u16,
    volume: f32,
    reverb: u8,
    origin: EffectOrigin,
}

static SE_QUEUE: Mutex<Vec<EffectRequest>> = Mutex::new(Vec::new());
/// Commands from the reveal system, consumed after it in the same frame.
enum VoiceRequest {
    Begin {
        spec: u8,
        voice_id: u16,
    },
    Character {
        phonemes: [u8; 3],
        scale: u8,
        default_melody: bool,
    },
    Status(u8),
    End,
}
static VOICE_QUEUE: Mutex<Vec<VoiceRequest>> = Mutex::new(Vec::new());

fn queue_voice(request: VoiceRequest) {
    if let Ok(mut queue) = VOICE_QUEUE.lock() {
        queue.push(request);
    }
}

pub(crate) fn voice_begin(spec: u8, voice_id: u16) {
    queue_voice(VoiceRequest::Begin { spec, voice_id });
}
pub(crate) fn voice_character(phonemes: [u8; 3], scale: u8, default_melody: bool) {
    queue_voice(VoiceRequest::Character {
        phonemes,
        scale,
        default_melody,
    });
}
pub(crate) fn voice_status(status: u8) {
    queue_voice(VoiceRequest::Status(status));
}
pub(crate) fn voice_end() {
    queue_voice(VoiceRequest::End);
}

#[derive(Resource, Default)]
struct VoicePlayback {
    voice: rustimal_logic::na_voice::Voice,
    voice_id: u16,
    seed: u32,
    frames: f32,
}

fn voice_random(seed: &mut u32) -> impl FnMut() -> u8 + '_ {
    move || {
        *seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
        (*seed >> 16) as u8
    }
}

fn select_voice(sound: &Sound, voice: &mut rustimal_logic::na_voice::Voice, spec: u8) {
    if let Some(seq) = voice.spec_change(spec) {
        let group = audio_group::VOICE as u8;
        sound.send(&[
            cmd(0x83, group, 0, 0, AudioParam::S32(0)),
            cmd(0x82, group, seq, 0, AudioParam::S32(0)),
        ]);
    }
}

/// Na_GameFrame's deferred second syllable follows the port's 30 Hz game
/// cadence. New text replaces pending lookahead; page/close cancels it.
fn play_voices(time: Res<Time>, sound: Option<Res<Sound>>, mut playback: ResMut<VoicePlayback>) {
    let queued = VOICE_QUEUE
        .lock()
        .map(|mut q| std::mem::take(&mut *q))
        .unwrap_or_default();
    let Some(sound) = sound else { return };
    let VoicePlayback {
        voice,
        voice_id,
        seed,
        frames,
    } = &mut *playback;
    for request in queued {
        match request {
            VoiceRequest::Begin { spec, voice_id: id } => {
                voice.begin_message();
                *voice_id = id;
                *frames = 0.0;
                select_voice(&sound, voice, spec);
            }
            VoiceRequest::Character {
                phonemes: [a, b, c],
                scale,
                default_melody,
            } => {
                voice.speaker_id = if default_melody { 0 } else { *voice_id };
                let out = voice.voice_se(a, b, c, scale, &mut voice_random(seed));
                apply_voice(&sound, voice, voice.speaker_id, out);
            }
            VoiceRequest::Status(status) => voice.set_status(status),
            VoiceRequest::End => voice.end_message(),
        }
    }
    *frames += time.delta_secs() * 30.0;
    while *frames >= 1.0 {
        *frames -= 1.0;
        let out = voice.game_frame(&mut voice_random(seed));
        if !out.is_empty() {
            apply_voice(&sound, voice, voice.speaker_id, out);
        }
    }
}

fn apply_voice(
    sound: &Sound,
    voice: &rustimal_logic::na_voice::Voice,
    voice_id: u16,
    outputs: Vec<rustimal_logic::na_voice::VoiceOut>,
) {
    use rustimal_logic::na_voice::VoiceOut;
    let group = audio_group::VOICE as u8;
    let mut commands = Vec::with_capacity(8);
    // Sou_VoiceMake updates both live slots, including punctuation changes.
    for (i, slot) in voice.slots.iter().enumerate() {
        commands.extend([
            cmd(0x01, group, i as u8, 0, AudioParam::F32(slot.volume())),
            cmd(0x04, group, i as u8, 0, AudioParam::F32(slot.pitch())),
            cmd(0x03, group, i as u8, 0, AudioParam::S8(64)),
        ]);
    }
    for out in outputs {
        match out {
            VoiceOut::Bebe => se(rustimal_logic::na_voice::SE_BEBE),
            VoiceOut::Melody => {
                if let Ok(mut engine) = sound.engine.lock() {
                    engine.voice_melody(voice_id);
                }
            }
            VoiceOut::Start { slot, phoneme } => {
                commands.push(cmd(0x06, group, slot, 0, AudioParam::S8(phoneme as i8)))
            }
        }
    }
    sound.send(&commands);
}

/// Original normal-size path retained for existing disc comparison probes.
#[cfg(test)]
fn play_voice(
    sound: &Sound,
    voice: &mut rustimal_logic::na_voice::Voice,
    seed: &mut u32,
    (spec, a, b, c): (u8, u8, u8, u8),
) {
    select_voice(sound, voice, spec);
    let out = voice.voice_se(a, b, c, 32, &mut voice_random(seed));
    apply_voice(sound, voice, 0, out);
}

/// One player door traversal at a time; a new traversal cancels its old tail.
static DOOR_REQUEST: Mutex<Option<(Option<rustimal_logic::shop::ShopTier>, bool)>> =
    Mutex::new(None);

pub fn door_sound(shop: Option<rustimal_logic::shop::ShopTier>, entering: bool) {
    if let Ok(mut request) = DOOR_REQUEST.lock() {
        *request = Some((shop, entering));
    }
}

#[derive(Resource, Default)]
struct DoorPlayback {
    cues: &'static [(u16, u16)],
    ticks: f32,
    next: usize,
}

fn advance_door_sound(time: Res<Time>, mut playback: ResMut<DoorPlayback>) {
    let request = DOOR_REQUEST.lock().ok().and_then(|mut r| r.take());
    if let Some((shop, entering)) = request {
        *playback = DoorPlayback {
            cues: rustimal_logic::action_audio::door_cues(shop, entering),
            ..default()
        };
    }
    playback.ticks += time.delta_secs() * 60.0;
    while let Some(&(tick, id)) = playback.cues.get(playback.next) {
        if f32::from(tick) > playback.ticks {
            break;
        }
        se(id);
        playback.next += 1;
    }
}

/// Plays a trigger effect (`sAdo_SysTrgStart`), from anywhere.
pub fn se(id: u16) {
    se_volume(id, 1.0);
}

/// Cursor feedback only when navigation actually changes a selection.
pub fn cursor_changed<T: PartialEq>(before: T, after: T) {
    if before != after {
        se(se_id::CURSOR);
    }
}

/// A trigger with the optional volume from Na_PlyWalkSe/Room.
pub fn se_volume(id: u16, volume: f32) {
    enqueue_effect(id, volume, 0, EffectOrigin::Center);
}

/// Positional actor/tool effect using the same listener as town ambience.
pub(crate) fn se_at(id: u16, position: Vec3) {
    enqueue_effect(
        id,
        1.0,
        0,
        EffectOrigin::World {
            position,
            attenuate: true,
        },
    );
}

/// Footsteps retain their accepted player/NPC gains. NPC volume already
/// includes its own source distance rule, so only pan is added here.
pub(crate) fn footstep(id: u16, volume: f32, position: Option<Vec3>, indoors: bool) {
    let origin = position.map_or(EffectOrigin::Center, |position| EffectOrigin::World {
        position,
        attenuate: false,
    });
    enqueue_effect(id, volume, if indoors { 5 } else { 0 }, origin);
}

fn enqueue_effect(id: u16, volume: f32, reverb: u8, origin: EffectOrigin) {
    if volume.is_finite() {
        if let Ok(mut queue) = SE_QUEUE.lock() {
            queue.push(EffectRequest {
                id,
                volume: volume.clamp(0.0, 1.0),
                reverb,
                origin,
            });
        }
    }
}

/// Submit after transforms/foot contacts; UI remains centered.
fn play_effects(
    sound: Option<Res<Sound>>,
    player: Query<&GlobalTransform, With<crate::player::Player>>,
) {
    let ids: Vec<_> = SE_QUEUE
        .lock()
        .map(|mut q| std::mem::take(&mut *q))
        .unwrap_or_default();
    let Some(sound) = sound else { return };
    if let Ok(mut engine) = sound.engine.lock() {
        let microphone = player
            .single()
            .ok()
            .map(|p| rustimal_logic::ambience::microphone((p.translation() * 40.0).to_array()));
        for request in ids {
            let (volume, pan) = match request.origin {
                EffectOrigin::Center => (request.volume, 64),
                EffectOrigin::World {
                    position,
                    attenuate,
                } => {
                    let Some(microphone) = microphone else {
                        continue;
                    };
                    let Some((gain, pan)) =
                        rustimal_logic::ambience::spatial((position * 40.0).to_array(), microphone)
                    else {
                        continue;
                    };
                    (request.volume * if attenuate { gain } else { 1.0 }, pan)
                }
            };
            engine.trigger_effect_spatial(request.id, volume, pan, request.reverb);
        }
    };
}

/// Raw slot path retained only for disc probes comparing the old allocator.
#[cfg(test)]
fn play_effect(sound: &Sound, s: u8, id: u16, volume: f32) {
    // The dedicated mono script has different playback behavior from the
    // six trigger scripts. Preserve MONO rather than only masking the ID.
    let s = if id & 0x1000 != 0 { 14 } else { s };
    sound.send(&[
        cmd(0x01, SE_GROUP, s, 0, AudioParam::F32(volume)),
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

/// Scene selection and game-layer gain, independent of sequence fades.
#[derive(Resource)]
struct Bgm {
    playing: Option<music_policy::Track>,
    handle: u8,
    ducking: music_policy::Ducking,
    volume: music_policy::VolumeRamp,
}

impl Default for Bgm {
    fn default() -> Self {
        Bgm {
            playing: None,
            handle: BGM1_GROUP,
            ducking: music_policy::Ducking::default(),
            volume: music_policy::VolumeRamp::default(),
        }
    }
}

/// Town/house/shop arbitration runs after gameplay and while paused.
/// A single state decision owns both BGM groups, preventing hourly music
/// from restarting outdoors' song while the player is inside.
fn update_music(
    sound: Option<Res<Sound>>,
    bgm: Option<ResMut<Bgm>>,
    house: Res<State<crate::house::HouseState>>,
    shop: Res<State<crate::shop::ShopLocation>>,
    shop_data: Res<crate::shop::ShopData>,
    focus: Res<crate::input::InputFocus>,
    time: Res<Time<Real>>,
    (room, current): (
        Res<crate::player_room::PlayerRoom>,
        Res<crate::house::CurrentRoom>,
    ),
) {
    let (Some(sound), Some(mut bgm)) = (sound, bgm) else {
        return;
    };
    use crate::input::InputFocus;
    let ducking = music_policy::Ducking {
        menu: matches!(
            *focus,
            InputFocus::Pockets | InputFocus::Mail | InputFocus::Paused
        ),
        // Shop transactions and quests currently use modal text panels.
        talking: matches!(
            *focus,
            InputFocus::Dialogue | InputFocus::Shop | InputFocus::Quest
        ),
    };
    if ducking != bgm.ducking {
        let frames = ducking.transition_frames(bgm.ducking);
        bgm.volume.retarget(ducking.volume(), frames);
        bgm.ducking = ducking;
    }
    let previous_volume = bgm.volume.current;
    bgm.volume.advance(time.delta_secs());

    let scene = if *shop.get() == crate::shop::ShopLocation::Inside {
        music_policy::Scene::Shop(shop_data.shop.shop_level)
    } else if *house.get() == crate::house::HouseState::Inside {
        music_policy::Scene::PlayerHouse
    } else {
        music_policy::Scene::Town
    };
    // Snow on the ground is not proof that it is snowing. Until weather
    // simulation supplies Common(weather), retain the accepted clear mask.
    let next = music_policy::select(
        scene,
        crate::clock::now().day_secs,
        music_policy::FieldWeather::Fine,
    )
    .or_else(|| {
        (scene == music_policy::Scene::PlayerHouse
            && crate::house::is_player_room(&current, &house))
        .then(|| room.music_track())
        .flatten()
    });
    let changed = next != bgm.playing;
    let mut commands = Vec::new();
    if changed {
        if let Some(track) = next {
            let Some(&seq) = sound.seq_table.get(track.id as usize) else {
                return;
            };
            let old = bgm.handle;
            let new = if old == BGM0_GROUP {
                BGM1_GROUP
            } else {
                BGM0_GROUP
            };
            // Retain the accepted hourly transition length. Scene cleanup
            // uses the original 0x168 duration; fade both songs to suit this
            // port's immediate door transitions (retail has a scene wipe).
            let fade = match (bgm.playing, scene) {
                (_, music_policy::Scene::PlayerHouse) if track.id >= 128 => 0,
                (None, _) => 0,
                (Some(previous), music_policy::Scene::Town) if (1..=24).contains(&previous.id) => {
                    60
                }
                _ => music_policy::SCENE_FADE_UPDATES,
            };
            commands.extend([
                cmd(0x83, old, 0, 0, AudioParam::S32(fade)),
                cmd(0x83, new, 0, 0, AudioParam::S32(0)),
                cmd(0x90, new, 0, 0, AudioParam::U16(0xFFFF)),
                cmd(0x08, new, 0xFF, 0, AudioParam::S8(0)),
                cmd(0x90, new, 0, 0, AudioParam::U16(track.mute_mask)),
                cmd(0x08, new, 0xFF, 0, AudioParam::S8(1)),
                cmd(0x82, new, seq, 0, AudioParam::S32(fade)),
            ]);
            bgm.handle = new;
            info!("music {} (sequence {seq}) on group {new}", track.id);
        } else {
            // Also stop a previous crossfade tail when entering a quiet room.
            let fade = if scene == music_policy::Scene::PlayerHouse
                && bgm.playing.is_some_and(|track| track.id >= 128)
            {
                0
            } else if matches!(scene, music_policy::Scene::Shop(_)) {
                music_policy::SHOP_CLOSING_FADE_UPDATES
            } else {
                music_policy::SCENE_FADE_UPDATES
            };
            for group in [BGM0_GROUP, BGM1_GROUP] {
                commands.push(cmd(0x83, group, 0, 0, AudioParam::S32(fade)));
            }
            info!("music quiet ({scene:?})");
        }
        bgm.playing = next;
    }
    if changed || previous_volume != bgm.volume.current {
        // Starting a sequence resets group gain. Reapply the current duck
        // after START_SEQ, to both handles including any outgoing tail.
        for group in [BGM0_GROUP, BGM1_GROUP] {
            commands.push(cmd(0x41, group, 0, 0, AudioParam::F32(bgm.volume.current)));
        }
    }
    if !commands.is_empty() {
        sound.send(&commands);
    }
}

/// Opt-in logging; no audio-thread I/O or extra engine locks by default.
#[derive(Resource, Default)]
struct AudioDiagnostics {
    seconds: f32,
}

fn report_mix(
    time: Res<Time<Real>>,
    diagnostics: Option<ResMut<AudioDiagnostics>>,
    sound: Option<Res<Sound>>,
) {
    let (Some(mut diagnostics), Some(sound)) = (diagnostics, sound) else {
        return;
    };
    diagnostics.seconds += time.delta_secs();
    if diagnostics.seconds < 5.0 {
        return;
    }
    diagnostics.seconds %= 5.0;
    if let Ok(engine) = sound.engine.lock() {
        let stats = engine.mix_stats();
        info!(
            "audio: peaks effects/music/voice/ambience={:?}, mix={:.3}, output={:.3}, min gain={:.3}, limited frames={}, nonfinite={}, active voices={}/24 (peaks/counters cumulative)",
            stats.bus_peaks, stats.peak, stats.output_peak, stats.minimum_gain,
            stats.limited_frames, stats.nonfinite_samples, stats.active_voices,
        );
    };
}

fn clear_requests() {
    if let Ok(mut queue) = SE_QUEUE.lock() {
        queue.clear();
    }
    if let Ok(mut queue) = VOICE_QUEUE.lock() {
        queue.clear();
    }
    if let Ok(mut request) = DOOR_REQUEST.lock() {
        *request = None;
    }
}

/// One shared reset for loading and leaving play; music is arbitrated separately.
fn reset_playback(
    voice: &mut VoicePlayback,
    door: &mut DoorPlayback,
    footsteps: Option<ResMut<Messages<crate::footsteps::Footstep>>>,
) {
    *voice = VoicePlayback::default();
    *door = DoorPlayback::default();
    clear_requests();
    if let Some(mut footsteps) = footsteps {
        footsteps.clear();
    }
}

fn reset_loaded_audio(
    mut resets: MessageReader<ResetAudio>,
    sound: Option<Res<Sound>>,
    mut voice: ResMut<VoicePlayback>,
    mut door: ResMut<DoorPlayback>,
    footsteps: Option<ResMut<Messages<crate::footsteps::Footstep>>>,
) {
    if resets.read().count() == 0 {
        return;
    }
    reset_playback(&mut voice, &mut door, footsteps);
    if let Some(sound) = sound {
        if let Ok(mut engine) = sound.engine.lock() {
            engine.stop_sequence(audio_group::VOICE);
            // Restart only SE: clears trigger/level slots and pending melodies.
            // The next ambience update repopulates sources at the loaded position.
            engine.start_sequence(audio_group::SE, SE_SEQUENCE as usize);
        }
    }
}

/// Despawning the owner drops its Bevy sink; old engines cannot keep streaming.
fn stop_sound(
    mut commands: Commands,
    streams: Query<Entity, With<SoundStream>>,
    sound: Option<Res<Sound>>,
    mut voice: ResMut<VoicePlayback>,
    mut door: ResMut<DoorPlayback>,
    footsteps: Option<ResMut<Messages<crate::footsteps::Footstep>>>,
    mut resets: ResMut<Messages<ResetAudio>>,
) {
    reset_playback(&mut voice, &mut door, footsteps);
    resets.clear();
    if let Some(sound) = sound {
        sound.send(&[cmd(0xFE, 0, 0, 0, AudioParam::S32(0))]);
    }
    for entity in streams {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<Sound>();
    commands.remove_resource::<Bgm>();
    commands.remove_resource::<AudioDiagnostics>();
}

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<NasStream>()
            .add_message::<ResetAudio>()
            .init_resource::<VoicePlayback>()
            .init_resource::<DoorPlayback>()
            .add_systems(
                OnEnter(AppState::InGame),
                start_sound.run_if(resource_exists::<GameDisc>),
            )
            .add_systems(
                Update,
                (reset_loaded_audio, advance_door_sound, play_voices)
                    .chain()
                    .after(crate::input::InteractSet::Doors)
                    .after(crate::msg_window::RevealSet)
                    .after(crate::save::LoadSet)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                PostUpdate,
                (
                    update_music,
                    play_effects,
                    crate::ambience::update_ambience,
                    report_mix,
                )
                    .after(crate::footsteps::play_footsteps)
                    .after(bevy::transform::TransformSystems::Propagate)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnExit(AppState::InGame), stop_sound);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check a complete register one-shot, including ordinary-slot traffic.
    #[test]
    #[ignore = "requires BEVY_CROSSING_ISO"]
    fn probe_register_tail() {
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut disc = disc::Disc::open_path(std::path::Path::new(&path)).unwrap();
        let rom = disc::RomImage::from_disc(&mut disc).unwrap();
        let catalog = disc::AssetCatalog::builtin(&rom);
        let data = Arc::new(DiscSound::new(Arc::new(
            sound_rom::SoundRom::read(&mut disc, &rom, &catalog).unwrap(),
        )));
        let tables = sound_rom::tables(&rom, &catalog).unwrap();
        let out = std::env::temp_dir().join("bevy-crossing-register");
        std::fs::create_dir_all(&out).unwrap();
        let mut durations = Vec::new();
        for (name, id, traffic) in [
            ("idle", 0xFFFF, false),
            ("ordinary", 0x50, false),
            ("ordinary_with_traffic", 0x50, true),
            ("mono", se_id::REGISTER, false),
            ("mono_with_traffic", se_id::REGISTER, true),
        ] {
            let sound = Sound {
                engine: Arc::new(Mutex::new(Engine::new(data.clone(), tables.clone()))),
                seq_table: Arc::from(Vec::new()),
            };
            sound.send(&[cmd(0x82, SE_GROUP, SE_SEQUENCE, 0, AudioParam::S32(0))]);
            let mut buf = vec![0.0; SAMPLES_PER_UPDATE * 2];
            for _ in 0..16 {
                buf.fill(0.0);
                sound.engine.lock().unwrap().update(&mut buf);
            }
            if id != 0xFFFF {
                play_effect(&sound, 0, id, 1.0);
            }
            let mut pcm = Vec::new();
            for update in 0..2400 {
                if traffic && update >= 64 && update % 8 == 0 {
                    // Exercise reuse of all six ordinary trigger slots;
                    // zero volume isolates the register in the mixed output.
                    play_effect(&sound, (update / 8 % 6) as u8, se_id::CURSOR, 0.0);
                }
                buf.fill(0.0);
                sound.engine.lock().unwrap().update(&mut buf);
                pcm.extend_from_slice(&buf);
            }
            let last = pcm
                .as_chunks::<2>()
                .0
                .iter()
                .rposition(|p| p.iter().any(|s| s.abs() > 0.0001));
            let duration = last.map_or(0.0, |i| i as f32 / SAMPLE_RATE as f32);
            let rms = (pcm.iter().map(|s| s * s).sum::<f32>() / pcm.len() as f32).sqrt();
            println!("{name}: duration {duration:.3}s, rms {rms:.4}");
            if id != 0xFFFF {
                assert!(duration > 0.05 && rms > 0.001, "{name} silent");
            }
            for (i, chunk) in pcm.chunks(SAMPLE_RATE as usize * 2).enumerate() {
                let r = (chunk.iter().map(|x| x * x).sum::<f32>() / chunk.len() as f32).sqrt();
                println!("  second {i}: rms {r:.5}");
            }
            std::fs::write(
                out.join(format!("{name}.wav")),
                sound_rom::wav_stereo(&pcm, SAMPLE_RATE),
            )
            .unwrap();
            durations.push(duration);
        }
        assert!(
            durations[4] >= durations[3] - 0.05,
            "ordinary traffic cut off mono register"
        );
        assert!(
            durations[2] < durations[1] * 0.9,
            "probe must reproduce ordinary-slot interruption"
        );
        assert!(
            durations[3] < 4.0,
            "register must finish and return to silence"
        );
        println!("full register clips: {}", out.display());
    }

    /// Real sequence-242 gameplay/UI effects, with no disc bytes in fixtures.
    #[test]
    #[ignore = "requires BEVY_CROSSING_ISO"]
    fn probe_gameplay_effects() {
        use rustimal_logic::footsteps;
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let mut disc = disc::Disc::open_path(std::path::Path::new(&path)).unwrap();
        let rom = disc::RomImage::from_disc(&mut disc).unwrap();
        let catalog = disc::AssetCatalog::builtin(&rom);
        let data = Arc::new(DiscSound::new(Arc::new(
            sound_rom::SoundRom::read(&mut disc, &rom, &catalog).unwrap(),
        )));
        let tables = sound_rom::tables(&rom, &catalog).unwrap();
        let floor_asset = catalog.get("SE_FLOOR_DATA").unwrap();
        let floors = rom
            .slice(floor_asset.source, floor_asset.offset, floor_asset.size)
            .unwrap();
        let mut labels: Vec<_> = [0, 4, 7, 23, 9, 22, 11]
            .into_iter()
            .map(|a| footsteps::outdoor_label(a, false))
            .collect();
        labels.push(footsteps::outdoor_label(0, true));
        labels.extend(
            floors
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| footsteps::FLOOR_START + u16::from_be_bytes([v[0], v[1]])),
        );
        labels.extend([
            se_id::CURSOR,
            se_id::MENU_EXIT,
            se_id::CHOICE_OPEN,
            se_id::CHOICE_CONFIRM,
            se_id::REGISTER,
            se_id::MENU_PAUSE,
            se_id::PAGE_OKURI,
            se_id::PANEL_OPEN,
            se_id::PANEL_CLOSE,
            se_id::MAILBOX_FLAG_UP,
            se_id::MAILBOX_FLAG_DOWN,
        ]);
        labels.sort_unstable();
        labels.dedup();
        let out = std::env::temp_dir().join("bevy-crossing-effects");
        std::fs::create_dir_all(&out).unwrap();
        for label in labels {
            let sound = Sound {
                engine: Arc::new(Mutex::new(Engine::new(data.clone(), tables.clone()))),
                seq_table: Arc::from(Vec::new()),
            };
            sound.send(&[cmd(0x82, SE_GROUP, SE_SEQUENCE, 0, AudioParam::S32(0))]);
            let mut buf = vec![0.0; SAMPLES_PER_UPDATE * 2];
            for _ in 0..16 {
                buf.fill(0.0);
                sound.engine.lock().unwrap().update(&mut buf);
            }
            let mut pcm = Vec::new();
            let mut variants = footsteps::FootstepVariants::default();
            for i in 0..4 {
                let footstep = label & 0x4000 != 0;
                let id = if footstep {
                    variants.select(label, false, if i == 0 { 0xFFFF } else { 0 })
                } else {
                    label
                };
                let volume = if footstep {
                    footsteps::volume(false, false)
                } else {
                    1.0
                };
                play_effect(&sound, i, id, volume);
                for _ in 0..80 {
                    buf.fill(0.0);
                    sound.engine.lock().unwrap().update(&mut buf);
                    pcm.extend_from_slice(&buf);
                }
            }
            let rms = (pcm.iter().map(|x| x * x).sum::<f32>() / pcm.len() as f32).sqrt();
            println!("effect {label:#06x}: rms {rms:.5}");
            assert!(rms > 0.0001, "silent effect {label:#06x}");
            assert!(pcm.iter().all(|x| x.is_finite()));
            std::fs::write(
                out.join(format!("effect_{label:04x}.wav")),
                sound_rom::wav_stereo(&pcm, SAMPLE_RATE),
            )
            .unwrap();
        }
        println!("listening clips: {}", out.display());
    }

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
                    buf.fill(0.0);
                    sound.engine.lock().unwrap().update(&mut buf);
                    clip.extend(buf);
                }
            }
            for _ in 0..120 {
                let mut buf = vec![0.0; SAMPLES_PER_UPDATE * 2];
                buf.fill(0.0);
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
    // These checks share the legacy game-facing queues; serialize only them.
    static QUEUE_CHECK: Mutex<()> = Mutex::new(());

    fn lifecycle_app() -> App {
        let mut app = App::new();
        app.add_message::<ResetAudio>()
            .add_message::<crate::footsteps::Footstep>()
            .init_resource::<VoicePlayback>()
            .init_resource::<DoorPlayback>();
        app
    }

    #[test]
    fn successful_load_reset_clears_pending_game_sounds_without_a_disc() {
        let _guard = QUEUE_CHECK.lock().unwrap();
        clear_requests();
        let mut app = lifecycle_app();
        app.add_systems(Update, reset_loaded_audio);
        app.world_mut().resource_mut::<DoorPlayback>().cues =
            rustimal_logic::action_audio::door_cues(None, true);
        app.world_mut()
            .resource_mut::<VoicePlayback>()
            .voice
            .spec_change(2);
        se(se_id::CURSOR);
        voice_begin(2, 7);
        door_sound(None, true);
        app.world_mut()
            .resource_mut::<Messages<crate::footsteps::Footstep>>()
            .write(crate::footsteps::Footstep {
                position: Vec3::ZERO,
                running: false,
                npc: false,
            });
        app.world_mut()
            .resource_mut::<Messages<ResetAudio>>()
            .write(ResetAudio);
        app.update();
        assert!(SE_QUEUE.lock().unwrap().is_empty());
        assert!(VOICE_QUEUE.lock().unwrap().is_empty());
        assert!(DOOR_REQUEST.lock().unwrap().is_none());
        assert!(app.world().resource::<DoorPlayback>().cues.is_empty());
        assert_eq!(app.world().resource::<VoicePlayback>().voice.spec, 0);
        assert!(app
            .world()
            .resource::<Messages<crate::footsteps::Footstep>>()
            .is_empty());
    }

    #[test]
    fn leaving_play_removes_all_stream_owners_and_audio_state() {
        let _guard = QUEUE_CHECK.lock().unwrap();
        clear_requests();
        let mut app = lifecycle_app();
        app.insert_resource(Bgm::default())
            .insert_resource(AudioDiagnostics::default())
            .add_systems(Update, stop_sound);
        app.world_mut().spawn(SoundStream);
        app.world_mut().spawn(SoundStream);
        se(se_id::CURSOR);
        app.update();
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<SoundStream>>();
        assert_eq!(query.iter(app.world()).count(), 0);
        assert!(!app.world().contains_resource::<Bgm>());
        assert!(!app.world().contains_resource::<AudioDiagnostics>());
        assert!(SE_QUEUE.lock().unwrap().is_empty());
        // A second cleanup is safe; a restart has no leftover owner/cues.
        app.update();
    }

    /// Offline combined render; no user save/settings writes or audio device.
    #[test]
    #[ignore = "requires BEVY_CROSSING_ISO"]
    fn probe_combined_lifecycle() {
        let _guard = QUEUE_CHECK.lock().unwrap();
        clear_requests();
        let path = std::env::var("BEVY_CROSSING_ISO").expect("set BEVY_CROSSING_ISO");
        let disc = crate::game_disc::load(std::path::PathBuf::from(path)).unwrap();
        let entry = disc.catalog.get("SEQ_TABLE").unwrap();
        let seq_table: Arc<[u8]> = disc
            .rom
            .slice(entry.source, entry.offset, entry.size)
            .unwrap()
            .into();
        let sound = Sound {
            engine: Arc::new(Mutex::new(Engine::new(
                Arc::new(DiscSound::new(disc.sound.clone().unwrap())),
                sound_rom::tables(&disc.rom, &disc.catalog).unwrap(),
            ))),
            seq_table,
        };
        sound.send(&[
            cmd(0x82, SE_GROUP, SE_SEQUENCE, 0, AudioParam::S32(0)),
            cmd(0x82, BGM0_GROUP, sound.seq_table[12], 0, AudioParam::S32(0)),
        ]);
        let mut app = lifecycle_app();
        app.insert_resource(sound.clone())
            .add_systems(Update, reset_loaded_audio);
        let mut voice = rustimal_logic::na_voice::Voice::default();
        let mut seed = 0;
        let mut buf = [0.0; SAMPLES_PER_UPDATE * 2];
        for update in 0..1800 {
            if update >= 24 && update % 16 == 0 {
                sound
                    .engine
                    .lock()
                    .unwrap()
                    .trigger_effect(se_id::CURSOR, 1.0);
                sound
                    .engine
                    .lock()
                    .unwrap()
                    .trigger_effect(rustimal_logic::footsteps::WOOD, 0.6);
            }
            if update == 64 {
                sound
                    .engine
                    .lock()
                    .unwrap()
                    .trigger_effect(se_id::REGISTER, 1.0);
            }
            if update % 32 == 0 {
                play_voice(&sound, &mut voice, &mut seed, (2, 0x5E, 0x5D, 0xFF));
            }
            if update == 500 {
                app.world_mut()
                    .resource_mut::<Messages<ResetAudio>>()
                    .write(ResetAudio);
                app.update();
                assert!(sound.engine.lock().unwrap().playing(BGM0_GROUP as usize));
                // The game-facing voice context was reset with the SE player.
                voice = rustimal_logic::na_voice::Voice::default();
            }
            if update == 800 {
                sound.ambience([
                    Some(rustimal_logic::ambience::Level {
                        key: 1,
                        id: 0x0B,
                        volume: 0.8,
                        pan: 20,
                    }),
                    None,
                    None,
                    None,
                ]);
            }
            if update == 1100 {
                sound.ambience([None; 4]);
            }
            if update == 1200 {
                sound.send(&[cmd(0x41, BGM0_GROUP, 0, 0, AudioParam::F32(0.5))]);
            }
            if update == 1300 {
                sound.send(&[cmd(0x41, BGM0_GROUP, 0, 0, AudioParam::F32(1.0))]);
            }
            buf.fill(0.0);
            sound.engine.lock().unwrap().update(&mut buf);
            assert!(buf.iter().all(|s| s.is_finite() && s.abs() <= 0.98001));
        }
        let stats = sound.engine.lock().unwrap().mix_stats();
        println!("combined mix: {stats:?}");
        assert!(stats.peak > 0.01 && stats.active_voices <= 24);
        assert_eq!(stats.nonfinite_samples, 0);
        sound.send(&[cmd(0xFE, 0, 0, 0, AudioParam::S32(0))]);
        for _ in 0..1200 {
            buf.fill(0.0);
            sound.engine.lock().unwrap().update(&mut buf);
        }
        assert!(
            buf.iter().all(|s| s.abs() < 1.0e-6),
            "stop drains notes and delay"
        );
        for group in [SE_GROUP, BGM0_GROUP, BGM1_GROUP, audio_group::VOICE as u8] {
            assert!(!sound.engine.lock().unwrap().playing(group as usize));
        }
        clear_requests();
    }
    #[test]
    fn no_disc_submissions_stay_silent_and_do_not_accumulate() {
        let _guard = QUEUE_CHECK.lock().unwrap();
        clear_requests();
        let mut app = lifecycle_app();
        app.init_resource::<Time>().add_systems(
            Update,
            (advance_door_sound, play_voices, play_effects).chain(),
        );
        se(se_id::CURSOR);
        voice_begin(2, 0);
        voice_character([0x5E, 0x5D, 0xFF], 32, false);
        door_sound(None, true);
        app.update();
        app.update();
        assert!(!app.world().contains_resource::<Sound>());
        assert!(VOICE_QUEUE.lock().unwrap().is_empty());
        assert!(SE_QUEUE.lock().unwrap().is_empty());
        assert!(DOOR_REQUEST.lock().unwrap().is_none());
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<SoundStream>>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }
}
