# Diary placement from pockets

Philip authorizes the next small slice: existing Place in room action supports
normal-condition diaries, on empty floor or eligible surfaces; blocked placement
retains the pocket item. Reuse existing source-backed adjacent-cell search,
surface metadata and runtime goods renderer. No writing UI/other loose families.

- [x] Claim, audit source and existing callers, implement.
- [x] Check/lint/format/build and bounded disposable save/load checks; no tests.
- [x] Record evidence, release claim; Philip acceptance separate. M4/migration open.

Preserve settings/.gitignore/actual save and codex-handoff.md. Actual-save SHA256
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.

Implementation: existing pockets Place in room accepts normal-condition
diary00..15 as well as furniture, leaving storage/selling/mail branches isolated.
Room transaction validates the original pocket slot/item/condition and disc goods
model before committing; success consumes the item once, refusal changes neither
inventory nor room. Diary placement is immediate and does not use furniture leaf
birth effects or actor slots. Existing goods renderer/Backspace handle the result.

Source: m_tag_ovl.c mTG_room_put_proc, mTG_get_unit_front_player,
mTG_search_put_pos2 and aMR_JudgePlace2ndLayer. Reuses existing logic surface
kernel (also present in pinned Rustimal), original eight-way facing thresholds,
front-surface-first search, then current cell and adjacent0,+1,-1,+2,-2,+3,-3,+4.
Surface requires empty secondary cell and eligible nonstorage profile. Floor
requires empty numeric/actor cells in both visible layers and stays on carpet.
Diagonal placements require at least one clear cardinal route. Full original
wall-ray/slope queries and loose-item throw animation remain outside this bounded
small flat-room adapter; other loose categories/writing UI remain open.

Evidence: workspace check, all-target clippy -D warnings, format and capture
build pass; no tests added/run. Disposable Temp/bevy-crossing-diary-placement:
original pockets renderer shows Place in room on diary. Confirmed surface
placement consumes11008 once, leaves box12536 at main17 and diary11008 at
secondary17. Second diary11009 falls back to main33 without replacing the surface
diary. Other pocket item9216 and condition bits32 remain intact. F5/F9 confirmed
room/pockets unchanged and both original diary variants visibly restored.
Fully occupied carpet plus occupied box surface refuses placement; all numeric
layers, stored wardrobe diary, pockets and conditions remain unchanged after F5.
Screenshots inspected; logs/synthetic saves stay Temp. Initial surface attempt
used a stale entity ID and placed on the floor from spawn; excluded as surface
evidence, repeated after identifying the actual player. Own preview23564 closed
before final builds. Local Ollama endpoint unavailable; source reviewed directly.
No separate no-disc cohort repeated; disc-less room path stays unavailable for
pocket placement and numeric save path unchanged. Actual save/settings/.gitignore
and handoff preserved. Philip acceptance pending; M4/migration remain open.

Final default workspace build and release build (-j1, 2m36s) pass. Actual-save
SHA256 still matches the current-task baseline above. No tests added/run.
