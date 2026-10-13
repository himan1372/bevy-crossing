# mailbox-flag — the mailbox raises its flag for new mail

- [x] ac_mailbox.c: aMBX_check_flag picks flag_on/flag_on_wait/flag_off animations.
- [x] Animate the obj_s/w_post skeleton (cKF_ba_r_obj_s_post_* ) and the flag tip texture (post_flag_saki_model type0/1).
- [x] Gate passes; live check against Philip's retail screenshot.

Done (claude): app/src/mailbox.rs. Mail = any unread letter (our mailbox keeps read letters). Flag tip EVW textures at 30 fps. Not done: the open/close and delivery animations, sound effects.
