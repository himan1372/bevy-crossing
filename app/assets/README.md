# assets/

Everything the game loads at runtime. It lives in `app/` because Bevy
resolves `assets/` next to the crate's `Cargo.toml` when started with
`cargo run` (set `BEVY_ASSET_ROOT` to override, e.g. for a release zip).

| Folder | What goes here | Format |
|---|---|---|
| `models/` | Characters, buildings, nature, furniture | `.glb` (Blender export — see `.claude/skills/blender-gltf` and `asset-pipeline`) |
| `textures/` | Standalone textures (UI, terrain) | `.png` (`.ktx2` later if needed) |
| `audio/music/` | Background music | `.ogg` |
| `audio/sfx/` | Sound effects | `.ogg` |
| `fonts/` | UI fonts | `.ttf` / `.otf` |

## Licensing rule (no exceptions)

- **No Nintendo IP**: no ripped or traced models, textures, audio, logos,
  or character likenesses from Animal Crossing or any other Nintendo game.
- Only **original** work made for this project, or assets under **CC0** or
  another licence that allows redistribution (e.g. SIL OFL for fonts).
- Every third-party file gets a line in `CREDITS.md` (file, author,
  licence, source URL) in the same commit that adds it.

## Loading

Register a file with `PendingAssets::track` (see `app/src/assets.rs`). The
game stays in its loading state until tracked files finish. A file that
fails to load is logged and the game falls back to its placeholder shape,
so missing art never crashes the game.
