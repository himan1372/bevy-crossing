//! Tortimer's commemorative speeches at the shrine (`SP_NPC_EV_SONCHO2`, `ac_ev_soncho2*.c`, `m_soncho.c`).
//!
//! While one of the mayor's events runs (`mSC_EVENT_*`, the scheduler's `SONCHO_*` rows and the three that use their
//! own: the meteor shower, the harvest moon and the countdown) he waits at the shrine. A talk starts with the calendar
//! mark of the event on that date (`mCD_calendar_event_on`). A player who has no commemorative item of the event yet
//! (`mSC_trophy_get`) hears `msg(0)` (`msg(1)` when the calendar already has the mark), then `msg(3)` with a free pocket
//! (`msg(2)` without) and receives the item (`mSC_trophy_item`, the trophy is set). A player who has it hears `msg(5)`,
//! or `msg(6 + RANDOM(3))` when the calendar has the mark. `msg(n)` is `0x3280 + 10 * event + n`
//! (`MSG_SONCHO_EVENTS`), the harvest festival's `0x3391 + n`.
//!
//! The item: the event's table entry, but the New Year's diary (`ITM_DIARY_START + (RANDOM(15) & 15)`, the original's
//! off-by-one kept), a doll by gender on Toy Day, a random flower model on Groundhog Day and a station toy on Town Day
//! (the port picks from the train goods list).
//!
//! On the sports fairs he talks during the foot race, the aerobics (standing in the line, `aerobics.rs`) and the tug of war
//! but not during the ball toss.
//!
//! Not built: the villagers' contests of the sports fairs, the lighthouse quest, the morning aerobics, his wandering
//! and the event's name string (`mSC_event_name_set`).

use bevy::prelude::*;
use rustimal_logic::event_schedule::ev;

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
use rustimal_logic::inventory::EMPTY_NO;

/// ALL_NPC_NUM + (SP_NPC_EV_SONCHO2 - SP_NPC_START): Tortimer's event model (`SP_NPC_START + 116`).
const SONCHO2_NPC: u32 = 238 + 116;
const SOUND: u8 = 2;
/// `MSG_SONCHO_EVENTS` and its per-event count, and the harvest festival's own base.
const MSG_EVENTS: u16 = 0x3280;
const MSG_EVENTS_COUNT: u16 = 10;
const MSG_HARVEST_FESTIVAL: u16 = 0x3391;
/// `mSC_EVENT_HARVEST_FESTIVAL`, `mSC_EVENT_TOY_DAY`, `mSC_EVENT_TOWN_DAY`, `mSC_EVENT_GROUNDHOG_DAY`,
/// `mSC_EVENT_NEW_YEARS_DAY`.
const HARVEST_FESTIVAL: u8 = 27;
const TOY_DAY: u8 = 25;
const TOWN_DAY: u8 = 4;
const GROUNDHOG_DAY: u8 = 13;
const NEW_YEARS_DAY: u8 = 0;
/// `mSC_SPECIAL_EVENT_JAN_VACATION` and `mSC_SPECIAL_EVENT_FEB_VACATION`, and the lighthouse messages' bases.
const JAN_VACATION: u8 = 101;
const FEB_VACATION: u8 = 102;
const MSG_LIGHTHOUSE: [u16; 2] = [0x33F4, 0x340B];
/// The strings of the days of the month (`mString_Load_DayStringFromRom`).
const DAY_STRINGS: usize = 0x64E;

/// The sports fairs (`mSC_EVENT_SPRING_SPORTS_FAIR` and `_FALL_`): during the aerobics and the ball toss he does not
/// talk (`aES2_THINK_AEROBICS`, `aES2_THINK_BALL_TOSS`: no talk request).
const SPORTS_FAIRS: [u8; 2] = [8, 20];

