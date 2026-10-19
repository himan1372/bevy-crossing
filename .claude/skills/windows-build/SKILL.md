---
name: windows-build
description: Building, packaging, and shipping bevy-crossing on Windows — toolchain setup, fast iteration builds, release profiles, and the EXE+assets distribution recipe. Use when setting up a build, cutting a release for Philip to playtest, or fixing a Windows-only build failure.
---

# windows-build

All development and playtesting happens on Philip's Windows PC. This skill owns everything from toolchain setup to a zipped build he can double-click. Write every step so a beginner can follow it — PowerShell commands, no assumed knowledge. (`rust-workflow` covers generic Rust conventions; this skill is Bevy-on-Windows specifically.)

## Step 1 — Toolchain setup (once per machine)

Two toolchains work on Windows; pick one and stick with it:

- **MSVC** (recommended default): install the Visual Studio Build Tools with the C++ workload, then `rustup default stable-x86_64-pc-windows-msvc`.
- **GNU**: `rustup default stable-x86_64-pc-windows-gnu` (MinGW). No Visual Studio needed, but some crates assume MSVC — if a dependency fails to build on GNU, switch to MSVC before debugging the crate.

Verify in PowerShell:

```powershell
rustc -V
cargo -V
rustc -Vv   # check the "host:" line shows the intended target triple
```

If `cargo build` fails with linker errors on a fresh machine, the cause is almost always the toolchain (missing Build Tools / wrong triple), not the code — fix the toolchain first.

## Step 2 — Fast iteration builds (daily development)

Iteration speed is the whole game. Rules:

1. **Never develop with `--release`.** Release mode disables debug assertions/validations and has slow incremental builds — it hides bugs and wastes time. Develop on the dev profile.
2. Enable Bevy's **`dynamic_linking`** feature for dev builds: it keeps link times down dramatically. (It's dev-only; never ship with it.)
3. If iteration is still slow, the next lever is the dev profile's optimization level for GPU-bound checks — see the [cheatbook performance pitfalls](https://Bevy-cheatbook.github.io/pitfalls/performance.html) before reaching for exotic flags.

```powershell
# the normal dev loop:
cargo run -p app --features dynamic_linking,remote
```

(Feature flags per project convention — `remote`/`capture` gate the BRP runbook tooling; check what the repo currently uses.)

## Step 3 — Release profile (shipping builds only)

When cutting a build for Philip to playtest or distribute, use an aggressive release profile. Add to the workspace `Cargo.toml` (or the `app` crate):

```toml
[profile.release]
lto = true
opt-level = 3
codegen-units = 1
incremental = false
```

Then:

```powershell
cargo build -p app --release
```

Expect the first release build to take a long time (LTO + single codegen unit). That's normal — it's a shipping build, not an iteration build. Never use `--release` for debugging; reproduce bugs on dev builds where assertions are on.

## Step 4 — Packaging recipe (what Philip actually gets)

The output of `cargo build --release` is a standalone EXE (no extra DLLs needed), but it is **not** shippable alone. The package is:

```
bevy-crossing/
  bevy-crossing.exe
  assets/            <-- the whole assets folder, same relative layout as the repo
```

Checklist:

- [ ] EXE built with `--release` (and without `dynamic_linking`).
- [ ] `assets/` copied next to the EXE, preserving the repo's internal layout. If the game can't find a texture/sound/font, the assets folder is in the wrong place relative to the EXE — that's the bug 90% of the time.
- [ ] **DX12 shader support:** for improved DX12 shader compilation, place Microsoft's `dxcompiler.dll` and `dxil.dll` next to the EXE. Bevy auto-detects them. (Only needed if targeting DX12; skip if the game runs on Vulkan/DX11 — but know this step exists.)
- [ ] **MSVC redist note:** binaries built with the MSVC toolchain may need the MSVC C/C++ Runtime Redistributables installed on the player's machine. If Philip's PC already has Build Tools, it works there and fails on a fresh machine — that's the redistributable missing, not a broken build. Either bundle the redist installer or link to it in the release notes.
- [ ] Zip it: `bevy-crossing-windows.zip` containing the folder above. That's the distributable.

Smoke test before handing it over: unzip to a *different* folder (or a different machine), double-click the EXE, confirm the game starts, a texture renders, and a sound plays. Testing the zip from the build folder proves nothing.

## Step 5 — Reference CI (so releases stop being manual)

Hand-rolled release zips rot. The model to copy is **`bevy_game_template`**, which ships reference release workflows for Windows, Linux, macOS, and WASM — use its Windows workflow as the template for this repo's release CI. What the CI should do:

1. Check out, install the Rust toolchain (pinned version), build `--release`.
2. Assemble the package (EXE + `assets/` + DXC DLLs if used).
3. Upload the zip as a build artifact / GitHub Release.

Reference: [bevyengine.org/assets](https://bevyengine.org/assets/) (Templates section).

## Windows-only failure triage

| Symptom | Check first |
|---|---|
| Linker errors on fresh machine | Toolchain: Build Tools installed? Correct target triple? (Step 1) |
| Game runs in dev, black screen / missing assets in release zip | `assets/` folder placement relative to EXE (Step 4) |
| Works on dev PC, crashes on another PC | MSVC redistributable missing (Step 4) |
| Shaders broken / slow on DX12 | `dxcompiler.dll`/`dxil.dll` next to EXE (Step 4) |
| Iteration builds suddenly slow | Someone committed `--release` into the dev workflow, or `dynamic_linking` got dropped (Step 2) |

## Sources

- [bevy-cheatbook — desktop platform development](https://deepwiki.com/bevy-cheatbook/bevy-cheatbook/5.1-desktop-platform-development)
- [bevy-cheatbook — performance pitfalls (profiles)](https://Bevy-cheatbook.github.io/pitfalls/performance.html)
- [bevyengine.org/assets — bevy_game_template (CI reference)](https://bevyengine.org/assets/)
