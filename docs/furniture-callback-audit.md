# Remaining special-furniture callbacks: bounded source audit

2026-10-19, Codex. Audited against main `c902cceb` while Claude owns
`room-insect-audio`. This is a seven-profile source/disc audit, not a complete
furniture inventory, implementation, runtime check or Philip acceptance.
No gameplay files, builds, previews, tests or user saves were touched.

Follow-up: the turkey-TV implementation is now recorded in
[its task note](../.claude-agents/completed/turkey-tv.md). The audit below
retains its original source snapshot. The [electric fan](../.claude-agents/completed/electric-fan.md)
follow-up is also implemented, as is the [gong/boxing bell](../.claude-agents/completed/gong-actions.md).
The [slot machine](../.claude-agents/completed/slot-machine.md) follow-up is also implemented.
The [cuckoo clock](../.claude-agents/completed/cuckoo-clock.md) is implemented. Remaining candidates: sprinkler and lawnmower callbacks.

## Source and lookup method

Original C root: `C:/Users/phili/game c/shmanimal crossing/ACGC-PC-Port`.
Paths below are relative to that root. Read the complete seven callback files,
plus `src/actor/ac_my_room_move.c_inc` switch/contact dispatch. This working
source includes TARGET_PC delta-time adapters; those additions are not evidence
of original GameCube frame timing. File digests below pin what was inspected.
For a future native port also compare pinned migration revision
`122867f762122eb266f04e319af0f0c31d54f476`, preserving existing native kernels.

Read-only Python disc-reader inspection confirmed GAFE01 revision 0, named
profiles, relocated table membership and selected assets. Offsets are file
positions in the decompressed REL, not RAM addresses. `furniture_quality` has
TWO symbol matches: REL `0x3172F4` and `0x3588F0`, each size `0x13C8`.
[AssetCatalog::get](../disc/src/assets.rs) chooses the lowest REL offset;
both tables resolve these seven profiles at identical indices. Do not silently
assume a symbol name is unique. Items below use South/rotation-zero encoding;
other orientations occupy their existing low two bits.

| Profile | Index | Item | Profile REL offset | Interaction bits |
|---|---:|---:|---:|---:|
| `iam_nog_fan01` | `0x22` | `0x1088` | `0x367664` | `0x0000` |
| `iam_nog_gong` | `0x4EE` | `0x33B8` | `0x36FFE8` | `0x0000` |
| `iam_sum_slot` | `0xC6` | `0x1318` | `0x36CAF0` | `0x0000` |
| `iam_nog_sprinkler` | `0x4DA` | `0x3368` | `0x370078` | `0x1000` |
| `iam_sum_log_hatoclk` | `0xC1` | `0x1304` | `0x36B8F8` | `0x0000` |
| `iam_iku_turkey_TV` | `0x4B9` | `0x32E4` | `0x36F030` | `0x1000` |
| `iam_nog_lawnmower` | `0x4E8` | `0x33A0` | `0x370030` | `0x0000` |

Every profile has a relocated vtable pointer at +0x30; all eight registered
model/rig/texture-animation pointer fields before +0x20 are null. The callbacks
supply the models or skeletons, so a generic profile loader cannot infer all
behavior from those fields. `0x1000` is `START_DISABLED`, not proof that E must
be permanently unavailable. Trace initialization/contact rules before adapting it.

## Existing app boundary

[Profile loading and rendering](../app/src/furniture.rs) has explicit TV/clock
adapters, skeleton-name inference and limited generic static-list fallbacks.
A drawable first-frame pose does not mean its move/draw callback is implemented.
[Clock specs](../app/src/furniture/clocks.rs) enumerate ten profiles and exclude
`sum_log_hatoclk`. [TV kinds](../app/src/furniture/television.rs) enumerate four
and exclude `iku_turkey_TV`. [Native TV timing](../logic/src/television.rs) advances
only while powered; that assumption does not fit the turkey callback below.

The inspected [room interaction dispatcher](../app/src/player_room.rs) handles
diary, music, those TVs, balloons and storage. It is not a generic implementation
of `aMR_FtrIdx2ChangeFtrSwitch`. Original C's final switch branch toggles silently
and raises `switch_changed_flag`; special callbacks decide which sound/action
follows. Do not add the TV's on/off sounds to every switch: a gong needs a strike,
and the sprinkler has its own transition gate. Original contact/release handling
also differs from the port's PC E adaptation. Recheck dispatch after Claude's
current edits are committed; do not edit his live room/audio files for this audit.

## Source-backed candidates

