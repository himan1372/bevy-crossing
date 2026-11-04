//! Gulliver (`SP_NPC_EV_DOZAEMON`), the seagull castaway who washes up on the beach.
//!
//! He is out while the original scheduler says `mEv_EVENT_DOZAEMON` is active (06:00-22:00 on the week's saved
//! Gulliver day, see `event_schedule`), until his errand is done (`gulliver_completed`).
//!
//! Placement (`make_actor_in_seaside_block` / `search_seaside_unit` in `ac_event_manager.c`): the bottom playable
//! row of blocks, block x = 1 + (seed + day + sec) % (xmax - 2), and a random wave-attribute unit of that block
//! with x and z in 1..=14 (`mFI_GetWaveUtinBlock`). The original rolls `RANDOM` and uses the clock's seconds, and
//! saves the chosen place; the port derives the pick from the game day instead, so a reload puts him in the same
//! spot. Waves are not walkable, so he stands on the nearest clear ground to that unit.
//!
//! Talking (`ac_ev_dozaemon_move.c_inc`): collapsed on the sand he answers with `0x23F9 + RANDOM(5)`, or with
//! probability 0.35 with `0x23F8`, gets up and says `0x23FE + RANDOM(6)` (the WAKEUP flag), then offers the gift:
//! `0x2408` when no pocket is free, else `0x2404` and a random furniture piece of the Jonason list into the pockets
//! (GIVE flag, `dozaemon_completed`), `0x2406` / `0x2407`. Once awake, a talk before the gift offers it again
//! (`0x2409` / `0x2405`, then `0x2407`) and a talk after it gives `0x240A + RANDOM(6)`. The three flags are kept by
//! `TodayEvents` and saved with the weekly state.
//!
//! Port adaptations: no lying / getting-up animation and no camera, the forced talk follows `0x23F8` directly, and
//! the hand-over plays after the gift message (as with K.K. Slider). A free pocket means an empty slot; the
//! original also asks for normal condition.
//!
//! Not built yet: the dock block exclusion (`dock_block` is not modelled).

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::town_gen::{ACRE_DEPTH, ACRE_WIDTH};

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::messages::MsgContext;
use crate::player::Player;
use crate::shop::ShopLocation;
use crate::shop_nook::handover::{HandOver, Item, Party, Request};
use crate::states::GameplaySet;
use crate::town::Town;
use rustimal_logic::gulliver::save_flag;
use rustimal_logic::inventory::EMPTY_NO;

/// ALL_NPC_NUM + (SP_NPC_EV_DOZAEMON - SP_NPC_START).
const GULLIVER_NPC: u32 = 238 + 100;
/// `l_sp_actor_name` sound id of `SP_NPC_EV_DOZAEMON`.
const GULLIVER_SOUND: u8 = 2;
/// `mSP_LIST_JONASON`, the furniture list of his gift.
const JONASON_LIST: usize = 11;

#[derive(Component)]
struct Gulliver {
    /// The game day he came out for.
    day: i64,
}

/// Cheap integer mixer for the per-day picks.
fn mix(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ce_b9fe_1a85_1a85);
    x ^ (x >> 33)
}

/// The wave units of playable column `ax` in the bottom row, with x and z in 1..=14.
fn wave_units(town: &Town, ax: usize) -> Vec<(usize, usize)> {
    let Some(ground) = town.ground.as_ref() else {
        return Vec::new();
    };
    let az = ACRE_DEPTH - 1;
    let mut out = Vec::new();
    for cz in 1..15 {
        for cx in 1..15 {
            let at = Town::cell_to_world(ax, az, cx, cz);
            if ground.is_wave(at.x, at.z) {
                out.push((cx, cz));
            }
        }
    }
    out
}

