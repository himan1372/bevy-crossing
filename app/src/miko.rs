//! The New Year's shrine maiden (`SP_NPC_EV_MIKO`, `ac_ev_miko.c`, `ac_ev_miko_move.c_inc`) who sells the omikuji.
//!
//! She stands at unit (3, 3) of the shrine block while the scheduler says `mEv_EVENT_NEW_YEARS_DAY` is active (January 1,
//! 06:00-10:00), turned toward (+10, +10) game units, and talks only to a player who stands in front of her (within
//! 33.75 degrees of where she faces): `0x190C` offers an omikuji for 50 Bells (Yes / No), otherwise `0x1916`.
//! Yes: no free letter slot `0x190F`; under 50 Bells `0x190E`; otherwise she draws `RANDOM(4)` (great luck, bad luck,
//! money luck, goods luck: the destiny of the player is set from `event_visitors::miko_destiny`), says
//! `0x1911 + n`, takes the Bells and gives the fortune slip: a letter (`ITM_PAPER25`, header `0x72 + RANDOM(3)`, four
//! random lines from the string banks `0x2B1`, `0x2A1`, `0x2DA`, `0x2CA` + `RANDOM(16)`, and the result string
//! `0x2C1 + [2, 3, 1, 0][n]`) goes into the player's letters.
//!
//! Not built: the five visiting villagers, Tortimer's New Year speech and the shrine's decorations, her turn toward the
//! point after each talk, the destiny's effects on prices and goods.

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;
use rustimal_logic::event_visitors as native;

use crate::character::{NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::event_today::TodayEvents;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::{Facing, PlayerInventory};
use crate::mail::{Letter, Mailbox};
use crate::messages::MsgContext;
use crate::player::Player;
use crate::shop::ShopLocation;
use crate::shop_nook::handover::{HandOver, Item, Party, Request};
use crate::states::GameplaySet;
use crate::town::Town;

/// ALL_NPC_NUM + (SP_NPC_EV_MIKO - SP_NPC_START).
const MIKO_NPC: u32 = 238 + 61;
/// `l_sp_actor_name` sound id of `SP_NPC_EV_MIKO`.
const MIKO_SOUND: u8 = 2;
/// The omikuji's price.
const PRICE: u32 = 50;
/// `ITM_FORTUNE_SLIP`.
const FORTUNE_SLIP: u16 = 0x2513;
/// `ITM_PAPER25`: the stationery index of the letter.
const PAPER: u8 = 25;
/// `talk_distance` 78 game units.
const TALK_RANGE: f32 = 78.0 / 40.0;
/// Where she faces: toward (+10, +10) game units.
const FACING_YAW: f32 = std::f32::consts::FRAC_PI_4;

#[derive(Component)]
struct Miko {
    day: i64,
}

/// Brings her out on New Year's morning.
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Miko)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let out = today.is_active(ev::NEW_YEARS_DAY, now.hour)
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    for (entity, m) in &existing {
        if !out || m.day != now.day_number {
            commands.entity(entity).despawn();
        }
    }
    if !out || !existing.is_empty() {
        return;
    }
    let Some(at) = crate::tortimer::shrine_cell(&town, 3, 3) else {
        return;
    };
    commands.spawn((
        Name::new("Miko"),
        Miko {
            day: now.day_number,
        },
        NpcClip::default(),
        NpcGait::default(),
        crate::event_body::EventBody::default(),
        NpcModel(MIKO_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y).with_rotation(Quat::from_rotation_y(FACING_YAW)),
    ));
    info!("The shrine maiden is out at {at:?}");
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// A plain line is up; the talk ends with it.
    Line,
    /// `0x190C` is up: Yes / No.
    Offer,
    /// The result line is up; the slip is handed over next.
    Result(u8),
    /// The slip is crossing.
    Handing,
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
}

