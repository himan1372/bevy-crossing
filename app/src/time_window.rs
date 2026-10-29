//! The clock window of `m_timeIn_ovl.c` (`mTI_OVL_OPEN_INITIAL`, "Adjust the clock."): hour, minute,
//! month, day and year, a clock whose hands sweep round to the new time, and OK. It slides in from the
//! top like the other submenus; closing it sets the game clock (`lbRTC_SetTime`).
use bevy::camera::visibility::RenderLayers;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use rustimal_logic::menu_slide::Slide;
use rustimal_logic::time_menu as tm;

use crate::game_disc::GameDisc;
use crate::msg_window::MsgAssets;
use crate::n64_material::N64Material;
use crate::pockets_disc::Paint;

/// `mTI_col_type0` (the field being edited) and `mTI_col_type1`.
const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [60, 90, 255, 255];

// `sAdo_SysTrgStart` ids from `m_timeIn_ovl.c`.
const NEXT: u16 = 0xD;
const CHANGE: u16 = 0x1;
const BACK: u16 = 0x1003;
const ARRIVED: u16 = 0x41B;

/// Frames a held up / down key waits before it repeats, and between repeats (`mSM_make_trigger_data`).
const REPEAT_FIRST: f32 = 25.0;
const REPEAT_EVERY: f32 = 3.0;

struct View {
    values: tm::Values,
    sel: usize,
    sweep: tm::Sweep,
    /// The hands are sweeping: input waits (`input_disabled_flag`).
    sweeping: bool,
    sweep_ticks: f32,
    slide: Slide,
    accepted: Option<tm::Values>,
    just_opened: bool,
    revision: u64,
    held: Option<(KeyCode, f32)>,
}

/// The window while it is up.
#[derive(Resource, Default)]
pub(crate) struct TimeWindow {
    view: Option<View>,
    finished: Option<tm::Values>,
}

impl TimeWindow {
    pub(crate) fn open(&self) -> bool {
        self.view.is_some()
    }

    /// `mTI_timeIn_ovl_init`: the window starts at the game's current time.
    pub(crate) fn begin(&mut self, now: crate::clock::Now) {
        let values = tm::Values::new(now.year, now.month, now.day, now.hour, now.minute);
        self.finished = None;
        self.view = Some(View {
            sweep: tm::Sweep::new(values.dial_minutes()),
            values,
            sel: tm::HOUR,
            sweeping: false,
            sweep_ticks: 0.0,
            slide: Slide::new(),
            accepted: None,
            just_opened: true,
            revision: 0,
            held: None,
        });
    }

    /// The time the player settled on, once the window has left.
    pub(crate) fn take(&mut self) -> Option<tm::Values> {
        self.finished.take()
    }
}

