//! Animalese (`game64.c_inc`: `Na_VoiceSe`, `Na_SpecChange`,
//! `Sou_VoiceMake`; `m_msg_sound.c_inc`: `mMsg_sound_voice_get`;
//! `m_npc.c`: `mNpc_GetNpcSoundSpec`). Each typed character's phoneme,
//! joined with the next two by the original's tables, goes to one of two
//! voice slots of the voice group, whose sequence (243-245, by the
//! speaker's spec) sings it.
//!
//! `tanboin_henkan`, `chouboin_henkan`, `connect_check` and
//! `boin_shiin_check` were generated from the C and checked against it
//! on every input. Melody requests route to the disc's sequence-248 fragments;
//! special NPC specs require their corresponding speaking interactions.

/// `voice_array` (`mMsg_sound_voice_get`): phoneme per message character.
#[rustfmt::skip]
const VOICE_ARRAY: [u8; 256] = [
        0x85, 0x85, 0x5D, 0x5D, 0x5D, 0x5D, 0x5D, 0x5D, 0x5F, 0x61, 0x61, 0x61, 0x61, 0x65, 0x65, 0x65, 
        0x65, 0x60, 0x6A, 0x6B, 0x6B, 0x6B, 0x6B, 0x6B, 0x85, 0x71, 0x71, 0x71, 0x71, 0x5E, 0x6C, 0x5D, 
        0x81, 0x83, 0x85, 0x5D, 0x5D, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x80, 
        0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x85, 0x85, 0x85, 0x85, 0x85, 0x82, 
        0x85, 0x5D, 0x5E, 0x5F, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 
        0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x5D, 0x85, 0x5D, 0x5D, 0x85, 
        0x5F, 0x5D, 0x5E, 0x5F, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 
        0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x61, 0x61, 0x61, 0x61, 0xFF, 
        0xFF, 0x65, 0x65, 0x65, 0x65, 0x86, 0x60, 0x6A, 0x6B, 0x6B, 0x6B, 0x6B, 0x6B, 0x85, 0x71, 0x71, 
        0x85, 0x71, 0x71, 0x75, 0x75, 0x6C, 0x75, 0x85, 0x85, 0x5D, 0x6B, 0x85, 0x85, 0x85, 0x85, 0x85, 
        0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 
        0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 
        0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 
        0x85, 0x85, 0x81, 0x81, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 
        0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 
        0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x85, 0x81
];

#[allow(unreachable_code, clippy::all)] // generated from the C
fn tanboin_henkan(a: u8, b: u8) -> u8 {
    #[allow(unused_mut)]
    let mut a = a;
    #[allow(unused_mut)]
    let mut b = b;
    match a {
        0x5d => {
            return 0x01;
        }
        0x65 => {
            return 0x14;
        }
        0x71 => {
            return 0x29;
        }
        0x61 => {
            return 0x0a;
        }
        0x6b => {
            return 0x1d;
        }
        0x6a => {
            return 0x1b;
        }
        0x53 => {
            return 0x32;
        }
        0x54 => {
            return 0x33;
        }
        0x55 | 0x70 => {
            return 0x26;
        }
        0x56 => {
            return 0x27;
        }
        0x57 => {
            return 0x34;
        }
        0x58 => {
            return 0x35;
        }
        0x59 => {
            return 0x36;
        }
        0x5a => {
            return 0x37;
        }
        0x5b => {
            return 0x10;
        }
        0x5c => {
            return 0x38;
        }
        0x5e => {
            return 0x07;
        }
        0x5f => {
            match b {
                0x65 | 0x61 => {
                    return 0x23;
                }
                _ => {
                    return 0x18;
                }
            }
            return 0x09;
        }
        0x60 => {
            return 0x09;
        }
        0x62 => {
            return 0x11;
        }
        0x63 => {
            return 0x12;
        }
        0x64 => {
            return 0x00;
        }
        0x66 => {
            return 0x17;
        }
        0x67 | 0x6d => {
            return 0x18;
        }
        0x68 => {
            return 0x19;
        }
        0x69 => {
            return 0x1a;
        }
        0x6c => {
            return 0x21;
        }
        0x6e => {
            return 0x22;
        }
        0x6f => {
            return 0x23;
        }
        0x72 => {
            return 0x2d;
        }
        0x73 => {
            return 0x2e;
        }
        0x74 => {
            return 0x31;
        }
        0x75 => {
            return 0x15;
        }
        0x76 => {
            return 0x30;
        }
        _ => {}
    }
    return a;
    #[allow(unreachable_code)]
    a
}

