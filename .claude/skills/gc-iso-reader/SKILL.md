---
name: gc-iso-reader
description: "Inspect raw GameCube ISO/GCM symbols, relocated pointers, REL/DOL bytes, and FST/RARC contents with the repository Python reader. Not ISO 9660 mounting or asset conversion."
---

# GC ISO Reader (pointer)

Read the [canonical skill](../../../skills/gc-iso-reader/SKILL.md) before use.
The helper lives alongside it. From the repo root, start with a small lookup:

```text
python skills/gc-iso-reader/gc_iso_reader.py sym rom furniture_quality --exact -x 32
```

Keep command semantics, image selection, offset interpretation and recovery
guidance in the canonical skill. Do not duplicate them here.
