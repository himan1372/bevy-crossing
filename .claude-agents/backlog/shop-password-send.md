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
