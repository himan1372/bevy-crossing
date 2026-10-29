//! Phase 3: save/load system.
//!
//! - **F5**: quick save to `savegame.json` (HUD notification)
//! - **F9**: quick load from `savegame.json` (HUD notification)
//! - On startup, an existing save restores the town seed, player position,
//!   inventory, playtime, and picked-up items.
//!
//! The logic crate's `save` module models the GameCube memory-card format
//! (GCI headers, checksums, `Save_t` regions) — that's the retail
//! serialization format, not a general-purpose API. The Bevy port uses a
//! Bevy-native JSON save instead; nothing from `logic::save` is called here.
//!
//! Villager positions are NOT saved: villagers re-randomize on load (they
//! wander continuously, so exact positions don't matter).

use bevy::prelude::*;
use bevy::text::FontSize;
use rustimal_logic::inventory::{Inventory, POCKETS_SLOT_COUNT};
use rustimal_logic::species::{FishType, InsectType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::character::PlayerLook;
use crate::field_items::FieldItems;
use crate::house::{HouseState, ReturnPosition};
use crate::input::{Action, Actions, InputFocus};
use crate::interaction::PlayerInventory;
use crate::items;
use crate::mail::Mailbox;
use crate::player::Player;
use crate::states::GameplaySet;
use crate::town::TownSeed;

const SAVE_FILE_NAME: &str = "savegame.json";
const SAVE_VERSION: u32 = 1;
const NOTIFY_DURATION_SECS: f32 = 2.5;

/// Philip's chosen identity until the deferred introduction supplies names.
#[derive(Resource, Serialize, Deserialize, Clone)]
pub(crate) struct PlayerIdentity {
    pub player: String,
    pub town: String,
    /// `player_ID.player_id` (0xF000 | 0..252); 0 until assigned.
    #[serde(default)]
    pub id: u16,
    /// The part-time-job event flags of this player (`m_event.c`).
    #[serde(default, with = "personal_events_serde")]
    pub events: rustimal_logic::personal_events::PersonalEvents,
    /// Day number of the last daily renewal of those flags.
    #[serde(default)]
    pub events_day: i64,
    /// `mSC_SPECIAL_EVENT_ARBEIT`: the player has talked to Tortimer for the job.
    #[serde(default)]
    pub arbeit_trophy: bool,
    /// `Common(map_flag)`: the player has the town map. Off from the new-game intro until the
    /// guide hands it over; towns that never ran the job always have it.
    #[serde(default = "map_flag_on")]
    pub map_flag: bool,
    /// The part-time job's errand (`mQst_GetFirstJobData`); absent until the job starts.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "first_job_serde"
    )]
    pub first_job: Option<rustimal_logic::first_job::FirstJob>,
    /// The new-game intro has been played (or skipped). Saves from before it never run it.
    #[serde(default = "intro_done_on")]
    pub intro_done: bool,
}

fn map_flag_on() -> bool {
    true
}

fn intro_done_on() -> bool {
    true
}

/// `FirstJob` as a plain record, since the logic crate has no serde.
mod first_job_serde {
    use rustimal_logic::first_job::{FirstJob, Job};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct Record {
        job: Option<u8>,
        progress: u8,
        recipient: Option<u16>,
        used: [Option<u16>; 2],
        used_num: u8,
        item: u16,
        pockets_idx: usize,
        wrong_cloth: bool,
    }

