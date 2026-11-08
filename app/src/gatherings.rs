//! The cherry blossom viewing, the harvest moon and the New Year's countdown
//! (`ac_hanami_npc0/1.c`, `ac_tukimi_npc0/1.c`, `ac_countdown_npc0/1.c`, `m_event_map_npc_data.c_inc`).
//!
//! * Cherry blossom viewing (April 5-7): five villagers (`SP_NPC_EV_HANAMI_0..4`) at the shrine acre's units (10,6)
//!   (12,8) (6,10) (4,12) (4,6), the first four sitting, facing 45, -90, -45 and 90 degrees; the fifth wanders. A line is
//!   `base + RANDOM(3) + 3 * idx` (bases `6445 6460 6430 6475 6490 6505` by looks) and the fifth's `6457 6472 6442 6487
//!   6502 6517 + RANDOM(3)`. Two sakura tables stand at (5,11) and (11,7).
//! * Harvest moon (the moon day, 18:00-20:59): four villagers (`SP_NPC_EV_TUKIMI_0..3`) in the pond acre's seven layouts,
//!   moved to the corner of their unit by `def_posX/Z` and facing by `def_angle`. A line is `base + RANDOM(3) + 3 * idx`
//!   with the bases `0x1EB0 0x1EBF 0x1EA1 0x1ECE 0x1EDD 0x1EEC`.
//! * New Year's countdown (December 31, 23:00 to January 1, 00:59): five villagers (`SP_NPC_EV_COUNTDOWN_0..4`) in the
//!   pond acre. The term follows the clock (one hour, 30, 10, 5, one minute, the new year, ten seconds after) and a line
//!   is `base + RANDOM(3) + 4 * term` (`+ 17` at the new year and after), the bases `7528 7549 7507 7570 7591 7612`; the
//!   year (`FREE_STR1`) and the minutes left (`FREE_STR2`) fill its blanks. The host speaks to the player by himself at
//!   the 30, 10, 5 and one minute marks (`base1 + (term - 1) * 4`, `7531 7552 7510 7573 7594 7615`), and fireworks go
//!   off at midnight (`eEC_EFFECT_HANABI_SWITCH`).
//!
//! The cherry viewing's mats and the harvest festival's tables are the disc's `obj_e_hanami_{a,b}` / `obj_e_hfes_{a,b,c}`.
//!
//! Not built: the sitting, toasting and cheering animations, the countdown's sign (a skeleton: a box stands), the
//! moon-viewing table (`obj_e_tukimi_*_endai`), the host's freezing the player at midnight, the visiting towns' names, the sounds.

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::fireworks::{burst, unit_spot, water_acre};
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::Facing;
use crate::player::Player;
use crate::roster::TownRoster;
use crate::shop::ShopLocation;
use crate::states::GameplaySet;
use crate::town::Town;

const UNIT: f32 = 40.0;
/// `aSTR_PAL_COUNT`: the countdown board's palette.
const PAL_COUNT: u32 = 74;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Cherry,
    Moon,
    Count,
    Meteor,
    Harvest,
}

/// The cherry viewing's units and facings (degrees), the fifth wanders.
const CHERRY: [(usize, usize); 5] = [(10, 6), (12, 8), (6, 10), (4, 12), (4, 6)];
const CHERRY_ANGLE: [f32; 4] = [45.0, -90.0, -45.0, 90.0];
const TABLES: [(usize, usize); 2] = [(5, 11), (11, 7)];
const CHERRY_BASE: [u16; 6] = [6445, 6460, 6430, 6475, 6490, 6505];
const CHERRY_BASE5: [u16; 6] = [6457, 6472, 6442, 6487, 6502, 6517];

