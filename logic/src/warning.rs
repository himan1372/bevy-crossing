//! The retail warning panel (`m_warning_ovl.c`): its wording and layout per
//! warning, and the grow/shrink of the panel. Pure; the frame model and the
//! font come from the disc.

/// One line: the offset from the previous line (`pos_x` from the panel's left
/// edge, `pos_y` below the previous line) and its text.
pub struct Line {
    pub x: f32,
    pub y: f32,
    pub text: &'static str,
}

/// A warning's lines and the panel's stretch (`wr_win_data`).
pub struct Warning {
    pub lines: &'static [Line],
    pub scale_x: f32,
    pub scale_y: f32,
}

/// `mWR_WARNING_HUKUBUKURO_OPEN`: a bag needs three empty slots.
pub const HUKUBUKURO_OPEN: Warning = Warning {
    lines: &[
        Line {
            x: 48.0,
            y: 37.0,
            text: "You can't open a bag",
        },
        Line {
            x: 50.0,
            y: 24.0,
            text: "unless you have room",
        },
        Line {
            x: 52.0,
            y: 24.0,
            text: "for three more items!",
        },
    ],
    scale_x: 1.0,
    scale_y: 1.15,
};

impl Warning {
    /// Where line `index` starts on the 320 by 240 screen at full size
    /// (`mWR_set_strings` with `scale` 1): left edge and top.
    pub fn line_origin(&self, index: usize) -> (f32, f32) {
        let down: f32 = self.lines[..=index].iter().map(|line| line.y).sum();
        (
            160.0 - self.scale_x * 120.0 + self.lines[index].x,
            120.0 - self.scale_y * 60.0 + down,
        )
    }
}

/// Where the panel is in its life (`mWR_STATE_IN` / `_OUT` and the wait).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Growing,
    Waiting,
    Shrinking,
    Gone,
}

/// `mWR_move_Move` / `mWR_move_Play`: 0.2 of the size per 60 Hz frame, then
/// A, B or Start closes it.
#[derive(Clone, Copy, Debug)]
pub struct Panel {
    pub scale: f32,
    pub phase: Phase,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            scale: 0.0,
            phase: Phase::Growing,
        }
    }

    /// One 60 Hz frame of growing or shrinking.
    pub fn frame(&mut self) {
        match self.phase {
            Phase::Growing => {
                self.scale += 0.2;
                if self.scale > 1.0 {
                    self.scale = 1.0;
                    self.phase = Phase::Waiting;
                }
            }
            Phase::Shrinking => {
                self.scale -= 0.2;
                if self.scale < 0.0 {
                    self.scale = 0.0;
                    self.phase = Phase::Gone;
                }
            }
            Phase::Waiting | Phase::Gone => {}
        }
    }

    /// A, B or Start while it waits starts the shrink; true when it did.
    pub fn dismiss(&mut self) -> bool {
        let waiting = self.phase == Phase::Waiting;
        if waiting {
            self.phase = Phase::Shrinking;
        }
        waiting
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}
