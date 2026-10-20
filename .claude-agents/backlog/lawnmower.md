# Lawnmower (nog_lawnmower)

Source: `src/furniture/ac_nog_lawnmower.c` (`fNLM_mv/dw`), audited in
[docs/furniture-callback-audit.md](../../docs/furniture-callback-audit.md) §7.
Profile index 0x4E8, item 0x33A0. A floor/contact effect, not a powered
toggle: grass appears only on FLOOR_NPC026 or FLOOR_NPC048 while the piece is
in WAIT_PUSH/WAIT_PUSH2/WAIT_PUSH3/PUSH. Ties into the existing
push/pull movement and decorating floor index. Planned, not started.
