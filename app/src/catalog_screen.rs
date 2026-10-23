//! Nook's catalog (`m_catalog_ovl`). Nine tabs, seven names a page, only
//! items the player has held. The window, name plates and tab pictures are
//! the disc's `clg_*` models on the talk-window camera. Picking an item
//! closes the screen; Nook's talk does the offer, the payment and the order.

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use disc::RomSource;

use crate::catalog_lists;
use crate::game_disc::GameDisc;
use crate::input::{Action, Actions, InteractSet};
use crate::msg_window::{self, MsgAssets};
use crate::n64_material::N64Material;
use crate::states::{AppState, GameplaySet};

const LAYER: usize = 1;
const PAGE: i16 = 7;
/// `mCL_set_page_dl` draws the models through a scale of 16, in font pixels.
const MODEL_SCALE: f32 = 16.0;

const NAMES: [&str; 7] = [
    "clg_win_na1T_model",
    "clg_win_na2T_model",
    "clg_win_na3T_model",
    "clg_win_na4T_model",
    "clg_win_na5T_model",
    "clg_win_na6T_model",
    "clg_win_na7T_model",
];

/// Tab pictures, in `mCL_win_data` order. The second model is the icon.
const TABS_GFX: [[&str; 2]; 9] = [
    ["clg_win_haT_model", "clg_win_ha2T_model"],
    ["clg_win_kabeT_model", "clg_win_kabe2T_model"],
    ["clg_win_jyuutanT_model", "clg_win_jyuutan2T_model"],
    ["clg_win_fukuT_model", "clg_win_fuku2T_model"],
    ["clg_win_kasaT_model", "clg_win_kasa2T_model"],
    ["clg_win_tegamiT_model", "clg_win_tegami2T_model"],
    ["clg_win_haniwaT_model", "clg_win_haniwa2T_model"],
    ["clg_win_honeT_model", "clg_win_hone2T_model"],
    ["clg_win_onpuT_model", "clg_win_onpu2T_model"],
];

type Model = Vec<(Handle<Mesh>, Handle<N64Material>)>;

/// The open catalog. `result` is set when it closes: `None` is cancel,
/// `Some(item)` is the chosen item.
#[derive(Resource)]
pub struct CatalogUi {
    pub open: bool,
    pub result: Option<Option<u16>>,
    page: usize,
    top: [i16; 9],
    row: [i16; 9],
    lists: [Vec<u16>; 9],
}

impl Default for CatalogUi {
    fn default() -> Self {
        Self {
            open: false,
            result: None,
            page: 0,
            top: [0; 9],
            row: [0; 9],
            lists: std::array::from_fn(|_| Vec::new()),
        }
    }
}

impl CatalogUi {
    /// Build the nine collected lists and show the screen.
    pub fn open_from(&mut self, has: impl Fn(u16) -> bool) {
        self.lists = catalog_lists::collected(has);
        self.page = 0;
        self.top = [0; 9];
        self.row = [0; 9];
        self.result = None;
        self.open = true;
        info!(
            "catalog: {} collected items",
            self.lists.iter().map(Vec::len).sum::<usize>()
        );
    }
}

#[derive(Resource)]
struct CatalogModels {
    window: Model,
    /// Name plates: dim row, then the red selected row (`mCL_set_page_dl`).
    names: [Model; 7],
    names_on: [Model; 7],
    extra: Model,
    buttons: Model,
    bell: Model,
    arrow: Model,
    star: Model,
    /// Unselected tab, then the lit tab (`mCL_set_wchange_dl`).
    tabs: [Model; 9],
    tabs_on: [Model; 9],
}

#[derive(Resource, Default)]
struct Drawn {
    key: Option<(usize, i16, i16, usize, bool)>,
    root: Option<Entity>,
}

