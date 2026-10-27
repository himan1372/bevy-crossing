//! m_music_ovl's original 11x5 collection, runtime display lists and font.
use super::*;
use crate::msg_window::{screen, text_mesh, text_width, MsgAssets};
use crate::n64_material::N64Material;
use bevy::camera::visibility::RenderLayers;

pub(super) struct Grid {
    owner: usize,
    selected: usize,
    menu: Option<usize>,
    take: bool,
    marks: [u32; 2],
    slide: f32,
    speed: f32,
    alternate: bool,
    ticks: f32,
    closing: bool,
    exit_sign: f32,
    cloth: f32,
    warning: bool,
}
impl Grid {
    pub(super) fn new(owner: usize, selected: usize, take: bool) -> Self {
        Self {
            owner,
            selected,
            menu: None,
            take,
            marks: [0; 2],
            slide: -300.0,
            speed: 75.0,
            alternate: false,
            ticks: 0.0,
            closing: false,
            exit_sign: -1.0,
            cloth: 0.0,
            warning: false,
        }
    }

    fn marked(&self) -> bool {
        self.marks[self.selected / 32] & (1 << (self.selected % 32)) != 0
    }

    fn choices(&self) -> &[&'static str] {
        if self.marked() {
            &["Take out marked", "Quit"]
        } else {
            &["Listen", "Take out", "Quit"]
        }
    }

    /// mSM_move_menu / move_Move: alternate 60 Hz ticks change speed;
    /// incoming slows after -120; decide exits right, cancel exits left.
    fn advance(&mut self, seconds: f32) -> bool {
        self.cloth = (self.cloth + seconds * 60.0 * 0.3535).rem_euclid(1024.0);
        self.ticks += seconds * 60.0;
        while self.ticks >= 1.0 {
            self.ticks -= 1.0;
            if self.closing || self.slide < 0.0 {
                let (threshold, target, direction, multiplier) = if self.closing {
                    (0.0, 300.0 * self.exit_sign, self.exit_sign, 2.0)
                } else {
                    (-120.0, 0.0, 1.0, 0.5)
                };
                if !self.alternate && direction * (self.slide - threshold) >= 0.0 {
                    self.speed = (self.speed * multiplier).clamp(1.0, 75.0);
                }
                self.alternate = !self.alternate;
                self.slide += self.speed * direction * 0.5;
                if direction * (self.slide - target) > 0.0 {
                    self.slide = target;
                    self.alternate = false;
                    self.ticks = 0.0;
                    return self.closing;
                }
            }
        }
        false
    }

    fn close(&mut self, decided: bool) {
        self.closing = true;
        self.exit_sign = if decided { 1.0 } else { -1.0 };
        self.alternate = false;
        self.ticks = 0.0;
        self.menu = None;
    }
}

type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;
#[derive(Resource)]
struct GridAssets {
    frame: Model,
    icon: Model,
    mark: Model,
    patterns: Vec<(
        Handle<N64Material>,
        Handle<crate::pockets_disc::PocketPattern>,
    )>,
}
#[derive(Default)]
struct Drawn {
    root: Option<Entity>,
    key: Option<(usize, Option<usize>, [u32; 2], [u32; 2], bool)>,
    meshes: Vec<Handle<Mesh>>,
    materials: Vec<Handle<StandardMaterial>>,
}
#[derive(Component)]
struct MusicPanel;

