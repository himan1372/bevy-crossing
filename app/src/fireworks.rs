//! The fireworks show (`ac_hanabi_npc0.c`, `ac_hanabi_npc1.c`, `ac_ev_yomise.c`, `ac_yatai.c`, `m_event_map_npc_data.c_inc`).
//!
//! While `mEv_EVENT_FIREWORKS_SHOW` runs (July 4, 19:00-20:59) the acre with the water has one of seven layouts
//! (`l_fws_pule1..7`, picked by the day here; the original by the pond's block type): five villagers
//! (`SP_NPC_EV_HANABI_0..4`), the night-stall seller (`SP_NPC_EV_YOMISE`, `_2` in the odd layouts) and his stall on their
//! units. The first villager wanders within 100 units of his place; the others stand fanning or clapping, facing as
//! `def_angle` says. A villager's talk is `base(looks) + 3 * k + RANDOM(3)` for the k-th villager, the bases being
//! `5711 5726 5696 5741 5756 5771` by looks.
//!
//! The seller (`ac_ev_yomise_move.c_inc`): the day's draw of kind (a bluebell fan 780 Bells, a yellow pinwheel 680 or
//! a red balloon 480; eight colours of it, saved as the event's area) and his talk. The offer `0x1758 / 0x1759 /
//! 0x175A` (Yes / No); Yes shows the goods three at a time with a fourth refusal ("I'm not buying!" when the list is
//! over, "I don't want it!" when more follow: `0x1761` or `0x1762`); with full pockets `0x175D`, too few Bells `0x175E`;
//! a purchase `0x1764`, then `0x1766` (anything else, Yes / No) or `0x1765` when he has none left; a refusal at the end of
//! the list `0x1763`, otherwise `0x1767` and the next three; with no goods left `0x1757`.
//!
//! Fireworks: bursts of coloured sparks over the acre while the show runs (a stand-in for `eEC_EFFECT_HANABI_*`).
//!
//! The stall is the disc's `obj_e_yatai_{l,r}` model (palette `aSTR_PAL_YATAI`).
//!
//! Not built: the villagers' fan and clap animations, the stall's shadow, the seller's fan, the sounds and
//! the effect's own timing.

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::inventory::EMPTY_NO;
use serde::{Deserialize, Serialize};

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::messages::MsgContext;
use crate::player::Player;
use crate::roster::TownRoster;
use crate::shop::ShopLocation;
use crate::shop_nook::handover::{HandOver, Item, Party, Request};
use crate::states::GameplaySet;
use crate::town::Town;

const SOUND: u8 = 2;
const UNIT: f32 = 40.0;
/// `aSTR_PAL_YATAI`: the stall's palette.
const PAL_YATAI: u32 = 70;
/// `SP_NPC_EV_YOMISE` and `SP_NPC_EV_YOMISE2` as model numbers.
const YOMISE_NPC: [u32; 2] = [238 + 44, 238 + 57];
/// The stall seller's kinds: base item, price, the offer's message.
const KINDS: [(u16, u32, u16); 3] = [
    (0x2254, 780, 0x1758), // bluebell fan
    (0x224C, 680, 0x1759), // yellow pinwheel
    (0x2244, 480, 0x175A), // red balloon
];
/// `aEYMS_GOODS_COUNT`.
const GOODS: usize = 8;
const BASE_MSG: [u16; 6] = [5711, 5726, 5696, 5741, 5756, 5771];
/// The seven layouts: the villagers' units, the seller's, and whether the seller is the second model.
const LAYOUTS: [([(usize, usize); 5], (usize, usize), bool); 7] = [
    ([(2, 3), (8, 1), (2, 11), (13, 7), (3, 3)], (12, 12), true),
    ([(13, 13), (1, 3), (13, 2), (3, 8), (14, 7)], (3, 12), false),
    ([(12, 2), (2, 3), (7, 2), (12, 7), (13, 7)], (12, 11), true),
    ([(13, 11), (2, 3), (7, 2), (2, 8), (9, 12)], (3, 12), false),
    ([(13, 6), (1, 7), (13, 13), (2, 2), (1, 11)], (12, 3), true),
    ([(13, 2), (1, 8), (8, 1), (9, 13), (6, 14)], (12, 11), true),
    ([(13, 12), (12, 2), (2, 11), (3, 7), (8, 1)], (3, 3), false),
];
/// `def_angle[villager - 1][layout]` in degrees (the villagers 1..4).
const ANGLES: [[f32; 7]; 4] = [
    [0.0, 45.0, 45.0, 45.0, 90.0, 90.0, 315.0],
    [135.0, 315.0, 0.0, 0.0, 225.0, 0.0, 135.0],
    [270.0, 90.0, 270.0, 90.0, 45.0, 180.0, 90.0],
    [315.0, 270.0, 180.0, 180.0, 90.0, 180.0, 0.0],
];