fn input(
    mut window: ResMut<TimeWindow>,
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut focus: ResMut<crate::input::InputFocus>,
    time: Res<Time>,
) {
    let Some(mut view) = window.view.take() else {
        events.clear();
        return;
    };
    let frames = time.delta_secs() * 60.0;
    let gone = view.slide.advance(frames);
    if let Some(values) = view.accepted {
        events.clear();
        if gone {
            window.finished = Some(values);
            *focus = crate::input::InputFocus::Dialogue;
        } else {
            window.view = Some(view);
        }
        return;
    }
    if *focus != crate::input::InputFocus::TimeWindow
        || std::mem::take(&mut view.just_opened)
        || !view.slide.settled()
    {
        events.clear();
        window.view = Some(view);
        return;
    }
    let before = (view.values, view.sel, view.sweep.minutes, view.sweeping);
    if view.sweeping {
        // `mTI_set_time`: the hands go round, twelve minutes of dial a tick.
        events.clear();
        view.sweep_ticks += frames;
        while view.sweep_ticks >= 1.0 && view.sweeping {
            view.sweep_ticks -= 1.0;
            if view.sweep.tick(view.values.dial_minutes()) {
                view.sweeping = false;
                crate::sound::se(ARRIVED);
            }
        }
    } else {
        let mut presses: Vec<KeyCode> = events
            .read()
            .filter(|e| e.state == ButtonState::Pressed)
            .map(|e| e.key_code)
            .collect();
        // A held up / down key repeats.
        match view.held {
            Some((key, _)) if !keys.pressed(key) => view.held = None,
            Some((key, left)) => {
                let left = left - frames;
                if left <= 0.0 {
                    presses.push(key);
                    view.held = Some((key, REPEAT_EVERY));
                } else {
                    view.held = Some((key, left));
                }
            }
            None => {}
        }
        for key in presses {
            match key {
                KeyCode::F4 => close(&mut view),
                KeyCode::Backspace | KeyCode::ArrowLeft => {
                    if view.sel > 0 {
                        crate::sound::se(BACK);
                        view.sel -= 1;
                    }
                }
                KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space | KeyCode::ArrowRight => {
                    if view.sel == tm::OK {
                        if key != KeyCode::ArrowRight {
                            close(&mut view);
                        }
                    } else {
                        view.sel += 1;
                        if view.sel == tm::OK && view.sweep.minutes != view.values.dial_minutes() {
                            view.sweeping = true;
                            view.sweep.restart();
                            view.sweep_ticks = 0.0;
                        }
                        crate::sound::se(NEXT);
                    }
                }
                KeyCode::ArrowUp | KeyCode::ArrowDown if view.sel != tm::OK => {
                    let sel = view.sel;
                    view.values.change(sel, key == KeyCode::ArrowUp);
                    crate::sound::se(CHANGE);
                    if view.held.is_none_or(|(held, _)| held != key) {
                        view.held = Some((key, REPEAT_FIRST));
                    }
                }
                _ => {}
            }
        }
    }
    if before != (view.values, view.sel, view.sweep.minutes, view.sweeping) {
        view.revision = view.revision.wrapping_add(1);
    }
    window.view = Some(view);
}

/// `mTI_window_close`.
fn close(view: &mut View) {
    crate::sound::se(crate::sound::se_id::MENU_EXIT);
    view.accepted = Some(view.values);
    view.slide.close();
}

type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;

/// `mSM_menu_control_move`: the window's cloth pattern creeps 0.3535 texels a tick
/// (`gDPSetTileSize_Dolphin(RENDERTILE, -texture_pos * k, ..)`; `k` quarter texels per unit, so the
/// cloth moves `k / 4` texels per unit). Only the cloth pieces (the ones with a second tile) slide.
pub(crate) fn scroll_cloth(
    model: &Model,
    slide: f32,
    n64: &mut Assets<N64Material>,
    images: &Assets<Image>,
) {
    for (_, handle) in model {
        let Some(mut material) = n64.get_mut(handle) else {
            continue;
        };
        if material.texture1.is_none() {
            continue;
        }
        let Some(size) = material
            .texture
            .as_ref()
            .and_then(|h| images.get(h))
            .map(|image| image.size_f32())
        else {
            continue;
        };
        material.combiner.extra.y = (slide / size.x.max(1.0)).fract();
        material.combiner.extra.z = (slide / size.y.max(1.0)).fract();
    }
}

fn creep(
    time: Res<Time>,
    mut assets: ResMut<TimeAssets>,
    mut n64: ResMut<Assets<N64Material>>,
    images: Res<Assets<Image>>,
) {
    assets.slide = (assets.slide + 0.3535 * time.delta_secs() * 60.0).rem_euclid(1024.0);
    if let Some(frame) = assets.frame.as_ref() {
        scroll_cloth(frame, assets.slide, &mut n64, &images);
    }
}

#[derive(Resource, Default)]
pub(crate) struct TimeAssets {
    slide: f32,
    frame: Option<Model>,
    hands: Option<[Model; 3]>,
    root: Option<Entity>,
    shown: Option<u64>,
}

#[derive(Component, Clone, Copy)]
enum Hand {
    Minute,
    Hour,
    Cap,
}

const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];
const WEEKDAYS: [&str; 7] = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];

