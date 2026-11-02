//! `m_editEndChk_ovl.c` (`mEE_move_Play`): the "Is this OK?" window shown after an edit ends.
//! The question slides in from the bottom and blinks; the first accept expands the answer window,
//! the cursor then moves on up/down and a second accept picks. Cancel picks Rewrite at once.

use crate::menu_slide::Slide;

/// `mEE_str_data0..2`.
pub const ANSWERS: [&str; 3] = ["Yes", "Rewrite", "Throw it out"];

/// `menu_info->data1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Yes,
    Rewrite,
    ThrowOut,
}

/// What the player pressed this tick (`BUTTON_A | START`, `BUTTON_B`, `CUP`, `CDOWN`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Press {
    None,
    Accept,
    Cancel,
    Up,
    Down,
}

/// The sound the window asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    /// `NA_SE_CURSOL`.
    Cursor,
    /// `NA_SE_MENU_EXIT`.
    Exit,
    /// `sAdo_SysTrgStart(0x1003)`.
    Refuse,
}

#[derive(Clone, Debug)]
pub struct EditEndCheck {
    /// Slides in from the bottom of the screen and back out once an answer is picked.
    pub slide: Slide,
    /// `moving_in`: the answer window is shown.
    pub moving_in: bool,
    /// The answer window's size, 0 to 1.
    pub scale: f32,
    pub selected: usize,
    alpha_step: u32,
    alpha_accum: f32,
    answer: Option<Answer>,
}

impl Default for EditEndCheck {
    fn default() -> Self {
        Self::new()
    }
}

/// One tick's outcome.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Update {
    pub sound: Option<Sound>,
    /// Set once the chosen answer has finished sliding out.
    pub done: Option<Answer>,
}

impl EditEndCheck {
    pub fn new() -> Self {
        Self {
            slide: Slide::new(),
            moving_in: false,
            scale: 0.0,
            selected: 0,
            alpha_step: 0,
            alpha_accum: 0.0,
            answer: None,
        }
    }

    pub fn leaving(&self) -> bool {
        self.answer.is_some()
    }

    /// The window takes input once it has come to rest.
    pub fn playing(&self) -> bool {
        self.answer.is_none() && self.slide.settled()
    }

    /// `question_alpha`: the blue question ink fades up and down over 30 ticks; it stays clear
    /// while the answers are open.
    pub fn question_alpha(&self) -> u8 {
        if self.moving_in {
            return 0;
        }
        let step = self.alpha_step;
        let value = if step < 15 { step } else { 30 - step };
        (value * 255 / 15) as u8
    }

    /// 60 Hz ticks (`dt_num_60fps_frames`) with this frame's press.
    pub fn update(&mut self, ticks: f32, press: Press) -> Update {
        let mut update = Update::default();
        if let Some(answer) = self.answer {
            if self.slide.advance(ticks) {
                update.done = Some(answer);
            }
            return update;
        }
        if !self.slide.settled() {
            self.slide.advance(ticks);
            return update;
        }
        match press {
            Press::Cancel => {
                update.sound = Some(Sound::Refuse);
                self.leave(Answer::Rewrite);
            }
            Press::Accept => {
                if self.moving_in {
                    let answer = match self.selected {
                        0 => Answer::Yes,
                        1 => Answer::Rewrite,
                        _ => Answer::ThrowOut,
                    };
                    update.sound = Some(if answer == Answer::ThrowOut {
                        Sound::Refuse
                    } else {
                        Sound::Exit
                    });
                    self.leave(answer);
                } else {
                    self.moving_in = true;
                    self.scale = 0.0;
                    update.sound = Some(Sound::Exit);
                }
            }
            Press::Up | Press::Down | Press::None if self.moving_in => {
                if self.scale < 1.0 {
                    self.scale = (self.scale + 0.2 * ticks).min(1.0);
                } else if press == Press::Up && self.selected > 0 {
                    self.selected -= 1;
                    update.sound = Some(Sound::Cursor);
                } else if press == Press::Down && self.selected + 1 < ANSWERS.len() {
                    self.selected += 1;
                    update.sound = Some(Sound::Cursor);
                }
                self.alpha_step = 0;
                self.alpha_accum = 0.0;
            }
            _ => {
                self.alpha_accum += ticks;
                while self.alpha_accum >= 1.0 {
                    self.alpha_step = (self.alpha_step + 1) % 30;
                    self.alpha_accum -= 1.0;
                }
            }
        }
        update
    }

    fn leave(&mut self, answer: Answer) {
        self.answer = Some(answer);
        self.slide.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settle(check: &mut EditEndCheck) {
        for _ in 0..200 {
            check.update(1.0, Press::None);
        }
        assert!(check.playing());
    }

    fn finish(check: &mut EditEndCheck) -> Answer {
        for _ in 0..400 {
            if let Some(answer) = check.update(1.0, Press::None).done {
                return answer;
            }
        }
        panic!("the window never left");
    }

    #[test]
    fn the_second_accept_picks_the_cursor_answer() {
        let mut check = EditEndCheck::new();
        // Input before the window rests is ignored.
        assert_eq!(check.update(1.0, Press::Accept), Update::default());
        settle(&mut check);
        assert_eq!(check.update(1.0, Press::Accept).sound, Some(Sound::Exit));
        assert!(check.moving_in && !check.leaving());
        for _ in 0..6 {
            check.update(1.0, Press::None);
        }
        assert_eq!(check.scale, 1.0);
        assert_eq!(check.update(1.0, Press::Down).sound, Some(Sound::Cursor));
        assert_eq!(check.update(1.0, Press::Down).sound, Some(Sound::Cursor));
        assert_eq!(check.update(1.0, Press::Down).sound, None);
        assert_eq!(check.update(1.0, Press::Accept).sound, Some(Sound::Refuse));
        assert_eq!(finish(&mut check), Answer::ThrowOut);
    }

    #[test]
    fn cancel_rewrites_at_once_and_the_question_blinks() {
        let mut check = EditEndCheck::new();
        settle(&mut check);
        let alphas: Vec<u8> = (0..30)
            .map(|_| {
                check.update(1.0, Press::None);
                check.question_alpha()
            })
            .collect();
        assert_eq!(alphas.iter().copied().max(), Some(255));
        assert_eq!(alphas.iter().copied().min(), Some(0));
        assert_eq!(check.update(1.0, Press::Cancel).sound, Some(Sound::Refuse));
        assert_eq!(finish(&mut check), Answer::Rewrite);
    }
}
