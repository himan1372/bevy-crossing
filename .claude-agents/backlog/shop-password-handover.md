# Present hand-over animation (counter passwords)

2026-10-26 claude. Philip: reuse the original rather than rebuild; keep new
kernels in `logic/` independent of the disc so they can move to another project;
check rustimal-crossing for existing kernels.

Findings: pinned `rust/src/hand_over_item.rs` (253 lines, identical in the
sibling working copy) is a complete safe kernel of `ac_handOverItem` (modes,
keyframe tables, 0.5/frame counter, SE 0x56). `name_editor.rs` there only has
buffer helpers; its keyboard tables/state machine were left in C (separate task).

- [x] Port hand_over_item into logic/ (2026-10-26): `logic/src/hand_over_item.rs`, the pinned
      safe kernel minus its tests (sibling working copy identical). Pure, no disc.
- [ ] Nook's present: Nook's hand pose + item actor following the kernel, item
      model from the disc, delivery when the transfer ends (not at message end).
- [ ] Gates, Temp QA, notes/roadmaps/comms, release claim, commit/push.

## Source findings for the visual integration (not started)

`ac_handOverItem_draw`: the item is drawn at the master NPC's left-hand matrix
(`aNPC_hand_c left_hand`, set in `ac_npc_draw.c_inc` from the NPC skeleton) offset
by the kernel's `trans` rotated by the master's yaw -90 degrees, scaled
`anm scale * 0.01` of the item's model (fish use the gyo displaylist); it chases
the hand with a small spring (`aHOI_chase_pos`). Nook's own arm gesture comes
from his NPC animation table in `aNSC_setupAction` (not yet read). Needed in
the app: Nook's left-hand joint position (the player already has
`CharacterRig::left_hand_position`, joint 16; NPC joint index to confirm), the
item's disc model (as loose room items already do), the kernel driven at
0.5/frame, SE 0x56 at frame 24, delivery when the transfer finishes. Release
claim until Philip chooses to continue.