#[allow(unreachable_code, clippy::all)] // generated from the C
fn chouboin_henkan(a: u8, b: u8) -> u8 {
    #[allow(unused_mut)]
    let mut a = a;
    #[allow(unused_mut)]
    let mut b = b;
    match a {
        0x5d => {
            return 0x02;
        }
        0x65 | 0x75 => {
            return 0x15;
        }
        0x71 => {
            return 0x2a;
        }
        0x61 => {
            return 0x0e;
        }
        0x6b => {
            return 0x1e;
        }
        0x6a => {
            return 0x1b;
        }
        0x53 => {
            return 0x32;
        }
        0x54 => {
            return 0x33;
        }
        0x55 | 0x70 => {
            return 0x26;
        }
        0x56 => {
            return 0x27;
        }
        0x57 => {
            return 0x34;
        }
        0x58 => {
            return 0x35;
        }
        0x59 => {
            return 0x36;
        }
        0x5a => {
            return 0x37;
        }
        0x5b => {
            return 0x10;
        }
        0x5c => {
            return 0x38;
        }
        0x5e => {
            return 0x07;
        }
        0x5f => {
            match b {
                0x65 | 0x61 => {
                    return 0x23;
                }
                _ => {
                    return 0x18;
                }
            }
            return 0x09;
        }
        0x60 => {
            return 0x09;
        }
        0x62 => {
            return 0x11;
        }
        0x63 => {
            return 0x12;
        }
        0x64 => {
            return 0x00;
        }
        0x66 => {
            return 0x17;
        }
        0x67 | 0x6d => {
            return 0x18;
        }
        0x68 => {
            return 0x19;
        }
        0x69 => {
            return 0x1a;
        }
        0x6c => {
            return 0x21;
        }
        0x6e => {
            return 0x22;
        }
        0x6f => {
            return 0x23;
        }
        0x72 => {
            return 0x2d;
        }
        0x73 => {
            return 0x2e;
        }
        0x74 => {
            return 0x31;
        }
        0x76 => {
            return 0x30;
        }
        _ => {}
    }
    return a;
    #[allow(unreachable_code)]
    a
}