fn load_models(
    mut commands: Commands,
    disc: Res<GameDisc>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<N64Material>>,
    mut images: ResMut<Assets<Image>>,
) {
    let draw = |names: &[&str],
                prim: [u8; 4],
                env: [u8; 4],
                meshes: &mut Assets<Mesh>,
                materials: &mut Assets<N64Material>,
                images: &mut Assets<Image>| {
        let entries: Vec<u32> = names
            .iter()
            .filter_map(|name| {
                let sym = disc.catalog.get(name)?;
                (sym.source == RomSource::Rel).then_some(sym.offset)
            })
            .collect();
        if entries.len() != names.len() {
            warn!("catalog model {} is not on the disc", names[0]);
            return Vec::new();
        }
        let sink = crate::disc_gfx::run_display_lists_with_colors(
            &disc.rom,
            &entries,
            prim,
            255,
            env,
            &[],
        );
        crate::disc_town::sink_meshes_scaled(
            &disc,
            &sink,
            None,
            true,
            &crate::disc_gfx::Segments::default(),
            MODEL_SCALE,
            (meshes, materials, images),
        )
    };
    let white = [255, 255, 255, 255];
    let window = draw(
        &["clg_mwin_mode", "clg_mwin1_model"],
        white,
        [0; 4],
        &mut meshes,
        &mut materials,
        &mut images,
    );
    let mut names = std::array::from_fn(|_| Vec::new());
    let mut names_on = std::array::from_fn(|_| Vec::new());
    for (i, name) in NAMES.iter().enumerate() {
        names[i] = draw(
            &["clg_name_mode", name],
            [10, 10, 50, 255],
            [0; 4],
            &mut meshes,
            &mut materials,
            &mut images,
        );
        names_on[i] = draw(
            &["clg_name_mode", name],
            [205, 0, 0, 255],
            [0; 4],
            &mut meshes,
            &mut materials,
            &mut images,
        );
    }
    let extra = draw(
        &["clg_name_mode", "clg_mwin2_model"],
        white,
        [0; 4],
        &mut meshes,
        &mut materials,
        &mut images,
    );
    let buttons = draw(
        &["clg_win_cbT_model"],
        white,
        [0; 4],
        &mut meshes,
        &mut materials,
        &mut images,
    );
    let bell = draw(
        &["clg_name_mode", "clg_win_beruT_model"],
        white,
        [0; 4],
        &mut meshes,
        &mut materials,
        &mut images,
    );
    let arrow = draw(
        &["clg_win_shirushi1T_model"],
        [0, 50, 255, 255],
        [0; 4],
        &mut meshes,
        &mut materials,
        &mut images,
    );
    let star = draw(
        &["clg_win_hoshiT_model"],
        white,
        [0; 4],
        &mut meshes,
        &mut materials,
        &mut images,
    );
    let mut tabs = std::array::from_fn(|_| Vec::new());
    let mut tabs_on = std::array::from_fn(|_| Vec::new());
    for (i, [back, icon]) in TABS_GFX.iter().enumerate() {
        tabs[i] = draw(
            &["clg_tag_win_mode", back],
            [0, 0, 0, 255],
            [50, 50, 125, 255],
            &mut meshes,
            &mut materials,
            &mut images,
        );
        tabs[i].extend(draw(
            &["clg_tag_picture_mode", icon],
            [145, 145, 205, 255],
            [0; 4],
            &mut meshes,
            &mut materials,
            &mut images,
        ));
        tabs_on[i] = draw(
            &["clg_tag_win_mode", back],
            [0, 20, 110, 255],
            [50, 50, 255, 255],
            &mut meshes,
            &mut materials,
            &mut images,
        );
        tabs_on[i].extend(draw(
            &["clg_tag_picture_mode", icon],
            white,
            [0; 4],
            &mut meshes,
            &mut materials,
            &mut images,
        ));
    }
    let loaded = window.len()
        + names.iter().map(Vec::len).sum::<usize>()
        + extra.len()
        + buttons.len()
        + tabs.iter().map(Vec::len).sum::<usize>();
    info!("catalog models loaded: {loaded}");
    commands.insert_resource(CatalogModels {
        window,
        names,
        names_on,
        extra,
        buttons,
        bell,
        arrow,
        star,
        tabs,
        tabs_on,
    });
}

fn drive(mut ui: ResMut<CatalogUi>, mut actions: ResMut<Actions>, mut fresh: Local<bool>) {
    if !ui.open {
        *fresh = true;
        return;
    }
    if *fresh {
        *fresh = false;
        return;
    }
    let page = ui.page;
    let len = ui.lists[page].len() as i16;
    let page_jump = actions.pressed(Action::Run);
    if actions.consume(Action::MoveUp) || actions.consume(Action::MoveDown) {
        let down = actions.just_pressed(Action::MoveDown);
        let before = (ui.top[page], ui.row[page]);
        nudge(&mut ui, page, len, down, page_jump);
        if (ui.top[page], ui.row[page]) != before {
            crate::sound::se(crate::sound::se_id::CURSOR);
        }
    } else if actions.consume(Action::MoveLeft) || actions.consume(Action::MoveRight) {
        let next = if actions.just_pressed(Action::MoveRight) {
            (ui.page + 1) % 9
        } else {
            (ui.page + 8) % 9
        };
        ui.page = next;
        crate::sound::se(crate::sound::se_id::PAGE_OKURI);
    } else if actions.consume(Action::Confirm) || actions.consume(Action::Interact) {
        let index = (ui.top[page] + ui.row[page]) as usize;
        if let Some(&item) = ui.lists[page].get(index) {
            ui.result = Some(Some(item));
            ui.open = false;
            crate::sound::se(crate::sound::se_id::CHOICE_CONFIRM);
        }
    } else if actions.consume(Action::Cancel) {
        ui.result = Some(None);
        ui.open = false;
        crate::sound::se(crate::sound::se_id::MENU_EXIT);
    }
}

