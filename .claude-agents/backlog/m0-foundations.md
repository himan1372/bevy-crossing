# M0 foundations (Philip: "do all of M0"; one bundled claim)
Covers the ROADMAP M0 code tasks: app-states, input-actions, assets-scaffold, ci-gate.
Philip asked that tests / game runs wait for his OK: code is compile-checked
(check, clippy, fmt) and kept in local commits until he approves running tests, then pushed.
- [ ] ci-gate — .github/workflows/ci.yml runs check → clippy -D warnings → fmt --check → test on main.
- [ ] assets-scaffold — assets/ tree + licensing README, AssetsPlugin with a Loading state and placeholder fallback.
- [ ] app-states — AppState (Loading/Title/InGame) + PauseState sub-state; gameplay systems gated; HouseState/ShopLocation become InGame sub-states.
- [ ] input-actions — Actions resource (one key map) + InputFocus (who owns input) + ordered, consumable Interact press.
- [ ] Philip OKs running tests → cargo test + live BRP check → push.
- [ ] slice-scope / ip-rename: proposals prepared for Philip (decisions are his).