#[allow(unreachable_code, clippy::all)] // generated from the C
fn connect_check(a: u8, b: u8, c: u8) -> u8 {
    #[allow(unused_mut)]
    let mut a = a;
    #[allow(unused_mut)]
    let mut b = b;
    #[allow(unused_mut)]
    let mut c = c;
    match a {
        0x5d => match b {
            0x65 => {
                a = 0x0f;
            }
            0x68 => {
                a = 0x02;
            }
            0x6e => {
                a = 0x16;
            }
            0x6a => {
                a = 0x39;
            }
            0x6f => {
                a = 0x3a;
            }
            _ => {}
        },
        0x61 => match b {
            0x5d => {
                a = 0x15;
            }
            0x6e => {
                a = 0x0d;
            }
            _ => {}
        },
        0x5e => match b {
            0x61 => {
                a = 0x3b;
            }
            0x75 => {
                a = 0x3c;
            }
            _ => {}
        },
        0x5f => match b {
            0x64 => {
                a = 0x08;
            }
            _ => {}
        },
        0x60 => match b {
            0x6b => {
                a = 0x3d;
            }
            _ => {}
        },
        0x62 => match b {
            0x5d => {
                a = 0x35;
            }
            0x6b => {
                a = 0x34;
            }
            _ => {}
        },
        0x63 => match b {
            0x61 => {
                a = 0x17;
            }
            0x6b => {
                a = 0x3e;
            }
            _ => {}
        },
        0x64 => match b {
            0x5d => {
                a = 0x3f;
            }
            0x61 => {
                a = 0x40;
            }
            0x65 => {
                a = 0x41;
            }
            0x6b => {
                a = 0x42;
            }
            _ => {}
        },
        0x65 => match b {
            0x6e => {
                a = 0x0c;
            }
            0x81 => {
                a = 0x05;
            }
            0x62 => {
                a = 0x43;
            }
            0x6a => {
                a = 0x44;
            }
            0x6f => {
                a = 0x45;
            }
            0x70 => {
                a = 0x46;
            }
            _ => {}
        },
        0x67 => match b {
            0x6f => {
                a = 0x31;
            }
            _ => {}
        },
        0x69 => match b {
            0x61 => {
                a = 0x47;
            }
            0x75 => {
                a = 0x48;
            }
            _ => {}
        },
        0x6a => match b {
            0x5d => {
                a = 0x38;
            }
            0x63 => {
                a = 0x1c;
            }
            0x6b => {
                a = 0x49;
            }
            _ => {}
        },
        0x6b => match b {
            0x6b => {
                a = 0x2a;
            }
            0x73 => {
                a = 0x06;
            }
            0x75 => {
                a = 0x1f;
            }
            0x71 => {
                a = 0x20;
            }
            0x6e => {
                a = 0x2c;
            }
            0x62 => {
                a = 0x4a;
            }
            0x64 => {
                a = 0x4b;
            }
            0x67 => {
                a = 0x4c;
            }
            0x6a => {
                a = 0x4d;
            }
            _ => {}
        },
        0x6e => match b {
            0x61 => {
                a = 0x0e;
            }
            _ => {}
        },
        0x6f => match b {
            0x61 => {
                a = 0x37;
            }
            0x64 => {
                a = 0x24;
            }
            0x65 => {
                a = 0x36;
            }
            0x71 => {
                a = 0x25;
            }
            0x6b => {
                a = 0x4f;
            }
            _ => {}
        },
        0x70 => match b {
            0x64 => {
                a = 0x28;
            }
            0x6b => {
                a = 0x50;
            }
            _ => {}
        },
        0x71 => match b {
            0x6c => {
                a = 0x51;
            }
            _ => {}
        },
        0x73 => match b {
            0x61 => {
                a = 0x52;
            }
            0x6b => {
                a = 0x33;
            }
            _ => {}
        },
        0x75 => match b {
            0x6b => match c {
                0x71 => {
                    a = 0x77;
                }
                _ => {}
            },
            _ => {}
        },
        0x76 => match b {
            0x61 => {
                a = 0x32;
            }
            _ => {}
        },
        _ => {}
    }
    return a;
    #[allow(unreachable_code)]
    a
}

#[allow(unreachable_code, clippy::all)] // generated from the C
fn boin_shiin_check(a: u8) -> u8 {
    #[allow(unused_mut)]
    let mut a = a;
    match a {
        0x5d | 0x61 | 0x65 | 0x6a | 0x6b | 0x71 => {
            return 0x00;
        }
        0x5e | 0x5f | 0x60 | 0x62 | 0x63 | 0x64 | 0x66 | 0x67 | 0x68 | 0x69 | 0x6c | 0x6d
        | 0x6e | 0x6f | 0x70 | 0x72 | 0x73 | 0x74 | 0x75 | 0x76 => {
            return 0x01;
        }
        _ => {}
    }
    return 0x02;
    #[allow(unreachable_code)]
    a
}

/// `NA_SE_BEBE` (`MONO(0x6D)`): the click voice and some specials.
pub const SE_BEBE: u16 = 0x1000 | 0x6D;

/// What `Na_VoiceSe` asks the engine to do.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VoiceOut {
    /// `Sou_VoiceStart`: phoneme into voice slot `slot`'s port 0.
    Start { slot: u8, phoneme: u8 },
    /// `Na_SysTrgStart(NA_SE_BEBE)`.
    Bebe,
    /// Na_MelodyVoice uses the current speaker's disc voice_type.
    Melody,
}

/// A voice slot's `_08`, `_0C`, `_14`, `_18` (`Sou_VoiceMake`: volume
/// `_08 * _0C`, pitch `_14 * _18`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoiceSlot {
    pub v08: f32,
    pub v0c: f32,
    pub v14: f32,
    pub v18: f32,
}

impl VoiceSlot {
    pub fn volume(&self) -> f32 {
        self.v08 * self.v0c
    }
    pub fn pitch(&self) -> f32 {
        self.v14 * self.v18
    }
}

impl Default for VoiceSlot {
    fn default() -> Self {
        VoiceSlot {
            v08: 1.0,
            v0c: 1.0,
            v14: 1.0,
            v18: 1.0,
        }
    }
}