### 1. Turkey TV: recommended next implementation

`src/furniture/ac_iku_turkey_TV.c`: `fITT_mv`, `fITT_dw`, `iam_iku_turkey_TV`.
The phase advances modulo 24 on every source update, including while off.
While powered, segment 8 uses g/h/i/h, each for six ticks; off uses f.
The draw order is model_b then model_a, both opaque. Powered positional level
sound is `0x5E`; a switch change emits `0x16` or `0x17`. Existing native switches,
TV material caching and audio ownership should be extended rather than replaced.

Disc-confirmed assets: `int_iku_turkey_TV_model_b_model` REL `0xB476E0` and
`model_a_model` REL `0xB47740`; screen `int_iku_turkey_TV_f_tex_txt` at
`0xB470A0`, g at `0xB47120`, h at `0xB471A0`, i at `0xB47220`.
Each screen symbol is only `0x80` bytes. Decode dimensions/format from the
actual list; do not copy a 64x64 TV-frame assumption. Registered profile lists
are null and generic `_on_model`/`_off_model` suffixes do not match these assets.

Implementation boundary: add one native TV kind with independent phase policy,
explicit model/screen names, matching complete placement preflight and existing
E/save/audio lifecycle integration. No new save schema or UI. This will touch
Claude-owned room/sound areas; start implementation after his task releases.

Checks: off-phase advance followed by power-on at a known phase; exact six-tick
screen order; E sounds and level `0x5E`; move/turn, full-pocket refusal, pickup,
F5/F9/fresh/no-disc state and existing four-TV regression. Inspect actual screen
and body renders. Source phase math and audible acceptance are separate evidence.

### 2. Electric fan

`src/furniture/ac_nog_fan01.c`: `aNogFan_ct/mv/dw`.
The switch eases speed toward 0.5 or 0 using add_calc parameters
0.05/0.25/0.0005 (inspect CALC_EASE for the chosen timestep). The skeletal
animation repeats at that speed. A separate texture counter uses the ten-entry
1/2/3/-1 delay table; six texture frames select three segment-8 textures via
frame >> 1. Powered level sound is 1; switch changes emit 0x16/0x17.

The callback uses `cKF_bs_r_int_nog_fan` and `cKF_ba_r_int_nog_fan`, not names
ending in fan01; both symbols exist. Current profile-name skeleton inference
needs an explicit alias and full texture preflight. Implement spin-up/coast-down,
texture delay and source sound gates, not just a binary animation flag.
Check start powered/off, both transitions, fully stopped texture, pause, ownership,
movement, save/reload and pickup. This is the room electric fan, separate from
implemented handheld summer fans.

### 3. Gong

`src/furniture/ac_nog_gong.c`: `aNogG_ct/mv/dw`.
Stopped skeletal animation is initialized at half speed. Once it is stopped,
a switch change restarts at frame 1 and emits positional strike `0x174` when
sound is allowed. While the animation is running the callback calls play twice;
preserve or explicitly reconcile that behavior against cKF before claiming
half-speed frame equivalence. A running strike does not take the restart branch.

On ringside-seating wallpaper `WALL_NPC065`, a permitted strike also requests
singleton sound `0x17E` and `eEC_EFFECT_FLASHC` camera flashes. The whole audited
callback includes this dependency: shipping a strike-only slice must explicitly
record the missing wallpaper effect rather than claim full gong fidelity.
No money reward or generic switch click appears here. Check rapid repeated E,
idle restart, pause, floor/surface positions, alternate wallpaper, sound retirement
and numeric state preservation.

### 4. Slot machine

`src/furniture/ac_sum_slot.c`: `aSumSlot_ct/mv/dw`.
Starts stopped, and an idle switch change starts a one-shot skeleton animation.
While running, the 242-tick texture counter indexes a two-texture sequence
through the 121-entry `aSumSlot_aime_idx` table using frame >> 1; positional
level sound is 2. On animation stop, the local running flag clears. Input while
running does not enter the idle restart branch. There is no wallet mutation,
payout, random prize or gambling UI in this callback; do not invent one.

Needs a cached segment-8 texture table and one-shot state, not generic repeat
rig playback. Verify the exact table, initial frame and stop boundary, repeated
E, pause, sound gates and unchanged wallet/inventory. Audit dependencies before
reusing a generic trigger animation helper for both slot machine and gong.

### 5. Sprinkler

