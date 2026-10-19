---
name: bevy-debug
description: Systematic ECS debugging ladder for Bevy — from log traces to schedule-graph visualization, system validation, linting, and runtime inspection. Use when a Bevy system misbehaves, panics, or silently does nothing; follow the ladder in order instead of trying random fixes.
---

# bevy-debug

Agents burn the most cycles on ECS bugs: system ordering, borrow conflicts, and queries that silently match nothing. This skill is a ladder — start at rung 1 and climb only when the rung below is exhausted. Random fix attempts are how a 10-minute bug becomes a 3-hour one.

Related: `bevy-019` (API notes), `brp-visual-qa` (runtime state via BRP), `rust-workflow` (general Rust debugging).

## Rung 1 — Logs and traces first

Before any tooling: reproduce the bug, then read what the app already tells you.

1. Run with Bevy's logging at debug level for the relevant subsystem (`RUST_LOG` / log plugin settings — check how `app/` configures logging).
2. Add temporary `info!`/`debug!` logs at the *boundaries*: system entry ("system X ran"), query results ("query matched N entities"), and decision points ("condition true/false"). Remove them when done — or keep the decision-point ones if they're genuinely useful.
3. Read panics fully: Bevy system panics name the system and often the exact schedule stage. The answer is usually in the first 5 lines, not the 50-line backtrace.

If the system never runs at all, skip to Rung 3 (ordering). If it runs but does nothing, stay here and check the query match count — "query matched 0 entities" explains most silent failures.

## Rung 2 — The common-pitfalls checklist

Run through this list before reaching for heavier tools. These are the bugs Bevy agents write most (adapted from the community Bevy agent skill's pitfalls reference):

- [ ] **Borrow conflicts** — two systems (or a system and the query itself) mutably accessing the same component in the same stage. The compiler usually catches these; when it doesn't (e.g. across schedules), the symptom is a runtime panic naming the conflicting accesses.
- [ ] **System ordering** — system A reads what system B writes, but they run in the wrong order (or in the same stage with no ordering constraint). Fix with explicit `.after()`/`.before()` or schedule placement, not by hoping.
- [ ] **Change detection** — `Changed<T>` / `Added<T>` filters only fire on actual writes through `Mut<T>`; writing the same value still counts, but *not writing at all* doesn't. A system gated on `Changed<T>` that never fires usually means nothing is writing `T`.
- [ ] **Query filters** — `With<T>` / `Without<T>` / `Or<...>` excluding the entities you expect. Test the query standalone: does it match what you think it matches?
- [ ] **System registration** — the system was written but never added to a schedule/plugin. (Embarrassing, common, and rung-1 logs catch it instantly — "system X ran" never prints.)
- [ ] **Asset handles** — systems running before assets load, operating on placeholder handles. Gate on asset-load state or check `is_loaded`.

Source for the checklist pattern: [eliteai.tools Bevy agent skill](https://eliteai.tools/agent-skills/bevy-1) (`common_pitfalls` reference).

## Rung 3 — Visualize the schedule

When ordering is suspect (rung 2's checklist points at it, or systems interact in confusing ways), stop reasoning about the schedule from code — **look at it**:

- **`bevy_mod_debugdump`** renders schedule graphs and render graphs (egui-based visualizers). Add it to the dev app, dump the schedule, and read the actual execution order and system dependencies. "I thought X ran before Y" becomes a picture instead of a belief.

This rung exists because schedule reasoning from source lies: plugins add systems you didn't write, and default stage placement isn't always what you'd guess.

## Rung 4 — Validate the system itself

For "invalid system" errors (wrong parameter types, bad combinations):

- **`bevycheck`** — annotate the suspect system with `#[bevycheck::system]` to get clear, specific error messages about what's invalid, instead of Bevy's generic system-param errors.

Use this when the compiler or runtime rejects a system signature and the message doesn't say *which* parameter is the problem.

## Rung 5 — Lint the codebase

- **`bevy_lint`** — Bevy-specific linter: catches footguns, performance pitfalls, and unidiomatic patterns agents commonly write. Run it in CI so the same mistakes don't get reintroduced by the next agent.

If CI doesn't run it yet, that's a task worth claiming — a linter that runs on every commit is worth more than this whole skill doc.

## Rung 6 — Inspect runtime state

When the code looks right but the world is wrong, look at the world:

- **`bevy-inspector-egui`** — runtime ECS inspection and editing: browse entities, components, and resources live while the game runs. Change a value and watch what happens — the fastest way to test "is this component the problem."
- **BRP queries** — per `brp-visual-qa`, query entities/components over BRP from a script. Better than the inspector when you need to assert state programmatically or compare across runs.

Pick the inspector for interactive poking, BRP queries for scripted checks. Both beat adding print statements to six systems.

## Tooling reference

All tools live in the ecosystem catalog — verify Bevy 0.19 compatibility before adding any of them:

- [bevy.org/assets](https://bevy.org/assets/) — Development tools section (`bevy_mod_debugdump`, `bevycheck`, `bevy_lint`, `bevy-inspector-egui`)

## Anti-patterns (don't do these)

- **Shotgun system reordering** — adding `.after()` everywhere until the bug disappears, without understanding the dependency. It "fixes" this bug and creates the next one.
- **Duplicating queries to work around borrows** — splitting one system into two that both run and both write is a data race with extra steps. Restructure the access instead.
- **Debugging from a stale mental model of the schedule** — if you haven't dumped the schedule (rung 3), you don't know the schedule.
- **Leaving debug systems/logs committed** — temporary instrumentation gets removed before the task closes. Permanent observability goes through proper logging levels, not `println!` in systems.

## Checklist (per debugging task)

- [ ] Bug reproduced; rung-1 logs show what actually happens (not what you assumed).
- [ ] Pitfalls checklist run; the specific failed item named in the task note.
- [ ] If ordering: schedule dumped and read (rung 3), fix is an explicit ordering constraint.
- [ ] If system signature: `bevycheck` used, invalid parameter identified.
- [ ] Fix verified by re-running the reproduction — plus the relevant test suite still green.
- [ ] Temporary instrumentation removed; task note records root cause (so the next agent with the same symptom finds it).
