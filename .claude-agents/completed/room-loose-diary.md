# Room loose diary rendering

Philip authorizes the next bounded loose-item slice, with remaining usage budget
kept in mind. Start with source starter diary: render saved numeric diaries on
floor/surfaces, reuse original goods display lists, exclude storage contents.
Do not insert defaults into existing/explicit empty saves. Other loose categories,
new placement controls and full diary writing UI remain outside this slice.
Existing Backspace also recovers a visible diary before its supporting furniture,
so restoring the default diary cannot permanently block pickup of its box.

- [x] Source/caller audit, implementation and required check/lint/fmt/build.
- [x] Record bounded evidence separately from Philip acceptance; release claim.

Actual save baseline SHA256:
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.
Preserve settings/.gitignore/save and codex-handoff.md. No tests added/run.
M4/migration remain open.

Implementation: mHm_SetDefaultPlayerRoomData sets ITM_DIARY00 (0x2B00) on
secondary(1,1), above the orange box. Added this only to absent-field disc defaults;
existing and explicit-empty numeric rooms are untouched. Source ac_shop_goods.c
aSG_DrawGoodsPicture mode1 draws both foreground layers; its diary00..15 entries
use identical mode0/mode1 lists, so reuse existing spawn_goods without changing
shop behavior. Cells use original cell centers, scale0.01/40 and supporting
furniture height/40. The original SG_angle_y initializes to0; arbitrary loose-item
rotation and moving loose surfaces remain deferred. No new imported assets.

New player_room/loose.rs rebuilds visible diaries on room dirtiness, omits storage
contents and layers2/3, and adds no collision. Backspace reuses existing forward
pickup corridor and layer0/1 ranges, returns one item only after inventory accepts
it, and otherwise preserves the room. Pickup is currently immediate with confirm
sound; exact original loose-item pickup presentation and placement remain open.

Evidence: workspace check, all-target warnings-denied clippy, format and capture
build pass; no tests added/run. Disposable Temp/bevy-crossing-diary-review uses
synthetic saves and explicit --disc. Default room screenshot visibly shows diary
on box. Backspace yielded one diary in pockets, secondary17=0, box17=12536.
Combined saved fixture visibly shows floor diary and box diary, while diary in
wardrobe contents is hidden. Full pockets display refusal; F5 preserves all four
numeric layers and pockets exactly, including wardrobe content. F9 explicit-empty
fixture renders an empty room and F5 has0 nonzero saved cells. All screenshots
inspected, logs/saves stay Temp. Two intermediate PowerShell array-preparation
attempts were replaced by a proper Python flat-layer fixture before F9; not
counted as app evidence. Own preview35404 closed before final builds.

Pinned Rustimal furniture module had no matching diary/goods helper; existing
shop renderer reused. Local Ollama command/endpoint unavailable; direct source
review used. Actual save SHA256 unchanged from current-turn baseline above.
Settings/.gitignore/handoff untouched. No separate no-disc GUI cohort repeated;
new renderer exits without a disc and existing numeric preservation path remains.
Philip acceptance pending; only diary family covered, other loose goods deferred.

Final default workspace build and release build (-j1, 2m21s) pass. No new tests.