const MOON: [[(usize, usize); 4]; 7] = [
    [(13, 7), (3, 3), (2, 4), (12, 2)],
    [(3, 8), (12, 13), (13, 12), (13, 2)],
    [(9, 13), (12, 3), (13, 3), (4, 4)],
    [(9, 12), (3, 2), (2, 3), (8, 2)],
    [(9, 2), (12, 6), (13, 7), (2, 2)],
    [(12, 8), (12, 2), (13, 3), (2, 13)],
    [(3, 7), (13, 8), (13, 9), (12, 2)],
];
const MOON_ANGLE: [f32; 7] = [225.0, 45.0, 135.0, 315.0, 315.0, 45.0, 225.0];
const MOON_POS: [(f32, f32); 7] = [
    (1.0, 1.0),
    (39.0, 39.0),
    (39.0, 1.0),
    (1.0, 39.0),
    (1.0, 39.0),
    (39.0, 39.0),
    (1.0, 1.0),
];
const MOON_BASE: [u16; 6] = [0x1EB0, 0x1EBF, 0x1EA1, 0x1ECE, 0x1EDD, 0x1EEC];
/// The meteor shower's lines (`msg_base2`) and layouts (`l_ms_pule*`).
const METEOR_BASE: [u16; 6] = [0x3F46, 0x3F55, 0x3F37, 0x3F64, 0x3F73, 0x3F82];
const METEOR: [[(usize, usize); 4]; 7] = [
    [(13, 7), (3, 3), (2, 4), (12, 2)],
    [(3, 8), (12, 13), (13, 12), (13, 2)],
    [(9, 13), (12, 3), (13, 3), (4, 4)],
    [(9, 12), (3, 2), (2, 3), (8, 2)],
    [(9, 2), (12, 6), (13, 7), (2, 2)],
    [(12, 8), (12, 2), (13, 3), (2, 13)],
    [(3, 7), (13, 8), (13, 9), (12, 2)],
];
/// The harvest festival: units, facings (the fifth stands), tables and the bases of the lines.
const HARVEST: [(usize, usize); 5] = [(10, 6), (12, 8), (5, 4), (3, 6), (4, 12)];
const HARVEST_ANGLE: [f32; 4] = [45.0, 270.0, 315.0, 90.0];
const HARVEST_TABLES: [(usize, usize); 3] = [(4, 5), (11, 7), (5, 10)];
const HARVEST_BASE: [u16; 6] = [15512, 15543, 15419, 15450, 15481, 15574];
const HARVEST_BASE5: [u16; 6] = [0x3CA4, 0x3CC3, 0x3C47, 0x3C66, 0x3C85, 0x3CE2];

type Units5 = [(usize, usize); 5];
type Units2 = [(usize, usize); 2];
/// The countdown's five villagers and its two signs per layout.
const COUNT: [(Units5, Units2); 7] = [
    (
        [(10, 13), (13, 9), (12, 2), (2, 3), (2, 11)],
        [(12, 12), (12, 11)],
    ),
    (
        [(5, 13), (13, 13), (13, 2), (4, 3), (13, 12)],
        [(3, 12), (3, 11)],
    ),
    (
        [(10, 13), (12, 8), (2, 12), (4, 3), (12, 3)],
        [(12, 12), (12, 11)],
    ),
    (
        [(5, 13), (8, 13), (13, 10), (12, 3), (2, 3)],
        [(3, 12), (3, 11)],
    ),
    (
        [(11, 4), (13, 6), (2, 2), (10, 2), (13, 13)],
        [(13, 4), (13, 3)],
    ),
    (
        [(10, 13), (13, 8), (12, 2), (4, 4), (2, 13)],
        [(12, 12), (12, 11)],
    ),
    (
        [(5, 2), (2, 7), (13, 3), (12, 13), (3, 13)],
        [(3, 3), (3, 2)],
    ),
];
const COUNT_BASE: [u16; 6] = [7528, 7549, 7507, 7570, 7591, 7612];
const COUNT_FORCE: [u16; 6] = [7531, 7552, 7510, 7573, 7594, 7615];

#[derive(Component)]
struct Guest {
    kind: Kind,
    day: i64,
    idx: usize,
    looks: u8,
    index: u32,
    home: Vec3,
    wanders: bool,
    heading: f32,
    wait: f32,
}

#[derive(Component)]
struct Prop {
    day: i64,
}

