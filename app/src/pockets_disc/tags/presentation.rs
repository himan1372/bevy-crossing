//! Retained tag geometry, with the original wait/open/return scale rates.
use super::{layout::Layout, *};

#[derive(Clone, PartialEq)]
struct Key {
    at: Vec2,
    lines: Vec<String>,
    prompt: bool,
    layout: Layout,
}

struct Shown {
    key: Key,
    entity: Entity,
    content: Entity,
    choice: Option<usize>,
}

#[derive(Default)]
pub(crate) struct Presentation {
    shown: [Option<Shown>; 3],
    returning: bool,
    last_open: bool,
}

#[derive(Component)]
pub(crate) struct Motion {
    kind: usize,
    base: Vec3,
    body: Vec3,
    progress: f32,
    wait: f32,
    closing: bool,
    pivot: bool,
}

impl Presentation {
    pub(crate) fn close(&mut self, commands: &mut Commands) {
        for shown in self.shown.iter_mut().filter_map(Option::take) {
            commands.entity(shown.entity).insert(Closing(true));
        }
        self.last_open = false;
        self.returning = false;
    }

    #[allow(clippy::too_many_arguments)] // One shared paint context and immutable popup content.
    pub(crate) fn show(
        &mut self,
        models: &TagModels,
        paint: &mut Paint,
        font: &MsgAssets,
        root: Entity,
        at: Vec2,
        label: &str,
        options: &[&str],
        choice: Option<usize>,
        discard: bool,
        pocket_row: Option<usize>,
    ) {
        if self.last_open && choice.is_none() {
            // Returning to NONE closes both tags, then restarts its name wait.
            for shown in self.shown.iter_mut().filter_map(Option::take) {
                paint.commands.entity(shown.entity).insert(Closing(false));
            }
            self.last_open = false;
            self.returning = true;
        }
        if self.returning {
            return;
        }
        self.last_open = choice.is_some();
        let name = Layout::item(at, text_width(label, &font.widths));
        let width = options
            .iter()
            .map(|s| text_width(s, &font.widths))
            .fold(0.0, f32::max);
        let action = Layout::action(name, width, options.len(), pocket_row);
        let confirm_width = ["Throw it out?", "Yes", "No"]
            .iter()
            .map(|s| text_width(s, &font.widths))
            .fold(0.0, f32::max);
        let confirmation = Layout::confirmation(action, confirm_width, 3, 2);
        for (kind, layout, lines, selected, prompt) in [
            (0, name, vec![label.to_string()], None, false),
            (
                1,
                action,
                options.iter().map(|s| (*s).to_string()).collect(),
                choice.map(|i| if discard { 2 } else { i }),
                false,
            ),
            (
                2,
                confirmation,
                vec!["Yes".to_string(), "No".to_string()],
                choice,
                true,
            ),
        ] {
            let visible = !label.is_empty()
                && match kind {
                    0 => true,
                    1 => choice.is_some(),
                    _ => discard && choice.is_some(),
                };
            if !visible {
                if let Some(old) = self.shown[kind].take() {
                    if kind == 0 {
                        // Empty slot: SOURCE arrow_dir=0 suppresses the old name immediately.
                        paint.commands.entity(old.entity).despawn();
                    } else {
                        paint.commands.entity(old.entity).insert(Closing(false));
                    }
                }
                continue;
            }
            let key = Key {
                at,
                lines,
                prompt,
                layout,
            };
            if self.shown[kind].as_ref().is_some_and(|old| old.key != key) {
                if let Some(old) = self.shown[kind].take() {
                    // A changed slot resets the source name timer, without an old label lingering.
                    paint.commands.entity(old.entity).despawn();
                }
            }
            if let Some(shown) = &mut self.shown[kind] {
                if kind == 0 && choice.is_some() {
                    paint.commands.entity(shown.entity).insert(Expedite);
                }
                if shown.choice != selected {
                    paint.commands.entity(shown.content).despawn();
                    shown.content = content(
                        models,
                        paint,
                        font,
                        shown.entity,
                        layout,
                        &key.lines,
                        selected,
                        prompt,
                    );
                    shown.choice = selected;
                }
                continue;
            }
            let base = (layout.base * 16.0).extend(190.0 + kind as f32 * 10.0);
            let entity = paint
                .commands
                .spawn((
                    Name::new(
                        [
                            "retail item popup",
                            "retail action popup",
                            "retail confirmation popup",
                        ][kind],
                    ),
                    Transform::from_translation(base).with_scale(Vec3::new(0.0001, 0.0001, 1.0)),
                    Visibility::Hidden,
                    RenderLayers::layer(1),
                    ChildOf(root),
                    Motion {
                        kind,
                        base,
                        body: (layout.body * 16.0).extend(0.0),
                        progress: 0.0,
                        wait: if kind == 0 && choice.is_none() {
                            20.0
                        } else {
                            0.0
                        },
                        closing: false,
                        pivot: false,
                    },
                ))
                .id();
            let previous_root = paint.root;
            paint.root = entity;
            models.layout(paint, layout, kind);
            paint.root = previous_root;
            let content = content(
                models, paint, font, entity, layout, &key.lines, selected, prompt,
            );
            self.shown[kind] = Some(Shown {
                key,
                entity,
                content,
                choice: selected,
            });
        }
    }
}

