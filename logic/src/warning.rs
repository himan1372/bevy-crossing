//! The retail warning panel (`m_warning_ovl.c`): its wording and layout per
//! warning, and the grow/shrink of the panel. Pure; the frame model and the
//! font come from the disc. The tables are generated from the C (`wr_win_data`).

/// One line: the offset from the previous line (`pos_x` from the panel's left
/// edge, `pos_y` below the previous line) and its text.
pub struct Line {
    pub x: f32,
    pub y: f32,
    pub text: &'static str,
}

/// A warning's lines and the panel's stretch (`wr_win_data`). The mailbox warning
/// is the one with an answer: its lines 3 and 4 are Yes and No.
pub struct Warning {
    pub lines: &'static [Line],
    pub scale_x: f32,
    pub scale_y: f32,
}

/// `mWR_WARNING_MAILBOX`: no room for more letters (Yes or No).
pub const MAILBOX: Warning = Warning {
    lines: &[
        Line {
            x: 70.0,
            y: 28.0,
            text: "You can't hold",
        },
        Line {
            x: 40.0,
            y: 24.0,
            text: "any more letters! Want to",
        },
        Line {
            x: 36.0,
            y: 24.0,
            text: "throw away some old ones?",
        },
        Line {
            x: 94.0,
            y: 24.0,
            text: "Yes",
        },
        Line {
            x: 140.0,
            y: 0.0,
            text: "No",
        },
    ],
    scale_x: 1.00,
    scale_y: 1.20,
};

/// `mWR_WARNING_PR_LEAVE`.
pub const PR_LEAVE: Warning = Warning {
    lines: &[
        Line {
            x: 44.0,
            y: 40.0,
            text: "You can't pull out a present",
        },
        Line {
            x: 40.0,
            y: 24.0,
            text: "when your item screen is full!",
        },
    ],
    scale_x: 1.15,
    scale_y: 1.00,
};

/// `mWR_WARNING_MONEY`.
pub const MONEY: Warning = Warning {
    lines: &[
        Line {
            x: 66.0,
            y: 40.0,
            text: "You can't carry",
        },
        Line {
            x: 44.0,
            y: 24.0,
            text: "more than 99,999 Bells!",
        },
    ],
    scale_x: 1.00,
    scale_y: 1.00,
};

/// `mWR_WARNING_PR_FOOD`.
pub const PR_FOOD: Warning = Warning {
    lines: &[
        Line {
            x: 70.0,
            y: 40.0,
            text: "You can't mail",
        },
        Line {
            x: 42.0,
            y: 24.0,
            text: "turnips or living things!",
        },
    ],
    scale_x: 1.00,
    scale_y: 1.00,
};

/// `mWR_WARNING_PR_QUEST`.
pub const PR_QUEST: Warning = Warning {
    lines: &[
        Line {
            x: 74.0,
            y: 40.0,
            text: "You can't mail",
        },
        Line {
            x: 38.0,
            y: 24.0,
            text: "someone else's belongings!",
        },
    ],
    scale_x: 1.00,
    scale_y: 1.00,
};

/// `mWR_WARNING_PUT_MAX_FURNITURE`.
pub const PUT_MAX_FURNITURE: Warning = Warning {
    lines: &[
        Line {
            x: 68.0,
            y: 40.0,
            text: "You can't put",
        },
        Line {
            x: 50.0,
            y: 24.0,
            text: "any more items out!",
        },
    ],
    scale_x: 0.95,
    scale_y: 1.00,
};

/// `mWR_WARNING_PUT_FURNITURE`.
pub const PUT_FURNITURE: Warning = Warning {
    lines: &[
        Line {
            x: 68.0,
            y: 40.0,
            text: "You can't put",
        },
        Line {
            x: 46.0,
            y: 24.0,
            text: "any more items there!",
        },
    ],
    scale_x: 0.95,
    scale_y: 1.00,
};

/// `mWR_WARNING_PUT_ITEM`.
pub const PUT_ITEM: Warning = Warning {
    lines: &[
        Line {
            x: 72.0,
            y: 40.0,
            text: "You can't drop",
        },
        Line {
            x: 54.0,
            y: 24.0,
            text: "any more items here!",
        },
    ],
    scale_x: 1.00,
    scale_y: 1.00,
};

/// `mWR_WARNING_PUT_PLANT`.
pub const PUT_PLANT: Warning = Warning {
    lines: &[Line {
        x: 39.0,
        y: 46.0,
        text: "You can't plant anything here!",
    }],
    scale_x: 1.15,
    scale_y: 0.90,
};

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
    scale_x: 1.00,
    scale_y: 1.15,
};

/// `mWR_WARNING_PRESENT_MAIL`.
pub const PRESENT_MAIL: Warning = Warning {
    lines: &[
        Line {
            x: 74.0,
            y: 40.0,
            text: "You can't mail",
        },
        Line {
            x: 62.0,
            y: 24.0,
            text: "a wrapped present!",
        },
    ],
    scale_x: 1.00,
    scale_y: 1.00,
};

