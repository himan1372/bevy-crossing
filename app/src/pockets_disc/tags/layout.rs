//! mTG_set_tag_win_scale_p and the supported after-item/after-select layouts.
//! Coordinates here are the original 320x240, centred, Y-up menu coordinates.
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Layout {
    pub base: Vec2,
    pub fraction: Vec2,
    pub body_scale: Vec2,
    pub body: Vec2,
    pub text: Vec2,
    pub arrow: u8,
    pub special: u8,
    pub frame_scale: f32,
}

impl Layout {
    fn sized(base: Vec2, width: f32, height: f32, item: bool) -> Self {
        let (minimum, corner, text_min, text_step, arrow_min, arrow_step, first, span) = if item {
            (
                Vec2::new(0.2972973, 0.8125),
                Vec2::new(-64.75, 14.0),
                Vec2::new(8.75, -4.375),
                Vec2::new(4.375, -2.625),
                Vec2::splat(-3.5),
                Vec2::new(-7.0, 3.5),
                21.0,
                84.0,
            )
        } else {
            (
                Vec2::new(0.5714286, 0.43243244),
                Vec2::new(-61.25, 64.75),
                Vec2::new(22.75, -14.875),
                Vec2::new(5.25, -15.75),
                Vec2::new(6.125, 15.75),
                Vec2::new(0.875, 26.25),
                31.5,
                42.0,
            )
        };
        let x = (width - first) / span;
        let fraction = Vec2::new(
            x,
            if height == 0.0 {
                x
            } else {
                (height - 28.0) / 42.0
            },
        );
        let body_scale = minimum + fraction * (Vec2::ONE - minimum);
        Self {
            base,
            fraction,
            body_scale,
            text: text_min + text_step * fraction + corner * body_scale,
            body: arrow_min + arrow_step * fraction - corner * body_scale,
            arrow: 0,
            special: 0,
            frame_scale: 0.875,
        }
    }

    pub(super) fn item(at: Vec2, width: f32) -> Self {
        let mut tag = Self::sized(
            Vec2::new(at.x - 160.0, 120.0 - at.y),
            width * 0.875,
            0.0,
            true,
        );
        if tag.base.x + tag.body.x + tag.body_scale.x * 64.75 <= 146.0 {
            tag.arrow = 1;
        } else {
            tag.body.x = -tag.body.x;
            tag.arrow = 2;
        }
        tag
    }

    pub(super) fn special(at: Vec2, width: f32, variant: u8) -> Self {
        let fraction = Vec2::splat((width * 0.75 - 52.5) / 67.5);
        let minimum = Vec2::new(0.5217391, 0.88235295);
        let body_scale = minimum + fraction * (Vec2::ONE - minimum);
        let corner = Vec2::new(-92.0, 34.0);
        let (arrow_min, arrow_step) = match variant {
            2 => (Vec2::new(-13.0, 4.0), Vec2::new(-6.0, 10.0)),
            3 => (Vec2::new(-13.0, 56.0), Vec2::new(-6.0, -2.0)),
            _ => (Vec2::new(-1.0, 21.0), Vec2::new(-11.0, 5.0)),
        };
        let mut tag = Self {
            base: Vec2::new(at.x - 160.0, 120.0 - at.y),
            fraction,
            body_scale,
            body: arrow_min + arrow_step * fraction - corner * body_scale,
            text: Vec2::new(22.0, -12.0) + Vec2::new(10.0, -4.0) * fraction + corner * body_scale,
            arrow: 1,
            special: variant,
            frame_scale: 1.0,
        };
        if variant == 1 {
            tag.body.y += 8.0;
        }
        if tag.base.x + tag.body.x + body_scale.x * 92.0 > 146.0 {
            tag.body.x = -tag.body.x;
            tag.arrow = 2;
        }
        tag
    }

    fn half(self) -> Vec2 {
        self.body_scale * Vec2::new(61.25, 64.75)
    }
    fn right(self) -> f32 {
        self.base.x + self.body.x + self.half().x
    }
    fn foot(self) -> bool {
        self.base.y + self.body.y - self.half().y - 3.0 >= -120.0
    }