/// The voice sequence for a spec (`Na_SpecChange`): 243, 244 or 245, and
/// `sou_now_voice_seq` 1-3.
pub fn voice_sequence(spec: u8) -> Option<(u8, u8)> {
    match spec {
        2 | 5 | 7 | 9 => Some((243, 1)),
        3 => Some((244, 2)),
        4 | 6 | 8 => Some((245, 3)),
        _ => None,
    }
}

/// `mNpc_GetNpcSoundSpec` for a villager, by `looks`.
pub fn villager_spec(looks: u8) -> u8 {
    const SPEC: [u8; 6] = [4, 4, 2, 2, 3, 4];
    SPEC.get(looks as usize).copied().unwrap_or(SPEC[2])
}

/// The phoneme of a message character (`mMsg_sound_voice_get`).
pub fn phoneme_of(code: u8) -> u8 {
    VOICE_ARRAY[code as usize]
}

/// `Na_VoiceSe`'s state (the `sou_*` globals it keeps), Animalese mode.
#[derive(Clone, Debug, Default)]
pub struct Voice {
    pub spec: u8,
    pub voice_seq: u8,
    /// Current Na_VoiceSe voice argument, including the font's melody override.
    pub speaker_id: u16,
    /// `sou_message_status` (`Na_MessageStatus`): the speaker's mood.
    pub message_status: u8,
    pub slots: [VoiceSlot; 2],
    toggle: bool,
    last_num: u8,
    last_num2: u8,
    num_org: u8,
    num2_org: u8,
    num3_org: u8,
    effect_counter: u8,
    effect: u8,
    sad_toggle: bool,
    pending: Option<(u8, u8, u16)>,
    called: bool,
}

impl Voice {
    /// A new message cancels lookahead from the previous page/speaker.
    pub fn begin_message(&mut self) {
        self.pending = None;
        self.called = false;
        self.last_num = 0;
        self.last_num2 = 0;
        self.set_status(0);
    }

    pub fn end_message(&mut self) {
        self.pending = None;
        self.called = false;
    }

    /// Na_MessageStatus resets the wobble counters, even for the same status.
    pub fn set_status(&mut self, status: u8) {
        self.message_status = status;
        self.effect = 0;
        self.effect_counter = 0;
    }

    /// Sou_SpecialRoutine10: wait a game frame after a typed request, then
    /// emit its pending second syllable if another character has not replaced it.
    pub fn game_frame(&mut self, rand: &mut impl FnMut() -> u8) -> Vec<VoiceOut> {
        if self.called {
            self.called = false;
            Vec::new()
        } else if let Some((a, scale, speaker_id)) = self.pending.take() {
            self.speaker_id = speaker_id;
            self.voice_inner(a, 0, 0, scale, true, rand)
        } else {
            Vec::new()
        }
    }

    /// `Na_SpecChange`: a new speaker's voice. Returns the sequence to
    /// start on the voice group when it changes.
    pub fn spec_change(&mut self, spec: u8) -> Option<u8> {
        if spec == self.spec {
            return None;
        }
        self.spec = spec;
        let (seq, n) = voice_sequence(spec)?;
        self.voice_seq = n;
        Some(seq)
    }

    /// `Na_VoiceSe(a, b, c, voice, scale, 0)` for Animalese: `a` is the
    /// typed character's phoneme, `b` and `c` the next two; `scale` is the
    /// text size (32 normal). `rand` is `Nap_GetRandom`.
    pub fn voice_se(
        &mut self,
        a: u8,
        b: u8,
        c: u8,
        scale: u8,
        rand: &mut impl FnMut() -> u8,
    ) -> Vec<VoiceOut> {
        self.voice_inner(a, b, c, scale, false, rand)
    }

