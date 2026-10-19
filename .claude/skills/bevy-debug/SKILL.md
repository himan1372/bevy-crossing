---
name: bevy-debug
description: "Diagnose Bevy system, query, schedule and runtime faults with focused source/log/BRP evidence before selecting a fix."
---

# Bevy debugging

Consult [project workflow](../references/project-workflow.md) once per task for claims, source authority, save isolation and validation boundaries.

Start from a narrow reproducible failure and a specific hypothesis. Choose the cheapest evidence that distinguishes likely causes; there is no requirement to climb a tooling ladder or install an inspector.

## Locate the failure

- **System never runs:** inspect plugin registration, schedule, state/run conditions and input focus. Check whether the required resource or scene exists.
- **Query matches nothing:** inspect the actual entity/components, hierarchy and spawn timing. `Added`/`Changed` filters and deferred commands can make an otherwise correct query miss the intended frame.
- **One-frame lag:** inspect explicit sets/dependencies, deferred application and transform propagation. Verify whether the reader needs local or global transforms and which schedule produces them before adding ordering edges.
- **Panic or access error:** read the complete diagnostic and named system, then inspect only the implicated parameters and callers.

Separate systems that mutate the same component are normally serialized by Bevy's scheduler. Overlapping mutable queries *within one system* can cause B0001. Use genuinely disjoint filters or a `ParamSet` with sequential access where appropriate; do not hide the conflict behind `unsafe` or a global mutex.

## Observe and correct

Use existing logs and [BRP inspection](../brp-visual-qa/SKILL.md), or add bounded subsystem logs around the uncertain boundary. Reflection coverage limits what BRP can show. Asset readiness, audio ownership and render extraction faults may require their subsystem's source rather than another ECS query.

Resolve API questions from the actual manifest and matching installed Bevy source. Schedule graphs, inspectors and lint tools are optional if already available or authorized and compatible; they are not prerequisites. Prefer explicit dependencies at the actual data boundary over broad ordering chains.

Apply the smallest justified fix, repeat the original scenario and check material adjacent behavior. Remove temporary instrumentation unless useful at an appropriate log level. Report observed evidence and remaining uncertainty; do not turn ordinary debugging into new CI, dependency or test work without that scope.
