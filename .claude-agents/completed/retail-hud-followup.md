# Retail HUD presentation follow-up

2026-11-01, codex. Philip selected handoff priority 2: audit and finish native clock/date, Bells and context prompts. Preserve the existing idle clock, pocket balance and F10 development overlay; reconcile prior completed notes with original C before changing behavior.

- [x] Audit original HUD source and current callers; record concrete gaps.
- [x] Implement source-backed corrections using runtime disc assets and existing input/state ownership.
- [x] Verify clock values/rollover, modal and intro visibility, Bells and F10; use isolated synthetic Temp data only. Widescreen side conversion source-reviewed; both-side runtime crossing not driven.
- [x] Sequential check/clippy/fmt and capture/default/release builds; no tests requested.
- [x] Update roadmaps/comms and complete task ownership. Explicit task-file publication follows; preserve unrelated handoff/.gitignore/save changes. M4 and Philip acceptance remain open.

## Implementation and original sources

Philip added missing fish/insect inventory menus and depositing/withdrawing money, then explicitly requested the native counter animation. He also asked whether the custom design menu worked; it is absent and no design-editor implementation was requested.

- `app/src/idle_clock.rs`: real-time presentation while PC pause stops world time, all-modal submenu allowlist, intro/demo suppression and centered fixed-height viewport coordinates. `dev_overlay.rs` no longer incorrectly claims the retail field has no clock. Native `m_banti.c` lines73/120/540; existing `logic/src/banti.rs`, digit rigs, palettes and art retained.
- `logic/src/critter_collection.rs` and `app/src/pockets_disc/collection.rs`: exact40-species orders, native8x5 anchors, active/inactive disc page frames/tabs, hand/name tags, native second icon palettes and tile0 cloth scroll. Native `m_inventory_ovl.c` lines74/88/113/1489/1885; `m_tag_ovl.c` lines77/105; `m_submenu_ovl.c`1467. `ecology.rs` records successful catches into the existing CatalogSave furniture bits. Generic possession and museum donation records stay separate; no save schema bump.
- `logic/src/pocket_money.rs`, `app/src/pockets/money.rs`: atomic whole-bag deposit/refusal and affordable100/1000/10000/30000 withdrawals, exact IDs2103/2100/2101/2102, cap99999, normal-condition guard. Native `mTG_itemNo_to_amount`, `mTG_make_money_sack`, `mTG_select_tag_decide_money`, `mTG_drop_item`, `mHD_drop_money`; no partial remainder or loss. Withdrawals require a free pocket and place the bag there; this adapts native held-bag ownership to the port's existing pocket transactions.
- Native `mTG_move_func`8080..8120 updates `disp_money` by signed amount/42 and snaps at42 ticks. The transient Counter reproduces those steps and input wait; actual wallet/items commit immediately, and saves never contain an intermediate visual value. Close/reload/app-exit cleanup discards the transient display state.

## Bounded verification

Temp evidence: `%TEMP%/bevy-hud-20261101`; all scenario saves are synthetic. Actual save baseline/final hash remains0495927D27B005E9FD0A4AED424EAAD883F56BBD8A9C4F5910167C1F9BBB1BD9. Actual settings and unrelated handoff/.gitignore changes preserved.

- Inspected native pockets, all40 fish and40 insect icons, names, last-slot cursor wrapping, tabs, wallet options and deposit target; inspected single Cricket entry after a real net catch, with all other entries undiscovered. Catch persisted in existing numeric collection bits; F9 retained the collection. Fish capture hook uses the same explicit successful-add boundary (source-reviewed).
- Bells12345: withdrawing100 and depositing10000 produced22245, emptied only the deposited slot, created exactly one100 bag and retained pocket_conds384 plus full loan/house/room/identity. Full-pocket withdrawal and wallet95000+10000 overflow refused with exact pocket/wallet/condition equality. Quest-conditioned bag could not enter the wallet target; cancel retained it.
- Final native10000 counter samples down11393/6633/5443/4015/2345 and up3059/8057/9247/10199/12345; screenshots show intermediate numbers and exact final values. F9 preserved wallet12345, original conditions and expected bag ownership.
- Native clock appeared outdoors; pause and map hid it; paused movement left the player transform fixed. F10 showed/hid only development labels. A synthetic process start at Nov30 23:59 crossed to Dec1 00:00 with correct weekday/12-hour display. Intro arrival/Porter screenshot had no idle clock root. Existing F9 does not apply clock_delta, so rollover was verified through process startup rather than assuming a changed F9 clock.
- Missing-disc fallback: readable compact fish/bug grids,100 withdrawal, F9 and complete numeric catalog/room/house/pocket/condition preservation. Expected missing-disc warning only; no panic.
- Source review by one reused subagent covered clock adapters, collection ordering/catch ownership, second palette, scoped cloth scrolling, money refusal rules and42-tick counter arithmetic/cleanup. Parent inspected changes and rendered evidence.

## Limits

Older catches were never recorded and cannot be reconstructed from donations or generic possessions. Full40-tick page-exchange motion and native held withdrawal-bag choreography remain separate presentation work. Both-side HUD crossing was not automated; the viewport conversion was checked against the actual fixed-vertical overlay camera. Audible retail comparison and Philip's visual/play acceptance remain open. No extracted art, dependencies, tests or real-save automation.

Final check/clippy(-D warnings)/fmt and capture/default/release builds passed; release5m36s. All owned previews closed and no pending automated input. Actual save final hash matches the baseline above. No tests added/run.