    pub fn serialize<S: Serializer>(job: &Option<FirstJob>, s: S) -> Result<S::Ok, S::Error> {
        job.as_ref()
            .map(|j| Record {
                job: j.job.map(|k| k.index() as u8),
                progress: j.progress,
                recipient: j.recipient,
                used: j.used,
                used_num: j.used_num,
                item: j.item,
                pockets_idx: j.pockets_idx,
                wrong_cloth: j.wrong_cloth,
            })
            .serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<FirstJob>, D::Error> {
        Ok(Option::<Record>::deserialize(d)?.map(|r| FirstJob {
            job: r.job.and_then(|i| Job::from_index(usize::from(i))),
            progress: r.progress,
            recipient: r.recipient,
            used: r.used,
            used_num: r.used_num.min(2),
            item: r.item,
            pockets_idx: r.pockets_idx,
            wrong_cloth: r.wrong_cloth,
        }))
    }
}

/// `PersonalEvents` as three booleans, since the logic crate has no serde.
mod personal_events_serde {
    use rustimal_logic::personal_events::PersonalEvents;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(ev: &PersonalEvents, s: S) -> Result<S::Ok, S::Error> {
        (ev.first_job, ev.hra_wait, ev.hra_talk).serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<PersonalEvents, D::Error> {
        let (first_job, hra_wait, hra_talk) = <(bool, bool, bool)>::deserialize(d)?;
        Ok(PersonalEvents {
            first_job,
            hra_wait,
            hra_talk,
        })
    }
}

/// `mEv_RenewalDataEveryDay`: once per calendar day.
fn renew_personal_events(mut identity: ResMut<PlayerIdentity>) {
    let today = crate::clock::now().day_number;
    if identity.events_day != today {
        identity.events_day = today;
        identity.events.renew_daily();
    }
}

impl Default for PlayerIdentity {
    fn default() -> Self {
        Self {
            player: "phi".into(),
            town: "woponese".into(),
            id: 0,
            events: Default::default(),
            events_day: 0,
            arbeit_trophy: false,
            map_flag: true,
            first_job: None,
            intro_done: false,
        }
    }
}

/// `mPr_SetNewPrivateID`: a player id is made once, `0xF000 | (rand * 253)`,
/// and kept. The draw here comes from the town's seed and the names, so an
/// older save is given the same id every time it is opened.
fn assign_player_id(mut identity: ResMut<PlayerIdentity>, seed: Res<TownSeed>) {
    if identity.id == 0 {
        let draw = identity
            .player
            .bytes()
            .chain(identity.town.bytes())
            .fold(seed.0, |s, b| s.wrapping_mul(31).wrapping_add(u32::from(b)));
        identity.id = 0xF000 | (draw % 253) as u16;
        info!("player id {:#06x}", identity.id);
    }
}

fn is_zero(n: &u16) -> bool {
    *n == 0
}

/// Serializable snapshot of game state.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SaveData {
    pub version: u32,
    #[serde(default)]
    pub(crate) identity: PlayerIdentity,
    pub town_seed: u32,
    pub player_pos: [f32; 3],
    pub bells: u32,
    /// House loan. Saves from before loans count as paid off (Philip).
    #[serde(default)]
    pub loan: u32,
    /// Upstairs and basement furniture (`home_furniture` is the main floor).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) home_floors: Option<[Option<crate::player_room::RoomSave>; 2]>,
    /// The collection record, tickets waiting for the mail and catalog
    /// orders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog: Option<crate::catalog::CatalogSave>,
    /// The house mailbox's letters (the pockets' are `letters`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub house_mail: Vec<crate::mail::Letter>,
    /// Villagers who remember the player (by name).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub villager_memory: Vec<String>,
    /// The post office's letters waiting, replies owed and the museum's
    /// record; `None` before mail went through the post office.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post: Option<crate::post::Post>,
    /// The museum's donations; `None` before the museum existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub museum: Option<crate::museum::MuseumSave>,
    /// The police box's lost and found; `None` before it existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub police_box: Option<[u16; 20]>,
    /// The day the lost and found last grew.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub police_grow_day: Option<i64>,
    /// House size, orders and roof colour; `None` before upgrades existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub house: Option<crate::home::HouseSave>,
    /// Pocket item IDs (`crate::items`), 15 slots. `None` in saves made
    /// before pockets existed; those are converted from the legacy counts.
    #[serde(default)]
    pub pockets: Option<Vec<u16>>,
    /// Packed 2-bit pocket conditions (`rustimal_logic::inventory`).
    #[serde(default)]
    pub pocket_conds: u32,
    /// Legacy (pre-pockets) fruit count; read for conversion, written as 0.
    pub fruit: u32,
    /// Legacy fish count (added after v1; defaults to 0 for old saves).
    #[serde(default)]
    pub fish: u32,
    /// Bugs caught (added after v1; defaults to 0 for old saves).
    #[serde(default)]
    pub bugs: u32,
    pub playtime_secs: f64,
    /// Legacy stand-in ground items picked up; read and ignored.
    #[serde(default)]
    pub picked_items: Vec<usize>,
    /// Town field items that differ from the new town's, as
    /// [acre, cell, item] (`crate::field_items`; added later).
    #[serde(default)]
    pub field_items: Option<Vec<[u16; 3]>>,
    /// `Save_Get(dust_flag)`: the town was rated down for trash and has not
    /// been cleared since (`crate::field_rank`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub dust_flag: bool,
    /// Received letters (added later; defaults to empty for old saves).
    #[serde(default)]
    pub letters: Vec<crate::mail::Letter>,
    /// Sent letters (added later; defaults to empty for old saves).
    #[serde(default)]
    pub sent_letters: Vec<crate::mail::Letter>,
    /// Nook's stock (added later; defaults to empty = regenerate).
    #[serde(default)]
    pub shop_stock: Vec<crate::shop::StockItem>,
    /// Sales sum toward the shop upgrade (added later).
    #[serde(default)]
    pub shop_sales: u32,
    /// Displayed shop tier as u8 (added later).
    #[serde(default)]
    pub shop_tier: u8,
    /// Pending shop renovation and native visitor gate; absent in legacy saves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) shop_progress: Option<crate::shop::schedule::Progress>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) shop_events: Option<crate::shop::bargain::Calendar>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) shop_turnips: Option<crate::shop::turnips::Market>,
    /// Legacy furniture names (pre-pockets); read for conversion only.
    #[serde(default)]
    pub furniture: Vec<String>,
    /// Placed furniture and its owner-cell storage. None preserves old saves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) home_furniture: Option<crate::player_room::RoomSave>,
    /// Item numbering of the pockets: 0/1 the slice's own, 2 the game's
    /// (`crate::items`). Older pockets are migrated on load.
    #[serde(default)]
    pub item_numbers: u32,
    /// Nook's list rarities (`Shop_c.priority_lists`).
    #[serde(default)]
    pub shop_priorities: Option<[[u8; 3]; 5]>,
    /// Game day the stock was laid out.
    #[serde(default)]
    pub shop_stock_day: Option<i64>,
    /// Next paint colour Nookway sells.
    #[serde(default)]
    pub shop_paint: u8,
    /// Nook's spotlight rare furniture (`Shop_c.rare_item`); 0 for none.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub shop_rare: u16,
    /// Active quest (added later; defaults to none for old saves).
    #[serde(default)]
    pub quest: Option<crate::quest::ActiveQuest>,
    /// The player's face type (`Private_c::face`, 0-7; added later, picked
    /// at random for old saves like a new player's).
    #[serde(default)]
    pub player_face: Option<u8>,
    /// The player's shirt (clothing bank index; added later).
    #[serde(default)]
    pub player_cloth: Option<u16>,
    /// Residents by villager table index, in move-in order (added later;
    /// none = a new town's starters).
    #[serde(default)]
    pub villagers: Option<Vec<u32>>,
    /// Villagers who have lived here (`npc_used_tbl`).
    #[serde(default)]
    pub villagers_appeared: Vec<u32>,
    /// Villagers the player has talked to.
    #[serde(default)]
    pub villagers_met: Vec<u32>,
    /// Game day of the last move-in.
    #[serde(default)]
    pub last_move_in_day: Option<i64>,
    /// The home slot of each resident (parallel to `villagers`; empty: slot = position).
    #[serde(default)]
    pub villager_slots: Vec<usize>,
    /// `force_remove_date`: the last move-in or removal.
    #[serde(default)]
    pub last_remove_day: Option<i64>,
    /// `remove_animal_idx`: the slot of the villager thinking of moving.
    #[serde(default)]
    pub remove_slot: Option<usize>,
}

/// Save data loaded at startup (PreStartup), consumed by Startup systems.
#[derive(Resource, Default)]
pub struct LoadedSave(pub Option<SaveData>);

/// Accumulated playtime in seconds.
#[derive(Resource, Default)]
pub struct Playtime {
    pub secs: f64,
}

/// Marker for the save notification text (top-center HUD).
#[derive(Component)]
struct NotifyText;

/// Countdown for how long the notification stays visible.
#[derive(Resource, Default)]
struct NotifyTimer(f32);

fn save_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_default()
        .join(SAVE_FILE_NAME)
}

