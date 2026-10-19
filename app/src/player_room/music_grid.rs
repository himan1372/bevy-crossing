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
}
impl Grid {
    pub(super) fn new(owner: usize, selected: usize, take: bool) -> Self {
        Self {
            owner,
            selected,
            menu: None,
            take,
        }
    }
}

type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;
#[derive(Resource)]
struct GridAssets {
    frame: Model,
    icon: Model,
    patterns: Vec<(
        Handle<N64Material>,
        Handle<crate::pockets_disc::PocketPattern>,
    )>,
}
#[derive(Default)]
struct Drawn {
    root: Option<Entity>,
    key: Option<(usize, Option<usize>, [u32; 2])>,
    meshes: Vec<Handle<Mesh>>,
    materials: Vec<Handle<StandardMaterial>>,
}

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
        let sink = if frame {
            crate::disc_gfx::run_inventory_frame(&disc.rom, &lists)
        } else {
            crate::disc_gfx::run_display_lists(&disc.rom, &lists)
        };
        let model = crate::disc_town::sink_meshes_scaled(
            &disc,
            &sink,
            None,
            true,
            &crate::disc_gfx::Segments::default(),
            16.0,
            (&mut meshes, &mut materials, &mut images),
        );
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
    mut dialogue: ResMut<DialogueState>,
    current: Res<CurrentRoom>,
    state: Res<State<HouseState>>,
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
    let cancel = actions.consume(Action::Cancel) || actions.consume(Action::Back);
    let item = 0x2A00 + grid.selected as u16;
    if let Some(menu) = &mut grid.menu {
        let before = *menu;
        if actions.consume(Action::MoveUp) || actions.consume(Action::MenuUp) {
            *menu = (*menu + 2) % 3;
        }
        if actions.consume(Action::MoveDown) || actions.consume(Action::MenuDown) {
            *menu = (*menu + 1) % 3;
        }
        crate::sound::cursor_changed(before, *menu);
        if cancel {
            grid.menu = None;
        } else if actions.consume(Action::Confirm) || actions.consume(Action::Interact) {
            if *menu == 0 {
                if let Some(actor) = room.runtime.actors[grid.owner].as_mut() {
                    actor.items[0] = item;
                }
                music::switch(&mut room, grid.owner, true);
                *focus = InputFocus::World;
                crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
                return;
            } else if *menu == 1 {
                if let Err(message) = take(&mut room, &mut inventory, item) {
                    notice(&mut dialogue, &mut room, &mut focus, message);
                } else {
                    *focus = InputFocus::World;
                }
                crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
                return;
            } else {
                grid.menu = None;
            }
        }
    } else {
        if cancel {
            *focus = InputFocus::World;
            crate::sound::se(crate::sound::se_id::MENU_EXIT);
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
        if (actions.consume(Action::Confirm) || actions.consume(Action::Interact))
            && music::registered(&room, 0x2A00 + grid.selected as u16)
        {
            grid.menu = Some(usize::from(grid.take));
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
        .map(|g| (g.selected, g.menu, room.music_box));
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
            Transform::default(),
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
    let choices = ["Listen", "Take out", "Quit"];
    let (centre, scale, origin) = crate::msg_window::choice_layout(&choices, &font.widths);
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
        let mesh = meshes.add(text_mesh(s, at, &font.widths, color, usize::MAX));
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
            draw.run_if(in_state(crate::states::AppState::InGame)),
        );
    }
}
