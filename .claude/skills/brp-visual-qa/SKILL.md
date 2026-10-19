---
name: brp-visual-qa
description: Visual verification over Bevy Remote Protocol — screenshots, BRP format discovery, and scripted UI drives against the running game. Designed to be merged into the existing BRP agent-loop runbook. Use when you need to confirm something on screen (art, UI, layout) or when BRP calls keep failing on JSON shape.
---

# brp-visual-qa

> **This doc is an amendment, not a standalone skill.** It is designed to be merged into the existing **BRP agent-loop runbook**: everything here assumes the runbook's baseline (game running, BRP reachable — default port 15702 in this project, gated behind the `remote`/`capture` features per project convention). Read the runbook first; this doc adds the *visual* layer the runbook lacks.

## When to use / when not to

- **Use:** verifying art-direction (M2), UI screens, texture/material application, layout — anything where "the state is right" isn't enough and you need to *see* it. Also use when `bevy/spawn`, `bevy/insert`, or `bevy/mutate` calls fail with schema errors (the format-discovery section fixes that whole class of failure).
- **Don't use:** for pure logic verification — the runbook's state queries already cover that. Don't screenshot as a substitute for assertions; screenshot to confirm what assertions can't express.

## Step 0 — Prerequisites (verify before proceeding)

Two things must be true in the `app/` crate; if either is missing, fix the crate first — nothing below works without them:

1. **BRP is enabled**: the `bevy_remote` feature plus `RemotePlugin`/`RemoteHttpPlugin` in the dev app. (Project convention gates this behind features — confirm which feature flag enables it before launching.)
2. **`bevy_brp_extras` is installed**: add its `BrpExtrasPlugin` to the dev app. It adds `screenshot`, `shutdown`, and `brp_extras/discover_format` BRP methods on top of stock BRP.

> **Open question (not yet verified):** whether `app/` already enables `bevy_remote`/BRP, and whether `bevy_brp_extras` compiles against **Bevy 0.19** — its docs predate 0.19. Check the repo and attempt the build before relying on this doc; if the extras crate doesn't build on 0.19, fall back to stock BRP methods and note it on the comms board.

Reference: [bevy_brp_mcp server docs](https://mcpservers.org/servers/natepiano/bevy_brp_mcp), [BRP protocol reference](https://docs.rs/bevy/0.15.0-rc.2/bevy/remote/index.html).

## Step 1 — Kill BRP trial-and-error with format discovery

The #1 BRP agent failure: `bevy/registry/schema` tells you a component's fields but **not** the exact JSON that `bevy/spawn`, `bevy/insert`, or `bevy/mutate` expect, so agents guess the envelope and fail repeatedly.

Fix: call **`brp_extras/discover_format`** for the operation + component *before* writing the real call. It returns the exact expected JSON shape, taken from the running app — and the same discovery info appears in BRP error messages, so read those before retrying blindly.

Workflow:

1. You want to e.g. insert component `Foo` on an entity.
2. `brp_extras/discover_format` → copy the returned JSON shape verbatim into your `bevy/insert` call.
3. If it still errors, the error message itself contains format hints — read it, don't guess again.

Make this a habit: **discover first, call second.** It removes an entire category of wasted round-trips.

## Step 2 — Screenshot via BRP

With `BrpExtrasPlugin` installed, request a screenshot over BRP. Recipe:

1. Get the game into the state you want to capture (drive it there via BRP, or position things with spawn/insert calls).
2. Call the screenshot method; save the returned image with a descriptive name: `<what>-<date>-<agent>.png` (e.g. `shop-interior-m2-2026-10-19-claude.png`).
3. **Look at it.** Compare against the expectation (source PNG for textures, mock for UI, previous screenshot for regressions). State explicitly in your task note what you compared and what matched.

Screenshots are for confirmation, not exploration: decide *before* capturing what "correct" looks like, then check.

## Step 3 — Deterministic screenshot-diff loop (for art regressions)

Eyeballing doesn't scale across M2's art iterations. For repeatable visual checks, make captures deterministic, following the vleue rendering-regression recipe:

1. Build/run with the **`bevy_ci_testing`** feature.
2. Drive the app with a **fixed frame time** (e.g. `Some(0.03)`), fire a screenshot event at a fixed frame (e.g. frame 200), and exit the app at a later fixed frame (e.g. 900).
3. Keep the run deterministic: fixed timestep, **seeded RNG only** — any unseeded randomness invalidates the diff.
4. Compare captures **hash-first** (identical → pass, no further work), then pixel-by-pixel / SSIM / FLIP only when hashes differ, to distinguish real regressions from noise.

Store baseline screenshots in the repo (they're binary assets — follow `asset-pipeline` conventions) and record which commit produced them. A diff failure means: real regression, nondeterminism crept in, or the baseline is stale — determine which before "fixing" anything.

Reference: [vleue — rendering regression testing in Bevy (PDF)](https://vleue.com/talks/bevy-meetup-2024-4-rendering-regression/rendering-regression.pdf).

## Step 4 — Scripted UI-drive pattern

For UI screens (retail, menus, dialogs), don't hand-drive clicks over BRP one at a time. Copy Bevy's own canonical pattern from `examples/remote/`:

- `app_under_test.rs` — the app you connect to and control.
- **`integration_test.rs`** — "Connects to a running Bevy app via BRP, finds a button, and clicks it." This is the template: connect → query for the UI entity → send the interaction → assert the resulting state.
- `client.rs` / `server.rs` — supporting pieces.

Adapt it per screen under test: find button → click → assert state change → screenshot the result (Step 2). The same script becomes a regression test you keep, not a one-off session.

Reference: [Bevy remote examples README](https://raw.githubusercontent.com/bevyengine/bevy/main/examples/README.md) (Remote Protocol section).

## Checklist (per verification task)

- [ ] Prerequisites confirmed (BRP enabled, extras plugin installed and building on 0.19 — or fallback noted).
- [ ] Format discovery used before any spawn/insert/mutate call (no blind retries).
- [ ] Screenshot captured with a descriptive name; expectation stated *before* capture.
- [ ] Comparison written in the task note: what was checked, what matched, what didn't.
- [ ] For regression-prone art: deterministic capture used, baseline recorded, diff method stated.

## Sources

- [bevy_brp_mcp — BRP access for coding agents](https://mcpservers.org/servers/natepiano/bevy_brp_mcp)
- [BRP JSON-RPC protocol reference](https://docs.rs/bevy/0.15.0-rc.2/bevy/remote/index.html)
- [Bevy remote examples README](https://raw.githubusercontent.com/bevyengine/bevy/main/examples/README.md)
- [vleue — rendering regression testing (PDF)](https://vleue.com/talks/bevy-meetup-2024-4-rendering-regression/rendering-regression.pdf)