/// `mWR_WARNING_WRITE`.
pub const WRITE: Warning = Warning {
    lines: &[
        Line {
            x: 85.0,
            y: 40.0,
            text: "You can't carry",
        },
        Line {
            x: 82.0,
            y: 24.0,
            text: "any more letters,",
        },
        Line {
            x: 45.0,
            y: 24.0,
            text: "so you can't write a new one!",
        },
    ],
    scale_x: 1.15,
    scale_y: 1.20,
};

/// `mWR_WARNING_MUSIC`.
pub const MUSIC: Warning = Warning {
    lines: &[
        Line {
            x: 47.0,
            y: 40.0,
            text: "You can't pull anything out",
        },
        Line {
            x: 41.0,
            y: 24.0,
            text: "when your item screen is full!",
        },
    ],
    scale_x: 1.15,
    scale_y: 1.00,
};

/// `mWR_WARNING_MUSIC2`.
pub const MUSIC2: Warning = Warning {
    lines: &[Line {
        x: 38.0,
        y: 46.0,
        text: "There's already music in there!",
    }],
    scale_x: 1.15,
    scale_y: 0.90,
};

/// `mWR_WARNING_ORIGINAL`.
pub const ORIGINAL: Warning = Warning {
    lines: &[
        Line {
            x: 54.0,
            y: 40.0,
            text: "Your item screen is full,",
        },
        Line {
            x: 48.0,
            y: 24.0,
            text: "so you can't use a pattern!",
        },
    ],
    scale_x: 1.13,
    scale_y: 1.00,
};

/// `mWR_WARNING_PUT_SIGN`.
pub const PUT_SIGN: Warning = Warning {
    lines: &[Line {
        x: 40.0,
        y: 46.0,
        text: "You can't put a sign up here!",
    }],
    scale_x: 1.13,
    scale_y: 0.90,
};

/// `mWR_WARNING_PUT_SIGN_OTHER`.
pub const PUT_SIGN_OTHER: Warning = Warning {
    lines: &[
        Line {
            x: 44.0,
            y: 40.0,
            text: "You can put signs up only",
        },
        Line {
            x: 64.0,
            y: 24.0,
            text: "in your own village!",
        },
    ],
    scale_x: 1.10,
    scale_y: 1.00,
};

/// `mWR_WARNING_PUT_SIGN_ISLAND`.
pub const PUT_SIGN_ISLAND: Warning = Warning {
    lines: &[Line {
        x: 31.0,
        y: 46.0,
        text: "You can't put signs on the island!",
    }],
    scale_x: 1.15,
    scale_y: 0.90,
};

/// `mWR_WARNING_PR_CARD`.
pub const PR_CARD: Warning = Warning {
    lines: &[
        Line {
            x: 60.0,
            y: 40.0,
            text: "You can't mail",
        },
        Line {
            x: 54.0,
            y: 24.0,
            text: "an exercise card!",
        },
    ],
    scale_x: 0.90,
    scale_y: 1.00,
};

/// `mWR_WARNING_LOCK_DIARY`.
pub const LOCK_DIARY: Warning = Warning {
    lines: &[Line {
        x: 38.0,
        y: 46.0,
        text: "This diary is locked!",
    }],
    scale_x: 0.90,
    scale_y: 0.90,
};

/// `mWR_WARNING_PUT_FAMI`.
pub const PUT_FAMI: Warning = Warning {
    lines: &[
        Line {
            x: 72.0,
            y: 40.0,
            text: "You can't leave",
        },
        Line {
            x: 44.0,
            y: 24.0,
            text: "this NES on the island!",
        },
    ],
    scale_x: 1.00,
    scale_y: 1.00,
};

/// `mWR_WARNING_WORD_OVER`.
pub const WORD_OVER: Warning = Warning {
    lines: &[
        Line {
            x: 54.0,
            y: 40.0,
            text: "You don't have room",
        },
        Line {
            x: 63.0,
            y: 24.0,
            text: "to write any more.",
        },
    ],
    scale_x: 1.00,
    scale_y: 1.00,
};

/// `mWR_WARNING_PW_CHK`.
pub const PW_CHK: Warning = Warning {
    lines: &[
        Line {
            x: 59.0,
            y: 40.0,
            text: "You entered letters",
        },
        Line {
            x: 45.0,
            y: 24.0,
            text: "that have nothing to do",
        },
        Line {
            x: 63.0,
            y: 24.0,
            text: "with secret codes!",
        },
    ],
    scale_x: 1.00,
    scale_y: 1.20,
};

/// `mWR_WARNING_PR_FORK`.
pub const PR_FORK: Warning = Warning {
    lines: &[
        Line {
            x: 60.0,
            y: 40.0,
            text: "You can't mail",
        },
        Line {
            x: 80.0,
            y: 24.0,
            text: "this item!",
        },
    ],
    scale_x: 0.90,
    scale_y: 1.00,
};

impl Warning {
    /// Whether the panel asks Yes or No (`menu_info->data0 == 0`, the mailbox warning).
    pub fn has_choice(&self) -> bool {
        self.lines.len() > 3
    }

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
