//! K.K. Slider (`SP_NPC_TOTAKEKE`), the Saturday-night musician.
//!
//! He is out while the original scheduler says `mEv_EVENT_KK_SLIDER` is active (every Saturday 20:00 to
//! 23:59, only when he is that week's visitor; see `event_schedule`). `staffroll_start` in
//! `ac_event_manager.c` puts him in block A-3, middle of the acre (`BlockOrUnit_c b = {1, 3}, u = {7, 7}`;
//! the port's playable acres start one block in, so that is acre (0, 2)).
//! Port adaptation: that cell can be under a building (the museum, in this town layout), so he stands on the
//! nearest clear ground south of it instead (buildings face south).
//!
//! Talking to him (`ac_npc_totakeke_talk.c_inc`): the greeting (first ever `+0xD`, first this evening `+1`,
//! later `+2`) and its question; yes leads to `+4` when he already gave this player a song tonight, else the
//! offer `+5`; the offer's first answer picks a song the player has not been given
//! (`totakeke::select_song`, up to three retries against the previous pick, everything given clears the
//! record), says `+10`, puts the disc in the pockets and hands it over with `+0xC` (`+0xB` when the pockets
//! are full).
//!
//! The offer's second answer opens the request window (`mLE_TYPE_REQUEST`, sixteen characters): the typed
//! text must equal a disc minidisk name exactly (padded with spaces, `mem_cmp`); a match gives that song
//! (`+8` for songs from 0x34 on, else `+9`, then the gift as above), anything else gets `+6` and no disc.
//!
//! Not built yet: the live song and the staff-roll cinematic that the original plays between the pick and
//! the gift (`mikanbox`, `majin5`, `think` ROLL states; for an unknown request he plays a random bonus
//! tune), his guitar animation (`aNPC_ANIM_ENSOU_E1`).

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::totakeke as native;

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::messages::{MsgChoice, MsgContext};
use crate::player::Player;
use crate::shop::ShopLocation;
use crate::shop_nook::handover::{HandOver, Item, Party, Request};
use crate::states::GameplaySet;
use crate::town::Town;

/// ALL_NPC_NUM + (SP_NPC_TOTAKEKE - SP_NPC_START).
const KK_NPC: u32 = 238 + 93;
/// `l_sp_actor_name` sound id of `SP_NPC_TOTAKEKE`.
const KK_SOUND: u8 = 2;
/// His stage: acre (0, 2), the middle cell.
const STAGE_ACRE: (usize, usize) = (0, 2);
const STAGE_CELL: (usize, usize) = (7, 7);

#[derive(Component)]
struct KkSlider {
    /// The game day he came out for.
    day: i64,
}

/// His spot: the original's stage cell, or the nearest clear ground when a building stands on it (the
/// museum does in some layouts, and the original places him there without checking).
fn stage(town: &Town) -> Vec3 {
    let centre = Town::cell_to_world(STAGE_ACRE.0, STAGE_ACRE.1, STAGE_CELL.0, STAGE_CELL.1);
    let mut best: Option<(f32, Vec3)> = None;
    // South only: the buildings' doors face south, and north of one is its back wall.
    for dz in 0i32..=12 {
        for dx in -12i32..=12 {
            let at = centre + Vec3::new(dx as f32, 0.0, dz as f32) * crate::town::CELL_SIZE;
            let blocked = |p: Vec3| !town.is_clear_ground(p) || town.in_retail_obstacle(p);
            // A cell of margin all round, so he does not stand against a wall.
            if [Vec3::ZERO, Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z]
                .iter()
                .any(|o| blocked(at + *o))
            {
                continue;
            }
            let d = (at - centre).length();
            if best.is_none_or(|(b, _)| d < b) {
                best = Some((d, at));
            }
        }
    }
    let at = best.map_or(centre, |(_, at)| at);
    at.with_y(town.ground_y(at))
}