/// The countdown's term (`aCD0_get_now_term`) from the seconds of the day.
fn term(secs: u32) -> u8 {
    if secs > 23 * 3600 + 59 * 60 {
        4
    } else if secs > 23 * 3600 + 55 * 60 {
        3
    } else if secs > 23 * 3600 + 50 * 60 {
        2
    } else if secs > 23 * 3600 + 30 * 60 {
        1
    } else if secs > 23 * 3600 {
        0
    } else if secs < 10 {
        5
    } else {
        6
    }
}

fn active_kind(today: &TodayEvents, hour: u8) -> Option<Kind> {
    if today.is_active(ev::CHERRY_BLOSSOM_FESTIVAL, hour) {
        Some(Kind::Cherry)
    } else if today.is_active(ev::HARVEST_MOON_FESTIVAL, hour) {
        Some(Kind::Moon)
    } else if today.is_active(ev::NEW_YEARS_EVE_COUNTDOWN, hour) {
        Some(Kind::Count)
    } else if today.is_active(ev::METEOR_SHOWER, hour) {
        Some(Kind::Meteor)
    } else if today.is_active(ev::HARVEST_FESTIVAL, hour) {
        Some(Kind::Harvest)
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    roster: Option<Res<TownRoster>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, Option<&Guest>, Option<&Prop>), Or<(With<Guest>, With<Prop>)>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    disc: Option<Res<crate::game_disc::GameDisc>>,
    mut props: ResMut<crate::event_body::Props>,
    mut n64: ResMut<Assets<crate::n64_material::N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let now = crate::clock::now();
    let kind = active_kind(&today, now.hour).filter(|_| {
        *state.get() == ShopLocation::Town && *house.get() == crate::house::HouseState::Town
    });
    let mut any = false;
    for (entity, guest, prop) in &existing {
        let keep = match (guest, prop, kind) {
            (Some(g), _, Some(k)) => g.kind == k && g.day == now.day_number,
            (_, Some(p), Some(_)) => p.day == now.day_number,
            _ => false,
        };
        if keep {
            any = true;
        } else {
            commands.entity(entity).despawn();
        }
    }
    let (Some(kind), false, Some(roster)) = (kind, any, roster) else {
        return;
    };
    let day = now.day_number;
    let layout = day.rem_euclid(7) as usize;
    let acre = water_acre(&town);
    let spot = |unit: (usize, usize)| -> Option<Vec3> {
        match kind {
            Kind::Cherry | Kind::Harvest => crate::tortimer::shrine_cell(&town, unit.0, unit.1),
            _ => acre.and_then(|a| unit_spot(&town, a, unit)),
        }
    };
    let mut guests: Vec<(Vec3, f32, bool)> = Vec::new();
    match kind {
        Kind::Cherry => {
            for (i, u) in CHERRY.iter().enumerate() {
                if let Some(at) = spot(*u) {
                    let deg = CHERRY_ANGLE.get(i).copied().unwrap_or(0.0);
                    guests.push((at, deg.to_radians(), i == 4));
                }
            }
        }
        Kind::Harvest => {
            for (i, u) in HARVEST.iter().enumerate() {
                if let Some(at) = spot(*u) {
                    let deg = HARVEST_ANGLE.get(i).copied().unwrap_or(0.0);
                    guests.push((at, deg.to_radians(), false));
                }
            }
        }
        Kind::Moon | Kind::Meteor => {
            let (px, pz) = MOON_POS[layout];
            let units = if kind == Kind::Moon {
                MOON[layout]
            } else {
                METEOR[layout]
            };
            for u in units {
                if let Some(at) = spot(u) {
                    let at = at + Vec3::new(px - 20.0, 0.0, pz - 20.0) / UNIT;
                    guests.push((at, MOON_ANGLE[layout].to_radians(), false));
                }
            }
        }
        Kind::Count => {
            for u in COUNT[layout].0 {
                if let Some(at) = spot(u) {
                    // They face the player's side, south (`mv_angl = 0`).
                    guests.push((at, 0.0, false));
                }
            }
        }
    }
    for (idx, ((at, heading, wanders), entry)) in
        guests.into_iter().zip(roster.residents.iter()).enumerate()
    {
        let mut guest = commands.spawn((
            Name::new(format!("{} ({kind:?})", entry.name)),
            Guest {
                kind,
                day,
                idx,
                looks: entry.looks,
                index: entry.index,
                home: at,
                wanders,
                heading,
                wait: 1.0,
            },
            NpcClip::default(),
            NpcGait::default(),
            crate::event_body::EventBody::default(),
            NpcModel(entry.index),
            Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
            Transform::from_translation(at + Vec3::Y).with_rotation(Quat::from_rotation_y(heading)),
        ));
        if let Some(idle) = seated(kind, idx) {
            guest.insert(idle);
        }
    }
    // The tables and signs, as boxes.
    let (units, size, colour): (Vec<(usize, usize)>, Vec3, Color) = match kind {
        Kind::Cherry => (
            TABLES.to_vec(),
            Vec3::new(1.6, 0.7, 1.0),
            Color::srgb(0.95, 0.7, 0.75),
        ),
        Kind::Moon | Kind::Meteor => (Vec::new(), Vec3::ONE, Color::WHITE),
        Kind::Harvest => (
            HARVEST_TABLES.to_vec(),
            Vec3::new(1.6, 0.7, 1.0),
            Color::srgb(0.8, 0.55, 0.3),
        ),
        Kind::Count => (
            COUNT[layout].1.to_vec(),
            Vec3::new(1.2, 2.0, 0.3),
            Color::srgb(0.9, 0.9, 0.5),
        ),
    };
    for (i, u) in units.into_iter().enumerate() {
        let Some(mut at) = spot(u) else { continue };
        // The disc's models, drawn translucent list then opaque: the cherry viewing's mats (`ac_goza`:
        // `obj_e_hanami_{a,b}`, alternating) and the harvest festival's tables (`ac_htable`: `obj_e_hfes_{a,b,c}`,
        // the third half a unit further south). The countdown's sign is a skeleton, still a box.
        let lists: Option<&[&str]> = match (kind, i) {
            (Kind::Cherry, _) if i % 2 == 0 => {
                Some(&["obj_e_hanami_a_modelT", "obj_e_hanami_a_model"])
            }
            (Kind::Cherry, _) => Some(&["obj_e_hanami_b_modelT", "obj_e_hanami_b_model"]),
            (Kind::Harvest, 0) => Some(&["obj_e_hfes_a_modelT", "obj_e_hfes_a_model"]),
            (Kind::Harvest, 1) => Some(&["obj_e_hfes_b_modelT", "obj_e_hfes_b_model"]),
            (Kind::Harvest, _) => Some(&["obj_e_hfes_c_modelT", "obj_e_hfes_c_model"]),
            _ => None,
        };
        if kind == Kind::Harvest && i == 2 {
            at.z += 0.5;
        }
        let parts = match (lists, disc.as_deref()) {
            (Some(lists), Some(d)) => {
                props.get(d, lists, None, None, (&mut meshes, &mut n64, &mut images))
            }
            _ => Vec::new(),
        };
        // The countdown's board (`ac_countdown`, `NEWYEAR_COUNTDOWN0`): the skeleton `cKF_bs_r_obj_e_count01` at frame 1,
        // joint 4 (the lamp panel) lit blue, the rest dark (`aCOU_actor_draw_before` outside the New Year).
        let board = (kind == Kind::Count && i == 0)
            .then_some(disc.as_deref())
            .flatten()
            .map(|d| {
                let palette = crate::disc_town::structure_palette(d, PAL_COUNT, false);
                crate::event_body::skeleton_parts(
                    d,
                    "cKF_bs_r_obj_e_count01",
                    "cKF_ba_r_obj_e_count01",
                    1.0,
                    palette,
                    &|joint| {
                        let (prim, env, lod) = match joint {
                            4 => ([100, 70, 200, 255], [10, 10, 80, 255], 128),
                            5 => ([0, 0, 0, 255], [0, 0, 60, 255], 40),
                            6 => ([0, 0, 0, 255], [0, 0, 60, 255], 20),
                            _ => ([255; 4], [0; 4], 255),
                        };
                        crate::disc_town::DrawColors {
                            prim,
                            prim_lod: lod,
                            env,
                        }
                    },
                    (&mut meshes, &mut n64, &mut images),
                )
            })
            .unwrap_or_default();
        let root = commands
            .spawn((
                Name::new("gathering prop"),
                Prop { day },
                Transform::from_translation(at),
                Visibility::default(),
            ))
            .id();
        for (mesh, material, at) in &board {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                *at,
                ChildOf(root),
            ));
        }
        if !board.is_empty() {
            crate::event_body::attach_shadow(
                &mut commands,
                root,
                &mut props,
                disc.as_deref(),
                &["obj_e_count_shadow_model"],
                Some("obj_e_count_shadow_v"),
                (&mut meshes, &mut n64, &mut images),
            );
        }
        if parts.is_empty() && board.is_empty() {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
                MeshMaterial3d(materials.add(colour)),
                Transform::from_xyz(0.0, size.y / 2.0, 0.0),
                ChildOf(root),
            ));
        } else {
            crate::event_body::attach(&mut commands, root, &parts);
            // `ac_goza` / `ac_htable` shadows: `obj_e_hfes_shadow_{a,b,c}`.
            let (shadow, vertices): (&'static str, &'static str) = match (kind, i) {
                (Kind::Cherry, _) if i % 2 == 0 => {
                    ("obj_e_hfes_shadow_a_modelT", "obj_e_hfes_shadow_a_v")
                }
                (Kind::Cherry, _) | (Kind::Harvest, 1) => {
                    ("obj_e_hfes_shadow_b_modelT", "obj_e_hfes_shadow_b_v")
                }
                (Kind::Harvest, 0) => ("obj_e_hfes_shadow_a_modelT", "obj_e_hfes_shadow_a_v"),
                _ => ("obj_e_hfes_shadow_c_modelT", "obj_e_hfes_shadow_c_v"),
            };
            crate::event_body::attach_shadow(
                &mut commands,
                root,
                &mut props,
                disc.as_deref(),
                &[shadow],
                Some(vertices),
                (&mut meshes, &mut n64, &mut images),
            );
        }
    }
    info!("The {kind:?} gathering is on");
}

