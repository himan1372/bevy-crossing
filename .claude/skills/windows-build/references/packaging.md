# Packaging a requested Windows build

Use only when a distributable is in scope. Preserve current profiles and toolchain unless a demonstrated packaging problem requires a change. Build the intended release/features; verify what was actually enabled rather than assuming the EXE has no runtime dependencies.

Inventory runtime file access and embedded assets in the current app. Some comments in `app/src/assets.rs` describe older GLB loading; actual callers, shader embedding and `app/src/game_disc.rs` determine present requirements. Include the executable, required non-disc runtime files and applicable licenses/instructions. Do not copy a whole assets tree without identifying its purpose.

Do not bundle the user's ISO, extracted or derived disc content, save, settings or verification captures. Explain how the player selects their own disc through `--disc` or supported settings. Determine DLL/runtime requirements from the actual build target and selected backend; do not blindly copy DXC DLLs or promise portability from success on a developer machine.

Assemble into a new directory and smoke-check from a clean disposable working directory using synthetic state and an explicit user-provided disc. Check the missing-disc path too. Verify startup, representative rendering, audio and save location; report any environment that was not checked. Keep verification state out of the archive.

Creating a build/archive does not authorize uploading or publishing it. Follow the user's existing scope for delivery rather than adding release CI or changing hosting as part of packaging.