/// Where he washes up on `day`: the nearest clear ground to a wave unit of the day's block.
pub(crate) fn beach_spot(town: &Town, day: i64) -> Option<Vec3> {
    let h = mix(day as u64);
    // The columns, starting from the day's pick, until one has waves (`search_seaside_unit` retries blocks).
    for step in 0..ACRE_WIDTH {
        let ax = (h as usize % ACRE_WIDTH + step) % ACRE_WIDTH;
        let waves = wave_units(town, ax);
        if waves.is_empty() {
            continue;
        }
        let (cx, cz) = waves[(mix(h) as usize) % waves.len()];
        let wave = Town::cell_to_world(ax, ACRE_DEPTH - 1, cx, cz);
        let mut best: Option<(f32, Vec3)> = None;
        for dz in -10i32..=4 {
            for dx in -10i32..=10 {
                let at = wave + Vec3::new(dx as f32, 0.0, dz as f32);
                if !town.is_clear_ground(at) || town.in_retail_obstacle(at) {
                    continue;
                }
                let d = (at - wave).length();
                if best.is_none_or(|(b, _)| d < b) {
                    best = Some((d, at));
                }
            }
        }
        if let Some((_, at)) = best {
            return Some(at.with_y(town.ground_y(at)));
        }
    }
    None
}