/// The cherry viewing's fifth villager wanders.
fn wander(
    time: Res<Time>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    town: Res<Town>,
    mut guests: Query<(Entity, &mut Guest, &mut Transform)>,
    player: Query<&Transform, (With<Player>, Without<Guest>)>,
) {
    let dt = time.delta_secs();
    let player_at = player.single().ok().map(|t| t.translation);
    let bodies: Vec<(Entity, Vec3)> = guests.iter().map(|(e, _, t)| (e, t.translation)).collect();
    for (entity, mut g, mut at) in &mut guests {
        if !g.wanders {
            continue;
        }
        g.wait -= dt;
        if g.wait <= 0.0 {
            g.heading = rand.0.fqrand() * std::f32::consts::TAU;
            g.wait = 1.0 + rand.0.fqrand() * 2.0;
        }
        let dir = Vec3::new(g.heading.sin(), 0.0, g.heading.cos());
        let next = at.translation + dir * 1.5 * dt;
        let away = (next - g.home).with_y(0.0);
        if away.length() > 100.0 / UNIT {
            g.heading = (-away.x).atan2(-away.z);
            continue;
        }
        if !town.can_step(at.translation, next)
            || !crate::event_body::step_clear(
                at.translation,
                next,
                bodies.iter().filter(|(e, _)| *e != entity).map(|(_, p)| *p),
                player_at,
            )
        {
            g.wait = 0.0;
            continue;
        }
        at.translation = next;
        at.rotation = Quat::from_rotation_y(g.heading);
    }
}

