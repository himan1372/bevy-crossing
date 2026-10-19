---
name: gc-iso-reader
description: "Inspect raw GameCube ISO/GCM images with the local zero-dependency Python reader: locate symbols and relocated pointers, inspect bounded REL/DOL bytes, or list FST/RARC contents. Use for runtime-disc source audits; not ISO 9660 mounting or game asset conversion."
---

# GameCube disc inspection

Run `python skills/gc-iso-reader/gc_iso_reader.py` from the repo root (`py` or
`python3` also work). Standard library only: no install, mount, build or Rust
probe needed. For runtime implementation, use the existing Rust `disc` crate.

## Resolve the image once

`rom` selects `BEVY_CROSSING_ISO`, otherwise the only `.iso`/`.gcm` in `./rom`.
If ambiguous, use the intended explicit path; quote paths with spaces. Run
`info` when the build is unknown: the supplied symbol maps target GAFE01.

This reader handles raw GameCube images, not CISO or ISO 9660. A `.iso` suffix
alone does not make the pycdlib/mounting skill appropriate.

## Choose a bounded lookup

| Question | Arguments after the script path |
|---|---|
| Header/build | `info rom` |
| Disc files in one directory | `ls rom DIRECTORY` |
| Symbol metadata | `sym rom NAME --max 5` |
| Exact symbol bytes | `sym rom NAME --exact -x 32` |
| Known file offset | `hex rom rel 0xOFFSET -n 64` (or `dol`) |
| Archive members | `arc rom forest_1st.arc` |
| Member bytes | `arc rom forest_1st.arc fgdata.bin -x 32` |

Find the name/path first; dump only needed fields. `sym` defaults to metadata
only; `--max` caps matches, `-x` bounds bytes per match. Single-quote PowerShell
symbol names containing dollar signs:

```powershell
python skills/gc-iso-reader/gc_iso_reader.py sym rom 'offset_data$396' --exact -x 32
```

The first REL lookup caches decompressed `foresta.rel.szs` in Temp; reuse it.
For custom Python imports, use `python -B` to avoid repository `__pycache__`.

## Interpret and recover

- Symbol matching is case-insensitive. Even exact names can have multiple
  entries: distinguish source/offset instead of silently taking the first.
- `rel`/`dol` offsets are decompressed REL/main.dol **file offsets**, matching
  `AssetCatalog`, not RAM or compressed-image offsets. `+4->name` annotates a
  relocated pointer; BSS/unresolved targets are not readable file bytes.
- Archive member offsets belong to decompressed RARC data, not REL/DOL.
  Keep the source tag with each offset; decode fields big-endian unless specified.
- No symbol match: shorten the distinctive substring and omit `--exact` once,
  then select the intended result. Missing map coverage does not mean no data.
- No member: list that archive and use its actual path. Invalid option: check
  subcommand `--help`. Bad magic/truncation/corruption: report the image error;
  unrelated flag retries will not repair it.

## Extraction or maintenance

Read [extraction and layout](references/extraction-and-layout.md) only for file
output, header/offset internals, or helper validation. Keep real images and all
extracted/derived game bytes outside the repository; never commit them.