/// Brings him out for the day and sends him off when the event ends.
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Gulliver)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let out = today.is_active(ev::DOZAEMON, now.hour)
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    for (entity, g) in &existing {
        if !out || g.day != now.day_number {
            commands.entity(entity).despawn();
        }
    }
    // `aEDZ_actor_ct` deletes him when his errand is done; one already out stays until the event ends.
    if !out || !existing.is_empty() || today.gulliver_completed() {
        return;
    }
    let Some(at) = beach_spot(&town, now.day_number) else {
        return;
    };
    commands.spawn((
        Name::new("Gulliver"),
        Gulliver {
            day: now.day_number,
        },
        NpcClip::default(),
        NpcGait::default(),
        NpcModel(GULLIVER_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        // Facing the sea (south).
        Transform::from_translation(at + Vec3::Y),
    ));
    info!("Gulliver washed up at {at:?}");
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// `0x23F8` is up: he gets up and the forced talk follows.
    Notice,
    /// `0x23FE + n` is up: the gift offer follows.
    Forced,
    /// `0x2404` / `0x2405` is up: the gift follows.
    Offer,
    /// `0x2406` / `0x2407` is up (`Some(item)`): the piece changes hands when it ends.
    Giving(u16),
    /// A closing line is up; the talk ends with it.
    Close,
    /// The piece is crossing; the talk ends after.
    Handing,
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
        talk_name: "Gulliver".into(),
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

/// Shows disc message `msg` in Gulliver's voice.
fn say(disc: &GameDisc, dialogue: &mut DialogueState, actor: Entity, ctx: &MsgContext, msg: u16) {
    let Some(text) = disc.text.as_ref() else {
        return;
    };
    let (pages, asked) = text.talk_asking(usize::from(msg), ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "Gulliver",
        (GULLIVER_NPC, GULLIVER_SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Gulliver message {msg:#06x}");
}

/// He turns to face the player while they talk.
fn face(
    time: Res<Time>,
    flow: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<Gulliver>)>,
    mut gulliver: Query<&mut Transform, With<Gulliver>>,
) {
    let (Ok(at), Ok(mut transform)) = (player.single(), gulliver.single_mut()) else {
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

/// `mPr_GetPossessionItemSumWithCond(.., EMPTY_NO, ..)` is not zero: a pocket is free.
fn has_free_pocket(inventory: &PlayerInventory) -> bool {
    inventory.pockets.pockets.contains(&EMPTY_NO)
}

/// `aEDZ_ageru`: a random furniture piece of the Jonason list goes into the pockets; `None` when the list is
/// missing (no disc goods) or the pockets filled up meanwhile.
fn give_piece(
    inventory: &mut PlayerInventory,
    rand: &mut crate::game_rand::GameRand,
) -> Option<u16> {
    let list = crate::goods::list(crate::goods::Kind::Furniture, JONASON_LIST);
    if list.is_empty() {
        return None;
    }
    let item = list[rand.0.random(list.len() as i32) as usize];
    inventory.add(item).then_some(item)
}

/// The talk, as `aEDZ_norm_talk_request` / `aEDZ_wander_talk_request` and the talk procs.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), With<Player>>,
    gulliver: Query<(Entity, &Transform), With<Gulliver>>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut today: ResMut<TodayEvents>,
) {
    use rustimal_logic::gulliver as native;
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, actor_at)) = gulliver.single() else {
        return;
    };
    // Only his own answers; other talkers in town take theirs.
    let answered = dialogue.speaker == Some(actor) && dialogue.external_result.take().is_some();
    if answered {
        *focus = InputFocus::Dialogue;
        let ctx = context(&identity, None);
        match std::mem::take(&mut flow.step) {
            Step::Notice => {
                // `aEDZ_set_force_talk_info`.
                let flags = today.dozaemon_flags() | save_flag::WAKEUP;
                today.set_dozaemon_flags(flags);
                let msg = native::force_talk_msg(rand.0.random(6) as u8);
                say(&disc, &mut dialogue, actor, &ctx, msg);
                flow.step = Step::Forced;
            }
            Step::Forced => {
                // `aEDZ_to_ageru`.
                if has_free_pocket(&inventory) {
                    let flags = today.dozaemon_flags() | save_flag::GAVE_FIRST;
                    today.set_dozaemon_flags(flags);
                    say(&disc, &mut dialogue, actor, &ctx, native::ageru_msg(true));
                    flow.step = Step::Offer;
                } else {
                    say(&disc, &mut dialogue, actor, &ctx, native::ageru_msg(false));
                    flow.step = Step::Close;
                }
            }
            Step::Offer => {
                // `aEDZ_talk_give` then `aEDZ_ageru`: the gift, the flags, the closing line.
                match give_piece(&mut inventory, &mut rand) {
                    Some(item) => {
                        let flags = today.dozaemon_flags() | save_flag::GIVE;
                        today.set_dozaemon_flags(flags);
                        today.complete_gulliver();
                        let first = flags & save_flag::GAVE_FIRST != 0;
                        let ctx = context(&identity, Some(item));
                        say(&disc, &mut dialogue, actor, &ctx, native::give_msg(first));
                        info!("Gulliver gives {item:#06x}");
                        flow.step = Step::Giving(item);
                    }
                    None => {
                        say(&disc, &mut dialogue, actor, &ctx, native::ageru_msg(false));
                        flow.step = Step::Close;
                    }
                }
            }
            Step::Giving(item) => {
                *dialogue = DialogueState::default();
                handover.start(Request {
                    item: Item::Goods(item),
                    giver: Party::Npc(actor),
                    taker: Party::Player,
                });
                flow.step = Step::Handing;
            }
            Step::Close | Step::Handing | Step::Idle => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
                flow.step = Step::Idle;
            }
        }
        return;
    }
    if matches!(flow.step, Step::Handing) {
        if handover.busy() {
            *focus = InputFocus::Dialogue;
        } else {
            *dialogue = DialogueState::default();
            *focus = InputFocus::World;
            flow.step = Step::Idle;
        }
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
    let delta = (actor_at.translation - at.translation).with_y(0.0);
    if delta.length() > 1.6 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    let ctx = context(&identity, None);
    let flags = today.dozaemon_flags();
    if flags & save_flag::WAKEUP == 0 {
        // `aEDZ_set_norm_talk_info`.
        if rand.0.fqrand() < 0.35 {
            say(&disc, &mut dialogue, actor, &ctx, 0x23F8);
            flow.step = Step::Notice;
        } else {
            let msg = native::norm_talk_msg(1.0, rand.0.random(5) as u8);
            say(&disc, &mut dialogue, actor, &ctx, msg);
            flow.step = Step::Close;
        }
    } else if flags & save_flag::GIVE == 0 {
        // `aEDZ_set_wander_talk_info` -> `aEDZ_to_ageru2`.
        if has_free_pocket(&inventory) {
            today.set_dozaemon_flags(flags & !save_flag::GAVE_FIRST);
            say(&disc, &mut dialogue, actor, &ctx, native::ageru2_msg(true));
            flow.step = Step::Offer;
        } else {
            say(&disc, &mut dialogue, actor, &ctx, native::ageru2_msg(false));
            flow.step = Step::Close;
        }
    } else {
        let msg = native::wander_msg(rand.0.random(6) as u8);
        say(&disc, &mut dialogue, actor, &ctx, msg);
        flow.step = Step::Close;
    }
}

pub struct GulliverPlugin;

impl Plugin for GulliverPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Talk>()
            .add_systems(
                Update,
                visit.after(crate::event_today::refresh).in_set(GameplaySet),
            )
            .add_systems(Update, face.in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