/// A window text model (`tim_win_*_model`) with the texture its draw sets in segment 8.
fn texture_model(
    disc: &GameDisc,
    list: &str,
    texture: &str,
    prim: [u8; 4],
    (meshes, n64, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) -> Model {
    let lists: Vec<u32> = ["tim_win_mode", list]
        .iter()
        .filter_map(|n| disc.catalog.get(n).map(|a| a.offset))
        .collect();
    let mut segments = crate::disc_gfx::Segments::default();
    if let Some(tex) = disc.catalog.get(&format!("tim_win_{texture}_tex_rgb_i4")) {
        if let Some(bytes) = disc.rom.slice(disc::RomSource::Rel, tex.offset, tex.size) {
            segments.set_segment(8, bytes.into());
        }
    }
    let sink = crate::disc_gfx::run_display_lists_with_prim(&disc.rom, &lists, prim, 255);
    crate::disc_town::sink_meshes_scaled(
        disc,
        &sink,
        None,
        false,
        &segments,
        16.0,
        (meshes, n64, images),
    )
}

#[allow(clippy::too_many_arguments)]
fn draw(
    mut commands: Commands,
    window: Res<TimeWindow>,
    disc: Option<Res<GameDisc>>,
    font: Option<Res<MsgAssets>>,
    mut assets: ResMut<TimeAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut n64: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(view) = window.view.as_ref() else {
        if let Some(root) = assets.root.take() {
            commands.entity(root).despawn();
        }
        assets.shown = None;
        return;
    };
    let (Some(disc), Some(font)) = (disc.as_deref(), font.as_deref()) else {
        return;
    };
    if assets.shown == Some(view.revision) && assets.root.is_some() {
        return;
    }
    let mut build = |names: &[&str]| -> Model {
        let lists: Vec<u32> = names
            .iter()
            .filter_map(|n| disc.catalog.get(n).map(|a| a.offset))
            .collect();
        let sink = crate::disc_gfx::run_display_lists(&disc.rom, &lists);
        crate::disc_town::sink_meshes_scaled(
            disc,
            &sink,
            None,
            true,
            &crate::disc_gfx::Segments::default(),
            16.0,
            (&mut meshes, &mut n64, &mut images),
        )
    };
    if assets.frame.is_none() {
        assets.frame = Some(build(&["tim_win_mode", "tim_win_model"]));
        assets.hands = Some([
            build(&["tim_hari_mode", "tim_hari_b2T_model"]),
            build(&["tim_hari_mode", "tim_hari_b1T_model"]),
            build(&["tim_hari_mode", "tim_hari_b3T_model"]),
        ]);
    }
    if let Some(root) = assets.root.take() {
        commands.entity(root).despawn();
    }
    let root = commands
        .spawn((
            Name::new("time window"),
            WindowRoot,
            Transform::from_xyz(0.0, view.slide.pos * 16.0, 0.0),
            Visibility::default(),
        ))
        .id();
    assets.root = Some(root);
    assets.shown = Some(view.revision);
    let spawn_model =
        |commands: &mut Commands, model: &Model, z: f32, parent: Entity, hand: Option<Hand>| {
            let at = match hand {
                // `mTI_set_clock`: the dial's centre, (x - 61, y + 45) less the model's (5, 4).
                Some(_) => Vec3::new(-66.0 * 16.0, 41.0 * 16.0, z),
                None => Vec3::new(0.0, 0.0, z),
            };
            let holder = commands
                .spawn((
                    Transform::from_translation(at),
                    Visibility::default(),
                    ChildOf(parent),
                ))
                .id();
            if let Some(hand) = hand {
                commands.entity(holder).insert(hand);
            }
            for (i, (mesh, material)) in model.iter().enumerate() {
                commands.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(0.0, 0.0, i as f32 * 0.1),
                    RenderLayers::layer(1),
                    ChildOf(holder),
                ));
            }
        };
    if let Some(frame) = assets.frame.clone() {
        spawn_model(&mut commands, &frame, 200.0, root, None);
    }
    // The month and the weekday are textures in the window's colour of the moment.
    let prim = if view.sel == tm::MONTH { RED } else { BLUE };
    let month = usize::from(view.values.month().clamp(1, 12)) - 1;
    let month_model = texture_model(
        disc,
        "tim_win_month_model",
        MONTHS[month],
        prim,
        (&mut meshes, &mut n64, &mut images),
    );
    let week_model = texture_model(
        disc,
        "tim_win_youbi_model",
        WEEKDAYS[usize::from(view.values.weekday()) % 7],
        prim,
        (&mut meshes, &mut n64, &mut images),
    );
    spawn_model(&mut commands, &month_model, 203.0, root, None);
    spawn_model(&mut commands, &week_model, 203.0, root, None);
    if let Some(hands) = assets.hands.clone() {
        for (model, hand, z) in [
            (&hands[0], Hand::Minute, 203.5),
            (&hands[1], Hand::Hour, 203.6),
            (&hands[2], Hand::Cap, 203.7),
        ] {
            spawn_model(&mut commands, model, z, root, Some(hand));
        }
    }
    let mut paint = Paint {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        root,
    };
    let scaled = |text: &str| {
        format!(
            "{}{}{}",
            crate::messages::markup::LINE_SCALE,
            crate::messages::markup::arg(28),
            text
        )
    };
    paint.text(
        font,
        &scaled("Adjust the clock."),
        Vec2::new(131.0, 82.0),
        [255, 255, 255, 255],
        230.0,
    );
    for (field, at) in [
        (tm::HOUR, Vec2::new(79.0, 115.0)),
        (tm::MINUTE, Vec2::new(103.0, 115.0)),
        (tm::DAY, Vec2::new(151.0, 141.0)),
        (tm::YEAR, Vec2::new(196.0, 141.0)),
    ] {
        let ink = if view.sel == field { RED } else { BLUE };
        let text = format!("{:02}", view.values.0[field] % 100);
        paint.text(font, &text, at, ink, 230.0);
    }
    let ink = if view.sel == tm::OK { RED } else { BLUE };
    paint.text(font, "OK", Vec2::new(220.0, 141.0), ink, 230.0);
    paint.text(
        font,
        &scaled("Enter: next  Bksp: back  Up/Down: change  F4: done"),
        Vec2::new(40.0, 4.0),
        [30, 0, 0, 255],
        230.0,
    );
}

