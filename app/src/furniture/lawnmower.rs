//! ac_nog_lawnmower: cut grass under the mower on the lawn carpets.
//!
//! fNLM_mv eases the grass level toward 1 while the mower is pushed (or
//! waiting to be pushed) from its back on FLOOR_NPC026/048, else toward 0;
//! fNLM_dw draws the grass list with primitive (120, 255, 180, level) and
//! scrolls its second tile by `-10 * frame` (fFTR_GetTwoTileGfx).
use super::*;
use crate::house::{CurrentRoom, HouseState};
use crate::player_room::{PlayerRoom, RoomPiece};
use rustimal_logic::furniture_lawnmower as native;

pub(crate) fn is_lawnmower(index: u16) -> bool {
    index == native::LAWNMOWER
}

fn lists(disc: &GameDisc) -> Option<(u32, u32)> {
    Some((
        disc.catalog.get("int_nog_lawnmower_bodyT_model")?.offset,
        disc.catalog.get("int_nog_lawnmower_grass_model")?.offset,
    ))
}

pub(super) fn available(disc: &GameDisc, _profile: &Profile) -> bool {
    lists(disc).is_some()
}

#[derive(Component)]
pub(crate) struct MowerRig {
    grass: Entity,
    level: f32,
    ticks: f32,
}

pub(super) fn spawn(
    commands: &mut Commands,
    disc: &GameDisc,
    root: Entity,
    profile: &Profile,
    (meshes, materials, images): (
        &mut Assets<Mesh>,
        &mut Assets<N64Material>,
        &mut Assets<Image>,
    ),
) {
    let Some((body, grass_list)) = lists(disc) else {
        warn!("incomplete lawnmower assets");
        return;
    };
    let segments = Segments::default();
    let body = run_display_lists(&disc.rom, &[body]);
    crate::interior::spawn_sink(
        commands,
        disc,
        root,
        &body,
        profile.palette,
        &segments,
        false,
        meshes,
        materials,
        images,
    );
    let grass = commands
        .spawn((
            Name::new("lawnmower grass"),
            Transform::default(),
            Visibility::default(),
            ChildOf(root),
        ))
        .id();
    let sink = run_display_lists(&disc.rom, &[grass_list]);
    crate::interior::spawn_sink(
        commands,
        disc,
        grass,
        &sink,
        profile.palette,
        &segments,
        true,
        meshes,
        materials,
        images,
    );
    commands.entity(root).insert(MowerRig {
        grass,
        level: 0.0,
        ticks: 0.0,
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update(
    time: Res<Time>,
    house: Res<State<HouseState>>,
    current: Res<CurrentRoom>,
    room: Res<PlayerRoom>,
    mut rigs: Query<(&mut MowerRig, &RoomPiece)>,
    children: Query<&Children>,
    handles: Query<&MeshMaterial3d<N64Material>>,
    mut materials: ResMut<Assets<N64Material>>,
    images: Res<Assets<Image>>,
) {
    // fFTR_GetTwoTileGfx(16, 16, 0, 0, 16, 16, 0, 10, ..): ofs = game frame.
    let frame = (time.elapsed_secs_f64() * 60.0) as i64 as i32;
    let shown = crate::house::is_player_room(&current, &house);
    for (mut rig, piece) in &mut rigs {
        if shown {
            let cutting = room.mower_cutting(piece);
            rig.ticks += time.delta_secs() * 60.0;
            let ticks = (rig.ticks as u32).min(4);
            rig.ticks = (rig.ticks - ticks as f32).min(1.0);
            for _ in 0..ticks {
                native::grass_step(&mut rig.level, cutting);
            }
        } else {
            // fNLM_ct on the next visit.
            rig.level = 0.0;
            rig.ticks = 0.0;
        }
        let alpha = native::grass_alpha(rig.level);
        let [r, g, b] = native::GRASS_PRIM;
        let prim = Vec4::new(r as f32, g as f32, b as f32, f32::from(alpha)) / 255.0;
        for entity in children.iter_descendants(rig.grass) {
            if let Ok(handle) = handles.get(entity) {
                if let Some(mut m) = materials.get_mut(handle.id()) {
                    m.combiner.prim = prim;
                    let y = frame.wrapping_mul(10).wrapping_neg();
                    crate::n64_material::set_tile1_scroll(&mut m, &images, 0, y);
                }
            }
        }
    }
}
