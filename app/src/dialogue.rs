//! Phase 2: dialogue UI — bottom text box, speaker name, advance/dismiss.
//!
//! Message selection uses `rustimal_logic::dialogue_topics::talk_check_msg`
//! (the decomp-verified `aNPC_set_talk_info_talk_request_check` pool:
//! `base + looks*3 + RANDOM(3)`). The message ID is real; the displayed
//! text is a placeholder — retail dialogue lives in ARAM message scripts
//! indexed by that ID, which the Bevy port doesn't ship. The original
//! lines below stand in until real text content exists.

use bevy::prelude::*;
use rustimal_logic::dialogue_topics::talk_check_msg;

use crate::interaction::{PlayerInventory, TalkEvent};
use crate::npc::Villager;

/// Dialogue box state.
#[derive(Resource, Default)]
pub struct DialogueState {
    /// Whether the box is currently shown.
    pub active: bool,
    lines: Vec<String>,
    index: usize,
}

/// Marker for the dialogue box node.
#[derive(Component)]
struct DialogueBox;

/// Marker for the dialogue text.
#[derive(Component)]
struct DialogueText;

/// Marker for the inventory HUD text.
#[derive(Component)]
struct HudText;

/// Counter feeding the RANDOM(3) slot of `talk_check_msg`.
#[derive(Resource, Default)]
struct TalkCounter(u32);

/// Original placeholder lines per personality (see module docs).
fn placeholder_lines(personality: &str) -> Vec<String> {
    let (a, b) = match personality {
        "normal" => (
            "What a lovely day for a stroll, don't you think?",
            "I was just thinking about planting some flowers.",
        ),
        "peppy" => (
            "OMG! Hi hi! Have you seen my new shoes?!",
            "We should totally have a dance party later!",
        ),
        "lazy" => (
            "I was just thinking about snacks... wanna hear about snacks?",
            "Naps are great. Talking is okay too, I guess.",
        ),
        "jock" => (
            "Gotta stay hydrated! Want to do some laps with me?",
            "I did fifty push-ups this morning. FIFTY.",
        ),
        "cranky" => (
            "Hmph. What do you want? Make it quick.",
            "Back in my day, we didn't just walk up to people.",
        ),
        "snooty" => (
            "That outfit... interesting choice. Anyway.",
            "I only drink juice from very specific oranges.",
        ),
        _ => ("...", "..."),
    };
    vec![a.to_string(), b.to_string()]
}

/// Build the UI: hidden dialogue box + inventory HUD.
fn setup_ui(mut commands: Commands) {
    // Dialogue box — bottom of screen, hidden until a talk starts.
    commands
        .spawn((
            DialogueBox,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(24.0),
                    left: Val::Px(24.0),
                    right: Val::Px(24.0),
                    padding: UiRect::all(Val::Px(18.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.88)),
                visibility: Visibility::Hidden,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                DialogueText,
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
            ));
        });

    // Inventory HUD — top-left corner.
    commands.spawn((
        HudText,
        TextBundle {
            text: Text::from_section(
                "Bells: 0   Fruit: 0",
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            },
            ..default()
        },
    ));
    println!("Dialogue UI ready");
}

/// Start dialogue when a TalkEvent arrives.
///
/// Talks the quest system claimed (offers / turn-ins) are skipped here —
/// the quest plugin shows its own panel for those.
fn on_talk(
    mut events: EventReader<TalkEvent>,
    villagers: Query<&Villager>,
    mut dialogue: ResMut<DialogueState>,
    mut counter: ResMut<TalkCounter>,
    quest_claim: Res<crate::quest::QuestClaim>,
    mut box_query: Query<&mut Visibility, With<DialogueBox>>,
    mut text_query: Query<&mut Text, With<DialogueText>>,
) {
    for event in events.read() {
        if quest_claim.0 == Some(event.villager) {
            continue;
        }
        let Ok(villager) = villagers.get(event.villager) else {
            continue;
        };
        counter.0 = counter.0.wrapping_add(1);
        let msg_id = talk_check_msg(villager.looks, counter.0, false);

        dialogue.active = true;
        dialogue.index = 0;
        dialogue.lines = placeholder_lines(villager.personality.english_name())
            .into_iter()
            .map(|l| format!("{}: {}", villager.name, l))
            .collect();
        // Show which decomp message ID this talk resolved to.
        dialogue
            .lines
            .push(format!("(msg #{})", msg_id));

        if let Ok(mut vis) = box_query.get_single_mut() {
            *vis = Visibility::Visible;
        }
        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value = dialogue.lines[0].clone();
        }
        println!("Talking to {} (msg #{})", villager.name, msg_id);
    }
}

/// Space or E advances the dialogue; the last line dismisses it.
///
/// Runs after `interaction::interact` so the same E press that opens a
/// talk can't also advance it in the same frame.
fn dialogue_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut dialogue: ResMut<DialogueState>,
    mut box_query: Query<&mut Visibility, With<DialogueBox>>,
    mut text_query: Query<&mut Text, With<DialogueText>>,
) {
    if !dialogue.active {
        return;
    }
    if !(keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::KeyE)) {
        return;
    }
    dialogue.index += 1;
    if dialogue.index >= dialogue.lines.len() {
        dialogue.active = false;
        dialogue.lines.clear();
        dialogue.index = 0;
        if let Ok(mut vis) = box_query.get_single_mut() {
            *vis = Visibility::Hidden;
        }
        println!("Dialogue closed");
    } else if let Ok(mut text) = text_query.get_single_mut() {
        text.sections[0].value = dialogue.lines[dialogue.index].clone();
    }
}

/// Keep the HUD in sync with the inventory.
fn update_hud(
    inventory: Res<PlayerInventory>,
    mut text_query: Query<&mut Text, With<HudText>>,
) {
    if !inventory.is_changed() {
        return;
    }
    if let Ok(mut text) = text_query.get_single_mut() {
        text.sections[0].value = format!(
            "Bells: {}   Fruit: {}   Fish: {}   Bugs: {}",
            inventory.bells, inventory.fruit, inventory.fish, inventory.bugs
        );
    }
}

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DialogueState::default())
            .insert_resource(TalkCounter::default())
            .add_systems(Startup, setup_ui)
            .add_systems(Update, on_talk.after(crate::quest::QuestInputSet))
            .add_systems(Update, dialogue_input.after(crate::interaction::interact))
            .add_systems(Update, update_hud);
    }
}
