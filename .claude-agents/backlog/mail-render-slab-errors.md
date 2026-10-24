# Renderer allocation errors while viewing delivered mail

Observed2026-10-24 at372e29bf during catalog-raffle-verification. Untriaged.

Fresh startup delivered ticket0x2C48 and catalog bed0x1358 into house mailbox.
Opened mailbox0, transferred its three letters, then read bed and ticket letters
from pockets. At2026-10-24T19:14:22.431065Z and.431578Z, the capture build logged
bevy_render::slab_allocator: "Use-after-free: attempted to copy element data
for an unallocated key". Process continued; inspected letter screenshots and
saved inventories/attachments were correct. No claim this is harmless.

Evidence: %TEMP%/bevy-crossing-room-critters/
catalog-checks-191b6221bb294ac48586df52c3f72862/stderr-151316.log,
order-letter.png, ticket-letter.png, restart-after.json and mail-collected.json.
Read bevy-debug/systematic-debugging/QA skills; reproduce with synthetic saves
before assigning cause to game code or Bevy. No source changes made here.
