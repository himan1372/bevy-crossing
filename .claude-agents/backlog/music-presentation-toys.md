# Music-player presentation and handheld toys

Philip accepts music playback and authorizes music-player presentation plus
fan/pinwheel holding and use. Preserve that bounded acceptance; M4 stays open.

Use original runtime-disc assets/source for the 55-song grid, furniture/player
interaction presentation and device playback behavior; add holding/use for all
eight fans and eight pinwheels. Preserve registration/storage/save transactions,
net behavior and existing room display. No actual-save automation, extracted
assets, new tests, or handoff edits. Actual-save baseline SHA256:
26CD8CBC01402486D1E9CF9A0874ABD59021815688CE8FDD86FF4DB2CB20ADEB.

One combined task claim; update roadmap.html and ROADMAP.md with final evidence,
acceptance distinctions and remaining plans. Check/clippy/fmt, appropriate
capture/default/release builds and disposable visual/runtime checks.

## Source findings and implementation

- m_music_ovl: 11 columns x5 rows, source note slots at(-100,+28) stepping20,
  title at y55, unknown title?????, original mus_win/mus_icon assets. Uses
  runtime font and existing source hand-render camera plus original choice UI.
- Source cloth uses tile0 for colour/tile1 for coverage. Reuses existing scoped
  inventory-mask interpreter and PocketPattern material for the base only;
  top/border lists retain their own texture/combiner. No global GX behavior change.
  Source per-part draw order is retained with small depth offsets. Dynamic text
  meshes/materials are released on redraw/close; no persistent extracted assets.
- Grid owns Furniture focus, blocks locomotion/G manipulation and consumes its
  actions before doors/objects. Existing movement needed an explicit grid guard.
  Navigation wraps across all55 slots, E/Enter opens Listen/Take out/Quit; Escape
  backs out. Full pockets do not clear registration; F9 clears transient UI.
- All16 original MUSIC_DISK constructors call MiniDiskCommonMove with0,0:
  their source menu open/close does not animate a lid or run storage kagu clips.
  Earlier proposed animation scope was based on a false assumption; do not invent it.
- Na_MDPlayerPos/Sou_GroupControl_MD: reuse native small-room distance2vol_md
  and angle2pan through logic::ambience::music_spatial, source profile gain
  classes1.08/1.4/1.0, both existing BGM groups and menu ducking. Device filter/hiss
  remains part of the pre-existing hardware mixer fidelity follow-up.
- Toys: original tol_fan1..8_model and cKF_bs_r_tol_kaza1..8, original player
  utiwa_wait1/utiwa_d1/kaza1 and rotor tol_kaza1_wait. Right-hand callback20;
  raw fan model units retain parent player scale, pinwheels retain native skins.
  BOY_part_data FAN table4 blends the active arm with existing locomotion.
- Fan: E repeats original wave clip at30frames/sec, source1.5-frame sound0x167,
  source8-frame movement interrupt, pose morph. Pinwheel samples actual rotor
  attachment motion along its normal, source target8x/damping/min/max/coasting
  and clip frame speed; stable source0x4D level scales with abs(speed)/44.
  No constant breeze invented; weather-driven wind remains separate.
- Holding stays backed by normal pocket items. Presented/conditioned toys do
  not offer Hold. Place/store/removal/F9 clear invalid held presentation; no item
  duplication or equipment save-format extension. 3 puts away. F1 is an explicit
  home-only two-free-pocket playtest grant; F2 music aid remains intact.

## Bounded runtime evidence so far

Temp bevy-crossing-music-presentation-toys only: original grid/frame/masked cloth,
title, note icons, source cursor and choice menu visually inspected. Listen35
and last54 preserve exact selections/switches and use correct sequences. Take-out
54 returns10806, clears source bit/selection and stops playback. Unregistered
slot/cancel and full-pocket take-out have exact room/pockets/conditions/wallet
equality. Grid WASD leaves player position unchanged. Mid-grid F5/F9 preserves
numeric state and removes the overlay. Earlier pre-fix focus/asset/cursor/mask
attempts were excluded and repeated after fixes.

All16 held roots have source render children; first/last fan and pinwheel variants
visually inspected, fan wave changes arm pose. Pinwheel rotor quaternions change
during walk/run. Variant use preserves complete room/pockets/conditions/wallet;
F9 hides every held toy and preserves exact numeric state. Held fan placement
removes its pocket copy, leaves exactly one room copy and hides the held root.
F1 grants exactly two copies on success and preserves pockets with only one
empty slot. Main preview logs have no WARN/ERROR. New visuals/toy listening await
Philip; only previously implemented music playback is accepted by him.

Final fresh-disc diagnostic preview confirms rotor motion with the corrected
local-Z axis and frozen rotor while paused. Conditioned fan exposes Grab only;
empty collection refusal preserves complete room state. Audio output stays
finite, no limiting/nonfinite samples, active voices settle to zero. Diagnostic
peaks are not listening acceptance. Python helper failures (stdlib inspect
shadowing and entity-order assumption) were corrected and excluded, then checks
repeated. Main and final disc logs contain no WARN/ERROR.

Fresh missing-disc preview34440 uses only the Temp synthetic save, displays the
expected placeholder-art warning, and F5 preserves exact home_furniture,
pockets, pocket_conds and bells. Screenshot inspected; own preview closed.
All evidence/helpers/screenshots stay in Temp. Actual save hash remains the
baseline above; settings, .gitignore and handoff preserved.

Final check, all-target clippy with -D warnings, fmt check and capture build
pass; default workspace build passes (47.31s). Browser verification confirms
72 roadmap items (38 implemented/6 in progress/28 planned), working search and
status filters, and no overflow at desktop1280/mobile390. Release workspace build
passes (3m12s). No previews, browser or Cargo left running. No tests added/run.
Implementation is complete; new presentation/toy acceptance and the fidelity
follow-ups below remain open. Release claim and publish authorized source/docs.

## Remaining source fidelity

Music multi-mark actions, source submenu slide/cloth timing, placement random
selection/toggle/contact details and hardware device filters/hiss. Toy face/part
blend and wind-driven rotation can receive further retail comparison once
weather exists. Broader source fidelity/M4/migration remain open. Roadmap next
proposals now prioritize remaining room item families and decorating; no automatic
new task or bulk migration is authorized by this completion.
