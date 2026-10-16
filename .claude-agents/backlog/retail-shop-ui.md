# Retail shop UI

2026-10-16, codex. First ordinary-day Cranny slice implemented; Philip's
shop playtest acceptance remains pending. This task and M4 remain open.

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

Before this slice, Bevy rendered merchandise/sold signs but retained no display
stock index and opened a Buy/Sell list at a fixed counter. The disc path now
uses those displays and original dialogue. The no-disc counter menu remains.

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

## Implementation and evidence, 2026-10-16

- shop_display.rs attaches exact stock indices/positions to goods and sold
  signs, including duplicate IDs. Corrected source sapling reserve FE05 versus
  flower FE08, so saplings no longer consume one of the two flower slots.
- shop_nook.rs connects facing/forward display selection to original messages,
  item0/free4 purchase substitutions, free2 sale offer, source message-number
  mapping, acceptance/refusal and tool response scripts. Transaction state
  rechecks stock identity, price, funds, capacity and the exact offered pocket
  slot/condition before committing. Wallet overflow stages 30,000-Bell bags
  before mutation; ledger and existing immediate-upgrade policy are retained.
- Nook uses source four-zone routing, waypoints, turn limit, distance stops and
  walk/run speeds; his existing disc rig walks instead of always idling. Room
  surface sampling supplies table/rack/pot collision to both actors, replacing
  the incorrect scaled placeholder counter in disc mode. This sampled geometry
  is an adaptation, not a full translation of mCoBG or NPC movement states.
- Dialogue uses existing retail frame/choices/font, Nook name plate and original
  sound spec2/draw-table voice ID. Runtime item article tables/string bank now
  support item articles and CUT_ARTICLE; existing villager contexts leave the
  new optional article vector empty. Existing villager branching is preserved.
- Talk to Nook offers the source Sell/cancel labels (other services hidden).
  Selling opens pockets mode0x5 with compact Sell/Quit, no mail/grab/tool actions.
  Selecting Sell returns the slot to Nook; his offer performs the confirmation.
  World focus stays blocked during that handoff; exit clears owned state.

Disposable review only (Temp cwd, synthetic saves, explicit user-disc path):
shovel stock5 purchased for500, 12345->11845bells, selected display sold sign,
one shovel added and ledger+500. Declined100-Bell apple offer retained both
duplicates; accepted second apple removed only slot1, paid100 and ledger+50.
Saved Temp state11945bells, slot0 apple/slot1 empty/other tools intact, ledger550.
Sapling purchase decline, insufficient funds (0bells) and full15 pockets showed
original refusal pages; neither failure changed stock/pockets/ledger or money.
F9 between those disposable scenarios restored world input/state. Sell menu
cancel followed by pockets cancel preserved12345bells and all3 items. Source
articles display "That's a sapling." Corrected collision stopped a sustained
forward walk at the table; Nook stayed beside it and dialogue still opened
across it. Explicit missing-disc preview entered the placeholder shop and
opened its existing counter menu. These are manual development observations,
not Philip acceptance or a claim that every save/restart/audio path passed.

Compile/lint/format/capture/default build results and final save hash are
recorded in comms.txt after the final gate. No tests added or run.

## Still open

- Philip's comparison of following, turning, display targeting, dialogue and
  selling. Precise retail cameras, approach/transfer poses and movement blending.
- Clothing try-on, carpet/wall previews and paint/house service. Ordinary
  clothing/wall/floor items currently use the normal purchase offer without a
  preview; paint interaction is deferred rather than adding useless paint items.
- Duplicate/all-item sale prompts, multi-marking, quest/present warning flows,
  zero-value disposal, turnip pricing and special services. Only normal,
  positive-price pocket items offer Sell; unsupported slots offer Quit.
- Ticket gifts with qualifying purchases, repeat-service conversations,
  catalogue/password/remodel/events and additional shop layouts/tiers. The
  existing immediate tier upgrade remains simplified, not retail remodeling.
- Overflow bag logic and duplicate display identity have source/code review
  only in this slice; those edge cases were not exercised live.

Load original assets at runtime from Philip's own disc; never commit extracted
bytes. Keep the no-disc placeholder usable. Use source and existing translations,
not guessed layouts or gameplay rules. Claim before implementation. Run suitable
compile/lint/format/build checks for code changes; no tests unless Philip requests
them. Record implementation evidence separately from Philip's actual acceptance.
M4 remains open; title and rename remain deferred.