fn read_save() -> Option<SaveData> {
    let text = fs::read_to_string(save_path()).ok()?;
    let save: SaveData = serde_json::from_str(&text).ok()?;
    if save.version != SAVE_VERSION {
        println!(
            "Save version mismatch ({} != {SAVE_VERSION}), starting fresh",
            save.version
        );
        return None;
    }
    Some(save)
}

fn write_save(save: &SaveData) -> bool {
    match serde_json::to_string_pretty(save) {
        Ok(json) => fs::write(save_path(), json).is_ok(),
        Err(e) => {
            println!("Failed to serialize save: {e}");
            false
        }
    }
}

/// Seed for a new player's random look: the clock's nanoseconds.
fn new_player_seed() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(1, |d| d.subsec_nanos() | 1)
}

/// PreStartup: read the save file (if any) and seed resources so the
/// Startup systems (town gen, player spawn, item spawn) can use them.
pub(crate) fn init_save(mut commands: Commands) {
    let loaded = read_save();
    commands.insert_resource(
        loaded
            .as_ref()
            .map(|s| s.identity.clone())
            .unwrap_or_default(),
    );
    match &loaded {
        Some(s) => {
            println!(
                "Save found: seed {}, {:.0}s played, {} bells, {} fruit",
                s.town_seed, s.playtime_secs, s.bells, s.fruit
            );
            commands.insert_resource(TownSeed(s.town_seed));
            commands.insert_resource(Playtime {
                secs: s.playtime_secs,
            });
            let new = PlayerLook::new_player(new_player_seed());
            commands.insert_resource(PlayerLook {
                face: s.player_face.map_or(new.face, usize::from),
                cloth: s.player_cloth.map_or(new.cloth, usize::from),
            });
        }
        None => {
            println!("No save found, starting fresh");
            commands.insert_resource(TownSeed(12345));
            commands.insert_resource(Playtime::default());
            commands.insert_resource(PlayerLook::new_player(new_player_seed()));
        }
    }
    commands.insert_resource(LoadedSave(loaded));
}