/// The seller's area for the show (`aEv_yomise_save_c`).
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StallEvent {
    /// The game day it belongs to.
    pub day: i64,
    /// `kind`: 0 fans, 1 pinwheels, 2 balloons.
    pub kind: u8,
    /// `goods`: the colours left (bit n is `base + n`).
    pub left: u8,
}

#[derive(Component)]
struct Show {
    day: i64,
}

#[derive(Component)]
struct Villager {
    looks: u8,
    index: u32,
    k: usize,
    home: Vec3,
    /// The walking one: the heading, and seconds before the next turn.
    heading: f32,
    wait: f32,
}

#[derive(Component)]
struct Seller;

#[derive(Component)]
pub(crate) struct Spark {
    pub(crate) vel: Vec3,
    pub(crate) age: f32,
}

/// The acre with the most water.
pub(crate) fn water_acre(town: &Town) -> Option<(usize, usize)> {
    let mut counts = std::collections::HashMap::new();
    for &(ax, az, ..) in &town.river_tiles {
        *counts.entry((ax, az)).or_insert(0usize) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|&(at, n)| (n, std::cmp::Reverse(at)))
        .map(|(at, _)| at)
}

/// A clear spot near unit (cx, cz) of an acre.
pub(crate) fn unit_spot(town: &Town, acre: (usize, usize), unit: (usize, usize)) -> Option<Vec3> {
    for r in 0..4i32 {
        for dz in -r..=r {
            for dx in -r..=r {
                let (cx, cz) = (unit.0 as i32 + dx, unit.1 as i32 + dz);
                if !(0..16).contains(&cx) || !(0..16).contains(&cz) {
                    continue;
                }
                let at = Town::cell_to_world(acre.0, acre.1, cx as usize, cz as usize);
                if town.is_clear_ground(at) && !town.in_retail_obstacle(at) {
                    return Some(at.with_y(town.ground_y(at)));
                }
            }
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    roster: Option<Res<TownRoster>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Show)>,
    mut stall: ResMut<StallEvent>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    disc: Option<Res<crate::game_disc::GameDisc>>,
    mut props: ResMut<crate::event_body::Props>,
    mut n64: ResMut<Assets<crate::n64_material::N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let now = crate::clock::now();
    let on = today.is_active(ev::FIREWORKS_SHOW, now.hour);
    let out =
        on && *state.get() == ShopLocation::Town && *house.get() == crate::house::HouseState::Town;
    for (entity, s) in &existing {
        if !out || s.day != now.day_number {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    let (Some(roster), Some(acre)) = (roster, water_acre(&town)) else {
        warn!("The fireworks show has no villagers or no water acre");
        return;
    };
    // `setUp_yomise_goods`: a new show's draw of goods.
    if stall.day != now.day_number {
        *stall = StallEvent {
            day: now.day_number,
            kind: rand.0.random(3) as u8,
            left: 0xFF,
        };
    }
    let layout = (now.day_number.rem_euclid(7)) as usize;
    let (units, seller_unit, second) = LAYOUTS[layout];
    let show = || Show {
        day: now.day_number,
    };
    for (k, entry) in roster.residents.iter().take(5).enumerate() {
        let Some(at) = unit_spot(&town, acre, units[k]) else {
            continue;
        };
        // The walking one starts anywhere; the others face by the table.
        let heading = if k == 0 {
            rand.0.fqrand() * std::f32::consts::TAU
        } else {
            ANGLES[k - 1][layout].to_radians()
        };
        let mut villager = commands.spawn((
            Name::new(format!("{} (fireworks)", entry.name)),
            show(),
            Villager {
                looks: entry.looks,
                index: entry.index,
                k,
                home: at,
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
        // The others fan, clap and cheer while they stand (`aHN1_ACT_FAN / CLAPPING / HURRAHS`).
        match k {
            0 => {}
            2 => {
                villager.insert(crate::event_body::Idle("cKF_ba_r_npc_1_clap1"));
            }
            3 => {
                villager.insert(crate::event_body::Idle("cKF_ba_r_npc_1_banzai1"));
            }
            _ => {
                villager.insert(crate::event_body::Idle("cKF_ba_r_npc_1_utiwa_wait1"));
            }
        }
    }
    if let Some(at) = unit_spot(&town, acre, seller_unit) {
        let centre = Town::cell_to_world(acre.0, acre.1, 8, 8);
        // He faces the land side, away from the water, with the stall before him.
        let to = (at - centre).with_y(0.0);
        // The stall beside him: `ac_yatai`'s `obj_e_yatai_{l,r}_modelT` then `_model` with the stall palette
        // (`aSTR_PAL_YATAI`); a box when the disc has no model.
        let stall_at = at + to.normalize_or_zero() * 1.3;
        let parts = disc.as_deref().map_or_else(Vec::new, |d| {
            let lists: &[&str] = if second {
                &["obj_e_yatai_r_modelT", "obj_e_yatai_r_model"]
            } else {
                &["obj_e_yatai_l_modelT", "obj_e_yatai_l_model"]
            };
            props.get(
                d,
                lists,
                Some(PAL_YATAI),
                None,
                (&mut meshes, &mut n64, &mut images),
            )
        });
        let stall_root = commands
            .spawn((
                Name::new("stall"),
                show(),
                Transform::from_translation(stall_at.with_y(town.ground_y(stall_at)))
                    .with_rotation(Quat::from_rotation_y(to.x.atan2(to.z))),
                Visibility::default(),
            ))
            .id();
        if parts.is_empty() {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.6, 1.0, 0.9))),
                MeshMaterial3d(materials.add(Color::srgb(0.7, 0.35, 0.2))),
                Transform::from_xyz(0.0, 0.5, 0.0),
                ChildOf(stall_root),
            ));
        } else {
            crate::event_body::attach(&mut commands, stall_root, &parts);
        }
        commands.spawn((
            Name::new("night stall seller"),
            show(),
            Seller,
            NpcClip::default(),
            NpcGait::default(),
            crate::event_body::EventBody::default(),
            NpcModel(YOMISE_NPC[usize::from(second)]),
            Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
            Transform::from_translation(at + Vec3::Y)
                .with_rotation(Quat::from_rotation_y(to.x.atan2(to.z))),
        ));
    }
    info!(
        "The fireworks show is on in acre {acre:?}, layout {}, goods kind {}",
        layout + 1,
        stall.kind
    );
}

/// The first villager wanders (`aHN0_think_main_proc`), within 100 units of his place.
fn wander(
    time: Res<Time>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    town: Res<Town>,
    mut villagers: Query<(Entity, &mut Villager, &mut Transform)>,
    others: Query<(Entity, &Transform), (With<crate::event_body::EventBody>, Without<Villager>)>,
    player: Query<
        &Transform,
        (
            With<Player>,
            Without<Villager>,
            Without<crate::event_body::EventBody>,
        ),
    >,
) {
    let dt = time.delta_secs();
    let player_at = player.single().ok().map(|t| t.translation);
    let mut bodies: Vec<(Entity, Vec3)> = villagers
        .iter()
        .map(|(e, _, t)| (e, t.translation))
        .collect();
    bodies.extend(others.iter().map(|(e, t)| (e, t.translation)));
    for (entity, mut v, mut at) in &mut villagers {
        if v.k != 0 {
            continue;
        }
        v.wait -= dt;
        if v.wait <= 0.0 {
            v.heading = rand.0.fqrand() * std::f32::consts::TAU;
            v.wait = 1.0 + rand.0.fqrand() * 2.0;
        }
        let dir = Vec3::new(v.heading.sin(), 0.0, v.heading.cos());
        let next = at.translation + dir * 1.5 * dt;
        let away = (next - v.home).with_y(0.0);
        if away.length() > 100.0 / UNIT {
            // Back toward the middle (`aHN0_revise_moveRange`).
            v.heading = (-away.x).atan2(-away.z);
            continue;
        }
        // The ground and the bodies; blocked, he turns elsewhere.
        if !town.can_step(at.translation, next)
            || !crate::event_body::step_clear(
                at.translation,
                next,
                bodies.iter().filter(|(e, _)| *e != entity).map(|(_, p)| *p),
                player_at,
            )
        {
            v.wait = 0.0;
            continue;
        }
        at.translation = next;
        at.rotation = Quat::from_rotation_y(v.heading);
    }
}

/// Sparks over the acre while the show runs.
#[allow(clippy::too_many_arguments)]
fn fireworks(
    mut commands: Commands,
    time: Res<Time>,
    town: Res<Town>,
    today: Res<TodayEvents>,
    mut next: Local<f32>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sparks: Query<(Entity, &mut Spark, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut s, mut at) in &mut sparks {
        s.age += dt;
        s.vel.y -= 4.5 * dt;
        at.translation += s.vel * dt;
        at.scale = Vec3::splat((1.0 - s.age / 2.2).max(0.05));
        if s.age > 2.2 {
            commands.entity(entity).despawn();
        }
    }
    let now = crate::clock::now();
    if !today.is_active(ev::FIREWORKS_SHOW, now.hour) {
        return;
    }
    let Some(acre) = water_acre(&town) else {
        return;
    };
    *next -= dt;
    if *next > 0.0 {
        return;
    }
    *next = 1.2 + rand.0.fqrand() * 2.8;
    let centre = Town::cell_to_world(acre.0, acre.1, 8, 8);
    let at = centre
        + Vec3::new(
            (rand.0.fqrand() - 0.5) * 10.0,
            9.0 + rand.0.fqrand() * 4.0,
            (rand.0.fqrand() - 0.5) * 10.0,
        );
    burst(&mut commands, &mut meshes, &mut materials, &mut rand, at);
}

/// One burst of coloured sparks (`eEC_EFFECT_HANABI_*` stand-in).
pub(crate) fn burst(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    rand: &mut crate::game_rand::GameRand,
    burst: Vec3,
) {
    let hue = rand.0.fqrand() * 360.0;
    let colour = Color::hsl(hue, 1.0, 0.6);
    let linear = colour.to_linear();
    let material = materials.add(StandardMaterial {
        base_color: colour,
        emissive: LinearRgba::new(linear.red * 4.0, linear.green * 4.0, linear.blue * 4.0, 1.0),
        unlit: true,
        ..default()
    });
    let mesh = meshes.add(Sphere::new(0.12));
    for _ in 0..36 {
        let a = rand.0.fqrand() * std::f32::consts::TAU;
        let b = (rand.0.fqrand() - 0.5) * std::f32::consts::PI;
        let speed = 3.0 + rand.0.fqrand() * 2.0;
        commands.spawn((
            Spark {
                vel: Vec3::new(a.cos() * b.cos(), b.sin(), a.sin() * b.cos()) * speed,
                age: 0.0,
            },
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(burst),
        ));
    }
}

pub(crate) fn context(
    identity: &crate::save::PlayerIdentity,
    disc: &GameDisc,
    index: u32,
    price: u32,
) -> MsgContext {
    let now = crate::clock::now();
    let mut free = vec![String::new(); 20];
    free[0] = price.to_string();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Villager".into(),
        tail: crate::dialogue::catchphrase(disc, index),
        free,
        items: vec![String::new(); 5],
        item_articles: vec![0; 5],
        town_name: identity.town.clone(),
        year: now.year,
        month: now.month,
        day: now.day,
        hour: now.hour,
        minute: now.minute,
        weekday: now.weekday,
        ..default()
    }
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// A line is up; the talk ends with it.
    Line,
    /// A Yes / No is up (the offer, or "anything else"); `from` is the first good of the list shown next.
    Ask(usize),
    /// The goods list is up: the goods behind the choices and where the next page starts (`None`: the list ends).
    Choose(Vec<usize>, Option<usize>),
    /// The bought item is crossing.
    Giving,
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
}

fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &MsgContext,
    name: &'static str,
    model: u32,
    msg: u16,
    labels: Option<Vec<String>>,
) -> bool {
    let Some(text) = disc.text.as_ref() else {
        return false;
    };
    let (pages, asked) = text.talk_asking(usize::from(msg), ctx, &mut |_| 0);
    let asked_own = asked.as_ref().map(|(_, c)| c.labels.clone());
    let has_choice = labels.is_some() || asked_own.is_some();
    dialogue.begin_special(actor, name, (model, SOUND), pages, labels.or(asked_own));
    info!("Fireworks message {msg:#06x}");
    has_choice
}

/// The goods still on offer from `from`, up to `max` (`aEYMS_set_choise_data`): their indices.
fn on_offer(left: u8, from: usize, max: usize) -> (Vec<usize>, Option<usize>) {
    let mut out = Vec::new();
    let mut j = from;
    while out.len() < max && j < GOODS {
        if left & (1 << j) != 0 {
            out.push(j);
        }
        j += 1;
    }
    // Whether more follow after these.
    let more = (j..GOODS).any(|g| left & (1 << g) != 0);
    (out, more.then_some(j))
}

#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), (With<Player>, Without<Show>)>,
    seller: Query<(Entity, &Transform), With<Seller>>,
    villagers: Query<(Entity, &Transform, &Villager)>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    mut stall: ResMut<StallEvent>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let (base, price, offer) = KINDS[usize::from(stall.kind).min(2)];
    let seller_e = seller.single().ok().map(|(e, _)| e);
    let model = YOMISE_NPC[0];
    let is_mine = |e: Entity| Some(e) == seller_e || villagers.get(e).is_ok();
    let item_name = |j: usize| crate::items::name(base + j as u16);
    // The seller's goods list as choices.
    let choose = |from: usize, stall: &StallEvent| -> (Vec<String>, Vec<usize>, Option<usize>) {
        let (idx, more) = on_offer(stall.left, from, 3);
        let mut labels: Vec<String> = idx.iter().map(|&j| item_name(j)).collect();
        // The fourth choice: the refusal, worded by whether more goods follow.
        labels.push(
            if more.is_some() {
                "I don't want it!"
            } else {
                "I'm not buying!"
            }
            .to_string(),
        );
        (labels, idx, more)
    };
    // A line was read, or an answer given.
    if let (Some(actor), true) = (
        dialogue.speaker.filter(|&e| is_mine(e)),
        dialogue.active || dialogue.external_result.is_some(),
    ) {
        if let Some(choice) = dialogue.external_result.take() {
            *focus = InputFocus::Dialogue;
            let ctx = context(&identity, &disc, 0, price);
            let name = "Seller";
            match std::mem::take(&mut flow.step) {
                Step::Ask(from) => {
                    if choice == 0 {
                        // `aEYMS_to_talk_buy`: the list, with the message by how many follow.
                        let (labels, idx, more) = choose(from, &stall);
                        let msg = if idx.len() < 4 && more.is_none() {
                            0x1761
                        } else {
                            0x1762
                        };
                        say(
                            &disc,
                            &mut dialogue,
                            actor,
                            &ctx,
                            name,
                            model,
                            msg,
                            Some(labels),
                        );
                        flow.step = Step::Choose(idx, more);
                    } else {
                        *dialogue = DialogueState::default();
                        *focus = InputFocus::World;
                    }
                }
                Step::Choose(idx, more) => {
                    let Some(&j) = idx.get(choice).filter(|_| choice < idx.len()) else {
                        // The refusal: the end of the list ends it, else the next three.
                        match more {
                            None => {
                                say(&disc, &mut dialogue, actor, &ctx, name, model, 0x1763, None);
                                flow.step = Step::Line;
                            }
                            Some(next) => {
                                say(&disc, &mut dialogue, actor, &ctx, name, model, 0x1767, None);
                                flow.step = Step::Ask(next);
                            }
                        }
                        return;
                    };
                    if !inventory.pockets.pockets.contains(&EMPTY_NO) {
                        say(&disc, &mut dialogue, actor, &ctx, name, model, 0x175D, None);
                        flow.step = Step::Line;
                    } else if inventory.bells < price {
                        say(&disc, &mut dialogue, actor, &ctx, name, model, 0x175E, None);
                        flow.step = Step::Line;
                    } else {
                        // `mPr_SetFreePossessionItem`, `mSP_get_sell_price`.
                        let item = base + j as u16;
                        inventory.add(item);
                        inventory.bells -= price;
                        stall.left &= !(1 << j);
                        info!("The seller sells {item:#06x} for {price} Bells");
                        let mut ctx = ctx;
                        ctx.items = vec![crate::items::name(item); 5];
                        say(&disc, &mut dialogue, actor, &ctx, name, model, 0x1764, None);
                        flow.step = Step::Giving;
                        handover.start(Request {
                            item: Item::Goods(item),
                            giver: Party::Npc(actor),
                            taker: Party::Player,
                        });
                    }
                }
                Step::Giving => {
                    // The purchase line is read: the item crosses, then the next question.
                    *dialogue = DialogueState::default();
                    flow.step = Step::Giving;
                }
                Step::Line | Step::Idle => {
                    *dialogue = DialogueState::default();
                    *focus = InputFocus::World;
                }
            }
            return;
        }
    }
    // The bought item crossing, then `0x1766` (anything else?) or `0x1765`.
    if matches!(flow.step, Step::Giving) {
        if dialogue.active {
            // The purchase line is being read.
            if dialogue.speaker == seller_e && dialogue.external_result.take().is_some() {
                *dialogue = DialogueState::default();
            }
            return;
        }
        if handover.busy() {
            *focus = InputFocus::Dialogue;
            return;
        }
        let Some(actor) = seller_e else {
            flow.step = Step::Idle;
            return;
        };
        let ctx = context(&identity, &disc, 0, price);
        *focus = InputFocus::Dialogue;
        if stall.left == 0 {
            say(
                &disc,
                &mut dialogue,
                actor,
                &ctx,
                "Seller",
                model,
                0x1765,
                None,
            );
            flow.step = Step::Line;
        } else {
            say(
                &disc,
                &mut dialogue,
                actor,
                &ctx,
                "Seller",
                model,
                0x1766,
                None,
            );
            flow.step = Step::Ask(0);
        }
        return;
    }
    // A villager's line ends when it is read.
    if matches!(flow.step, Step::Idle)
        && dialogue.active
        && dialogue.speaker.is_some_and(|e| villagers.get(e).is_ok())
        && dialogue.external_result.take().is_some()
    {
        *dialogue = DialogueState::default();
        *focus = InputFocus::World;
        return;
    }
    if !matches!(flow.step, Step::Idle)
        || !focus.is_world()
        || !actions.just_pressed(Action::Interact)
    {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    let near = |t: &Transform, reach: f32| {
        let delta = (t.translation - at.translation).with_y(0.0);
        delta.length() < reach && facing.dir.dot(delta.normalize_or_zero()) >= 0.5
    };
    // The seller talks across his counter.
    if let Some((actor, t)) = seller.iter().find(|(_, t)| near(t, 2.8)) {
        // `aEYMS_talk_request`: he must face the player too.
        let to = (at.translation - t.translation).with_y(0.0);
        let looks_at = (t.rotation * Vec3::Z).dot(to.normalize_or_zero()) > 0.0;
        if !looks_at {
            return;
        }
        actions.consume(Action::Interact);
        *focus = InputFocus::Dialogue;
        let ctx = context(&identity, &disc, 0, price);
        if stall.left == 0 {
            say(
                &disc,
                &mut dialogue,
                actor,
                &ctx,
                "Seller",
                model,
                0x1757,
                None,
            );
            flow.step = Step::Line;
        } else {
            say(
                &disc,
                &mut dialogue,
                actor,
                &ctx,
                "Seller",
                model,
                offer,
                None,
            );
            flow.step = Step::Ask(0);
        }
        return;
    }
    if let Some((actor, _, v)) = villagers.iter().find(|(_, t, _)| near(t, 1.8)) {
        actions.consume(Action::Interact);
        *focus = InputFocus::Dialogue;
        let ctx = context(&identity, &disc, v.index, price);
        let msg = BASE_MSG[usize::from(v.looks).min(5)] + 3 * v.k as u16 + rand.0.random(3) as u16;
        say(
            &disc,
            &mut dialogue,
            actor,
            &ctx,
            "Villager",
            v.index,
            msg,
            None,
        );
    }
}

pub struct FireworksPlugin;

impl Plugin for FireworksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .init_resource::<StallEvent>()
            .add_systems(
                Update,
                visit.after(crate::event_today::refresh).in_set(GameplaySet),
            )
            .add_systems(Update, (wander, fireworks).in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
