# Repository skill refinement

Philip requests reading and improving the skills in skills/. Scope: gamedb-cli
and gc-iso-reader entrypoints/supporting references, plus the existing GC pointer
if needed for consistent routing. Apply skill-creator guidance: concise main
workflow, progressive reference detail, actual local capabilities, no repeated
indexing or stale workspace assumptions. No game/runtime helper changes.

Preserve existing CLI semantics and useful caveats. Validate frontmatter,
reference paths, documented GC commands against local help and read-only small
lookups. gamedb is not on PATH; do not install/build or claim live validation.
Use rg fallback and label version-specific retained notes. No tests or Cargo
required for documentation-only changes. Preserve save/settings/.gitignore and
handoff; update comms/task evidence, release claim, commit/push authorized work.

## Completed refinements

- Read both canonical skills, original GC helper/parser/help, pointer and
  skill-creator. No executable changes or new helpers were needed.
- gamedb entrypoint now routes direct text to rg, uses one source-root/DB/rules
  setup and refresh only when needed, falls back when CLI is absent, and uses
  Windows/source-aware paths with optional Temp DB. Graph path filtering is
  correctly distinguished from definition selection; duplicate/missing edges,
  unsupported language/.c_inc coverage and stale reads are explicit.
- Module/index flags and SQL schema/caveats retained as linked references,
  labeled original v1.0 notes rather than claiming current installed behavior.
  Queries are bounded; index metadata does not imply runtime or M4 acceptance.
- GC entrypoint now routes raw GameCube vs ISO9660/CISO correctly, separates
  symbol/REL/DOL/archive offset spaces, warns about duplicate exact names, and
  gives specific recovery for missing symbols/members vs damaged images. Header
  and extraction detail moved to a linked reference; preserve runtime-only
  assets, Temp caching, single-quoted dollar symbols and -B imports.
- Existing .claude GC pointer links canonical guidance instead of duplicating
  semantics. No unrelated .claude skills, game roadmap, scripts or settings edited.

## Validation and limits

Built-in checks pass for all three entrypoints' two-field frontmatter, valid
JSON-quoted YAML scalar descriptions, names, fences and all relative links.
git diff --check passes. Local GC --help/sym help/info, exact/pattern-symbol
metadata/bytes (including dollar symbol), known REL hex and FST/RARC/member
lookups pass against the user's disc, stdout only. Exact furniture_quality
actually has multiple map entries, supporting the new ambiguity guidance.

gamedb is absent from PATH; its commands were not executed or installed. The
official skill-creator quick_validate.py was attempted but cannot import yaml
(PyYAML absent). No dependency was installed; direct checks above validate the
restricted flat frontmatter used here, not a general YAML parser. Original v1.0
tool details remain explicitly version-scoped. No tests/Cargo or new scripts.

Final main entrypoints: gamedb1617->703 words and GC633->444 words, a49% reduction
in combined entrypoint context. Advanced references load only on demand.
Claim released; publish authorized documentation only. .gitignore/save preserved.
