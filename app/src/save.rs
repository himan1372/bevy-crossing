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
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::house::{HouseState, ReturnPosition};
use crate::interaction::{spawn_ground_item, GroundItem, PlayerInventory, ITEM_SPOTS};
use crate::mail::Mailbox;
use crate::player::Player;
use crate::town::{Town, TownSeed};

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
    pub fruit: u32,
    /// Fish caught (added after v1; defaults to 0 for old saves).
    #[serde(default)]
    pub fish: u32,
    /// Bugs caught (added after v1; defaults to 0 for old saves).
    #[serde(default)]
    pub bugs: u32,
    pub playtime_secs: f64,
    /// Indices into `ITEM_SPOTS` that had been picked up when saving.
    pub picked_items: Vec<usize>,
    /// Received letters (added later; defaults to empty for old saves).
    #[serde(default)]
    pub letters: Vec<crate::mail::Letter>,
    /// Sent letters (added later; defaults to empty for old saves).
    #[serde(default)]
    pub sent_letters: Vec<crate::mail::Letter>,
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
        }
        None => {
            println!("No save found, starting fresh");
            commands.insert_resource(TownSeed(12345));
            commands.insert_resource(Playtime::default());
        }
    }
    commands.insert_resource(LoadedSave(loaded));
}

/// Startup (after `spawn_player`): move the player to the saved position.
fn apply_loaded_player(loaded: Res<LoadedSave>, mut player: Query<&mut Transform, With<Player>>) {
    let Some(save) = loaded.0.as_ref() else {
        return;
    };
    if let Ok(mut transform) = player.get_single_mut() {
        transform.translation = Vec3::from_array(save.player_pos);
        println!("Player position restored");
    }
}

/// Startup: restore inventory counts from the loaded save.
fn apply_loaded_inventory(loaded: Res<LoadedSave>, mut inventory: ResMut<PlayerInventory>) {
    if let Some(save) = loaded.0.as_ref() {
        inventory.bells = save.bells;
        inventory.fruit = save.fruit;
        inventory.fish = save.fish;
        inventory.bugs = save.bugs;
        println!(
            "Inventory restored: {} bells, {} fruit, {} fish, {} bugs",
            save.bells, save.fruit, save.fish, save.bugs
        );
    }
}

/// Gather the current game state into a `SaveData`.
///
/// If the player is inside the house, the saved position is the town-map
/// spot they entered from (interior coordinates are meaningless on load).
fn collect_save(
    player: &Query<&Transform, With<Player>>,
    items: &Query<&GroundItem>,
    inventory: &PlayerInventory,
    playtime: &Playtime,
    seed: &TownSeed,
    house_state: &State<HouseState>,
    return_pos: Option<&ReturnPosition>,
    mailbox: &Mailbox,
) -> SaveData {
    let player_pos = if *house_state == HouseState::Inside {
        return_pos
            .map(|r| r.0.to_array())
            .unwrap_or([0.0, 1.0, 0.0])
    } else {
        player
            .get_single()
            .map(|t| t.translation.to_array())
            .unwrap_or([0.0, 1.0, 0.0])
    };
    let remaining: Vec<usize> = items.iter().map(|i| i.index).collect();
    let picked_items: Vec<usize> = (0..ITEM_SPOTS.len())
        .filter(|i| !remaining.contains(i))
        .collect();
    SaveData {
        version: SAVE_VERSION,
        town_seed: seed.0,
        player_pos,
        bells: inventory.bells,
        fruit: inventory.fruit,
        fish: inventory.fish,
        bugs: inventory.bugs,
        playtime_secs: playtime.secs,
        picked_items,
        letters: mailbox.letters.clone(),
        sent_letters: mailbox.sent.clone(),
    }
}

/// Build the top-center notification text (empty = invisible).
fn setup_notify(mut commands: Commands) {
    let mut text = Text::from_section(
        "",
        TextStyle {
            font_size: 28.0,
            color: Color::WHITE,
            ..default()
        },
    );
    text.justify = bevy::text::JustifyText::Center;
    commands.spawn((
        NotifyText,
        TextBundle {
            text,
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(56.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                ..default()
            },
            ..default()
        },
    ));
}

