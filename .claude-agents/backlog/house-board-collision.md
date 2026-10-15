# Player home and board collision

Status: implemented; Philip playtest acceptance pending.

Philip reports player homes, bulletin board and map boards lack collision.

- [x] All four player homes use footprints measured from the same size-one disc model/idle cKF pose as rendering. Apply each actor's actual shift and rotation; the old 1x1 actor-origin box was too small and misplaced.
- [x] Player-house entry panel uses the posed door centroid plus the existing outward offset, keeping entry/return reachable.
- [x] Bulletin 0007/000B, map 000C/000D and music 000E/000F occupied units block movement. Original `src/game/m_name_table.c::obj_hight_table_item0_nogrow` supplies raised wood height records for both halves (4 bulletin, 7 map/music). `src/bg_item/bg_item_data.c_inc` draws only the first half. Both units must block; deriving collision solely from displayed entities misses half the board.
- [x] Live FieldItems changes synchronize before player movement, including restored save edits. `can_stand` now includes retail obstacles so NPC placement/navigation targets also avoid them. Player and NPC steps share Town::can_step.
- [x] Existing escape behavior permits movement out from embedded loaded positions. No-disc placeholder path remains independent of retail footprints.
- [x] Workspace cargo check, clippy --all-targets -- -D warnings, fmt and capture/default build passed. No automated tests or disc probes added/run.
- [ ] Philip playtest acceptance.

Implementation files: app/src/town.rs, disc_town.rs, field_items.rs, house.rs.

## Agent runtime evidence (2026-10-15)

Launched capture build from existing synthetic Temp/bevy-crossing-house-review town with an absolute read-only disc path. No save inputs sent. All four player homes blocked sustained forward walking from three units south of their origins: final z about -25.99 for the northern pair (origin -28) and -18.99 for southern pair (origin -21). Checked rear approach to home 0 too.

Bulletin left/right halves stopped at z about -24 from start -23. Northern map halves stopped about -40 from -39; southern map halves about 7.01 from 8. Music board also blocked. Clear space beside the bulletin allowed passage. Artificially positioned player inside bulletin/home could walk out. E entered the player's house from the reachable diagonal approach, then E exited and returned outside. Direction reversal follows existing eased turning and can arc sideways; captures/positions taken after held input/release completed.

Inspected collision-house.png and final collision-board.png in the Temp folder. These are agent evidence, not Philip's acceptance or full retail collision-solver parity. Home footprints reuse the public-building model-bound technique; boards use occupied cell boxes rather than porting the entire original five-height terrain solver. No screenshots or disc bytes committed.

Preview closed gracefully. Default game rebuilt. Real save SHA256 remained
0CD03EE33FE4FCFAAFF893271D79A0C6C5885C34B5C9A93D1C375350AA4D49A0.
Unrelated .gitignore/savegame.json preserved.

## Philip acceptance route

Walk into each player home from front, back and sides; walk around the corners and use E to enter/exit your home. Walk into both sides of the bulletin and both map boards, then around their ends. Confirm collision feels appropriate and doors/path remain usable. No-disc runtime and save/load collision refresh were reviewed in code, not separately exercised in this preview. Full render-look/M4 remain open.
