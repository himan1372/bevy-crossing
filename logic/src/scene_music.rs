//! Music policy for the scenes currently playable in the port.
//!
//! Source: m_bgm.c (mBGMRoom_make_scene_bgm[_shop_get], volume_talk,
//! volume_pause, main_req_volume), m_shop.c (GetShopCloseTime_Bgm), and
//! game64.c_inc (Sou_BgmTenkiConv). The app supplies the player's active
//! furniture song after this policy's quiet-house result. Event arbitration
//! remains a follow-up.

use crate::audio::{BgmWeather, BGM_MUTE_TABLE_FINE, BGM_MUTE_TABLE_SAKURA, BGM_MUTE_TABLE_SNOW};
use crate::shop::ShopTier;

pub use crate::audio::BgmWeather as FieldWeather;

/// Retail scene cleanup requests 0x168 sequencer updates.
pub const SCENE_FADE_UPDATES: i32 = 0x168;
/// Closing's quiet-room stop type 0x13DE carries a 0x3DE update fade.
pub const SHOP_CLOSING_FADE_UPDATES: i32 = 0x3DE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scene {
    Town,
    PlayerHouse,
    Shop(ShopTier),
    /// `mFI_FIELD_ROOM_POST_OFFICE`; `true` while Phyllis is on duty.
    PostOffice(bool),
    /// `mFI_FIELD_ROOM_POLICE_BOX`.
    PoliceBox,
}

/// Music ID (resolved through the disc's SEQ_TABLE), plus hourly arrangement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Track {
    pub id: u8,
    pub mute_mask: u16,
}

/// The player house does not request a room song. Without a furniture player
/// it is quiet; the outdoor hourly track must not follow the player inside.
/// Shop close themes begin five minutes before the tier's nominal closing
/// hour. Door opening hours/part-time work are separate gameplay follow-ups.
pub fn select(scene: Scene, day_secs: u32, weather: FieldWeather) -> Option<Track> {
    let (id, mute_mask) = match scene {
        Scene::PlayerHouse => return None,
        Scene::PostOffice(night) => (crate::post_office::music(night), 0),
        Scene::PoliceBox => (45, 0),
        Scene::Town => {
            let hour = (day_secs / 3600 % 24) as usize;
            match weather {
                BgmWeather::Rain => (0x45, 0),
                BgmWeather::Fine => (hour as u8 + 1, BGM_MUTE_TABLE_FINE[hour]),
                BgmWeather::Snow => (hour as u8 + 1, BGM_MUTE_TABLE_SNOW[hour]),
                BgmWeather::Sakura => (hour as u8 + 1, BGM_MUTE_TABLE_SAKURA[hour]),
                BgmWeather::Other => (hour as u8 + 1, 0),
            }
        }
        Scene::Shop(tier) => {
            let i = tier.tier_idx() as usize;
            let close_hour = [22, 23, 22, 22][i];
            let closing_start = close_hour * 3600 - 5 * 60;
            let seconds = day_secs % 86400;
            // mBGMRoom_shop_close_check inserts a five-second quiet interval
            // before swapping to the closing arrangement.
            if (closing_start - 5..closing_start).contains(&seconds) {
                return None;
            }
            let closing = seconds >= closing_start;
            let ids = if closing {
                [79, 80, 81, 82]
            } else {
                [44, 37, 38, 39]
            };
            (ids[i], 0)
        }
    };
    Some(Track { id, mute_mask })
}

/// Inventory/pause requests take precedence over conversation ducking,
/// following the original's minimum-volume arbitration rather than products.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ducking {
    pub menu: bool,
    pub talking: bool,
}

impl Ducking {
    pub fn volume(self) -> f32 {
        if self.menu {
            0.5
        } else if self.talking {
            0.7
        } else {
            1.0
        }
    }

    /// Na_BGMVolume's timer is in 60 Hz game frames. Menu in/out takes two
    /// frames; talking starts/ends over thirty.
    pub fn transition_frames(self, previous: Self) -> u16 {
        if self.menu != previous.menu {
            2
        } else {
            30
        }
    }
}

/// Linear game-layer volume movement, independent of the sequencer's own
/// song fade. The app supplies real time so menu fades continue while paused.
pub struct VolumeRamp {
    pub current: f32,
    target: f32,
    remaining: f32,
}

impl Default for VolumeRamp {
    fn default() -> Self {
        Self {
            current: 1.0,
            target: 1.0,
            remaining: 0.0,
        }
    }
}

impl VolumeRamp {
    pub fn retarget(&mut self, target: f32, frames: u16) {
        self.target = target.clamp(0.0, 1.0);
        self.remaining = f32::from(frames) / 60.0;
        if frames == 0 {
            self.current = self.target;
        }
    }

    pub fn advance(&mut self, seconds: f32) {
        if self.remaining > 0.0 {
            let elapsed = seconds.max(0.0).min(self.remaining);
            self.current += (self.target - self.current) * elapsed / self.remaining;
            self.remaining -= elapsed;
            if self.remaining == 0.0 {
                self.current = self.target;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returning_from_a_quiet_house_uses_the_current_town_hour() {
        assert_eq!(
            select(Scene::Town, 3 * 3600, FieldWeather::Fine)
                .unwrap()
                .id,
            4
        );
        assert!(select(Scene::PlayerHouse, 3 * 3600, FieldWeather::Fine).is_none());
        assert_eq!(
            select(Scene::Town, 23 * 3600, FieldWeather::Fine)
                .unwrap()
                .id,
            24
        );
    }

    #[test]
    fn menu_priority_and_real_time_ramps_restore_levels() {
        let menu = Ducking {
            menu: true,
            talking: true,
        };
        let talk = Ducking {
            menu: false,
            talking: true,
        };
        assert_eq!(menu.volume(), 0.5);
        assert_eq!(talk.volume(), 0.7);
        assert_eq!(menu.transition_frames(talk), 2);
        let mut ramp = VolumeRamp::default();
        ramp.retarget(menu.volume(), 2);
        // Real time continues while the virtual game clock is paused.
        ramp.advance(0.25);
        assert_eq!(ramp.current, 0.5);
        ramp.retarget(talk.volume(), 2);
        ramp.advance(0.25);
        assert_eq!(ramp.current, 0.7);
        ramp.retarget(1.0, 30);
        ramp.advance(1.0);
        assert_eq!(ramp.current, 1.0);
    }
}
