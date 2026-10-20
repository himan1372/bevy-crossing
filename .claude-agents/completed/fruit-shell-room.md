# Fruit and shells in player room

Philip authorizes fruit and shells together: audit original indoor goods models,
extend existing loose placement/pickup/save path, preserve storage rules and
blocked/full-pocket atomicity. No other foods/actions or animation expansion.

- [x] Claim/audit and implement.
- [x] Required check/lint/fmt/build plus bounded disposable GUI; no tests.
- [x] Record evidence separately from Philip acceptance; release claim.

Preserve save/settings/.gitignore/codex-handoff.md; M4/migration remain open.
Current actual-save SHA256:
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.

Implementation: centralized supports adds six fruit0x2800..2804/2807 and eight
shells0x2514..251B (coral included), excluding neighboring mushroom/candy codes.
Original m_name_table.h IDs and ac_shop_goods_data.c_inc indoor/shop display-list
pairs audited: same mode0/1 models, existing renderer reusable without new assets.
Reuses existing normal-condition pocket action, surface kernel, local placement
search, atomic pickup/save/storage exclusion. Mail/selling/storage rules untouched.

Evidence: workspace check, all-target clippy -D warnings, format and capture build
pass; no tests added/run. Disposable Temp/bevy-crossing-fruit-shell-review:
coconut10247 placed on box12536 at secondary17, coral9499 fell back to main33;
Backspace restored coconut once, secondary17 cleared, box retained. F5/F9 matched
room/pockets/condition32; coral visibly restored and stored shell9492 stayed hidden
and numerically intact. Gallery of six fruit/eight shells inspected from two
positions to uncover models behind the avatar. Full-pocket pickup and blocked
placement notices appeared; subsequent F5 matched numeric room/pockets/conditions
exactly. Own preview8876 closed before final builds. Evidence stays Temp.

Ollama endpoint unavailable; direct C/existing shared path review used. No separate
no-disc cohort repeated; shared no-disc numeric path unchanged. Immediate loose
presentation, exact throw/wall-ray/slope fidelity and other item families remain
open. Philip acceptance pending; M4/migration remain open. Actual save hash above
unchanged; settings/.gitignore/handoff preserved.

Final default workspace build and release build (-j1, 2m30s) pass. No tests.
