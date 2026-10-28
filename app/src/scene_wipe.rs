//! The screen fade of a scene change (`fbdemo_fade`, `Game_play_fbdemo_wipe_*`).
//!
//! Leaving a scene fades the picture to black (`FADE_TYPE_OUT`: alpha rises over the fade rate's
//! 30 frames); the next scene starts black, holds ten frames and fades back in
//! (`FADE_TYPE_IN`, `fbdemo_fade_startup`'s timer of 10). Every change of building or room
//! fades in here; the fade out needs the leaving system to wait for it, which the part-time
//! job's send-away does (`guide::send_away`). The circle wipes (`WIPE_TYPE_CIRCLE_LEFT/RIGHT`,
//! the triforce wipe's `ef_wipe*` models) are not built: they fade like the others.

use bevy::prelude::*;

use crate::house::HouseState;
use crate::shop::ShopLocation;

/// `Common_Get(transition).fade_rate`: frames a fade takes.
const FADE_FRAMES: f32 = 30.0;
/// `fbdemo_fade_startup`: the frames a fade in stays black first.
const HOLD_FRAMES: f32 = 10.0;
/// How long a fade out waits for the scene to change.
const COVER_LIMIT: f32 = 600.0;

#[derive(Clone, Copy, Default, PartialEq, Debug)]
enum Phase {
    #[default]
    Clear,
    /// Fading to black, frames so far.
    Out(f32),
    /// Faded out, waiting for the scene to change, frames so far.
    Covered(f32),
    /// The new scene's first black frames, so far.
    Black(f32),
    /// Fading back in, frames so far.
    In(f32),
}

#[derive(Resource, Default)]
pub(crate) struct SceneWipe {
    phase: Phase,
}

impl SceneWipe {
    /// Nothing is covering the picture.
    pub(crate) fn clear(&self) -> bool {
        self.phase == Phase::Clear
    }

    /// The picture is fully black.
    pub(crate) fn black(&self) -> bool {
        matches!(self.phase, Phase::Covered(_) | Phase::Black(_))
    }

    /// `FADE_TYPE_OUT`: starts fading to black, once.
    pub(crate) fn fade_out(&mut self) {
        if self.clear() {
            info!("scene wipe: out");
            self.phase = Phase::Out(0.0);
        }
    }

    /// The new scene starts black and fades in.
    fn arrive(&mut self) {
        info!("scene wipe: in");
        self.phase = Phase::Black(0.0);
    }

    fn alpha(&self) -> f32 {
        match self.phase {
            Phase::Clear => 0.0,
            Phase::Out(f) => (f / FADE_FRAMES).min(1.0),
            Phase::Covered(_) | Phase::Black(_) => 1.0,
            Phase::In(f) => 1.0 - (f / FADE_FRAMES).min(1.0),
        }
    }

    fn advance(&mut self, frames: f32) {
        self.phase = match self.phase {
            Phase::Clear => Phase::Clear,
            // A fade out ends black and waits there for the scene change.
            Phase::Out(f) if f + frames >= FADE_FRAMES => Phase::Covered(0.0),
            Phase::Out(f) => Phase::Out(f + frames),
            // Should the scene never change, the screen is not left black.
            Phase::Covered(f) if f + frames >= COVER_LIMIT => Phase::In(0.0),
            Phase::Covered(f) => Phase::Covered(f + frames),
            Phase::Black(f) if f + frames >= HOLD_FRAMES => Phase::In(0.0),
            Phase::Black(f) => Phase::Black(f + frames),
            Phase::In(f) if f + frames >= FADE_FRAMES => Phase::Clear,
            Phase::In(f) => Phase::In(f + frames),
        };
    }
}

#[derive(Component)]
struct WipeCover;

fn setup(mut commands: Commands) {
    commands.spawn((
        WipeCover,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        GlobalZIndex(i32::MAX - 1),
        Visibility::Hidden,
    ));
}

/// A new building or room: the screen starts black.
fn arrive(
    shop: Option<Res<State<ShopLocation>>>,
    house: Option<Res<State<HouseState>>>,
    mut wipe: ResMut<SceneWipe>,
    mut started: Local<bool>,
) {
    // The states exist once the game starts.
    let (Some(shop), Some(house)) = (shop, house) else {
        return;
    };
    if !*started {
        *started = true;
        return;
    }
    if shop.is_changed() || house.is_changed() {
        wipe.arrive();
    }
}

fn run(
    time: Res<Time<Real>>,
    mut wipe: ResMut<SceneWipe>,
    mut cover: Query<(&mut BackgroundColor, &mut Visibility), With<WipeCover>>,
) {
    wipe.advance(time.delta_secs() * 60.0);
    let alpha = wipe.alpha();
    for (mut color, mut visibility) in &mut cover {
        color.0 = Color::srgba(0.0, 0.0, 0.0, alpha);
        *visibility = if alpha > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub struct SceneWipePlugin;

impl Plugin for SceneWipePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneWipe>()
            .add_systems(Startup, setup)
            .add_systems(Update, (arrive, run).chain());
    }
}