`src/furniture/ac_nog_sprinkler.c`: `fNS_ct/mv/dw/dt`.
Copies saved switch into a local on flag, fades water alpha to 100 or 0 in
steps of 4, and only handles a new switch-change in the equal-alpha branch.
Thus a toggle during a fade is not equivalent to immediately retargeting alpha.
A supported transition emits 0x16/0x17; local on state supplies level `0x5B`.
The destructor copies local state back to switch_bit, an important save/lifecycle
boundary when the actor switch and local state disagree temporarily.

Opaque bodyT plus translucent water are explicit callback lists. Segment-8
commands scroll two 16x16 tiles, with the second moving by -10 times phase;
water environment colour is (0,180,255,alpha). Needs dynamic scrolling and
material colour, not merely a skinned rig. Verify rapid E during both fades,
phase wrap, reload during a transition and lifecycle/save synchronization.
The PC source scales scroll phase with dt but not every alpha operation;
resolve original 60 Hz semantics rather than copying mixed timing blindly.

### 6. Cuckoo clock

`src/furniture/ac_sum_log_hatoclk.c`: `aSumLogHatoclk_*`.
A separate three-state mechanism starts on minute 0 (hour-count table, midnight
and noon = 12) or minute 30 (one call). It alternates stop/play handling and
rearms only after leaving both minute windows. Entry during a trigger minute
can start the sequence: do not require a crossed-time event without comparison.
Sound `0x133` is gated at frame GETREG(MYKREG,33)+2; resolve the register default
and cKF stopping behavior before fixing that event frame in a native helper.

Before/after draw callbacks suppress joint models 7/8 and draw separate short/
long hands with the source world-space pivot/rotation, not a simple generic
local-joint Z replacement. The skeleton, animation and both extra hand symbols
exist. A blanket addition to the ten-clock spec would miss those semantics.
This is automatic time-based behavior, not configurable alarm UI. Check noon,
midnight, half-hour, rearm, room entry, F6 jumps and pause as distinct cases.

### 7. Lawnmower

`src/furniture/ac_nog_lawnmower.c`: `fNLM_mv/dw`.
This is a floor/contact effect, not a powered motor toggle: grass appears only
on `FLOOR_NPC026` or `FLOOR_NPC048`, in WAIT_PUSH/WAIT_PUSH2/WAIT_PUSH3/PUSH,
with back contact. Its alpha eases via add_calc(0.04,0.1,0.001).
The callback draws an opaque bodyT plus translucent scrolling grass with
primitive colour (120,255,180,alpha). It contains no sound request.
Needs source contact-side/state integration and dynamic draw commands; do not
invent outdoor mowing, persistent cut grass, a switch sound or new floor state.
Check both source floor IDs, a third floor, front/back contact, pull versus push,
stop/fade and unchanged installed decoration/item state.

## File fingerprints

SHA256 of the inspected C working files (no sibling files were modified):

| `src/furniture/` filename | SHA256 |
|---|---|
| `ac_nog_fan01.c` | `f3e09ec549a7077000ac04f72f390b9e0cfaaaea8bca80ec650f7f361100e456` |
| `ac_nog_gong.c` | `a7cf3301ff47ef2f29ba04786021c1237b15d8a2570fa4881ae7d6f0776382e2` |
| `ac_sum_slot.c` | `2927ede8a9a4de7db37bb29fcc7ce671b955f3dc6283a8e403677fa60e075e38` |
| `ac_nog_sprinkler.c` | `cd515eda3aadbddf88d5e2ca7397b3ed0f3c40cf84a497fe7abdc4cb95fd999b` |
| `ac_sum_log_hatoclk.c` | `bebe8e0194778081aaa01652ee5a0e466e71f031305652cca959a844822fa521` |
| `ac_iku_turkey_TV.c` | `94bae9afff413a82ea69640086e7fdfae316a991917cebdb2b1d0cfcac208feb` |
| `ac_nog_lawnmower.c` | `6179bf4f545d2974b15997429a903164d0e53ccdd40430d0087cc3c2a0aefa39` |

## Completion and next task

Read-only profile/table/asset lookups and current dispatch review are complete.
This audit does not close the roadmap's remaining special-furniture item or
increase implemented-gameplay counts. The implementation shortlist is turkey
TV, electric fan, then gong/slot with their dependency limits; sprinkler,
cuckoo clock and lawnmower need the extra state/render/contact work above.
No runtime success or Philip sign-off is claimed. Insect/glow/frog implementation
remains Claude's task. Recheck his committed interfaces before beginning any
candidate. Any resulting Rust task uses the shared sequential build gates and
Temp-only runtime checks; tests still require Philip's request.
