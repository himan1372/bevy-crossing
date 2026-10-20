//! ac_nog_sprinkler: E toggles the water with a fade, its sounds and the
//! running level sound.
//!
//! E flips the actor's switch bit through the generic silent branch; the
//! callback only acts on that change on a frame when the water fade has
//! settled (`switch_changed_flag` is one frame long), so a press mid-fade
//! leaves the local state alone. Leaving the room runs fNS_dt, writing the
//! local state back to the switch bit. The water list reads its environment
//! colour from the draw code: (0, 180, 255, fade alpha); its second tile
//! scrolls by `-counter * 10` (two_tex_scroll_dolphin), giving the flow.
use super::*;
use crate::house::{CurrentRoom, HouseState};
use crate::player_room::{PlayerRoom, RoomPiece};
use rustimal_logic::furniture_sprinkler::{self as native, Sprinkler};

pub(crate) fn is_sprinkler(index: u16) -> bool {
    index == 0x4DA
}

fn lists(disc: &GameDisc) -> Option<(u32, u32)> {
    Some((
        disc.catalog.get("int_nog_sprinkler_bodyT_model")?.offset,
        disc.catalog.get("int_nog_sprinkler_water_model")?.offset,
    ))
}

pub(super) fn available(disc: &GameDisc, _profile: &Profile) -> bool {
    lists(disc).is_some()
}

#[derive(Component)]
pub(crate) struct SprinklerRig {
    water: Entity,
    state: Option<Sprinkler>,
    switch: Option<u8>,
    ticks: f32,
    pub(crate) sound_on: bool,
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
    let Some((body, water_list)) = lists(disc) else {
        warn!("incomplete sprinkler assets");
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
    let water = commands
        .spawn((
            Name::new("sprinkler water"),
            Transform::default(),
            Visibility::default(),
            ChildOf(root),
        ))
        .id();
    let sink = run_display_lists(&disc.rom, &[water_list]);
    crate::interior::spawn_sink(
        commands,
        disc,
        water,
        &sink,
        profile.palette,
        &segments,
        true,
        meshes,
        materials,
        images,
    );
    commands.entity(root).insert(SprinklerRig {
        water,
        state: None,
        switch: None,
        ticks: 0.0,
        sound_on: false,
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update(
    time: Res<Time>,
    house: Res<State<HouseState>>,
    current: Res<CurrentRoom>,
    mut room: ResMut<PlayerRoom>,
    mut rigs: Query<(&mut SprinklerRig, &GlobalTransform, &RoomPiece)>,
    children: Query<&Children>,
    handles: Query<&MeshMaterial3d<N64Material>>,
    mut materials: ResMut<Assets<N64Material>>,
    images: Res<Assets<Image>>,
) {
    let shown = crate::house::is_player_room(&current, &house);
    for (mut rig, at, piece) in &mut rigs {
        let switch = room.switch_bit(piece);
        if !shown {
            // fNS_dt: the local state becomes the switch bit.
            if let (Some(state), Some(bit)) = (rig.state.take(), switch) {
                if (bit != 0) != state.on {
                    room.set_switch(piece, state.on);
                }
            }
            rig.switch = None;
            rig.ticks = 0.0;
            rig.sound_on = false;
            continue;
        }
        let Some(bit) = switch else { continue };
        let mut state = rig.state.unwrap_or_else(|| Sprinkler::new(bit != 0));
        if rig.state.is_none() {
            rig.switch = Some(bit);
        }
        rig.ticks += time.delta_secs() * 60.0;
        let ticks = (rig.ticks as u32).min(4);
        rig.ticks = (rig.ticks - ticks as f32).min(1.0);
        for _ in 0..ticks {
            let switch_changed = rig.switch.is_some_and(|last| last != bit);
            rig.switch = Some(bit);
            if let Some(sound) = state.tick(switch_changed) {
                if room.clock_sound_allowed(piece) {
                    crate::sound::se_at(sound, at.translation());
                }
            }
        }
        rig.state = Some(state);
        rig.sound_on = state.on && room.clock_sound_allowed(piece);
        let alpha = state.water_alpha();
        let [r, g, b] = native::WATER_ENV;
        let env = Vec4::new(r as f32, g as f32, b as f32, f32::from(alpha)) / 255.0;
        // fNS_dw: fNS_GetTwoTileGfx(0, 0, 0, -counter * 10).
        let scroll = state.tile1_scroll();
        for entity in children.iter_descendants(rig.water) {
            if let Ok(handle) = handles.get(entity) {
                if let Some(mut m) = materials.get_mut(handle.id()) {
                    m.combiner.env = env;
                    crate::n64_material::set_tile1_scroll(&mut m, &images, 0, scroll);
                }
            }
        }
    }
}