/// Startup (after `spawn_player`): move the player to the saved position.
fn apply_loaded_player(loaded: Res<LoadedSave>, mut player: Query<&mut Transform, With<Player>>) {
    let Some(save) = loaded.0.as_ref() else {
        return;
    };
    if let Ok(mut transform) = player.single_mut() {
        transform.translation = Vec3::from_array(save.player_pos);
        println!("Player position restored");
    }
}

/// The pockets' item numbering this build writes (see `SaveData`).
const ITEM_NUMBERS: u32 = 2;

/// A pre-pockets furniture name, as the slice's stand-in catalog had it.
fn legacy_furniture(name: &str) -> Option<u16> {
    const NAMES: [&str; 6] = [
        "Wooden Chair",
        "Potted Plant",
        "Wall Clock",
        "Modern Lamp",
        "Bookshelf",
        "Cozy Bed",
    ];
    let i = NAMES.iter().position(|n| *n == name)?;
    Some(items::migrate_v1(0x1000 + i as u16))
}

/// Pockets from a save. Saves made before pockets existed carry counts
/// instead: fruit and furniture convert directly; fish and bugs become one
/// stand-in species each (the counts never recorded which species). An
/// active delivery gets its parcel, so it can still be handed over.
fn pockets_from_save(save: &SaveData) -> Inventory {
    let mut inv = Inventory::default();
    if let Some(saved) = save.pockets.as_ref() {
        for (slot, &item) in saved.iter().take(POCKETS_SLOT_COUNT).enumerate() {
            inv.pockets[slot] = if save.item_numbers < ITEM_NUMBERS {
                items::migrate_v1(item)
            } else {
                item
            };
        }
        inv.item_conditions = save.pocket_conds;
        return inv;
    }
    let legacy = std::iter::repeat_n(items::FRUIT, save.fruit as usize)
        .chain(std::iter::repeat_n(
            items::fish(FishType::CrucianCarp),
            save.fish as usize,
        ))
        .chain(std::iter::repeat_n(
            items::bug(InsectType::CommonButterfly),
            save.bugs as usize,
        ))
        .chain(save.furniture.iter().filter_map(|n| legacy_furniture(n)));
    let mut dropped = 0;
    for item in legacy {
        if inv.put(item, 0).is_none() {
            dropped += 1;
        }
    }
    let delivery = matches!(
        save.quest.as_ref().map(|q| &q.kind),
        Some(crate::quest::QuestKind::Delivery { .. })
    );
    if delivery && inv.find_item(items::PARCEL).is_none() && inv.put(items::PARCEL, 0).is_none() {
        dropped += 1;
    }
    if dropped > 0 {
        warn!("old save: {dropped} item(s) didn't fit in 15 pockets and were dropped");
    }
    println!("Converted an old save's item counts into pockets");
    inv
}

