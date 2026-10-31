# Nook Welcome and House Choice Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan
> task by task. Philip has already directed execution; keep the existing native
> workflow and no-tests instruction.

**Goal:** Continue the accepted station arrival into Nook's original welcome,
house tour, inspection/choice and payment, then the existing part-time job.

**Architecture:** Keep original message/choice/order data on the disc. A focused
app intro-house controller owns transient movement/talk/door/payment stages.
Persist the selected home in the existing optional version-1 house record and
route ownership-dependent presentation/services through that slot. Reuse native
inventory/first-job and handover kernels rather than duplicate gameplay systems.

**Tech Stack:** Rust 2021, Bevy 0.19.1, existing disc/logic crates and serde JSON.

**Spec/source:** completed/intro-new-game-gate.md slice 4; original
ac_intro_demo_move.c_inc and ac_npc_rcn_guide*.c_inc. Native flow is the agreed
approach; a fixed-house shortcut would discard the requested choice, and a new
dialogue/scene framework would duplicate existing infrastructure.

## Global Constraints

- No new tests or test runs; use source checks, Cargo gates and bounded runtime QA.
- No real-save automation, extracted assets, new dependencies or sibling edits.
- Retain legacy saves' home 0, numeric room data and existing first-job behavior.
- One claimed task, current main checkout, at most one reused reviewer.

## Review Focus

Cancellation must preserve the bag, wallet and loan; inspecting a house must not
own it. A nonzero selected slot must drive the door, roof, mailbox, gyroid and
later reload. Scene transitions and dialogue completion must release input.
Nook/player movement must animate and respect the existing terrain. Missing disc
data must preserve existing fallback gameplay and saved numeric state.

### 1. Home ownership and inspection

- [x] Add a defaulted selected-home slot to home::HouseSave, publish the slot at
  initialization/load and connect the existing door/exterior/mail/gyroid/statue
  consumers. Default remains 0 for legacy saves.
- [x] Reuse the existing player room/door transition for each vacant home during
  the intro; keep the inspected slot transient until the purchase is accepted.
- [x] Audit all ownership consumers and compare a nonzero-slot save/reload.

### 2. Nook's intro controller

- [x] Add app/src/intro_house.rs and register its plugin. Continue from station
  exit (original z >= 970) rather than calling finish_intro after the short walk.
- [x] Drive original 0x07DE/DF/E1 greeting and tour, original house-entry/exit
  messages and choices, using DemoOrders and the existing message renderer.
- [x] Animate Nook and player movement/facing; keep choice-acre and modal input
  ownership consistent with the original sequence.

### 3. Payment and first-job handoff

- [x] Offer the starting quest Bell bag through the existing pockets renderer;
  validate its exact slot/condition, cancel without mutation, and reuse the
  player-to-NPC money handover at the original message order.
- [x] Commit the selected home and starting loan/job state only at completion;
  clear transients and let the existing shop guide run the first job.

### 4. Verification and handoff

- [x] Run check, clippy and fmt sequentially, then build capture.
- [x] Isolated arrival-to-house run: greeting/tour, inspection rejection and
  acceptance, payment cancellation/acceptance, nonzero home, save/reload and
  first-job entry. Inspect screenshots plus numeric state.
- [x] Focused review; resolve findings, build default/release as needed.
- [x] Update roadmaps/task/comms, complete claim, commit explicit files and push.
