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
use rustimal_logic::{menu_slide::Slide, notice_board::Roll};

type Paper = (Vec<(Handle<Mesh>, Handle<N64Material>)>, [u8; 4]);

#[derive(Resource, Default)]
struct MailAssets {
    papers: HashMap<u8, Paper>,
    root: Option<Entity>,
    skin: crate::shop_nook::editor::Skin,
}

/// Transient submenu presentation; none of this belongs in a saved letter.
#[derive(Resource, Default)]
pub(crate) struct MailPresentation {
    active: bool,
    closing: bool,
    slide: Slide,
    keyboard: Option<Slide>,
    roll: Roll,
    roll_ticks: f32,
    cursor_line: usize,
}

impl MailPresentation {
    pub(crate) fn busy(&self) -> bool {
        self.active
            && (self.closing
                || !self.slide.settled()
                || self.keyboard.is_some_and(|slide| !slide.settled()))
    }

    pub(crate) fn closing(&self) -> bool {
        self.active && self.closing
    }
}

#[derive(Component)]
enum LetterPart {
    Paper,
    Keyboard,
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
    mut assets: ResMut<MailAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut n64: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
    mut motion: ResMut<MailPresentation>,
    mut reset: MessageReader<crate::sound::ResetAudio>,
) {
    if reset.read().count() > 0 {
        if let Some(root) = assets.root.take() {
            commands.entity(root).despawn();
        }
        *motion = MailPresentation::default();
        return;
    }
    let visible =
        ui.open && (ui.reading.is_some() || ui.writing) && *focus == crate::input::InputFocus::Mail;
    if !visible {
        if assets.root.is_some() && !motion.closing {
            motion.closing = true;
            if let Some(keyboard) = motion.keyboard.as_mut() {
                keyboard.close();
            }
        }
        return;
    }
    if !motion.active {
        *motion = MailPresentation {
            active: true,
            keyboard: ui.writing.then(Slide::new),
            ..default()
        };
    }
    if motion.closing {
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
    let index = if ui.writing {
        ui.write_paper
            .map(|(_, paper)| paper)
            .or_else(|| {
                ui.rewriting
                    .and_then(|i| mailbox.letters.get(i))
                    .map(|letter| letter.paper)
            })
            .unwrap_or(0)
    } else {
        letter.map_or(0, |l| l.paper)
    };
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
    let paper_root = commands
        .spawn((
            LetterPart::Paper,
            Transform::default(),
            Visibility::Inherited,
            ChildOf(root),
        ))
        .id();
    let model = assets.papers[&index].0.clone();
    let color_value = assets.papers[&index].1;
    let color = &color_value;
    let writing = ui.writing;
    // The editor is drawn over the letter in the native submenu order.
    // Keep all six body lines, allowing its panel to cover the lower ones.
    let ink_z = if writing { 209.0 } else { 230.0 };
    for (i, (mesh, material)) in model.iter().enumerate() {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(0.0, 0.0, 200.0 + i as f32 * 0.1),
            RenderLayers::layer(1),
            ChildOf(paper_root),
        ));
    }
    if writing {
        if motion.keyboard.is_none() {
            motion.keyboard = Some(Slide::new());
        }
        let keyboard_root = commands
            .spawn((
                LetterPart::Keyboard,
                Transform::default(),
                Visibility::Inherited,
                ChildOf(root),
            ))
            .id();
        let mut stores = crate::shop_nook::editor::Stores {
            meshes: &mut meshes,
            n64: &mut n64,
            images: &mut images,
            materials: &mut materials,
        };
        crate::shop_nook::editor::spawn(
            &mut commands,
            keyboard_root,
            &mut assets.skin,
            &disc,
            &font,
            &ui.write_keyboard,
            ui.write_held,
            &mut stores,
        );
    }
    let mut paint = Paint {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        root: paper_root,
    };
    let recipient = if ui.writing {
        ui.write_recipients
            .get(ui.write_recipient)
            .map_or("?", |r| r.name())
    } else {
        &identity.player
    };
    // A disc letter's own header (mMl: header with the name in place).
    let header = letter
        .and_then(|l| l.header.clone())
        .unwrap_or_else(|| format!("To {recipient}"));
    paint.text(&font, &header, Vec2::new(64.0, 36.0), *color, ink_z);
    let body = letter.map_or(ui.write_text.as_str(), |l| l.body.as_str());
    let wrapped = lines(body, &font.widths);
    let count = wrapped.len().div_ceil(6);
    if writing {
        // Native mBD_roll_control keeps the cursor within three body lines.
        let codes = crate::mail::letter_codes(body);
        let cursor = ui.write_cursor.unwrap_or(codes.len()).min(codes.len());
        let (line, column) = rustimal_logic::diary::cursor_line(&codes, cursor, &font.widths);
        motion.cursor_line = line;
        for (row, text) in wrapped.iter().take(6).enumerate() {
            let y = 64.0 + row as f32 * 16.0;
            paint.text(&font, text, Vec2::new(64.0, y), *color, ink_z);
            if row == line {
                let at = Vec2::new(64.0 + column as f32, y);
                if let Some(caret) =
                    paint.text(&font, "_", at, crate::shop_nook::editor::CARET_INK, ink_z)
                {
                    paint
                        .commands
                        .entity(caret)
                        .insert(crate::shop_nook::editor::Caret(at));
                }
            }
        }
    }
    let page = ui.page.min(count - 1);
    for (row, line) in wrapped
        .iter()
        .skip(page * 6)
        .take(6)
        .enumerate()
        .filter(|_| !writing)
    {
        paint.text(
            &font,
            line,
            Vec2::new(64.0, 64.0 + row as f32 * 16.0),
            *color,
            230.0,
        );
    }
    let sender = letter.map_or(identity.player.as_str(), |l| l.sender.as_str());
    let from = letter
        .and_then(|l| l.footer.clone())
        .unwrap_or_else(|| format!("From {sender}"));
    paint.text(
        &font,
        &from,
        Vec2::new(256.0 - text_width(&from, &font.widths), 172.0),
        *color,
        ink_z,
    );
    if let Some(letter) = letter.filter(|l| l.header.is_none() && !l.subject.is_empty()) {
        let subject = format!(
            "{}{}{}",
            crate::messages::markup::LINE_SCALE,
            crate::messages::markup::arg(20),
            letter.subject
        );
        paint.text(&font, &subject, Vec2::new(64.0, 52.0), *color, ink_z);
    }
    let hints = if writing {
        vec![
            format!("To {recipient}   PgUp/PgDn: change   Ctrl+X: accent"),
            "Enter: key  Bksp  Ctrl+arrows: cursor  F4: send  Esc: cancel".to_string(),
        ]
    } else {
        vec![format!(
            "{}/{}  Left/Right: page  Esc: back",
            page + 1,
            count
        )]
    };
    // PC help stays in screen coordinates while stationery and text roll together.
    paint.root = root;
    for (row, hint) in hints.iter().enumerate() {
        let hint = format!(
            "{}{}{}",
            crate::messages::markup::LINE_SCALE,
            crate::messages::markup::arg(20),
            hint
        );
        let at = if writing {
            Vec2::new(64.0, row as f32 * 10.0)
        } else {
            Vec2::new(64.0, 186.0)
        };
        paint.text(&font, &hint, at, *color, 230.0);
    }
}

