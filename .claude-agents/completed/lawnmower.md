# Lawnmower (nog_lawnmower)

Source: `src/furniture/ac_nog_lawnmower.c` (`fNLM_mv/dw`), audited in
[docs/furniture-callback-audit.md](../../docs/furniture-callback-audit.md) §7.
Profile index 0x4E8, item 0x33A0. A floor/contact effect, not a powered
toggle: grass appears only on FLOOR_NPC026 or FLOOR_NPC048 while the piece is
in WAIT_PUSH/WAIT_PUSH2/WAIT_PUSH3/PUSH. Ties into the existing
push/pull movement and decorating floor index. Planned, not started.

## Implementation (claude, 2026-10-19)

logic::furniture_lawnmower (LAWN_FLOORS 26/48, grass_step with retail
add_calc, grass_alpha, SE 0x177, GRASS_PRIM). app::furniture::lawnmower:
body + grass anchor from disc lists, 60 Hz grass level from
PlayerRoom::mower_cutting (decoration floor in 26/48, Movement::pushing():
active Push or grip held with input toward the piece, and player facing along
the mower's forward = back contact), grass materials' prim (120,255,180,
level), reset on leaving (fNLM_ct). movement.rs plays 0x177 when a Push
commits for the mower with back contact on the lawn carpets (aMR_SetMoveSE);
the port still has no generic floor drag sounds.

## Verification (Temp only; no tests)

Real save C15154C0... unchanged. Floor 26: holding G+S from the mower's back
shows a lighter cut-grass patch (screenshot); two pushes to (2,3) and (2,4)
logged two 0x177 sounds (temporary log, removed); third push blocked by the
wall with the original notice. Floor 38: same pushes, no grass, 0 sounds.
Gates: check, all-target clippy -D warnings, fmt, capture, default 39s,
release 3m01s.
Limits: grass scroll (TEXEL1) not rendered; side/front-contact negative case
not driven separately (same back-contact test); generic floor drag sounds
absent port-wide.
