//! mTG's supported three-line letter/delivery descriptions.
use super::*;

#[derive(Clone)]
pub(in crate::pockets_disc) struct Label {
    pub lines: Vec<String>,
    /// 0 ordinary, 1 normal quest pointer, 2 upper pointer, 3 lower pointer.
    pub special: u8,
}

fn colored(name: &str, rgb: [u8; 3]) -> String {
    use crate::messages::markup::{arg, COLOR_CHARS};
    format!(
        "{COLOR_CHARS}{}{}{}{}{name}",
        arg(rgb[0]),
        arg(rgb[1]),
        arg(rgb[2]),
        arg(name.chars().count().min(255) as u8)
    )
}

fn placement(mail_row: Option<usize>, pocket_row: Option<usize>) -> u8 {
    if pocket_row == Some(0) || mail_row == Some(2) {
        2
    } else if mail_row == Some(1) {
        3
    } else {
        1
    }
}

impl Label {
    pub(in crate::pockets_disc) fn ordinary(label: &str) -> Self {
        Self {
            lines: vec![label.to_string()],
            special: 0,
        }
    }

    pub(in crate::pockets_disc) fn mail(
        letter: &crate::mail::Letter,
        player: &str,
        row: usize,
    ) -> Self {
        use crate::mail::Recipient;
        let museum_to = matches!(letter.to, Some(Recipient::Museum));
        let recipient = letter.to.as_ref().map_or(player, Recipient::name);
        let recipient = colored(
            recipient,
            if museum_to {
                [60, 150, 65]
            } else {
                [205, 40, 40]
            },
        );
        let recipient = if museum_to {
            format!("the {recipient}")
        } else {
            recipient
        };
        let (sender, color) = match letter.sender.as_str() {
            crate::post::MOM_NAME => ("home", [60, 50, 155]),
            crate::post::MUSEUM_NAME => (crate::post::MUSEUM_NAME, [60, 150, 65]),
            "Nook" | "Tom Nook" | "Redd" | "Crazy Redd" => (letter.sender.as_str(), [60, 50, 155]),
            "HRA" | "the HRA" => ("the HRA", [60, 150, 65]),
            "Post Office" | "the post office" => ("the post office", [60, 150, 65]),
            "Jingle" | "Snowman" | "Chip" => (letter.sender.as_str(), [165, 30, 255]),
            _ => (letter.sender.as_str(), [100, 65, 195]),
        };
        Self {
            lines: vec![
                "Letter to".into(),
                recipient,
                format!("from {}", colored(sender, color)),
            ],
            special: placement(Some(row), None),
        }
    }

    fn delivery(to: &str, from: &str, row: usize) -> Self {
        Self {
            lines: vec![
                "Delivery for".into(),
                colored(to, [205, 40, 40]),
                format!("from {}", colored(from, [100, 65, 195])),
            ],
            special: placement(None, Some(row)),
        }
    }
}

/// Only actual delivery records supply provenance. Other quest-conditioned
/// objects (including the intro Bell bag) retain their ordinary item names.
pub(in crate::pockets_disc) fn delivery_label(
    disc: &GameDisc,
    identity: &crate::save::PlayerIdentity,
    inventory: &PlayerInventory,
    quest: &crate::quest::QuestState,
    slot: usize,
) -> Option<Label> {
    let item = *inventory.pockets.pockets.get(slot)?;
    let condition = inventory.pockets.item_cond(slot);
    if condition & 2 != 0 {
        if let Some(job) = identity.first_job.as_ref().filter(|job| {
            // Match by item and quest condition, as first_job_talk does: the port's
            // pocket moves do not update the record's pockets_idx.
            identity.events.first_job && job.progress >= 2 && job.item == item
        }) {
            let recipient = usize::from(job.recipient?);
            let raw = disc.npc_names.get(recipient * 8..recipient * 8 + 8)?;
            let to = crate::messages::plain(raw);
            if !to.is_empty() {
                return Some(Label::delivery(&to, "Tom Nook", slot / 5));
            }
        }
    }
    // Legacy port deliveries used normal-condition PARCEL; display their
    // existing active provenance without rewriting pocket conditions.
    if item == crate::items::PARCEL && condition & 1 == 0 {
        if let Some(crate::quest::ActiveQuest {
            kind: crate::quest::QuestKind::Delivery { giver, recipient },
            ..
        }) = &quest.active
        {
            return Some(Label::delivery(recipient, giver, slot / 5));
        }
    }
    None
}
