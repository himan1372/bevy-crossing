---
name: gamedb-cli
description: "Query a decompiled source tree with gamedb for function bodies, candidate callers/callees, module maps, or SQL aggregates. Use for repeated structured source navigation; use rg for direct text, macros, or declarations the parser misses."
---

# gamedb source navigation

Use the SQLite index to narrow a question, then confirm the relevant source and
call sites. A call graph is a set of candidates, not proof of runtime behavior.
The intended targets are C/C++, C# and Java; other languages parse heuristically.

## Choose the fastest route

- Start with `rg` for an exact string, macro, filename, or one-off lookup.
- Use gamedb for repeated function reads, call tracing, or indexed summaries.
- Check ACGC `.c_inc` coverage explicitly: that suffix is absent from the
  retained v1.0 extension list. Use `rg` for omitted include bodies and callers.
- Locate the executable with `Get-Command gamedb` on PowerShell or
  `command -v gamedb` on POSIX. Use a known executable path if already configured.
  If unavailable, say so and continue with `rg` and bounded file reads. Do not
  install/build it or block ordinary source investigation just to use this skill.
- Check `gamedb --help` and relevant subcommand help once for an unfamiliar
  installation. Historical Linux workspace paths are not this project's setup.

## One setup per source root

1. Select the source root from the task or handoff and confirm it exists. For
   original C in this project, the known root is
   `C:/Users/phili/game c/shmanimal crossing/ACGC-PC-Port/src`; the Bevy checkout
   is a different source tree. Quote paths containing spaces.
2. Choose one `(root, database, rules)` combination and retain it throughout the
   investigation. Default database: `ROOT/.gamedb/index.sqlite`. Indexing writes
   files; for a read-only/external source root, use `--db` in a writable Temp
   location rather than writing into that tree. Use a distinct DB per root.
3. Refresh the index once at the start of an indexed investigation, unless it is
   already known current. Refresh again only after source changes or evidence
   of stale line ranges. Do not index before every search/read/graph request.
4. Check `stats` once for total coverage and read indexing warnings. Incremental
   `index` counts describe this run, not the whole database. Small or empty totals
   may mean a wrong root, skipped directories, or unsupported syntax; inspect the
   source before concluding a symbol is absent. Fall back to `rg` when needed.

PowerShell example, from this repo, when gamedb is installed:

```powershell
$sourceRoot = 'C:/Users/phili/game c/shmanimal crossing/ACGC-PC-Port/src'
$indexPath = Join-Path $env:TEMP 'acgc-source.gamedb.sqlite'
gamedb index -r $sourceRoot --db $indexPath
gamedb stats -r $sourceRoot --db $indexPath
gamedb search -r $sourceRoot --db $indexPath 'MiniDisk' --limit 10
gamedb read -r $sourceRoot --db $indexPath 'aMR_MiniDiskCommonCt' --path 'ac_my_room.c'
gamedb graph -r $sourceRoot --db $indexPath 'aMR_MiniDiskCommonCt' --direction callers --limit 10
```

## Find, read, trace

- `search` finds function-name substrings; `strings` finds string literals
  (historically four or more characters). Start with a narrow query and small
  limit. Add `--json` for programmatic processing instead of parsing text rows.
- `read` and `graph` require the exact, case-sensitive name returned by search.
  Search/strings are case-insensitive. Avoid empty searches, which match all.
- If `read` reports multiple definitions, narrow with `--path SUBSTR`. Read each
  relevant definition instead of accepting whichever appears first.
- Graph edges resolve by name, without type/scope information. Duplicate names
  fan out; declaration lines can appear as self-edges. Check the reported source
  line and exclude declaration anchors. Missing edges do not prove no callers,
  especially for callbacks/function pointers and macro-heavy C.
- For `graph`, `--path` filters the **call-site file**, not a unique target
  definition. The parenthesized path names the other definition; `called at`
  names the actual call site. Confirm it with a bounded source read.
- Empty search/strings/graph output can exit successfully. Inspect the result,
  not just the exit code. If a body is shifted, refresh once and re-read.
- Report the relevant file/line and what the source establishes. Keep inferred
  relationships distinct from source-confirmed behavior.

## Read advanced details only when needed

- [Indexing and modules](references/index-and-modules.md): skip rules, force/dry
  run behavior, taxonomy changes, and explicit module-progress updates.
- [SQL](references/sql.md): schema, symbol-ID joins, numeric output quirks, and
  bounded aggregate examples. Prefer normal subcommands for individual lookups.

Module status updates are for an authorized tracking task and require actual
supporting evidence. They do not establish Philip's acceptance or close M4.
