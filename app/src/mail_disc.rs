//! Runtime stationery and text: m_board_ovl's paper tables and 192px/6-line body.
use std::collections::HashMap;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use disc::{RomSource, Target};

use crate::game_disc::GameDisc;
use crate::mail::{MailUi, Mailbox};
use crate::msg_window::{text_width, MsgAssets};
use crate::n64_material::N64Material;
use crate::pockets_disc::Paint;

type Paper = (Vec<(Handle<Mesh>, Handle<N64Material>)>, [u8; 4]);

#[derive(Resource, Default)]
struct MailAssets {
    papers: HashMap<u8, Paper>,
    root: Option<Entity>,
}

fn pointer(disc: &GameDisc, name: &str, index: u8) -> Option<u32> {
    let table = disc.catalog.get(name)?;
    match disc
        .rom
        .pointer_at(table.source, table.offset + u32::from(index) * 4)?
    {
        Target::File(RomSource::Rel, offset) => Some(offset),
        _ => None,
    }
}

fn paper(
    disc: &GameDisc,
    index: u8,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<N64Material>,
    images: &mut Assets<Image>,
) -> Paper {
    // The original 256-entry table has 64 populated papers; reserved entries
    // have neither a display list nor a text color. Keep old/custom mail readable.
    let index = if pointer(disc, "paper_disp_model", index).is_some() {
        index
    } else {
        0
    };
    let mut lists = Vec::new();
    if let (Some(mode), Some(model)) = (
        disc.catalog.get("lat_letter_mode"),
        pointer(disc, "paper_disp_model", index),
    ) {
        lists.extend([mode.offset, model]);
        if let (Some(mode), Some(model)) = (
            disc.catalog.get("lat_letter_sen_mode"),
            pointer(disc, "paper_disp_sen_model", index),
        ) {
            lists.extend([mode.offset, model]);
        }
    }
    let sink = crate::disc_gfx::run_display_lists(&disc.rom, &lists);
    let model = crate::disc_town::sink_meshes_scaled(
        disc,
        &sink,
        None,
        true,
        &crate::disc_gfx::Segments::default(),
        16.0,
        (meshes, materials, images),
    );
    // I4 stationery ruling uses intensity as coverage, as does the disc font.
    for (part, (_, handle)) in sink.parts.iter().zip(&model) {
        let Some(key) = part.key.tex.as_ref() else {
            continue;
        };
        if key.format != rustimal_logic::graphics::texture::GxTexFormat::I4 {
            continue;
        }
        if let Some(mut rgba) = crate::disc_gfx::decode_texture(
            &disc.rom,
            key,
            None,
            &crate::disc_gfx::Segments::default(),
        ) {
            for p in rgba.as_chunks_mut::<4>().0 {
                p[3] = p[0];
            }
            if let Some(mut material) = materials.get_mut(handle) {
                material.texture = Some(images.add(crate::n64_material::texture_image(rgba, key)));
            }
        }
    }
    let color = disc
        .catalog
        .get("letter_color")
        .and_then(|table| {
            disc.rom
                .slice(table.source, table.offset + u32::from(index) * 4, 4)
        })
        .and_then(|b| b.try_into().ok())
        .unwrap_or([70, 40, 50, 255]);
    (model, color)
}

/// Character-width wrapping retains every character of the port's long letters.
fn lines(text: &str, widths: &[u8; 256]) -> Vec<String> {
    let mut result = Vec::new();
    let mut line = String::new();
    let mut width = 0.0;
    for ch in text.chars() {
        if ch == '\r' {
            continue;
        }
        let next = f32::from(widths[crate::messages::ac_code(ch) as usize]);
        if ch == '\n' || (width + next > 192.0 && !line.is_empty()) {
            result.push(std::mem::take(&mut line));
            width = 0.0;
        }
        if ch != '\n' {
            line.push(ch);
            width += next;
        }
    }
    result.push(line);
    result
}

