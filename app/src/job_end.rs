//! The end of the part-time job outdoors: `complete_payment_type = ARBEIT`
//! (`aID_first_job`), after the guide has sent the player out of the shop. The player plays
//! YATTA1 (`m_player_main_complete_payment.c_inc`) to the fanfare BGM_INTRO_CHORES_COMPLETE; after
//! 42 frames the green message 0x17B6 ("I'm free!") shows, with no name plate, and the player is
//! released when it is read.

use bevy::prelude::*;

use crate::character::NpcClip;
use crate::dialogue::DialogueState;
use crate::game_disc::GameDisc;
use crate::input::InputFocus;
use crate::player::Player;
use crate::shop::ShopLocation;
use crate::states::GameplaySet;

/// `BGM_INTRO_CHORES_COMPLETE` in `audio_defs.h`.
const FANFARE: u8 = 73;
/// The fanfare loops until the message is read (`Player_actor_settle_main_Complete_payment`
/// withdraws it; sequence 232 never ends by itself): this only bounds a message left open for an
/// hour.
const FANFARE_SECONDS: f32 = 3600.0;
/// `mDemo_Set_talk_window_color` in `Player_actor_Complete_payment_demo_ct`.
const WINDOW_COLOR: [u8; 4] = [185, 245, 80, 255];
/// `Player_actor_MessageControl_Complete_payment`: the message waits 42 frames.
const MESSAGE_FRAMES: f32 = 42.0;
const MORPH: f32 = 8.0;

#[derive(Default)]
enum Stage {
    #[default]
    Idle,
    /// Seconds since the player came out.
    Wait(f32),
    /// Frames of the cheer so far.
    Cheer(f32),
    Talking,
}

#[derive(Resource, Default)]
pub(crate) struct CompletePayment {
    stage: Stage,
}

impl CompletePayment {
    /// `Common_Set(complete_payment_type, ARBEIT)`.
    pub(crate) fn begin(&mut self) {
        self.stage = Stage::Wait(0.0);
    }
}

#[allow(clippy::too_many_arguments)]
fn cheer(
    time: Res<Time>,
    mut payment: ResMut<CompletePayment>,
    state: Res<State<ShopLocation>>,
    house: Res<State<crate::house::HouseState>>,
    mut focus: ResMut<InputFocus>,
    mut dialogue: ResMut<DialogueState>,
    mut fanfare: ResMut<crate::sound::Fanfare>,
    mut player: Query<(Entity, Option<&mut NpcClip>), With<Player>>,
    disc: Option<Res<GameDisc>>,
    identity: Res<crate::save::PlayerIdentity>,
) {
    if matches!(payment.stage, Stage::Idle) {
        return;
    }
    let Ok((entity, mut clip)) = player.single_mut() else {
        return;
    };
    // The outdoor player only (`Player_actor_request_proc_index_fromOutdoor`).
    if *state.get() != ShopLocation::Town || *house.get() != crate::house::HouseState::Town {
        return;
    }
    match payment.stage {
        Stage::Idle => {}
        Stage::Wait(waited) => {
            let waited = waited + time.delta_secs();
            payment.stage = if waited >= 0.8 && focus.is_world() {
                if let Some(clip) = clip.as_mut() {
                    clip.start("cKF_ba_r_ply_1_yatta1", true, MORPH);
                }
                fanfare.start(FANFARE, FANFARE_SECONDS);
                info!("complete payment: the cheer starts");
                Stage::Cheer(0.0)
            } else {
                Stage::Wait(waited)
            };
        }
        Stage::Cheer(frames) => {
            let frames = frames + time.delta_secs() * 60.0;
            if frames < MESSAGE_FRAMES {
                payment.stage = Stage::Cheer(frames);
                return;
            }
            let Some(text) = disc.as_ref().and_then(|d| d.text.as_ref()) else {
                payment.stage = Stage::Idle;
                return;
            };
            let ctx = crate::messages::MsgContext {
                player_name: identity.player.clone(),
                free: vec![String::new(); 20],
                items: vec![String::new(); 5],
                item_articles: vec![0; 5],
                ..default()
            };
            let (pages, _) = text.talk(0x17B6, &ctx, &mut |_| 0);
            dialogue.begin_special(entity, "", (0, 0), pages, None);
            dialogue.window_color = Some(WINDOW_COLOR);
            *focus = InputFocus::Dialogue;
            payment.stage = Stage::Talking;
            info!("complete payment: message 0x17b6");
        }
        Stage::Talking => {
            if dialogue.speaker == Some(entity) && dialogue.external_result.take().is_some() {
                *dialogue = DialogueState::default();
                *focus = InputFocus::World;
                if let Some(clip) = clip.as_mut() {
                    clip.stop(MORPH);
                }
                // `Player_actor_settle_main_Complete_payment`: the fanfare is withdrawn.
                fanfare.stop();
                payment.stage = Stage::Idle;
                info!("complete payment: done");
            }
        }
    }
}

pub struct JobEndPlugin;

impl Plugin for JobEndPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CompletePayment>()
            .init_resource::<crate::sound::Fanfare>()
            .add_systems(Update, cheer.in_set(GameplaySet));
    }
}
