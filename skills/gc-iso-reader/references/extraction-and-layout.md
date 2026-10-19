# Extraction and binary layout

Read this for file output or manual header/structure decoding. Ordinary symbol
and archive lookups only need the main skill.

## Extract only the required file

From the repository root, keep output in Temp, never in this repository:

```powershell
$outFile = Join-Path $env:TEMP 'bevy-crossing-opening.bnr'
python skills/gc-iso-reader/gc_iso_reader.py extract rom opening.bnr -o $outFile
$dolFile = Join-Path $env:TEMP 'bevy-crossing-main.dol'
python skills/gc-iso-reader/gc_iso_reader.py dol rom -o $dolFile
```

`extract` resolves the exact FST file path case-insensitively, with a leading
slash permitted; use `ls` to find it first. `dol` sizes main.dol from its 18
section headers. Parent output directories are created as needed. Select an
output filename belonging to this task rather than overwriting other evidence.
Do not commit real images, extracted bytes, or derived game assets.

## Offset spaces

- Header magic: `0xC2339F3D` at 0x1C; game ID at 0x00, maker at 0x04, disc number
  at 0x06 and revision at 0x07.
- DOL image offset: header 0x420. FST image offset/size:0x424/0x428. FST entries
  are 12 bytes, followed by a null-terminated name table.
- `foresta.sym` `r`/`d` values are relative to REL sections 4/5. The helper adds
  section file offsets; `dol.sym` RAM addresses are mapped through DOL sections.
  `sym`/`hex` therefore print file offsets, matching Rust AssetCatalog.
- REL `R_PPC_ADDR32` annotations name relocated targets; BSS has no file bytes.
  Archive member offsets are relative to decompressed archive data.

These mirror `disc/src/image.rs`, `rom.rs`, `rarc.rs` and `yaz0.rs`; consult those
implementations and the original structure definition when decoding fields.

## Validate the change being made

For documentation, check frontmatter, linked paths, CLI help and small read-only
examples; no Cargo build or test run is needed. If the reader code changes,
follow project validation instructions. `test_gc_iso_reader.py` is the existing
synthetic harness, available when tests are requested; it needs no real image.
