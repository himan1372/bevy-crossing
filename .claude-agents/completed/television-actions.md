# Television power, screens and sound

Philip authorizes the next bounded special-furniture slice. Audit lamps/TVs
against original C; implement sum_tv01, sum_tv02 and sum_fruittv01 only. The
audited lamp profiles/draw callbacks have no switch-dependent display behavior;
do not invent a lamp power action. Other TVs and room illumination are separate.

Use native room switch bits, source 60Hz screen cycles and OngenPos4/5/6 through
the existing ambience mixer. Both on_model/off_model are drawn together. Retain
movement, surface support, pickup, storage, music and optional no-disc state.
No save schema change; phase is transient. No tests requested. Follow the
shared workflow and previously read Rust/Bevy/disc/audio/save/QA/build skills.
Only Temp synthetic saves/previews. Run sequential check/clippy/fmt and capture,
default/release builds, update roadmaps/comms, release claim, commit and push.

Implementation and bounded runtime evidence (2026-10-19): logic/television.rs
ports the three US frame tables (54/10/26 ticks at60Hz while on, paused while
off), level sound5/4/6 and off-screen1 selection. furniture::toggle_switch
extends the existing native actor/layer representation; no save schema change.
App builds both original TV model parts, caches each decoded screen once and
changes material texture handles only. Other rooms retain static presentation;
player-room E toggles the audited sets, through existing focus/tool/range guards.
The original common generic branch does not require front contact for TVs.

Lamp audit: sum_bla_lanp/whi_lanp/gre_lanp/hal_lanp callbacks always draw their
extra off parts; other asi/ratan/x/licca, snow/turkey/tent lamp profiles expose
static lists. No switch-dependent lamp drawing was found in this audited set.
No claim of lamp power or room lighting support. Other TV callbacks remain out
of scope. Source US branches used; AUS global-phase animation is not imported.
Pinned Rustimal furniture kernel checked; retain current safe native representation
rather than wholesale ABI integration. New source callbacks do not finish migration.

Temp disc preview29848: all three on/off actions change only the correct switch
bit; full layer/pocket/condition/wallet/music/decor comparisons pass. TV01 F5/F9
restores power; screenshots show off and distinct animated frames for TV01,
wideTV02 and fruitTV. Powered push34->18 moves switch512->2; turn changes
item orientation while keeping power. Surface TV toggles secondary bit only;
pickup clears power, retains box and returns exact item. Two TVs run independently;
untargeted E is unchanged. Pause/resume and quickloads do not grow voices; verified
house exit retires TV loops, diagnostics settle to0 active voices with nonfinite0.
No audible-retail acceptance inferred from counters. Initial fixture assertions
missed existing default decoration serialization and TV02's horizontal footprint;
corrected and repeated. Early exit attempts had insufficient facing settlement;
repeat with south-facing and on-landing position verifies actual exit. A helper
wildcard import overwrote its Temp path only after verified exit; fresh fixture
creation repeated with an explicit path. No source fault inferred from those helpers.

Fresh disc35112: both powered sets and complete numeric state restore from startup;
inventory Place starts off, E starts it, pickup restores exact original pocket item
and empty room with cleared switch. No-disc27292 F5 preserves all numeric room
layers/switches/storage/music/decor and pocket/condition/wallet data with no profiles.
All previews used disposable Temp synthetic saves and absolute EXE, now closed.
Actual save hash remains26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB;
unrelated .gitignore/settings/handoff preserved. Evidence/helpers/assets Temp-only.

Check, all-target clippy -D warnings, fmt and capture47.74s pass, no tests added
or run. First compile caught two API mismatches corrected before gates; a transient
missing-Bevy-crate check then passed on retry with no dependency/config changes.
Roadmap metadata/unique IDs/local anchors pass:77 items44 done6 doing27 planned,
not game completion. No new browser-layout claim. Standard/release builds follow.
Philip visual/audio acceptance remains pending, M4 and migration stay open.
Final default workspace build passes43.29s; release build running. Static roadmap fallback counts/bar updated alongside dynamic counts. No gameplay-code edits after the passing check/clippy/fmt/capture gates.
Final source lifecycle check: f_furniture.c aFTR_CAN_PLAY_SE excludes birth/birth-wait/bye/death, but permits pushes and turns. New TVs start off and retired pickups lose RoomPiece immediately, matching the bounded audio gating without silencing powered movement.
Final release workspace build passes3m08s. All required gates complete; no tests added/run. No owned preview or Cargo process remains. Publish authorized source/docs and release task ownership. Philip playtest remains pending.
