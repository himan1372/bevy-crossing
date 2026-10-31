//! Pete (SP_NPC_POST_MAN, ac_npc_post_man): the postman.
//!
//! Called by post.rs when the mail is due and the player is in the houses'
//! acre: he appears 160 units up in the next acre toward the post office
//! (unit 15 or 0, row 7), flies in at 2 a frame to the landing spot
//! (2240, 1620), brakes (fall easing toward 8 a frame, SE 0x12B), hovers
//! below 100 units (the fall easing to a stop), drops below 40 (toward 14),
//! lands, looks about, walks the path (speed 1) to each house with mail,
//! turns to its mailbox and delivers (the mail goes in then, the box plays
//! its delivery animation), and flies up and away toward the far side
//! (1280 or 3200, 1620). When the counter was left full and the player is
//! by the post office, he instead takes off beside it and the letters go
//! in once he is gone. While looking about, turning or walking he can be
//! talked to (0x08BF + RANDOM(12); from the post office 0x08CB +
//! RANDOM(4)).
//!
//! Positions are the original's game units, placed through the player
//! houses' acre (mailbox 0 stands at (2140, 1420)). Height eases as the
//! actor's does: vertical speed moves toward its limit by `gravity` a
//! frame (Actor_position_speed_set).
use bevy::prelude::*;

use crate::character::{NpcClip, NpcGait};
use crate::dialogue::DialogueState;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::Facing;
use crate::player::Player;
use crate::post::{Courier, CourierCall, Post};
use crate::states::GameplaySet;
use crate::town::Town;

