These notes retain the original skill's gamedb v1.0 behavior. Check installed
subcommand help before relying on version-specific flags or limits.

# Indexing and module reference

Every file is assigned a module **when it is parsed**, from one of:

- `--rules FILE`: an explicit rule file;
- `--rules derive`: buckets by the first two path/namespace segments, ignoring any rule file;
- default: `<root>/.gamedb/modules.txt` if it exists, otherwise derive.

Rule selectors: `b:` file, `x:` exact namespace, `n:` namespace prefix,
`m:` path segment. See `modules.example.txt` in the gamedb checkout for syntax
and specificity ranking. Unclaimed files land in `unmapped`.

`gamedb modules` lists `module_id files functions state verified_by system` and ends with `taxonomy: <rule source>`.

**Changing the rules does not reassign already-indexed files.** After adding or editing a rule file, run `gamedb index -r SRC --force` (plus the same `--rules`), otherwise `modules` shows the new module ids with `0 0` and the old assignments stay in `unmapped` or their old buckets. Pass the same `--rules` to `index`, `modules`, and `set-module`; `set-module` rejects a module id the active rules do not know.

### Tracking rewrite progress

```sh
gamedb set-module -r SRC --module core.audio --state PARTIALLY_IMPLEMENTED --remaining "12 functions left"
gamedb set-module -r SRC --module core.audio --state IMPLEMENTED --verified --verified-by alice
```

- States: `UNIMPLEMENTED`, `PARTIALLY_IMPLEMENTED`, `IMPLEMENTED`.
- `IMPLEMENTED` requires `--verified` in the same call. To retract, lower the state and pass `--unverified` together; `--unverified` alone on an `IMPLEMENTED` module is refused.
- Omitted fields keep their previous values. `--verified` without `--verified-by` records `user`.

## Indexing flags and scope

- `--force` re-parses every file. Use it after changing module rules.
- `--dry-run` parses everything and reports file/function/string counts and decode problems without writing (symbols and edges show `0`). It needs an existing index, and it does not diff against it: it counts every file, not just changed ones.
- `-v N` (or `--verbose=N`) prints progress to stderr every N files. The value is required: `-v0` is a usage error, write `-v 0`.
- `--db PATH` stores the index somewhere other than `<root>/.gamedb/index.sqlite`.
- Directories named `build`, `bin`, `obj`, `dist`, `node_modules`, `.git`, `.svn`, or `.gamedb` are skipped at any depth. If decompiler output lives under one of those, point `-r` inside it.
- Indexed extensions: `c h cpp hpp cc cs java kt kts scala swift go rs dart js jsx mjs cjs ts tsx php txt asm`. C, C++, C#, and Java are the supported targets; the others parse heuristically and miss some declaration shapes.
- `read` slices the *current* file by the line range stored at index time. If a file changed since the last `index`, the body comes back shifted: re-index first.


## Matching and failure details (v1.0)

- Search/strings treat `%` and `_` literally. Search orders shortest name first,
  then path and line; strings order is unspecified.
- `--limit` defaults to 25. Search/strings cap at 60; graph limits each direction
  independently. Use narrow nonempty queries instead of increasing limits first.
- Exit 0 includes empty search/strings/graph (`[]` with `--json`); read with no
  match exits 1. Missing index/runtime errors exit 1; invalid usage exits 2.
- A missing-index error needs index setup. An empty query result needs coverage
  and source inspection, not repeated indexing of an already-current tree.
