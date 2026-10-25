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

Updated2026-10-25 at Philip's request. Priority order:

1. Turnip-price questions and selling, with source date/price/quantity/spoilage.
2. Repeat-service conversations and password service/validation/item delivery.
3. Additional shop layouts/tiers and their opening/closing/service rules,
   Nookington's paint floor and Nookway clock/uniform/exterior; replace immediate
   tier upgrade with the original renovation timing.
4. Remaining seasonal stock/service branches, following original availability.
5. Original pockets quest-warning presentation, remaining approach/transfer
   poses, cameras and movement blending, plus
   Philip's comparison of following, turning, targeting, dialogue, selling and
   the recent visual/audio changes.

Already implemented by later slices: normal marked/duplicate-item selling,
Nookway paint/roof service and its ordinary
room/merchandise layout, clothing try-on and carpet/wall previews,
clothing-change sparkles/wear trigger, ticket gifts/full-pocket ticket mail,
catalog orders and delivery, raffle, and house expansion/roof selection during
expansion/basement/statue progression. These supersede the older missing-feature
notes above. Development checks are recorded separately from Philip acceptance.
Original single-item sale and exact stock-slot validation remain the foundation;
the first slice's limited edge-case evidence is historical, not a new claim
that every current combination has been played.

Load original assets at runtime from Philip's own disc; never commit extracted
bytes. Keep the no-disc placeholder usable. Use source and existing translations,
not guessed layouts or gameplay rules. Claim before implementation. Run suitable
compile/lint/format/build checks for code changes; no tests unless Philip requests
them. Record implementation evidence separately from Philip's actual acceptance.
M4 remains open; title and rename remain deferred.

2026-10-24 codex: completed/retail-tag-art.md updates the shared pockets
Sell/Quit popup to original sen_win frame/shadow/cursor/colors/font, with source
body sizing. Inspected actual shop Sell/Quit; Quit then pockets cancellation
preserves every saved field except position/playtime. No shop transaction or
category-preview behavior changed; parent ownership/acceptance unchanged.

2026-10-25 codex: completed/retail-tag-layout-motion.md extends the shared
Sell/Quit popup with native after-item positioning and source open/return rates.
Shop capture/cancel comparison preserved every field except position/playtime.
Category previews/services and Philip acceptance remain open.

2026-10-25 codex: shop-category-previews implements the bounded ordinary-day
clothing/wallpaper/carpet slice (see completed task evidence). Original18/40
preview/skip/cancel ->19/20/41/42 purchase questions, temporary material swaps,
MENU_CHANGE1/frame36 shirt switch and restoration before transaction, bare
mannequin, native room camera endpoints/28tick Hermite and reverse. Exact stock
identity, funds, pocket and ticket handling retained; no saved appearance writes.
Input remains reserved throughout changing; leaving/loading resets owned talk.
Parent ownership/M4/Philip acceptance remain open. Change puff/wear sound,
other shop services, pause/save presentation and special tag families deferred.

2026-10-25 codex: shop-change-effects completes the clothing gesture effect:
source ef_kigae spiral/ef_kigae_light children and one positional NA_SE_WEAR
trigger at frame16 on try-on and restoration. Native cached sparkle art,
scale/fall/timing and feel-joint anchor; load/scene/AppState cleanup and gameplay
pause gating. Captures, event/particle samples and full saved-state comparisons
pass. Audible comparison remains Philip acceptance; no new shop services in
this bounded detail task. See completed/shop-change-effects.md for evidence.

2026-10-25 codex: completed/shop-paint-service.md adds native202/203 service,
charging/selling the selected paint display and queueing next_outlook_pal with
no pocket item/ticket/capacity refusal. Roof loads/redraws own same-size house
only through cached original palettes. Ordinary Nookway room/layout/collision/
door/preview coordinates were a necessary dependency: Cranny has no paint slot.
Normal/full, refusal/cancel, duplicate slot,12palettes/cache, reload/restart and
Cranny checks pass; see evidence for the separate startup field_items delta.
Higher-tier completion (including Nookington's paint access), Nookway clock/
uniform/exterior, other services and Philip acceptance remain open.

2026-10-25 codex: completed/shop-multi-selling.md adds native M marks/Sell All,
duplicate all-or-one and paper sheet quantities for normal positive-price goods.
Exact slots/conditions are validated, all removals/remainders and wallet/bags
commit together after final confirmation. Batch625, duplicate100/200, paper10/60,
overflow bag and full-pocket partial-paper refusal checked in synthetic Temp;
unrelated/protected state and cancel/reopen preserved. Mark art/count prompts
and no-disc pockets inspected; final build/lint gates and source review pass.
No tests or actual-save loads. Quest/present warnings, zero disposal, turnips,
other services and Philip acceptance remain open; parent ownership unchanged.

2026-10-25 codex: completed/shop-selling-warnings.md adds native57 quest
refusal, exact wrapped-slot selling and56/216 zero-value disposal, with58/59
accept/cancel. Any marked quest refuses the whole offer; mixed junk/priced
goods pay only priced value. Synthetic Temp checks preserve all unrelated saved
fields and cancellation; quest/gift/junk transactions inspected. Original pockets
pre-transfer warning-window presentation remains fidelity work, not implemented
by this slice. Turnips are next; parent ownership/M4/Philip acceptance stay open.