/// What they do while they stand: the viewers sit (`aHM0_ACT_WAIT / MERRY`), the moon-gazers sit on the pier edge
/// (`aNPC_ANIM_SANBASI1`), the harvest guests clap now and then.
fn seated(kind: Kind, idx: usize) -> Option<crate::event_body::Idle> {
    use crate::event_body::Idle;
    match kind {
        Kind::Cherry if idx < 4 && idx.is_multiple_of(2) => {
            Some(Idle("cKF_ba_r_npc_1_sitdown_wait1"))
        }
        Kind::Cherry if idx < 4 => Some(Idle("cKF_ba_r_npc_1_sitdown_clap1")),
        Kind::Moon | Kind::Meteor => Some(Idle("cKF_ba_r_npc_1_sanbasi1")),
        Kind::Harvest if idx < 4 && idx % 2 == 1 => Some(Idle("cKF_ba_r_npc_1_clap1")),
        _ => None,
    }
}

fn line(g: &Guest, term: u8, rand: &mut crate::game_rand::GameRand) -> u16 {
    let looks = usize::from(g.looks).min(5);
    let r = rand.0.random(3) as u16;
    match g.kind {
        Kind::Cherry => {
            if g.wanders {
                CHERRY_BASE5[looks] + r
            } else {
                CHERRY_BASE[looks] + r + 3 * g.idx as u16
            }
        }
        Kind::Moon => MOON_BASE[looks] + r + 3 * g.idx as u16,
        Kind::Meteor => METEOR_BASE[looks] + r + 3 * g.idx as u16,
        Kind::Harvest => {
            if g.idx == 4 {
                HARVEST_BASE5[looks] + r
            } else {
                HARVEST_BASE[looks] + r + 3 * g.idx as u16
            }
        }
        Kind::Count => {
            let more = if term >= 5 { 17 } else { u16::from(term) * 4 };
            COUNT_BASE[looks] + r + more
        }
    }
}

