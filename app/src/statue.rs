//! ac_douzou: the statue Nook has built by the train station once the
//! player's biggest house is paid off (`next_size == mHm_HOMESIZE_STATUE`).
//!
//! The DOUZOU actor stands on its layout unit; each house's statue is
//! drawn at its own offset (xpostbl/zpostbl). The skeleton holds a pedestal,
//! boy and girl figures (joints 2-4 and 5-7, one shown by the owner's sex)
//! and a name plate; the figure is scaled by rank (1.0 / 0.85 / 0.70 /
//! 0.55 at joint 2) and tinted gold, silver, bronze or jade (primitive and
//! environment colours), with the owner's eyes and mouth in segments 8 and
//! 9 and the pedestal palette (aSTR_PAL_DOUZOU_DAI) in A. The port has no
//! player sex yet and draws the boy, as the player model does; the idle
//! clip is held at its first frame and the sparkle effect is not drawn.
use bevy::prelude::*;

use crate::character::PlayerLook;
use crate::disc_gfx::{run_display_lists_with_colors, Segments};
use crate::game_disc::GameDisc;
use crate::n64_material::N64Material;
use crate::skeleton::{Animation, Skeleton};
use crate::town::Town;

/// aDOU xpostbl / zpostbl, game units from the actor, by house.
const OFFSETS: [(f32, f32); 4] = [(0.0, 200.0), (200.0, 200.0), (-40.0, 280.0), (160.0, 280.0)];
/// aDOU_actor_draw_before, by rank: primitive, environment, figure scale.
const PRIM: [[u8; 3]; 4] = [[255, 255, 0], [200, 200, 255], [255, 100, 50], [0, 255, 40]];
const ENV: [[u8; 3]; 4] = [[100, 30, 0], [80, 80, 100], [40, 20, 0], [0, 30, 30]];
const SCALE: [f32; 4] = [1.0, 0.85, 0.70, 0.55];
/// aSTR_PAL_DOUZOU_DAI.
const PEDESTAL_PALETTE: u32 = 89;
/// Model units (x0.01 game units) to cells.
const MODEL_SCALE: f32 = 0.01 / 40.0;
const BOY: [usize; 3] = [2, 3, 4];
const GIRL: [usize; 3] = [5, 6, 7];

/// The statue's raised unit (aDOU_set_bgOffset): solid to walk into.
#[derive(Resource)]
pub(crate) struct StatueUnit(Vec3);

impl StatueUnit {
    pub(crate) fn blocks(&self, x: f32, z: f32) -> bool {
        // The unit plus the player's reach (radius about 0.35 cells).
        (x - self.0.x).abs() < 0.85 && (z - self.0.z).abs() < 0.85
    }
}

/// Spawn house 0's statue if Nook has built it.
fn spawn(
    mut commands: Commands,
    disc: Option<Res<GameDisc>>,
    town: Res<Town>,
    house: Option<Res<crate::home::PlayerHouse>>,
    look: Option<Res<PlayerLook>>,
    (mut meshes, mut materials, mut images): (
        ResMut<Assets<Mesh>>,
        ResMut<Assets<N64Material>>,
        ResMut<Assets<Image>>,
    ),
) {
    let (Some(disc), Some(house)) = (disc, house) else {
        return;
    };
    if !house.0.statue_built() {
        return;
    }
    let Some((ax, az, cx, cz)) = town.retail.as_ref().and_then(|r| r.statue) else {
        warn!("statue built but the layout has no DOUZOU unit");
        return;
    };
    let winter = crate::clock::now().season == rustimal_logic::game_time::Season::Winter;
    let s = if winter { "w" } else { "s" };
    let list = |name: &str| disc.catalog.get(name).map(|a| a.offset);
    let (Some(bs), Some(ba)) = (
        list(&format!("cKF_bs_r_obj_{s}_douzou")),
        list(&format!("cKF_ba_r_obj_{s}_douzou")),
    ) else {
        warn!("statue skeleton missing from the disc");
        return;
    };
    let Some(skeleton) = Skeleton::read(&disc.rom, bs) else {
        return;
    };
    let Some(animation) = Animation::read(&disc.rom, ba, skeleton.joints.len()) else {
        return;
    };
    let rank = (house.0.statue_rank as usize).min(3);
    let face = look.map_or(0, |l| l.face.min(7));
    // aDOU_get_douzou_type: sex * 8 + face (boy: 0..7).
    let (eye, mouth) = (
        format!("obj_{s}_douzou_b{}_tex_pic_i4", face + 1),
        // mouth_tbl: the boys' mouths by face.
        format!(
            "obj_{s}_douzou_bm{}_tex_pic_i4",
            [2, 1, 1, 2, 1, 1, 1, 2][face]
        ),
    );
    let mut segs = Segments::default();
    for (seg, name) in [(0x08, &eye), (0x09, &mouth)] {
        if let Some(bytes) = disc
            .catalog
            .get(name)
            .and_then(|a| disc.rom.slice(disc::RomSource::Rel, a.offset, a.size))
        {
            segs.set_segment(seg, bytes.into());
        }
    }
    let palette = crate::disc_town::structure_palette(&disc, PEDESTAL_PALETTE, winter);
    if let Some(bytes) = palette.and_then(|p| disc.rom.slice(disc::RomSource::Rel, p, 0x20)) {
        segs.set_segment(0x0A, bytes.into());
    }

    // Pose at the clip's first frame, the figure scaled by rank.
    let mut locals = animation.pose(1.0).local_transforms(&skeleton);
    if let Some(figure) = locals.get_mut(2) {
        figure.scale *= SCALE[rank];
    }
    let world = skeleton.world_from_locals(&locals);

    let (dx, dz) = OFFSETS[0];
    let at = Town::cell_to_world(ax, az, cx, cz) + Vec3::new(dx, 0.0, dz) / 40.0;
    let root = commands
        .spawn((
            Name::new("station statue"),
            Transform::from_translation(at.with_y(town.ground_y(at)))
                .with_scale(Vec3::splat(MODEL_SCALE)),
            Visibility::default(),
        ))
        .id();
    let [r, g, b] = PRIM[rank];
    let [er, eg, eb] = ENV[rank];
    for (j, joint) in skeleton.joints.iter().enumerate() {
        let Some(model) = joint.model else { continue };
        if GIRL.contains(&j) {
            continue;
        }
        // The figure and name plate take the rank's colours.
        let tinted = BOY.contains(&j) || j == 8;
        let (prim, env) = if tinted {
            ([r, g, b, 255], [er, eg, eb, 255])
        } else {
            ([255; 4], [0, 0, 0, 255])
        };
        let mut sink = run_display_lists_with_colors(&disc.rom, &[model], prim, 255, env, &[]);
        let m = world.get(j).copied().unwrap_or(Mat4::IDENTITY);
        for v in sink.parts.iter_mut().flat_map(|p| p.vertices.iter_mut()) {
            v.pos = m.transform_point3(Vec3::from(v.pos)).to_array();
            v.normal = m
                .transform_vector3(Vec3::from(v.normal))
                .normalize_or_zero()
                .to_array();
        }
        for (mesh, material) in crate::disc_town::sink_meshes_scaled(
            &disc,
            &sink,
            palette,
            false,
            &segs,
            1.0,
            (&mut meshes, &mut materials, &mut images),
        ) {
            commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), ChildOf(root)));
        }
    }
    commands.insert_resource(StatueUnit(at));
    info!("station statue: rank {rank}, face {face} at {at:?}");
}

pub struct StatuePlugin;

impl Plugin for StatuePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn.after(crate::town::render_town));
    }
}
