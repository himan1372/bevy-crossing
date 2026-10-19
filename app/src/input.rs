//! Input: one key map, consumable presses, and a single input-focus rule.
//!
//! - [`KeyBindings`] maps each [`Action`] to keys. Systems ask for actions,
//!   never raw `KeyCode`s, so rebinding (or a gamepad) is a one-place change.
//! - [`Actions`] is rebuilt every frame in `PreUpdate`. `consume` hands a
//!   press to the first system that asks, so one E press can never fire two
//!   handlers (door + talk, open + advance, ...).
//! - [`InputFocus`] says who owns input this frame, derived in one place from
//!   the open UIs. World handlers run only while focus is `World`; each UI
//!   reads its own keys while it has focus.
//!
//! Interact order (first to consume wins), see [`InteractSet`]:
//! doors → objects (mailbox, shop counter) → held tool → talk / pick up.
//! Free-text typing in the letter composer still reads raw keyboard
//! messages; it only runs while the mail UI has focus.

use bevy::input::InputSystems;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;

use crate::states::PauseState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    /// Hold to run instead of walk.
    Run,
    /// Hold beside furniture, then use movement directions to push/pull/turn.
    FurnitureGrab,
    /// Doors, mailbox, talk, pick up, use tool.
    Interact,
    /// Advance dialogue (E also advances, via `Interact`).
    Advance,
    Tool1,
    Tool2,
    Tool3,
    /// Close / back out of a UI.
    Cancel,
    /// Confirm a menu selection.
    Confirm,
    /// Back one step in a UI (letter reader, composer delete).
    Back,
    MenuUp,
    MenuDown,
    /// Quest panel accept.
    Yes,
    /// Quest panel decline.
    No,
    /// Mailbox: write a new letter.
    Compose,
    /// Shop menu: buy tab.
    Tab1,
    /// Shop menu: sell tab.
    Tab2,
    QuickSave,
    QuickLoad,
    Pause,
    /// Open / close the pockets screen.
    Pockets,
    /// Debug: move the game clock forward one hour.
    DebugHour,
    /// Debug: talk to the nearest villager outdoors, wherever they are.
    DebugTalk,
    /// Debug: compare the same spoken line across villager personalities.
    DebugVoice,
    /// Playtest aid: add an original wardrobe to an empty pocket in your room.
    DebugFurniture,
    /// Playtest aid: add original wide drawers to an empty pocket in your room.
    DebugDrawers,
    /// Playtest aid: add K.K. Love Song to an empty pocket in your room.
    DebugMusic,
    DebugToys,
    /// Toggle a registered music title's take-out mark.
    Mark,
}

/// Keys for each action. Several actions may share a key: they are read in
/// different focus contexts (e.g. Digit1 is Tool1 in the world, Tab1 in the
/// shop menu).
#[derive(Resource)]
pub struct KeyBindings(pub HashMap<Action, Vec<KeyCode>>);

impl Default for KeyBindings {
    fn default() -> Self {
        use Action::*;
        use KeyCode as K;
        let map = [
            (MoveUp, vec![K::KeyW, K::ArrowUp]),
            (MoveDown, vec![K::KeyS, K::ArrowDown]),
            (MoveLeft, vec![K::KeyA, K::ArrowLeft]),
            (MoveRight, vec![K::KeyD, K::ArrowRight]),
            (Run, vec![K::ShiftLeft, K::ShiftRight]),
            (FurnitureGrab, vec![K::KeyG]),
            (Interact, vec![K::KeyE]),
            (Advance, vec![K::Space]),
            (Tool1, vec![K::Digit1]),
            (Tool2, vec![K::Digit2]),
            (Tool3, vec![K::Digit3]),
            (Cancel, vec![K::Escape]),
            (Confirm, vec![K::Enter]),
            (Back, vec![K::Backspace]),
            (MenuUp, vec![K::ArrowUp]),
            (MenuDown, vec![K::ArrowDown]),
            (Yes, vec![K::KeyY]),
            (No, vec![K::KeyN]),
            (Compose, vec![K::KeyN]),
            (Tab1, vec![K::Digit1]),
            (Tab2, vec![K::Digit2]),
            (QuickSave, vec![K::F5]),
            (QuickLoad, vec![K::F9]),
            (Pause, vec![K::KeyP]),
            (Pockets, vec![K::KeyI, K::Tab]),
            (DebugHour, vec![K::F6]),
            (DebugTalk, vec![K::F7]),
            (DebugVoice, vec![K::F8]),
            (DebugFurniture, vec![K::F4]),
            (DebugDrawers, vec![K::F3]),
            (DebugMusic, vec![K::F2]),
            (DebugToys, vec![K::F1]),
            (Mark, vec![K::KeyM]),
        ];
        Self(map.into_iter().collect())
    }
}