#[derive(Component)]
struct WindowRoot;

/// Slides the window and turns the hands (`mTI_set_clock`: the minute hand once round an hour, the hour
/// hand once round twelve).
fn animate(
    window: Res<TimeWindow>,
    mut roots: Query<&mut Transform, (With<WindowRoot>, Without<Hand>)>,
    mut hands: Query<(&Hand, &mut Transform), Without<WindowRoot>>,
) {
    let Some(view) = window.view.as_ref() else {
        return;
    };
    for mut transform in &mut roots {
        transform.translation.y = view.slide.pos * 16.0;
    }
    let minutes = f32::from(view.sweep.minutes);
    for (hand, mut transform) in &mut hands {
        let turns = match hand {
            Hand::Minute => (minutes % 60.0) / 60.0,
            Hand::Hour => minutes / 720.0,
            Hand::Cap => 0.0,
        };
        transform.rotation = Quat::from_rotation_z(-turns * std::f32::consts::TAU);
    }
}

pub struct TimeWindowPlugin;

impl Plugin for TimeWindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TimeWindow>()
            .init_resource::<TimeAssets>()
            .add_systems(
                Update,
                input
                    .in_set(crate::input::InteractSet::Ui)
                    .in_set(crate::states::GameplaySet),
            )
            .add_systems(
                Update,
                (draw, creep, animate)
                    .chain()
                    .after(crate::input::InteractSet::Ui)
                    .in_set(crate::states::GameplaySet)
                    .run_if(resource_exists::<MsgAssets>),
            );
    }
}
