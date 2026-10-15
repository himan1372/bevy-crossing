# special-building-look — Philip's side-by-sides

2026-10-15, codex. Implemented and visually inspected; Philip acceptance pending.
Philip requested fixes using C:/Users/phili/game c/plat-test-pics (our game left,
retail right). This concrete render-look feedback takes priority over net actions.

- [x] Read all four comparison PNGs: Able Sisters, police box, Cranny, post office.
- [x] Audit palettes/model lists/season selection against C and runtime disc table.
- [x] Correct demonstrated palettes, winter post-office part and Cranny door state.
- [x] Match closed-shop window-light policy; preserve runtime-only content loading.
- [x] Workspace check/clippy/fmt and capture-enabled build.
- [x] In-game visual review, with screenshots outside the repository.
- [ ] Philip: compare the rebuilt game with his retail screenshots and accept.
- [x] Final default-feature build: cargo build --workspace passed.

## Root causes and implementation

The expected special-building geometry was already selected. Most pictured color
differences came from BUILDINGS palette indices missing the 25 NPC-house entries
at the start of the structure palette enum. Corrected:
Cranny 12 -> 37 (obj_shop1_pal), post office 16 -> 41 (obj_s_post_office_pal),
police box 34 -> 59 (obj_police_box_pal), museum 66 -> 91 (obj_s_museum_pal),
Able Sisters 68 -> 93 (obj_s_tailor_pal). These are aSTR_PAL enum values,
not structure-type indices. Both seasonal tables are still resolved at runtime.

Winter post office uses obj_w_yubinkyoku_t3_model; the summer symbol is
obj_s_yubinkyoku_t3_1_model. Include the seasonal spelling instead of omitting it.

Cranny was baked at closed cKF frame 1 at every hour. Its door now has its own
entity, joint-local mesh and sampled disc cKF matrix: frame 16 while normally
open (09:00–22:00), frame 1 otherwise, moving at original 0.5 frames/60 Hz tick
when clock state changes. Failed rig lookup retains the prior static door.
logic::shop::ShopTier::normally_open holds normal tier hours from m_shop.c.
This is visual state only: existing shop access/transactions are unchanged.
Lottery/event/chore/remodel overrides and exterior upgrade models remain
separate gameplay/rendering follow-ups; not implemented by this fix.

BuildingLights now groups normal, tailor and Cranny glass/glow independently.
Able Sisters' night lighting also requires its open interval (closed 02:00–07:00).
Cranny's night light requires normal business hours, or an already-present player
inside (source PL_INTO_WAIT adaptation). Police/post/museum retain normal night
policy. Constructors initialize at the current target; transitions retain the
existing light chase. No global recoloring or replacement art.

Files: app/src/disc_town.rs, app/src/environment.rs, logic/src/shop.rs.

## Original sources

Under C:/Users/phili/game c/shmanimal crossing/ACGC-PC-Port:
- include/ac_structure.h: enum structure_palette (25 NPC + 12 player palettes first).
- src/data/model/structure/structure_pal.c: summer/winter pointer tables.
- src/actor/ac_needlework_shop_draw.c_inc, ac_post_office_draw.c_inc,
  ac_police_box_draw.c_inc, ac_shop_draw.c_inc: selected palettes and light inputs.
- src/actor/ac_needlework_shop_move.c_inc: aNW_check_opend / aNW_ctrl_light.
- src/actor/ac_shop_move.c_inc: aSHOP_setup_action frame 1/16 and 0.5 speed;
  aSHOP_ctrl_light requires night plus open/present-player state.
- src/game/m_shop.c: normal opening {9,7,9,9}, closing {22,23,22,22}.
- src/data/model/obj_s_shop1.c: door joint and cKF animation.
- disc/data/foresta.sym: winter post-office t3 symbol.
Disc inspection: gc_iso_reader.py sym rom structure_pal_adrs_nowinter --exact -x 384
confirmed the resolved palette names; no extracted bytes saved in the repository.

## Evidence and limits

Workspace check, all-target clippy with -D warnings, fmt and diff whitespace pass.
cargo build --workspace --features bevy-crossing/capture passes.
No automated tests or test probes added or run.

Launched a temporary capture build from the workspace and reviewed all five
buildings at 09:38–09:39, then Able Sisters/Cranny/police at 04:40 via F6.
Observed correct green/red Able Sisters, green POST OFFICE sign, white/red Cranny
sign, open daytime and closed nighttime Cranny door, normal police blue-grey/red
lamp, cream museum, and dark Able Sisters glass at 04:40. Screenshots are in
Windows Temp: building-{tailor,cranny,post,police,museum}-day-full.png and
building-{tailor,cranny,police}-4am.png. Not committed.

Initial BRP launcher rebuilt default features and used app/ as cwd, so it opened
a no-disc placeholder instance without BRP. Closed that process; launched the
capture binary directly with the workspace cwd. Do not count the first launch
as disc-render evidence. Rapid F6 automation initially skipped inputs; final
04:40 screenshots verify the actual HUD time, not assumed key count.

Preview shut down cleanly. Save SHA256 before/after is unchanged:
0CD03EE33FE4FCFAAFF893271D79A0C6C5885C34B5C9A93D1C375350AA4D49A0.
No settings file created or changed; no save actions sent. .gitignore and
savegame.json remain unrelated and excluded. Geometry size/camera/seasonal ground,
general lighting fidelity and full render-look sign-off are not closed here.

## Philip acceptance

Restart normally; compare the four pictured buildings (and museum) at matched
clock times. At 4 a.m. Able Sisters windows should be dark and police lit.
At 10 a.m. Cranny should have its white/red sign and open entrance; F6 across
09:00/22:00 should move its actual door without duplicates. Confirm shop entry
still works and no textures/parts are missing. Winter post-office selection is
source-checked; visual winter acceptance remains open.
