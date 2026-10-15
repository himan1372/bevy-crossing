//! The sequence player of the game's sound engine (`jaudio_NES`, the
//! Zelda 64 "Nas" engine): groups (sequence players) run scripts that
//! start subtracks (channels), which start notes (layers), which play
//! samples on voices with an ADSR envelope, vibrato and pitch sweeps; a
//! mixer renders the voices to 48 kHz stereo.
//!
//! Ported from `track.c` (scripts), `effect.c` (control, modulation,
//! envelopes) and `channel.c` (voices). Simplified, for now: voices are
//! taken from one shared pool (the engine keeps per-group reserves),
//! resampling is linear (the RSP uses a filter), and there is no reverb,
//! filters, Haas or surround.

use std::sync::Arc;

/// `AUDIO_SUBTRACK_NUM`, notes per subtrack, `AUDIO_NOTE_MAX`, groups.
const SUBTRACKS: usize = 16;
const LAYERS: usize = 4;
const NOTE_MAX: usize = 128;
pub const GROUPS: usize = 5;
/// `AUDIO_TATUMS_PER_BEAT`, `updates_per_frame`, output rate.
const TATUMS: i32 = 48;
const UPDATES_PER_FRAME: f32 = 4.0;
/// The engine's spec says 48 kHz and 200 samples an update, but the
/// GameCube's mixer runs at 32028.5 Hz and takes one 4-update frame per
/// 560 samples (`JAC_FRAMESAMPLES`): 140 real samples an update, 228.8
/// updates a second. Rendering at that rate keeps tempo and pitch right.
pub const SAMPLE_RATE: u32 = 32028;
pub const SAMPLES_PER_UPDATE: usize = 140;
/// The spec's samples per update (48000 / 60 / 4), which pitch assumes.
const SPEC_SAMPLES_PER_UPDATE: f32 = 200.0;

/// `AUDIO_MUTE_FLAG_*`.
const MUTE_STOP_SAMPLES: u8 = 1 << 3;
const MUTE_STOP_SUBTRACK: u8 = 1 << 4;
const MUTE_SOFTEN: u8 = 1 << 5;
const MUTE_STOP_NOTE: u8 = 1 << 6;
const MUTE_STOP_SCRIPT: u8 = 1 << 7;
/// `VOICE_TYPE_*` (instruments are `prog + 2`).
const PERCUSSION: u8 = 0;
const SOUND_EFF: u8 = 1;
/// `_0A` flags, big-endian bitfield order.
const SKIP_VOLUME: u16 = 0x8000;
const SKIP_FREQ: u16 = 0x4000;
const USE_SUB_REVERB: u16 = 0x2000;
const USE_SUB_VIBRATO: u16 = 0x1000;
const ADD_SUB_TRANSPOSE: u16 = 0x0800;
const SKIP_PAN: u16 = 0x0080;
/// `GROUP_STATE_*`.
const STATE_FADE_IN: u8 = 1;
const STATE_FADE_OUT: u8 = 2;

/// A decoded sample with its loop and tuning.
#[derive(Debug)]
pub struct Pcm {
    pub data: Vec<i16>,
    pub loop_start: u32,
    pub loop_end: u32,
    /// 0: no loop; otherwise loops (forever unless it's 2 and stopped).
    pub loop_count: u32,
}

/// A sample and its tuning (`wtstr`).
#[derive(Clone, Debug)]
pub struct Tuned {
    pub pcm: Arc<Pcm>,
    pub tuning: f32,
}

/// An envelope (`envdat` entries).
pub type Envelope = Arc<[(i16, i16)]>;

/// An instrument (`voicetable`).
#[derive(Clone, Debug)]
pub struct Instrument {
    pub range: [u8; 2],
    pub decay: u8,
    pub envelope: Envelope,
    pub low: Option<Tuned>,
    pub normal: Option<Tuned>,
    pub high: Option<Tuned>,
}

/// A drum (`perctable`).
#[derive(Clone, Debug)]
pub struct Drum {
    pub decay: u8,
    pub pan: u8,
    pub sample: Tuned,
    pub envelope: Envelope,
}

/// A loaded bank, ready to play.
#[derive(Clone, Debug, Default)]
pub struct Bank {
    pub instruments: Vec<Option<Instrument>>,
    pub drums: Vec<Option<Drum>>,
    pub sfx: Vec<Option<Tuned>>,
}

/// What the engine reads from the disc.
pub trait SoundData: Send + Sync {
    fn sequence(&self, id: usize) -> Option<Vec<u8>>;
    fn bank(&self, id: usize) -> Option<Arc<Bank>>;
    /// `AudiomapHeaderStart` bytes: per sequence a byte offset (u16
    /// big-endian at `seq * 2`) to a count and that many bank ids.
    fn map(&self) -> &[u8];
}

/// The engine's tables from the DOL.
#[derive(Clone, Debug)]
pub struct Tables {
    pub pitch: [f32; 128],
    pub pcent: [f32; 256],
    pub pcent2: [f32; 256],
    pub velocities: [u8; 16],
    pub gate_times: [u8; 16],
    pub default_env: Envelope,
    pub stereo: [f32; 128],
    /// `WAVEMEM_TABLE[2]`: the vibrato curve.
    pub vibrato: [i16; 64],
    /// `BDB_SEQDATA`: the main group's script.
    pub main_script: Vec<u8>,
}

/// Reads an envelope from bytes at `at` (big-endian `delay, value` pairs)
/// up to its end marker.
pub fn read_envelope(b: &[u8], at: usize) -> Envelope {
    let mut out = Vec::new();
    for i in 0..32 {
        let p = at + i * 4;
        let (Some(d), Some(v)) = (b.get(p..p + 2), b.get(p + 2..p + 4)) else {
            break;
        };
        let d = i16::from_be_bytes([d[0], d[1]]);
        let v = i16::from_be_bytes([v[0], v[1]]);
        out.push((d, v));
        if matches!(d, 0 | -1 | -3) {
            break;
        }
    }
    out.into()
}

#[derive(Clone, Debug, Default)]
struct Macro {
    pc: usize,
    stack: [usize; 4],
    loops: [u8; 4],
    depth: usize,
    value: i8,
}

#[derive(Clone, Debug, Default)]
struct Vibrato {
    rate_target: u16,
    rate_start: u16,
    depth_target: u16,
    depth_start: u16,
    rate_delay: u16,
    depth_delay: u16,
    delay: u16,
}

