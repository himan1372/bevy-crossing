# walk-run — player locomotion through logic::player_move
- [ ] Walk / run / dash speeds, acceleration and turning from the translated controller (logic::player_move), not a fixed speed.
- [ ] Run action (hold Shift; gamepad later) in the input action map.
- [ ] One shared locomotion step used by town, house and shop movement (replaces three copies).
- [ ] Facing follows the smoothed turn; animation speed hook (player_move::anim_speed) exposed for the M2 player model.
- [ ] Gate passes; live check over BRP; Philip playtest.
