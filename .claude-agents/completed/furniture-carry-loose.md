# Furniture carries loose surface items

Philip authorizes pushing/pulling furniture with supported loose goods on top,
using the existing movement adapter and original carrying behavior. Commit room
destinations atomically, animate together, preserve save/load/blocked behavior.
Retain safe refusal for unsupported top goods/unsupported turns. No new pivots.

- [x] Claim/audit source and existing motion kernel/callers; implement.
- [x] Check/lint/fmt/build and bounded disposable previews, no tests added/run.
- [x] Record evidence separately from Philip acceptance; release claim.

Preserve save/settings/.gitignore/codex-handoff.md; M4/migration remain open.
Actual-save SHA25626CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.

Implementation: C aMR_RequestItemToFitFurniture collects relative cell-center
offsets at profile height, removes loose foreground while fitting, and UnFit
restores item numbers. DrawItemOnMovingFurniture draws goods at carried offsets.
Existing/pinned furniture kernels had no loose carry helper; native relocate is
unchanged. New app adapter temporarily removes supported loose tops on copied
layers, calls native relocation, validates empty destination secondary cells and
base ownership, restores goods on copies, then passes existing swept player-path
validation before publishing. Overlapping old/new footprint cells are safe.

New transient carried visuals use exactly the existing base movement progress;
root x/z interpolate together while original surface height/scale remain intact.
Refresh creates destination entities before animation; explicit disjoint query
filters avoid mutable Transform conflicts. Save commits destinations before
presentation, F9 resets transient state as before. Supported goods only on source
Surface/nonstorage profiles. Loaded turns and unsupported tops still refuse;
no rotation-angle/pivot expansion, native actor/storage/switch code unchanged.

Evidence: workspace check, all-target clippy -D warnings, format and capture build
pass; no tests added/run. Temp/bevy-crossing-carry-review synthetic disc preview:
source country table2 (TypeC, native4532) carries diary11008, paper8387,
coconut10247 and coral9499 from cells34/35/50/51 to18/19/34/35. Mid-push paused
screenshot shows all four aligned; F5 stores all four destination items and switch
512->2. Resume/pull returns exact original room/pockets/conditions/switches. F9
from the mid-motion save restores all four at destination with no duplicate/old
cells. Mixed tape7768 plus paper/coconut/coral move together, both actor switch
masks512->2. Occupied destination secondary18 (synthetic orphan apple), unsupported
mushroom top and loaded turn each refuse, preserving complete numeric room,
pockets and conditions. Single-cell orange box12536 carries diary34->18; F5/F9
shortly after motion begins restores both and clears old cells. Screenshots
inspected; fixture/script/log/save evidence stays Temp. Own preview13324 closed
before final builds. Ollama unavailable; C and existing native/caller review used.

Actual save hash unchanged; settings/.gitignore/handoff preserved. No separate
no-disc GUI repeated; shared numeric path unchanged. Every profile, TypeB contact
pivots, exact root/contact behavior and loose turning remain open. Philip has not
accepted this slice yet; M4/migration remain open.

Final default workspace build and release build (-j1, 2m23s) pass. No tests.
