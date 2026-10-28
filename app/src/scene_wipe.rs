//! The screen fade of a scene change (`fbdemo_fade`, `Game_play_fbdemo_wipe_*`).
//!
//! Leaving a scene fades the picture to black (`FADE_TYPE_OUT`: alpha rises over the fade rate's
//! 30 frames); the next scene starts black, holds ten frames and fades back in
//! (`FADE_TYPE_IN`, `fbdemo_fade_startup`'s timer of 10). Every change of building or room
//! fades in here. An ordinary door's change is caught after the door system has made it
//! (`doors`): the player is put back, input is frozen, the picture fades out, then the change is
//! applied. The part-time job's send-away drives its own fade (`guide::send_away`), and the town it sends the player to opens as the
//! circle wipe (`WIPE_TYPE_CIRCLE_LEFT`: `fbdemo_triforce` with `ef_wipe2_modelT`, a 4:3 picture,
//! black beside it on a wider window). `WIPE_TYPE_CIRCLE_RIGHT` (`ef_wipe3_modelT`) and the
//! closing direction are not used by anything yet.

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::game_disc::GameDisc;
use crate::house::HouseState;
use crate::input::InputFocus;
use crate::interaction::Facing;
use crate::n64_material::N64Material;
use crate::player::Player;
use crate::shop::ShopLocation;

/// `Common_Get(transition).fade_rate`: frames a fade takes.
const FADE_FRAMES: f32 = 30.0;
/// `fbdemo_fade_startup`: the frames a fade in stays black first.
const HOLD_FRAMES: f32 = 10.0;
/// How long a fade out waits for the scene to change.
const COVER_LIMIT: f32 = 600.0;
/// `fbdemo_triforce`: the wipe texture's full scroll (`txt`), in quarter texels.
const CIRCLE_SCROLL: f32 = 548.0;
/// `fbdemo_triforce_draw`: the model's scale, and `guLookAt` / `guPerspective`'s camera.
const CIRCLE_SCALE: f32 = 0.019;
const CIRCLE_CAMERA_Z: f32 = 400.0;
/// The render layer the circle wipe's camera shows.
const CIRCLE_LAYER: usize = 2;

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
    /// A circle wipe: the texture's scroll (`txt`, 548 closed). Opening on the new scene, or
    /// `closing` on the old one; `right` is `ef_wipe3_modelT` (`WIPE_TYPE_CIRCLE_RIGHT`).
    Circle {
        txt: f32,
        closing: bool,
        right: bool,
    },
}

#[derive(Resource, Default)]
pub(crate) struct SceneWipe {
    phase: Phase,
    /// The next arrival opens as a circle (`Common_Get(transition).wipe_type`): right or left.
    circle_next: Option<bool>,
    /// A scene change that is no state change (the museum's room doors) waits for the black.
    holding: bool,
}

impl SceneWipe {
    /// The picture is fading out or waiting for the scene change: input is frozen.
    fn locking(&self) -> bool {
        matches!(self.phase, Phase::Out(_) | Phase::Covered(_))
    }

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

    /// `FADE_TYPE_OUT` for a change the caller applies itself once `black()`: input is frozen
    /// until `release`. False when the picture is already covered or fading.
    pub(crate) fn hold(&mut self) -> bool {
        if !self.clear() {
            return false;
        }
        self.holding = true;
        self.fade_out();
        true
    }

    /// The change made under `hold` is in: the new scene fades in.
    pub(crate) fn release(&mut self) {
        self.holding = false;
        self.arrive();
    }

    /// The new scene starts black and fades in.
    fn arrive(&mut self) {
        info!("scene wipe: in");
        self.phase = match self.circle_next.take() {
            Some(right) => Phase::Circle {
                txt: CIRCLE_SCROLL,
                closing: false,
                right,
            },
            None => Phase::Black(0.0),
        };
    }

