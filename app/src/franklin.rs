//! Franklin the turkey (`SP_NPC_TURKEY`, `ac_ev_turkey.c`) on the Harvest Festival.
//!
//! He is out while the original scheduler says `mEv_EVENT_HARVEST_FESTIVAL_FRANKLIN` is active (the fourth Thursday of
//! November, 15:00-20:00), hidden in a free unit among obstacles (`make_actor_in_free_block_hide`: a unit of
//! `mNpc_GetMakeUtNuminBlock_hide_hard_area`, i.e. one with obstacles around it). The port picks a clear cell of a day's
//! acre with at least two blocked neighbours.
//!
//! Talking: within 55 units (1.4 cells) of him he speaks to the player by himself (`mDemo_TYPE_SPEAK`). The first talk
//! of the event explains the feast in four messages (`0x3BFE..=0x3C01`); a later one starts with `0x3C05 + present`
//! (`aETKY_Find_Reac`). Then, with a knife and fork (`ITM_KNIFE_AND_FORK`) in the pockets, he says `0x3C11 + present`,
//! takes it and gives the present; without one he answers `0x3C02 + RANDOM(3)`. The present is a random one of the
//! twelve he has not given yet (`aETKY_DecidePresent`, the list restarts when all are given; the bitfield is saved).
//! After each talk he walks to another hidden unit (`turkey_behind`), where he is a fresh actor again.
//!
//! Not built: his walking and look-around animations (he stands), the sound of the hiding, the guide arrow
//! (`turkey_guide_arrow`), the `0x3C1D` chatter for a talk with him after he already spoke (a relocated Franklin has not
//! spoken again), and other players.

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::town_gen::{ACRE_DEPTH, ACRE_WIDTH};
use serde::{Deserialize, Serialize};

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::input::{InputFocus, InteractSet};
use crate::interaction::PlayerInventory;
use crate::messages::MsgContext;
use crate::player::Player;
use crate::save::LoadedSave;
use crate::shop::ShopLocation;
use crate::shop_nook::handover::{HandOver, Item, Party, Request};
use crate::states::GameplaySet;
use crate::town::Town;

/// ALL_NPC_NUM + (SP_NPC_TURKEY - SP_NPC_START).
const FRANKLIN_NPC: u32 = 238 + 141;
/// `l_sp_actor_name` sound id of `SP_NPC_TURKEY`.
const FRANKLIN_SOUND: u8 = 2;
/// `ITM_KNIFE_AND_FORK`.
const KNIFE_AND_FORK: u16 = 0x2530;
/// `aETKY_present_table`: ten pieces of the turkey set, a carpet and a wall.
const PRESENTS: [u16; 12] = [
    0x32d0, 0x32e0, 0x32e4, 0x32e8, 0x32ec, 0x32f0, 0x32f4, 0x3300, 0x3308, 0x3310, 0x2642, 0x2742,
];
/// `aETKY_ALL_BITS`.
const ALL_BITS: u16 = (1 << PRESENTS.len()) - 1;
/// The distance at which he speaks first (55 units of 40 per cell).
const SPEAK_RANGE: f32 = 55.0 / 40.0;

/// The event's save area (`aEv_turkey_save_c`) for the one player.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FranklinEvent {
    /// The game day this belongs to.
    pub day: i64,
    /// `given_present_bitfield`.
    pub given: u16,
}

#[derive(Component)]
struct Franklin {
    day: i64,
    /// `present_idx`.
    present: usize,
    /// Talks so far this event (`aEv_turkey_common_c._00`); also seeds the next hiding place.
    talks: i64,
}

/// Startup: the saved event area.
fn apply_loaded(loaded: Res<LoadedSave>, mut event: ResMut<FranklinEvent>) {
    if let Some(save) = loaded.0.as_ref() {
        *event = save.franklin_event.clone().unwrap_or_default();
    }
}

/// `aETKY_DecidePresent`.
fn decide_present(given: &mut u16, rand: &mut crate::game_rand::GameRand) -> usize {
    if *given & ALL_BITS == ALL_BITS {
        *given = 0;
    }
    let left: Vec<usize> = (0..PRESENTS.len())
        .filter(|i| *given >> i & 1 == 0)
        .collect();
    left[rand.0.random(left.len() as i32) as usize]
}

/// Cheap integer mixer for the per-day picks.
fn mix(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ce_b9fe_1a85_1a85);
    x ^ (x >> 33)
}