pub(crate) fn page_count(text: &str, widths: &[u8; 256]) -> usize {
    lines(text, widths).len().div_ceil(6)
}

fn draw(
    mut commands: Commands,
    disc: Res<GameDisc>,
    font: Res<MsgAssets>,
    ui: Res<MailUi>,
    mailbox: Res<Mailbox>,
    identity: Res<crate::save::PlayerIdentity>,
    focus: Res<crate::input::InputFocus>,
    villagers: Query<&crate::npc::Villager>,
    mut assets: ResMut<MailAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut n64: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let visible =
        ui.open && (ui.reading.is_some() || ui.writing) && *focus == crate::input::InputFocus::Mail;
    if !visible {
        if let Some(root) = assets.root.take() {
            commands.entity(root).despawn();
        }
        return;
    }
    if assets.root.is_some() && !ui.is_changed() && !mailbox.is_changed() && !identity.is_changed()
    {
        return;
    }
    if let Some(root) = assets.root.take() {
        commands.entity(root).despawn();
    }
    let letter = ui.reading.and_then(|i| mailbox.letters.get(i));
    let index = letter.map_or(0, |l| l.paper);
    assets
        .papers
        .entry(index)
        .or_insert_with(|| paper(&disc, index, &mut meshes, &mut n64, &mut images));
    let root = commands
        .spawn((
            Name::new("retail letter"),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    assets.root = Some(root);
    let (model, color) = &assets.papers[&index];
    for (i, (mesh, material)) in model.iter().enumerate() {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(0.0, 0.0, 200.0 + i as f32 * 0.1),
            RenderLayers::layer(1),
            ChildOf(root),
        ));
    }
    let mut paint = Paint {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        root,
    };
    let mut names: Vec<_> = villagers.iter().map(|v| v.name).collect();
    names.sort_unstable();
    let recipient = if ui.writing {
        names.get(ui.write_recipient).copied().unwrap_or("?")
    } else {
        &identity.player
    };
    paint.text(
        &font,
        &format!("To {recipient}"),
        Vec2::new(64.0, 36.0),
        *color,
        230.0,
    );
    let draft = format!("{}_", ui.write_text);
    let body = letter.map_or(draft.as_str(), |l| l.body.as_str());
    let wrapped = lines(body, &font.widths);
    let count = wrapped.len().div_ceil(6);
    let page = if ui.writing {
        count - 1
    } else {
        ui.page.min(count - 1)
    };
    for (row, line) in wrapped.iter().skip(page * 6).take(6).enumerate() {
        paint.text(
            &font,
            line,
            Vec2::new(64.0, 64.0 + row as f32 * 16.0),
            *color,
            230.0,
        );
    }
    let sender = letter.map_or(identity.player.as_str(), |l| l.sender.as_str());
    let from = format!("From {sender}");
    paint.text(
        &font,
        &from,
        Vec2::new(256.0 - text_width(&from, &font.widths), 172.0),
        *color,
        230.0,
    );
    if let Some(letter) = letter {
        let subject = format!(
            "{}{}{}",
            crate::messages::markup::LINE_SCALE,
            crate::messages::markup::arg(20),
            letter.subject
        );
        paint.text(&font, &subject, Vec2::new(64.0, 52.0), *color, 230.0);
    }
    let hint = if ui.writing {
        "Up/Down: recipient  Enter: send  Esc: cancel".to_string()
    } else {
        format!("{}/{}  Left/Right: page  Esc: back", page + 1, count)
    };
    let hint = format!(
        "{}{}{}",
        crate::messages::markup::LINE_SCALE,
        crate::messages::markup::arg(20),
        hint
    );
    paint.text(&font, &hint, Vec2::new(64.0, 186.0), *color, 230.0);
}

pub struct MailDiscPlugin;
impl Plugin for MailDiscPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MailAssets>().add_systems(
            Update,
            draw.after(crate::input::InteractSet::Ui)
                .run_if(resource_exists::<MsgAssets>),
        );
    }
}