    /// `WIPE_TYPE_CIRCLE_LEFT` (or `_RIGHT`) for the scene about to load.
    pub(crate) fn circle_next(&mut self, right: bool) {
        self.circle_next = Some(right);
    }

    /// The fade out as a closing circle (`fbdemo_triforce` with direction 1), once.
    #[allow(dead_code)] // No scene change in the source leaves by a circle yet.
    pub(crate) fn circle_out(&mut self, right: bool) {
        if self.clear() {
            info!("scene wipe: circle out");
            self.phase = Phase::Circle {
                txt: 0.0,
                closing: true,
                right,
            };
        }
    }

    fn alpha(&self) -> f32 {
        match self.phase {
            Phase::Clear => 0.0,
            Phase::Out(f) => (f / FADE_FRAMES).min(1.0),
            Phase::Covered(_) | Phase::Black(_) => 1.0,
            // The circle's own geometry covers the picture.
            Phase::Circle { .. } => 0.0,
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
            // `fbdemo_triforce_move`: it opens a little faster as it goes.
            Phase::Circle {
                txt,
                closing,
                right,
            } => {
                let t = 1.0 - txt / CIRCLE_SCROLL;
                let step = 0.5 * (14.0 + 12.0 * t) * frames;
                if closing {
                    // Closed, the scene change follows.
                    if txt + step >= CIRCLE_SCROLL {
                        Phase::Covered(0.0)
                    } else {
                        Phase::Circle {
                            txt: txt + step,
                            closing,
                            right,
                        }
                    }
                } else if txt - step <= 0.0 {
                    Phase::Clear
                } else {
                    Phase::Circle {
                        txt: txt - step,
                        closing,
                        right,
                    }
                }
            }
        };
    }
}

#[derive(Component)]
struct WipeCover;