impl Vibrato {
    fn fresh() -> Self {
        Vibrato {
            rate_target: 2048,
            rate_start: 2048,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Sweep {
    mode: u8,
    current: u32,
    speed: u32,
    extent: f32,
}

#[derive(Clone, Debug)]
struct AdsrSrc {
    envelope: Envelope,
    decay: u8,
    sustain: u8,
}

/// A note layer (`note`).
#[derive(Clone, Debug)]
struct Layer {
    sub: (usize, usize),
    enabled: bool,
    finished: bool,
    muted: bool,
    continuous: bool,
    continuous_released: bool,
    attached: bool,
    ignore_drum_pan: bool,
    need_init: bool,
    m: Macro,
    delay: u16,
    gate_delay: u16,
    last_delay: u16,
    short_delay: u16,
    gate_time: u8,
    velocity_sq: f32,
    velocity_sq2: f32,
    pan: u8,
    transposition: i8,
    inst_or_wave: u8,
    instrument: Option<Instrument>,
    tuned: Option<Tuned>,
    frequency_scale: f32,
    bend: f32,
    semitone: u8,
    sweep: Sweep,
    sweep_target: u8,
    sweep_time: u16,
    adsr: AdsrSrc,
    vibrato: Vibrato,
    flags: u16,
    reverb: u8,
    note_frequency: f32,
    note_velocity: f32,
    note_pan: u8,
    voice: Option<usize>,
}

/// A subtrack (`sub`).
#[derive(Clone, Debug)]
struct Sub {
    enabled: bool,
    finished: bool,
    stop_script: bool,
    has_instrument: bool,
    large_notes: bool,
    muted: bool,
    mute_flags: u8,
    m: Macro,
    delay: u16,
    layers: [Option<usize>; LAYERS],
    bank: u8,
    inst_or_wave: u8,
    instrument: Option<Instrument>,
    transposition: i8,
    volume: f32,
    volume_scale: f32,
    applied_volume: f32,
    new_pan: u8,
    pan_weight: u8,
    pan: u16,
    frequency_scale: f32,
    changes: u8,
    vibrato: Vibrato,
    adsr: AdsrSrc,
    note_priority: u8,
    priority2: u8,
    reverb: u8,
    velocity_variance: u8,
    gate_variance: u8,
    port: [i8; 8],
    dyn_tbl: usize,
    dynamic_value: u16,
    sample_start: u16,
    index: usize,
}

const CHANGE_VOLUME: u8 = 1;
const CHANGE_FREQ: u8 = 2;
const CHANGE_PAN: u8 = 4;

/// A sequence player (`group`).
#[derive(Clone, Debug)]
struct Group {
    enabled: bool,
    finished: bool,
    muted: bool,
    stop_script: bool,
    recalc_volume: bool,
    seq: Vec<u8>,
    seq_id: usize,
    bank: u8,
    m: Macro,
    delay: u16,
    tempo: i32,
    tempo_change: i32,
    tempo_acc: i32,
    transposition: i8,
    state: u8,
    fade_timer: u16,
    stored_fade_timer: u16,
    fade_volume: f32,
    fade_velocity: f32,
    fade_volume_scale: f32,
    applied_fade_volume: f32,
    mute_volume_scale: f32,
    mute_flags: u8,
    velocities: usize,
    gate_times: usize,
    port: [i8; 8],
    subs: Vec<Sub>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Copy)]
enum Adsr {
    #[default]
    Disabled,
    Initial,
    Loop,
    Fade,
    Hang,
    Decay,
    Release,
    Sustain,
}

#[derive(Clone, Debug, Default)]
struct Env {
    status: Adsr,
    data: Option<Envelope>,
    idx: usize,
    delay: i32,
    current: f32,
    target: f32,
    velocity: f32,
    fadeout: f32,
    sustain: f32,
    decay_flag: bool,
    release_flag: bool,
}

#[derive(Clone, Debug, Default)]
struct VibState {
    active: bool,
    params: Vibrato,
    time: u32,
    rate: f32,
    depth: f32,
    rate_timer: u16,
    depth_timer: u16,
    delay: u16,
}

/// What a released voice keeps playing with (`playbackparams`).
#[derive(Clone, Debug, Default)]
struct Params {
    pitch: f32,
    velocity: f32,
    pan: u8,
}

/// A voice (`channel`).
#[derive(Clone, Debug, Default)]
struct Voice {
    current: Option<usize>,
    previous: Option<usize>,
    wanted: Option<usize>,
    priority: u8,
    status: u8,
    env: Env,
    vib: VibState,
    sweep: Sweep,
    vib_scale: f32,
    sweep_scale: f32,
    params: Params,
    sample: Option<Tuned>,
    pos: f64,
    finished: bool,
    step: f64,
    target: [f32; 2],
    volume: [f32; 2],
}

/// The engine.
pub struct Engine {
    data: Arc<dyn SoundData>,
    t: Tables,
    groups: Vec<Group>,
    layers: Vec<Option<Layer>>,
    free_layers: Vec<usize>,
    voices: Vec<Voice>,
    max_tempo: i32,
    tick_factor: f32,
    resample: f32,
    decay_table: [f32; 256],
    seed: u32,
    random: u32,
    /// `thread_cmd_group_mask`: which subtracks an all-subtracks command
    /// reaches.
    group_mask: [u16; GROUPS],
    triggers: crate::audio::TrgSeManager,
    levels: crate::audio::LevSeManager,
    ambience: [Option<crate::ambience::Level>; crate::ambience::VOICES],
    level_cooldown: [u8; crate::ambience::VOICES],
}

fn sub_fresh(index: usize) -> Sub {
    Sub {
        enabled: false,
        finished: false,
        stop_script: false,
        has_instrument: false,
        large_notes: false,
        muted: false,
        mute_flags: 0,
        m: Macro::default(),
        delay: 0,
        layers: [None; LAYERS],
        bank: 0,
        inst_or_wave: 0,
        instrument: None,
        transposition: 0,
        volume: 1.0,
        volume_scale: 1.0,
        applied_volume: 1.0,
        new_pan: 64,
        pan_weight: 128,
        pan: 64 * 128,
        frequency_scale: 1.0,
        changes: 0xFF,
        vibrato: Vibrato::fresh(),
        adsr: AdsrSrc {
            envelope: Arc::from(Vec::new()),
            decay: 240,
            sustain: 0,
        },
        note_priority: 3,
        priority2: 1,
        reverb: 0,
        velocity_variance: 0,
        gate_variance: 0,
        port: [-1; 8],
        dyn_tbl: 0,
        dynamic_value: 0,
        sample_start: 0,
        index,
    }
}

impl Engine {
    pub fn new(data: Arc<dyn SoundData>, tables: Tables) -> Self {
        // __Nas_MemoryReconfig with the GAFE01 spec (48 kHz, 4 updates).
        let deviation = 1000.0 * 1.00278 / 60.0;
        let max_tempo =
            ((60.0 * 1000.0 * TATUMS as f32 * UPDATES_PER_FRAME) / 48.0 / deviation / 1.04613)
                as i32;
        let tick_factor = 60.0 * UPDATES_PER_FRAME / 48000.0 / max_tempo as f32;
        let inv_scaled = 1.0 / 256.0 / UPDATES_PER_FRAME;
        let rel = |v: f32| inv_scaled * 256.0 / v;
        let mut decay_table = [0.0f32; 256];
        decay_table[0xFF] = rel(0.25);
        decay_table[0xFE] = rel(0.33);
        decay_table[0xFD] = rel(0.5);
        decay_table[0xFC] = rel(0.66);
        decay_table[0xFB] = rel(0.75);
        for (i, d) in decay_table.iter_mut().enumerate() {
            match i {
                0x80..=0xFA => *d = rel((0xFB - i) as f32),
                0x10..=0x7F => *d = rel(((0x80 - i) * 4 + 0x3C) as f32),
                1..=0xF => *d = rel(((0xF - i) * 0x3C + 0x1E0) as f32),
                _ => {}
            }
        }
        let mut e = Engine {
            data,
            groups: Vec::new(),
            layers: vec![None; NOTE_MAX],
            free_layers: (0..NOTE_MAX).rev().collect(),
            voices: vec![Voice::default(); 24],
            max_tempo,
            tick_factor,
            // resample_rate (33476.156 / 48000), spread over the 140 real
            // samples an update.
            resample: 33476.156 / 48000.0 * SPEC_SAMPLES_PER_UPDATE / SAMPLES_PER_UPDATE as f32,
            decay_table,
            seed: 0x1234_5678,
            random: 0x3039,
            group_mask: [0xFFFF; GROUPS],
            triggers: crate::audio::TrgSeManager::new(),
            levels: crate::audio::LevSeManager::new(),
            ambience: [None; crate::ambience::VOICES],
            level_cooldown: [0; crate::ambience::VOICES],
            t: tables,
        };
        for _ in 0..=GROUPS {
            let g = e.group_fresh();
            e.groups.push(g);
        }
        // Nas_InitPlayer: the main group runs BDB_SEQDATA on subtrack 0.
        let main = GROUPS;
        e.groups[main].seq = e.t.main_script.clone();
        e.groups[main].bank = 0xFF;
        e.groups[main].seq_id = 0xFF;
        e.groups[main].enabled = true;
        e
    }

    fn group_fresh(&self) -> Group {
        Group {
            enabled: false,
            finished: false,
            muted: false,
            stop_script: false,
            recalc_volume: false,
            seq: Vec::new(),
            seq_id: 0,
            bank: 0xFF,
            m: Macro::default(),
            delay: 0,
            tempo: 120 * TATUMS,
            tempo_change: 0,
            tempo_acc: 0,
            transposition: 0,
            state: STATE_FADE_IN,
            fade_timer: 0,
            stored_fade_timer: 0,
            fade_volume: 1.0,
            fade_velocity: 0.0,
            fade_volume_scale: 1.0,
            applied_fade_volume: 1.0,
            mute_volume_scale: 0.5,
            mute_flags: MUTE_SOFTEN | MUTE_STOP_NOTE,
            velocities: usize::MAX,
            gate_times: usize::MAX,
            port: [-1; 8],
            subs: (0..SUBTRACKS).map(sub_fresh).collect(),
        }
    }

    /// `__Nas_StartSeq`: group `g` plays sequence `id` with the last bank
    /// its map entry lists.
    pub fn start_sequence(&mut self, g: usize, id: usize) -> bool {
        self.release_group(g);
        let map = self.data.map();
        let Some(at) = map
            .get(id * 2..id * 2 + 2)
            .map(|b| u16::from_be_bytes([b[0], b[1]]) as usize)
        else {
            return false;
        };
        let count = map.get(at).copied().unwrap_or(0) as usize;
        let bank = if count == 0 {
            0xFF
        } else {
            map.get(at + count).copied().unwrap_or(0xFF)
        };
        let Some(seq) = self.data.sequence(id) else {
            return false;
        };
        let mut fresh = self.group_fresh();
        fresh.port = self.groups[g].port;
        // Nas_InitSubTrack leaves `muted` alone: a mute set before the
        // start (the hourly mute masks) holds.
        for (new, old) in fresh.subs.iter_mut().zip(&self.groups[g].subs) {
            new.muted = old.muted;
        }
        fresh.seq = seq;
        fresh.seq_id = id;
        fresh.bank = bank;
        fresh.enabled = true;
        self.groups[g] = fresh;
        true
    }

    /// Stops group `g`.
    pub fn stop_sequence(&mut self, g: usize) {
        self.release_group(g);
    }

    /// Whether group `g` is playing.
    pub fn playing(&self, g: usize) -> bool {
        self.groups.get(g).is_some_and(|g| g.enabled)
    }

    /// Writes a group's port (the game's way to talk to a script).
    pub fn set_group_port(&mut self, g: usize, port: usize, value: i8) {
        if let Some(p) = self.groups.get_mut(g).and_then(|g| g.port.get_mut(port)) {
            *p = value;
        }
    }

    /// Writes a subtrack's port.
    pub fn set_sub_port(&mut self, g: usize, s: usize, port: usize, value: i8) {
        if let Some(p) = self
            .groups
            .get_mut(g)
            .and_then(|g| g.subs.get_mut(s))
            .and_then(|s| s.port.get_mut(port))
        {
            *p = value;
        }
    }

    /// Safe completion-port access; uninitialized/missing ports read -1.
    pub fn sub_port(&self, g: usize, s: usize, port: usize) -> i8 {
        self.groups
            .get(g)
            .and_then(|g| g.subs.get(s))
            .and_then(|s| s.port.get(port))
            .copied()
            .unwrap_or(-1)
    }

    /// Submit a game effect through retail trigger allocation and MONO routing.
    /// The caller shares the engine lock with the renderer, so allocation,
    /// parameter updates and request submission are atomic.
    pub fn trigger_effect(&mut self, id: u16, volume: f32) -> crate::audio::TrgStartResult {
        let mut ring = crate::audio::CommandRing::new();
        let result = self
            .triggers
            .start(&mut ring, id, volume, 1.0, 1.0, 64, 0, 0.0);
        for command in ring.drain() {
            self.process(&command);
        }
        result
    }

    /// Replace desired ambient emitters atomically. Never resubmit a loop
    /// just because its spatial parameters changed.
    pub fn set_ambience(
        &mut self,
        levels: [Option<crate::ambience::Level>; crate::ambience::VOICES],
    ) {
        self.ambience = levels;
    }

    fn advance_levels(&mut self) {
        use crate::audio::{AudioParam, AudioPort};
        for i in 0..crate::ambience::VOICES {
            let sub = 8 + i;
            if !self.groups[0].enabled || !self.groups[0].subs[sub].enabled {
                continue;
            }
            let wanted = self.ambience[i];
            let id = wanted.map_or(0, |v| v.id);
            let old = self.levels.current(i);
            if old != 0 && old != id {
                self.levels.stop(i, old);
                self.set_sub_port(0, sub, 0, -1); // cancel an unconsumed start
                                                  // Sou_LevStop
                self.set_sub_port(0, sub, 5, -1);
                // Allow a complete sequencer tick to consume STOP before
                // submitting another request on this subtrack.
                self.level_cooldown[i] = 4;
            }
            if self.level_cooldown[i] > 0 {
                self.level_cooldown[i] -= 1;
                self.sub_param(0, sub, 0x01, 0, 0, 0.0, 0);
                continue;
            }
            let Some(level) = wanted else { continue };
            // Sou_LevSet strips the high echo bit and writes port 0 only.
            if self.levels.current(i) == 0 {
                self.levels.start(i, id);
                self.set_sub_port(0, sub, 0, (id & 0x7F) as i8);
            }
            for (opcode, param) in [
                (0x01, AudioParam::F32(level.volume)),
                (0x04, AudioParam::F32(1.0)),
                (0x03, AudioParam::U8(level.pan)),
                (0x05, AudioParam::U8(0)),
            ] {
                self.process(&AudioPort {
                    opcode,
                    arg0: 0,
                    arg1: sub as u8,
                    arg2: 0,
                    param,
                });
            }
        }
    }

    fn finish_triggers(&mut self) {
        let pending = std::array::from_fn(|i| self.sub_port(0, i, 1) != -1);
        let busy = std::array::from_fn(|i| self.sub_port(0, i, 5));
        let grace = SAMPLE_RATE.div_ceil(2 * SAMPLES_PER_UPDATE as u32);
        let expired = self.triggers.observe_completion(pending, busy, grace);
        for (i, expired) in expired.into_iter().enumerate() {
            if expired {
                self.set_sub_port(0, i, 0, -1);
                self.set_sub_port(0, i, 1, -1);
            }
        }
    }

    pub fn group_port(&self, g: usize, port: usize) -> i8 {
        self.groups
            .get(g)
            .and_then(|g| g.port.get(port))
            .copied()
            .unwrap_or(-1)
    }

    /// Sets a group's volume scale (0..1).
    pub fn set_group_volume(&mut self, g: usize, v: f32) {
        if let Some(g) = self.groups.get_mut(g) {
            g.fade_volume_scale = v;
            g.recalc_volume = true;
        }
    }

    /// `Nap_Process1Command`: one command from the game's audio layer
    /// (`AudioPort`, `audiocommon.h`'s `AUDIOCMD_*`). Loads, callbacks,
    /// filters and the sound mode don't apply here.
    pub fn process(&mut self, c: &crate::audio::AudioPort) {
        use crate::audio::AudioParam as P;
        let s32 = match c.param {
            P::S8(v) => v as i32,
            P::U8(v) => v as i32,
            P::S16(v) => v as i32,
            P::U16(v) => v as i32,
            P::S32(v) => v,
            P::U32(v) => v as i32,
            P::F32(v) => v as i32,
        };
        let f32v = match c.param {
            P::F32(v) => v,
            _ => s32 as f32,
        };
        let s8 = s32 as i8;
        let g = c.arg0 as usize;
        match c.opcode {
            // AUDIOCMD_START_SEQ (fade-in time in param: start at once).
            0x82 => {
                if g < GROUPS {
                    self.start_sequence(g, c.arg1 as usize);
                    if s32 > 0 {
                        let gr = &mut self.groups[g];
                        gr.fade_volume = 0.0;
                        gr.state = 0;
                        gr.fade_timer = s32 as u16;
                        gr.fade_velocity = 1.0 / s32 as f32;
                    }
                }
            }
            0x85 => {
                if g < GROUPS {
                    self.start_sequence(g, c.arg1 as usize);
                }
            }
            // AUDIOCMD_STOP_SEQ: at once, or fading over the param.
            0x83 => {
                if g < GROUPS && self.groups[g].enabled {
                    if s32 == 0 {
                        self.release_group(g);
                    } else {
                        self.fade_out(g, s32 as u16);
                    }
                }
            }
            0xF1 | 0xF2 => {
                let mute = c.opcode == 0xF1;
                for (i, gr) in self.groups.iter_mut().enumerate().take(GROUPS) {
                    if c.arg0 == 0xFF || i == g {
                        gr.muted = mute;
                        gr.recalc_volume = true;
                    }
                }
            }
            0x90 => {
                if g < GROUPS {
                    self.group_mask[g] = s32 as u16;
                }
            }
            // AUDIOCMD_FORCE_STOP_ALL_GROUPS
            0xFE => {
                for g in 0..GROUPS {
                    if self.groups[g].enabled {
                        self.release_group(g);
                    }
                }
            }
            // AUDIOCMD_MAIN_GROUP_SUBPORT_CMD
            0xFA => {
                let port = &mut self.groups[GROUPS].subs[0].port;
                if s32 == 0 {
                    port[4] = -1;
                } else {
                    port[2] = c.arg1 as i8;
                    port[1] = c.arg0 as i8;
                    port[0] = c.arg2 as i8;
                }
            }
            // AUDIOCMD_MAIN_GROUP_SET_FREQ_SCALE
            0xFD => {
                let pc = self.t.pcent2[(128 + s32).clamp(0, 255) as usize];
                let sub = &mut self.groups[GROUPS].subs[0];
                sub.frequency_scale = pc;
                sub.changes |= CHANGE_FREQ;
                sub.vibrato.depth_target = c.arg2 as u16 * 4;
                sub.vibrato.depth_delay = 32;
            }
            // AUDIOCMD_MAIN_GROUP_SET_VOL_SCALE
            0xFF => {
                let sub = &mut self.groups[GROUPS].subs[0];
                sub.volume_scale = f32v;
                sub.changes |= CHANGE_VOLUME;
            }
            0x41..=0x4F if g < GROUPS => self.group_param(g, c, s32, f32v, s8),
            0x01..=0x15 if g < GROUPS => {
                if (c.arg1 as usize) < SUBTRACKS {
                    self.sub_param(g, c.arg1 as usize, c.opcode, c.arg2, s32, f32v, s8);
                } else if c.arg1 == 0xFF {
                    let mask = self.group_mask[g];
                    for i in 0..SUBTRACKS {
                        if mask >> i & 1 != 0 {
                            self.sub_param(g, i, c.opcode, c.arg2, s32, f32v, s8);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// `__SetGrpParam`.
    fn group_param(&mut self, g: usize, c: &crate::audio::AudioPort, s32: i32, f: f32, s8: i8) {
        let gr = &mut self.groups[g];
        match c.opcode {
            0x41 => {
                if gr.fade_volume_scale != f {
                    gr.fade_volume_scale = f;
                    gr.recalc_volume = true;
                }
            }
            0x47 => gr.tempo = s32 * TATUMS,
            0x49 => gr.tempo_change = s32 * TATUMS,
            0x4E => gr.tempo_change = s32,
            0x48 => gr.transposition = s8,
            0x46 => {
                if let Some(p) = gr.port.get_mut(c.arg2 as usize) {
                    *p = s8;
                }
            }
            0x4A | 0x4B => {
                let v = if c.opcode == 0x4A {
                    c.arg1 as f32 / 127.0
                } else {
                    c.arg1 as f32 / 100.0 * gr.fade_volume
                };
                if gr.state != STATE_FADE_OUT {
                    if s32 == 0 {
                        gr.fade_volume = v;
                    } else {
                        gr.fade_velocity = (v - gr.fade_volume) / s32 as f32;
                        gr.state = 0;
                        gr.fade_timer = s32 as u16;
                    }
                }
            }
            _ => {}
        }
    }

    /// `__SetSubParam`.
    #[allow(clippy::too_many_arguments)]
    fn sub_param(&mut self, g: usize, s: usize, op: u8, arg2: u8, s32: i32, f: f32, s8: i8) {
        let sub = &mut self.groups[g].subs[s];
        match op {
            0x01 => {
                sub.volume_scale = f;
                sub.changes |= CHANGE_VOLUME;
            }
            0x02 => {
                sub.volume = f;
                sub.changes |= CHANGE_VOLUME;
            }
            0x03 => {
                sub.new_pan = s8 as u8;
                sub.changes |= CHANGE_PAN;
            }
            0x07 => {
                sub.pan_weight = s8 as u8;
                sub.changes |= CHANGE_PAN;
            }
            0x04 => {
                sub.frequency_scale = f;
                sub.changes |= CHANGE_FREQ;
            }
            0x05 => sub.reverb = s8 as u8,
            0x06 => {
                if let Some(p) = sub.port.get_mut(arg2 as usize) {
                    *p = s8;
                }
            }
            0x08 => sub.muted = s8 != 0,
            0x09 => sub.mute_flags = s8 as u8,
            0x0A => {
                sub.vibrato.depth_target = (s32 as u8) as u16 * 8;
                sub.vibrato.depth_delay = 1;
            }
            0x0B => {
                sub.vibrato.rate_target = (s32 as u8) as u16 * 32;
                sub.vibrato.rate_delay = 1;
            }
            0x0F => sub.sample_start = s32 as u16,
            0x15 => sub.transposition = s8,
            _ => {}
        }
    }

    /// Fades group `g` out over `updates`.
    pub fn fade_out(&mut self, g: usize, updates: u16) {
        if let Some(gr) = self.groups.get_mut(g) {
            if updates == 0 {
                self.release_group(g);
                return;
            }
            gr.state = STATE_FADE_OUT;
            gr.fade_timer = updates;
            gr.fade_velocity = -gr.fade_volume / updates as f32;
        }
    }

    fn rand(&mut self) -> u32 {
        // Nap_GetRandom: an LCG.
        self.random = self.random.wrapping_mul(1_103_515_245).wrapping_add(12345);
        self.random >> 8
    }

    /// One update (a quarter frame): scripts, control, voices, then
    /// `SAMPLES_PER_UPDATE` stereo samples added into `out` (interleaved).
    pub fn update(&mut self, out: &mut [f32]) {
        self.seed = self
            .seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        self.advance_levels();
        for g in 0..=GROUPS {
            if self.groups[g].enabled {
                self.group_seq(g);
                self.main_ctrl(g);
            }
        }
        self.finish_triggers();
        self.update_voices();
        self.mix(out);
    }

    // ---- scripts ----

    fn read_u8(seq: &[u8], m: &mut Macro) -> u8 {
        let v = seq.get(m.pc).copied().unwrap_or(0xFF);
        m.pc += 1;
        v
    }

    fn read_s16(seq: &[u8], m: &mut Macro) -> i16 {
        let hi = Self::read_u8(seq, m);
        let lo = Self::read_u8(seq, m);
        i16::from_be_bytes([hi, lo])
    }

    fn read_len(seq: &[u8], m: &mut Macro) -> u16 {
        let d = Self::read_u8(seq, m) as u16;
        if d & 0x80 != 0 {
            ((d << 8) & 0x7F00) | Self::read_u8(seq, m) as u16
        } else {
            d
        }
    }

    /// `SCOM_TABLE` for 0xA0..0xFF: argument count and which are s16.
    fn scom(cmd: u8) -> (usize, [bool; 3]) {
        let (n, wide): (usize, [bool; 3]) = match cmd {
            0xA0 | 0xA2 | 0xB0 | 0xB2 | 0xB7 | 0xBC | 0xBD | 0xC2 | 0xCB | 0xCE | 0xCF | 0xDA
            | 0xDE | 0xE7 => (1, [true, false, false]),
            0xA4
            | 0xA7
            | 0xB3
            | 0xB8..=0xBA
            | 0xBE
            | 0xC1
            | 0xC6
            | 0xC8..=0xCA
            | 0xCC
            | 0xCD
            | 0xD0..=0xD9
            | 0xDB
            | 0xDC
            | 0xDD
            | 0xDF
            | 0xE0
            | 0xE3
            | 0xE5
            | 0xE6
            | 0xE9
            | 0xED
            | 0xEE
            | 0xF1..=0xF4
            | 0xF8 => (1, [false; 3]),
            0xA6 | 0xBB | 0xC7 => (2, [false, true, false]),
            0xEB => (2, [false; 3]),
            0xEF => (2, [true, false, false]),
            0xE1 | 0xE2 | 0xE8 => (3, [false; 3]),
            0xF5 | 0xF9..=0xFC => (1, [true, false, false]),
            _ => (0, [false; 3]),
        };
        (n, wide)
    }

    /// `Common_Com`: control flow shared by all three script levels.
    /// `Some(n)`: stop for `n` ticks; `Some(-1)`: the script ended.
    fn common(seq: &[u8], m: &mut Macro, cmd: u8, arg: u16) -> Option<i32> {
        match cmd {
            0xFF => {
                if m.depth == 0 {
                    return Some(-1);
                }
                m.depth -= 1;
                m.pc = m.stack[m.depth];
            }
            0xFD => return Some(Self::read_len(seq, m) as i32),
            0xFE => return Some(1),
            0xFC => {
                if m.depth < 4 {
                    m.stack[m.depth] = m.pc;
                    m.depth += 1;
                }
                m.pc = arg as usize;
            }
            0xF8 => {
                if m.depth < 4 {
                    m.loops[m.depth] = arg as u8;
                    m.stack[m.depth] = m.pc;
                    m.depth += 1;
                }
            }
            0xF7 => {
                if m.depth > 0 {
                    let i = m.depth - 1;
                    m.loops[i] = m.loops[i].wrapping_sub(1);
                    if m.loops[i] != 0 {
                        m.pc = m.stack[i];
                    } else {
                        m.depth -= 1;
                    }
                }
            }
            0xF6 => m.depth = m.depth.saturating_sub(1),
            0xF5 | 0xF9 | 0xFA | 0xFB => {
                let skip = (cmd == 0xFA && m.value != 0)
                    || (cmd == 0xF9 && m.value >= 0)
                    || (cmd == 0xF5 && m.value < 0);
                if !skip {
                    m.pc = arg as usize;
                }
            }
            0xF2..=0xF4 => {
                let skip = (cmd == 0xF3 && m.value != 0) || (cmd == 0xF2 && m.value >= 0);
                if !skip {
                    m.pc = (m.pc as isize + (arg as u8 as i8) as isize) as usize;
                }
            }
            _ => {}
        }
        None
    }

    fn group_seq(&mut self, g: usize) {
        let max_tempo = self.max_tempo;
        {
            let gr = &mut self.groups[g];
            if gr.muted && gr.mute_flags & MUTE_STOP_SCRIPT != 0 {
                return;
            }
            let tempo = (gr.tempo + gr.tempo_change).min(max_tempo);
            gr.tempo_acc += tempo;
            if gr.tempo_acc < max_tempo {
                return;
            }
            gr.tempo_acc -= max_tempo;
            if gr.stop_script {
                return;
            }
        }
        if self.groups[g].delay > 1 {
            self.groups[g].delay -= 1;
        } else {
            self.groups[g].recalc_volume = true;
            loop {
                let gr = &mut self.groups[g];
                let mut m = std::mem::take(&mut gr.m);
                let seq = &gr.seq;
                let cmd = Self::read_u8(seq, &mut m);
                if cmd >= 0xF2 {
                    let (n, wide) = Self::scom(cmd);
                    let arg = if n == 1 {
                        if wide[0] {
                            Self::read_s16(seq, &mut m) as u16
                        } else {
                            Self::read_u8(seq, &mut m) as u16
                        }
                    } else {
                        0
                    };
                    let r = Self::common(seq, &mut m, cmd, arg);
                    gr.m = m;
                    match r {
                        Some(-1) => {
                            self.release_group(g);
                            return;
                        }
                        Some(d) => {
                            self.groups[g].delay = d as u16;
                            break;
                        }
                        None => continue,
                    }
                }
                let lo = (cmd & 0xF) as usize;
                let mut stop = false;
                match cmd {
                    0xF1 | 0xF0 => {
                        if cmd == 0xF1 {
                            Self::read_u8(seq, &mut m);
                        }
                    }
                    0xDF => {
                        gr.transposition = 0;
                        gr.transposition = gr
                            .transposition
                            .wrapping_add(Self::read_u8(seq, &mut m) as i8);
                    }
                    0xDE => {
                        gr.transposition = gr
                            .transposition
                            .wrapping_add(Self::read_u8(seq, &mut m) as i8)
                    }
                    0xDD => {
                        gr.tempo = (Self::read_u8(seq, &mut m) as i32 * TATUMS).clamp(1, max_tempo)
                    }
                    0xDC => gr.tempo_change = Self::read_u8(seq, &mut m) as i8 as i32 * TATUMS,
                    0xDA => {
                        let state = Self::read_u8(seq, &mut m);
                        let t = Self::read_s16(seq, &mut m) as u16;
                        match state {
                            0 | STATE_FADE_IN => {
                                if gr.state != STATE_FADE_OUT {
                                    gr.stored_fade_timer = t;
                                    gr.state = state;
                                }
                            }
                            STATE_FADE_OUT => {
                                gr.fade_timer = t;
                                gr.state = state;
                                gr.fade_velocity = -gr.fade_volume / t.max(1) as f32;
                            }
                            _ => {}
                        }
                    }
                    0xDB => {
                        let v = Self::read_u8(seq, &mut m) as f32 / 127.0;
                        if gr.state == STATE_FADE_IN {
                            gr.fade_volume = 0.0;
                            gr.state = 0;
                        }
                        if gr.state == 0 {
                            gr.fade_timer = gr.stored_fade_timer;
                            if gr.fade_timer != 0 {
                                gr.fade_velocity = (v - gr.fade_volume) / gr.fade_timer as f32;
                            } else {
                                gr.fade_volume = v;
                            }
                        }
                    }
                    0xD9 => gr.fade_volume_scale = Self::read_u8(seq, &mut m) as i8 as f32 / 127.0,
                    0xD7 => {
                        let mask = Self::read_s16(seq, &mut m) as u16;
                        let bank = gr.bank;
                        for (i, s) in gr.subs.iter_mut().enumerate() {
                            if mask >> i & 1 != 0 {
                                s.bank = bank;
                                s.mute_flags = gr.mute_flags;
                            }
                        }
                    }
                    0xD6 => {
                        Self::read_s16(seq, &mut m);
                    }
                    0xD5 => gr.mute_volume_scale = Self::read_u8(seq, &mut m) as i8 as f32 / 127.0,
                    0xD4 => gr.muted = true,
                    0xD3 => gr.mute_flags = Self::read_u8(seq, &mut m),
                    0xD1 | 0xD2 => {
                        let at = Self::read_s16(seq, &mut m) as u16 as usize;
                        if cmd == 0xD2 {
                            gr.velocities = at
                        } else {
                            gr.gate_times = at
                        }
                    }
                    0xD0 => {
                        Self::read_u8(seq, &mut m);
                    }
                    0xCE => {
                        let n = Self::read_u8(seq, &mut m) as u32;
                        let r = self.seed >> 2;
                        m.value = if n == 0 {
                            (r & 0xFF) as i8
                        } else {
                            (r % n) as i8
                        };
                    }
                    0xCD => {
                        let at = Self::read_s16(seq, &mut m) as u16 as usize;
                        if m.value != -1 && m.depth != 3 {
                            let p = at + (m.value as u8 as usize) * 2;
                            m.stack[m.depth] = m.pc;
                            m.depth += 1;
                            m.pc = seq
                                .get(p..p + 2)
                                .map_or(m.pc, |b| u16::from_be_bytes([b[0], b[1]]) as usize);
                        }
                    }
                    0xCC => m.value = Self::read_u8(seq, &mut m) as i8,
                    0xC9 => m.value &= Self::read_u8(seq, &mut m) as i8,
                    0xC8 => m.value = m.value.wrapping_sub(Self::read_u8(seq, &mut m) as i8),
                    0xC7 => {
                        let add = Self::read_u8(seq, &mut m);
                        let at = Self::read_s16(seq, &mut m) as u16 as usize;
                        let v = (m.value as u8).wrapping_add(add);
                        if let Some(b) = gr.seq.get_mut(at) {
                            *b = v;
                        }
                    }
                    0xC2 => {
                        let at = Self::read_s16(seq, &mut m) as u16 as usize;
                        if m.value != -1 {
                            let p = at + (m.value as u8 as usize) * 2;
                            m.pc = seq
                                .get(p..p + 2)
                                .map_or(m.pc, |b| u16::from_be_bytes([b[0], b[1]]) as usize);
                        }
                    }
                    0xC6 => {
                        gr.stop_script = true;
                        stop = true;
                    }
                    0xC5 => {
                        Self::read_s16(seq, &mut m);
                    }
                    0xEF => {
                        Self::read_s16(seq, &mut m);
                        Self::read_u8(seq, &mut m);
                    }
                    0xC4 => {
                        let target = Self::read_u8(seq, &mut m);
                        let id = Self::read_u8(seq, &mut m) as usize;
                        let target = if target == 0xFF { g } else { target as usize };
                        gr.m = m;
                        self.start_sequence(target, id);
                        if target == g {
                            return;
                        }
                        continue;
                    }
                    0xC3 => {
                        let at = Self::read_s16(seq, &mut m) as u16 as usize;
                        if m.value != -1 {
                            let p = at + (m.value as u8 as usize) * 2;
                            let mut mask = seq
                                .get(p..p + 2)
                                .map_or(0, |b| u16::from_be_bytes([b[0], b[1]]));
                            for s in gr.subs.iter_mut() {
                                s.muted = mask & 1 != 0;
                                mask >>= 1;
                            }
                        }
                    }
                    0xBE => {
                        let n = Self::read_u8(seq, &mut m);
                        if n > 4 {
                            stop = true;
                        }
                    }
                    _ => match cmd & 0xF0 {
                        0x00 => m.value = (!gr.subs[lo].enabled) as i8,
                        0x50 => m.value = m.value.wrapping_sub(gr.port[lo & 7]),
                        0x70 => gr.port[lo & 7] = m.value,
                        0x80 => {
                            m.value = gr.port[lo & 7];
                            if lo < 2 {
                                gr.port[lo] = -1;
                            }
                        }
                        0x40 => {
                            gr.m = m;
                            self.release_sub(g, lo);
                            continue;
                        }
                        0x90 => {
                            let at = Self::read_s16(seq, &mut m) as u16 as usize;
                            gr.m = m;
                            self.open_sub(g, lo, at);
                            continue;
                        }
                        0xA0 => {
                            let rel = Self::read_s16(seq, &mut m);
                            let at = (m.pc as isize + rel as isize) as usize;
                            gr.m = m;
                            self.open_sub(g, lo, at);
                            continue;
                        }
                        0xB0 => {
                            // SeqLoad: everything is in memory already.
                            Self::read_u8(seq, &mut m);
                            Self::read_s16(seq, &mut m);
                            gr.port[lo & 7] = 0;
                        }
                        0x60 => {
                            // MK_load: likewise done at once.
                            Self::read_u8(seq, &mut m);
                            Self::read_u8(seq, &mut m);
                            gr.port[lo & 7] = 0;
                        }
                        _ => {}
                    },
                }
                gr.m = m;
                if stop {
                    break;
                }
            }
        }
        for s in 0..SUBTRACKS {
            if self.groups[g].subs[s].enabled {
                self.sub_seq(g, s);
            }
        }
    }

    fn open_sub(&mut self, g: usize, s: usize, at: usize) {
        for l in 0..LAYERS {
            self.close_layer(g, s, l);
        }
        let sub = &mut self.groups[g].subs[s];
        sub.enabled = true;
        sub.finished = false;
        sub.m = Macro {
            pc: at,
            ..Default::default()
        };
        sub.delay = 0;
    }

    fn release_sub(&mut self, g: usize, s: usize) {
        self.groups[g].subs[s].finished = true;
        for l in 0..LAYERS {
            self.close_layer(g, s, l);
        }
        self.groups[g].subs[s].enabled = false;
    }

    fn release_group(&mut self, g: usize) {
        if g == crate::audio::group::SE {
            self.triggers = crate::audio::TrgSeManager::new();
            self.levels = crate::audio::LevSeManager::new();
            self.ambience = [None; crate::ambience::VOICES];
            self.level_cooldown = [0; crate::ambience::VOICES];
        }
        for s in 0..SUBTRACKS {
            self.release_sub(g, s);
        }
        let gr = &mut self.groups[g];
        gr.enabled = false;
        gr.finished = true;
    }

    fn close_layer(&mut self, g: usize, s: usize, l: usize) {
        if let Some(i) = self.groups[g].subs[s].layers[l].take() {
            self.release_layer(i);
            self.free_layers.push(i);
        }
    }

    fn release_layer(&mut self, i: usize) {
        let force = self.layers[i]
            .as_ref()
            .is_some_and(|n| self.groups[n.sub.0].finished);
        self.release_voice(i, !force);
        if let Some(n) = self.layers[i].as_mut() {
            n.enabled = false;
            n.finished = true;
        }
    }

    /// `Nas_EntryNoteTrack`.
    fn entry_layer(&mut self, g: usize, s: usize, l: usize, pc: usize) {
        let i = match self.groups[g].subs[s].layers[l] {
            Some(i) => {
                self.release_voice(i, true);
                i
            }
            None => {
                let Some(i) = self.free_layers.pop() else {
                    return;
                };
                self.groups[g].subs[s].layers[l] = Some(i);
                i
            }
        };
        let sub = &self.groups[g].subs[s];
        self.layers[i] = Some(Layer {
            sub: (g, s),
            enabled: true,
            finished: false,
            muted: false,
            continuous: false,
            continuous_released: false,
            attached: false,
            ignore_drum_pan: false,
            need_init: false,
            m: Macro {
                pc,
                ..Default::default()
            },
            delay: 0,
            gate_delay: 0,
            last_delay: 0,
            short_delay: 0,
            gate_time: 128,
            velocity_sq: 0.0,
            velocity_sq2: 0.0,
            pan: 64,
            transposition: 0,
            inst_or_wave: 0xFF,
            instrument: None,
            tuned: None,
            frequency_scale: 1.0,
            bend: 1.0,
            semitone: 0,
            sweep: Sweep::default(),
            sweep_target: 0,
            sweep_time: 0,
            adsr: AdsrSrc {
                envelope: sub.adsr.envelope.clone(),
                decay: 0,
                sustain: sub.adsr.sustain,
            },
            vibrato: Vibrato::fresh(),
            flags: 0xFFFF,
            reverb: sub.reverb,
            note_frequency: 1.0,
            note_velocity: 0.0,
            note_pan: 64,
            voice: None,
        });
    }

    fn bank(&self, id: u8) -> Option<Arc<Bank>> {
        self.data.bank(id as usize)
    }

    /// `Nas_ProgramChanger`.
    fn program(&self, bank: u8, prog: u8) -> Option<Instrument> {
        self.bank(bank)?
            .instruments
            .get(prog as usize)
            .cloned()
            .flatten()
    }

    fn sub_seq(&mut self, g: usize, s: usize) {
        let skip = {
            let gr = &self.groups[g];
            let sub = &gr.subs[s];
            sub.stop_script || (gr.muted && sub.mute_flags & MUTE_STOP_SCRIPT != 0)
        };
        if !skip {
            if self.groups[g].subs[s].delay > 1 {
                self.groups[g].subs[s].delay -= 1;
            } else {
                self.sub_commands(g, s);
            }
        }
        for l in 0..LAYERS {
            if let Some(i) = self.groups[g].subs[s].layers[l] {
                self.layer_seq(i);
            }
        }
    }

    fn sub_commands(&mut self, g: usize, s: usize) {
        loop {
            let (cmd, args, mut m) = {
                let gr = &mut self.groups[g];
                let mut m = std::mem::take(&mut gr.subs[s].m);
                let cmd = Self::read_u8(&gr.seq, &mut m);
                let mut args = [0u16; 3];
                if cmd >= 0xA0 {
                    let (n, wide) = Self::scom(cmd);
                    for (a, w) in args.iter_mut().zip(wide).take(n) {
                        *a = if w {
                            Self::read_s16(&gr.seq, &mut m) as u16
                        } else {
                            Self::read_u8(&gr.seq, &mut m) as u16
                        };
                    }
                }
                (cmd, args, m)
            };
            if cmd >= 0xF2 {
                let r = Self::common(&self.groups[g].seq, &mut m, cmd, args[0]);
                self.groups[g].subs[s].m = m;
                match r {
                    Some(-1) => {
                        self.release_sub(g, s);
                        return;
                    }
                    Some(d) => {
                        self.groups[g].subs[s].delay = d as u16;
                        return;
                    }
                    None => continue,
                }
            }
            let a8 = args[0] as u8;
            let lo = (cmd & 0xF) as usize;
            let gr = &mut self.groups[g];
            let bank = gr.bank;
            let seq_id = gr.seq_id;
            let sub = &mut gr.subs[s];
            let mut voice_set = None;
            let mut bank_pick = None;
            match cmd {
                0xEA => {
                    sub.stop_script = true;
                    sub.m = m;
                    return;
                }
                0xF1 | 0xF0 => {}
                0xC2 => sub.dyn_tbl = args[0] as usize,
                0xC5 => {
                    if m.value != -1 {
                        let p = sub.dyn_tbl + (m.value as u8 as usize) * 2;
                        m.pc = gr
                            .seq
                            .get(p..p + 2)
                            .map_or(m.pc, |b| u16::from_be_bytes([b[0], b[1]]) as usize);
                    }
                }
                0xEB => {
                    bank_pick = Some(a8);
                    voice_set = Some(args[1] as u8);
                }
                0xC1 => voice_set = Some(a8),
                0xC3 => sub.large_notes = false,
                0xC4 => sub.large_notes = true,
                0xDF => {
                    sub.volume = a8 as f32 / 127.0;
                    sub.changes |= CHANGE_VOLUME;
                }
                0xE0 => {
                    sub.volume_scale = a8 as f32 / 128.0;
                    sub.changes |= CHANGE_VOLUME;
                }
                0xDE => {
                    sub.frequency_scale = args[0] as f32 / 32768.0;
                    sub.changes |= CHANGE_FREQ;
                }
                0xD3 => {
                    sub.frequency_scale = self.t.pcent[a8.wrapping_add(128) as usize];
                    sub.changes |= CHANGE_FREQ;
                }
                0xEE => {
                    sub.frequency_scale = self.t.pcent2[a8.wrapping_add(128) as usize];
                    sub.changes |= CHANGE_FREQ;
                }
                0xDD => {
                    sub.new_pan = a8;
                    sub.changes |= CHANGE_PAN;
                }
                0xDC => {
                    sub.pan_weight = a8;
                    sub.changes |= CHANGE_PAN;
                }
                0xDB => sub.transposition = a8 as i8,
                0xDA => sub.adsr.envelope = read_envelope(&gr.seq, args[0] as usize),
                0xD9 => sub.adsr.decay = a8,
                0xD8 => {
                    sub.vibrato.depth_target = a8 as u16 * 8;
                    sub.vibrato.depth_start = 0;
                    sub.vibrato.depth_delay = 0;
                }
                0xD7 => {
                    sub.vibrato.rate_target = a8 as u16 * 32;
                    sub.vibrato.rate_start = a8 as u16 * 32;
                    sub.vibrato.rate_delay = 0;
                }
                0xE2 => {
                    sub.vibrato.depth_start = a8 as u16 * 8;
                    sub.vibrato.depth_target = (args[1] as u8) as u16 * 8;
                    sub.vibrato.depth_delay = (args[2] as u8) as u16 * 16;
                }
                0xE1 => {
                    sub.vibrato.rate_start = a8 as u16 * 32;
                    sub.vibrato.rate_target = (args[1] as u8) as u16 * 32;
                    sub.vibrato.rate_delay = (args[2] as u8) as u16 * 16;
                }
                0xE3 => sub.vibrato.delay = a8 as u16 * 16,
                0xD4 => sub.reverb = a8,
                0xC6 => bank_pick = Some(a8),
                0xC7 => {
                    let v = (m.value as u8).wrapping_add(a8);
                    if let Some(b) = gr.seq.get_mut(args[1] as usize) {
                        *b = v;
                    }
                }
                0xC8 => m.value = m.value.wrapping_sub(a8 as i8),
                0xCC => m.value = a8 as i8,
                0xC9 => m.value &= a8 as i8,
                0xCD => {
                    sub.m = m;
                    self.release_sub(g, a8 as usize & 15);
                    continue;
                }
                0xCA => {
                    sub.mute_flags = a8;
                    sub.changes |= CHANGE_VOLUME;
                }
                0xCB => {
                    let p = args[0] as usize + m.value as u8 as usize;
                    m.value = gr.seq.get(p).copied().unwrap_or(0) as i8;
                }
                0xCE => sub.dynamic_value = args[0],
                0xCF => {
                    let v = sub.dynamic_value.to_be_bytes();
                    let at = args[0] as usize;
                    if at + 1 < gr.seq.len() {
                        gr.seq[at] = v[0];
                        gr.seq[at + 1] = v[1];
                    }
                }
                0xD0 | 0xD1 | 0xBB | 0xB0 | 0xB1 | 0xB3 | 0xE5 | 0xE6 | 0xA4 => {}
                0xD2 => sub.adsr.sustain = a8,
                0xE4 => {
                    if m.value != -1 && m.depth < 4 {
                        let p = sub.dyn_tbl + (m.value as u8 as usize) * 2;
                        m.stack[m.depth] = m.pc;
                        m.depth += 1;
                        m.pc = gr
                            .seq
                            .get(p..p + 2)
                            .map_or(m.pc, |b| u16::from_be_bytes([b[0], b[1]]) as usize);
                    }
                }
                0xE7 => {
                    let d: Vec<u8> = gr
                        .seq
                        .get(args[0] as usize..args[0] as usize + 8)
                        .map_or(vec![0; 8], |d| d.to_vec());
                    sub.mute_flags = d[0];
                    priority(sub, d[2]);
                    sub.transposition = d[3] as i8;
                    sub.new_pan = d[4];
                    sub.pan_weight = d[5];
                    sub.reverb = d[6];
                    sub.changes |= CHANGE_PAN;
                }
                0xE8 => {
                    sub.mute_flags = a8;
                    priority(sub, args[2] as u8);
                    let seq = &gr.seq;
                    sub.transposition = Self::read_u8(seq, &mut m) as i8;
                    sub.new_pan = Self::read_u8(seq, &mut m);
                    sub.pan_weight = Self::read_u8(seq, &mut m);
                    sub.reverb = Self::read_u8(seq, &mut m);
                    Self::read_u8(seq, &mut m);
                    sub.changes |= CHANGE_PAN;
                }
                0xEC => {
                    sub.frequency_scale = 1.0;
                    sub.vibrato = Vibrato::default();
                    sub.adsr.sustain = 0;
                    sub.velocity_variance = 0;
                    sub.gate_variance = 0;
                    sub.sample_start = 0;
                }
                0xE9 => priority(sub, a8),
                0xED => {}
                0xB2 => {
                    let p = args[0] as usize + (m.value as u8 as usize) * 2;
                    sub.dynamic_value = gr
                        .seq
                        .get(p..p + 2)
                        .map_or(0, |b| u16::from_be_bytes([b[0], b[1]]));
                }
                0xB4 => sub.dyn_tbl = sub.dynamic_value as usize,
                0xB5 => {
                    let p = sub.dyn_tbl + (m.value as u8 as usize) * 2;
                    sub.dynamic_value = gr
                        .seq
                        .get(p..p + 2)
                        .map_or(0, |b| u16::from_be_bytes([b[0], b[1]]));
                }
                0xB6 => {
                    let p = sub.dyn_tbl + m.value as u8 as usize;
                    m.value = gr.seq.get(p).copied().unwrap_or(0) as i8;
                }
                0xB7 | 0xB8 => {
                    let r = self.random.wrapping_mul(1_103_515_245).wrapping_add(12345);
                    self.random = r;
                    let r = r >> 8;
                    let v = if args[0] == 0 { r } else { r % args[0] as u32 };
                    if cmd == 0xB7 {
                        sub.dynamic_value = v as u16
                    } else {
                        m.value = v as i8
                    }
                }
                0xB9 => sub.velocity_variance = a8,
                0xBA => sub.gate_variance = a8,
                0xBC => sub.dynamic_value = sub.dynamic_value.wrapping_add(args[0]),
                0xA8 => {
                    let d = sub.dynamic_value;
                    sub.dynamic_value = (d & 0x7F) | 0x8000 | ((d & 0x7FC0) << 1);
                }
                0xBD => sub.sample_start = args[0],
                0xBE => {}
                0xA0..=0xA3 => {}
                0xA5 => m.value = m.value.wrapping_add(sub.index as i8),
                0xA6 => {
                    let at = args[1] as usize + sub.index;
                    let v = (m.value as u8).wrapping_add(a8);
                    if let Some(b) = gr.seq.get_mut(at) {
                        *b = v;
                    }
                }
                0xA7 => {
                    let shift = (a8 & 0xF) as u32;
                    let top = m.value as u8 & 0x80;
                    let mut v = if a8 & 0x80 == 0 {
                        (m.value as u8) << shift
                    } else {
                        (m.value as u8) >> shift
                    };
                    if a8 & 0x40 != 0 {
                        v = (v & 0x7F) | top;
                    }
                    m.value = v as i8;
                }
                0x70..=0x9F => {
                    let l = if cmd & 0xF8 != 0x70 && (cmd & 7) >= 4 {
                        0
                    } else {
                        (cmd & 7) as usize
                    };
                    match cmd & 0xF8 {
                        0x80 => {
                            m.value = sub.layers[l]
                                .and_then(|i| self.layers[i].as_ref().map(|n| n.finished as i8))
                                .unwrap_or(-1);
                        }
                        0x88 => {
                            let at = Self::read_s16(&gr.seq, &mut m) as u16 as usize;
                            sub.m = m;
                            self.entry_layer(g, s, l, at);
                            continue;
                        }
                        0x90 => {
                            sub.m = m;
                            self.close_layer(g, s, l);
                            continue;
                        }
                        0x98 => {
                            if m.value != -1 {
                                let p = sub.dyn_tbl + (m.value as u8 as usize) * 2;
                                let at = gr
                                    .seq
                                    .get(p..p + 2)
                                    .map_or(0, |b| u16::from_be_bytes([b[0], b[1]]) as usize);
                                sub.m = m;
                                self.entry_layer(g, s, l, at);
                                continue;
                            }
                        }
                        0x70 => sub.port[l] = m.value,
                        0x78 => {
                            let rel = Self::read_s16(&gr.seq, &mut m);
                            let at = (m.pc as isize + rel as isize) as usize;
                            sub.m = m;
                            self.entry_layer(g, s, l, at);
                            continue;
                        }
                        _ => {}
                    }
                }
                _ => match cmd & 0xF0 {
                    0x00 => {
                        sub.delay = lo as u16;
                        if lo != 0 {
                            sub.m = m;
                            return;
                        }
                    }
                    // VoiceLoad: samples are always ready (done = 2).
                    0x10 => sub.port[lo & 7] = 2,
                    0x60 => {
                        m.value = sub.port[lo & 7];
                        if lo < 2 {
                            sub.port[lo] = -1;
                        }
                    }
                    0x50 => m.value = m.value.wrapping_sub(sub.port[lo & 7]),
                    0x20 => {
                        let at = Self::read_s16(&gr.seq, &mut m) as u16 as usize;
                        sub.m = m;
                        self.open_sub(g, lo, at);
                        continue;
                    }
                    0x30 => {
                        let p = Self::read_u8(&gr.seq, &mut m) as usize & 7;
                        let v = m.value;
                        sub.m = m;
                        gr.subs[lo].port[p] = v;
                        continue;
                    }
                    0x40 => {
                        let p = Self::read_u8(&gr.seq, &mut m) as usize & 7;
                        m.value = gr.subs[lo].port[p];
                    }
                    _ => {}
                },
            }
            if let Some(b) = bank_pick {
                // The bank, by reverse index into the sequence's map list.
                let id = if bank != 0xFF {
                    let map = self.data.map();
                    let at = map
                        .get(seq_id * 2..seq_id * 2 + 2)
                        .map_or(0, |w| u16::from_be_bytes([w[0], w[1]]) as usize);
                    let n = map.get(at).copied().unwrap_or(0) as usize;
                    map.get(at + n - (b as usize).min(n)).copied().unwrap_or(b)
                } else {
                    b
                };
                if self.data.bank(id as usize).is_some() {
                    self.groups[g].subs[s].bank = id;
                }
            }
            if let Some(v) = voice_set {
                self.sub_voice_set(g, s, v);
            }
            self.groups[g].subs[s].m = m;
        }
    }

    /// `Nas_SubVoiceSet`.
    fn sub_voice_set(&mut self, g: usize, s: usize, id: u8) {
        let bank = self.groups[g].subs[s].bank;
        let inst = if id < 0x7E {
            self.program(bank, id)
        } else {
            None
        };
        let sub = &mut self.groups[g].subs[s];
        match id {
            0x80.. => {
                sub.inst_or_wave = id;
                sub.instrument = None;
            }
            0x7F => sub.inst_or_wave = PERCUSSION,
            0x7E => sub.inst_or_wave = SOUND_EFF,
            _ => match inst {
                Some(i) => {
                    sub.adsr.envelope = i.envelope.clone();
                    sub.adsr.decay = i.decay;
                    sub.inst_or_wave = id + 2;
                    sub.instrument = Some(i);
                }
                None => {
                    sub.inst_or_wave = 0;
                    sub.has_instrument = false;
                    return;
                }
            },
        }
        sub.has_instrument = true;
    }

    fn layer_seq(&mut self, i: usize) {
        let Some(n) = self.layers[i].as_mut() else {
            return;
        };
        if !n.enabled {
            return;
        }
        if n.delay > 1 {
            n.delay -= 1;
            if !n.muted && n.delay <= n.gate_delay {
                n.muted = true;
                self.release_voice(i, true);
            }
            return;
        }
        // __Stop_Note
        let release = !n.continuous || self.layers[i].as_ref().and_then(|n| n.voice).is_some();
        if release {
            self.release_voice(i, true);
        }
        let Some(n) = self.layers[i].as_mut() else {
            return;
        };
        if matches!(n.sweep.mode & 0x7F, 1 | 2 | 6) {
            n.sweep.mode = 0;
        }
        n.need_init = true;
        let mut cmd;
        loop {
            cmd = self.layer_commands(i);
            if cmd < 0 {
                return;
            }
            cmd = self.set_note(i, cmd as u8);
            let delay = self.layers[i].as_ref().map_or(1, |n| n.delay);
            if !(cmd < 0 && delay == 0) {
                break;
            }
        }
        let mut same = 0;
        if cmd >= 0 {
            same = self.set_voice(i, cmd as u8);
        }
        if cmd >= 0 && same >= 0 {
            self.set_channel(i, same == 1);
        }
        let n = self.layers[i].as_ref();
        if n.is_some_and(|n| n.muted && (n.voice.is_some() || n.continuous)) {
            self.release_voice(i, true);
        }
    }

    /// `__Command_Seq`: returns the note command (< 0xC1) or -1.
    fn layer_commands(&mut self, i: usize) -> i32 {
        loop {
            let Some(n) = self.layers[i].as_mut() else {
                return -1;
            };
            let (g, s) = n.sub;
            let gr = &mut self.groups[g];
            let seq = &gr.seq;
            let mut m = std::mem::take(&mut n.m);
            let cmd = Self::read_u8(seq, &mut m);
            if cmd <= 0xC0 {
                n.m = m;
                return cmd as i32;
            }
            if cmd >= 0xF2 {
                let (cnt, wide) = Self::scom(cmd);
                let arg = if cnt == 1 {
                    if wide[0] {
                        Self::read_s16(seq, &mut m) as u16
                    } else {
                        Self::read_u8(seq, &mut m) as u16
                    }
                } else {
                    0
                };
                let r = Self::common(seq, &mut m, cmd, arg);
                n.m = m;
                if r.is_none() {
                    continue;
                }
                self.release_layer(i);
                return -1;
            }
            match cmd {
                0xC1 => {
                    let v = Self::read_u8(seq, &mut m) as f32;
                    n.velocity_sq = v * v / (127.0 * 127.0);
                }
                0xCA => n.pan = Self::read_u8(seq, &mut m),
                0xC2 => n.transposition = Self::read_u8(seq, &mut m) as i8,
                0xC9 => n.gate_time = Self::read_u8(seq, &mut m),
                0xC4 | 0xC5 => {
                    n.continuous = cmd == 0xC4;
                    n.continuous_released = false;
                    n.m = m;
                    self.release_voice(i, true);
                    continue;
                }
                0xC3 => n.short_delay = Self::read_len(seq, &mut m),
                0xC6 => {
                    let id = Self::read_u8(seq, &mut m);
                    if id >= 0x7E {
                        n.inst_or_wave = match id {
                            0x7E => SOUND_EFF,
                            0x7F => PERCUSSION,
                            _ => {
                                n.instrument = None;
                                id
                            }
                        };
                        if id == 0xFF {
                            n.adsr.decay = 0;
                        }
                    } else {
                        let bank = gr.subs[s].bank;
                        n.m = m;
                        let inst = self.program(bank, id);
                        let Some(n) = self.layers[i].as_mut() else {
                            return -1;
                        };
                        match inst {
                            Some(inst) => {
                                n.adsr.envelope = inst.envelope.clone();
                                n.adsr.decay = inst.decay;
                                n.instrument = Some(inst);
                                n.inst_or_wave = id + 2;
                            }
                            None => {
                                n.instrument = None;
                                n.inst_or_wave = 0xFF;
                            }
                        }
                        continue;
                    }
                }
                0xC7 => {
                    n.sweep.mode = Self::read_u8(seq, &mut m);
                    let mut t = Self::read_u8(seq, &mut m);
                    t = t
                        .wrapping_add(gr.subs[s].transposition as u8)
                        .wrapping_add(n.transposition as u8)
                        .wrapping_add(gr.transposition as u8);
                    if !matches!(n.sweep.mode & 0x7F, 6 | 7) && t > 127 {
                        t = 0;
                    }
                    n.sweep_target = t;
                    n.sweep_time = if n.sweep.mode & 0x80 != 0 {
                        Self::read_u8(seq, &mut m) as u16
                    } else {
                        Self::read_len(seq, &mut m)
                    };
                }
                0xC8 => n.sweep.mode = 0,
                0xCB => {
                    let at = Self::read_s16(seq, &mut m) as u16 as usize;
                    n.adsr.envelope = read_envelope(seq, at);
                    n.adsr.decay = Self::read_u8(seq, &mut m);
                }
                0xCF => n.adsr.decay = Self::read_u8(seq, &mut m),
                0xCC => n.ignore_drum_pan = true,
                0xCD => {
                    Self::read_u8(seq, &mut m);
                }
                0xCE => {
                    n.bend = self.t.pcent2[Self::read_u8(seq, &mut m).wrapping_add(128) as usize]
                }
                0xF0 => n.flags &= !(Self::read_s16(seq, &mut m) as u16),
                0xF1 => {
                    Self::read_u8(seq, &mut m);
                }
                _ => match cmd & 0xF0 {
                    0xD0 => {
                        let v = if gr.velocities == usize::MAX {
                            self.t.velocities[(cmd & 0xF) as usize]
                        } else {
                            seq.get(gr.velocities + (cmd & 0xF) as usize)
                                .copied()
                                .unwrap_or(0)
                        } as f32;
                        n.velocity_sq = v * v / (127.0 * 127.0);
                    }
                    0xE0 => {
                        n.gate_time = if gr.gate_times == usize::MAX {
                            self.t.gate_times[(cmd & 0xF) as usize]
                        } else {
                            seq.get(gr.gate_times + (cmd & 0xF) as usize)
                                .copied()
                                .unwrap_or(0)
                        };
                    }
                    _ => {}
                },
            }
            let Some(n) = self.layers[i].as_mut() else {
                return -1;
            };
            n.m = m;
        }
    }

    /// `__SetNote`: delay and velocity of a note command; returns the
    /// semitone or -1.
    fn set_note(&mut self, i: usize, cmd: u8) -> i32 {
        let seed = self.seed;
        let Some(n) = self.layers[i].as_mut() else {
            return -1;
        };
        let (g, s) = n.sub;
        let gr = &self.groups[g];
        let sub = &gr.subs[s];
        let mut m = std::mem::take(&mut n.m);
        let seq = &gr.seq;
        if cmd == 0xC0 {
            n.delay = Self::read_len(seq, &mut m);
            n.muted = true;
            n.continuous_released = false;
            n.m = m;
            return -1;
        }
        n.muted = false;
        let delay;
        if sub.large_notes {
            let velocity;
            match cmd & 0xC0 {
                0x00 => {
                    delay = Self::read_len(seq, &mut m);
                    velocity = Self::read_u8(seq, &mut m);
                    n.gate_time = Self::read_u8(seq, &mut m);
                    n.last_delay = delay;
                }
                0x40 => {
                    delay = Self::read_len(seq, &mut m);
                    velocity = Self::read_u8(seq, &mut m);
                    n.gate_time = 0;
                    n.last_delay = delay;
                }
                _ => {
                    delay = n.last_delay;
                    velocity = Self::read_u8(seq, &mut m);
                    n.gate_time = Self::read_u8(seq, &mut m);
                }
            }
            let v = if velocity > 127 {
                127.0
            } else {
                velocity as f32
            };
            n.velocity_sq = v * v / (127.0 * 127.0);
        } else {
            delay = match cmd & 0xC0 {
                0x00 => {
                    let d = Self::read_len(seq, &mut m);
                    n.last_delay = d;
                    d
                }
                0x40 => n.short_delay,
                _ => n.last_delay,
            };
        }
        n.m = m;
        let note = cmd & 0x3F;
        n.velocity_sq2 = if sub.velocity_variance != 0 {
            let mut d = n.velocity_sq * (seed % sub.velocity_variance as u32) as f32 / 100.0;
            if seed & 0x8000 != 0 {
                d = -d;
            }
            (n.velocity_sq + d).clamp(0.0, 1.0)
        } else {
            n.velocity_sq
        };
        n.delay = delay;
        n.gate_delay = ((n.gate_time as u32 * delay as u32) >> 8) as u16;
        if sub.gate_variance != 0 && sub.velocity_variance != 0 {
            // @BUG kept: the variance read is the velocity's.
            let mut d = (n.gate_delay as u32 * (seed % sub.velocity_variance as u32) / 100) as i32;
            if seed & 0x4000 != 0 {
                d = -d;
            }
            n.gate_delay = (n.gate_delay as i32 + d).clamp(0, n.delay as i32) as u16;
        }
        if gr.muted && sub.mute_flags & (MUTE_STOP_SUBTRACK | MUTE_STOP_NOTE) != 0 {
            n.muted = true;
            return -1;
        }
        note as i32
    }

    /// `__SetVoice`: picks the sample and pitch; returns 1 if the sample
    /// is the same as before, 0 if not, -1 to skip.
    fn set_voice(&mut self, i: usize, semitone: u8) -> i32 {
        let Some(n) = self.layers[i].as_ref() else {
            return -1;
        };
        let (g, s) = n.sub;
        let gr = &self.groups[g];
        let sub = &gr.subs[s];
        let mut inst_or_wave = n.inst_or_wave;
        if inst_or_wave == 0xFF {
            if !sub.has_instrument {
                return -1;
            }
            inst_or_wave = sub.inst_or_wave;
        }
        let bank = self.bank(sub.bank);
        let sub_muted = sub.muted;
        let sub_transposition = sub.transposition;
        let gr_transposition = gr.transposition;
        let tempo = gr.tempo;
        let sub_inst = sub.instrument.clone();
        let pitch_of = |st: usize| self.t.pitch[st.min(127)];
        let mut same = true;
        let n = self.layers[i].as_mut().unwrap();
        match inst_or_wave {
            PERCUSSION => {
                let mut st = semitone.wrapping_add(n.transposition as u8);
                if n.flags & ADD_SUB_TRANSPOSE != 0 {
                    st = st.wrapping_add(sub_transposition as u8);
                }
                n.semitone = st;
                let drum = bank
                    .as_ref()
                    .and_then(|b| b.drums.get(st as usize).cloned().flatten());
                let Some(drum) = drum.filter(|_| !sub_muted) else {
                    n.muted = true;
                    return -1;
                };
                n.adsr.envelope = drum.envelope.clone();
                n.adsr.decay = drum.decay;
                if !n.ignore_drum_pan {
                    n.pan = drum.pan;
                }
                n.frequency_scale = drum.sample.tuning;
                n.tuned = Some(drum.sample);
            }
            SOUND_EFF => {
                let id = semitone as usize + ((n.transposition as u8 as usize) << 6);
                n.semitone = semitone;
                let fx = bank.as_ref().and_then(|b| b.sfx.get(id).cloned().flatten());
                let Some(fx) = fx.filter(|_| !sub_muted) else {
                    n.muted = true;
                    return -1;
                };
                n.frequency_scale = fx.tuning;
                n.tuned = Some(fx);
            }
            _ => {
                let mut st = semitone
                    .wrapping_add(gr_transposition as u8)
                    .wrapping_add(n.transposition as u8);
                if n.flags & ADD_SUB_TRANSPOSE != 0 {
                    st = st.wrapping_add(sub_transposition as u8);
                }
                n.semitone = st;
                if st > 127 || sub_muted {
                    n.muted = true;
                    return -1;
                }
                let inst = if n.inst_or_wave == 0xFF {
                    sub_inst
                } else {
                    n.instrument.clone()
                };
                let pick = |inst: &Instrument, k: u8| {
                    if k < inst.range[0] {
                        inst.low.clone()
                    } else if k <= inst.range[1] {
                        inst.normal.clone()
                    } else {
                        inst.high.clone()
                    }
                };
                if n.sweep.mode != 0 {
                    let mode = n.sweep.mode & 0x7F;
                    let key = if matches!(mode, 6 | 7) {
                        st
                    } else {
                        st.max(n.sweep_target)
                    };
                    let tuning = match inst.as_ref().and_then(|inst| pick(inst, key)) {
                        Some(t) => {
                            same = n
                                .tuned
                                .as_ref()
                                .is_some_and(|o| Arc::ptr_eq(&o.pcm, &t.pcm));
                            let tu = t.tuning;
                            n.tuned = Some(t);
                            tu
                        }
                        None => {
                            n.tuned = None;
                            1.0
                        }
                    };
                    let t0 = tuning * pitch_of(st as usize);
                    let mut st2 = n.sweep_target as i8 as i32;
                    if matches!(mode, 6 | 7) {
                        st2 += st as i32;
                    }
                    let t1 = tuning * pitch_of(st2.clamp(0, 127) as usize);
                    let (f, f2) = match mode {
                        1 | 3 | 5 | 6 | 7 => (t1, t0),
                        2 | 4 => (t0, t1),
                        _ => (t0, t0),
                    };
                    n.sweep.extent = f2 / f - 1.0;
                    let mut speed = if n.sweep.mode & 0x80 != 0 {
                        let sp = tempo * 0x8000 / self.max_tempo;
                        if n.delay != 0 {
                            sp * 0x100 / (n.delay as i32 * n.sweep_time.max(1) as i32)
                        } else {
                            sp
                        }
                    } else {
                        (2 * 0x10000) / (n.sweep_time.max(1) as i32 * UPDATES_PER_FRAME as i32)
                    };
                    speed = speed.clamp(1, 0x7FFF);
                    n.sweep.speed = speed as u32;
                    n.sweep.current = 0;
                    n.frequency_scale = f;
                    if mode == 5 {
                        n.sweep_target = st;
                    }
                } else {
                    match inst.as_ref().and_then(|inst| pick(inst, st)) {
                        Some(t) => {
                            same = n
                                .tuned
                                .as_ref()
                                .is_some_and(|o| Arc::ptr_eq(&o.pcm, &t.pcm));
                            n.frequency_scale = pitch_of(st as usize) * t.tuning;
                            n.tuned = Some(t);
                        }
                        None => {
                            n.tuned = None;
                            n.frequency_scale = pitch_of(st as usize);
                        }
                    }
                }
            }
        }
        n.frequency_scale *= n.bend;
        if n.delay == 0 {
            let end = n.tuned.as_ref().map_or(0.0, |t| t.pcm.loop_end as f32);
            let time =
                (end * tempo as f32 * self.tick_factor / n.frequency_scale).min(0x7FFE as f32);
            n.delay = time as u16 + 1;
            n.gate_delay = 0;
        }
        same as i32
    }

    /// `__SetChannel`.
    fn set_channel(&mut self, i: usize, same: bool) {
        let Some(n) = self.layers[i].as_ref() else {
            return;
        };
        if n.continuous && n.continuous_released {
            return;
        }
        let keep = n.continuous
            && n.attached
            && same
            && n.voice.is_some_and(|v| self.voices[v].current == Some(i));
        if !keep {
            if !same {
                self.release_voice(i, true);
            }
            let v = self.allocate(i);
            if let Some(n) = self.layers[i].as_mut() {
                n.voice = v;
            }
            if let Some(v) = v {
                if self.voices[v].current == Some(i) {
                    self.mod_init(v);
                }
            }
        }
        if let Some(v) = self.layers[i].as_ref().and_then(|n| n.voice) {
            if self.voices[v].current == Some(i) {
                self.voices[v].sweep_scale = 1.0;
                self.voices[v].sweep = self.layers[i]
                    .as_ref()
                    .map(|n| n.sweep.clone())
                    .unwrap_or_default();
            }
        }
    }

    // ---- control (effect.c) ----

    fn main_ctrl(&mut self, g: usize) {
        let gr = &mut self.groups[g];
        if gr.fade_timer != 0 {
            gr.recalc_volume = true;
            gr.fade_volume = (gr.fade_volume + gr.fade_velocity).clamp(0.0, 1.0);
            gr.fade_timer -= 1;
            if gr.fade_timer == 0 && gr.state == STATE_FADE_OUT {
                self.release_group(g);
                return;
            }
        }
        if gr.recalc_volume {
            gr.applied_fade_volume = gr.fade_volume * gr.fade_volume_scale;
        }
        let recalc = gr.recalc_volume;
        for s in 0..SUBTRACKS {
            if self.groups[g].subs[s].enabled {
                self.sub_ctrl(g, s, recalc);
            }
        }
        self.groups[g].recalc_volume = false;
    }

    /// `__Nas_CallWaveProcess_Sub`.
    fn sub_ctrl(&mut self, g: usize, s: usize, recalc: bool) {
        let gr = &mut self.groups[g];
        let (fade, muted, mute_scale) = (gr.applied_fade_volume, gr.muted, gr.mute_volume_scale);
        let sub = &mut gr.subs[s];
        if sub.changes & CHANGE_VOLUME != 0 || recalc {
            let mut v = sub.volume * sub.volume_scale * fade;
            if muted && sub.mute_flags & MUTE_SOFTEN != 0 {
                v *= mute_scale;
            }
            sub.applied_volume = v * v;
        }
        if sub.changes & CHANGE_PAN != 0 {
            sub.pan = sub.new_pan as u16 * sub.pan_weight as u16;
        }
        let (changes, freq, vol, pan, weight) = (
            sub.changes,
            sub.frequency_scale,
            sub.applied_volume,
            sub.pan,
            sub.pan_weight,
        );
        let layers = sub.layers;
        sub.changes = 0;
        for i in layers.into_iter().flatten() {
            let Some(n) = self.layers[i].as_mut() else {
                continue;
            };
            if !n.enabled || n.voice.is_none() {
                continue;
            }
            let ch = if n.need_init {
                n.need_init = false;
                0xFF
            } else {
                changes
            };
            if ch & CHANGE_FREQ != 0 {
                n.note_frequency = if n.flags & SKIP_FREQ == 0 {
                    n.frequency_scale
                } else {
                    n.frequency_scale * freq
                };
            }
            if ch & CHANGE_VOLUME != 0 || recalc {
                n.note_velocity = if n.flags & SKIP_VOLUME == 0 {
                    n.velocity_sq2
                } else {
                    n.velocity_sq2 * vol
                };
            }
            if ch & CHANGE_PAN != 0 {
                if n.flags & SKIP_PAN == 0 {
                    // @BUG kept: the pan lands in the velocity.
                    n.note_velocity = n.pan as f32;
                } else {
                    n.note_pan = ((pan as u32 + n.pan as u32 * (128 - weight as u32)) >> 7) as u8;
                }
            }
        }
    }

    fn mod_init(&mut self, v: usize) {
        let Some(i) = self.voices[v].current else {
            return;
        };
        let Some(n) = self.layers[i].as_ref() else {
            return;
        };
        let params = if n.flags & USE_SUB_VIBRATO != 0 {
            self.groups[n.sub.0].subs[n.sub.1].vibrato.clone()
        } else {
            n.vibrato.clone()
        };
        let vib = &mut self.voices[v].vib;
        vib.active = true;
        vib.depth_timer = params.depth_delay;
        vib.depth = if params.depth_delay == 0 {
            params.depth_target
        } else {
            params.depth_start
        } as f32;
        vib.rate_timer = params.rate_delay;
        vib.rate = if params.rate_delay == 0 {
            params.rate_target
        } else {
            params.rate_start
        } as f32;
        vib.time = 0;
        vib.delay = params.delay;
        vib.params = params;
        self.voices[v].vib_scale = 1.0;
    }

    fn sweep_scale(&self, sw: &mut Sweep) -> f32 {
        sw.current = sw.current.wrapping_add(sw.speed);
        let mut lo = (sw.current >> 8) & 0xFF;
        if lo >= 127 {
            lo = 127;
            sw.mode = 0;
        }
        1.0 + sw.extent * (self.t.pcent[(lo + 128) as usize] - 1.0)
    }

    fn vibrato(&self, t: &mut VibState) -> f32 {
        if t.delay != 0 {
            t.delay -= 1;
            return 1.0;
        }
        let p = &t.params;
        if t.depth_timer != 0 {
            if t.depth_timer == 1 {
                t.depth = p.depth_target as f32;
            } else {
                t.depth += (p.depth_target as f32 - t.depth) / t.depth_timer as f32;
            }
            t.depth_timer -= 1;
        } else if p.depth_target as f32 != t.depth {
            t.depth_timer = p.depth_delay;
            if p.depth_delay == 0 {
                t.depth = p.depth_target as f32;
            }
        }
        if t.rate_timer != 0 {
            if t.rate_timer == 1 {
                t.rate = p.rate_target as f32;
            } else {
                t.rate += (p.rate_target as f32 - t.rate) / t.rate_timer as f32;
            }
            t.rate_timer -= 1;
        } else if p.rate_target as f32 != t.rate {
            t.rate_timer = p.rate_delay;
            if p.rate_delay == 0 {
                t.rate = p.rate_target as f32;
            }
        }
        if t.depth == 0.0 {
            return 1.0;
        }
        t.time = t.time.wrapping_add(t.rate as u32);
        let m = self.t.vibrato[((t.time >> 10) & 0x3F) as usize] as f32;
        let depth = 1.0 + t.depth / 4096.0;
        let inv = 1.0 / depth;
        1.0 / ((depth - inv) * (m + 32768.0) / 65536.0 + inv)
    }

    /// `Nas_EnvProcess`.
    fn envelope(e: &mut Env) -> f32 {
        let start = e.status;
        let mut guard = 0;
        loop {
            guard += 1;
            if guard > 40 {
                e.status = Adsr::Disabled;
                break;
            }
            match e.status {
                Adsr::Disabled => return 0.0,
                Adsr::Initial => {
                    e.idx = 0;
                    e.status = Adsr::Loop;
                    continue;
                }
                Adsr::Loop => {
                    let Some(&(d, v)) = e.data.as_ref().and_then(|d| d.get(e.idx)) else {
                        e.status = Adsr::Disabled;
                        break;
                    };
                    match d {
                        0 => e.status = Adsr::Disabled,
                        -1 => e.status = Adsr::Hang,
                        -2 => {
                            e.idx = v.max(0) as usize;
                            continue;
                        }
                        -3 => e.status = Adsr::Initial,
                        -4 => {
                            e.idx += 1;
                            continue;
                        }
                        _ => {
                            e.delay = (d as i32).max(1);
                            let t = v as f32 / 32767.0;
                            e.target = t * t;
                            e.velocity = (e.target - e.current) / e.delay as f32;
                            e.status = Adsr::Fade;
                            e.idx += 1;
                        }
                    }
                    if e.status != Adsr::Fade {
                        break;
                    }
                    e.current += e.velocity;
                    e.delay -= 1;
                    if e.delay <= 0 {
                        e.status = Adsr::Loop;
                    }
                    break;
                }
                Adsr::Fade => {
                    e.current += e.velocity;
                    e.delay -= 1;
                    if e.delay <= 0 {
                        e.status = Adsr::Loop;
                    }
                    break;
                }
                Adsr::Hang => break,
                Adsr::Decay | Adsr::Release => {
                    e.current -= e.fadeout;
                    if e.sustain != 0.0 && start == Adsr::Decay {
                        if e.current < e.sustain {
                            e.current = e.sustain;
                            e.delay = 128;
                            e.status = Adsr::Sustain;
                        }
                        break;
                    }
                    if e.current < 0.00001 {
                        e.current = 0.0;
                        e.status = Adsr::Disabled;
                    }
                    break;
                }
                Adsr::Sustain => {
                    e.delay -= 1;
                    if e.delay == 0 {
                        e.status = Adsr::Release;
                    }
                    break;
                }
            }
        }
        if e.decay_flag {
            e.status = Adsr::Decay;
            e.decay_flag = false;
        }
        if e.release_flag {
            e.status = Adsr::Release;
            e.release_flag = false;
        }
        e.current.clamp(0.0, 1.0)
    }

    // ---- voices (channel.c) ----

    /// `Nas_Release_Channel` (`decay`) or `_Force`.
    fn release_voice(&mut self, i: usize, decay: bool) {
        let Some(n) = self.layers[i].as_mut() else {
            return;
        };
        n.attached = false;
        let Some(v) = n.voice else { return };
        let (pitch, velocity, pan) = (n.note_frequency, n.note_velocity, n.note_pan);
        let (g, s) = n.sub;
        let decay_idx = n.adsr.decay;
        let voice = &mut self.voices[v];
        if voice.wanted == Some(i) {
            voice.wanted = None;
        }
        if voice.current != Some(i) {
            if voice.current.is_none()
                && voice.wanted.is_none()
                && voice.previous == Some(i)
                && !decay
            {
                voice.env.fadeout = 1.0 / UPDATES_PER_FRAME;
                voice.env.release_flag = true;
            }
            return;
        }
        if voice.env.status != Adsr::Decay {
            voice.params = Params {
                pitch,
                velocity,
                pan,
            };
            let sub = &self.groups[g].subs[s];
            if self.groups[g].muted && sub.mute_flags & MUTE_STOP_SAMPLES != 0 {
                voice.finished = true;
            }
            voice.priority = sub.priority2;
            voice.previous = voice.current;
            voice.current = None;
            if !decay {
                voice.env.fadeout = 1.0 / UPDATES_PER_FRAME;
                voice.env.release_flag = true;
                voice.status = 2;
            } else {
                voice.status = 1;
                voice.env.decay_flag = true;
                let idx = if decay_idx == 0 {
                    sub.adsr.decay
                } else {
                    decay_idx
                };
                voice.env.fadeout = self.decay_table[idx as usize];
                voice.env.sustain = sub.adsr.sustain as f32 * voice.env.current / 256.0;
            }
        }
    }

    /// `Nas_AllocationOnRequest`, from one shared pool: a free voice, else
    /// one already releasing at no higher priority, else a playing one at
    /// a lower priority.
    fn allocate(&mut self, i: usize) -> Option<usize> {
        let (g, s) = self.layers[i].as_ref()?.sub;
        let prio = self.groups[g].subs[s].note_priority;
        if let Some(v) = (0..self.voices.len()).find(|&v| {
            self.voices[v].current.is_none()
                && self.voices[v].wanted.is_none()
                && self.voices[v].status == 0
                && self.voices[v].env.status == Adsr::Disabled
        }) {
            self.entry_voice(v, i);
            return Some(v);
        }
        let releasing = (0..self.voices.len())
            .filter(|&v| {
                self.voices[v].current.is_none()
                    && self.voices[v].wanted.is_none()
                    && self.voices[v].priority <= prio
            })
            .min_by_key(|&v| self.voices[v].priority);
        if let Some(v) = releasing {
            self.voices[v].wanted = Some(i);
            self.voices[v].priority = prio;
            return Some(v);
        }
        let playing = (0..self.voices.len())
            .filter(|&v| self.voices[v].current.is_some() && self.voices[v].priority < prio)
            .min_by_key(|&v| self.voices[v].priority);
        if let Some(v) = playing {
            if let Some(cur) = self.voices[v].current {
                self.release_voice(cur, false);
            }
            self.voices[v].wanted = Some(i);
            return Some(v);
        }
        None
    }

    /// `Nas_EntryTrack` + `Nas_StartVoice`.
    fn entry_voice(&mut self, v: usize, i: usize) {
        let Some(n) = self.layers[i].as_mut() else {
            return;
        };
        let (g, s) = n.sub;
        let sub = &self.groups[g].subs[s];
        n.need_init = true;
        n.attached = true;
        n.voice = Some(v);
        n.note_velocity = 0.0;
        let start = match sub.sample_start {
            1 => n.tuned.as_ref().map_or(0, |t| t.pcm.loop_start),
            p => p as u32,
        };
        let env_data = if n.adsr.envelope.is_empty() {
            self.t.default_env.clone()
        } else {
            n.adsr.envelope.clone()
        };
        let sample = n.tuned.clone();
        let voice = &mut self.voices[v];
        voice.previous = None;
        voice.current = Some(i);
        voice.priority = sub.note_priority;
        voice.env = Env {
            status: Adsr::Initial,
            data: Some(env_data),
            ..Default::default()
        };
        voice.status = 0;
        voice.sample = sample;
        voice.finished = false;
        voice.pos = if voice
            .sample
            .as_ref()
            .is_some_and(|t| start < t.pcm.loop_end)
        {
            start as f64
        } else {
            0.0
        };
        voice.volume = [0.0; 2];
    }

    fn stop_voice(&mut self, v: usize) {
        let voice = &mut self.voices[v];
        voice.finished = false;
        voice.priority = 0;
        voice.status = 0;
        voice.current = None;
        voice.previous = None;
        voice.env.status = Adsr::Disabled;
        voice.env.current = 0.0;
        voice.sample = None;
    }

    /// `Nas_UpdateChannel`: each voice's envelope and target levels.
    fn update_voices(&mut self) {
        for v in 0..self.voices.len() {
            // Taken-over voices: once the old note has faded, start the new.
            let free = {
                let voice = &self.voices[v];
                voice.status >= 1 || voice.finished || voice.current.is_none()
            };
            if free && (self.voices[v].env.status == Adsr::Disabled || self.voices[v].finished) {
                if let Some(w) = self.voices[v].wanted.take() {
                    self.stop_voice(v);
                    if self.layers[w].is_some() {
                        self.entry_voice(v, w);
                        self.mod_init(v);
                        let sweep = self.layers[w]
                            .as_ref()
                            .map(|n| n.sweep.clone())
                            .unwrap_or_default();
                        self.voices[v].sweep = sweep;
                        self.voices[v].sweep_scale = 1.0;
                    }
                } else {
                    if let Some(p) = self.voices[v].current.or(self.voices[v].previous) {
                        if let Some(n) = self.layers[p].as_mut() {
                            n.continuous_released = true;
                        }
                    }
                    self.stop_voice(v);
                    continue;
                }
            }
            if let Some(i) = self.voices[v].current {
                let stolen = self.layers[i].as_ref().is_none_or(|n| n.voice != Some(v));
                if stolen && self.voices[v].status == 0 {
                    self.voices[v].env.release_flag = true;
                    self.voices[v].env.fadeout = 1.0 / UPDATES_PER_FRAME;
                    self.voices[v].priority = 1;
                    self.voices[v].status = 2;
                }
            }
            if self.voices[v].env.status == Adsr::Disabled && self.voices[v].current.is_none() {
                continue;
            }
            let mut env = std::mem::take(&mut self.voices[v].env);
            let scale = Self::envelope(&mut env);
            self.voices[v].env = env;
            let mut sweep = std::mem::take(&mut self.voices[v].sweep);
            if sweep.mode != 0 {
                self.voices[v].sweep_scale = self.sweep_scale(&mut sweep);
            }
            self.voices[v].sweep = sweep;
            let mut vib = std::mem::take(&mut self.voices[v].vib);
            if vib.active {
                self.voices[v].vib_scale = self.vibrato(&mut vib);
            }
            self.voices[v].vib = vib;
            let voice = &self.voices[v];
            let (mut pitch, mut velocity, pan) = match (
                voice.status,
                voice.current.and_then(|i| self.layers[i].as_ref()),
            ) {
                (0, Some(n)) => {
                    let (g, s) = n.sub;
                    let sub = &self.groups[g].subs[s];
                    if self.groups[g].muted && sub.mute_flags & MUTE_STOP_SAMPLES != 0 {
                        (0.0, 0.0, n.note_pan)
                    } else {
                        (n.note_frequency, n.note_velocity, n.note_pan)
                    }
                }
                _ => (voice.params.pitch, voice.params.velocity, voice.params.pan),
            };
            pitch *= voice.vib_scale.max(0.0) * voice.sweep_scale.max(0.0) * self.resample;
            velocity = (velocity * scale).clamp(0.0, 1.0);
            let p = (pan & 0x7F) as usize;
            let (l, r) = (self.t.stereo[p], self.t.stereo[0x7F - p]);
            let voice = &mut self.voices[v];
            voice.step = pitch.clamp(0.0, 3.99996) as f64;
            voice.target = [velocity * l, velocity * r];
        }
    }

    // ---- mixing ----

    fn mix(&mut self, out: &mut [f32]) {
        let n = out.len() / 2;
        for voice in self.voices.iter_mut() {
            let Some(t) = voice.sample.clone() else {
                continue;
            };
            if voice.env.status == Adsr::Disabled || voice.finished {
                voice.volume = [0.0; 2];
                continue;
            }
            let pcm = &t.pcm;
            let end = (pcm.loop_end as usize).min(pcm.data.len());
            if end == 0 {
                voice.finished = true;
                continue;
            }
            let (from, to) = (voice.volume, voice.target);
            for k in 0..n {
                let f = k as f32 / n as f32;
                let (vl, vr) = (
                    from[0] + (to[0] - from[0]) * f,
                    from[1] + (to[1] - from[1]) * f,
                );
                let p = voice.pos as usize;
                if p >= end {
                    if pcm.loop_count != 0 && (pcm.loop_start as usize) < end {
                        voice.pos -= (end - pcm.loop_start as usize) as f64;
                    } else {
                        voice.finished = true;
                        break;
                    }
                }
                let p = voice.pos as usize;
                let frac = (voice.pos - p as f64) as f32;
                let a = pcm.data[p.min(end - 1)] as f32;
                let next = if p + 1 < end {
                    p + 1
                } else if pcm.loop_count != 0 {
                    pcm.loop_start as usize
                } else {
                    p
                };
                let b = pcm.data[next.min(pcm.data.len() - 1)] as f32;
                let x = (a + (b - a) * frac) / 32768.0;
                out[2 * k] += x * vl;
                out[2 * k + 1] += x * vr;
                voice.pos += voice.step;
            }
            voice.volume = to;
        }
    }
}

/// `Nas_PriorityChanger`.
fn priority(sub: &mut Sub, p: u8) {
    if p & 0xF != 0 {
        sub.note_priority = p & 0xF;
    }
    if p >> 4 != 0 {
        sub.priority2 = p >> 4;
    }
    if sub.note_priority < 2 {
        sub.note_priority = 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct One {
        seq: Vec<u8>,
        bank: Arc<Bank>,
        map: Vec<u8>,
    }

    impl SoundData for One {
        fn sequence(&self, _: usize) -> Option<Vec<u8>> {
            Some(self.seq.clone())
        }
        fn bank(&self, _: usize) -> Option<Arc<Bank>> {
            Some(self.bank.clone())
        }
        fn map(&self) -> &[u8] {
            &self.map
        }
    }

    fn tables() -> Tables {
        let mut pitch = [0.0f32; 128];
        for (i, p) in pitch.iter_mut().enumerate() {
            *p = 2f32.powf((i as f32 - 39.0) / 12.0);
        }
        let pc = std::array::from_fn(|i| 2f32.powf((i as f32 - 128.0) / 128.0));
        Tables {
            pitch,
            pcent: pc,
            pcent2: pc,
            velocities: [127; 16],
            gate_times: [200; 16],
            default_env: Arc::from(vec![(1, 32767), (-1, 0)]),
            stereo: [0.707; 128],
            vibrato: [0; 64],
            main_script: vec![0xFF],
        }
    }

    /// A tone: one instrument with a looping square wave.
    fn tone() -> Arc<Bank> {
        let data: Vec<i16> = (0..64).map(|i| if i < 32 { 8000 } else { -8000 }).collect();
        let pcm = Arc::new(Pcm {
            data,
            loop_start: 0,
            loop_end: 64,
            loop_count: u32::MAX,
        });
        let t = Tuned { pcm, tuning: 1.0 };
        Arc::new(Bank {
            instruments: vec![Some(Instrument {
                range: [0, 127],
                decay: 240,
                envelope: Arc::from(vec![(1, 32767), (-1, 0)]),
                low: Some(t.clone()),
                normal: Some(t.clone()),
                high: Some(t),
            })],
            drums: vec![],
            sfx: vec![],
        })
    }

    #[test]
    fn a_sequence_plays_a_note_then_goes_quiet() {
        // Group: enable subtrack 0, tempo 120, start it, wait, end.
        // Subtrack 0 at 0x0C: instrument 0, volume 127, pan 64, layer 0 at
        // 0x18, wait 0x60 ticks, end. Layer: velocity 127 (short), note 39
        // for 0x30 ticks, end.
        let mut seq: Vec<u8> = vec![
            0xD7, 0x00, 0x01, // alloc subtrack 0
            0xDD, 120, // tempo
            0x90, 0x00, 0x0C, // start subtrack 0 at 0x0C
            0xFD, 0x7F, // wait 127
            0xFF, 0x00, // 0x0C: subtrack
            0xC1, 0x00, 0xDF, 127, 0xDD, 64, 0x88, 0x00, 0x18, 0xFD, 0x60, 0xFF,
        ];
        assert_eq!(seq.len(), 0x18);
        seq.extend_from_slice(&[0xC1, 127, 0xC9, 200, 39 | 0x00, 0x30, 0xFF]);
        let map = vec![0, 2, 1, 0];
        let data = Arc::new(One {
            seq,
            bank: tone(),
            map,
        });
        let mut e = Engine::new(data, tables());
        assert!(e.start_sequence(0, 0));
        let mut peak_early = 0.0f32;
        let mut peak_late = 0.0f32;
        for u in 0..400 {
            let mut buf = vec![0.0f32; SAMPLES_PER_UPDATE * 2];
            e.update(&mut buf);
            let p = buf.iter().fold(0.0f32, |m, x| m.max(x.abs()));
            if (20..60).contains(&u) {
                peak_early = peak_early.max(p);
            }
            if u > 300 {
                peak_late = peak_late.max(p);
            }
        }
        assert!(peak_early > 0.05, "the note sounds ({peak_early})");
        assert!(peak_late < 0.01, "and stops ({peak_late})");
    }
}
