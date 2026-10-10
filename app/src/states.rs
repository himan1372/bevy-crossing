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
use bevy::text::FontSize;

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

#[derive(Component)]
struct PausedText;

/// P toggles pause, but only while the world has input focus (so typing a
/// letter or browsing the shop never pauses by accident). The pause menu
/// (roadmap `pause-menu`) will replace the bare "Paused" label.
fn toggle_pause(
    mut actions: ResMut<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<PauseState>>,
    mut next: ResMut<NextState<PauseState>>,
) {
    if !matches!(*focus, InputFocus::World | InputFocus::Paused) {
        return;
    }
    if !actions.consume(Action::Pause) {
        return;
    }
    next.set(match state.get() {
        PauseState::Running => PauseState::Paused,
        PauseState::Paused => PauseState::Running,
    });
}

fn enter_pause(
    mut time: ResMut<Time<Virtual>>,
    mut text: Query<&mut Visibility, With<PausedText>>,
) {
    time.pause();
    for mut vis in text.iter_mut() {
        *vis = Visibility::Visible;
    }
    println!("Paused");
}

fn exit_pause(mut time: ResMut<Time<Virtual>>, mut text: Query<&mut Visibility, With<PausedText>>) {
    time.unpause();
    for mut vis in text.iter_mut() {
        *vis = Visibility::Hidden;
    }
    println!("Resumed");
}

fn setup_paused_text(mut commands: Commands) {
    commands.spawn((
        PausedText,
        Text::new("Paused — press P to resume"),
        TextFont {
            font_size: FontSize::Px(32.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(45.0),
            left: Val::Percent(35.0),
            ..default()
        },
        Visibility::Hidden,
    ));
}

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_sub_state::<PauseState>()
            .configure_sets(Update, GameplaySet.run_if(in_state(PauseState::Running)))
            .add_systems(Startup, setup_paused_text)
            .add_systems(Update, toggle_pause.run_if(in_state(AppState::InGame)))
            .add_systems(OnEnter(PauseState::Paused), enter_pause)
            .add_systems(OnExit(PauseState::Paused), exit_pause);
    }
}
