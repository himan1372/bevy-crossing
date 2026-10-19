---
name: windows-build
description: "Build and diagnose bevy-crossing on Windows using its real package names and features; package a playable build when requested."
---

# Windows builds

Consult [project workflow](../references/project-workflow.md) once per task for claims, source authority, save isolation and validation boundaries.

## Use the checkout's configuration

The app package/binary is `bevy-crossing`; other workspace packages are `rustimal-logic` and `disc`. Manifests currently use edition 2021 and Bevy 0.19. `remote` enables BRP and `capture` implies `remote`; there is no app feature named `dynamic_linking`.

Choose the profile required by the task. Philip plays release builds; reproduce profile-specific failures there. Do not switch toolchains, add linking features or rewrite optimization profiles merely to follow a generic recipe. Existing dev settings reduce dependency debug information to avoid Windows linker/PDB pressure.

For Rust changes, use the shared sequential gate. A capture build, when runtime inspection is needed, is:

```powershell
cargo build -p bevy-crossing --features capture -j 1
```

Default and release builds, when required for playable changes:

```powershell
cargo build --workspace -j 1
cargo build --workspace --release -j 1
```

These are separate commands: finish/poll each before starting the next. Documentation edits need none. Launch previews using the [isolated BRP procedure](../brp-visual-qa/references/protocol.md).

## Triage before changing configuration

Read the first substantive error and command/profile. For linker failures, distinguish missing build tools, file locks, PDB corruption and memory pressure. Inspect the installed host/toolchain only when relevant. Close an owned preview before rebuilding its executable; do not kill all Rust processes or delete caches by default.

For missing content, trace disc selection, symbol loading, shader embedding and current asset callers. Do not assume copying an assets directory fixes runtime disc data. Rendering faults also need the actual selected WGPU backend and adapter, not a guessed DirectX version.

For a requested distributable, read [packaging](references/packaging.md). A successful build does not establish portability or authorize publishing a release.
