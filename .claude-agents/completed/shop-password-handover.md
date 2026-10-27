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
- [x] Nook's present (2026-10-26): `shop_nook/handover.rs` drives the kernel at 0.5 count a 60 Hz
      frame: Out (TRANSFER, Nook's joint-16 origin plus the kernel offset, SE 0x56
      at count 24), a 0.3 s hold, Get and Away (PUTAWAY into the player's left hand,
      the present's after-mode 7). Model: the wrapped present from the disc's
      `present_DL_mode`/`present_DL_vtx` (passwords always draw the present, not
      the item). Pocket commit stays at message end; the animation is visual and
      locks input until done.
- [x] Gates, Temp QA, notes/roadmaps/comms, release claim, commit/push.

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

## Implemented and verified (2026-10-26)

`CharacterRig::left_hand_point` generalises the player's hand helper (NPCs use the
joint origin, `aNPC_set_left_hand_item`; the player keeps VecX 1100). The present
is spawned from the disc's present display lists on the first use and scaled
`kernel scale * 0.01 / 40` per model unit, offsets `/40` toward the other talker.
Input focus stays locked while it plays. Leaving the shop despawns it.

Verified in a Temp preview (disc, synthetic save): ten frames after the final
message page show the white wrapped present with the red ribbon appearing at
Nook's left hand, growing, passing to the player's side and shrinking away;
pockets/conds as before. No WARN/ERROR in the log. Not done: Nook's own arm
gesture (his NPC animation table in `aNSC_setupAction` was not read), the
`aHOI_chase_pos` spring, and the exact script-ordered start (it starts when
the message ends, not mid-message). Direction signs of the lateral offset are
unverified against the original.