/// Startup: restore the wallet and pockets from the loaded save.
pub(crate) fn apply_loaded_inventory(
    loaded: Res<LoadedSave>,
    mut inventory: ResMut<PlayerInventory>,
) {
    if let Some(save) = loaded.0.as_ref() {
        inventory.bells = save.bells;
        inventory.loan = save.loan;
        inventory.pockets = pockets_from_save(save);
        println!(
            "Inventory restored: {} bells, {}/{} pockets",
            save.bells,
            inventory.used(),
            POCKETS_SLOT_COUNT
        );
    } else {
        // A new game owes the starter house (mPlayer_DEBT0).
        inventory.loan = rustimal_logic::house::DEBT_BUY_HOUSE;
    }
}

/// Startup: restore the active quest from the loaded save.
fn apply_loaded_quest(loaded: Res<LoadedSave>, mut quest: ResMut<crate::quest::QuestState>) {
    if let Some(save) = loaded.0.as_ref() {
        quest.active = save.quest.clone();
        if quest.active.is_some() {
            println!("Quest restored from save");
        }
    }
}

/// Gather the current game state into a `SaveData`.
///
/// If the player is inside the house, the saved position is the town-map
/// spot they entered from (interior coordinates are meaningless on load).
fn collect_save(
    player: &Query<&Transform, With<Player>>,
    field: Option<&FieldItems>,
    inventory: &PlayerInventory,
    playtime: &Playtime,
    seed: &TownSeed,
    house_state: &State<HouseState>,
    return_pos: Option<&ReturnPosition>,
    shop_loc: &State<crate::shop::ShopLocation>,
    shop_return: Option<&crate::shop::ShopReturnPos>,
    shop: &crate::shop::ShopData,
    mailbox: &Mailbox,
    quest: &crate::quest::QuestState,
) -> SaveData {
    let player_pos = if *house_state == HouseState::Inside {
        return_pos
            .map(|r| r.0.to_array())
            .unwrap_or([0.0, 1.0, 0.0])
    } else if *shop_loc != crate::shop::ShopLocation::Town {
        shop_return
            .map(|r| r.0.to_array())
            .unwrap_or([0.0, 1.0, 0.0])
    } else {
        player
            .single()
            .map(|t| t.translation.to_array())
            .unwrap_or([0.0, 1.0, 0.0])
    };
    SaveData {
        version: SAVE_VERSION,
        identity: PlayerIdentity::default(),
        town_seed: seed.0,
        player_pos,
        bells: inventory.bells,
        loan: inventory.loan,
        house: None,
        police_box: None,
        police_grow_day: None,
        museum: None,
        post: None,
        villager_memory: Vec::new(),
        house_mail: Vec::new(),
        catalog: None,
        pockets: Some(inventory.pockets.pockets.to_vec()),
        pocket_conds: inventory.pockets.item_conditions,
        fruit: 0,
        fish: 0,
        bugs: 0,
        playtime_secs: playtime.secs,
        picked_items: Vec::new(),
        field_items: field.map(FieldItems::changes),
        dust_flag: field.is_some_and(|f| f.dust_flag),
        letters: mailbox.letters.clone(),
        sent_letters: mailbox.sent.clone(),
        shop_stock: shop.stock.clone(),
        shop_sales: shop.shop.sales_sum,
        shop_tier: shop.shop.shop_level as u8,
        shop_progress: Some(crate::shop::schedule::Progress {
            played: shop.played,
            ..crate::shop::schedule::Progress::from(&shop.shop)
        }),
        shop_events: Some(shop.events.clone()),
        shop_turnips: Some(shop.turnips.clone()),
        furniture: Vec::new(),
        home_furniture: None,
        home_floors: None,
        item_numbers: ITEM_NUMBERS,
        shop_priorities: shop.priorities,
        shop_stock_day: shop.stock_day,
        shop_paint: shop.paint,
        shop_rare: shop.rare_item,
        quest: quest.active.clone(),
        player_face: None,
        player_cloth: None,
        villagers: None,
        villagers_appeared: Vec::new(),
        villagers_met: Vec::new(),
        last_move_in_day: None,
        villager_slots: Vec::new(),
        last_remove_day: None,
        remove_slot: None,
    }
}