#[allow(clippy::too_many_arguments)] // The immutable content of one retained popup.
fn content(
    models: &TagModels,
    paint: &mut Paint,
    font: &MsgAssets,
    root: Entity,
    layout: Layout,
    lines: &[String],
    choice: Option<usize>,
    prompt: bool,
) -> Entity {
    let entity = paint
        .commands
        .spawn((Transform::default(), Visibility::default(), ChildOf(root)))
        .id();
    let previous_root = paint.root;
    paint.root = entity;
    let origin = layout.body + layout.text;
    if prompt {
        paint.text(
            font,
            &scaled_text("Throw it out?", 28),
            Vec2::new(160.0 + origin.x, 120.0 - origin.y),
            [220, 30, 220, 255],
            2.0,
        );
    }
    for (i, line) in lines.iter().enumerate() {
        let row = i + usize::from(prompt);
        let at = Vec2::new(160.0 + origin.x, 120.0 - origin.y + row as f32 * 14.0);
        let color = if choice == Some(i) {
            [120, 35, 255, 255]
        } else if choice.is_some() {
            [140, 150, 190, 255]
        } else {
            [90, 60, 50, 255]
        };
        paint.text(font, &scaled_text(line, 28), at, color, 2.0);
        if choice == Some(i) {
            // mTG_set_win_field: cursor has its own native vertices/offset.
            let cursor = origin + Vec2::new(-3.5, -7.0 - row as f32 * 14.0);
            models.cursor.draw(
                paint,
                Transform::from_translation((cursor * 16.0).extend(3.0)),
                "retail tag cursor",
            );
        }
    }
    paint.root = previous_root;
    entity
}

#[derive(Component)]
pub(crate) struct Closing(bool);
#[derive(Component)]
pub(crate) struct Expedite;

pub(crate) fn animate(
    mut commands: Commands,
    time: Res<Time>,
    assets: Option<ResMut<PocketAssets>>,
    mut popups: Query<(
        Entity,
        &mut Motion,
        &mut Transform,
        &mut Visibility,
        Option<&Closing>,
        Option<&Expedite>,
    )>,
) {
    let ticks = time.delta_secs() * 60.0;
    let name_ready = popups
        .iter()
        .filter(|(_, m, _, _, closing, _)| m.kind == 0 && !m.closing && closing.is_none())
        .all(|(_, m, _, _, _, _)| m.progress >= 1.0);
    let action_ready = popups
        .iter()
        .filter(|(_, m, _, _, closing, _)| m.kind == 1 && !m.closing && closing.is_none())
        .all(|(_, m, _, _, _, _)| m.progress >= 1.0);
    for (entity, mut motion, mut transform, mut visible, closing, expedite) in &mut popups {
        if let Some(closing) = closing {
            motion.closing = true;
            motion.pivot = closing.0;
            commands.entity(entity).remove::<Closing>();
        }
        if expedite.is_some() {
            motion.wait = 0.0;
            commands.entity(entity).remove::<Expedite>();
        }
        if motion.closing {
            motion.progress = (motion.progress - ticks * 0.125).max(0.0);
            if motion.progress == 0.0 {
                commands.entity(entity).despawn();
                continue;
            }
        } else if motion.wait > 0.0 {
            motion.wait = (motion.wait - ticks).max(0.0);
        } else if (motion.kind != 1 || name_ready) && (motion.kind != 2 || action_ready) {
            let rate = if motion.kind == 1 { 0.185 } else { 0.125 };
            motion.progress = (motion.progress + ticks * rate).min(1.0);
        }
        *visible = if motion.progress == 0.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        transform.scale = Vec3::new(
            motion.progress.max(0.0001),
            motion.progress.max(0.0001),
            1.0,
        );
        transform.translation = motion.base
            + if motion.pivot {
                motion.body * (1.0 - motion.progress)
            } else {
                Vec3::ZERO
            };
    }
    if let Some(mut assets) = assets {
        if assets.tag_state.returning
            && popups
                .iter()
                .filter(|(_, m, _, _, c, _)| m.closing || c.is_some())
                .all(|(_, m, _, _, _, _)| m.progress == 0.0)
        {
            assets.tag_state.returning = false;
            assets.tag_dirty = true;
        }
        assets.tags_busy = assets.tag_state.returning
            || popups.iter().any(|(_, m, _, _, c, _)| {
                if m.closing || c.is_some() {
                    m.progress > 0.0
                } else {
                    m.kind > 0 && m.progress < 1.0
                }
            });
    }
}
