# Wear a shirt from the pockets

Philip: "do those" (2026-10-28): (1) wear a shirt from the pockets (saved, player model updates), (2) demo-order markers in messages, (3) the guide flow for stage 1, (4) stages 2 and 3.
This is item 1. Source: `m_tag_ovl.c` (inventory cloth "Change"), `mPr_` cloth, `Now_Private->cloth.item`, the player's `cloth.item` drives the player's shirt texture; the worn shirt becomes the pocket item.

## Implemented (2026-10-28, claude)
Source: `m_tag_ovl.c`. A shirt is worn by carrying it to the player, not from a menu: `mTG_move_cursol_between_table_inventory_upper` moves a hand carrying a normal-condition
cloth (also fruit, tool, turnips: not built) from the item table's top row up onto `mTG_TABLE_PLAYER` (`mTG_move_check_hand_item_on_player`), down returns, and dropping there wears it
(`mTG_drop_item`, sound `NA_SE_WEAR`); the old shirt takes the slot the hand came from.
- `app/src/pockets.rs`: `PocketsUi.on_player`; Up from the top row with a grabbed shirt (0x2400..0x24FD, condition normal) puts the hand on the player, Down returns it, Confirm wears it
  (`PlayerLook.cloth`, the old shirt item goes into the grabbed slot, sound `WEAR`); Cancel/close clear it. `PlayerLook` is saved as `player_cloth`, so the shirt persists.
- `app/src/pockets_disc.rs`: the hand sits at `PLAYER_AT` (screen 101, 57 = tag col -59 / line 63).
- `app/src/shop_nook/preview.rs::textures`: the player's shirt parts show `PlayerLook.cloth` whenever no try-on preview is active, so the world model follows the worn shirt (and returns to it after a try-on).
Gates: fmt, clippy -D warnings (workspace, all targets), capture build.
Verified (Temp, disc, synthetic tier-0 save with the work uniform in slot 1): open pockets, Enter, Enter (Grab), Up -> the hand and the label "work uniform" sit on the player portrait, Enter -> the label of slot 1 becomes the old shirt ("flame shirt"); F5
saved `player_cloth` 16 and pocket 0x2400 (the old shirt); the world model shows the green uniform from above. No WARN/ERROR.
Not done: the pockets' player portrait is built once and does not change shirt until the pockets are reopened after a reload; `mIV_ANIM_CHANGE` (the portrait's wear animation) and the fruit/tool/turnip drops onto the player; a
shirt in a wrapped or quest condition is refused by `wearable_shirt` (source: only normal condition goes to the player table).