/// Build the top-center notification text (empty = invisible).
fn setup_notify(mut commands: Commands) {
    commands.spawn((
        NotifyText,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(28.0),
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(56.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            ..default()
        },
    ));
}

fn show_notification(
    msg: &str,
    timer: &mut ResMut<NotifyTimer>,
    mut query: Query<&mut Text, With<NotifyText>>,
) {
    if let Ok(mut text) = query.single_mut() {
        text.0 = msg.to_string();
    }
    timer.0 = NOTIFY_DURATION_SECS;
}

fn clear_notification(mut query: Query<&mut Text, With<NotifyText>>) {
    if let Ok(mut text) = query.single_mut() {
        text.0.clear();
    }
}

/// Hide the notification once its timer expires.
fn tick_notification(
    time: Res<Time>,
    mut timer: ResMut<NotifyTimer>,
    query: Query<&mut Text, With<NotifyText>>,
) {
    if timer.0 > 0.0 {
        timer.0 -= time.delta().as_secs_f32();
        if timer.0 <= 0.0 {
            clear_notification(query);
        }
    }
}

/// F5: write the current state to disk.
fn quick_save(
    // One tuple param: these systems are at Bevy's 16-parameter limit.
    (actions, focus, mut request): (
        Res<Actions>,
        Res<InputFocus>,
        ResMut<crate::pause_menu::SaveRequest>,
    ),
    player: Query<&Transform, With<Player>>,
    field: Option<Res<FieldItems>>,
    inventory: Res<PlayerInventory>,
    playtime: Res<Playtime>,
    seed: Res<TownSeed>,
    house_state: Res<State<HouseState>>,
    return_pos: Option<Res<ReturnPosition>>,
    shop_loc: Res<State<crate::shop::ShopLocation>>,
    shop_return: Option<Res<crate::shop::ShopReturnPos>>,
    shop: Res<crate::shop::ShopData>,
    mailbox: Res<Mailbox>,
    quest: Res<crate::quest::QuestState>,
    mut timer: ResMut<NotifyTimer>,
    notify_query: Query<&mut Text, With<NotifyText>>,
    (
        look,
        roster,
        identity,
        room,
        home,
        floors,
        lost,
        museum,
        grow,
        post,
        memory,
        house_mail,
        catalog,
    ): (
        Res<PlayerLook>,
        Option<Res<crate::roster::TownRoster>>,
        Res<PlayerIdentity>,
        Res<crate::player_room::PlayerRoom>,
        Option<Res<crate::home::PlayerHouse>>,
        Res<crate::player_room::floors::HomeFloors>,
        Option<Res<crate::police_box::LostAndFound>>,
        Option<Res<crate::museum::MuseumCollection>>,
        Option<Res<crate::police_box::LostGrowDay>>,
        Option<Res<crate::post::Post>>,
        Option<Res<crate::villager_memory::VillagerMemory>>,
        Option<Res<crate::mail::HouseMailbox>>,
        Option<Res<crate::catalog::Catalog>>,
    ),
) {
    // Furniture movement already has an atomic committed destination. Other
    // modal UIs must close before saving to avoid retaining stale UI state.
    // F5, or Save in the pause menu (taken once play resumes).
    if !matches!(*focus, InputFocus::World | InputFocus::Furniture) {
        return;
    }
    let asked = std::mem::take(&mut request.0);
    if !(actions.just_pressed(Action::QuickSave) || asked) {
        return;
    }
    let mut save = collect_save(
        &player,
        field.as_deref(),
        &inventory,
        &playtime,
        &seed,
        &house_state,
        return_pos.as_deref(),
        &shop_loc,
        shop_return.as_deref(),
        &shop,
        &mailbox,
        &quest,
    );
    save.player_face = Some(look.face as u8);
    save.identity = identity.clone();
    let (main, extra) = floors.snapshot(&room);
    save.home_furniture = main;
    save.home_floors = extra.iter().any(Option::is_some).then_some(extra);
    save.house = home.map(|h| h.0);
    save.police_box = lost.map(|l| l.0);
    save.police_grow_day = grow.and_then(|g| g.0);
    save.museum = museum.map(|m| m.save());
    save.post = post.map(|p| p.clone());
    save.villager_memory = memory
        .map(|m| m.0.iter().cloned().collect())
        .unwrap_or_default();
    save.house_mail = house_mail.map(|h| h.0.letters.clone()).unwrap_or_default();
    save.catalog = catalog.map(|c| c.save());
    save.player_cloth = Some(look.cloth as u16);
    if let Some(r) = roster {
        save.villagers = Some(r.residents.iter().map(|e| e.index).collect());
        save.villagers_appeared = r.appeared.clone();
        save.villagers_met = r.met.clone();
        save.last_move_in_day = r.last_grow_day;
        save.villager_slots = r.slots.clone();
        save.last_remove_day = r.remove_day;
        save.remove_slot = r.leaver;
    }
    if write_save(&save) {
        println!("Saved to {}", save_path().display());
        show_notification("Saved!", &mut timer, notify_query);
    } else {
        println!("Save FAILED");
        show_notification("Save failed!", &mut timer, notify_query);
    }
}

