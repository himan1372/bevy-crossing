# Slot-machine animation and sound

Authorized by Philip 2026-10-19. Identity codex; bounded to original
`ac_sum_slot.c` callback, furniture index0xC6/item0x1318.

- [x] Native stopped-only one-shot, sample-before-advance cKF control and exact
  121-entry reel table (242 ticks). No payout, random prize or wallet changes.
- [x] Runtime disc skeleton, animation and two cached reel textures; full placement
  preflight, silent generic E switch and existing positional level source2.
- [x] Pause/load/room-exit and movement/pickup lifecycle; synthetic Temp QA only.
- [x] Sequential check/clippy/fmt/capture/default/release gates, no tests.
- [x] Roadmaps/comms and owned-hunk staging prepared; release claim and publish below.

Preserve Claude's room-insect WIP, unrelated .gitignore and actual save.
Actual save SHA256 C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.

## Source and implementation

Original `src/furniture/ac_sum_slot.c` SHA256
`2927ede8a9a4de7db37bb29fcc7ce671b955f3dc6283a8e403677fa60e075e38`;
`src/c_keyframe.c` sample-before-advance `stop_proc` crossing reviewed.
Pinned migration122867f has furniture.rs but no slot callback kernel; existing
native furniture switches/transactions retained, new bounded C timing port.
GAFE01 rev0 disc `cKF_ba_r_int_sum_slot` REL0xD39DC4 has121 frames. Exact
121-entry reel table independently compared to C; 242-tick phase, index>>1.
Constructor samples1, next1.5, speed0; first activation samples1 before setting
speed0.5, subsequent activations retain half speed. Running callback samples
before crossing check, advances textures/sound only on non-stopped ticks;
stopping pose end-0.5 retained. Idle triggers start, busy changes are consumed.
No payout/random prize/UI, no new sound/save framework.

App uses the established fan renderer pattern with cached dynamic-texture
materials/images, skinned joint hierarchy and dynamic bounds. Full decoded
placement preflight checks both segment8 textures, skeleton and animation.
Native state and pose stay transient. Stable entity level2 goes through the
existing four-source ambience mixer and room/birth/pickup/modal gates.
Load uses existing ResetAudio message; scene exit resets even reused entities.
No edits to Claude's rigs/game_disc/sound/insect code; shared room/lib hooks
will be staged separately from his WIP. Build gates include shared WIP and do
not validate or publish his task. 60Hz catch-up bounded4 as other adapters;
full retail/hardware audio comparison remains outside this slice.

## Verification in progress

No tests added/run. Source table comparison and synthetic Temp BRP checks only.
Initial preview3848 found missing QuickLoad reset because refresh retains rigs;
fixed explicitly. Its final lifecycle checks are excluded. Its helper also
initially assumed cell34 for pocket placement, corrected to observed cell/bit.
Corrected final preview24388 repeats the complete scenario.
Final check3.23s (before parameter grouping), all-target clippy-Dwarnings4.57s,
owned rustfmt and corrected capture19.46s pass. Workspace fmt has only known
Claude sound.rs/insects.rs differences after formatting owned furniture.rs.
Runtime evidence/helpers/screenshots stay in Temp/bevy-crossing-slot-machine.

## Runtime evidence

Corrected preview24388: placement from pockets, initially stopped5-joint home
rig, E motion, both reel images (screens inspected), exact pause, busy E/no
restart, settled stop and reverse switch-direction activation pass. Exact
room/pockets/conditions/bells compared, including no wallet mutation. Mid-spin
F5/F9 now resets pose/reels and does not replay. Pickup returns exact item and
retires the home rig. A helper initially treated the shop's separate slot rig
(root x3602.5) as a leaked home rig; numeric pickup passed, observed remaining
5 shop joints, corrected the assertion. No source fix needed for that assertion.
Remaining checks resumed on the same final executable: movement34->18 preserves
switch512->2, turning preserves identity/ownership, exact pickup and full-pocket
refusal pass. Mixed powered fan/Turkey-TV/slot plus hidden stored slot/shirt and
wall/floor data round-trip exactly; scene exit returns town/render as expected.
Fresh29540 restores the complete mixed state, no replay, one visible home rig
plus the separate shop rig; hidden slot does not spawn. Pickup during an active
spin separately returns exact item and leaves only the shop's5 joints.
No-disc26220 F5 preserves complete numeric room/pockets/conditions/bells.
Idle/spinning/paused/fresh/mixed/town/placeholder screenshots actually inspected.
Audio diagnostics show nonfinite0, active voices0 after isolated spins; sound
is emitted on the existing ambience channel. Audible retail/Philip acceptance
is pending. All owned previews closed before final builds. No actual save used;
read-only SHA256 remains the recorded baseline.

Default workspace build42.45s passed. Release build at unchanged original
settings is running; no source changes since corrected capture/clippy.
Owned hook patch contains exactly2 room hunks and1 export, excludes all insect
changes. HTML86=53/6/27, duplicate IDs/local audit/task links checked. Actual
save SHA256 unchanged after all previews; all four owned preview PIDs closed.

## Complete

Release workspace build3m00s passes at original settings. Corrected capture
19.46s/default42.45s, check3.23s and final all-target clippy-Dwarnings4.57s pass.
Owned rustfmt passes; final workspace fmt flags only Claude's pre-existing
sound.rs/insects.rs formatting, left untouched. No tests added/run.
All owned previews closed, actual save unchanged. Final source unchanged since
successful runtime/default/release checks. Both roadmaps/audit/task current;
86 tracked items53/6/27, not overall completion. Release own claim and publish
only owned slot source/hooks/docs/logs to main; Claude WIP remains excluded.
Philip visual/audio acceptance, M4 and broader migration remain open.
