//! The intro's `aNPC_check_manpu_demoCode` reactions. Scripted clips keep ownership of movement.
use bevy::prelude::*;
use disc::RomSource;

use crate::character::{CharacterAnimationSet, NpcClip, NpcGait, NpcModel};
use crate::dialogue::DialogueState;
use crate::game_disc::GameDisc;
use crate::intro::Intro;
use crate::msg_window::{DemoOrders, RevealSet};

#[derive(Component)]
pub(crate) struct DialogueReaction {
    pub(crate) clip: NpcClip,
    pub(crate) effect: Option<(i16, u32)>,
    next: Option<&'static str>,
    trigger: Option<(i16, f32)>,
    generation: u32,
    base: (Option<&'static str>, u32),
}

// aNPC_manpu_anime_init_tbl / aNPC_manpu_anime_tbl, bounded to the actual intro messages.
fn clips(order: u16) -> Option<(&'static str, Option<&'static str>)> {
    macro_rules! pair {
        ($a:literal, $b:literal) => {
            Some((
                concat!("cKF_ba_r_npc_1_", $a),
                Some(concat!("cKF_ba_r_npc_1_", $b)),
            ))
        };
    }
    match order {
        2 => pair!("gaaan1", "gaaan2"),
        3 => pair!("smile1", "smile2"),
        7 => pair!("aseru1", "aseru2"),
        11 => pair!("hate1", "hate2"),
        12 => pair!("hirameki1", "hirameki2"),
        15 => pair!("muuuuu1", "muuuuu2"),
        21 => Some(("cKF_ba_r_npc_1_niko1", None)),
        23 => Some(("cKF_ba_r_npc_1_komari1", None)),
        24 => pair!("smile_d1", "smile_d2"),
        25 => pair!("gaaan_d1", "gaaan_d2"),
        26 => pair!("hirameki_d1", "hirameki_d2"),
        27 => pair!("ha_d1", "ha_d2"),
        28 => Some(("cKF_ba_r_npc_1_musu_d1", None)),
        29 => Some(("cKF_ba_r_npc_1_niko_d1", None)),
        30 => Some(("cKF_ba_r_npc_1_komari_d1", None)),
        31 => pair!("hate_d1", "hate_d2"),
        254 => Some(("cKF_ba_r_npc_1_sitdown_wait_d1", None)),
        255 => Some(("cKF_ba_r_npc_1_wait1", None)),
        _ => None,
    }
}

impl DialogueReaction {
    fn start(&mut self, disc: &GameDisc, name: &'static str) -> bool {
        let Some(asset) = disc.catalog.get(name) else {
            return false;
        };
        let Some(data) = disc.rom.slice(RomSource::Rel, asset.offset, 0x38) else {
            return false;
        };
        let float = |i| f32::from_be_bytes(data[i..i + 4].try_into().unwrap());
        let short = |i| i16::from_be_bytes(data[i..i + 2].try_into().unwrap());
        let mode = i32::from_be_bytes(data[0x1C..0x20].try_into().unwrap());
        // cKF morph counters advance at 0.5 per 60-Hz tick; the shared NPC blender uses ticks.
        self.clip.start(name, mode == 0, float(0x20).abs() * 2.0);
        self.clip.range = Some((float(0x14), float(0x18)));
        // Intro tables contain no per-animation effect tables: these are the native feel effects.
        self.trigger =
            (short(0x34) >= 0 && short(0x36) >= 0).then_some((short(0x36), f32::from(short(0x34))));
        true
    }
}

fn update(
    mut commands: Commands,
    dialogue: Res<DialogueState>,
    intro: Res<Intro>,
    disc: Option<Res<GameDisc>>,
    mut orders: ResMut<DemoOrders>,
    mut actors: Query<(
        Entity,
        &NpcModel,
        &NpcClip,
        &NpcGait,
        Option<&mut DialogueReaction>,
    )>,
    mut generation: Local<u32>,
    mut loads: MessageReader<crate::sound::ResetAudio>,
) {
    let loaded = loads.read().count() != 0;
    let running = !loaded && matches!(*intro, Intro::Train | Intro::Houses);
    let order = if running {
        let value = orders.get(1, 0);
        orders.clear(1, 0);
        value
    } else {
        0
    };
    for (entity, model, base, gait, reaction) in &mut actors {
        let eligible = running
            && matches!(model.0, 242 | 258)
            && dialogue.active
            && dialogue.speaker == Some(entity)
            && gait.0 == 0
            && matches!(base.name, None | Some("cKF_ba_r_npc_1_sitdown_wait_d1"));
        let base_key = (base.name, base.revision);
        if !eligible {
            if reaction.is_some() {
                commands.entity(entity).remove::<DialogueReaction>();
            }
            continue;
        }
        let Some(disc) = disc.as_deref() else {
            continue;
        };
        if let Some((name, next)) = clips(order) {
            // aNPC_Animation_init preserves the current frame for the same seqNo.
            if reaction.as_ref().is_some_and(|r| r.clip.name == Some(name)) {
                continue;
            }
            *generation = generation.wrapping_add(1);
            let mut new = DialogueReaction {
                clip: NpcClip::default(),
                effect: None,
                next,
                trigger: None,
                generation: *generation,
                base: base_key,
            };
            new.clip.revision = *generation;
            if new.start(disc, name) {
                if let Some(retained) =
                    reaction
                        .as_ref()
                        .and_then(|r| r.effect)
                        .filter(|(kind, _)| {
                            matches!(kind, 3 | 10 | 30)
                                && new.trigger.is_some_and(|(new_kind, _)| new_kind == *kind)
                        })
                {
                    // eEC_Name2EffectMake deduplicates continuous effects by type and npc_id.
                    new.effect = Some(retained);
                    new.trigger = None;
                }
                info!("intro reaction {order}: {name}");
                commands.entity(entity).insert(new);
            }
        } else if let Some(mut reaction) = reaction {
            if reaction.base != base_key {
                commands.entity(entity).remove::<DialogueReaction>();
                continue;
            }
            if let Some((effect, frame)) = reaction.trigger {
                let (first, last) = reaction.clip.range.unwrap_or((1.0, 2.0));
                if first + reaction.clip.progress * (last - first) >= frame {
                    reaction.effect = Some((effect, reaction.generation));
                    reaction.trigger = None;
                }
            }
            if reaction.clip.done {
                if let Some(next) = reaction.next.take() {
                    reaction.start(disc, next);
                }
            }
        }
    }
}

pub(crate) fn install(app: &mut App) {
    app.add_systems(
        Update,
        update
            .after(RevealSet)
            .after(crate::save::LoadSet)
            .after(crate::input::InteractSet::Talk)
            .after(crate::intro_house::run)
            .before(CharacterAnimationSet),
    );
    crate::intro_reaction_effects::register(app);
}