/// `mTG_move_cursol_in_catalog` for one axis. `jump` is the C-button page.
fn nudge(ui: &mut CatalogUi, page: usize, count: i16, down: bool, jump: bool) {
    if count <= 0 {
        return;
    }
    let mut top = ui.top[page];
    let mut row = ui.row[page];
    let target = top + row;
    if down {
        if jump {
            let idx = count - PAGE;
            if top + PAGE <= idx {
                top += PAGE;
            } else if target < count - 1 {
                top = idx.max(0);
                row = count - top - 1;
            }
        } else if target < count - 1 {
            if row < 5 || (target == count - 2 && row < 6) {
                row += 1;
            } else {
                top += 1;
            }
        }
    } else if jump {
        if top - PAGE >= 0 {
            top -= PAGE;
        } else if target > 0 {
            top = 0;
            row = 0;
        }
    } else if target > 0 {
        if row > 1 || (target == 1 && row > 0) {
            row -= 1;
        } else {
            top -= 1;
        }
    }
    ui.top[page] = top;
    ui.row[page] = row;
}

fn draw(
    mut commands: Commands,
    ui: Res<CatalogUi>,
    models: Option<Res<CatalogModels>>,
    assets: Res<MsgAssets>,
    mut drawn: ResMut<Drawn>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut text_materials: ResMut<Assets<StandardMaterial>>,
) {
    let key = ui.open.then(|| {
        let page = ui.page;
        (
            page,
            ui.top[page],
            ui.row[page],
            ui.lists[page].len(),
            models.is_some(),
        )
    });
    if key == drawn.key {
        return;
    }
    if let Some(root) = drawn.root.take() {
        commands.entity(root).despawn();
    }
    drawn.key = key;
    if !ui.open {
        return;
    }
    let layer = RenderLayers::layer(LAYER);
    let root = commands
        .spawn((
            Name::new("catalog"),
            Transform::default(),
            Visibility::default(),
            layer.clone(),
        ))
        .id();
    drawn.root = Some(root);
    if let Some(models) = &models {
        // One depth for the whole window. Each model used to restart at
        // z = 0, so the frame's later quads covered the bars, tabs and arrows.
        // Retail draws those after `clg_mwin1_model`. Text stays at z = 4.
        let mut z = 0.0;
        paint(
            &mut commands,
            &models.window,
            Vec3::ZERO,
            &mut z,
            &layer,
            root,
        );
        for i in 0..PAGE as usize {
            let plate = if i == ui.row[ui.page] as usize {
                &models.names_on[i]
            } else {
                &models.names[i]
            };
            paint(&mut commands, plate, Vec3::ZERO, &mut z, &layer, root);
        }
        paint(
            &mut commands,
            &models.extra,
            Vec3::ZERO,
            &mut z,
            &layer,
            root,
        );
        paint(
            &mut commands,
            &models.buttons,
            Vec3::ZERO,
            &mut z,
            &layer,
            root,
        );
        let list = &ui.lists[ui.page];
        let priced = list
            .get((ui.top[ui.page] + ui.row[ui.page]) as usize)
            .is_some_and(|item| catalog_lists::order_price(*item) != 0);
        if priced {
            paint(
                &mut commands,
                &models.bell,
                Vec3::ZERO,
                &mut z,
                &layer,
                root,
            );
        }
        // Scroll arrows: `Matrix_translate(-11, 80)` and the flipped one at -66.
        let top = ui.top[ui.page];
        if top != 0 {
            paint(
                &mut commands,
                &models.arrow,
                Vec3::new(-11.0 * MODEL_SCALE, 80.0 * MODEL_SCALE, 0.0),
                &mut z,
                &layer,
                root,
            );
        }
        if top + PAGE < list.len() as i16 {
            paint_rot(
                &mut commands,
                &models.arrow,
                Vec3::new(-11.0 * MODEL_SCALE, -66.0 * MODEL_SCALE, 0.0),
                &mut z,
                &layer,
                root,
            );
        }
        for (i, tab) in models.tabs.iter().enumerate() {
            if i != ui.page {
                paint(&mut commands, tab, Vec3::ZERO, &mut z, &layer, root);
            }
        }
        paint(
            &mut commands,
            &models.tabs_on[ui.page],
            Vec3::ZERO,
            &mut z,
            &layer,
            root,
        );
        if list.len() == catalog_lists::TABS[ui.page].len() && !list.is_empty() {
            let digits = list.len().to_string().len() as f32;
            let x = 38.0 + 7.0 + digits * 8.0 * 0.875;
            paint(
                &mut commands,
                &models.star,
                Vec3::new(x * MODEL_SCALE, -66.0 * MODEL_SCALE, 0.0),
                &mut z,
                &layer,
                root,
            );
        }
    }
    let page = ui.page;
    let list = &ui.lists[page];
    let widths = assets.widths.map(|w| ((w as f32) * 0.875).round() as u8);
    let mut y = 53.0;
    for i in 0..PAGE {
        let Some(&item) = list.get((ui.top[page] + i) as usize) else {
            break;
        };
        let color = if i == ui.row[page] {
            [255, 255, 255, 255]
        } else {
            [155, 155, 155, 255]
        };
        put_text(
            &mut commands,
            &mut meshes,
            &mut text_materials,
            &assets.font,
            &crate::items::name(item),
            Vec2::new(120.0, y),
            &widths,
            color,
            4.0,
            root,
            &layer,
        );
        y += 18.0;
    }
    let shown = if list.is_empty() {
        0
    } else {
        ui.top[page] + ui.row[page] + 1
    };
    let index = shown.to_string();
    let index_w = msg_window::text_width(&index, &widths);
    put_text(
        &mut commands,
        &mut meshes,
        &mut text_materials,
        &assets.font,
        &index,
        Vec2::new(188.0 - index_w, 181.0),
        &widths,
        [20, 20, 70, 255],
        4.0,
        root,
        &layer,
    );
    put_text(
        &mut commands,
        &mut meshes,
        &mut text_materials,
        &assets.font,
        &list.len().to_string(),
        Vec2::new(198.0, 181.0),
        &widths,
        [20, 20, 70, 255],
        4.0,
        root,
        &layer,
    );
    if let Some(&item) = list.get((ui.top[page] + ui.row[page]) as usize) {
        let price = catalog_lists::order_price(item);
        if price == 0 {
            put_text(
                &mut commands,
                &mut meshes,
                &mut text_materials,
                &assets.font,
                "Not for Sale",
                Vec2::new(48.0, 167.0),
                &widths,
                [205, 0, 0, 255],
                4.0,
                root,
                &layer,
            );
        } else {
            let text = bells(price);
            let w = msg_window::text_width(&text, &widths);
            put_text(
                &mut commands,
                &mut meshes,
                &mut text_materials,
                &assets.font,
                &text,
                Vec2::new(86.5 - w, 167.0),
                &widths,
                [205, 0, 0, 255],
                4.0,
                root,
                &layer,
            );
        }
    }
}