fn context_for(
    identity: &crate::save::PlayerIdentity,
    disc: &GameDisc,
    index: u32,
    counting: bool,
) -> crate::messages::MsgContext {
    let now = crate::clock::now();
    let mut ctx = crate::fireworks::context(identity, disc, index, 0);
    // `aCD0_set_free_str`: the coming year and the minutes left in the hour.
    if counting {
        ctx.free[1] = (u32::from(now.year) + 1).to_string();
        ctx.free[2] = (59 - u32::from(now.minute)).to_string();
    }
    ctx
}

fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &crate::messages::MsgContext,
    index: u32,
    msg: u16,
) {
    let Some(text) = disc.text.as_ref() else {
        return;
    };
    let (pages, _) = text.talk_asking(usize::from(msg), ctx, &mut |_| 0);
    dialogue.begin_special(actor, "Villager", (index, 2), pages, None);
    info!("Gathering message {msg:#06x}");
}

#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), (With<Player>, Without<Guest>)>,
    guests: Query<(Entity, &Transform, &Guest)>,
    mut dialogue: ResMut<DialogueState>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut last_term: Local<Option<u8>>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let now = crate::clock::now();
    let term_now = term(now.day_secs);
    // A line ends when it is read.
    if dialogue.active
        && dialogue.speaker.is_some_and(|e| guests.get(e).is_ok())
        && dialogue.external_result.take().is_some()
    {
        *dialogue = DialogueState::default();
        *focus = InputFocus::World;
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    // The countdown host speaks by himself at the marks (`aCD0_force_talk_request`).
    let counting = guests.iter().any(|(_, _, g)| g.kind == Kind::Count);
    if !counting {
        *last_term = None;
    } else if *last_term != Some(term_now) {
        let first = last_term.is_none();
        *last_term = Some(term_now);
        if !first && (1..=4).contains(&term_now) && focus.is_world() && !dialogue.active {
            if let Some((host, t, g)) = guests.iter().find(|(_, _, g)| g.idx == 0) {
                if (t.translation - at.translation).with_y(0.0).length() < 14.0 {
                    let msg =
                        COUNT_FORCE[usize::from(g.looks).min(5)] + (u16::from(term_now) - 1) * 4;
                    *focus = InputFocus::Dialogue;
                    let ctx = context_for(&identity, &disc, g.index, true);
                    say(&disc, &mut dialogue, host, &ctx, g.index, msg);
                    return;
                }
            }
        }
    }
    if !focus.is_world() || !actions.just_pressed(Action::Interact) {
        return;
    }
    let Some((actor, _, g)) = guests.iter().find(|(_, t, _)| {
        let delta = (t.translation - at.translation).with_y(0.0);
        delta.length() < 1.8 && facing.dir.dot(delta.normalize_or_zero()) >= 0.5
    }) else {
        return;
    };
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    let ctx = context_for(&identity, &disc, g.index, g.kind == Kind::Count);
    let msg = line(g, term_now, &mut rand);
    say(&disc, &mut dialogue, actor, &ctx, g.index, msg);
}

