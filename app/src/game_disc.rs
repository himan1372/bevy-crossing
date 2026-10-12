//! The player's own Animal Crossing disc: found and decoded at startup.
//!
//! Search order: `--disc <path>` on the command line, then `disc_path` in
//! `settings.json` (working directory), then any `.iso`/`.gcm`/`.ciso` in
//! `rom/`. The disc is read and `foresta.rel` decompressed during startup. Renderers read the
//! [`GameDisc`] resource; when it's absent they keep their placeholder art.
//! A missing or wrong disc never stops the game: it shows a notice instead.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy::prelude::*;
use disc::{AssetCatalog, Disc, Rarc, RomImage};

use crate::states::AppState;

const SETTINGS_FILE: &str = "settings.json";
const NOTICE_SECONDS: f32 = 10.0;

/// The decoded disc, present only when one was found and read.
#[derive(Resource, Clone)]
pub struct GameDisc {
    pub path: PathBuf,
    pub rom: Arc<RomImage>,
    pub catalog: Arc<AssetCatalog<'static>>,
    /// `fgdata.bin` from `forest_1st.arc`: the town object layouts.
    pub fgdata: Arc<Vec<u8>>,
    /// The player's face, clothing and clothing-palette banks from
    /// `forest_1st.arc` (`RESOURCE_FACE_BOY`, `_TEX_BOY`, `_PALLET_BOY`).
    pub player_faces: Arc<[u8]>,
    pub player_cloth: Arc<[u8]>,
    pub player_cloth_palettes: Arc<[u8]>,
}

/// Why there is no [`GameDisc`], for the on-screen notice.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
enum DiscStatus {
    Loaded,
    NotFound,
    Failed(String),
}

#[derive(Component)]
struct DiscNotice(Timer);

/// Where to look for the disc, before checking `rom/`.
fn configured_path(args: &[String], settings_json: Option<&str>) -> Option<PathBuf> {
    let flag = args
        .windows(2)
        .find(|w| w[0] == "--disc")
        .map(|w| PathBuf::from(&w[1]));
    flag.or_else(|| {
        let v: serde_json::Value = serde_json::from_str(settings_json?).ok()?;
        Some(PathBuf::from(v.get("disc_path")?.as_str()?))
    })
}

fn load(path: PathBuf) -> Result<GameDisc, String> {
    let fail = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
    let mut disc = Disc::open_path(&path).map_err(|e| fail(&e))?;
    let rom = Arc::new(RomImage::from_disc(&mut disc).map_err(|e| fail(&e))?);
    let catalog = Arc::new(AssetCatalog::builtin(&rom));
    let arc = Rarc::parse(disc.read_file("forest_1st.arc").map_err(|e| fail(&e))?)
        .ok_or_else(|| fail(&"forest_1st.arc is not a RARC archive"))?;
    let fgdata = Arc::new(
        arc.file("fgdata.bin")
            .ok_or_else(|| fail(&"no fgdata.bin in forest_1st.arc"))?
            .to_vec(),
    );
    let bank = |name: &str| -> Arc<[u8]> { arc.file(name).unwrap_or_default().into() };
    Ok(GameDisc {
        path,
        rom,
        catalog,
        fgdata,
        player_faces: bank("face_boy.bin"),
        player_cloth: bank("tex_boy.bin"),
        player_cloth_palettes: bank("pallet_boy.bin"),
    })
}

/// Reads the disc during startup (about half a second), so the town and
/// everything placed in it can be built from the disc's own layout.
pub fn start_disc_load(mut commands: Commands) {
    let args: Vec<String> = std::env::args().collect();
    let settings = std::fs::read_to_string(Path::new(SETTINGS_FILE)).ok();
    let Some(path) = configured_path(&args, settings.as_deref()).or_else(disc::find_image) else {
        info!("no game disc found; using placeholder art");
        commands.insert_resource(DiscStatus::NotFound);
        return;
    };
    info!("reading game disc {}", path.display());
    match load(path) {
        Ok(game_disc) => {
            info!(
                "game disc ready: {} ({} assets)",
                game_disc.path.display(),
                game_disc.catalog.len()
            );
            commands.insert_resource(game_disc);
            commands.insert_resource(DiscStatus::Loaded);
        }
        Err(e) => {
            warn!("game disc unusable, using placeholder art: {e}");
            commands.insert_resource(DiscStatus::Failed(e));
        }
    }
}

fn show_notice(mut commands: Commands, status: Res<DiscStatus>) {
    let text = match &*status {
        DiscStatus::NotFound => "No Animal Crossing disc found - using placeholder art.\n\
             Put your own disc image (.iso/.gcm/.ciso) in the rom folder."
            .to_string(),
        DiscStatus::Failed(e) => {
            format!("Couldn't use the disc image - using placeholder art.\n{e}")
        }
        DiscStatus::Loaded => return,
    };
    commands.spawn((
        DiscNotice(Timer::from_seconds(NOTICE_SECONDS, TimerMode::Once)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(16.0),
            left: Val::Px(16.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        children![(
            Text::new(text),
            TextFont {
                font_size: FontSize::Px(16.0),
                ..default()
            },
            TextColor(Color::WHITE),
        )],
    ));
}

fn tick_notice(
    mut commands: Commands,
    time: Res<Time>,
    mut notices: Query<(Entity, &mut DiscNotice)>,
) {
    for (entity, mut notice) in &mut notices {
        if notice.0.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub struct GameDiscPlugin;

impl Plugin for GameDiscPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, start_disc_load)
            .add_systems(Update, tick_notice)
            .add_systems(OnEnter(AppState::InGame), show_notice);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn flag_beats_settings() {
        let settings = r#"{"disc_path": "C:/games/ac.iso"}"#;
        assert_eq!(
            configured_path(&args(&["game", "--disc", "x.iso"]), Some(settings)),
            Some(PathBuf::from("x.iso"))
        );
        assert_eq!(
            configured_path(&args(&["game"]), Some(settings)),
            Some(PathBuf::from("C:/games/ac.iso"))
        );
    }

    #[test]
    fn nothing_configured() {
        assert_eq!(configured_path(&args(&["game"]), None), None);
        assert_eq!(configured_path(&args(&["game", "--disc"]), None), None);
        assert_eq!(configured_path(&args(&["game"]), Some("not json")), None);
        assert_eq!(
            configured_path(&args(&["game"]), Some(r#"{"disc_path": 3}"#)),
            None
        );
    }
}
