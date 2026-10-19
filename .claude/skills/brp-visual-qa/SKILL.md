---
name: brp-visual-qa
description: "Inspect game state, drive isolated previews and verify rendered output through this checkout's Bevy Remote Protocol and screenshot extras."
---

# BRP visual QA

Consult [project workflow](../references/project-workflow.md) once per task for claims, source authority, save isolation and validation boundaries.

The app already registers `BrpExtrasPlugin` in `app/src/remote.rs`. `remote` enables BRP; `capture` includes it. Default builds omit both. Do not add another plugin or dependency to begin verification.

Read [protocol and isolated launch](references/protocol.md) when launching a preview or constructing requests. This local reference is sufficient; historical sibling runbooks may use obsolete methods and payloads.

## Drive a bounded scenario

State the expected numeric result and visible behavior first. Use a disposable working directory and synthetic save; query fresh entities after scene changes or reloads. Use reflected components that actually exist, with names/relationships to disambiguate them. Query order and entity IDs are not stable. A reflection error does not prove the gameplay object is absent.

Prefer input through the existing game flow. Direct component edits are setup tools and can bypass facing, collision, focus and gameplay transactions. Run dependent inputs sequentially, wait for the observed state/animation to settle, release held input and dismiss modal notices before the next action. Do not drive one world from competing scripts.

## Assess evidence

Compare full relevant numeric state for transactions and refusals, then capture and actually inspect the image for rendering claims. Include orientation, selection/focus and meaningful edge cases for the changed behavior. A screenshot cannot establish inventory equality; a correct save cannot establish visual alignment.

Keep images/helpers/synthetic saves in Temp. Record what was compared, what passed and what remains uncertain; distinguish bounded automation from Philip's acceptance. Deterministic capture or image diffs are useful when the task needs them, not grounds to add CI features, committed baselines or tests automatically. Close only the preview process you launched.
