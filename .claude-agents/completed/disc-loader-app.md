# disc-loader-app

**Problem:** the game needs the player's disc loaded at startup (or a clear fallback) before rendering from it.

**Solution:** `app/src/game_disc.rs` (`GameDiscPlugin`): find the image (`--disc <path>`, then `disc_path` in `settings.json`, then `rom/`), open + decode on the async compute pool during `AppState::Loading`, store `GameDisc` (RomImage + AssetCatalog) as a resource. Missing/wrong disc → on-screen notice, placeholder art, game still starts.

- [x] Plugin + resource + async load holding the loading gate
- [x] Path resolution with tests
- [x] Notice on missing/bad disc
- [x] Gate passes; run with and without the disc
