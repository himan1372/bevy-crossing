# Letter sending (rustimal reuse)

Philip 2026-10-20: "check the rust modules [in the rustimal-crossing repo]
to see if any are already there to cover what we are working on and work on
sending letter next. letters written by us should be blue while npc letters
are red".
- Reuse: npc_reply.rs (ours, shared) for replies; address_book.rs,
  submenu/inventory_ui/menu_overlays (source only) where they cover the
  letter flow.
- Letter icons: player-written letters blue, received letters red.

## Result (claude, 2026-10-20)

Survey of rustimal-crossing (pinned 122867f7) for the mail work:
- Already shared and now used: npc_reply.rs (made `pub`; post.rs uses
  score_received_letter, receive_friendship_delta, good_reply_plan,
  free_string_indices, bad_reply_msg_no instead of my own copies),
  mail.rs (next_delivery_time).
- Source only, adopted: address_book.rs -> logic/src/address_book.rs, a
  safe rewrite of pc_ad_player_list / pc_ad_npc_pages / pc_ad_page_model
  (C-ABI pointer kernels not copied). Decision: same rules, Rust types.
- Source only, not adopted yet (later work): submenu.rs / inventory_ui.rs
  (per-mode pocket eligibility, tag menus), menu_overlays.rs (hand
  stacking), hand_over_item.rs, museum_display.rs, lost_found.rs,
  police.rs (our police box / copper were written natively; could be
  reconciled), mother_mail.rs / npc_event_mail.rs / special_delivery.rs /
  leaflet.rs (shared, unused: Mom's, event, catalog and leaflet mail).
- Envelopes: mSM_draw_mail letter_tex_data_table, six icons (received
  red, sent blue (otegami), read; each with a present variant).
- villager_memory.rs: which villagers remember the player (talk or
  letter), saved as SaveData.villager_memory; the address book lists
  the museum then those villagers; empty book refuses (mAD_PROC_REFUSE).
Verified in Temp preview (real save unchanged): blue sent letter, red
received, gift variant; book = Museum only, then Museum + Bubbles after
talking to her.