fn setup(
    mut commands: Commands,
    disc: Res<GameDisc>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
    mut patterns: ResMut<Assets<crate::pockets_disc::PocketPattern>>,
) {
    let mut build = |names: &[&str]| {
        let lists: Vec<_> = names
            .iter()
            .filter_map(|name| disc.catalog.get(name).map(|s| s.offset))
            .collect();
        let frame = names.contains(&"mus_win_model");
        let mut sink = if frame {
            crate::disc_gfx::run_inventory_frame(&disc.rom, &lists)
        } else {
            crate::disc_gfx::run_display_lists(&disc.rom, &lists)
        };
        // mus_mark_before uses legacy load-block/tile-size commands rather
        // than the Dolphin-sized tile form. Its source IA8 mark is 16x16.
        if names.contains(&"mus_mark_model") {
            if let Some(texture) = disc.catalog.get("mus_win_mark_tex") {
                for part in &mut sink.parts {
                    part.key.tex = Some(crate::disc_gfx::TexKey {
                        img_addr: texture.offset,
                        format: rustimal_logic::graphics::texture::GxTexFormat::IA4,
                        width: 16,
                        height: 16,
                        wrap_s: 2,
                        wrap_t: 2,
                        tlut_addr: 0,
                        n64: None,
                        tlut_rgba5551: false,
                    });
                }
            }
        }
        let model = crate::disc_town::sink_meshes_scaled(
            &disc,
            &sink,
            None,
            true,
            &crate::disc_gfx::Segments::default(),
            16.0,
            (&mut meshes, &mut materials, &mut images),
        );
        if names.contains(&"mus_mark_model") {
            if let Some(texture) = disc.catalog.get("mus_win_mark_tex") {
                if let (Some(bytes), Some(key)) = (
                    disc.rom.slice(texture.source, texture.offset, 256),
                    sink.parts.first().and_then(|p| p.key.tex),
                ) {
                    // Unlike Dolphin-converted UI textures this legacy IA8
                    // load-block stores row-major intensity/alpha nibbles.
                    let rgba = bytes
                        .iter()
                        .flat_map(|v| {
                            let i = (v >> 4) * 17;
                            [i, i, i, (v & 15) * 17]
                        })
                        .collect();
                    let image = images.add(crate::n64_material::texture_image(rgba, &key));
                    for (_, handle) in &model {
                        if let Some(mut material) = materials.get_mut(handle) {
                            if let Some(old) = material.texture.replace(image.clone()) {
                                images.remove(old.id());
                            }
                        }
                    }
                }
            }
        }
        (sink, model)
    };
    let (sink, mut frame) = build(&["mus_win_model_before", "mus_win_model"]);
    let (_, overlays) = build(&[
        "mus_win_model_before",
        "mus_win_ueT_model",
        "mus_win_ue2T_model",
        "mus_tag_nes_model",
    ]);
    frame.extend(overlays);
    let (_, icon) = build(&["mus_icon_model_before", "mus_icon_model"]);
    let (_, mark) = build(&["mus_mark_before", "mus_mark_model"]);
    let mut masks = Vec::new();
    if let (Some(tex), Some(pal)) = (
        disc.catalog.get("mus_win_nuno_tex_rgb_ci4"),
        disc.catalog.get("mus_win_nuno_tex_rgb_ci4_pal"),
    ) {
        let key = crate::disc_gfx::TexKey {
            img_addr: tex.offset,
            format: rustimal_logic::graphics::texture::GxTexFormat::C4,
            width: 32,
            height: 32,
            wrap_s: 1,
            wrap_t: 1,
            tlut_addr: pal.offset,
            n64: None,
            tlut_rgba5551: false,
        };
        if let Some(rgba) = crate::disc_gfx::decode_texture(
            &disc.rom,
            &key,
            Some(pal.offset),
            &crate::disc_gfx::Segments::default(),
        ) {
            let cloth = images.add(crate::n64_material::texture_image(rgba, &key));
            for (part, (_, material)) in sink.parts.iter().zip(&frame) {
                if !part.key.two_cycle {
                    continue;
                }
                let Some(key) = &part.key.tex else { continue };
                if let Some(mut rgba) = crate::disc_gfx::decode_texture(
                    &disc.rom,
                    key,
                    None,
                    &crate::disc_gfx::Segments::default(),
                ) {
                    if key.format == rustimal_logic::graphics::texture::GxTexFormat::I4 {
                        for pixel in rgba.as_chunks_mut::<4>().0 {
                            pixel[3] = pixel[0];
                        }
                    }
                    let mask = images.add(crate::n64_material::texture_image(rgba, key));
                    masks.push((
                        material.clone(),
                        patterns.add(crate::pockets_disc::PocketPattern::new(cloth.clone(), mask)),
                    ));
                }
            }
        }
    }
    commands.insert_resource(GridAssets {
        frame,
        icon,
        mark,
        patterns: masks,
    });
}

fn take(
    room: &mut PlayerRoom,
    inventory: &mut PlayerInventory,
    item: u16,
) -> Result<(), &'static str> {
    if !inventory.add(item) {
        return Err("Your pockets are full.");
    }
    let i = (item - 0x2A00) as usize;
    room.music_box[i / 32] &= !(1 << (i % 32));
    for slot in 0..room.runtime.actors.len() {
        if music::is_player(room, slot) {
            if let Some(actor) = room.runtime.actors[slot].as_mut() {
                if actor.items[0] == item {
                    actor.items[0] = 0;
                    actor.switch_bit = 0;
                    actor.switch_changed = true;
                    if let Some(bit) = RoomLayer::switch_bit_index(actor.cell_x, actor.cell_z) {
                        room.layers[actor.layer].ftr_switch &= !(1u64 << bit);
                    }
                }
            }
        }
    }
    Ok(())
}