/// F9: read the save and apply it — player position, inventory, playtime,
/// and ground items (despawn all, respawn everything not marked picked).
fn quick_load(
    // One tuple param: these systems are at Bevy's 16-parameter limit.
    (actions, focus, mut audio_reset, mut net, mut identity, mut room, disc): (
        Res<Actions>,
        Res<InputFocus>,
        MessageWriter<crate::sound::ResetAudio>,
        ResMut<crate::player_tools::NetAction>,
        ResMut<PlayerIdentity>,
        ResMut<crate::player_room::PlayerRoom>,
        Option<Res<crate::game_disc::GameDisc>>,
    ),
    mut player_query: Query<&mut Transform, With<Player>>,
    field: Option<ResMut<FieldItems>>,
    mut inventory: ResMut<PlayerInventory>,
    mut playtime: ResMut<Playtime>,
    mut mailbox: ResMut<Mailbox>,
    mut next_house: ResMut<NextState<HouseState>>,
    mut next_shop: ResMut<NextState<crate::shop::ShopLocation>>,
    mut shop: ResMut<crate::shop::ShopData>,
    mut quest: ResMut<crate::quest::QuestState>,
    mut timer: ResMut<NotifyTimer>,
    notify_query: Query<&mut Text, With<NotifyText>>,
    (home, mut floors, mut commands, lost, museum, post, memory, house_mail, catalog, mut rank): (
        Option<ResMut<crate::home::PlayerHouse>>,
        ResMut<crate::player_room::floors::HomeFloors>,
        Commands,
        Option<ResMut<crate::police_box::LostAndFound>>,
        Option<ResMut<crate::museum::MuseumCollection>>,
        Option<ResMut<crate::post::Post>>,
        Option<ResMut<crate::villager_memory::VillagerMemory>>,
        Option<ResMut<crate::mail::HouseMailbox>>,
        Option<ResMut<crate::catalog::Catalog>>,
        ResMut<crate::field_rank::FieldRank>,
    ),
) {
    if !actions.just_pressed(Action::QuickLoad)
        || !matches!(*focus, InputFocus::World | InputFocus::Furniture)
    {
        return;
    }
    let Some(save) = read_save() else {
        show_notification("No save found!", &mut timer, notify_query);
        return;
    };
    net.cancel();
    *identity = save.identity.clone();
    floors.restore(save.home_floors.as_ref());
    if let (Some(mut lost), Some(kept)) = (lost, save.police_box) {
        lost.0 = kept;
    }
    if let Some(mut museum) = museum {
        museum.restore(save.museum.as_ref());
    }
    if let Some(mut post) = post {
        *post = save.post.clone().unwrap_or_default();
    }
    if let Some(mut memory) = memory {
        memory.0 = save.villager_memory.iter().cloned().collect();
    }
    if let Some(mut house_mail) = house_mail {
        house_mail.0.letters = save.house_mail.clone();
    }
    if let Some(mut catalog) = catalog {
        catalog.restore(save.catalog.as_ref());
    }
    room.restore(save.home_furniture.as_ref(), disc.as_deref());
    if disc.is_some() {
        crate::house::refresh_player_room_bounds(&mut commands);
    }

    if let Ok(mut transform) = player_query.single_mut() {
        transform.translation = Vec3::from_array(save.player_pos);
    }
    // Loading always drops the player back on the town map.
    next_house.set(HouseState::Town);
    next_shop.set(crate::shop::ShopLocation::Town);
    inventory.bells = save.bells;
    inventory.loan = save.loan;
    inventory.pockets = pockets_from_save(&save);
    if let Some(mut home) = home {
        crate::home::restore(&mut home, save.house, &mut inventory);
    }
    playtime.secs = save.playtime_secs;
    mailbox.letters = save.letters.clone();
    mailbox.sent = save.sent_letters.clone();
    shop.restore(&save);
    // Quest: restore the active quest (markers regenerate via cooldown).
    quest.active = save.quest.clone();
    if quest.active.is_some() {
        println!("Quest restored from save");
    }

    if let Some(mut field) = field {
        field.restore(save.field_items.as_deref().unwrap_or_default());
        field.dust_flag = save.dust_flag;
        rank.day = None;
    }

    audio_reset.write(crate::sound::ResetAudio);
    println!("Loaded save (seed {})", save.town_seed);
    show_notification("Loaded!", &mut timer, notify_query);
}