    pub(super) fn action(parent: Self, width: f32, rows: usize, pocket_row: Option<usize>) -> Self {
        let mut tag = Self::sized(parent.base, width * 0.875, rows as f32 * 14.0, false);
        let special = parent.special != 0;
        let parent_half = if special {
            Vec2::new(92.0, 34.0)
        } else {
            Vec2::new(64.75, 14.0)
        };
        let shift = if special {
            Vec2::new(52.5, 39.375)
        } else {
            Vec2::new(27.125, 54.25)
        };
        let child_shift = if special {
            Vec2::new(-27.125, -28.0)
        } else {
            Vec2::new(-29.75, -28.875)
        };
        let gap = if special { 4.375 } else { 1.75 };
        let gap_step = if special { 0.0 } else { 1.75 };
        let body_gap = if special { 7.0 } else { 6.125 };
        // mTG_init_tag_data_select_win_after_item, ordinary name-tag branch.
        let half = tag.half();
        tag.base.y += gap + parent.body.y - parent.body_scale.y * parent_half.y
            + parent.fraction.y * gap_step;
        tag.body.y = body_gap + tag.fraction.y * 8.75 - half.y;
        tag.base.x += parent.body.x - parent.body_scale.x * parent_half.x;
        tag.body.x = half.x;
        let down = tag.foot() && !(pocket_row == Some(1) && rows == 2);
        if down {
            tag.base.x += shift.x + parent.fraction.x * shift.y;
            tag.body.x += child_shift.x + tag.fraction.x * child_shift.y;
        } else {
            tag.base.y = parent.base.y + parent.body.y + parent.body_scale.y * parent_half.y
                - gap
                - parent.fraction.y * gap_step;
            tag.body.y = half.y - body_gap - tag.fraction.y * 8.75;
            let top = tag.base.y + tag.body.y + half.y;
            if top > 120.0 {
                tag.base.y -= top - 120.0;
            }
            tag.base.x += shift.x + parent.fraction.x * shift.y;
            tag.body.x += child_shift.x + tag.fraction.x * child_shift.y;
        }
        let right = tag.right();
        if right >= 167.0 || parent.body.x < 0.0 {
            tag.base.x = parent.base.x + parent.body.x + parent.body_scale.x * parent_half.x
                - (shift.x + parent.fraction.x * shift.y);
            tag.body.x = -half.x - (child_shift.x + tag.fraction.x * child_shift.y);
            let left = tag.base.x + tag.body.x - half.x + 134.0;
            if left < 0.0 {
                tag.body.x -= left;
            }
        } else if right >= 131.0 {
            tag.base.x -= right - 131.0;
        }
        tag
    }

    pub(super) fn confirmation(parent: Self, width: f32, rows: usize, selected: usize) -> Self {
        // mTG_chg_tag_func2 followed by select_win_after_select.
        let base = Vec2::new(
            parent.base.x + parent.body.x + parent.text.x - 3.5,
            parent.base.y + (parent.body.y + parent.text.y - selected as f32 * 16.0 - 8.0) * 0.875,
        );
        let mut tag = Self::sized(base, width * 0.875, rows as f32 * 14.0, false);
        tag.base.x -= tag.fraction.x * 0.875 + 0.875;
        tag.base.y += tag.fraction.y * 8.75 + 6.125 - 7.0;
        tag.body = tag.half() * Vec2::new(1.0, -1.0);
        if tag.right() > 134.0 {
            tag.body.x -= tag.right() - 134.0;
        }
        if !tag.foot() {
            tag.body.y = -tag.body.y;
            tag.base.y = base.y - (tag.fraction.y * 8.75 + 6.125) + 7.0;
            let top = tag.base.y + tag.body.y + tag.half().y;
            if top > 120.0 {
                tag.base.y -= top - 120.0;
            }
        }
        tag
    }
}