/// This frame's action state.
#[derive(Resource, Default)]
pub struct Actions {
    pressed: HashSet<Action>,
    just_pressed: HashSet<Action>,
    consumed: HashSet<Action>,
}

impl Actions {
    /// Test helper: these actions held down (not just pressed).
    #[cfg(test)]
    pub fn holding(held: &[Action]) -> Self {
        Self {
            pressed: held.iter().copied().collect(),
            ..Self::default()
        }
    }

    pub fn pressed(&self, action: Action) -> bool {
        self.pressed.contains(&action)
    }

    pub fn just_pressed(&self, action: Action) -> bool {
        self.just_pressed.contains(&action)
    }

    /// True for the first caller this frame whose action was just pressed;
    /// false for everyone after. Use for presses that must fire one handler.
    pub fn consume(&mut self, action: Action) -> bool {
        self.just_pressed.contains(&action) && self.consumed.insert(action)
    }

    /// Movement direction on the ground plane (x, 0, z), not normalized.
    /// Up is -z (into the screen), matching the camera.
    pub fn move_dir(&self) -> Vec3 {
        let mut dir = Vec3::ZERO;
        if self.pressed(Action::MoveUp) {
            dir.z -= 1.0;
        }
        if self.pressed(Action::MoveDown) {
            dir.z += 1.0;
        }
        if self.pressed(Action::MoveLeft) {
            dir.x -= 1.0;
        }
        if self.pressed(Action::MoveRight) {
            dir.x += 1.0;
        }
        dir
    }
}

/// Who owns input this frame. Derived in [`update_focus`] — the one place
/// the "is some UI open?" question is answered.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFocus {
    #[default]
    World,
    Dialogue,
    Mail,
    Shop,
    Quest,
    Pockets,
    Paused,
    Furniture,
}

impl InputFocus {
    pub fn is_world(self) -> bool {
        self == InputFocus::World
    }
}

/// Order for systems that may consume `Action::Interact`: the first set to
/// consume the press wins.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InteractSet {
    Doors,
    Objects,
    Tools,
    Talk,
    /// UIs that open from an Interact press read their keys after it.
    Ui,
}

fn read_actions(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    mut actions: ResMut<Actions>,
) {
    actions.pressed.clear();
    actions.just_pressed.clear();
    actions.consumed.clear();
    for (action, codes) in bindings.0.iter() {
        if keys.any_pressed(codes.iter().copied()) {
            actions.pressed.insert(*action);
        }
        if keys.any_just_pressed(codes.iter().copied()) {
            actions.just_pressed.insert(*action);
        }
    }
}

/// Priority: pause, then the modal UIs, else the world.
pub(crate) fn update_focus(
    pause: Option<Res<State<PauseState>>>,
    dialogue: Res<crate::dialogue::DialogueState>,
    mail: Res<crate::mail::MailUi>,
    shop: Res<crate::shop::ShopUi>,
    quest: Res<crate::quest::QuestUi>,
    pockets: Res<crate::pockets::PocketsUi>,
    room: Res<crate::player_room::PlayerRoom>,
    mut focus: ResMut<InputFocus>,
) {
    let next = if pause.is_some_and(|p| *p.get() == PauseState::Paused) {
        InputFocus::Paused
    } else if quest.open {
        InputFocus::Quest
    } else if dialogue.active
        || dialogue.external_result.is_some()
        || (pockets.selling && !pockets.open)
        || pockets.room_request.is_some()
        || (pockets.storing.is_some() && !pockets.open)
    {
        InputFocus::Dialogue
    } else if mail.open {
        InputFocus::Mail
    } else if shop.open {
        InputFocus::Shop
    } else if pockets.open {
        InputFocus::Pockets
    } else if room.manipulating() {
        InputFocus::Furniture
    } else {
        InputFocus::World
    };
    if *focus != next {
        *focus = next;
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeyBindings>()
            .init_resource::<Actions>()
            .init_resource::<InputFocus>()
            .add_systems(
                PreUpdate,
                (read_actions, update_focus).chain().after(InputSystems),
            )
            .configure_sets(
                Update,
                (
                    InteractSet::Doors,
                    InteractSet::Objects,
                    InteractSet::Tools,
                    InteractSet::Talk,
                    InteractSet::Ui,
                )
                    .chain(),
            );
    }
}
