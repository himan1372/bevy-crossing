Retained gamedb v1.0 schema/behavior; confirm against the installed version
or its `src/db.rs` before using a changed schema. Read this only for aggregates,
joins, or filters unavailable through the normal subcommands.

# SQL reference

`gamedb sql -r SRC --sql "QUERY" [--param VALUE]...` runs one read-only statement (writes fail with `attempt to write a readonly database`). `?` placeholders bind `--param` values in order, always as text.

Use `--json` with `sql`: the plain-text output prints integer columns as empty strings. Without `--json`, `CAST(x AS TEXT)` the numbers.

Schema (authoritative copy: `SCHEMA` in `src/db.rs`):

| Table | Columns |
|---|---|
| `files` | `id, path, mtime, size, edges_mtime, module` |
| `functions` | `id, file_id, name, params, sig, start_line, end_line, sym_id` |
| `symbols` | `id, file_id, kind, name, line` — `kind` is `namespace`, `type`, `method`, `property`, or `field` |
| `strings` | `id, file_id, line, text` |
| `edges` | `src_id, dst_id, line, hits` |
| `module_status` | `module_id, system, state, verified, verified_by, verified_at, remaining` |

**`edges.src_id`/`dst_id` are `symbols.id`, not `functions.id`.** Reach a function through `functions.sym_id`. `dst_id` can also be a `type` symbol. `edges.line` is in the caller's file.

```sql
-- symbols with the most indexed incoming edges
SELECT d.name, d.kind, p.path, COUNT(*) AS callers
FROM edges e JOIN symbols d ON d.id = e.dst_id JOIN files p ON p.id = d.file_id
GROUP BY e.dst_id ORDER BY callers DESC LIMIT 20;

-- no resolved incoming edges: review candidates, not proof of dead code
SELECT f.name, p.path FROM functions f JOIN files p ON p.id = f.file_id
WHERE NOT EXISTS (SELECT 1 FROM edges e WHERE e.dst_id = f.sym_id AND e.src_id <> f.sym_id) LIMIT 20;

-- every function in one module
SELECT p.path, f.name, f.start_line FROM functions f JOIN files p ON p.id = f.file_id
WHERE p.module = ? ORDER BY 1, 3 LIMIT 20;
```

Prefer the subcommands for single lookups. Reach for `sql` when you need aggregates, joins across tables, or a filter the commands lack (by module, by symbol kind, by line range).
