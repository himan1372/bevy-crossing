//! The house's other floors: upstairs (LL2) and the basement.
//!
//! Each floor keeps its own furniture layers, switches and wallpaper/carpet
//! (`mHm_hs_c` floors[3]); the diary and music box belong to the house. The
//! stairs (obj_myhome_step_down/up, aMI_DrawMyStep) are cut into the south
//! wall beside the entrance; walking south into one changes floor, as the
//! door items in those units do in the original (goto_next_scene). Only the
//! main floor has the front door.
use bevy::prelude::*;

use super::{PlayerRoom, RoomSave};
use crate::disc_gfx::{run_display_lists, Segments};
use crate::game_disc::GameDisc;
use crate::home::{self, PlayerHouse};
use crate::house::{CurrentRoom, HouseState, INTERIOR_ORIGIN};
use crate::input::{Action, Actions, InputFocus};
use crate::interaction::Facing;
use crate::n64_material::N64Material;
use crate::player::Player;

/// The floors not on show, by floor number (the shown one lives in
/// [`PlayerRoom`]).
#[derive(Resource, Default)]
pub(crate) struct HomeFloors {
    pub(crate) saved: [Option<RoomSave>; 3],
}

impl HomeFloors {
    /// (main floor, [upstairs, basement]) with the shown floor current.
    pub(crate) fn snapshot(&self, room: &PlayerRoom) -> (Option<RoomSave>, [Option<RoomSave>; 2]) {
        let mut all = self.saved.clone();
        all[home::floor() as usize] = room.snapshot();
        let [main, upper, basement] = all;
        (main, [upper, basement])
    }

    /// After a load: the main floor is shown, the others are kept.
    pub(crate) fn restore(&mut self, extra: Option<&[Option<RoomSave>; 2]>) {
        let [upper, basement] = extra.cloned().unwrap_or_default();
        self.saved = [None, upper, basement];
        home::set_floor(home::MAIN_FLOOR);
    }
}

#[derive(Component)]
struct StairModel;

/// What the spawned stairs were built for (floor, size, basement).
#[derive(Default)]
struct Built(Option<(u8, u8, bool)>);

fn sync_stairs(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    house: Option<Res<PlayerHouse>>,
    stairs: Query<Entity, With<StairModel>>,
    mut built: Local<Built>,
    (mut meshes, mut materials, mut images): (
        ResMut<Assets<Mesh>>,
        ResMut<Assets<N64Material>>,
        ResMut<Assets<Image>>,
    ),
) {
    let (Some(disc), Some(house)) = (disc, house) else {
        return;
    };
    let key = (home::floor(), house.0.size, house.0.has_basement);
    if built.0 == Some(key) {
        return;
    }
    built.0 = Some(key);
    for e in &stairs {
        commands.entity(e).despawn();
    }
    let pos = home::stair_pos() / 40.0 - home::room_centre();
    for stair in home::stairs(&house.0) {
        let Some(list) = disc.catalog.get(stair.model).map(|a| a.offset) else {
            continue;
        };
        let sx = if stair.mirror { -1.0 } else { 1.0 };
        let root = commands
            .spawn((
                Name::new(stair.model),
                StairModel,
                Transform::from_translation(INTERIOR_ORIGIN + pos)
                    .with_scale(Vec3::new(sx, 1.0, 1.0) * (0.01 / 40.0)),
                Visibility::default(),
            ))
            .id();
        let sink = run_display_lists(&disc.rom, &[list]);
        for (mesh, material) in crate::disc_town::sink_meshes_scaled(
            &disc,
            &sink,
            None,
            false,
            &Segments::default(),
            1.0,
            (&mut meshes, &mut materials, &mut images),
        ) {
            commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), ChildOf(root)));
        }
    }
}

/// An empty floor with the house's starting wallpaper and carpet.
fn empty_floor(template: &RoomSave, disc: &GameDisc) -> RoomSave {
    RoomSave {
        layers: std::array::from_fn(|_| vec![0; 256]),
        switches: [0; 4],
        music_box: template.music_box,
        decoration: super::decorating::Decoration::defaults(disc),
        diary: template.diary.clone(),
    }
}

#[allow(clippy::too_many_arguments)]
fn climb(
    mut commands: Commands,
    actions: Res<Actions>,
    focus: Res<InputFocus>,
    state: Res<State<HouseState>>,
    current: Res<CurrentRoom>,
    disc: Option<Res<GameDisc>>,
    house: Option<Res<PlayerHouse>>,
    mut room: ResMut<PlayerRoom>,
    mut floors: ResMut<HomeFloors>,
    mut player: Query<(&mut Transform, &mut Facing), With<Player>>,
    mut held: Local<bool>,
) {
    let (Some(disc), Some(house)) = (disc, house) else {
        return;
    };
    // After a change of floor, the stairs wait for the key to be let go.
    if !actions.pressed(Action::MoveDown) {
        *held = false;
    }
    if *held
        || !crate::house::is_player_room(&current, &state)
        || !focus.is_world()
        || !actions.pressed(Action::MoveDown)
    {
        return;
    }
    let Ok((mut transform, mut facing)) = player.single_mut() else {
        return;
    };
    let local = transform.translation - INTERIOR_ORIGIN;
    let wall = home::room_half();
    if local.z < wall - 0.5 || facing.dir.z < 0.7 {
        return;
    }
    let Some(stair) = home::stairs(&house.0).into_iter().find(|s| {
        let (a, b) = s.span();
        (a..=b).contains(&local.x)
    }) else {
        return;
    };
    let from = home::floor();
    let Some(shown) = room.snapshot() else {
        return;
    };
    floors.saved[from as usize] = Some(shown.clone());
    let mut next = floors.saved[stair.to as usize]
        .clone()
        .unwrap_or_else(|| empty_floor(&shown, &disc));
    // The diary and music box belong to the house, not a floor.
    next.diary = shown.diary.clone();
    next.music_box = shown.music_box;
    home::set_floor(stair.to);
    *held = true;
    room.restore(Some(&next), Some(&disc));
    crate::house::refresh_player_room_bounds(&mut commands);
    // Arrive just north of the stairs leading back.
    let back = home::stairs(&house.0)
        .into_iter()
        .find(|s| s.to == from)
        .map_or(0.0, |s| {
            let (a, b) = s.span();
            (a + b) / 2.0
        });
    transform.translation = INTERIOR_ORIGIN + Vec3::new(back, 1.0, home::room_half() - 1.0);
    facing.dir = Vec3::NEG_Z;
    crate::sound::door_sound(None, true);
    info!("house floor {from} -> {}", stair.to);
}

pub(super) struct FloorsPlugin;

impl Plugin for FloorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HomeFloors>().add_systems(
            Update,
            (sync_stairs, climb.in_set(crate::states::GameplaySet)),
        );
    }
}
