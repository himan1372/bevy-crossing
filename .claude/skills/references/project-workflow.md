# Shared project workflow

Applies to these project skills. Read once per task; the user's current instructions and repository state take precedence over historical skill recipes.

## Scope and coordination

Use the root `task-claimer.sh` with your actual agent identity (`codex` for Codex); inspect ownership and claim a bounded task before edits. Do not manually edit claims. Append findings to root `comms.txt`, keep the relevant task note current and release ownership at handoff. Use `docs/codex-handoff.md` for standing project context; never overwrite unrelated work. A skill does not authorize sibling-repository edits, tool installs, publishing or extra gameplay work.

## Source and architecture

Check manifests and local callers first. Current packages are `bevy-crossing`, `rustimal-logic`, `disc`; edition 2021, Bevy 0.19 (installed 0.19.1). Use matching installed API source, not another cached Bevy version. `app/Cargo.toml` declares extras 0.22.1 with a compatible-version range; inspect the actual resolved package before relying on its details.

Original C is at `C:/Users/phili/game c/shmanimal crossing/ACGC-PC-Port`. Migration authority is `docs/rustimal-migration-inventory.md` and pinned Git object `122867f762122eb266f04e319af0f0c31d54f476:rust/src/...`. A sibling working checkout can be older. Compare original semantics and current callers; preserve deliberate Bevy changes and native kernels.

Models, fonts, textures and audio load at runtime from Philip's disc. N64-derived Gfx/cKF content is not a generic BMD-to-GLB pipeline. Keep no-disc fallback playable and numeric save state intact when profiles are unavailable. Extracted/derived assets do not belong in commits.

## Runtime isolation

Never load, overwrite, stage or delete Philip's actual save for automation. Use an absolute executable path with a disposable Temp working directory, synthetic `savegame.json` and explicit `--disc` path when needed. Do not launch automated previews from the repository working directory. Hash the real save before/after runtime work to detect unintended changes; do not copy it into a synthetic scenario. Preserve settings, unrelated `.gitignore` changes and handoff notes. Screenshots, diagnostic exports and helpers stay in Temp. Close only processes you launched and identified.

## Validation and evidence

No tests are added or run without Philip's request. Old skill instructions or historical pilot permission do not override this. For Rust changes, run appropriate gates sequentially, fixing failures before continuing:

```powershell
cargo check --workspace -j 1
cargo clippy --workspace --all-targets -j 1 -- -D warnings
cargo fmt --all -- --check
```

One Cargo process at a time; poll an existing session to completion. Playable changes may also need capture/default/release builds and bounded runtime checks. Documentation-only work needs metadata/link/source checks, not Cargo. Do not repeat expensive checks without changed code or unresolved evidence.

Record source revision, what changed, observable verification and remaining limits. Keep implementation, bounded checks and Philip's acceptance distinct; compiling does not complete M4 or the migration. Update the game roadmap when gameplay scope/progress changes, rather than counting internal documentation maintenance as gameplay.
