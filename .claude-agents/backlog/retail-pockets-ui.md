# Retail pockets UI

2026-10-16, codex. First implementation; final Philip comparison pending.

First remaining retail UI task in M4; existing inventory behavior is implemented.

## Scope

Trace original inventory layout, runtime icon/texture assets, selection and
available action windows. Reuse disc font/window rendering and translated
inventory logic; retain existing inventory capacity, tool use and input ownership.

Relevant files: app/src/pockets.rs, items.rs, input.rs, msg_window.rs, game_disc.rs;
logic inventory and original inventory/window code identified during tracing.

## Acceptance and boundaries

Philip compares the 15-slot pocket layout, item icons and selection/action
windows with retail. Existing item actions and closing/back navigation remain
usable; opening/closing must not leak input to gameplay.

Load original assets at runtime from Philip's own disc; never commit extracted
bytes. Keep the no-disc placeholder usable. Use source and existing translations,
not guessed layouts or gameplay rules. Claim before implementation. Run suitable
compile/lint/format/build checks for code changes; no tests unless Philip requests
them. Record implementation evidence separately from Philip's actual acceptance.
M4 remains open; title and rename remain deferred.

## Implementation

- [x] Runtime inv_mwin frame, 5x3 item slots and 2x5 letter placeholders.
- [x] Original font, live bells, named selection and retail category icons.
- [x] Source player walk and hand point/catch clips; menu slide and leftward scroll.
- [x] Arrow/WASD slot selection; Enter/E opens actions and confirms; I/Tab closes.
- [x] Grab/swap keeps item IDs and packed condition bits together; Esc/Backspace cancels.
- [x] Hold for normal net/rod reuses existing equipped-tool behavior; gifts offer Grab only.
- [x] Keep the no-disc text screen and existing inventory/save schema.
- [ ] Philip: final moving UI/retail comparison acceptance.

app/src/pockets_disc.rs owns rendering/animation. pockets.rs retains state and
fallback listing and adds selection/actions. msg_window.rs shares existing font,
mesh and choice-window primitives. input.rs exposes the existing focus update so
pockets acquires focus in PreUpdate, before any world interaction. The new module
is registered by main.rs; character.rs shares current-player texture segments.
No original bytes or extracted textures are embedded in the implementation.