fn input(
    mut room: ResMut<PlayerRoom>,
    mut actions: ResMut<Actions>,
    mut focus: ResMut<InputFocus>,
    mut inventory: ResMut<PlayerInventory>,
    mut warning_panel: ResMut<crate::warning_panel::WarningPanel>,
    current: Res<CurrentRoom>,
    state: Res<State<HouseState>>,
    time: Res<Time>,
) {
    let Some(mut grid) = room.music_grid.take() else {
        return;
    };
    if !crate::house::is_player_room(&current, &state) || !music::is_player(&room, grid.owner) {
        return;
    }
    if *focus == InputFocus::Paused {
        room.music_grid = Some(grid);
        return;
    }
    *focus = InputFocus::Furniture;
    if grid.advance(time.delta_secs()) {
        *focus = InputFocus::World;
        return;
    }
    if grid.warning {
        if !warning_panel.is_open() {
            grid.warning = false;
            actions.consume(Action::Confirm);
            actions.consume(Action::Interact);
            actions.consume(Action::Advance);
            actions.consume(Action::Cancel);
        } else {
            *focus = InputFocus::Dialogue;
        }
        room.music_grid = Some(grid);
        return;
    }
    if grid.closing || grid.slide < 0.0 {
        room.music_grid = Some(grid);
        return;
    }
    let cancel = actions.consume(Action::Cancel) || actions.consume(Action::Back);
    let item = 0x2A00 + grid.selected as u16;
    let many = grid.marked();
    let choice_count = grid.choices().len();
    if let Some(menu) = &mut grid.menu {
        let before = *menu;
        if actions.consume(Action::MoveUp) || actions.consume(Action::MenuUp) {
            *menu = (*menu + choice_count - 1) % choice_count;
        }
        if actions.consume(Action::MoveDown) || actions.consume(Action::MenuDown) {
            *menu = (*menu + 1) % choice_count;
        }
        crate::sound::cursor_changed(before, *menu);
        if cancel {
            grid.menu = None;
        } else if actions.consume(Action::Confirm) || actions.consume(Action::Interact) {
            if *menu == 0 && !many {
                if let Some(actor) = room.runtime.actors[grid.owner].as_mut() {
                    actor.items[0] = item;
                }
                music::switch(&mut room, grid.owner, true);
                grid.close(true);
                crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
            } else if (*menu == 1 && !many) || (*menu == 0 && many) {
                let result = if many {
                    let mut result = Err("Your pockets are full.");
                    for i in 0..55 {
                        if grid.marks[i / 32] & (1 << (i % 32)) != 0
                            && music::registered(&room, 0x2A00 + i as u16)
                        {
                            if take(&mut room, &mut inventory, 0x2A00 + i as u16).is_err() {
                                break;
                            }
                            result = Ok(());
                        }
                    }
                    result
                } else {
                    take(&mut room, &mut inventory, item)
                };
                if result.is_err() {
                    // `mTG_music_proc`: taking music out with no free pocket.
                    warning_panel.open(&rustimal_logic::warning::MUSIC);
                    grid.warning = true;
                    grid.menu = None;
                    *focus = InputFocus::Dialogue;
                    room.music_grid = Some(grid);
                    return;
                }
                grid.close(true);
                crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
            } else {
                grid.menu = None;
            }
        }
    } else {
        if cancel {
            grid.close(false);
            crate::sound::se(crate::sound::se_id::MENU_EXIT);
            room.music_grid = Some(grid);
            return;
        }
        let before = grid.selected;
        let mut x = (grid.selected % 11) as i32;
        let mut y = (grid.selected / 11) as i32;
        if actions.consume(Action::MoveLeft) {
            x = (x + 10) % 11;
        }
        if actions.consume(Action::MoveRight) {
            x = (x + 1) % 11;
        }
        if actions.consume(Action::MoveUp) || actions.consume(Action::MenuUp) {
            y = (y + 4) % 5;
        }
        if actions.consume(Action::MoveDown) || actions.consume(Action::MenuDown) {
            y = (y + 1) % 5;
        }
        grid.selected = (y * 11 + x) as usize;
        crate::sound::cursor_changed(before, grid.selected);
        if actions.consume(Action::Mark) && music::registered(&room, 0x2A00 + grid.selected as u16)
        {
            grid.marks[grid.selected / 32] ^= 1 << (grid.selected % 32);
            crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
        }
        if (actions.consume(Action::Confirm) || actions.consume(Action::Interact))
            && music::registered(&room, 0x2A00 + grid.selected as u16)
        {
            grid.menu = Some(usize::from(grid.take && !grid.marked()));
            crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
        }
    }
    room.music_grid = Some(grid);
}

