# Retail shop UI

2026-10-16, codex. Planned only; no implementation or Philip acceptance.

Third UI task, after pockets and mail.

## Scope

Trace original buying/selling, choices, item icons and transaction feedback.
Reuse existing shop/inventory rules and disc window/font assets; keep missing
shop gameplay features named separately.

Relevant files: app/src/shop.rs, shop_display.rs, pockets.rs, msg_window.rs;
logic/src/shop.rs and original shop/inventory/window code.

## Acceptance and boundaries

Philip compares buying/selling and item-selection visuals with retail.
Prices, bells, inventory capacity and transaction results remain correct;
cancel/back paths leave no stale selection or leaked gameplay input.

Load original assets at runtime from Philip's own disc; never commit extracted
bytes. Keep the no-disc placeholder usable. Use source and existing translations,
not guessed layouts or gameplay rules. Claim before implementation. Run suitable
compile/lint/format/build checks for code changes; no tests unless Philip requests
them. Record implementation evidence separately from Philip's actual acceptance.
M4 remains open; title and rename remain deferred.