Original source: m_inventory_ovl.c::mIV_set_normal_frame_dl/set_item/set_money,
mIV_pl_load_player_anime; m_submenu_ovl.c::mSM_draw_item/set_dl_item/mSM_move_menu;
m_tag_ovl.c grid positions and Grab; m_hand_ovl.c pointing/catch clips/rotation;
m_private.c default background ITM_CLOTH226. The upstream
[inventory draw setup](https://raw.githubusercontent.com/ACreTeam/ac-decomp/master/src/game/m_inventory_ovl.c)
confirms the same lists and runtime cloth binding as Philip's local port.
The upstream raw model page could not be opened; model list details were read
from local C source and graphics from the disc, not assumed from that page.

Inventory-specific run_inventory_frame exposes tile1 coverage without changing
the global world-model TEXEL1 fallback. Runtime CPU composition combines that
coverage with the default cloth pattern in the first pass. The correction below
replaces that CPU composition with shared GPU sampling. UI I4 intensity supplies
coverage, avoiding opaque white strips. Explicit
per-part depths preserve retail list order for transparent layers on reopening.
The player walks at0.5 source frames/tick, hand points at0.275/catches at0.45,
and scroll advances0.3535 per60Hz tick with quarter-texel offsets. Philip requests
horizontal leftward motion, so the original simultaneous Y increment is omitted.
Animation updates reuse mesh/image buffers; preview walking emits no town steps.

## Evidence and remaining scope

Philip initially reports rough visuals, then says the corrected static layout
"looks much better" and requests motion and a fully leftward sliding background.
That is provisional layout feedback, not acceptance of the final animation or
all retail UI. Live disposable-town screenshots show all15 filled slots with
fruit/gift, net/rod, fish/bug, furniture, clothing, flooring/wallpaper, stationery,
fossil, plant and gyroid icons. Separate frames show player/hand movement and
background scrolling. Menu reopen retains frame labels; Backspace returns from
actions, gift has no Hold, and pause/resume returns to pockets.

Full-inventory Grab/swap moved net/gift IDs between slots0/1, preserved15 items,
12345 bells and moved the gift condition from packed value1 to4. Only the Temp
save was written to inspect this result. Movement while pockets owns focus stays
at the same world position. Local Qwen reviewed the bounded swap/condition code;
its conclusion was checked against the original hand swap and inventory helpers.

Port adaptations/follow-ups: five-row mail integration, identity fields and full
original tag layout/popup placement
need further retail comparison. The existing choice window supplies action
windows. Only Grab/swap and already-supported normal net/rod equipment are
exposed; eating, dropping/placing, clothing changes, gift opening, letters,
collection pages, item effects and broader tool actions require their gameplay
chains. No fictional identity text is substituted into the empty name bars.
The current HUD stays until retail-hud-ui; title/rename remain deferred. This
task stays open for comparison and named fidelity work, without claiming M4 done.

Final manual review: source hand-catch animation closes around a carried gift;
its source slot is visually empty until swap/cancel, without changing storage
before confirmation. Esc restored the original item layout and condition bits.
Selected slot15 (cherry) successfully, then closed; player position moved from
(0,4,-20) to approximately(0,4,-21.483) after W, confirming movement resumes.
F9 with a Temp-only empty inventory showed15 empty slots and unchanged bells.
Explicit missing-disc startup kept the plain pockets screen usable: open,
select slot2, close. Screenshots and decoded-mask diagnostics remain in Temp;
no extracted bytes or images are committed. The final text-only cleanup removes
an instruction outside the disc frame; the fallback screen retains instructions.

Workspace check, all-target clippy with warnings denied and formatting pass;
capture build/live review and normal workspace build recorded in comms.txt.
No automated tests added or run. User save SHA256 remains unchanged; user
settings were not edited. Claim released at handoff; Philip final acceptance pending.

## Pockets motion and alignment correction, 2026-10-16

Philip reports only the upper-left background moved in sync, rough player/hand
motion, and present/shirt/fishing-pole/plant icons offset from their circles.
These are unresolved feedback on the first pass, not final acceptance.

- All cloth parts now share one panel-local GPU texture phase. Coverage retains
  each part's own UVs; the entire cloth scrolls left continuously, with no CPU
  image replacement or tile-size-dependent phase. Frame and circle masks stay fixed.
- Player and hand now render into separate transparent perspective views using
  original mSM_change_view/mIV_set_player/mHD camera, root scale and rotation.
  Fixed source menu lighting replaces town lighting/fog only for these materials.
  Source walk/point speeds remain unchanged. Ordinary pocket Grab keeps the
  player walking; the source player catch clip belongs to the equipment table.
- Hand closes, holds and reverses open at source speed; navigation/confirm waits
  during close/release. Carried icon/shadow follows the eased hand, with source
  shadow offset. Cancel stays available. Full original hand offset/morph/tag
  behavior remains a fidelity follow-up.
- Disc inv_mwin_v vertices for all15 circles confirm centers on columns
  -77,-53,-29,-5,19 and rows -10,-34,-58. Removed mSM's separate item-view X
  correction from the shared orthographic frame/icon layout. Every icon, pointer
  target and carried-item destination now uses those same centers.

Manual disposable-town screenshots reviewed: all15 filled slots, unobstructed
gift/shirt/rod and plant, shared pattern at separate times, player/hand movement,
gift carry/cancel, sapling/gyroid swap followed by release and menu reopen.
Screenshots stay in Temp; no user save/settings or disc bytes are committed.
Check/clippy/fmt, capture and default builds pass; no tests added or run.
Philip comparison acceptance remains pending and M4 remains open.