#[allow(clippy::too_many_arguments)] // Shared disc UI mesh/material stores, no duplicated asset pipeline.
fn draw(
    mut commands: Commands,
    room: Res<PlayerRoom>,
    assets: Option<Res<GridAssets>>,
    font: Option<Res<MsgAssets>>,
    pockets: Option<Res<crate::pockets_disc::PocketAssets>>,
    mut drawn: Local<Drawn>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if assets.is_none() && room.music_grid.is_some() {
        return;
    }
    let key = room
        .music_grid
        .as_ref()
        .map(|g| (g.selected, g.menu, room.music_box, g.marks, g.closing));
    if drawn.key == key {
        return;
    }
    drawn.key = key;
    if let Some(root) = drawn.root.take() {
        commands.entity(root).despawn();
    }
    for handle in drawn.meshes.drain(..) {
        meshes.remove(handle.id());
    }
    for handle in drawn.materials.drain(..) {
        materials.remove(handle.id());
    }
    let (Some(grid), Some(assets), Some(font)) = (&room.music_grid, assets, font) else {
        return;
    };
    let root = commands
        .spawn((
            Name::new("original music collection"),
            MusicPanel,
            Transform::from_xyz(grid.slide * 16.0, 0.0, 0.0),
            Visibility::default(),
            RenderLayers::layer(1),
        ))
        .id();
    drawn.root = Some(root);
    let mut model = |parts: &[(Handle<Mesh>, Handle<N64Material>)], at: Vec3| {
        for (i, (mesh, material)) in parts.iter().enumerate() {
            let mut entity = commands.spawn((
                Mesh3d(mesh.clone()),
                Transform::from_translation(at + Vec3::Z * (i as f32 * 0.02)),
                RenderLayers::layer(1),
                ChildOf(root),
            ));
            if let Some((_, pattern)) = assets
                .patterns
                .iter()
                .find(|(source, _)| source == material)
            {
                entity.insert(MeshMaterial3d(pattern.clone()));
            } else {
                entity.insert(MeshMaterial3d(material.clone()));
            }
        }
    };
    model(&assets.frame, Vec3::ZERO);
    for i in 0..55 {
        if music::registered(&room, 0x2A00 + i) {
            model(
                &assets.icon,
                Vec3::new(
                    (-100.0 + (i % 11) as f32 * 20.0) * 16.0,
                    (28.0 - (i / 11) as f32 * 20.0) * 16.0,
                    2.0,
                ),
            );
        }
        if grid.marks[i as usize / 32] & (1 << (i % 32)) != 0 {
            model(
                &assets.mark,
                Vec3::new(
                    (-100.0 + (i % 11) as f32 * 20.0) * 16.0,
                    (28.0 - (i / 11) as f32 * 20.0) * 16.0,
                    3.0,
                ),
            );
        }
    }
    let at = Vec2::new(
        60.0 + (grid.selected % 11) as f32 * 20.0,
        95.0 + (grid.selected / 11) as f32 * 20.0,
    );
    let hand = pockets.map(|p| p.hand_image());
    let title = if music::registered(&room, 0x2A00 + grid.selected as u16) {
        crate::items::name(0x2A00 + grid.selected as u16)
    } else {
        "?????".into()
    };
    let choices = grid.choices();
    let (centre, scale, origin) = crate::msg_window::choice_layout(choices, &font.widths);
    if grid.menu.is_some() {
        for (mesh, material) in &font.choice_body {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(screen(centre).with_z(4.0))
                    .with_scale(scale.extend(1.0)),
                RenderLayers::layer(1),
                ChildOf(root),
            ));
        }
    }
    let font_material = materials.add(StandardMaterial {
        base_color_texture: Some(font.font.clone()),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });
    drawn.materials.push(font_material.clone());
    if let Some(hand) = hand {
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(hand),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        });
        drawn.materials.push(material.clone());
        let mesh = meshes.add(crate::msg_window::quad_mesh(
            at - Vec2::new(12.0, 21.0),
            Vec2::splat(32.0),
        ));
        drawn.meshes.push(mesh.clone());
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, 0.0, 6.0),
            RenderLayers::layer(1),
            ChildOf(root),
        ));
    }
    let mut text = |s: &str, at: Vec2, color: [u8; 4]| {
        let Some(mesh) = text_mesh(s, at, &font.widths, color, usize::MAX) else {
            return;
        };
        let mesh = meshes.add(mesh);
        drawn.meshes.push(mesh.clone());
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(font_material.clone()),
            Transform::from_xyz(0.0, 0.0, 8.0),
            RenderLayers::layer(1),
            ChildOf(root),
        ));
    };
    text(
        &title,
        Vec2::new((320.0 - text_width(&title, &font.widths)) * 0.5, 55.0),
        [255; 4],
    );
    text("M: mark / unmark", Vec2::new(60.0, 207.0), [255; 4]);
    if let Some(menu) = grid.menu {
        for (i, label) in choices.iter().enumerate() {
            text(
                label,
                origin + centre - Vec2::new(160.0, 120.0) + Vec2::new(0.0, i as f32 * 16.0),
                if i == menu {
                    [120, 50, 50, 255]
                } else {
                    [180, 150, 110, 255]
                },
            );
        }
        let cursor = materials.add(StandardMaterial {
            base_color_texture: Some(font.cursor.clone()),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        });
        drawn.materials.push(cursor.clone());
        let mesh = meshes.add(crate::msg_window::quad_mesh(
            origin + centre - Vec2::new(176.0, 120.0) + Vec2::new(0.0, menu as f32 * 16.0),
            Vec2::splat(16.0),
        ));
        drawn.meshes.push(mesh.clone());
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(cursor),
            Transform::from_xyz(0.0, 0.0, 9.0),
            RenderLayers::layer(1),
            ChildOf(root),
        ));
    }
}