/// `event_table`: the scheduler row of each `mSC_EVENT_*`.
const EVENT_ROWS: [u16; 28] = [
    ev::SONCHO_NEW_YEARS_DAY,
    ev::SONCHO_FOUNDERS_DAY,
    ev::SONCHO_GRADUATION_DAY,
    ev::SONCHO_APRILFOOLS_DAY,
    ev::SONCHO_TOWN_DAY,
    ev::SONCHO_MOTHERS_DAY,
    ev::SONCHO_SALE_DAY,
    ev::SONCHO_CHERRY_BLOSSOM_FESTIVAL,
    ev::SONCHO_SPRING_SPORTS_FAIR,
    ev::SONCHO_NATURE_DAY,
    ev::SONCHO_SPRING_CLEANING,
    ev::SONCHO_FATHERS_DAY,
    ev::SONCHO_FISHING_TOURNEY_1,
    ev::SONCHO_GROUNDHOG_DAY,
    ev::SONCHO_EXPLORERS_DAY,
    ev::SONCHO_FIREWORKS_SHOW,
    ev::METEOR_SHOWER,
    ev::HARVEST_MOON_FESTIVAL,
    ev::SONCHO_MAYORS_DAY,
    ev::SONCHO_OFFICERS_DAY,
    ev::SONCHO_FALL_SPORTS_FAIR,
    ev::SONCHO_HALLOWEEN,
    ev::SONCHO_FISHING_TOURNEY_2,
    ev::SONCHO_SNOW_DAY,
    ev::SONCHO_LABOR_DAY,
    ev::SONCHO_TOY_DAY,
    ev::NEW_YEARS_EVE_COUNTDOWN,
    ev::SONCHO_HARVEST_FESTIVAL,
];

/// `soncho_item_table` (furniture items from the decomp's furniture enum; foods are `ITM_FOOD_*`).
const ITEM_TABLE: [u16; 28] = [
    0x2800, // New Year's Day: an apple (replaced by the diary)
    0x3080, // Founders' Day: weeds
    0x3084, // Graduation Day: tailor
    0x3280, // April Fools' Day
    0x2800, // Town Day: an apple (replaced by a station toy)
    0x3044, // Mothers' Day
    0x3048, // Sale Day
    0x307C, // Cherry blossom festival
    0x32A8, // Spring sports fair
    0x3078, // Nature Day
    0x3088, // Spring cleaning
    0x30A0, // Fathers' Day
    0x3374, // Fishing tourney 1
    0x2802, // Groundhog Day: a pear (replaced by a flower model)
    0x1FC0, // Explorers' Day
    0x31F0, // Fireworks show
    0x3238, // Meteor shower
    0x309C, // Harvest moon
    0x30E4, // Mayors' Day
    0x302C, // Officers' Day
    0x32AC, // Fall sports fair
    0x304C, // Halloween
    0x3210, // Fishing tourney 2
    0x3090, // Snow Day
    0x3060, // Labor Day
    0x2800, // Toy Day: an apple (replaced by a doll)
    0x33BC, // Countdown
    0x33B4, // Harvest festival
];

#[derive(Component)]
struct Soncho {
    day: i64,
    event: u8,
    /// He stands in the aerobics line (`SP_NPC_SONCHO_D078`) rather than at his own place.
    lined: bool,
    /// He stands by the foot race (unit (5, 8)).
    foot: bool,
    /// He stands by the ball toss (unit (9, 4)).
    tama: bool,
    /// He stands by the tug of war (unit (10, 8)).
    tug: bool,
    /// `soncho->item`.
    item: u16,
}

/// `mSC_get_soncho_event`: the first running event (the second fishing tourney last).
fn running_event(today: &TodayEvents, hour: u8) -> Option<u8> {
    let mut order = (0..EVENT_ROWS.len() as u8)
        .filter(|&e| e != 22)
        .chain(std::iter::once(22));
    order
        .find(|&e| today.is_active(EVENT_ROWS[usize::from(e)], hour))
        // Then the lighthouse vacations (`mSC_SPECIAL_EVENT_JAN_VACATION` and `_FEB_`).
        .or_else(|| {
            today
                .is_active(ev::SONCHO_VACATION_JANUARY, hour)
                .then_some(JAN_VACATION)
        })
        .or_else(|| {
            today
                .is_active(ev::SONCHO_VACATION_FEBRUARY, hour)
                .then_some(FEB_VACATION)
        })
}

