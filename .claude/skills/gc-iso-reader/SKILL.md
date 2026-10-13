---
name: gc-iso-reader
description: Look up GameCube disc data fast for bevy-crossing - find a symbol in foresta.rel/main.dol and hex-dump it with pointer names, list/dump RARC archives, show the disc header and file list. Use before writing an ignored Rust probe to inspect disc data; zero-dependency Python, ~0.3 s per lookup.
---

# GC ISO Reader (pointer)

The skill and script live in `skills/gc-iso-reader/` (read its `SKILL.md`).
Quick use from the repo root:

```text
python skills/gc-iso-reader/gc_iso_reader.py sym rom <symbol-substring> -x 32
python skills/gc-iso-reader/gc_iso_reader.py hex rom rel 0x<offset> -n 64
python skills/gc-iso-reader/gc_iso_reader.py arc rom forest_1st.arc [member] -x 32
```

`rom` = `$BEVY_CROSSING_ISO` or the one image in `./rom`. Lookups print only;
never write disc data into the repo.
