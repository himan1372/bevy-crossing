//! The new-game intro (`SCENE_START_DEMO`, `ac_intro_demo`): the train ride with Rover, the arrival,
//! Nook's house tour and the hand-off into the first job. Until the whole of it is playable a fresh
//! game does not run it (`BEVY_CROSSING_INTRO=play` is for development; `=skip` runs `finish_intro`).

use bevy::prelude::*;

use crate::interaction::PlayerInventory;
use crate::save::PlayerIdentity;

/// `ITM_MONEY_1000`, the starting bell bag of `mSDI_StartInitNew`.
const MONEY_1000: u16 = 0x2100;
/// `mPr_ITEM_COND_QUEST`.
const COND_QUEST: u32 = 2;

/// How far a fresh game's intro has got.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intro {
    /// Not running: a played game, or a fresh one before the intro is built.
    Off,
    /// On the train with Rover (`SCENE_START_DEMO`).
    Train,
    /// Coming in to the station (`ac_intro_demo`).
    Arrival,
    /// Nook's welcome, home inspection and initial payment.
    Houses,
}

/// What the intro leaves behind (`aID_*` act 11, `aNGD_scene_change_wait_init`): the part-time job is
/// on, the map is not yet the player's, the 1000-bell bag is in the pockets (marked as a quest item)
/// and the starter house is owed. Playing the intro and skipping it both end here.
pub fn finish_intro(identity: &mut PlayerIdentity, inventory: &mut PlayerInventory) {
    starting_bells(inventory);
    complete_intro(identity, inventory);
}

pub(crate) fn starting_bells(inventory: &mut PlayerInventory) {
    if inventory
        .pockets
        .find_item_with_cond(MONEY_1000, COND_QUEST)
        .is_none()
    {
        inventory.pockets.put(MONEY_1000, COND_QUEST);
    }
}

/// Finishing the played payment must not recreate the bag Nook just received.
pub(crate) fn complete_intro(identity: &mut PlayerIdentity, inventory: &mut PlayerInventory) {
    identity.intro_done = true;
    identity.events.set_first_job();
    identity.map_flag = false;
    inventory.loan = rustimal_logic::house::DEBT_BUY_HOUSE;
}

fn begin_intro(
    mut commands: Commands,
    mut identity: ResMut<PlayerIdentity>,
    mut inventory: ResMut<PlayerInventory>,
    disc: Option<Res<crate::game_disc::GameDisc>>,
) {
    let replay = std::env::var_os("BEVY_CROSSING_INTRO_ARRIVE").is_some()
        || std::env::var_os("BEVY_CROSSING_INTRO_HOUSES").is_some();
    if identity.intro_done && !replay {
        commands.insert_resource(Intro::Off);
        return;
    }
    let requested = matches!(std::env::var("BEVY_CROSSING_INTRO").as_deref(), Ok("play")) || replay;
    if requested && disc.as_ref().is_none_or(|d| d.text.is_none()) {
        finish_intro(&mut identity, &mut inventory);
        commands.insert_resource(Intro::Off);
        info!("intro assets unavailable: using playable fallback");
        return;
    }
    if replay {
        identity.intro_done = false;
        identity.events.first_job = false;
        identity.map_flag = false;
        starting_bells(&mut inventory);
        commands.insert_resource(Intro::Arrival);
        return;
    }
    match std::env::var("BEVY_CROSSING_INTRO").as_deref() {
        Ok("skip") => {
            finish_intro(&mut identity, &mut inventory);
            info!("intro skipped: the part-time job starts");
        }
        Ok("play") => {
            starting_bells(&mut inventory);
            commands.insert_resource(Intro::Train);
            return;
        }
        // The intro itself is built in later slices; a fresh game plays as before.
        _ => identity.intro_done = true,
    }
    commands.insert_resource(Intro::Off);
}

pub struct IntroPlugin;

impl Plugin for IntroPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            begin_intro
                .after(crate::save::apply_loaded_inventory)
                .after(crate::game_disc::start_disc_load),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finishing_the_intro_starts_the_job_once() {
        let mut identity = PlayerIdentity::default();
        let mut inventory = PlayerInventory::default();
        assert!(!identity.intro_done && identity.map_flag && !identity.events.first_job);
        finish_intro(&mut identity, &mut inventory);
        finish_intro(&mut identity, &mut inventory);
        assert!(identity.intro_done && identity.events.first_job && !identity.map_flag);
        assert_eq!(inventory.pockets.count_item(MONEY_1000), 1);
        assert_eq!(
            inventory.loan,
            rustimal_logic::house::DEBT_BUY_HOUSE,
            "the starter house is owed"
        );
    }

    #[test]
    fn saves_without_the_flag_count_as_played() {
        let identity: PlayerIdentity =
            serde_json::from_str(r#"{"player":"a","town":"b"}"#).expect("an old identity");
        assert!(identity.intro_done);
    }
}
