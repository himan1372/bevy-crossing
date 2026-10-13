---
name: gc-iso-reader
description: Read Nintendo GameCube (GCM/ISO) disc images on Windows with a zero-dependency Python CLI: show the disc header, list files from the FST, and extract files or main.dol. Use when Claude needs to inspect the contents of a GameCube disc image (offsets, file list, extracting a file) without building anything.
---

# GC ISO Reader

`gc_iso_reader.py` reads raw GameCube disc images (`.gcm` / `.iso`). Pure Python
standard library — no install, no build step, works on Windows with the
`python` or `py` launcher. Output is plain text (no ANSI codes) so it renders
correctly in legacy PowerShell/conhost as well as Windows Terminal.

Its offsets mirror the repo's Rust `disc` crate (`disc/src/image.rs`), so both
readers agree: header magic `0xC2339F3D` at `0x1C`, game id at `0x00`, DOL
offset at `0x420`, FST offset/size at `0x424`/`0x428`, 12-byte FST entries.

## Commands

All commands take the image path first. On Windows run them as
`python skills\gc-iso-reader\gc_iso_reader.py <cmd> ...` from the repo root
(or `py ...` if `python` is not on PATH).

```text
info <image>                    header: game id/name, maker, disc#, version, DOL/FST offsets
ls <image> [dir]                list files (optionally under one directory)
extract <image> <path> -o <out> extract one file by disc path (case-insensitive, leading / ok)
dol <image> -o <out>            extract main.dol (size from its 18 section headers)
```

Examples:

```text
python skills\gc-iso-reader\gc_iso_reader.py info "C:\dumps\game.gcm"
python skills\gc-iso-reader\gc_iso_reader.py ls "C:\dumps\game.gcm" files
python skills\gc-iso-reader\gc_iso_reader.py extract "C:\dumps\game.gcm" opening.bnr -o out\opening.bnr
```

## Operating rules

1. **Never commit a real game image to the repo.** The skill only reads images
   from outside the repo (the user's own dumps).
2. **Paths with spaces** must be quoted. Output directories are created as needed.
3. A non-zero exit with `error: ...` on stderr means: bad magic (not a GameCube
   image), truncated file, corrupt FST, or file not found on disc. Report the
   message; do not retry with different flags.
4. For Rust code, prefer the in-repo `disc` crate (`Disc::open`, `read_file`,
   `read_dol`) over shelling out to this script — the script exists for quick
   interactive inspection on machines without a Rust toolchain.
5. `test_gc_iso_reader.py` builds synthetic images in a temp dir (no game data)
   and covers header parsing, nested directories, byte-exact extraction, DOL
   sizing, and the error paths. Run it after any change:
   `python skills\gc-iso-reader\test_gc_iso_reader.py`.
