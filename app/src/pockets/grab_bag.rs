//! mTG_hukubukuro_open_proc and mIV item shrink/staggered growth.
use crate::interaction::PlayerInventory;

pub(crate) struct Opening {
    pub bag_slot: usize,
    slots: [usize; 3],
    pub ticks: f32,
}

pub(super) enum Refusal {
    Capacity,
    Unavailable,
}

impl Opening {
    pub(super) fn start(
        inventory: &mut PlayerInventory,
        slot: usize,
        rand: &mut rustimal_logic::qrand::Qrand,
    ) -> Result<Self, Refusal> {
        let free: Vec<usize> = inventory
            .pockets
            .pockets
            .iter()
            .enumerate()
            .filter_map(|(i, &item)| (item == 0).then_some(i))
            .take(3)
            .collect();
        let slots: [usize; 3] = free.try_into().map_err(|_| Refusal::Capacity)?;
        if inventory.pockets.pockets.get(slot) != Some(&crate::goods::GRAB_BAG)
            || inventory.pockets.item_cond(slot) != 0
        {
            return Err(Refusal::Unavailable);
        }
        // `RANDOM(n)` on the shared `qrand` state.
        let mut below = |n: u32| rand.random(n.max(1) as i32) as u32;
        let items = crate::goods::grab_bag_contents(&mut below).ok_or(Refusal::Unavailable)?;
        let mut staged = inventory.pockets.clone();
        staged.pockets[slot] = 0;
        staged.set_item_cond(slot, 0);
        for (destination, item) in slots.into_iter().zip(items) {
            staged.pockets[destination] = item;
            staged.set_item_cond(destination, 0);
        }
        inventory.pockets = staged;
        bevy::log::info!("Opened grab bag slot={slot} destinations={slots:?} items={items:04x?}");
        Ok(Self {
            bag_slot: slot,
            slots,
            ticks: 0.0,
        })
    }

    /// Source 70-tick timer: bag shrinks in the final12; growths are14 apart.
    pub fn scale(&self, slot: usize) -> f32 {
        let remaining = (70.0 - self.ticks).max(0.0);
        if slot == self.bag_slot {
            (remaining / 12.0).clamp(0.0, 1.0)
        } else if let Some(i) = self.slots.iter().position(|&s| s == slot) {
            (1.0 - (remaining - 14.0 - i as f32 * 14.0) / 12.0).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }
}