    fn voice_inner(
        &mut self,
        a: u8,
        b: u8,
        c: u8,
        scale: u8,
        deferred: bool,
        rand: &mut impl FnMut() -> u8,
    ) -> Vec<VoiceOut> {
        self.called = true;
        let mut out = Vec::new();
        let (mut a, mut b) = (a, b);
        let mut i = 0u8;
        if deferred {
            if a == 0xFF {
                return out;
            }
            a = if a == 0x61
                && self.num3_org > 0x77
                && boin_shiin_check(self.last_num2) == 0
                && boin_shiin_check(self.num_org) == 1
            {
                0
            } else {
                tanboin_henkan(a, self.num3_org)
            };
        } else {
            self.num_org = a;
            self.num2_org = b;
            self.num3_org = c;
            if a == 0x85 && b == 0xFF && c == 0xFF {
                return out;
            }
            let mut r28 = 0;
            if a == 0x61
                && b > 0x77
                && boin_shiin_check(self.last_num) == 0
                && boin_shiin_check(self.last_num2) == 1
            {
                a = b;
                r28 = 4;
            }
            if b == 0x61
                && c > 0x77
                && boin_shiin_check(self.last_num2) == 0
                && boin_shiin_check(a) == 1
            {
                b = 0;
                r28 = 4;
            }
            if r28 != 4 {
                a = connect_check(a, b, c);
                r28 = if a == self.num_org { 1 } else { 2 };
                if b == 0xFF || (0x80..=0x86).contains(&b) {
                    r28 = 3;
                    if self.last_num2 == 0xFF || (0x80..=0x86).contains(&self.last_num2) {
                        r28 = 1;
                    }
                }
            }
            match r28 {
                1 | 4 => {
                    a = if boin_shiin_check(a) == 0 && boin_shiin_check(b) == 1 && c == 0x61 {
                        chouboin_henkan(a, b)
                    } else {
                        tanboin_henkan(a, b)
                    };
                    self.pending = (b != 0).then_some((b, scale, self.speaker_id));
                }
                3 if a <= 0x77 => {
                    let x = connect_check(self.last_num2, self.num_org, self.num2_org);
                    if (0x53..=0x76).contains(&x) {
                        a = tanboin_henkan(x, self.num_org);
                    } else {
                        a = x;
                    }
                }
                _ => {}
            }
        }
        if (deferred || self.num_org == self.last_num) && self.num_org == self.last_num2 {
            i = 1;
        }
        self.last_num = self.num_org;
        self.last_num2 = self.num2_org;
        if a == 0x80 {
            out.push(VoiceOut::Melody);
            return out;
        }
        let r29 = a;
        let slot = if self.toggle { 1 } else { 0 };
        self.toggle = !self.toggle;
        let (v08, v14) = match self.spec {
            2..=4 => (Some(0.65), 1.0),
            5 | 6 => (Some(0.9), 1.0),
            7 => (Some(0.9), 1.3),
            8 => (Some(0.9), 0.75),
            9 => (Some(0.7), 0.65),
            _ => (None, 1.0),
        };
        if let Some(v) = v08 {
            for s in self.slots.iter_mut() {
                s.v08 = v;
                s.v14 = v14;
            }
        }
        if matches!(self.spec, 5 | 6) {
            if matches!(a, 0x81 | 0x84 | 0x85 | 0x86) {
                out.push(VoiceOut::Bebe);
                a = 0;
            }
        } else {
            match a {
                0x86 => {
                    out.push(VoiceOut::Bebe);
                    a = 0;
                }
                0x81 | 0x84 | 0x85 => a = 0,
                0x82 => {
                    a = 0;
                    let f = match self.voice_seq {
                        1 => 1.24,
                        2 => 1.3,
                        3 => 1.13,
                        _ => 0.0,
                    };
                    self.slots[0].v14 *= f;
                    self.slots[1].v14 = self.slots[0].v14;
                }
                0x83 => {
                    a = 0;
                    self.slots[0].v08 *= 1.15;
                    self.slots[1].v08 = self.slots[0].v08;
                }
                _ => {}
            }
        }
        if matches!(self.spec, 5 | 6) || r29 > 0x77 {
            i = 0;
        }
        if i == 1 {
            a = rand() >> 2;
        }
        if r29 <= 0x77 && self.spec != 5 && self.spec != 6 {
            self.mood(rand);
        }
        if r29 != 0x85 && r29 != 0x81 && r29 != 0x84 {
            // The retail code overwrites the scaled value with a constant.
            if scale > 0x20 {
                self.slots[0].v08 = 4.0;
                self.slots[1].v08 = 4.0;
            } else if scale < 0x20 {
                self.slots[0].v08 = 0.4;
                self.slots[1].v08 = 0.4;
            }
            if a != 0 && a <= 0x77 {
                out.push(VoiceOut::Start { slot, phoneme: a });
            }
        }
        out
    }