/// `aES2_wait`: the field event of a sports fair that keeps him from talking (in the aerobics he talks from the line).
fn busy_with_field_event(today: &TodayEvents, hour: u8) -> bool {
    today.is_active(ev::SPORTS_FAIR_BALL_TOSS, hour)
}

fn trophy(identity: &crate::save::PlayerIdentity, event: u8) -> bool {
    identity.soncho_trophies & (1 << event) != 0
}

/// `mCD_calendar_event_check` for today.
fn calendar_marked(identity: &crate::save::PlayerIdentity, event: u8) -> bool {
    let now = crate::clock::now();
    identity
        .soncho_calendar
        .contains(&(event, now.year, now.month, now.day))
}

/// `mSC_trophy_item`.
fn trophy_item(event: u8, female: bool, rand: &mut crate::game_rand::GameRand) -> u16 {
    if event >= JAN_VACATION {
        return EMPTY_NO;
    }
    match event {
        NEW_YEARS_DAY => 0x2B00 + (rand.0.random(15) & 15) as u16,
        TOY_DAY => {
            if female {
                0x30A4
            } else {
                0x3094
            }
        }
        TOWN_DAY => {
            let list = crate::goods::list(crate::goods::Kind::Furniture, 4);
            if list.is_empty() {
                ITEM_TABLE[usize::from(event)]
            } else {
                list[rand.0.random(list.len() as i32) as usize]
            }
        }
        // `mRmTp_FtrIdx2FtrItemNo(0x4DE + RANDOM(9), SOUTH)`.
        GROUNDHOG_DAY => 0x3000 + (0x4DE - 0x400 + rand.0.random(9) as u16) * 4,
        _ => ITEM_TABLE[usize::from(event)],
    }
}

/// `aES2_kinenhin_msg`.
fn msg(event: u8, n: u16) -> u16 {
    if event == HARVEST_FESTIVAL {
        MSG_HARVEST_FESTIVAL + n
    } else {
        u16::from(event) * MSG_EVENTS_COUNT + MSG_EVENTS + n
    }
}

/// Brings him to the shrine while an event of his runs.
#[allow(clippy::too_many_arguments)]
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    today: Res<TodayEvents>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<(Entity, &Soncho)>,
    look: Res<crate::character::PlayerLook>,
    identity: Res<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let now = crate::clock::now();
    let event = running_event(&today, now.hour);
    let lined = event.is_some_and(|e| SPORTS_FAIRS.contains(&e))
        && today.is_active(ev::SPORTS_FAIR_AEROBICS, now.hour);
    let foot = event.is_some_and(|e| SPORTS_FAIRS.contains(&e))
        && today.is_active(ev::SPORTS_FAIR_FOOT_RACE, now.hour);
    let tama = event.is_some_and(|e| SPORTS_FAIRS.contains(&e))
        && today.is_active(ev::SPORTS_FAIR_BALL_TOSS, now.hour);
    let tug = event.is_some_and(|e| SPORTS_FAIRS.contains(&e))
        && today.is_active(ev::SPORTS_FAIR_TUG_OF_WAR, now.hour);
    let out = event.is_some()
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    for (entity, s) in &existing {
        if !out
            || s.day != now.day_number
            || Some(s.event) != event
            || s.lined != lined
            || s.foot != foot
            || s.tama != tama
            || s.tug != tug
        {
            commands.entity(entity).despawn();
        }
    }
    let (true, Some(event)) = (out, event) else {
        return;
    };
    if !existing.is_empty() {
        return;
    }
    // `aES2_wait`: once somebody has started the quest he does not come out for the vacation again.
    if event >= JAN_VACATION
        && identity
            .lighthouse
            .talk_after((now.year, now.month, now.day))
    {
        return;
    }
    let unit = if lined {
        crate::aerobics::LEADER_UNIT
    } else if foot {
        crate::footrace::TORTIMER_UNIT
    } else if tama {
        crate::balltoss::TORTIMER_UNIT
    } else if tug {
        crate::tugofwar::TORTIMER_UNIT
    } else {
        (11, 6)
    };
    let Some(at) = crate::tortimer::shrine_cell(&town, unit.0, unit.1) else {
        return;
    };
    let item = trophy_item(event, look.female, &mut rand);
    commands.spawn((
        Name::new("Tortimer (event)"),
        Soncho {
            day: now.day_number,
            event,
            lined,
            foot,
            tama,
            tug,
            item,
        },
        NpcClip::default(),
        NpcGait::default(),
        // `SP_NPC_SONCHO_D078` in the line.
        NpcModel(if lined { 238 + 120 } else { SONCHO2_NPC }),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y),
    ));
    info!(
        "Tortimer is at the shrine for event {event}, item {item:#06x} ({})",
        crate::items::name(item)
    );
}

