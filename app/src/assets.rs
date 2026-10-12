//! Asset loading gate and placeholder fallback.
//!
//! Files live under `app/assets/` (see `app/assets/README.md` for layout and the
//! licensing rule). A plugin that needs a file registers it at startup:
//!
//! ```ignore
//! let handle: Handle<Gltf> = asset_server.load("models/player.glb");
//! pending.track(&handle, "models/player.glb");
//! ```
//!
//! `AppState::Loading` holds until every tracked handle has finished
//! (loaded or failed). A failed file is logged and recorded in
//! [`AssetStatus`]; the game still starts, and the caller keeps showing its
//! placeholder primitive (check [`AssetStatus::failed`] or simply whether
//! the asset exists in `Assets<T>`). Missing art never crashes the game.

use bevy::asset::{RecursiveDependencyLoadState, UntypedAssetId};
use bevy::prelude::*;

use crate::states::AppState;

/// Handles (and named non-asset jobs, like reading the disc) still being waited on.
#[derive(Resource, Default)]
pub struct PendingAssets(Vec<(UntypedAssetId, String)>, Vec<&'static str>);

impl PendingAssets {
    /// Hold the loading screen until [`PendingAssets::release`] is called with `job`.
    pub fn hold(&mut self, job: &'static str) {
        self.1.push(job);
    }

    pub fn release(&mut self, job: &'static str) {
        self.1.retain(|j| *j != job);
    }

    /// Hold the loading screen until `handle` finishes loading.
    #[allow(dead_code)] // first caller arrives with the M2 art tasks
    pub fn track<A: Asset>(&mut self, handle: &Handle<A>, path: &str) {
        self.0.push((handle.id().untyped(), path.to_string()));
    }
}

/// Paths that failed to load (callers fall back to placeholders).
#[derive(Resource, Default)]
pub struct AssetStatus {
    failed: Vec<String>,
}

impl AssetStatus {
    #[allow(dead_code)] // first caller arrives with the M2 art tasks
    pub fn failed(&self, path: &str) -> bool {
        self.failed.iter().any(|p| p == path)
    }
}

/// Drop finished handles; leave `Loading` once nothing is pending.
fn check_loading(
    server: Res<AssetServer>,
    mut pending: ResMut<PendingAssets>,
    mut status: ResMut<AssetStatus>,
    mut next: ResMut<NextState<AppState>>,
) {
    pending.0.retain(
        |(id, path)| match server.get_recursive_dependency_load_state(*id) {
            Some(RecursiveDependencyLoadState::Loaded) => false,
            Some(RecursiveDependencyLoadState::Failed(err)) => {
                warn!("asset {path} failed to load ({err}); using placeholder");
                status.failed.push(path.clone());
                false
            }
            // Untracked by the server (e.g. never requested): nothing to wait for.
            None => false,
            _ => true,
        },
    );
    if pending.0.is_empty() && pending.1.is_empty() {
        // No title screen yet (roadmap `title-screen`): go straight in.
        next.set(AppState::InGame);
    }
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingAssets>()
            .init_resource::<AssetStatus>()
            .add_systems(Update, check_loading.run_if(in_state(AppState::Loading)));
    }
}
