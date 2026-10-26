# Password service follow-ups: send, Famicom, magazine, mini card-e

Left over from shop-password-service (see completed/). Unclaimed.

- Send direction (Other things choice 1): msgs 300/301, `m_passwordMake_ovl`
  recipient town/name entry (8-char fields), item picker restricted to
  `check_possession_item_make_password` slots, same-town refusal (302), confirm
  (305/306), `make_password(type 4, hit 1, ...)` consuming the slot and showing
  the code lines. Codec and line helpers already in `logic/src/password.rs`.
- Famicom (type 0, `pswd_famicom_list`), magazine (type 3, hit-rate table with
  RANDOM_F), mini card-e (type 5) redemption.
- Retail character-grid entry window; hand-over animation timing for the present.
- Foreign-visitor refusal (321) once visitors exist; furniture-form tool/diary
  checks if needed.

## Send direction implemented (2026-10-26, claude)

Other things now shows all four disc answers: Turnip Prices, **Send a code**,
Say code, return. Send follows `aNSC_pw_*` with the disc's own message links:
nothing tradeable (plain-condition pocket items `present_user` accepts) -> 300
and the menu; else 301 -> receiver window (`m_passwordMake_ovl` as typed fields:
town, Tab, player name, Enter once the town is not empty, Esc backs out to 308) ->
same town as ours: 302 and the window again; else 303 (That's right -> 304,
You're wrong -> 301, Maybe not -> 308) -> pockets in a new `sending` mode (Send/
Quit on eligible items; closing without a pick -> 305 Yes -> 304 / No -> 308)
-> 306 (That's fine -> 307, You're wrong -> 301, Maybe not -> 308). On
confirmation the item leaves the pocket, `make_password(type 4, hit 1, player,
town, item)` is shown on two lines in 307 and the talk ends. The picked slot is
revalidated at that moment. `#` types the retail 0xD1 symbol. Refusal replies
(gift limit, full pockets, no item, declined) now end on the message's own menu
instead of asking again. Fixed: the key closing the pockets/picker also answered
the next question (consumed in the handler).

Not done: character-grid entry and hand-over animations; Famicom, magazine and
mini card-e codes; Timmy's department-store counter uses the same flow through
his message bank but follow-up links are Nook's numbering untested; foreign
visitor refusal. A code made here is for another player/town by design (same
town is refused), so redeeming it needs that player; round trips were checked
with generator codes for the QA player (incl. the # symbol: spooky paper).

Verification (Temp, disc image, synthetic save; no tests): receiver window and
names, 303 text ("send this to Zed in Springs"), picker with Send/Quit, 306
text, 307 code lines identical to a scratch run of the same codec (Springs/Zed/
blue bed: WmuO#aCb7UWmWJSU%2Z7XB6Txbhz), pockets [0, 0x1008, present 0x1358],
conds and bells unchanged otherwise; same-town refusal; Esc back; picker cancel
-> 305 -> Yes -> pick again; You're wrong -> 301 -> window -> Esc -> 308; only
present-tagged item -> 300 + menu. Logs clean. Real save hash unchanged.