/// Black beside the 4:3 circle wipe on a wider window.
#[derive(Component)]
struct WipeBar;

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
    for right in [false, true] {
        let mut node = Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            height: Val::Percent(100.0),
            width: Val::Px(0.0),
            ..default()
        };
        if right {
            node.right = Val::Px(0.0);
        } else {
            node.left = Val::Px(0.0);
        }
        commands.spawn((
            WipeBar,
            node,
            BackgroundColor(Color::BLACK),
            GlobalZIndex(i32::MAX - 1),
            Visibility::Hidden,
        ));
    }
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
    windows: Query<&Window>,
    mut cover: Query<(&mut BackgroundColor, &mut Visibility), (With<WipeCover>, Without<WipeBar>)>,
    mut bars: Query<(&mut Node, &mut BackgroundColor, &mut Visibility), With<WipeBar>>,
) {
    wipe.advance(time.delta_secs() * 60.0);
    // The circle is a 4:3 picture: the sides of a wider window stay black until it has nearly
    // opened, then fade away.
    let side = match (wipe.phase, windows.single()) {
        (Phase::Circle { txt, .. }, Ok(window)) => {
            let aspect = window.width() / window.height().max(1.0);
            let width = ((1.0 - (4.0 / 3.0) / aspect) / 2.0).max(0.0) * 100.0;
            Some((width, (txt / (CIRCLE_SCROLL * 0.35)).min(1.0)))
        }
        _ => None,
    };
    for (mut node, mut color, mut visibility) in &mut bars {
        match side {
            Some((width, alpha)) => {
                node.width = Val::Percent(width);
                color.0 = Color::srgba(0.0, 0.0, 0.0, alpha);
                *visibility = Visibility::Inherited;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
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

/// The circle wipe's model and entities, built when first needed.
#[derive(Default)]
struct CircleState {
    /// (mesh, material, texels across the wipe texture, the mesh's unscrolled UVs).
    parts: Vec<(Handle<Mesh>, Handle<N64Material>, f32, Vec<[f32; 2]>)>,
    camera: Option<Entity>,
    entity: Option<Entity>,
    /// Which model the parts are (`ef_wipe3_modelT` when right).
    right: bool,
}

/// `fbdemo_triforce_draw`: `ef_wipe2_modelT` (circle left) under a 60 degree perspective camera
/// 400 units out, its texture's visible window scrolled by `txt` (`gfx_tex_scroll2`).
#[allow(clippy::too_many_arguments)]
fn circle(
    mut commands: Commands,
    wipe: Res<SceneWipe>,
    disc: Option<Res<GameDisc>>,
    mut state: Local<CircleState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Phase::Circle { txt, right, .. } = wipe.phase else {
        if let Some(entity) = state.entity.take() {
            commands.entity(entity).despawn();
        }
        if let Some(entity) = state.camera.take() {
            commands.entity(entity).despawn();
        }
        return;
    };
    let Some(disc) = disc else { return };
    if state.right != right {
        // The other side's model.
        state.parts.clear();
        state.right = right;
        if let Some(entity) = state.entity.take() {
            commands.entity(entity).despawn();
        }
    }
    if state.parts.is_empty() {
        let Some(list) = disc
            .catalog
            .get(["ef_wipe2_modelT", "ef_wipe3_modelT"][usize::from(right)])
        else {
            return;
        };
        let sink = crate::disc_gfx::run_display_lists(&disc.rom, &[list.offset]);
        let model = crate::disc_town::sink_meshes_scaled(
            &disc,
            &sink,
            None,
            true,
            &crate::disc_gfx::Segments::default(),
            1.0,
            (&mut meshes, &mut materials, &mut images),
        );
        for (part, (mesh, material)) in sink.parts.iter().zip(model) {
            // Keep the geometry in the main world: its texture coordinates are rewritten every
            // frame.
            let Some(mut copy) = meshes.get(&mesh).cloned() else {
                continue;
            };
            copy.asset_usage = bevy::asset::RenderAssetUsages::default();
            let uvs = match copy.attribute(Mesh::ATTRIBUTE_UV_0) {
                Some(bevy::mesh::VertexAttributeValues::Float32x2(v)) => v.clone(),
                _ => Vec::new(),
            };
            let _ = meshes.insert(mesh.id(), copy);
            let width = part.key.tex.as_ref().map_or(16.0, |k| f32::from(k.width));
            state.parts.push((mesh, material, width, uvs));
        }
    }
    if state.camera.is_none() {
        state.camera = Some(
            commands
                .spawn((
                    Name::new("circle wipe camera"),
                    Camera3d::default(),
                    Camera {
                        order: 20,
                        clear_color: ClearColorConfig::None,
                        ..default()
                    },
                    bevy::core_pipeline::tonemapping::Tonemapping::None,
                    Projection::Perspective(PerspectiveProjection {
                        fov: 60f32.to_radians(),
                        near: 10.0,
                        far: 12800.0,
                        ..default()
                    }),
                    Transform::from_xyz(0.0, 0.0, CIRCLE_CAMERA_Z),
                    RenderLayers::layer(CIRCLE_LAYER),
                ))
                .id(),
        );
    }
    if state.entity.is_none() {
        let root = commands
            .spawn((
                Name::new("circle wipe"),
                Transform::from_scale(Vec3::splat(CIRCLE_SCALE)),
                Visibility::default(),
                RenderLayers::layer(CIRCLE_LAYER),
            ))
            .id();
        for (mesh, material, _, _) in &state.parts {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                RenderLayers::layer(CIRCLE_LAYER),
                ChildOf(root),
            ));
        }
        state.entity = Some(root);
    }
    // The texture window starts `txt` quarter texels in: the picture shifts the other way.
    for (mesh, _, width, base) in &state.parts {
        if let Some(mut mesh) = meshes.get_mut(mesh) {
            let shift = txt / 4.0 / width;
            let uvs: Vec<[f32; 2]> = base.iter().map(|[u, v]| [u - shift, *v]).collect();
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        }
    }
}

/// Where the player stood and faced at the start of the frame.
#[derive(Resource, Default)]
struct Before(Option<(Vec3, Vec3)>);

/// A door's scene change held back until the screen is black.
#[derive(Resource, Default)]
struct HeldDoor {
    placed: Option<(Vec3, Vec3)>,
    shop: Option<ShopLocation>,
    house: Option<HouseState>,
    waiting: bool,
}

fn remember(
    wipe: Res<SceneWipe>,
    held: Res<HeldDoor>,
    player: Query<(&Transform, &Facing), With<Player>>,
    mut before: ResMut<Before>,
) {
    before.0 = match player.single() {
        Ok((transform, facing)) if wipe.clear() && !held.waiting => {
            Some((transform.translation, facing.dir))
        }
        _ => None,
    };
}

/// While the screen fades out and waits, nothing takes input.
fn lock(wipe: Res<SceneWipe>, held: Res<HeldDoor>, mut focus: ResMut<InputFocus>) {
    // Only a door's own fade: the job's send-away drives its fade itself.
    if wipe.locking() && (held.waiting || wipe.holding) {
        *focus = InputFocus::Transition;
    }
}

/// `FADE_TYPE_OUT` at an ordinary door: the door systems change the scene the moment the player
/// walks in; this puts the player back, holds the change, fades the picture out and applies
/// the change once it is black (the arrival then fades in).
#[allow(clippy::too_many_arguments)]
fn doors(
    keys: Res<ButtonInput<KeyCode>>,
    before: Res<Before>,
    mut held: ResMut<HeldDoor>,
    mut wipe: ResMut<SceneWipe>,
    shop: Option<ResMut<NextState<ShopLocation>>>,
    house: Option<ResMut<NextState<HouseState>>>,
    mut player: Query<(&mut Transform, &mut Facing), With<Player>>,
) {
    let (Some(mut shop), Some(mut house)) = (shop, house) else {
        return;
    };
    let Ok((mut transform, mut facing)) = player.single_mut() else {
        return;
    };
    if held.waiting {
        // The change goes through once the screen is black.
        if wipe.black() {
            if let Some((at, dir)) = held.placed.take() {
                transform.translation = at;
                facing.dir = dir;
            }
            if let Some(next) = held.shop.take() {
                shop.set(next);
            }
            if let Some(next) = held.house.take() {
                house.set(next);
            }
            held.waiting = false;
        } else {
            // A door system fired again on the frozen player: drop its change.
            *shop = NextState::Unchanged;
            *house = NextState::Unchanged;
        }
        return;
    }
    let pending_shop = match &*shop {
        NextState::Pending(next) => Some(*next),
        _ => None,
    };
    let pending_house = match &*house {
        NextState::Pending(next) => Some(*next),
        _ => None,
    };
    if pending_shop.is_none() && pending_house.is_none() {
        return;
    }
    // A quick-load, or a change made while the screen is already fading, goes straight through.
    let Some((at, dir)) = before
        .0
        .filter(|_| wipe.clear() && !keys.just_pressed(KeyCode::F9))
    else {
        return;
    };
    held.placed = Some((transform.translation, facing.dir));
    held.shop = pending_shop;
    held.house = pending_house;
    held.waiting = true;
    *shop = NextState::Unchanged;
    *house = NextState::Unchanged;
    transform.translation = at;
    facing.dir = dir;
    wipe.fade_out();
}

pub struct SceneWipePlugin;

impl Plugin for SceneWipePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneWipe>()
            .add_systems(Startup, setup)
            .init_resource::<Before>()
            .init_resource::<HeldDoor>()
            .add_systems(
                PreUpdate,
                (remember, lock).chain().after(crate::input::update_focus),
            )
            .add_systems(Update, (arrive, run, circle).chain())
            .add_systems(Last, doors);
    }
}