#[derive(Default)]
enum Step {
    #[default]
    Idle,
    /// A line is up; the talk ends with it.
    Line,
    /// A question is up (the lighthouse offer); each answer's own follow-up line.
    Ask(Vec<Option<usize>>),
    /// `msg(0)` / `msg(1)` is up: the item (or the full-pockets line) follows.
    BeforeGive,
    /// `msg(3)` is up: the item is handed over next.
    Give,
    /// The item is crossing.
    Handing,
}

#[derive(Resource, Default)]
struct Talk {
    step: Step,
}

fn context(identity: &crate::save::PlayerIdentity, item: u16) -> MsgContext {
    let now = crate::clock::now();
    MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Tortimer".into(),
        free: vec![String::new(); 20],
        items: vec![crate::items::name(item); 5],
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

fn say_with(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    ctx: &MsgContext,
    message: u16,
) -> Option<Vec<Option<usize>>> {
    let text = disc.text.as_ref()?;
    let (pages, asked) = text.talk_asking(usize::from(message), ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "Tortimer",
        (SONCHO2_NPC, SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Tortimer message {message:#06x}");
    asked.map(|(_, c)| c.next)
}

fn say(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    identity: &crate::save::PlayerIdentity,
    item: u16,
    message: u16,
) {
    let Some(text) = disc.text.as_ref() else {
        return;
    };
    let ctx = context(identity, item);
    let (pages, asked) = text.talk_asking(usize::from(message), &ctx, &mut |_| 0);
    dialogue.begin_special(
        actor,
        "Tortimer",
        (SONCHO2_NPC, SOUND),
        pages,
        asked.as_ref().map(|(_, c)| c.labels.clone()),
    );
    info!("Tortimer message {message:#06x}");
}

/// `aES2_set_norm_talk_info` for the lighthouse vacations: the quest's offer, which starts it
/// (`mSC_LightHouse_Quest_Start`, the free strings the days a week and a week and a day on), or after it `+ 6 + RANDOM(3)`.
fn lighthouse_talk(
    disc: &GameDisc,
    dialogue: &mut DialogueState,
    actor: Entity,
    identity: &mut crate::save::PlayerIdentity,
    event: u8,
    rand: &mut crate::game_rand::GameRand,
) -> Option<Vec<Option<usize>>> {
    let now = crate::clock::now();
    let date = (now.year, now.month, now.day);
    let base = MSG_LIGHTHOUSE[usize::from(event - JAN_VACATION)];
    let mut ctx = context(identity, EMPTY_NO);
    if identity.lighthouse.talk_after(date) {
        let msg = base + 6 + rand.0.random(3) as u16;
        return say_with(disc, dialogue, actor, &ctx, msg);
    }
    // The quest starts as the talk does; the two days are named in the offer.
    identity.lighthouse.quest_start(date);
    if let Some(text) = disc.text.as_ref() {
        for (slot, n) in [(0usize, 7), (1, 8)] {
            let (_, day) = crate::lighthouse::after_days(now.year, now.month, now.day, n);
            ctx.free[slot] = text
                .string(DAY_STRINGS + usize::from(day) - 1)
                .unwrap_or_default();
        }
    }
    info!("The lighthouse quest starts on {date:?}");
    say_with(disc, dialogue, actor, &ctx, base)
}

/// He faces the player while they talk.
fn face(
    time: Res<Time>,
    flow: Res<Talk>,
    player: Query<&Transform, (With<Player>, Without<Soncho>)>,
    mut soncho: Query<&mut Transform, With<Soncho>>,
) {
    let (Ok(at), Ok(mut transform)) = (player.single(), soncho.single_mut()) else {
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

/// The talk, as `aES2_set_norm_talk_info` and the talk procs.
#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), (With<Player>, Without<Soncho>)>,
    soncho: Query<(Entity, &Transform, &Soncho)>,
    mut flow: ResMut<Talk>,
    mut dialogue: ResMut<DialogueState>,
    mut inventory: ResMut<PlayerInventory>,
    mut handover: ResMut<HandOver>,
    mut identity: ResMut<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
    today: Res<TodayEvents>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, actor_at, me)) = soncho.single() else {
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
            Step::Ask(next) => {
                let ctx = context(&identity, me.item);
                match next.get(choice).copied().flatten() {
                    Some(m) => {
                        flow.step = say_with(&disc, &mut dialogue, actor, &ctx, m as u16)
                            .map_or(Step::Line, Step::Ask);
                    }
                    None => {
                        *dialogue = DialogueState::default();
                        *focus = InputFocus::World;
                    }
                }
            }
            Step::BeforeGive => {
                // `aES2_talk_before_give`.
                if !inventory.pockets.pockets.contains(&EMPTY_NO) {
                    say(
                        &disc,
                        &mut dialogue,
                        actor,
                        &identity,
                        me.item,
                        msg(me.event, 2),
                    );
                    flow.step = Step::Line;
                } else {
                    say(
                        &disc,
                        &mut dialogue,
                        actor,
                        &identity,
                        me.item,
                        msg(me.event, 3),
                    );
                    flow.step = Step::Give;
                }
            }
            Step::Give => {
                // `aES2_talk_give`: the item, then the trophy.
                *dialogue = DialogueState::default();
                inventory.add(me.item);
                identity.soncho_trophies |= 1 << me.event;
                info!(
                    "Tortimer gives {:#06x} ({}) for event {}",
                    me.item,
                    crate::items::name(me.item),
                    me.event
                );
                handover.start(Request {
                    item: Item::Goods(me.item),
                    giver: Party::Npc(actor),
                    taker: Party::Player,
                });
                flow.step = Step::Handing;
            }
            Step::Line | Step::Idle | Step::Handing => {
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
    if delta.length() > 1.8 || facing.dir.dot(delta.normalize_or_zero()) < 0.5 {
        return;
    }
    // He takes no talk during a sports fair's aerobics or ball toss.
    if SPORTS_FAIRS.contains(&me.event) && busy_with_field_event(&today, crate::clock::now().hour) {
        return;
    }
    actions.consume(Action::Interact);
    *focus = InputFocus::Dialogue;
    let event = me.event;
    if event >= JAN_VACATION {
        let asked = lighthouse_talk(&disc, &mut dialogue, actor, &mut identity, event, &mut rand);
        flow.step = asked.map_or(Step::Line, Step::Ask);
        return;
    }
    let marked = calendar_marked(&identity, event);
    // `aES2_talk_init`: the calendar mark is made as the talk starts.
    let now = crate::clock::now();
    let mark = (event, now.year, now.month, now.day);
    if !identity.soncho_calendar.contains(&mark) {
        identity.soncho_calendar.push(mark);
    }
    if trophy(&identity, event) {
        let n = if marked {
            6 + rand.0.random(3) as u16
        } else {
            5
        };
        say(
            &disc,
            &mut dialogue,
            actor,
            &identity,
            me.item,
            msg(event, n),
        );
        flow.step = Step::Line;
    } else {
        let n = u16::from(marked);
        say(
            &disc,
            &mut dialogue,
            actor,
            &identity,
            me.item,
            msg(event, n),
        );
        flow.step = Step::BeforeGive;
    }
}

pub struct SonchoPlugin;

impl Plugin for SonchoPlugin {
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