fn show_notification(msg: &str, timer: &mut ResMut<NotifyTimer>, mut query: Query<&mut Text, With<NotifyText>>) {
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].value = msg.to_string();
    }
    timer.0 = NOTIFY_DURATION_SECS;
}

fn clear_notification(mut query: Query<&mut Text, With<NotifyText>>) {
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].value.clear();
    }
}

/// Hide the notification once its timer expires.
fn tick_notification(
    time: Res<Time>,
    mut timer: ResMut<NotifyTimer>,
    query: Query<&mut Text, With<NotifyText>>,
) {
    if timer.0 > 0.0 {
        timer.0 -= time.delta_seconds();
        if timer.0 <= 0.0 {
            clear_notification(query);
        }
    }
}

/// F5: write the current state to disk.
fn quick_save(
    keyboard: Res<ButtonInput<KeyCode>>,
    player: Query<&Transform, With<Player>>,
    items: Query<&GroundItem>,
    inventory: Res<PlayerInventory>,
    playtime: Res<Playtime>,
    seed: Res<TownSeed>,
    house_state: Res<State<HouseState>>,
    return_pos: Option<Res<ReturnPosition>>,
    mailbox: Res<Mailbox>,
    mut timer: ResMut<NotifyTimer>,
    notify_query: Query<&mut Text, With<NotifyText>>,
) {
    if !keyboard.just_pressed(KeyCode::F5) {
        return;
    }
    let save = collect_save(
        &player,
        &items,
        &inventory,
        &playtime,
        &seed,
        &house_state,
        return_pos.as_deref(),
        &mailbox,
    );
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
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    town: Res<Town>,
    mut player_query: Query<&mut Transform, With<Player>>,
    items_query: Query<Entity, With<GroundItem>>,
    mut inventory: ResMut<PlayerInventory>,
    mut playtime: ResMut<Playtime>,
    mut mailbox: ResMut<Mailbox>,
    mut next_house: ResMut<NextState<HouseState>>,
    mut timer: ResMut<NotifyTimer>,
    notify_query: Query<&mut Text, With<NotifyText>>,
) {
    if !keyboard.just_pressed(KeyCode::F9) {
        return;
    }
    let Some(save) = read_save() else {
        show_notification("No save found!", &mut timer, notify_query);
        return;
    };

    if let Ok(mut transform) = player_query.get_single_mut() {
        transform.translation = Vec3::from_array(save.player_pos);
    }
    // Loading always drops the player back on the town map.
    next_house.set(HouseState::Town);
    inventory.bells = save.bells;
    inventory.fruit = save.fruit;
    inventory.fish = save.fish;
    inventory.bugs = save.bugs;
    playtime.secs = save.playtime_secs;
    mailbox.letters = save.letters.clone();
    mailbox.sent = save.sent_letters.clone();

    for entity in items_query.iter() {
        commands.entity(entity).despawn();
    }
    for (index, (kind, x, z)) in ITEM_SPOTS.iter().enumerate() {
        if save.picked_items.contains(&index) {
            continue;
        }
        let pos = Vec3::new(*x, 0.3, *z);
        if !town.is_walkable(Vec3::new(*x, 0.0, *z)) {
            continue;
        }
        spawn_ground_item(&mut commands, &mut meshes, &mut materials, index, *kind, pos);
    }

    println!("Loaded save (seed {})", save.town_seed);
    show_notification("Loaded!", &mut timer, notify_query);
}

/// Accumulate playtime every frame.
fn tick_playtime(time: Res<Time>, mut playtime: ResMut<Playtime>) {
    playtime.secs += time.delta_seconds() as f64;
}

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NotifyTimer::default())
            .add_systems(PreStartup, init_save)
            .add_systems(Startup, setup_notify)
            .add_systems(
                Startup,
                (apply_loaded_player, apply_loaded_inventory).after(crate::player::spawn_player),
            )
            .add_systems(
                Update,
                (tick_playtime, quick_save, quick_load, tick_notification),
            );
    }
}
