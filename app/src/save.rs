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

/// Serializable snapshot of game state.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SaveData {
    pub version: u32,
    pub town_seed: u32,
    pub player_pos: [f32; 3],
    pub bells: u32,
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
    /// Legacy furniture names (pre-pockets); read for conversion only.
    #[serde(default)]
    pub furniture: Vec<String>,
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
fn init_save(mut commands: Commands) {
    let loaded = read_save();
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
fn apply_loaded_inventory(loaded: Res<LoadedSave>, mut inventory: ResMut<PlayerInventory>) {
    if let Some(save) = loaded.0.as_ref() {
        inventory.bells = save.bells;
        inventory.pockets = pockets_from_save(save);
        println!(
            "Inventory restored: {} bells, {}/{} pockets",
            save.bells,
            inventory.used(),
            POCKETS_SLOT_COUNT
        );
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
    } else if *shop_loc == crate::shop::ShopLocation::Inside {
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
        town_seed: seed.0,
        player_pos,
        bells: inventory.bells,
        pockets: Some(inventory.pockets.pockets.to_vec()),
        pocket_conds: inventory.pockets.item_conditions,
        fruit: 0,
        fish: 0,
        bugs: 0,
        playtime_secs: playtime.secs,
        picked_items: Vec::new(),
        field_items: field.map(FieldItems::changes),
        letters: mailbox.letters.clone(),
        sent_letters: mailbox.sent.clone(),
        shop_stock: shop.stock.clone(),
        shop_sales: shop.shop.sales_sum,
        shop_tier: shop.shop.shop_level as u8,
        furniture: Vec::new(),
        item_numbers: ITEM_NUMBERS,
        shop_priorities: shop.priorities,
        shop_stock_day: shop.stock_day,
        shop_paint: shop.paint,
        quest: quest.active.clone(),
        player_face: None,
        player_cloth: None,
        villagers: None,
        villagers_appeared: Vec::new(),
        villagers_met: Vec::new(),
        last_move_in_day: None,
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
    (look, roster): (Res<PlayerLook>, Option<Res<crate::roster::TownRoster>>),
) {
    // World focus only: saving or loading mid-UI would leave that UI stale.
    // F5, or Save in the pause menu (taken once play resumes).
    if !focus.is_world() {
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
    save.player_cloth = Some(look.cloth as u16);
    if let Some(r) = roster {
        save.villagers = Some(r.residents.iter().map(|e| e.index).collect());
        save.villagers_appeared = r.appeared.clone();
        save.villagers_met = r.met.clone();
        save.last_move_in_day = r.last_grow_day;
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
    (actions, focus, mut audio_reset): (
        Res<Actions>,
        Res<InputFocus>,
        MessageWriter<crate::sound::ResetAudio>,
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
) {
    if !actions.just_pressed(Action::QuickLoad) || !focus.is_world() {
        return;
    }
    let Some(save) = read_save() else {
        show_notification("No save found!", &mut timer, notify_query);
        return;
    };

    if let Ok(mut transform) = player_query.single_mut() {
        transform.translation = Vec3::from_array(save.player_pos);
    }
    // Loading always drops the player back on the town map.
    next_house.set(HouseState::Town);
    next_shop.set(crate::shop::ShopLocation::Town);
    inventory.bells = save.bells;
    inventory.pockets = pockets_from_save(&save);
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