    /// The mood wobble on volume (`_0C`) and pitch (`_18`).
    fn mood(&mut self, rand: &mut impl FnMut() -> u8) {
        let set = |slots: &mut [VoiceSlot; 2], p: f32, v: f32| {
            for s in slots.iter_mut() {
                s.v18 = p;
                s.v0c = v;
            }
        };
        match self.message_status {
            1 | 3 => {
                if self.effect_counter == 0 {
                    self.effect_counter = (rand() >> 6) + 0xA;
                } else {
                    self.effect_counter -= 1;
                }
                let (p, v) = match (self.message_status, self.effect_counter) {
                    (1, 0..=2) | (1, 9 | 10) => (1.25, 1.25),
                    (1, 3 | 4) => (1.2, 1.2),
                    (1, _) => (0.9, 1.05),
                    (_, 0 | 1) => (1.05, 1.05),
                    (_, 2) => (0.9, 1.15),
                    _ => (1.15, 1.05),
                };
                set(&mut self.slots, p, v);
            }
            2 | 4 | 5 => {
                if self.effect_counter == 0 {
                    self.effect = (rand() >> 6) + 0xE;
                    self.effect_counter = 1;
                } else if self.effect_counter == self.effect {
                    self.effect_counter = 0;
                    self.sad_toggle = !self.sad_toggle;
                } else {
                    self.effect_counter += 1;
                }
                let base = if self.sad_toggle { 1.1 } else { 1.0 };
                let n = self.effect_counter as f32;
                set(&mut self.slots, base - n * 0.022, base - n * 0.025);
            }
            _ => set(&mut self.slots, 1.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_hello_starts_voices_on_alternating_slots() {
        let mut v = Voice::default();
        assert_eq!(v.spec_change(villager_spec(2)), Some(243));
        let codes: Vec<u8> = "hello".bytes().map(phoneme_of).collect();
        let mut starts = Vec::new();
        let mut r = || 0x40u8;
        for k in 0..codes.len() {
            let b = codes.get(k + 1).copied().unwrap_or(0xFF);
            let c = codes.get(k + 2).copied().unwrap_or(0xFF);
            for o in v.voice_se(codes[k], b, c, 32, &mut r) {
                if let VoiceOut::Start { slot, phoneme } = o {
                    starts.push((slot, phoneme));
                }
            }
        }
        assert!(starts.len() >= 2, "{starts:?}");
        assert!(starts.iter().all(|&(_, p)| p != 0 && p <= 0x77));
        assert!(
            starts.windows(2).any(|w| w[0].0 != w[1].0),
            "slots alternate"
        );
        assert!((v.slots[0].volume() - 0.65).abs() < 1e-6);
    }
    #[test]
    fn deferred_syllable_keeps_its_speaker_and_cancels_on_close() {
        let mut voice = Voice::default();
        voice.spec_change(2);
        voice.speaker_id = 17;
        voice.voice_se(0x5E, 0x5D, 0xFF, 32, &mut || 0x40);
        voice.speaker_id = 99;
        assert!(voice.game_frame(&mut || 0x40).is_empty());
        assert!(voice
            .game_frame(&mut || 0x40)
            .iter()
            .any(|o| matches!(o, VoiceOut::Start { .. })));
        assert_eq!(voice.speaker_id, 17);
        voice.voice_se(0x5E, 0x5D, 0xFF, 32, &mut || 0x40);
        voice.end_message();
        assert!(voice.game_frame(&mut || 0x40).is_empty());
        assert!(voice.game_frame(&mut || 0x40).is_empty());
    }

    #[test]
    fn punctuation_and_mood_update_live_slot_parameters() {
        let mut voice = Voice::default();
        voice.spec_change(2);
        let out = voice.voice_se(0x82, 0xFF, 0xFF, 32, &mut || 0x40);
        assert!(out.is_empty());
        assert!((voice.slots[0].pitch() - 1.24).abs() < 1.0e-6);
        voice.set_status(1);
        voice.voice_se(0x5E, 0x5D, 0xFF, 32, &mut || 0x40);
        assert!(voice.slots[0].volume() > 0.65);
        voice.begin_message();
        assert_eq!(voice.message_status, 0);
        assert!(voice.game_frame(&mut || 0x40).is_empty());
    }
}
