# Retail shop UI

2026-10-16, codex. Planned only; no implementation or Philip acceptance.

Third UI task, after pockets and mail.

## Scope: shop-floor browsing, Nook dialogue and pockets selling

Philip clarifies the retail flow: walk around the displayed merchandise, interact
with the actual item, and Nook follows/approaches and talks about that item and its
price. His purchase dialogue, choices and response lines carry the transaction.
Selling opens the existing pockets screen in selling mode, with a Sell drop-down
in place of ordinary pocket actions. This task includes the supporting shopkeeper
behavior and transaction handoff as well as presentation.

Reuse existing stock, prices, inventory rules, disc goods, dialogue/font/choices
and pockets rendering. Bind every selectable display to its actual stock slot;
duplicate item IDs must still identify the correct displayed piece. Use original
disc message scripts/substitutions and C state transitions for Nook's lines.

Relevant files: app/src/shop.rs, shop_display.rs, pockets.rs, pockets_disc.rs,
dialogue.rs, messages.rs, msg_window.rs, character.rs, input.rs; logic/src/shop.rs
and inventory helpers. Add a focused Nook controller module if needed.

## Source review, 2026-10-16

Read ACreTeam upstream and Philip's local ACGC-PC-Port counterparts:

- ac_npc_shop_master_move.c_inc: Nook's four zones/waypoints, next-zone table,
  player-facing turn and common move controller. Following is source zone-based
  behavior, with room-safe routing and stopping for interaction.
- ac_npc_shop_common.c::aNSC_message_ctrl_talk_request_normal_day: player action
  queries the forward shop unit through shop_design_clip, identifies its item
  and requests Nook dialogue. Clothing and wall/floor previews have distinct
  branches; ordinary merchandise uses the sale dialogue path.
- Common actor purchase/offer/confirmation states: item/name/price substitutions,
  insufficient funds, full pockets, refusal and reportGoodsSale for the selected
  display. Merchant buy/sell function names are from Nook's perspective; trace
  state transitions rather than inferring the player's operation from a name.
- aNSC_buy_menu_close_wait_init opens inventory mode0x5; m_tag_ovl.c's
  mSM_IV_OPEN_SELL, mTG_sell_item and mTG_sell_proc implement Sell/Quit, return
  the selected pocket slot and apply quest-item restrictions. Nook then computes
  the offer and continues dialogue; selecting Sell must not prematurely commit.

Upstream:
[Nook movement](https://raw.githubusercontent.com/ACreTeam/ac-decomp/master/src/actor/npc/ac_npc_shop_master_move.c_inc),
[common shop controller](https://raw.githubusercontent.com/ACreTeam/ac-decomp/master/src/actor/npc/ac_npc_shop_common.c),
[inventory tags](https://raw.githubusercontent.com/ACreTeam/ac-decomp/master/src/game/m_tag_ovl.c).
Local source root: C:/Users/phili/game c/shmanimal crossing/ACGC-PC-Port.
The upstream ac_shop_goods page could not be fetched; local ac_shop_goods.c was
read. Do not claim that upstream file was inspected.

Current Bevy code already renders daily merchandise and sold-out signs, but the
display renderer retains item/sold values without an interaction stock-index
component. shop.rs still opens a Buy/Sell list at a fixed counter position and
commits transactions from that list. Those are the concrete integration gaps.

## First implementation slice

One complete Cranny purchase and selling chain:

1. Preserve stock-slot identity on rendered goods and pick the item in front of
   the player from the actual display placement.
2. Give the existing Nook model the source zone-follow/facing/stop controller;
   route display interaction into original item-name/price dialogue and choices.
3. Complete an ordinary merchandise purchase or refusal, updating that display,
   bells and inventory only at the source confirmation point.
4. Talk to Nook to sell; open pockets in selling mode with Sell/Quit, return the
   selected slot to Nook's offer dialogue, confirm or decline, then restore the
   correct shop/pockets/world input ownership.

These connected behaviors establish the shop flow before category-specific
try-on/wall/floor previews, multi-item marking and additional services. The task
stays open until its applicable supported categories and Philip's comparison
are recorded; one ordinary purchase is not full shop acceptance.

## Acceptance and boundaries

Philip compares walking among goods, Nook following/stopping/facing, interaction
with the exact display, original dialogue/price/choices, and pockets Sell mode.
Automated/manual review evidence must cover purchase, refusal, sold-out display,
insufficient bells/full pockets, sale offer/accept/decline and cancel/back paths.
Prices, bells, item IDs/conditions and stock identity remain correct; duplicate
goods and per-slot selling cannot accidentally target another item. No leaked
movement/tool/door input or stale selection; the WorldCamera identity stays
explicit when menu/preview cameras are active. Use disposable save data only.

Research and scope corrected only; no shop gameplay implementation, build/tests
or Philip acceptance is claimed by this note update.

Load original assets at runtime from Philip's own disc; never commit extracted
bytes. Keep the no-disc placeholder usable. Use source and existing translations,
not guessed layouts or gameplay rules. Claim before implementation. Run suitable
compile/lint/format/build checks for code changes; no tests unless Philip requests
them. Record implementation evidence separately from Philip's actual acceptance.
M4 remains open; title and rename remain deferred.
