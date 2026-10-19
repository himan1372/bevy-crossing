//! ac_my_room_msg_ctrl / mTG_putin_proc / mTG_music_listen/takeout_proc.
//! The home owns the registered copies; each device stores only a selection.
use super::*;

const FIRST: u16 = 0x2A00;
const COUNT: u16 = 55;

pub(super) enum Flow {
    Choose(usize),
}

pub(super) fn is_song(item: u16) -> bool {
    (FIRST..FIRST + COUNT).contains(&item)
}

pub(super) fn registered(room: &PlayerRoom, item: u16) -> bool {
    is_song(item) && {
        let i = (item - FIRST) as usize;
        room.music_box[i / 32] & (1 << (i % 32)) != 0
    }
}

pub(super) fn is_player(room: &PlayerRoom, owner: usize) -> bool {
    room.runtime
        .actors
        .get(owner)
        .and_then(Option::as_ref)
        .and_then(|a| room.profiles.get(&a.name))
        .is_some_and(|p| p.interaction_type & ftr::interaction::MUSIC_DISK != 0)
}

pub(super) fn switch(room: &mut PlayerRoom, owner: usize, on: bool) {
    // aMR_AllMDSwitchOff / OneMDSwitchOn_TheOtherSwitchOff.
    for (slot, actor) in room.runtime.actors.iter_mut().enumerate() {
        let Some(actor) = actor else { continue };
        if room.profiles.get(&actor.name).is_some_and(|p| {
            p.interaction_type & (ftr::interaction::MUSIC_DISK | ftr::interaction::RADIO_AEROBICS)
                != 0
        }) {
            let selected = on && slot == owner;
            actor.switch_bit = u8::from(selected);
            actor.switch_changed = true;
            if let Some(bit) = RoomLayer::switch_bit_index(actor.cell_x, actor.cell_z) {
                let bits = &mut room.layers[actor.layer].ftr_switch;
                *bits &= !(1u64 << bit);
                if selected {
                    *bits |= 1u64 << bit;
                }
            }
        }
    }
}

pub(super) fn register(
    room: &mut PlayerRoom,
    inventory: &mut PlayerInventory,
    owner: usize,
    slot: usize,
    item: u16,
) -> Result<(), &'static str> {
    if !is_player(room, owner)
        || !is_song(item)
        || inventory.pockets.pockets.get(slot) != Some(&item)
        || inventory.pockets.item_cond(slot) != 0
    {
        return Err("That music is no longer available.");
    }
    if registered(room, item) {
        return Err("That song is already registered.");
    }
    let i = (item - FIRST) as usize;
    room.music_box[i / 32] |= 1 << (i % 32);
    inventory.pockets.pockets[slot] = 0;
    inventory.pockets.set_item_cond(slot, 0);
    if let Some(actor) = room.runtime.actors[owner].as_mut() {
        actor.items[0] = item;
    }
    switch(room, owner, true);
    Ok(())
}

pub(super) fn begin(
    room: &mut PlayerRoom,
    dialogue: &mut DialogueState,
    focus: &mut InputFocus,
    owner: usize,
) {
    let title = room.runtime.actors[owner]
        .as_ref()
        .map_or(0, |a| a.items[0]);
    let message = if registered(room, title) {
        format!(
            "Selected: {}\nWhat would you like to do?",
            crate::items::name(title)
        )
    } else {
        "What would you like to do?".into()
    };
    dialogue.begin_furniture(
        message,
        Some(vec![
            "Register a song".into(),
            "Choose a song".into(),
            "Stop playing".into(),
            "Take out a song".into(),
            "Quit".into(),
        ]),
    );
    room.flow = StorageFlow::Music(Flow::Choose(owner));
    *focus = InputFocus::Dialogue;
}

pub(super) fn answer(
    room: &mut PlayerRoom,
    inventory: &mut PlayerInventory,
    ui: &mut PocketsUi,
    dialogue: &mut DialogueState,
    focus: &mut InputFocus,
    flow: Flow,
    choice: usize,
) {
    let Flow::Choose(owner) = flow;
    if !is_player(room, owner) {
        return;
    }
    match flow {
        Flow::Choose(_) => match choice {
            0 => {
                if !inventory
                    .pockets
                    .pockets
                    .iter()
                    .enumerate()
                    .any(|(i, &item)| is_song(item) && inventory.pockets.item_cond(i) == 0)
                {
                    notice(dialogue, room, focus, "You aren't carrying any music.");
                    return;
                }
                ui.open = true;
                ui.selling = false;
                ui.storing = Some(owner);
                ui.registering = true;
                ui.menu = None;
                ui.grabbed = None;
                ui.mail_slot = None;
                room.flow = StorageFlow::Pockets;
                *focus = InputFocus::Pockets;
            }
            1 | 3 => {
                if room.music_box == [0, 0] {
                    notice(dialogue, room, focus, "There are no registered songs.");
                    return;
                }
                let selected = room.runtime.actors[owner]
                    .as_ref()
                    .map_or(0, |a| a.items[0].saturating_sub(FIRST).min(COUNT - 1))
                    as usize;
                room.music_grid = Some(super::music_grid::Grid::new(owner, selected, choice == 3));
                *focus = InputFocus::Furniture;
            }
            2 if room.runtime.actors[owner]
                .as_ref()
                .is_some_and(|a| a.switch_bit != 0) =>
            {
                switch(room, owner, false)
            }
            _ => {}
        },
    }
}

impl PlayerRoom {
    pub(crate) fn music_source(&self) -> Option<(Vec3, u16)> {
        self.runtime
            .actors
            .iter()
            .enumerate()
            .find_map(|(owner, actor)| {
                let actor = actor.as_ref()?;
                (is_player(self, owner)
                    && actor.switch_bit != 0
                    && registered(self, actor.items[0]))
                .then(|| (super::presentation::actor_center(self, actor), actor.name))
            })
    }
    pub(crate) fn music_menu_open(&self) -> bool {
        self.music_grid.is_some()
    }
    pub(crate) fn music_track(&self) -> Option<rustimal_logic::scene_music::Track> {
        self.runtime
            .actors
            .iter()
            .enumerate()
            .find_map(|(owner, actor)| {
                let actor = actor.as_ref()?;
                (is_player(self, owner)
                    && actor.switch_bit != 0
                    && registered(self, actor.items[0]))
                .then(|| rustimal_logic::scene_music::Track {
                    id: 128 + (actor.items[0] - FIRST) as u8, // BGM_MD0
                    mute_mask: 0,
                })
            })
    }
}
