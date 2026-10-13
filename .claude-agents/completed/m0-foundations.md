# M0 foundations (Philip: "do all of M0"; one bundled claim)
Covers the ROADMAP M0 code tasks: app-states, input-actions, assets-scaffold, ci-gate.
Philip asked that tests / game runs wait for his OK: code is compile-checked
(check, clippy, fmt) and kept in local commits until he approves running tests, then pushed.
- [x] ci-gate — .github/workflows/ci.yml runs check → clippy -D warnings → fmt --check → test on main.
- [x] assets-scaffold — assets/ tree + licensing README, AssetsPlugin with a Loading state and placeholder fallback.
- [x] app-states — AppState (Loading/Title/InGame) + PauseState sub-state; gameplay systems gated; HouseState/ShopLocation become InGame sub-states.
- [x] input-actions — Actions resource (one key map) + InputFocus (who owns input) + ordered, consumable Interact press.
- [x] Philip OKs running tests → cargo test + live BRP check → push.
- [x] slice-scope / ip-rename: proposals given to Philip in chat (decisions are his).

Status: done — tested (475 tests + live BRP) and pushed to main.