fn animate_panel(
    room: Res<PlayerRoom>,
    assets: Option<Res<GridAssets>>,
    mut panels: Query<&mut Transform, With<MusicPanel>>,
    mut patterns: ResMut<Assets<crate::pockets_disc::PocketPattern>>,
    mut materials: ResMut<Assets<N64Material>>,
) {
    let (Some(grid), Some(assets)) = (&room.music_grid, assets) else {
        return;
    };
    for mut panel in &mut panels {
        panel.translation.x = grid.slide.trunc() * 16.0;
        // Existing dialogue draws its warning at z=0..3 on this UI camera.
        // Keep the collection behind it while preserving all marks/cursor.
        panel.translation.z = if grid.warning { -20.0 } else { 0.0 };
    }
    for (_, pattern) in &assets.patterns {
        if let Some(mut pattern) = patterns.get_mut(pattern) {
            pattern.scroll(grid.slide.trunc(), grid.cloth);
        }
    }
    // mMU_set_dl: triangular 20-frame green mark pulse, graph phase at 30 Hz.
    let frame = (grid.cloth / 0.3535 * 0.5).rem_euclid(20.0).floor();
    let frame = frame.min(20.0 - frame);
    for (_, handle) in &assets.mark {
        if let Some(mut material) = materials.get_mut(handle) {
            material.combiner.prim = Vec4::new(0.0, (95.0 + frame * 9.0) / 255.0, 0.0, 1.0);
            material.combiner.env =
                Vec4::new(frame * 6.0, 210.0 + frame * 4.5, frame * 6.0, 255.0) / 255.0;
        }
    }
}

pub(super) struct MusicGridPlugin;
impl Plugin for MusicGridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            setup
                .run_if(resource_exists::<GameDisc>)
                .run_if(not(resource_exists::<GridAssets>)),
        )
        .add_systems(
            Update,
            input
                .after(super::movement::advance)
                .before(InteractSet::Doors)
                .after(crate::save::LoadSet)
                .in_set(GameplaySet),
        )
        .add_systems(
            PostUpdate,
            (draw, animate_panel.after(draw)).run_if(in_state(crate::states::AppState::InGame)),
        );
    }
}