/// Brings him out for the evening and sends him home after.
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &KkSlider)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut flow: ResMut<Talk>,
) {
    let now = crate::clock::now();
    let out = today.is_active(ev::KK_SLIDER, now.hour)
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    for (entity, kk) in &existing {
        if !out || kk.day != now.day_number {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    let at = stage(&town);
    commands.spawn((
        Name::new("K.K. Slider"),
        KkSlider {
            day: now.day_number,
        },
        NpcClip::default(),
        NpcGait::default(),
        NpcModel(KK_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y),
    ));
    // The evening's save area (`mEv_reserve_save_area`) starts empty.
    flow.visit = Visit::default();
    info!("K.K. Slider is on stage at {at:?}");
}

/// What he remembers of this evening (`aNTT_event_save_c.bitfield` and `_99f`).
#[derive(Default)]
struct Visit {
    /// `aNTT_FLAG_HELLO`: the player has answered his greeting tonight.
    hello: bool,
    /// `aNTT_FLAG_AIRCHECK_RECEIVED`.
    aircheck: bool,
    /// The last song picked (`_99f`), to avoid repeating it.
    prev_song: Option<u8>,
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// The greeting's question is up; its answers' own follow-ups.
    Greeting { next: Vec<Option<usize>> },
    /// The offer's question is up (`+5`).
    Offer,
    /// The request window is open.
    Typing,
    /// `+10` is up; the gift follows (`Some(item)`, or `None` when the pockets were full).
    Picked(Option<u16>),
    /// `+0xC` is up: the disc changes hands when it ends.
    Giving(u16),
    /// A closing line is up; the talk ends with it.
    Close,
    /// The disc is crossing; the talk ends after.
    Handing,
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
    context: MsgContext,
    visit: Visit,
}

fn msg(offset: u16) -> usize {
    usize::from(native::MSG_START + offset)
}

/// Shows disc message `msg`; returns the question it ends on.
fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &MsgContext,
    msg: usize,
) -> Option<(usize, MsgChoice)> {
    let text = disc.text.as_ref()?;
    let (pages, asked) = text.talk_asking(msg, ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "K.K. Slider",
        (KK_NPC, KK_SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("K.K. Slider message {msg:#06x}");
    asked
}

fn context(identity: &crate::save::PlayerIdentity) -> MsgContext {
    let now = crate::clock::now();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "K.K. Slider".into(),
        free: vec![String::new(); 20],
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

/// `aMKBC_clip_search_merody` with its retry rule (`aNTT_talk_select1`). The record is cleared first when
/// every song has been given.
fn pick_song(
    identity: &mut crate::save::PlayerIdentity,
    prev: Option<u8>,
    rand: &mut crate::game_rand::GameRand,
) -> u8 {
    let [w0, w1] = identity.kk_aircheck;
    if native::count_uncollected(w0, w1) == 0 {
        identity.kk_aircheck = [0, 0];
    }
    let [w0, w1] = identity.kk_aircheck;
    let n = native::count_uncollected(w0, w1) as i32;
    let mut song = native::select_song(w0, w1, rand.0.random(n) as u32);
    for _ in 0..3 {
        if Some(song) != prev {
            break;
        }
        song = native::select_song(w0, w1, rand.0.random(n) as u32);
    }
    song
}

/// The disc minidisk names as the editor holds them: sixteen codes, space padded (`mLE_md_name`).
fn song_name_codes(name: &str) -> [u8; 16] {
    let mut codes = [rustimal_logic::editor_keyboard::CHAR_SPACE; 16];
    for (slot, ch) in codes.iter_mut().zip(name.chars()) {
        *slot = crate::messages::ac_code_opt(ch).unwrap_or(0);
    }
    codes
}

/// `mLE_move_Wait`: the song whose name the typed text equals, if any.
fn requested_song(typed: &[u8; 16]) -> Option<u8> {
    (0..rustimal_logic::radio::MINIDISK_NUM as u8).find(|&song| {
        let name = crate::items::name(native::collection::MINIDISK_START + u16::from(song));
        !name.is_empty() && song_name_codes(&name) == *typed
    })
}

/// `aTTN_give_merody`: the disc goes into the pockets now (none free, none given), and a given one is
/// recorded as an aircheck for this player and this evening.
fn give_song(
    song: u8,
    inventory: &mut PlayerInventory,
    identity: &mut crate::save::PlayerIdentity,
    visit: &mut Visit,
) -> Option<u16> {
    let item = native::collection::MINIDISK_START + u16::from(song);
    if !inventory.add(item) {
        return None;
    }
    let [w0, w1] = identity.kk_aircheck;
    let (w0, w1) = native::collect(w0, w1, song);
    identity.kk_aircheck = [w0, w1];
    visit.aircheck = true;
    Some(item)
}

/// He turns to face the player while they talk.
fn face(
    time: Res<Time>,
    flow: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<KkSlider>)>,
    mut kk: Query<&mut Transform, With<KkSlider>>,
) {
    let (Ok(at), Ok(mut transform)) = (player.single(), kk.single_mut()) else {
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

/// The talk, as `aNTT_talk_select0` / `select1` / the gift.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), With<Player>>,
    kk: Query<(Entity, &Transform), With<KkSlider>>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    mut identity: ResMut<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut names: ResMut<crate::name_entry::NameEntry>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, actor_at)) = kk.single() else {
        return;
    };
    // `aNTT_talk_submenu2`: the window has gone; what was typed decides the song.
    if matches!(flow.step, Step::Typing) {
        let Some(typed) = names.take_request() else {
            return;
        };
        *focus = InputFocus::Dialogue;
        let text: String = typed
            .iter()
            .map(|&c| crate::messages::char_of(c).unwrap_or('?'))
            .collect::<String>()
            .trim_end()
            .to_string();
        flow.context.items[3] = text.clone();
        match requested_song(&typed) {
            Some(song) => {
                flow.context.items[2] =
                    crate::items::name(native::collection::MINIDISK_START + u16::from(song));
                let visit = &mut flow.visit;
                let given = give_song(song, &mut inventory, &mut identity, visit);
                info!(
                    "K.K. Slider: requested {text:?} is song {song}, given: {}",
                    given.is_some()
                );
                let offset = if native::request_is_long(song) { 8 } else { 9 };
                say(&disc, &mut dialogue, actor, &flow.context, msg(offset));
                flow.step = Step::Picked(given);
            }
            None => {
                // He does not know it: a random bonus tune that the cinematic would play, and no disc.
                let bonus = native::request_fallback_song(rand.0.random(3) as u8);
                info!("K.K. Slider: requested {text:?} is unknown; bonus tune {bonus}");
                say(&disc, &mut dialogue, actor, &flow.context, msg(6));
                flow.step = Step::Close;
            }
        }
        return;
    }
    // Only his own answers; other talkers in town take theirs.
    let answer = if dialogue.speaker == Some(actor) {
        dialogue.external_result.take()
    } else {
        None
    };
    if let Some(choice) = answer {
        *focus = InputFocus::Dialogue;
        let end = |dialogue: &mut DialogueState, focus: &mut InputFocus, flow: &mut Talk| {
            *dialogue = DialogueState::default();
            *focus = InputFocus::World;
            flow.step = Step::Idle;
        };
        match std::mem::take(&mut flow.step) {
            Step::Greeting { next } => {
                // `aNTT_talk_select0`: any answer counts as hello.
                flow.visit.hello = true;
                match choice {
                    0 => {
                        let offer = if flow.visit.aircheck { 4 } else { 5 };
                        let asked = say(&disc, &mut dialogue, actor, &flow.context, msg(offer));
                        flow.step = if offer == 5 && asked.is_some() {
                            Step::Offer
                        } else {
                            Step::Close
                        };
                    }
                    n => match next.get(n).copied().flatten() {
                        Some(m) => {
                            say(&disc, &mut dialogue, actor, &flow.context, m);
                            flow.step = Step::Close;
                        }
                        None => end(&mut dialogue, &mut focus, &mut flow),
                    },
                }
            }
            Step::Offer => match choice {
                0 => {
                    let song = pick_song(&mut identity, flow.visit.prev_song, &mut rand);
                    flow.visit.prev_song = Some(song);
                    let item = native::collection::MINIDISK_START + u16::from(song);
                    let name = crate::items::name(item);
                    flow.context.items[1] = name.clone();
                    flow.context.items[2] = name;
                    let visit = &mut flow.visit;
                    let given = give_song(song, &mut inventory, &mut identity, visit);
                    info!(
                        "K.K. Slider picked song {song} (disc {item:#06x}), given: {}",
                        given.is_some()
                    );
                    say(&disc, &mut dialogue, actor, &flow.context, msg(10));
                    flow.step = Step::Picked(given);
                }
                _ => {
                    // `aNTT_talk_select1`: the window for the typed request.
                    *dialogue = DialogueState::default();
                    names.begin(crate::name_entry::Kind::Request);
                    flow.step = Step::Typing;
                }
            },
            Step::Picked(Some(item)) => {
                say(&disc, &mut dialogue, actor, &flow.context, msg(0xC));
                flow.step = Step::Giving(item);
            }
            Step::Picked(None) => {
                say(&disc, &mut dialogue, actor, &flow.context, msg(0xB));
                flow.step = Step::Close;
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
            Step::Close | Step::Handing | Step::Typing | Step::Idle => {
                end(&mut dialogue, &mut focus, &mut flow)
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
    flow.context = context(&identity);
    // `aNTT_set_norm_talk_info`: the introduction first, then the evening's greeting, then the short one.
    let offset = if !identity.kk_introduced {
        identity.kk_introduced = true;
        0xD
    } else if !flow.visit.hello {
        1
    } else {
        2
    };
    let asked = say(&disc, &mut dialogue, actor, &flow.context, msg(offset));
    flow.step = match asked {
        Some((_, choice)) => Step::Greeting { next: choice.next },
        None => Step::Close,
    };
}

pub struct KkSliderPlugin;

impl Plugin for KkSliderPlugin {
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
