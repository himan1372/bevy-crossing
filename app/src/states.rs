//! Top-level game states.
//!
//! - [`AppState`]: `Loading` (wait for assets) → `Title` (title screen; not
//!   built yet, so loading goes straight to `InGame`) → `InGame`.
//! - [`PauseState`]: a sub-state of `InGame`, so pausing keeps the
//!   `InGame` sub-states (house / shop location) intact.
//!
//! Every gameplay system belongs to [`GameplaySet`], which only runs while
//! in game and not paused. World setup still happens in `Startup`, behind
//! the `Loading` state.

use bevy::prelude::*;

use crate::input::{Action, Actions, InputFocus};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    /// Reserved for the title screen (roadmap `title-screen`).
    #[allow(dead_code)]
    Title,
    InGame,
}

#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::InGame)]
pub enum PauseState {
    #[default]
    Running,
    Paused,
}

/// Systems that simulate or take gameplay input. Runs only while
/// `PauseState::Running` (which implies `AppState::InGame`).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameplaySet;

/// P toggles pause, but only while the world has input focus (so typing a
/// letter or browsing the shop never pauses by accident). The pause menu
/// (roadmap `pause-menu`) will replace the bare "Paused" label.
pub(crate) fn toggle_pause(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<PauseState>>,
    mut next: ResMut<NextState<PauseState>>,
) {
    if !matches!(
        *focus,
        InputFocus::World | InputFocus::Furniture | InputFocus::Paused
    ) {
        return;
    }
    if !actions.consume(Action::Pause) {
        return;
    }
    crate::sound::se(match state.get() {
        PauseState::Running => crate::sound::se_id::MENU_PAUSE,
        PauseState::Paused => crate::sound::se_id::MENU_EXIT,
    });
    next.set(match state.get() {
        PauseState::Running => PauseState::Paused,
        PauseState::Paused => PauseState::Running,
    });
}

fn enter_pause(mut time: ResMut<Time<Virtual>>) {
    time.pause();
    println!("Paused");
}

fn exit_pause(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
    println!("Resumed");
}

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_sub_state::<PauseState>()
            .configure_sets(Update, GameplaySet.run_if(in_state(PauseState::Running)))
            .add_systems(Update, toggle_pause.run_if(in_state(AppState::InGame)))
            .add_systems(OnEnter(PauseState::Paused), enter_pause)
            .add_systems(OnExit(PauseState::Paused), exit_pause);
    }
}
