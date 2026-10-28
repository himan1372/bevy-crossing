# HRATALK: Nook's day-after-part-time-job greeting

Philip: "hratalk" (2026-10-28). Open item from `completed/shop-start-audit.md`. Source: `m_event.c` (`mEv_UnSetFirstJob` turns FIRSTJOB off and HRAWAIT on;
`mEv_RenewalDataEveryDay` turns HRAWAIT off and HRATALK on at the next daily renewal; `mEv_ClearPersonalEventFlag`), `ac_npc_shop_common.c` `aNSC_start_wait`
(resident player: HRATALK wins over balloon, house talks and greetings; the flag is cleared when the talk starts; the talk is message `aNSC_get_msg_no(-1)` = 0x082A,
then `SAY_HELLO_APPROACH`, i.e. the keeper walks forward as for the greeting). The part-time-job arc that sets FIRSTJOB/UnSetFirstJob is not in the app
(`app/src/copper.rs:278`), so nothing sets the flag in normal play yet.

- [x] Kernel in `logic/` for the personal event flags (FIRSTJOB, HRAWAIT, HRATALK): unset-first-job, daily renewal, clear, take-talk.
- [x] Saved optional per-player state in the save; renewal on date change.
- [x] Nook: HRATALK first in the arrival order, message 0x082A, flag cleared at talk start, forward walk.
- [x] Gates, Temp QA with a synthetic save (hra_wait set, then date change; hra_talk set), docs/comms, release claim, commit/push.

## Implemented (2026-10-28, claude)
- `logic/src/personal_events.rs` (new): `PersonalEvents { first_job, hra_wait, hra_talk }` with `set_first_job`, `unset_first_job`, `renew_daily`, `clear`, `real_arbeit`, `arbeit`,
  `take_hra_talk`, and `HRATALK_MESSAGE` = 0x082A. Two own tests pass (the job leads to one greeting the next day; clear drops every flag).
- `app/src/save.rs`: `PlayerIdentity` gains the flags (`events`, saved as three booleans, absent in old saves = all off) and `events_day`; `renew_personal_events` runs
  `renew_daily` once per calendar day (`mEv_RenewalDataEveryDay`).
- `app/src/shop_nook.rs`: first in the arrival order (before balloon, house talk and greetings) `take_hra_talk` starts message 0x082A, clears the flag as the talk starts
  and sets the forward walk (`SAY_HELLO_APPROACH`).
- Still nothing in the app calls `set_first_job`/`unset_first_job`: the part-time job arc (`copper.rs:278`) is not played, so the flag is only reachable by a save edit or
  once that arc exists. This task builds the receiving end only.
Gates: fmt, clippy -D warnings (workspace, all targets), capture build; `cargo test -p rustimal-logic personal_events` (the kernel's own tests only).
Verified (Temp, disc, synthetic tier-0 save with a loan, game clock 15:00, fresh process per case): `hra_talk` set -> first message 0x082A ("Oh! <name>!!! Very good! Great
timing!", then the Happy Room Academy talk pages, screenshots read), Nook at the walk limit z -2997.5; `hra_wait` set with `events_day` 0 -> renewed on load, same 0x082A;
`first_job` set -> the ordinary greeting 0x1091 (no talk while the job runs). No WARN/ERROR.
Not verified: the end of the long 0x082A talk and the saved clearing of the flag (F5 never saved because the talk kept focus for 45 presses; the clear is `mem::take` at talk start);
the follow-up conversation after it.
