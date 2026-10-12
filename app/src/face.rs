//! Eye and mouth texture animation: blinking, and villagers' mouths moving
//! while they talk.
//!
//! The face parts of a character model sample their texture from segment
//! 8 (eyes) or 9 (mouth); the game picks the frame each draw. Here every
//! frame is decoded once and the part's material texture is swapped.
//!
//! - Player (`Player_actor_set_eye_pattern_normal`): a 16-step countdown
//!   reads `0,0,0,0,1,1,2,2,2,2,2,2,1,1,0,0` (open, half, closed), with
//!   60-180 frames open between blinks and 0-2 extra blinks in a row.
//! - Villagers (`aNPC_set_tex_anime` / `aNPC_tex_anm_ctrl_rnd_seq`): a
//!   short sequence of (pattern, ticks) pairs, ticking 0.5 per frame;
//!   blinks repeat 1-3 times, then the eyes stay open 32-48 ticks. While
//!   talking the mouth runs one of three open/close sequences.

use bevy::prelude::*;

use crate::dialogue::DialogueState;
use crate::n64_material::N64Material;

/// Which texture a face part shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceSlot {
    Eye,
    Mouth,
}

/// A face part's material and its decoded frames (by pattern index).
pub struct FacePart {
    pub slot: FaceSlot,
    pub material: Handle<N64Material>,
    pub frames: Vec<Option<Handle<Image>>>,
}

/// Who the face belongs to: the blink rules differ.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceKind {
    Player,
    Villager,
}

/// A character's animated face.
#[derive(Component)]
pub struct Face {
    pub kind: FaceKind,
    pub parts: Vec<FacePart>,
    /// The character entity (talking is decided by who the speaker is).
    pub owner: Entity,
    eye: Sequence,
    mouth: Sequence,
    /// Player blink: countdown timer and blinks left in this run.
    player_timer: f32,
    player_blinks: u32,
    shown: (u8, u8),
    rng: u32,
}

impl Face {
    pub fn new(kind: FaceKind, parts: Vec<FacePart>, owner: Entity) -> Self {
        Self {
            kind,
            parts,
            owner,
            eye: Sequence::default(),
            mouth: Sequence::default(),
            player_timer: 0.0,
            player_blinks: 0,
            shown: (u8::MAX, u8::MAX),
            // Any non-zero seed; differ per character so blinks don't sync.
            rng: owner.index_u32().wrapping_mul(0x9E37_79B9) | 1,
        }
    }

    /// `fqrand()`: uniform in [0, 1).
    fn rand(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng >> 8) as f32 / (1u32 << 24) as f32
    }
}

/// `eye_normal_blink`: (pattern, ticks); played as entry 0 (the wait),
/// then entries 2, 1, 0 (`aNPC_get_seq_cnt` = 3 counts down).
const EYE_BLINK: &[(u8, u8)] = &[(0, 1), (1, 1), (2, 3), (1, 1)];
const EYE_SEQ_COUNT: u8 = 3;
/// `mouth_normal_move_type{A,B,C}` with their sequence counts.
const MOUTH_MOVES: [(&[(u8, u8)], u8); 3] = [
    (&[(0, 1), (1, 1), (2, 1), (1, 1), (0, 1), (0, 0)], 4),
    (&[(0, 1), (1, 1), (0, 1), (0, 0)], 2),
    (
        &[
            (0, 1),
            (1, 1),
            (2, 1),
            (1, 1),
            (2, 1),
            (1, 1),
            (0, 1),
            (0, 0),
        ],
        6,
    ),
];
/// Player blink table (`pattern_table` in `Player_actor_set_eye_pattern_normal`).
const PLAYER_BLINK: [u8; 16] = [0, 0, 0, 0, 1, 1, 2, 2, 2, 2, 2, 2, 1, 1, 0, 0];

/// A villager texture sequence (`aNPC_tex_anim_c`).
#[derive(Default)]
struct Sequence {
    table: &'static [(u8, u8)],
    count: u8,
    counter: u8,
    pattern: u8,
    ticks: f32,
    loops: u8,
    running: bool,
}

impl Sequence {
    /// `aNPC_set_tex_anime`: start at entry 0, waiting `wait` ticks (or
    /// the entry's own), for `loops` runs.
    fn start(&mut self, table: &'static [(u8, u8)], count: u8, wait: Option<f32>, loops: u8) {
        self.table = table;
        self.count = count;
        self.counter = count;
        self.pattern = table[0].0;
        self.ticks = wait.unwrap_or(table[0].1 as f32);
        self.loops = loops;
        self.running = true;
    }