/// The midnight salvo (`eEC_EFFECT_HANABI_SWITCH` at the new year).
#[allow(clippy::too_many_arguments)]
fn midnight(
    mut commands: Commands,
    time: Res<Time>,
    town: Res<Town>,
    guests: Query<&Guest>,
    mut left: Local<f32>,
    mut next: Local<f32>,
    mut seen: Local<Option<u8>>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = term(crate::clock::now().day_secs);
    let counting = guests.iter().any(|g| g.kind == Kind::Count);
    if counting && *seen != Some(t) {
        if t == 5 && seen.is_some() {
            *left = 12.0;
        }
        *seen = Some(t);
    }
    if !counting {
        *seen = None;
        *left = 0.0;
    }
    if *left <= 0.0 {
        return;
    }
    let dt = time.delta_secs();
    *left -= dt;
    *next -= dt;
    if *next > 0.0 {
        return;
    }
    *next = 0.3 + rand.0.fqrand() * 0.6;
    let Some(acre) = water_acre(&town) else {
        return;
    };
    let centre = Town::cell_to_world(acre.0, acre.1, 8, 8);
    let at = centre
        + Vec3::new(
            (rand.0.fqrand() - 0.5) * 12.0,
            9.0 + rand.0.fqrand() * 4.0,
            (rand.0.fqrand() - 0.5) * 12.0,
        );
    burst(&mut commands, &mut meshes, &mut materials, &mut rand, at);
}

/// Shooting stars over the pond acre while the meteor shower runs (a stand-in for the sky effect).
#[allow(clippy::too_many_arguments)]
fn meteors(
    mut commands: Commands,
    time: Res<Time>,
    town: Res<Town>,
    guests: Query<&Guest>,
    mut next: Local<f32>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !guests.iter().any(|g| g.kind == Kind::Meteor) {
        return;
    }
    *next -= time.delta_secs();
    if *next > 0.0 {
        return;
    }
    *next = 0.6 + rand.0.fqrand() * 1.8;
    let Some(acre) = water_acre(&town) else {
        return;
    };
    let centre = Town::cell_to_world(acre.0, acre.1, 8, 8);
    let at = centre
        + Vec3::new(
            10.0 + rand.0.fqrand() * 14.0,
            24.0 + rand.0.fqrand() * 8.0,
            (rand.0.fqrand() - 0.5) * 30.0,
        );
    let vel = Vec3::new(-16.0, -3.0, (rand.0.fqrand() - 0.5) * 4.0);
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: LinearRgba::new(6.0, 6.0, 5.0, 1.0),
        unlit: true,
        ..default()
    });
    commands.spawn((
        crate::fireworks::Spark { vel, age: 0.0 },
        Mesh3d(meshes.add(Cuboid::new(2.4, 0.08, 0.08))),
        MeshMaterial3d(material),
        Transform::from_translation(at)
            .looking_to(vel, Vec3::Y)
            .with_rotation(Quat::from_rotation_arc(Vec3::X, -vel.normalize())),
    ));
}

pub struct GatheringsPlugin;

impl Plugin for GatheringsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            visit.after(crate::event_today::refresh).in_set(GameplaySet),
        )
        .add_systems(Update, (wander, midnight, meteors).in_set(GameplaySet))
        .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
