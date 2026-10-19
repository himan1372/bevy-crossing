# Fish, insects and balloons in rooms

Requested next room-item slice (from the session that ran the light switch):
audit original mRmTp item->furniture conversion for fish (aquarium/fish model
furniture), insects (cage furniture) and balloons, plus the reverse pickup.
Reuse native logic/src/furniture.rs conversions and app/src/player_room
loose/placement/pickup paths; port only what is missing. If the whole family is
too large, take the most source-certain sub-slice and record the rest.

Done: supported items place via pockets "Place in room", render from disc at
runtime, pick up as the exact original item after movement/rotation/reload.
Full/blocked refusals do not mutate state; hidden storage and no-disc numeric
data preserved. check/clippy -D warnings/fmt, capture/default/release builds,
isolated Temp preview with screenshots and F5/F9 numeric equality. No tests.
Actual save hashed before/after, never loaded. Worktree branch only; no push.

## Source audit (ACGC-PC-Port, matches ac-decomp logic)

- m_room_type.c mRmTp_Item1ItemNo2FtrItemNo_AtPlayerRoom: insects
  0x2D00..0x2D27 -> FTR_SUM_MONSHIRO (0x2F2)+n facing south; fish
  0x2300..=ITM_FISH_END -> FTR_SUM_FUNA (0x31A)+n; balloons 0x2244..0x224B ->
  FTR_NOG_BALLOON_COMMON0 (1020=0x3FC)+n, crossing the furniture bank boundary
  at 0x400. These branches precede the no_convert_tools check, so room
  placement converts them. The inclusive fish bound would send non-fish
  0x2328 to umbrella stand 0x342; only the forty real fish convert here.
- mRmTp_FtrItemNo2Item1ItemNo reverses each range, discarding orientation
  (fish via mNT_FishIdx2FishItemNo). aMR_ItemNo2IconNo gives all three the
  default leaf birth/pickup icon. mSM_check_item_for_furniture already
  forbids storing fish/insects in furniture.
- ac_my_room.c aMR_SystemAnimeCt_UniqueCt and the move loop play every
  profile rig as cKF standard repeat at speed 0.5. All fish and most insects
  are plain rig profiles (iam_sum_funa etc., no vtable); iam_nog_ari is plain
  display lists.
- Callback critters: cicadas (0x2F6..0x2F9) and grasshoppers/crickets
  (0x2FF..0x304) start stopped and play only when sAdo_RoomIncectPos admits
  their room sound; firefly 0x30D, mole cricket 0x313, mosquito 0x319 and
  frog 0x33B carry their own move/draw callbacks (room-insect sound, glow).
  No room-insect audio path exists in the app, so these stay in pockets.
- ac_nog_balloon_common.c fNBC: one profile for all eight; ct picks
  cKF_bs/ba_r_int_nog_balloon{1..8} by (name-0x3FC)&7, standard repeat, speed
  dynamic_work_f[0]=0 approaching target 0.5 by 0.01/tick; switch_changed_flag
  sets target 1.25, reaching it returns to 0.5. Balloons have interaction 0,
  so aMR_FtrIdx2ChangeFtrSwitch's generic branch toggles the switch bit
  silently on contact. aMR placement adds +1 orientation for balloons
  (already native special_orientation_offset).

## Scope decision

Balloons are not offered for placement in this slice. Their head model
(int_nog_balloon*_head_model, rel 0xBC48A8 for balloon1) sets G_TEXTURE_GEN
(geometry 0x00270405) with combiner lerp(ENV, PRIM, TEX0) * SHADE and TEX0
alpha. emu64 texture_gen builds UVs from the model-view normal projected on
the look-at X/Y axes, scaled by gSPTexture scale, tile shift and size. The app
ignores tex_gen (tracked in logic graphics state only), so the stored ST
values sample mostly transparent texels and the alpha cutout leaves two rim
slivers (observed in a Temp preview). room_item_to_furniture no longer offers
balloons; balloon_to/from_furniture, balloon_base, the fNBC BalloonSpeed ramp
and the switch kick remain for the texgen follow-up. Placing a balloon now
shows only Grab. The skeleton joint flags are all 0 (opaque lists).

## Verification (Temp only, synthetic saves, absolute EXE, explicit disc)

Builds used CARGO_TARGET_DIR = main checkout target/ (warm deps; the worktree
target/ was cold). Real save SHA256 26CD8CBC...2CB20ADEB before and after;
never loaded. Preview %TEMP%/bevy-crossing-room-critters:
- Place -> render -> F9 reload equal -> exact pickup: fish 0x2300 -> 0x1C68,
  last fish 0x2327 -> 0x1D04 (two-cell, filler 0xFE1F), insect 0x2D00 ->
  0x1BC8, ant 0x2D26 -> 0x1C60 (plain lists), balloons (before the scope
  decision) 0x2244 -> 0x1FF1 and 0x224B -> 0x300D with the +1 orientation.
- Fish pushed 49 -> 33 and turned rot 0 -> 3, F9 equal, exact pickup.
- Balloon first/last, robust cicada 0x2D04 and cricket 0x2D0E: menu shows Grab
  only (screenshots), complete layers/switches/pockets/conds/bells unchanged.
- Full room (15 cages, player cell free): "There's no room to put that here.",
  pockets and layers unchanged.
- Four rigs: 5 of 23 joints change over 1.5 s; pause freezes all.
- Screenshots inspected (tanks with fish, cages, full room, menus). No
  WARN/ERROR besides the gamepad mapping notice.
- No-disc %TEMP%/bevy-crossing-room-critters-nodisc: F5 keeps all 15 cage
  furniture, pockets, conditions and bells.
- After rebasing onto main 67e5dc3c (window sunshine): check, all-target
  clippy -D warnings, fmt, capture 60s; repeated fish/ant roundtrip and rig
  animation/pause pass with sunshine present. Release 3m35s, default 56s
  (pre-rebase), no tests.
Driver notes: pocket cursor presses are dropped during its slide animation
and wrap at row ends; affected runs were excluded and repeated with slot-0
items and a verified cursor. The first "blocked" attempt was not blocked (the
existing placement chose another free cell); replaced by the full-room case.

## Follow-ups

- G_TEXTURE_GEN reflection-map rendering (balloons; likely other shiny
  furniture) -> then enable balloon placement.
- Room-insect audio (sAdo_RoomIncectPos) for cicadas, crickets, firefly glow,
  mole cricket, mosquito, frog.
- Generic rig animation for other furniture with plain rigs; custom designs.