const UNIT: f32 = 40.0;
/// ALL_NPC_NUM + (SP_NPC_POST_MAN - SP_NPC_START); l_sp_actor_name sound 3.
const PETE_NPC: u32 = 238 + 17;
const PETE_SOUND: u8 = 3;
/// aPMAN_mailbox_pos[0]: where house 0's mailbox stands.
const MAILBOX0: Vec2 = Vec2::new(2140.0, 1420.0);
const LANDING: Vec2 = Vec2::new(2240.0, 1620.0);
/// aPMAN_move_pos.
const PATH: [Vec2; 7] = [
    Vec2::new(2172.0, 1452.0),
    Vec2::new(2308.0, 1452.0),
    Vec2::new(2172.0, 1732.0),
    Vec2::new(2308.0, 1732.0),
    Vec2::new(2180.0, 1620.0),
    Vec2::new(2300.0, 1620.0),
    Vec2::new(2240.0, 1620.0),
];
/// aPMAN_set_move_idx: next point by (house, point now).
const NEXT: [[usize; 7]; 4] = [
    [0, 0, 4, 4, 0, 1, 4],
    [1, 1, 5, 5, 1, 1, 5],
    [4, 5, 2, 2, 2, 2, 2],
    [4, 5, 3, 3, 3, 3, 3],
];
const EXITS: [Vec2; 2] = [Vec2::new(1280.0, 1620.0), Vec2::new(3200.0, 1620.0)];
/// DEG2SHORT_ANGLE2(5.625) a frame.
const TURN: f32 = 5.625;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Act {
    Enter,
    Break,
    Hover,
    Landing,
    /// Looking about, then the action after.
    Kyoro(Next),
    Turn,
    Walk,
    DeliveryTurn,
    Delivery,
    TalkTurn,
    Talk,
    FlyUp,
    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Next {
    Turn,
    Walk,
    FlyUp,
}

#[derive(Component)]
struct Pete {
    from_post_office: bool,
    act: Act,
    /// Game-unit position (x, z) and height above the ground.
    at: Vec2,
    height: f32,
    yaw: f32,
    speed: f32,
    max_speed: f32,
    accel: f32,
    vy: f32,
    vy_limit: f32,
    gravity: f32,
    /// The house being delivered to (only the player's, 0, has mail).
    delivery: Option<usize>,
    now_idx: usize,
    move_idx: usize,
    exit: usize,
    /// The flyaway clip's frame (it lifts at 8).
    clip_time: f32,
    age: f32,
}

/// Game units to the scene, through the houses' acre.
#[derive(Resource, Clone, Copy)]
struct Origin(Vec2);

impl Origin {
    fn world(self, p: Vec2) -> Vec2 {
        self.0 + p / UNIT
    }
    fn game(self, w: Vec2) -> Vec2 {
        (w - self.0) * UNIT
    }
}

fn origin(town: &Town) -> Option<Origin> {
    let cell = town.retail.as_ref()?.mailboxes[0]?;
    let m = crate::disc_town::mailbox_position(cell);
    Some(Origin(Vec2::new(m.x, m.z) - MAILBOX0 / UNIT))
}

/// aPMAN_check_delivery: house 0 when the store holds letters.
fn first_delivery(post: &Post) -> Option<usize> {
    (!post.store.is_empty()).then_some(crate::home::index())
}

fn chase_angle(from: f32, to: f32, step: f32) -> (f32, bool) {
    let d =
        (to - from + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
    if d.abs() <= step {
        (to, true)
    } else {
        (from + step * d.signum(), false)
    }
}

fn yaw_to(from: Vec2, to: Vec2) -> f32 {
    let d = to - from;
    d.x.atan2(d.y)
}

#[allow(clippy::too_many_arguments)]
fn spawn(
    mut commands: Commands,
    mut call: ResMut<CourierCall>,
    town: Res<Town>,
    post: Res<Post>,
    mut meshes: ResMut<Assets<Mesh>>,
    existing: Query<(), With<Pete>>,
) {
    let Some(courier) = call.0 else { return };
    if !existing.is_empty() {
        return;
    }
    let Some(o) = origin(&town) else {
        call.0 = None;
        return;
    };
    let retail = town.retail.as_ref();
    let houses = retail.and_then(|r| r.mailboxes[0]);
    let post_office = retail
        .and_then(|r| r.buildings.iter().find(|b| b.item == 0x5808))
        .map(|b| b.origin);
    let (Some((hx, hz, _, _)), Some(po)) = (houses, post_office) else {
        call.0 = None;
        return;
    };
    let po_acre = Town::cell_indices_of(po).map_or(hx, |(ax, ..)| ax);
    let (at, from_post_office) = match courier {
        Courier::Pete => {
            // bx_add / ux_table: the next acre toward the post office.
            let (ax, ux) = if hx >= po_acre {
                (hx.saturating_sub(1), 15)
            } else {
                (hx + 1, 0)
            };
            let w = Town::cell_to_world(ax, hz, ux, 7);
            (o.game(Vec2::new(w.x, w.z)), false)
        }
        Courier::PeteFromPostOffice => {
            let w = po - Vec3::X * 3.0;
            (o.game(Vec2::new(w.x, w.z)), true)
        }
    };
    let pete_acre = Town::cell_indices_of(Vec3::new(o.world(at).x, 0.0, o.world(at).y))
        .map_or(hx, |(ax, ..)| ax);
    let mut pete = Pete {
        from_post_office,
        act: Act::Enter,
        at,
        height: if from_post_office { 0.0 } else { 160.0 },
        yaw: yaw_to(at, LANDING),
        speed: 2.0,
        max_speed: 2.0,
        accel: 0.2,
        vy: 0.0,
        vy_limit: 0.0,
        gravity: 0.0,
        delivery: first_delivery(&post),
        now_idx: 6,
        move_idx: 6,
        // post_office_direct: toward the post office's side.
        exit: usize::from(pete_acre < po_acre),
        clip_time: 0.0,
        age: 0.0,
    };
    if from_post_office {
        pete.act = Act::FlyUp;
        pete.speed = 0.0;
        pete.max_speed = 0.0;
    }
    let mut clip = NpcClip::default();
    clip.start(
        if from_post_office {
            "cKF_ba_r_npc_1_flyaway1"
        } else {
            "cKF_ba_r_npc_1_fly1"
        },
        from_post_office,
        8.0,
    );
    let w = o.world(at);
    let ground = town.ground_y(Vec3::new(w.x, 0.0, w.y));
    commands.spawn((
        Name::new("Pete"),
        pete,
        clip,
        NpcGait::default(),
        crate::character::NpcModel(PETE_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_xyz(w.x, ground + 1.0, w.y),
    ));
    commands.insert_resource(o);
    call.0 = None;
    info!("Pete is coming ({courier:?})");
}

#[allow(clippy::too_many_arguments)]
fn fly(
    mut commands: Commands,
    time: Res<Time>,
    town: Res<Town>,
    origin: Option<Res<Origin>>,
    dialogue: Res<DialogueState>,
    mut pete: Query<(
        Entity,
        &mut Transform,
        &mut Pete,
        &mut NpcClip,
        &mut NpcGait,
    )>,
    player: Query<&Transform, (With<Player>, Without<Pete>)>,
    mut boxes: Query<&mut crate::mailbox::MailboxFlag>,
    (mut post, mut house_box): (ResMut<Post>, ResMut<crate::mail::HouseMailbox>),
    mut last: Local<Option<Act>>,
) {
    let (Some(o), Ok((entity, mut transform, mut p, mut clip, mut gait))) =
        (origin, pete.single_mut())
    else {
        return;
    };
    let o = *o;
    let ticks = time.delta_secs() * 60.0;
    p.age += time.delta_secs();
    let talking = dialogue.active && dialogue.speaker == Some(entity);
    let step = TURN.to_radians() * ticks;
    let set_clip = |clip: &mut NpcClip, name: &'static str, once: bool| {
        if clip.name != Some(name) {
            clip.start(name, once, 8.0);
        }
    };

    match p.act {
        Act::Enter => {
            let (yaw, _) = chase_angle(p.yaw, yaw_to(p.at, LANDING), step);
            p.yaw = yaw;
            if p.at.distance(LANDING) < 40.0 {
                // aPMAN_break_init.
                p.act = Act::Break;
                p.max_speed = 0.05;
                p.accel = 0.05;
                p.vy_limit = -8.0;
                p.gravity = 0.8;
                set_clip(&mut clip, "cKF_ba_r_npc_1_landing1", false);
                let w = o.world(p.at);
                crate::sound::se_at(0x12B, Vec3::new(w.x, transform.translation.y, w.y));
            }
        }
        Act::Break => {
            if p.height < 100.0 {
                p.act = Act::Hover;
                p.vy_limit = 0.0;
                p.gravity = 0.5;
                set_clip(&mut clip, "cKF_ba_r_npc_1_landing2", false);
            }
        }
        Act::Hover => {
            // The original just reaches 40 as the fall eases out; never stall.
            if p.height < 40.0 || p.vy > -0.5 {
                p.act = Act::Landing;
                p.vy_limit = -14.0;
                p.gravity = 1.4;
                set_clip(&mut clip, "cKF_ba_r_npc_1_landing3", true);
            }
        }
        Act::Landing => {
            if p.height <= 0.0 {
                p.speed = 0.0;
                p.max_speed = 0.0;
            }
            if clip.done && p.height <= 0.0 {
                let next = if p.delivery.is_some() {
                    Next::Turn
                } else {
                    Next::FlyUp
                };
                p.act = Act::Kyoro(next);
                p.vy_limit = -8.0;
                p.gravity = 0.8;
                clip.start("cKF_ba_r_npc_1_kyoro1", true, 8.0);
            }
        }
        Act::Kyoro(next) => {
            if clip.done {
                match next {
                    Next::Turn => {
                        p.move_idx = p.delivery.map_or(6, |d| NEXT[d][p.now_idx]);
                        p.act = Act::Turn;
                    }
                    Next::Walk => {
                        p.act = Act::Walk;
                        p.max_speed = 1.0;
                        p.accel = 0.1;
                    }
                    Next::FlyUp => {
                        p.act = Act::FlyUp;
                        p.clip_time = 0.0;
                        clip.start("cKF_ba_r_npc_1_flyaway1", true, 8.0);
                    }
                }
                if p.act != Act::FlyUp {
                    clip.stop(8.0);
                }
            }
        }
        Act::Turn => {
            let (yaw, done) = chase_angle(p.yaw, yaw_to(p.at, PATH[p.move_idx]), step);
            p.yaw = yaw;
            if done {
                p.act = Act::Kyoro(Next::Walk);
                clip.start("cKF_ba_r_npc_1_kyoro1", true, 8.0);
            }
        }
        Act::Walk => {
            let target = PATH[p.move_idx];
            p.yaw = chase_angle(p.yaw, yaw_to(p.at, target), step).0;
            if p.at.distance(target) < p.speed.max(1.0) * ticks {
                p.at = target;
                p.now_idx = p.move_idx;
                match p.delivery {
                    Some(d) if d == p.now_idx => {
                        p.act = Act::DeliveryTurn;
                        p.speed = 0.0;
                        p.max_speed = 0.0;
                    }
                    Some(d) => p.move_idx = NEXT[d][p.now_idx],
                    None => {}
                }
            }
        }
        Act::DeliveryTurn => {
            let d = p.delivery.unwrap_or(0);
            let mbox = [
                MAILBOX0,
                Vec2::new(2340.0, 1420.0),
                Vec2::new(2140.0, 1700.0),
                Vec2::new(2340.0, 1700.0),
            ][d];
            let (yaw, done) = chase_angle(p.yaw, yaw_to(p.at, mbox), step);
            p.yaw = yaw;
            if done {
                // aPMAN_delivery_init: the letters go in, the box opens.
                p.act = Act::Delivery;
                crate::post::deliver_to_house(&mut post, &mut house_box.0);
                if let Some(mut b) = boxes.iter_mut().find(|b| b.house == d) {
                    b.action = Some((crate::mailbox::BoxAction::Delivery, 1.0));
                }
                p.delivery = None;
                clip.start("cKF_ba_r_npc_1_delivery1", true, 8.0);
            }
        }
        Act::Delivery => {
            if clip.done {
                p.act = Act::FlyUp;
                p.clip_time = 0.0;
                clip.start("cKF_ba_r_npc_1_flyaway1", true, 8.0);
            }
        }
        Act::TalkTurn => {
            if let Ok(pl) = player.single() {
                let to = o.game(Vec2::new(pl.translation.x, pl.translation.z));
                let (yaw, done) = chase_angle(p.yaw, yaw_to(p.at, to), step);
                p.yaw = yaw;
                if done {
                    p.act = Act::Talk;
                }
            }
        }
        Act::Talk => {
            if !talking {
                p.act = if p.delivery.is_some() {
                    Act::Turn
                } else {
                    Act::Kyoro(Next::FlyUp)
                };
                if p.delivery.is_none() {
                    clip.start("cKF_ba_r_npc_1_kyoro1", true, 8.0);
                }
            }
        }
        Act::FlyUp => {
            let before = p.clip_time;
            p.clip_time += 0.5 * ticks;
            if before < 8.0 && p.clip_time >= 8.0 {
                p.vy = 4.0;
                p.vy_limit = -4.0;
                p.gravity = 0.23;
            } else if p.clip_time >= 8.0 && p.vy < 0.0 {
                p.vy_limit = 1.0;
                p.gravity = 0.1;
            }
            if clip.done {
                if p.from_post_office {
                    // EXIT2: gone; the letters go in now (aPMAN_actor_save).
                    crate::post::deliver_to_house(&mut post, &mut house_box.0);
                    commands.entity(entity).despawn();
                    commands.remove_resource::<Origin>();
                    info!("Pete flew off from the post office");
                    return;
                }
                p.act = Act::Exit;
                set_clip(&mut clip, "cKF_ba_r_npc_1_fly1", false);
            }
        }
        Act::Exit => {
            let target = EXITS[p.exit];
            p.yaw = chase_angle(p.yaw, yaw_to(p.at, target), step).0;
            p.vy_limit = if p.height > 155.0 { 0.0 } else { 1.0 };
            if p.height > 140.0 {
                p.max_speed = 2.0;
                p.accel = 0.2;
            }
            // Culled once well out of the player's way.
            let far = player.single().is_ok_and(|pl| {
                let w = o.world(p.at);
                (Vec2::new(pl.translation.x, pl.translation.z) - w).length() > 24.0
            });
            if far || p.at.distance(target) < 8.0 || p.age > 90.0 {
                commands.entity(entity).despawn();
                commands.remove_resource::<Origin>();
                info!("Pete flew away");
                return;
            }
        }
    }
    if *last != Some(p.act) {
        debug!(
            "Pete: {:?} at {:?} h{:.0} age {:.1}",
            p.act, p.at, p.height, p.age
        );
        *last = Some(p.act);
    }
    // Talk interrupts looking about, turning or walking (talk_permit).
    if talking && matches!(p.act, Act::Kyoro(_) | Act::Turn | Act::Walk) {
        p.act = Act::TalkTurn;
        p.speed = 0.0;
        clip.stop(8.0);
    }

    // Movement: speed eases to its limit; height eases as the actor's does.
    let accel = p.accel * ticks;
    p.speed = if p.speed < p.max_speed {
        (p.speed + accel).min(p.max_speed)
    } else {
        (p.speed - accel).max(p.max_speed)
    };
    let dir = Vec2::new(p.yaw.sin(), p.yaw.cos());
    let step_len = p.speed * ticks;
    p.at += dir * step_len;
    let g = p.gravity * ticks;
    p.vy = if p.vy < p.vy_limit {
        (p.vy + g).min(p.vy_limit)
    } else {
        (p.vy - g).max(p.vy_limit)
    };
    p.height = (p.height + p.vy * ticks).max(0.0);
    gait.0 = u8::from(p.act == Act::Walk && p.speed > 0.05);
    let w = o.world(p.at);
    let ground = town.ground_y(Vec3::new(w.x, 0.0, w.y));
    transform.translation = Vec3::new(w.x, ground + 1.0 + p.height / UNIT, w.y);
    transform.rotation = Quat::from_rotation_y(p.yaw);
}

/// aPMAN_set_talk_info_talk_check.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<crate::game_disc::GameDisc>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    mut dialogue: ResMut<DialogueState>,
    identity: Res<crate::save::PlayerIdentity>,
    player: Query<(&Transform, &Facing), With<Player>>,
    pete: Query<(Entity, &Transform, &Pete)>,
    mut seed: Local<u32>,
) {
    let Some(text) = disc.as_ref().and_then(|d| d.text.as_ref()) else {
        return;
    };
    let (Ok((at, facing)), Ok((entity, pete_at, p))) = (player.single(), pete.single()) else {
        return;
    };
    if !focus.is_world()
        || !actions.just_pressed(Action::Interact)
        || !matches!(p.act, Act::Kyoro(_) | Act::Turn | Act::Walk)
    {
        return;
    }
    let delta = (pete_at.translation - at.translation).with_y(0.0);
    if delta.length() > 1.6 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
    let roll = (*seed >> 16) as usize;
    let msg = if p.from_post_office {
        0x08CB + roll % 4
    } else {
        0x08BF + roll % 12
    };
    let ctx = crate::messages::MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Pete".into(),
        free: vec![String::new(); 20],
        items: vec![String::new(); 5],
        item_articles: vec![0; 5],
        ..default()
    };
    let (pages, choice) = text.talk(msg, &ctx, &mut |_| 0);
    dialogue.begin_special(
        entity,
        "Pete",
        (PETE_NPC, PETE_SOUND),
        pages,
        choice.map(|c| c.labels),
    );
    *focus = InputFocus::Dialogue;
    info!("Pete message {msg:#06x}");
}

pub struct PostmanPlugin;

impl Plugin for PostmanPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn, fly).chain().in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