fn context(identity: &crate::save::PlayerIdentity) -> MsgContext {
    let now = crate::clock::now();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Katrina".into(),
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

fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    identity: &crate::save::PlayerIdentity,
    msg: u16,
) {
    let Some(text) = disc.text.as_ref() else {
        return;
    };
    let ctx = context(identity);
    let (pages, asked) = text.talk_asking(usize::from(msg), &ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "Katrina",
        (MIKO_NPC, MIKO_SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Shrine maiden message {msg:#06x}");
}

/// `aEMK_get_omikuji`: the fortune slip as a letter.
fn omikuji_letter(
    disc: &GameDisc,
    identity: &crate::save::PlayerIdentity,
    omikuji: usize,
    rand: &mut crate::game_rand::GameRand,
) -> Option<Letter> {
    const TABLE: [usize; 4] = [0x2B1, 0x2A1, 0x2DA, 0x2CA];
    const RESULT: [usize; 4] = [2, 3, 1, 0];
    let text = disc.text.as_ref()?;
    let mut ctx = context(identity);
    for (i, base) in TABLE.iter().enumerate() {
        ctx.free[i] = text.string(base + rand.0.random(16) as usize)?;
    }
    ctx.free[4] = text.string(0x2C1 + RESULT[omikuji % 4])?;
    let letter = text.letter(0x72 + rand.0.random(3) as usize, &ctx)?;
    Some(Letter {
        sender: "Katrina".into(),
        header: Some(letter.header),
        body: letter.body,
        footer: Some(letter.footer),
        paper: PAPER,
        ..default()
    })
}

/// She faces the player.
fn face(
    time: Res<Time>,
    flow: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<Miko>)>,
    mut miko: Query<&mut Transform, With<Miko>>,
) {
    let (Ok(at), Ok(mut transform)) = (player.single(), miko.single_mut()) else {
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

/// The talk, as `aEMK_talk_request` and the talk procs.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), (With<Player>, Without<Miko>)>,
    miko: Query<(Entity, &Transform), With<Miko>>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    mut letters: ResMut<Mailbox>,
    mut identity: ResMut<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, actor_at)) = miko.single() else {
        return;
    };
    let answered = if dialogue.speaker == Some(actor) {
        dialogue.external_result.take()
    } else {
        None
    };
    if let Some(choice) = answered {
        *focus = InputFocus::Dialogue;
        match std::mem::take(&mut flow.step) {
            Step::Offer if choice == 0 => {
                // `aEMK_talk_select`.
                if !letters.has_room() {
                    say(&disc, &mut dialogue, actor, &identity, 0x190F);
                    flow.step = Step::Line;
                } else if inventory.bells < PRICE {
                    say(&disc, &mut dialogue, actor, &identity, 0x190E);
                    flow.step = Step::Line;
                } else {
                    // `aEMK_talk_omikuji_init` and `aEMK_talk_omikuji`.
                    let n = rand.0.random(4) as u8;
                    say(&disc, &mut dialogue, actor, &identity, native::miko_msg(n));
                    flow.step = Step::Result(n);
                }
            }
            Step::Result(n) => {
                // `aEMK_talk_give`: the destiny, the Bells, the slip and its letter.
                *dialogue = DialogueState::default();
                identity.destiny = native::miko_destiny(n);
                identity.destiny_day = crate::clock::now().day_number;
                inventory.bells = inventory.bells.saturating_sub(PRICE);
                match omikuji_letter(&disc, &identity, usize::from(n), &mut rand) {
                    Some(letter) => {
                        info!(
                            "The fortune slip letter: {}",
                            letter.body.replace('\n', " / ")
                        );
                        letters.push(letter);
                    }
                    None => warn!("The fortune slip letter could not be made"),
                }
                info!(
                    "The shrine maiden's omikuji {n}: destiny {}",
                    identity.destiny
                );
                handover.start(Request {
                    item: Item::Goods(FORTUNE_SLIP),
                    giver: Party::Npc(actor),
                    taker: Party::Player,
                });
                flow.step = Step::Handing;
            }
            Step::Offer | Step::Line | Step::Idle | Step::Handing => {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
            }
        }
        return;
    }
    if matches!(flow.step, Step::Handing) {
        if handover.busy() {
            *focus = InputFocus::Dialogue;
        } else {
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
    if delta.length() > TALK_RANGE || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    // `miko->in_front`: the player is within 33.75 degrees of where she faces.
    let to_player = -delta.normalize_or_zero();
    let faces = Vec3::new(FACING_YAW.sin(), 0.0, FACING_YAW.cos());
    let in_front = faces.dot(to_player) > (33.75f32).to_radians().cos();
    if in_front {
        say(&disc, &mut dialogue, actor, &identity, 0x190C);
        flow.step = Step::Offer;
    } else {
        say(&disc, &mut dialogue, actor, &identity, 0x1916);
        flow.step = Step::Line;
    }
}

pub struct MikoPlugin;

impl Plugin for MikoPlugin {
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
