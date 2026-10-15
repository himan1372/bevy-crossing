//! Door sound timelines from ac_my_house_move/ac_house_move and the
//! upgraded shops' setup_action. Each pair is (60 Hz ticks, sound ID).
//! Houses advance animation frames at 0.5 per tick, starting at frame 1.
//! The app currently teleports through doors; these cues preserve spacing
//! without claiming the missing door animations/scene wipe are implemented.
use crate::shop::ShopTier;

pub fn door_cues(shop: Option<ShopTier>, entering: bool) -> &'static [(u16, u16)] {
    match shop {
        // Cranny's door stands open during business; player traversal
        // does not run its business-hours opening/closing animation.
        Some(ShopTier::Zakka) => &[],
        // Sliding doors: open at action start, close after the 30-tick
        // opening span. Proximity-driven hold timing awaits actor animation.
        Some(_) => &[(0, 0x44B), (30, 0x44C)],
        None if entering => &[(2, 6), (14, 7), (64, 8), (78, 9)],
        None => &[(18, 6), (26, 7), (68, 8), (98, 9)],
    }
}
