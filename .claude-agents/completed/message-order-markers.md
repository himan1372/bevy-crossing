# Demo-order codes in messages

Philip: "do those" / "do" (2026-10-28), item 2 of the first-job plan. `mFont_CONT_CODE_SET_DEMO_ORDER_PLAYER/NPC0/NPC1/NPC2/QUEST` (codes 8-12, args `[index:u8][value:u16]`) tell the speaker's actor
what to do at that point of the text (the first job's "show the item now" is NPC0 index 1 value 4). `app/src/messages.rs::render` drops them. Keep them as page markers and let a caller read
the orders on the page now showing.

## Implemented (2026-10-28, claude)
- `app/src/messages.rs`: codes 8-12 (`SET_DEMO_ORDER_PLAYER/NPC0/NPC1/NPC2/QUEST`, args `[index:u8][value:u16]`) are kept in the page text as `markup::ORDER` + kind, index, value (high, low);
  `markup::parse` gives `Item::Order(kind, index, value)` (kind 0 player, 1-3 NPC 0-2, 4 quest); `strip` drops them.
- `app/src/msg_window.rs`: `DemoOrders` resource (`mDemo_Set_OrderValue` / `Get_OrderValue`; values persist until replaced). The reveal sets an order the moment the typed text reaches its code and logs it.
  The drawing and width code ignore `Order`. `DemoOrders::get` is for the first job's guide (`#[allow(dead_code)]` until then).
Gates: fmt, clippy -D warnings (workspace, all targets), capture build.
Verified (Temp, disc, synthetic save, Nook's HRATALK talk 0x082A/0x082E): the log showed `message order kind 1 index 0 value 0x0015`, `0x00ff`, `0x000b` at the pages where the source puts them (0x082A has 21, 255, 11; 0x082E starts 11 then 255),
none before the text reached them; no WARN/ERROR.