    /// `aNPC_tex_anm_ctrl_rnd_seq` step; returns true when a run ended
    /// (the caller restarts it).
    fn step(&mut self, frames: f32) -> bool {
        if self.ticks > 0.0 {
            self.ticks -= 0.5 * frames;
            return false;
        }
        if self.counter == 0 {
            return true;
        }
        self.counter -= 1;
        let (pattern, ticks) = self.table[self.counter as usize];
        self.pattern = pattern;
        self.ticks = ticks as f32;
        false
    }
}

fn step_villager_eyes(face: &mut Face, frames: f32) {
    if !face.eye.running {
        let loops = 1 + (face.rand() * 3.0) as u8;
        face.eye.start(EYE_BLINK, EYE_SEQ_COUNT, None, loops);
        return;
    }
    if face.eye.step(frames) {
        let loops = face.eye.loops.saturating_sub(1);
        if loops == 0 {
            // Done blinking: eyes open for 32-48 ticks.
            let wait = 32.0 + (face.rand() * 16.0).floor();
            let loops = 1 + (face.rand() * 3.0) as u8;
            face.eye.start(EYE_BLINK, EYE_SEQ_COUNT, Some(wait), loops);
        } else {
            face.eye.start(EYE_BLINK, EYE_SEQ_COUNT, None, loops);
        }
    }
}

fn step_mouth(face: &mut Face, talking: bool, frames: f32) {
    if !talking {
        face.mouth.running = false;
        face.mouth.pattern = 0;
        return;
    }
    let pick = |face: &mut Face| MOUTH_MOVES[((face.rand() * 3.0) as usize).min(2)];
    if !face.mouth.running {
        let (table, count) = pick(face);
        face.mouth.start(table, count, Some(0.0), 1);
        return;
    }
    if face.mouth.step(frames) {
        // `aNPC_tex_anm_ctrl_rnd_seq_mouth`: next sequence straight away.
        let (table, count) = pick(face);
        let loops = 1 + (face.rand() * 3.0) as u8;
        face.mouth.start(table, count, Some(0.0), loops);
    }
}

fn step_player_eyes(face: &mut Face, frames: f32) -> u8 {
    if face.player_timer > 0.0 {
        face.player_timer = (face.player_timer - frames).max(0.0);
    }
    if face.player_timer <= 0.0 {
        if face.player_blinks == 0 {
            face.player_blinks = (face.rand() * 3.0) as u32;
            face.player_timer = 60.0 + (face.rand() * 120.0).floor();
        } else {
            face.player_timer = 16.0;
            face.player_blinks -= 1;
        }
    }
    let step = face.player_timer as usize;
    PLAYER_BLINK[if step >= 16 { 0 } else { step }]
}

/// Steps every face and swaps textures when a pattern changes.
fn animate_faces(
    time: Res<Time>,
    dialogue: Res<DialogueState>,
    mut faces: Query<&mut Face>,
    mut materials: ResMut<Assets<N64Material>>,
) {
    let frames = time.delta_secs() * 60.0;
    for mut face in &mut faces {
        let eye = match face.kind {
            FaceKind::Player => step_player_eyes(&mut face, frames),
            FaceKind::Villager => {
                step_villager_eyes(&mut face, frames);
                face.eye.pattern
            }
        };
        let talking = dialogue.active && dialogue.speaker == Some(face.owner);
        step_mouth(&mut face, talking, frames);
        let shown = (eye, face.mouth.pattern);
        if shown == face.shown {
            continue;
        }
        face.shown = shown;
        for part in &face.parts {
            let pattern = match part.slot {
                FaceSlot::Eye => shown.0,
                FaceSlot::Mouth => shown.1,
            } as usize;
            let Some(Some(frame)) = part.frames.get(pattern).or(part.frames.first()) else {
                continue;
            };
            if let Some(mut m) = materials.get_mut(&part.material) {
                m.texture = Some(frame.clone());
            }
        }
    }
}

pub struct FacePlugin;

impl Plugin for FacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_faces);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn villager_blink_plays_closed_half_open() {
        let mut s = Sequence::default();
        s.start(EYE_BLINK, EYE_SEQ_COUNT, Some(0.0), 1);
        let mut seen = vec![s.pattern];
        for _ in 0..40 {
            if s.step(1.0) {
                break;
            }
            if seen.last() != Some(&s.pattern) {
                seen.push(s.pattern);
            }
        }
        assert_eq!(seen, vec![0, 2, 1, 0]);
    }

    #[test]
    fn player_blink_table_closes_mid_way() {
        assert_eq!(PLAYER_BLINK[0], 0);
        assert_eq!(PLAYER_BLINK[8], 2);
    }
}
