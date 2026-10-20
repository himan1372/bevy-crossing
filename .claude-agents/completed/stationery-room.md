# Stationery in player room

Philip authorizes the next bounded loose household item slice: stationery model,
pocket placement on floor/surface, pickup, atomic blocked/full refusal and saved
numeric restoration. Reuse the diary path; preserve writing/mail/storage/selling
flows and normal-condition gate. No extra item families or new animation scope.

- [x] Claim/source audit and implement.
- [x] Check/lint/fmt/build and disposable GUI evidence; no tests added/run.
- [x] Record evidence separately from Philip acceptance and release claim.

Actual-save baseline SHA256:
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.
Preserve settings/.gitignore/save/codex-handoff.md. M4/migration stay open.

Implementation: centralized loose::supports now covers stationery0x2000..0x20FF
and diaries. Original m_name_table.h defines64 designs times4 quantities across
this range. Full encoded IDs pass unchanged through existing placement, pickup,
storage and save paths. Original ac_shop_goods_data.c_inc paper entry uses shared
obj_paper_mat_model/obj_paper_gfx_model for mode0/1. Reuses mTG_room_put_proc ITEM1
surface/floor behavior and existing surface kernel/renderer; no new assets.
Pockets home-action/normal-condition guard, mail/sell/store branches are unchanged.
No writing UI existed in item_actions; none added or claimed here.

Evidence: workspace check, warnings-denied all-target clippy, fmt and capture
build pass; no tests added/run. Disposable Temp/bevy-crossing-stationery-review:
8387 (orange paper, four sheets) offered Place in room, placed at secondary17 on
box12536;8192 (one sheet) fell back to main33 without overwriting surface paper.
Backspace restored8387 exactly once, cleared secondary17 and kept box. Stored
paper8340 stayed hidden/numerically intact. F5/F9 preserved room/pockets/condition
bits32; restored floor model visible. Full pockets refused pickup, fully blocked
placement refused, and subsequent F5 matched all numeric layers/pockets/conditions
including stored paper. Screenshots inspected; synthetic saves/logs stay Temp.
Own preview35292 closed before builds. Ollama endpoint unavailable; reviewed C,
existing inventory.put and pinned Rustimal surface helper directly.

No separate no-disc cohort or every stationery design repeated. Shared existing
no-disc numeric path preserved. Existing immediate loose pickup/placement and
wall-ray/slope/throw fidelity limits remain; no broader family expansion.
Philip acceptance pending. Save baseline above unchanged; settings/.gitignore
and handoff preserved. M4/migration open.

Final default workspace build and release build (-j1, 2m26s) pass. No tests.
