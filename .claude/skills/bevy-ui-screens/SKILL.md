---
name: bevy-ui-screens
description: Cookbook for building full UI screens (title, shop, inventory, dialogs) in Bevy 0.19 — layout patterns, BSN declarative UI, Feathers widgets, text input, and interaction. Use when implementing or fixing any game UI screen; pairs with brp-visual-qa for verifying the result on screen.
---

# bevy-ui-screens

Builds complete, working UI screens — not isolated widgets. The existing `bevy-019` skill covers Bevy 0.19 API notes; this skill is the *screen-building* cookbook: structure, declaration, interaction, and debugging.

## Step 1 — Screen structure (think in trees, not pixels)

Every screen is a UI node tree with the same anatomy:

1. **Root** — full-screen container, holds the screen's background and input blocking.
2. **Layout regions** — header / content / footer (or dialog box / button row for popups).
3. **Widgets** — buttons, labels, lists, text fields inside the regions.
4. **State driver** — a resource or state enum (`ScreenState::Shop`, `ScreenState::Title`) that owns which screen is visible; screens show/hide on state transitions, never by manually despawning half the tree.

Rules:
- One screen = one spawn function + one despawn/cleanup path. If showing screen B requires manually hiding three things from screen A, the state driver is wrong — fix that first.
- `bevy_ui` layout is flexbox-based: `Node` with flex direction, justify/align, padding, margins. Build the skeleton with placeholder colors first, get the layout right, then skin it.
- Keep UI code out of gameplay systems: UI systems read game state/resources and write UI components; they never mutate game logic directly. Gameplay changes flow through events (same direction as `gc-audio-pipeline`'s event→sound pattern).

## Step 2 — Declare UI with BSN

**BSN (Bevy Scene Notation)** is the 0.19-recommended way to declare scenes and UI, via the `bsn!` macro — declarative instead of imperative spawn chains. Feathers widgets are moving to BSN as their primary definition format, so learn it once and use it for screens:

- Declare the node tree (structure + styles + widget types) in `bsn!`, then spawn it — instead of 200 lines of `commands.spawn(Node { .. })` chains.
- Keep one BSN declaration per screen in its own module/file; the spawn function becomes "spawn this BSN, wire the interactions."

If you inherit a screen written as imperative spawn chains, don't rewrite it to BSN in the same task as a behavior fix — land the fix, then convert (or claim a conversion task). Mixed paradigms in one diff are unreviewable.

## Step 3 — Widget catalog

Two widget sources, in this preference order:

1. **`bevy_ui_widgets`** (enabled by default) — the smaller built-in set. Start here.
2. **`bevy_feathers`** (feature-gated; experimental through 0.19) — styled/themed collection: button, checkbox, radio, slider, toggle, color inputs, virtual keyboard. More widgets land in 0.20. Intended for editors but a legitimate starting point for game UI — don't reimplement a slider from scratch when Feathers has one.

Reference: [Feathers controls docs](https://docs.rs/bevy/latest/bevy/feathers/controls/index.html).

Theming: Feathers supports contextual theming — define the game's visual theme once (the M2 art direction's UI style) and apply it rather than styling every button individually. Unthemed one-off styles drift; that's how five screens end up with five button looks.

**Text input** is supported in 0.19 — use the built-in text input support for name entry / search fields rather than hand-rolling key-event accumulation.

## Step 4 — Interaction: observers and picking

- UI interaction goes through **observers** on pointer events (click/hover) — the picking system routes them to the entity under the cursor.
- Pattern: spawn widget → observe its interaction events → the observer writes a UI event or updates screen state. Keep the observer small; route through the same event types the screen's logic already uses.
- Every interactive element needs: a visible hover/press state, and a disabled state when its action is unavailable. A button that silently does nothing is a bug report waiting to happen.
- Keyboard/gamepad navigation: decide per screen whether focus navigation is required (menus: yes; HUD readouts: no). If yes, implement it in the same task — retrofitting focus later touches every widget.

## Step 5 — Debug layout with bevy_ui_debug

When a screen looks wrong, don't guess at flex values:

1. Enable **`bevy_ui_debug`** to visualize the layout (outlines, spacing, hierarchy).
2. Read the actual computed layout — the usual culprits are: a parent with no size constraint collapsing children, conflicting alignment on nested flex containers, or an element sized in the wrong units.
3. Fix the constraint, not the symptom: adding magic padding numbers to compensate for a collapsed parent breaks the next screen that reuses the component.

**Units note:** Bevy 0.20 adds `Val::Em`/`Val::Rem` and retained UI rendering changes (`extract_uinode_changes`, `ExtractedUiNode` API). You don't need them on 0.19, but if you touch render-world UI code, know the 0.20 migration moves that ground — check the [0.19→0.20 migration guide](https://bevy.org/learn/migration-guides/0-19-to-0-20/) before writing custom UI render code.

## Step 6 — Fonts and UI atlases

- Fonts and icon/UI atlases are assets: follow `asset-pipeline` conventions for where they live.
- One UI font family for the game (plus defined fallbacks), loaded once and shared — not per-screen loads.
- Text sizing: define a small scale ramp (e.g. title / heading / body / caption) and use it everywhere; arbitrary per-label sizes drift exactly like unthemed buttons.

## Step 7 — Verify the screen

Per `brp-visual-qa`:

1. Drive the screen with the scripted UI-drive pattern (find button → click → assert state).
2. Screenshot every state that matters: default, hover, pressed, disabled, error/empty states, and at least one non-default window size/aspect if the game supports resizing.
3. Text checks: no overflow/clipping, no placeholder text left in, text input actually accepts and commits input.

## Checklist (per screen task)

- [ ] Screen has one spawn function + one cleanup path, driven by a screen-state enum.
- [ ] Layout skeleton verified before skinning (placeholder colors are fine at this stage).
- [ ] Widgets from `bevy_ui_widgets`/`bevy_feathers` where they exist; no hand-rolled duplicates.
- [ ] Theme applied; no one-off button styles.
- [ ] All interactive elements have hover/press/disabled states.
- [ ] UI→game communication via events only; no direct gameplay mutation from UI systems.
- [ ] `bevy_ui_debug` used to confirm layout (not eyeballed flex guesses).
- [ ] Scripted drive + screenshots cover all states; committed per `brp-visual-qa`.

## Sources

- [taintedcoders — Bevy UI guide](https://taintedcoders.com/bevy/ui)
- [Feathers controls docs](https://docs.rs/bevy/latest/bevy/feathers/controls/index.html)
- [Bevy 0.19 → 0.20 migration guide](https://bevy.org/learn/migration-guides/0-19-to-0-20/)
- [Bevy 0.19 release highlights](https://singulism.com/en/2026-06-20-bevy-0-19-release-highlights/)
