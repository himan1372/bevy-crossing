use bevy::prelude::*;

/// Feature-gated BRP stack (port 15702 when `remote` / `capture` is enabled).
/// Purely additive: with default features this plugin does nothing.
pub struct RemotePluginGate;

impl Plugin for RemotePluginGate {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "remote")]
        {
            app.add_plugins(bevy_brp_extras::BrpExtrasPlugin::default());
        }
        let _ = app;
    }
}