fn animate(
    mut commands: Commands,
    time: Res<Time>,
    mut motion: ResMut<MailPresentation>,
    mut assets: ResMut<MailAssets>,
    mut parts: Query<(&LetterPart, &mut Transform, &mut Visibility)>,
) {
    if !motion.active {
        return;
    }
    let ticks = time.delta_secs() * 60.0;
    motion.roll_ticks += ticks;
    let roll_ticks = motion.roll_ticks as u32;
    motion.roll_ticks -= roll_ticks as f32;
    if motion.closing {
        if let Some(keyboard) = motion.keyboard.as_mut() {
            if keyboard.advance(ticks) {
                motion.keyboard = None;
            }
        }
        // Like mBD_move_Obey, settle the rolled paper as the editor leaves.
        for _ in 0..roll_ticks {
            motion.roll.release();
        }
        if motion.keyboard.is_none() && motion.roll.position <= 0.0 {
            if !motion.slide.closing {
                motion.slide.close();
            }
            if motion.slide.advance(ticks) {
                if let Some(root) = assets.root.take() {
                    commands.entity(root).despawn();
                }
                *motion = MailPresentation::default();
                return;
            }
        }
    } else {
        motion.slide.advance(ticks);
        if let Some(keyboard) = motion.keyboard.as_mut() {
            keyboard.advance(ticks);
        }
        if motion.slide.settled() && motion.keyboard.is_some_and(|keyboard| keyboard.settled()) {
            let line = motion.cursor_line;
            for _ in 0..roll_ticks {
                motion.roll.follow(line);
            }
        }
    }
    for (part, mut transform, mut visibility) in &mut parts {
        match part {
            LetterPart::Paper => {
                transform.translation.y = (motion.slide.pos + motion.roll.position) * 16.0;
            }
            LetterPart::Keyboard => {
                *visibility = if motion.keyboard.is_some() {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                transform.translation.y = -motion.keyboard.map_or(300.0, |slide| slide.pos) * 16.0;
            }
        }
    }
}

pub struct MailDiscPlugin;
impl Plugin for MailDiscPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MailAssets>()
            .init_resource::<MailPresentation>()
            .add_systems(
                Update,
                (draw, animate)
                    .chain()
                    .after(crate::input::InteractSet::Ui)
                    .run_if(resource_exists::<MsgAssets>),
            );
    }
}
