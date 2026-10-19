---
name: module-port
description: Per-module playbook for porting Rust modules from the sibling rustimal-crossing repo into bevy-crossing's logic/ crate, and for merging the ~80 modules that exist in both but have diverged. Use when starting, continuing, or reviewing any migration port — one module at a time.
---

# module-port

Moves one pure-Rust research module from `rustimal-crossing` (`rust/src/<module>.rs`) into `bevy-crossing`'s `logic/` crate (`logic/src/<module>.rs`), or merges a module that exists in both repos but has diverged. This is the project's largest workstream (~110 modules to port, ~80 diverged merges). Work **one module per task claim** — never batch.

Related skills: read `task-coordination` first (claim protocol, comms.txt board), `rust-workflow` (build/test conventions), and `gamedb-cli` (navigating the sibling repo's source). The established pattern to imitate is the **furniture pilot**: `furniture.rs` was migrated to `logic/src/furniture.rs` with the C ABI stripped and native `pub` functions exposed, carrying over the bitflag bugfix.

## The per-module loop

Follow these steps in order for every module. Do not skip step 2 — it decides everything downstream.

### Step 0 — Claim the module

Claim exactly one module on the coordination board before touching code. See `task-coordination` for the claim format and the rule to re-read claims fresh before claiming. Suggested claim names: `port-<module>` (module only in the sibling repo) or `merge-<module>` (exists in both).

### Step 1 — Inventory the module

1. Read the sibling module: `rustimal-crossing/rust/src/<module>.rs` (full file, including its `#[cfg(test)]` tests).
2. Check whether `bevy-crossing/logic/src/<module>.rs` already exists.
   - **No** → this is a **port**.
   - **Yes** → this is a **merge**; go to Step 2 first, it matters most here.
3. List the module's public surface on both sides: `pub fn`, `pub struct`/`pub enum`, `pub const`, `pub trait`. Note any `extern "C"` / `#[no_mangle]` items — these get stripped in Bevy (see Step 4).
4. Find callers on both sides (`gamedb-cli` callers query, or `rg`). A module nobody calls is still worth porting if it's on the migration inventory, but caller lists decide shim needs in Step 5.

### Step 2 — Diff the public API first (merges especially)

Before writing any code, enumerate exactly what diverged. Use `cargo-public-api`, which diffs public API via rustdoc JSON:

```sh
# needs a nightly toolchain INSTALLED (not necessarily active):
#   rustup toolchain install nightly
cargo install cargo-public-api
cd <repo>
cargo public-api diff <ref-a>..<ref-b>
```

Practical approach for a merge: generate the public API list for the sibling module and for the `logic/` module, then compare. Items fall into three buckets:

- **Only in sibling** → candidate to bring over (usually bugfixes or new tests added upstream).
- **Only in logic/** → Bevy-side additions (e.g. C-ABI stripping, native API reshaping from the furniture pilot). Keep unless proven dead.
- **In both but different signature** → the real merge decisions; handle each in the merge note (Step 3).

Complementary tool: `cargo-semver-checks` lints *why* an API change breaks, on top of the same rustdoc JSON data:

```sh
cargo install cargo-semver-checks
cargo semver-checks
```

(Bonus signal in a big workspace: `cargo-shape-check` hashes only the public API surface of each workspace crate, telling you which downstream crates actually changed public interface and need attention.)

### Step 3 — Decision tree and merge note

**Port (module only in sibling):** decision is just *what to strip and reshape* — go to Step 4.

**Merge (exists in both):** for each diverged item from Step 2, pick one:

| Choice | When | Record |
|---|---|---|
| `take-new` (sibling wins) | Sibling has the bugfix / newer logic; `logic/` copy is stale | one line per item |
| `take-old` (logic/ wins) | `logic/` copy was deliberately reshaped for Bevy (furniture-pilot style) | one line per item |
| `cherry-pick` | Take the fix from one side, keep the shape of the other | one line per item, say which |

Write the decisions in a **merge note** — a short comment block at the top of the merged file or a `MERGE_NOTES.md` next to it:

```md
## merge note: <module> (YYYY-MM-DD, <agent>)
- take-new: `fn foo` — sibling fixed off-by-one (rustimal-crossing@<sha>)
- take-old: `struct Bar` — logic/ version already Bevy-reshaped, no C repr
- cherry-pick: `fn baz` — sibling's algorithm, logic/'s signature
```

If you cannot decide an item from the code alone, mark it `UNDECIDED` in the note and ask on the comms board — do not guess. Inconsistent merge choices across agents are the failure mode this skill exists to prevent.

### Step 4 — Move and reshape the code

1. Copy the module into `logic/src/<module>.rs` (port) or apply the merge decisions (merge).
2. **Strip the C ABI surface** (furniture-pilot rule): remove `extern "C"`, `#[no_mangle]`, C-repr shims, and raw-pointer adapter glue. Expose native `pub fn` / `pub struct` instead. If some other `logic/` module still needs the old C-shaped entry point, that is what the shim in Step 5 is for — do not keep C ABI in the canonical module.
3. Fix the module declaration: add `mod <module>;` to `logic/src/lib.rs` in the right alphabetical/section position (check how neighboring modules are declared; keep the existing ordering convention).
4. Carry over **tests**: the sibling modules have `#[cfg(test)]` unit tests — bring them. Tests are standing-authorized; they must pass (see `rust-workflow`).
5. Carry over **bugfixes, not bugs**: if the merge note says `take-new` for a fixed function, verify the fix is actually present in what you moved (spot-check against the sibling source, don't trust memory).

### Step 5 — Strangler-fig shim (only when something still references the old location)

The strangler-fig pattern for crate-to-crate Rust migration: the canonical implementation lives in the new place; the old path keeps compiling through a thin shim while callers migrate. You need a shim when:

- the sibling repo is still the source of truth for other agents' work, or
- `app/` or another `logic/` module imports the module through its old path.

Shim shape — re-export plus deprecation, nothing more:

```rust
// old location: rustimal-crossing/rust/src/<module>.rs  (or logic/src/old_<module>.rs)
#![allow(deprecated)]
#[deprecated(since = "0.x", note = "moved to bevy-crossing logic::<module>; migrate imports")]
pub use bevy_crossing_logic::<module>::*; // canonical path in the new home
```

Rules for shims:
- A shim contains **zero logic** — only `pub use`. If you find yourself adding logic to a shim, the migration step was wrong; redo it.
- Every shim gets a retirement entry in your task note (Step 7): who still imports the old path, and what has to change before the shim dies.
- Never create a shim "just in case". No callers on the old path → no shim.

Sources for the pattern: [fastruby strangler-fig guide](https://www.fastruby.io/blog/how-the-strangler-fig-pattern-helps-teams-pay-down-technical-debt.html), [programguru reference](https://programguru.org/system-design/strangler-fig-pattern/).

### Step 6 — Mechanical rewrites with ast-grep

For the repetitive parts (import-path rewrites, renamed items across callers), let `cargo check` surface the breakage, then apply codemods:

```sh
cargo install ast-grep
# dry-run the pattern first, then apply interactively:
ast-grep -p '<pattern>' -r '<replacement>' -i
```

Example: rewriting `use rustimal::<module>::` imports to the new crate path across callers. Always dry-run before `-i`, and hand-fix whatever the codemod can't express. Playbook source: [ast-grep Bevy migration post](https://ast-grep.github.io/blog/migrate-bevy.html).

### Step 7 — Per-module verification checklist

All boxes must be ticked before the module counts as done:

- [ ] `cargo test -p logic <module>` — all tests pass, including the carried-over sibling tests.
- [ ] `cargo doc -p logic --no-deps` builds with no new warnings for this module.
- [ ] Public-API diff (`cargo public-api`) between the pre- and post-change `logic/` crate shows **only intended changes** (the module addition + any deliberate reshapes from the merge note). Unexpected diffs = something leaked; investigate.
- [ ] Bevy-side smoke check: if `app/` consumes this module, `cargo check -p app` passes. (Full game runs are the BRP runbook's job, not this step's.)
- [ ] `mod <module>;` declared in `logic/src/lib.rs`; no duplicate declarations.
- [ ] Merge note written (merges) or port note written (ports: one line saying what was stripped/reshaped and which sibling commit it came from).
- [ ] Task note filed per `task-coordination`: module, port vs merge, decisions, shim status (created / not needed / retired), test result.

### Done definition and shim retirement

A module is **done** when: it lives canonically in `logic/`, all checklist boxes are ticked, the merge/port note exists, and either no shim was needed or the shim's retirement entry names its remaining callers.

Retire a shim when zero callers import the old path (verify with `rg` / `gamedb-cli`, not by assumption): delete the shim file, remove its `mod` line, re-run the checklist. Record the retirement in the task note and on the comms board so other agents stop using the old path.

## Common pitfalls

- **Batching modules in one claim.** The loop's value is per-module decision records. Batching produces one vague note covering five modules and the next agent can't tell what was decided for which.
- **Skipping the API diff on "obvious" merges.** The ~80 diverged modules diverged *because* both sides kept evolving. Eyeballing misses signature changes the diff catches in seconds.
- **Porting the C ABI along.** The sibling modules carry `extern "C"` adapter surface for the old wiring. Bevy-side canonical modules must not. Strip it; shim it only if callers demand it.
- **"Take-new" without checking the fix survived.** Copy-paste merges silently drop the very bugfix that motivated `take-new`. Diff the function after merging.
- **Leaving shims forever.** A shim without a retirement entry is tech debt with no owner. Every shim names its killer condition in the task note.

## Sources

- [fastruby.io — strangler fig pattern](https://www.fastruby.io/blog/how-the-strangler-fig-pattern-helps-teams-pay-down-technical-debt.html)
- [programguru.org — strangler fig reference](https://programguru.org/system-design/strangler-fig-pattern/)
- [cargo-public-api docs](https://docs.rs/cargo-public-api/latest/cargo_public_api/)
- [cargo-semver-checks](https://lib.rs/crates/cargo-semver-checks)
- [ast-grep — migrate Bevy playbook](https://ast-grep.github.io/blog/migrate-bevy.html)
