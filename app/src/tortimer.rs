//! Tortimer (`SP_NPC_SONCHO`), the mayor, who stands at the shrine while the part-time job
//! runs (`m_field_make.c` places him only for `mEv_CheckFirstJob()`, at unit (10, 10) of the
//! shrine block, `fd_npc_land.c`). Talking to him (`ac_npc_soncho_talk.c_inc`) sets the
//! "arbeit" trophy at once (`mSC_trophy_set(mSC_SPECIAL_EVENT_ARBEIT)`), which the
//! introductions job needs (`mSC_check_ArbeitPlayer`); his message is 0x3AFC the first time and
//! 0x3AFF + RANDOM(3) afterwards.

use bevy::prelude::*;

use crate::character::{NpcClip, NpcGait};
use crate::dialogue::DialogueState;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InputFocus, InteractSet};
use crate::interaction::Facing;
use crate::player::Player;
use crate::shop::ShopLocation;
use crate::states::GameplaySet;
use crate::town::Town;

/// ALL_NPC_NUM + (SP_NPC_SONCHO - SP_NPC_START) = 238 + 127; `l_sp_actor_name` sound 2.
const TORTIMER_NPC: u32 = 238 + 127;
const TORTIMER_SOUND: u8 = 2;

#[derive(Component)]
struct Tortimer;

/// Out only while the job runs, in the town.
fn visit(
    mut commands: Commands,
    town: Res<Town>,
    identity: Res<crate::save::PlayerIdentity>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    existing: Query<Entity, With<Tortimer>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let out = identity.events.first_job
        && *state.get() == ShopLocation::Town
        && *house.get() == crate::house::HouseState::Town;
    if !out {
        for entity in &existing {
            commands.entity(entity).despawn();
        }
        return;
    }
    if !existing.is_empty() {
        return;
    }
    // The shrine block: the wishing well's acre.
    let Some(acre) = town
        .plan
        .acres
        .iter()
        .position(|a| a.feature == rustimal_logic::town_gen::Feature::WishingWell as u8)
    else {
        return;
    };
    let width = rustimal_logic::town_gen::ACRE_WIDTH;
    let at = Town::cell_to_world(acre % width, acre / width, 10, 10);
    let at = at.with_y(town.ground_y(at));
    commands.spawn((
        Name::new("Tortimer"),
        Tortimer,
        NpcClip::default(),
        NpcGait::default(),
        crate::character::NpcModel(TORTIMER_NPC),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        Transform::from_translation(at + Vec3::Y),
    ));
    info!("Tortimer is at the shrine {at:?}");
}

/// He turns to face the player when they are near.
fn face(
    time: Res<Time>,
    dialogue: Res<DialogueState>,
    player: Query<&Transform, (With<Player>, Without<Tortimer>)>,
    mut mayor: Query<(Entity, &mut Transform), With<Tortimer>>,
) {
    let (Ok(at), Ok((entity, mut transform))) = (player.single(), mayor.single_mut()) else {
        return;
    };
    let to = (at.translation - transform.translation).with_y(0.0);
    if dialogue.speaker == Some(entity) && to.length() > 0.01 {
        let want = Quat::from_rotation_y(to.x.atan2(to.z));
        transform.rotation = transform
            .rotation
            .slerp(want, (time.delta_secs() * 8.0).min(1.0));
    }
}

#[allow(clippy::too_many_arguments)]
fn talk(
    disc: Option<Res<GameDisc>>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    player: Query<(&Transform, &Facing), With<Player>>,
    mayor: Query<(Entity, &Transform), With<Tortimer>>,
    mut dialogue: ResMut<DialogueState>,
    mut identity: ResMut<crate::save::PlayerIdentity>,
    mut rand: ResMut<crate::game_rand::GameRand>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    if *state.get() != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    let Ok((actor, actor_at)) = mayor.single() else {
        return;
    };
    // His answer ends the talk (only his own: others take theirs).
    if dialogue.speaker == Some(actor) && dialogue.external_result.take().is_some() {
        *dialogue = DialogueState::default();
        *focus = InputFocus::World;
        return;
    }
    if !focus.is_world() || !actions.just_pressed(Action::Interact) {
        return;
    }
    let Ok((at, facing)) = player.single() else {
        return;
    };
    let to = (actor_at.translation - at.translation).with_y(0.0);
    if to.length() > 1.8 || facing.dir.dot(to.normalize_or_zero()) < 0.5 {
        return;
    }
    actions.consume(Action::Interact);
    // `aNS_set_norm_talk_info`: 0x3AFF + RANDOM(3) once he has been met, else 0x3AFC. (The
    // lighthouse trip's 0x3B02 needs the island, which is not built.)
    let msg = if identity.arbeit_trophy {
        0x3AFF + rand.0.random(3) as usize
    } else {
        0x3AFC
    };
    // `aNS_talk_init`: the trophy is set as the talk starts.
    if !identity.arbeit_trophy {
        identity.arbeit_trophy = true;
        info!("Tortimer: the arbeit trophy is set");
    }
    let ctx = crate::messages::MsgContext {
        player_name: identity.player.clone(),
        talk_name: "Tortimer".into(),
        free: vec![String::new(); 20],
        items: vec![String::new(); 5],
        item_articles: vec![0; 5],
        ..default()
    };
    if let Some(text) = disc.text.as_ref() {
        let (pages, asked) = text.talk(msg, &ctx, &mut |_| 0);
        dialogue.begin_special(
            actor,
            "Tortimer",
            (TORTIMER_NPC, TORTIMER_SOUND),
            pages,
            asked.map(|c| c.labels),
        );
        *focus = InputFocus::Dialogue;
        info!("Tortimer message {msg:#06x}");
    }
}

pub struct TortimerPlugin;

impl Plugin for TortimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (visit, face).in_set(GameplaySet))
            .add_systems(Update, talk.in_set(InteractSet::Talk).in_set(GameplaySet));
    }
}