/// Accumulate playtime every frame.
fn tick_playtime(time: Res<Time>, mut playtime: ResMut<Playtime>) {
    playtime.secs += time.delta().as_secs_f32() as f64;
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct LoadSet;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NotifyTimer::default())
            .add_systems(PreStartup, init_save)
            .add_systems(Startup, setup_notify)
            .add_systems(
                Startup,
                (
                    apply_loaded_player,
                    apply_loaded_inventory,
                    apply_loaded_quest,
                )
                    .after(crate::player::spawn_player),
            )
            .add_systems(
                Update,
                (
                    tick_playtime,
                    assign_player_id,
                    renew_personal_events,
                    quick_save,
                    quick_load.in_set(LoadSet),
                    tick_notification,
                )
                    .in_set(GameplaySet),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn old_save(extra: &str) -> SaveData {
        let json = format!(
            r#"{{"version":1,"town_seed":7,"player_pos":[0.0,1.0,0.0],"bells":200,
                "fruit":2,"fish":1,"bugs":3,"playtime_secs":0.0,"picked_items":[]{extra}}}"#
        );
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn old_save_counts_become_pocket_items() {
        let save = old_save(r#","furniture":["Bookshelf"]"#);
        let inv = pockets_from_save(&save);
        assert_eq!(inv.count_item(items::FRUIT), 2);
        assert_eq!(inv.count_item(items::fish(FishType::CrucianCarp)), 1);
        assert_eq!(inv.count_item(items::bug(InsectType::CommonButterfly)), 3);
        assert_eq!(inv.count_item(legacy_furniture("Bookshelf").unwrap()), 1);
        assert_eq!(inv.pockets.iter().filter(|&&i| i != 0).count(), 7);
    }

    #[test]
    fn old_save_with_delivery_gets_its_parcel() {
        let save = old_save(
            r#","quest":{"kind":{"Delivery":{"giver":"A","recipient":"B"}},"reward":200,"fish_baseline":0,"bugs_baseline":0}"#,
        );
        assert_eq!(pockets_from_save(&save).count_item(items::PARCEL), 1);
    }

    #[test]
    fn new_save_pockets_round_trip() {
        let mut save = old_save("");
        let mut slots = vec![0u16; POCKETS_SLOT_COUNT];
        slots[4] = items::FRUIT;
        save.pockets = Some(slots);
        save.pocket_conds = 0b11 << 8;
        let inv = pockets_from_save(&save);
        assert_eq!(inv.pockets[4], items::FRUIT);
        assert_eq!(inv.item_cond(4), 3);
        assert_eq!(inv.count_item(items::FRUIT), 1); // legacy counts ignored
    }
}
