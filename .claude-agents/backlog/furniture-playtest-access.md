# Furniture playtest access

2026-10-17, codex. Philip cannot check storage animation because he has no
wardrobe or spawning control; authorizes adding access or deferring the slice.

Bounded follow-up: F4 while in the player-owned room adds the original blue
cabinet (the source wardrobe interaction type) to an empty pocket, resolving its
numeric ID from the runtime disc profile table. Full pockets/missing disc must
not replace items. Show actionable
instructions and expose the control in the house HUD; use existing inventory,
notice, placement and save paths. No automatic grant or actual-save editing.

- [x] Claim and implement the bounded control.
- [x] Check, warnings-denied lint, format, appropriate builds and disposable
  synthetic-save previews (grant, modal exclusion, full pockets, placement).
  No tests added/run without Philip's request.
- [x] Record evidence and preserve actual save/settings/.gitignore.

Storage animation acceptance remains pending Philip's playtest; M4/migration open.

Implementation: centralized Action::DebugFurniture/F4, handled only with world
focus in the disc-backed player-owned room. Runtime iam_sum_blue_cab01 is found
in furniture_quality, requires its decoded skeleton/animation and audited common
door endpoint, then uses native ftr_idx_2_item_no. PlayerInventory::add -> existing
Inventory::put inserts normal condition into the first empty slot. Normal
furniture notice owns focus until acknowledgment. No automatic save or grant.
The HUD advertises F4; short multiline instructions use the actual pocket label
blue cabinet. Missing disc/villager room/modal focus cannot grant it.

Evidence, 2026-10-17: workspace check, warnings-denied all-target clippy -j1 and
format pass; final capture build passed and its executable was visually checked.
After a connection interruption, normal builds repeatedly failed with Windows
allocation/LLVM out-of-memory errors, including single-job, no-debug/single-codegen,
outside-sandbox and incremental-disabled attempts. No compiler process remains;
target/debug/bevy-crossing.exe is still the successful final capture executable
(2026-10-17 20:23:26 local), with F4 and corrected text. A normal rebuild awaits
available machine memory; no compiler settings were committed. No tests
added/run. Local Ollama was not reloaded after its preceding request timed out
and exhausted paging capacity during storage animation work.

Disposable Temp/bevy-crossing-furniture-access-review GUI: F4 in town did nothing.
Inside own house it filled the empty slot1 with4144 while slots0/2 stayed
10240/9216 and pocket_conds32 stayed unchanged. F4 again while its notice had
focus did not duplicate the cabinet. I selected it by its actual blue cabinet
label; Place in room removed slot1 and saved4144 at main cell34. E from its front
opened the cabinet and storage choices. Full15 pockets refused the grant;
post-notice F5 preserved pockets, condition bits and all room data exactly.
Initial long notice text overflowed the original dialogue box; corrected with
short explicit lines, rebuilt and visually inspected final notice/HUD.
All own previews33880/31016 closed. Screenshots/logs/synthetic saves stay Temp.
Actual save hash unchanged DD89745CCFEDDEAEA619A1C931FEE5233EC533C1860E3B5A910A00B8121E5721;
user settings and unrelated .gitignore preserved. Await Philip's playtest.
Philip requested no codex-handoff.md update; that file is unchanged in this slice.
