# Furniture placement and pickup presentation

2026-10-17, codex. Philip accepts the movement slice ("works, next").

## Bounded scope

Audit and connect original furniture birth/reservation/bye timing, source scale
curves and the original player pickup clip to accepted Place in room/Backspace
transactions. Investigate the retail leaf/effect renderer before selecting a
runtime disc-backed presentation. Keep inventory/storage/layout transactions
atomic, supported surfaces intact, P/F5/F9/restart safe and no-disc playable.

Use existing logic furniture kernels and runtime Gfx/cKF decoders. Never commit
disc data. Preserve actual saves/settings and unrelated .gitignore. One task
claim and one Cargo process at a time. No tests added/run without Philip request;
check, warnings-denied all-target lint, format and builds as appropriate.

## Source starting points

ac_my_room.c FurnitureCt; ac_my_room_move.c_inc ReserveFurniture,
SetReserveFurniture, BirthWait/FtrBirth/FtrBye, Furniture2ItemBag;
m_player_main_pickup_furniture.c_inc and its animation/leaf callers;
retail kagu_happa/dust effects and existing icon/Gfx loading where applicable.

- [x] Audit source, callers and existing translations.
- [x] Implement bounded presentation with explicit remaining fidelity limits.
- [x] Record validation evidence separately from playtest acceptance.
- [x] Philip playtest acceptance of this bounded slice (2026-10-17: "wokrs good whats next").

M4/migration and long-piece contact-dependent pivot fidelity remain open.

## Implementation, 2026-10-17

app/src/player_room/presentation.rs owns temporary birth/pickup visuals separately
from numeric inventory/layer transactions. Native FurnitureBirth in
logic/src/furniture.rs translates the source signed-angle scale oscillation:
initial amplitude0.5/step6000, amplitude*0.4 and step+1500 at negative-to-positive
crossings, stop below0.02. Existing RSV_FRAME_COUNT supplies the46-tick delay.
The existing pockets renderer exposes when its close slide ends, matching the
C submenu-wait gate. New furniture stays invisible during the wait and then
plays the damped pop. The body remains idle during placement, as in this flow.

Pickup still validates the whole furniture-plus-contents transfer before any
mutation. A temporary render root shrinks by source1/12 per60Hz tick and vanishes
below1/4, independently of the already committed foreground/collision change.
The original pickup1 player clip runs at0.5 frames/tick. Original room icon
display lists come from runtime aMR_icon_display_data (all six categories);
the leaf flight uses ef_kagu_happa interpolation/hop/roll. Pickup draws the leaf
until it shrinks into the left hand over clip frames10..20. The hand uses the
actual rendered local joint transforms, source callback16 and VecX(1100), rather
than a guessed world offset. Source cleanup/appearance and frame10/20 item/rustle
sound IDs are emitted once. No new content/art files are included.

TypeB scale is centred using the source half-unit model offset. Full footprint
collision is retained during birth rather than implementing retail0.6->1 collision
growth; exclusive furniture focus prevents competing actions. P freezes the
presentation; F5/pause Save snapshots the committed result; F9 or scene departure
discards temporary state/leaf/retiring meshes. Initialization/restoration shows
already-saved furniture at full scale. No-disc saves retain numeric layers.
Pickup and new placement now clear the old/new primary-cell switch bit, so an
empty/reused cell cannot inherit the removed furniture's switch on reload.

## Evidence and limits

Workspace check, all-target clippy with warnings denied, format and capture build
pass; final default build completion recorded in comms. No tests added/run.
Reviewed current original room ctor/draw/reservation/birth/bye/pickup code,
player pickup state/draw callbacks, ef_kagu_happa, ef_effect_lib and ef_dust.
Compared the pinned canonical furniture module: it has reservation metadata,
but no equivalent birth/pickup presentation kernel to copy. Icon enum ranges
were mechanically counted from the original m_name_table X lists and reviewed
against the existing haniwa/fossil ranges. A bounded local Ollama calculation
request timed out and supplied no evidence.

Tool-driven disposable GUI checks in Temp/bevy-crossing-furniture-birth-review:
- Wardrobe4096 placed at cell34: leaf flight, stretched pop and final full shape
  inspected. Pocket item removed once. Backspace showed shrinking geometry and
  pickup clip; F5 during pickup saved wardrobe back in pockets and cell34 empty.
- Full pockets visibly refused pickup without starting its animation. A retry
  attempted while the notice still had focus was excluded from evidence.
- Filled wardrobe pickup returned4096/10240/8960/9216 together. Pause/resume and
  pause-menu Save during the clip preserved that result; all four owner cells
  and switch512 cleared. Leaf at the source cell while paused then moved toward
  the computed hand (position2999.5276,0.0748,2999.7676 at scale0.59385 observed).
  Leaf entity count was0 after completion.
- Pause/Save during placement and F9 during its presentation restored one full
  wardrobe, pocket4096 absent. F9 during pickup cleared the temporary leaf.
- Tape7768 placed on box12536 in main/secondary cell34 at original surface height;
  animated pickup recovered the tape and retained the box. Screenshot inspected.
  Fresh process loaded one box with tape in pockets and no repeated birth demo.
- Explicit missing-disc startup F5/F9/F5 preserved home_furniture exactly. This
  pass checked town fallback, not placeholder-house entry again.

Screenshots/logs/synthetic saves remain in Temp; all own preview processes closed.
Actual save SHA256 stayed13A8755B8D093084928EF214197945D909D55FE31F4097BD63AACBB45F06A3C4.
Settings and unrelated .gitignore unchanged.

Exact dust effects remain open: retail ef_dust uses a two-texture blend, while
the current N64 material exposes one texture. No replacement cloud art was
invented. Other per-type constructors, collision-edge fidelity/growth, loose
starter diary, storage opening/closing, full source scripts and every model's
visual fidelity remain separate. Birth math uses Rust sine, not a bit-exact
retail trig-table kernel. Philip accepts this bounded placement/pickup presentation
slice (2026-10-17: "wokrs good whats next"); the listed fidelity gaps remain open.
Next proposal: furniture-storage-animation.md. No storage animation implementation
or new automated evidence is implied by this acceptance update.