/// A hidden unit: clear ground with at least two blocked neighbours, in an acre picked from `seed`.
fn hiding_spot(town: &Town, seed: u64) -> Option<Vec3> {
    let blocked = |p: Vec3| !town.is_clear_ground(p) || town.in_retail_obstacle(p);
    let (x_acres, z_acres) = (ACRE_WIDTH - 2, ACRE_DEPTH - 3);
    let n = x_acres * z_acres;
    let first = mix(seed) as usize % n;
    for step in 0..n {
        let k = (first + step) % n;
        let (ax, az) = (1 + k % x_acres, 2 + k / x_acres);
        let mut spots = Vec::new();
        for cz in 1..15 {
            for cx in 1..15 {
                let at = Town::cell_to_world(ax, az, cx, cz);
                if blocked(at) {
                    continue;
                }
                let around = [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)]
                    .iter()
                    .filter(|(dx, dz)| blocked(at + Vec3::new(*dx, 0.0, *dz)))
                    .count();
                if around >= 2 {
                    spots.push(at);
                }
            }
        }
        if !spots.is_empty() {
            let at = spots[mix(seed ^ 0x55) as usize % spots.len()];
            return Some(at.with_y(town.ground_y(at)));
        }
    }
    None
}

/// Brings him out for the afternoon and sends him off when the event ends.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Franklin)>,
    mut event: ResMut<FranklinEvent>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let out = today.is_active(ev::HARVEST_FESTIVAL_FRANKLIN, now.hour)
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    for (entity, f) in &existing {
        if !out || f.day != now.day_number {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    // A new festival starts a new save area (`mEv_reserve_save_area`).
    if event.day != now.day_number {
        *event = FranklinEvent {
            day: now.day_number,
            given: 0,
        };
    }
    let Some(at) = hiding_spot(&town, now.day_number as u64) else {
        return;
    };
    let present = decide_present(&mut event.given, &mut rand);
    commands.spawn((
        Name::new("Franklin"),
        Franklin {
            day: now.day_number,
            present,
            talks: 0,
        },
        NpcClip::default(),
        NpcGait::default(),
        crate::event_body::EventBody::default(),
        NpcModel(FRANKLIN_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y),
    ));
    info!(
        "Franklin hides at {at:?}, present {present} ({})",
        crate::items::name(PRESENTS[present])
    );
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// Explanation message `n` of four (`0x3BFE + n`) is up.
    Explain(u16),
    /// The find-reaction line is up; the fork check follows.
    Find,
    /// `0x3C11 + present` is up; the fork goes to him next.
    Fork,
    /// The fork is crossing.
    ForkHanding,
    /// The present is crossing.
    PresentHanding,
    /// A closing line is up.
    Close,
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
}

fn context(identity: &crate::save::PlayerIdentity, item: Option<u16>) -> MsgContext {
    let now = crate::clock::now();
    let name = item.map(crate::items::name).unwrap_or_default();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Franklin".into(),
        free: vec![String::new(); 20],
        items: vec![name; 5],
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

/// Shows disc message `msg` in Franklin's voice.
fn say(disc: &GameDisc, dialogue: &mut DialogueState, actor: Entity, ctx: &MsgContext, msg: u16) {
    let Some(text) = disc.text.as_ref() else {
        return;
    };
    let (pages, asked) = text.talk_asking(usize::from(msg), ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "Franklin",
        (FRANKLIN_NPC, FRANKLIN_SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Franklin message {msg:#06x}");
}

/// He turns to face the player while they talk.
fn face(
    time: Res<Time>,
    flow: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<Franklin>)>,
    mut franklin: Query<&mut Transform, With<Franklin>>,
) {
    let (Ok(at), Ok(mut transform)) = (player.single(), franklin.single_mut()) else {
        return;
    };
    if matches!(flow.step, Step::Idle) {
        return;
    }
    let to = (at.translation - transform.translation).with_y(0.0);
    if to.length() > 0.01 {
        let want = Quat::from_rotation_y(to.x.atan2(to.z));
        transform.rotation = transform
            .rotation
            .slerp(want, (time.delta_secs() * 8.0).min(1.0));
    }
}

/// `aETKY_SetKnifeForkSequence`: the fork check after the explanation.
fn fork_check(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    flow: &mut Talk,
    actor: Entity,
    inventory: &PlayerInventory,
    identity: &crate::save::PlayerIdentity,
    me: &Franklin,
    rand: &mut crate::game_rand::GameRand,
) {
    if inventory.pockets.pockets.contains(&KNIFE_AND_FORK) {
        let ctx = context(identity, Some(PRESENTS[me.present]));
        say(disc, dialogue, actor, &ctx, 0x3C11 + me.present as u16);
        flow.step = Step::Fork;
    } else {
        let ctx = context(identity, None);
        say(
            disc,
            dialogue,
            actor,
            &ctx,
            0x3C02 + rand.0.random(3) as u16,
        );
        flow.step = Step::Close;
    }
}

/// The talk, as `aETKY_TalkRequest` and the talk procs.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    town: Res<Town>,
    mut focus: ResMut<InputFocus>,
    player: Query<&Transform, (With<Player>, Without<Franklin>)>,
    mut franklin: Query<(Entity, &mut Transform, &mut Franklin)>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut event: ResMut<FranklinEvent>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, mut actor_at, mut me)) = franklin.single_mut() else {
        return;
    };
    let ctx = context(&identity, None);
    let answered = if dialogue.speaker == Some(actor) {
        dialogue.external_result.take()
    } else {
        None
    };
    if answered.is_some() {
        *focus = InputFocus::Dialogue;
        match std::mem::take(&mut flow.step) {
            Step::Explain(n) if n < 3 => {
                say(&disc, &mut dialogue, actor, &ctx, 0x3BFE + n + 1);
                flow.step = Step::Explain(n + 1);
            }
            Step::Explain(_) | Step::Find => {
                fork_check(
                    &disc,
                    &mut dialogue,
                    &mut flow,
                    actor,
                    &inventory,
                    &identity,
                    &me,
                    &mut rand,
                );
            }
            Step::Fork => {
                // `aETKY_DeleteKnifeInPlayerBag` and the hand-over of the fork.
                *dialogue = DialogueState::default();
                inventory.remove(KNIFE_AND_FORK);
                handover.start(Request {
                    item: Item::Goods(KNIFE_AND_FORK),
                    giver: Party::Player,
                    taker: Party::Npc(actor),
                });
                flow.step = Step::ForkHanding;
            }
            Step::Close | Step::ForkHanding | Step::PresentHanding | Step::Idle => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
                relocate(&town, &mut actor_at, &mut me);
            }
        }
        return;
    }
    match flow.step {
        Step::ForkHanding | Step::PresentHanding => {
            if handover.busy() {
                *focus = InputFocus::Dialogue;
            } else if matches!(flow.step, Step::ForkHanding) {
                // `aETKY_Give_You_Present`: the piece into a free pocket, then it changes hands.
                let present = PRESENTS[me.present];
                inventory.add(present);
                event.given |= 1 << me.present;
                info!(
                    "Franklin gives {present:#06x} ({})",
                    crate::items::name(present)
                );
                handover.start(Request {
                    item: Item::Goods(present),
                    giver: Party::Npc(actor),
                    taker: Party::Player,
                });
                flow.step = Step::PresentHanding;
            } else {
                *focus = InputFocus::World;
                flow.step = Step::Idle;
                relocate(&town, &mut actor_at, &mut me);
            }
            return;
        }
        Step::Idle => {}
        _ => return,
    }
    // He speaks first when the player comes near (`mDemo_TYPE_SPEAK`, 55 units).
    if !focus.is_world() {
        return;
    }
    let Ok(at) = player.single() else {
        return;
    };
    let delta = (actor_at.translation - at.translation).with_y(0.0);
    if delta.length() > SPEAK_RANGE {
        return;
    }
    *focus = InputFocus::Dialogue;
    me.talks += 1;
    if me.talks > 1 {
        // `aETKY_GetTalkStartStatus`: he has spoken before this event (`_00 != 0`).
        say(
            &disc,
            &mut dialogue,
            actor,
            &ctx,
            0x3C05 + me.present as u16,
        );
        flow.step = Step::Find;
    } else {
        say(&disc, &mut dialogue, actor, &ctx, 0x3BFE);
        flow.step = Step::Explain(0);
    }
}

/// `walk_actor_at_wade_hide`: he hides in another unit after a talk.
fn relocate(town: &Town, at: &mut Transform, me: &mut Franklin) {
    let seed = (me.day as u64)
        .wrapping_mul(31)
        .wrapping_add(me.talks as u64 * 7919);
    if let Some(spot) = hiding_spot(town, seed) {
        at.translation = spot + Vec3::Y;
        info!("Franklin hides again at {spot:?}");
    }
}

pub struct FranklinPlugin;

impl Plugin for FranklinPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .init_resource::<FranklinEvent>()
            .add_systems(Startup, apply_loaded)
            .add_systems(
                Update,
                visit.after(crate::event_today::refresh).in_set(GameplaySet),
            )
            .add_systems(Update, face.in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