fn bells(price: u32) -> String {
    let digits = price.to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn paint(
    commands: &mut Commands,
    model: &Model,
    at: Vec3,
    z: &mut f32,
    layer: &RenderLayers,
    root: Entity,
) {
    for (mesh, material) in model {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(at + Vec3::new(0.0, 0.0, *z)),
            layer.clone(),
            ChildOf(root),
        ));
        *z += 0.02;
    }
}

/// The lower scroll arrow is the same model rotated 180° (`Matrix_RotateZ`).
fn paint_rot(
    commands: &mut Commands,
    model: &Model,
    at: Vec3,
    z: &mut f32,
    layer: &RenderLayers,
    root: Entity,
) {
    for (mesh, material) in model {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(at + Vec3::new(0.0, 0.0, *z))
                .with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
            layer.clone(),
            ChildOf(root),
        ));
        *z += 0.02;
    }
}

fn put_text(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    font: &Handle<Image>,
    text: &str,
    at: Vec2,
    widths: &[u8; 256],
    color: [u8; 4],
    z: f32,
    root: Entity,
    layer: &RenderLayers,
) {
    let mesh = msg_window::text_mesh(text, at, widths, color, usize::MAX);
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(font.clone()),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, 0.0, z),
        layer.clone(),
        ChildOf(root),
    ));
}

pub struct CatalogScreenPlugin;

impl Plugin for CatalogScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CatalogUi>()
            .init_resource::<Drawn>()
            .add_systems(
                OnEnter(AppState::InGame),
                load_models.run_if(resource_exists::<GameDisc>),
            )
            .add_systems(
                Update,
                (
                    drive.in_set(InteractSet::Ui).in_set(GameplaySet),
                    draw.run_if(resource_exists::<MsgAssets>),
                ),
            );
    }
}
