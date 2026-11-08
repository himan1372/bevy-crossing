# Next30 tasks 5 and 6: player cottage and island naming

2026-11-07, codex; explicitly selected by Philip. Source baseline ee0ea5db, native C at ACGC-PC-Port (m_home.c default cottage, ac_my_indoor.c SCENE_COTTAGE_MY, ac_npc_sendo_move.c_inc first visit, m_ledit_ovl/m_editor_ovl island entry). One reused subagent implemented naming and reviewed cottage integration.

Implemented:
- Native large cottage shell: 10x10 model, 8x8 usable, floor/wall13 and four aloha mannequins26/27 at(3,3),(6,3),(3,6),(6,6). Preserve the accepted south-facing exterior door.
- Reuse editable PlayerRoom transactions by swapping independent room records. Optional island_cottage retains furniture layers, owner-cell storage, switches, music, diary and decoration. Mainland main/upper/basement remain separate. Rebuild furniture/shell/stairs on site changes; fix furniture collision centres for larger rooms. Cached cottage turnips participate in the existing spoil pass.
- First crossing Yes:3079 -> original shi_win naming window (required nonempty name, up to8 characters/80px) ->30BE. Persist identity.island_name; repeat3076; FREE_STR0 uses accepted island name. Gender-selected fixed messages; no double-shifted song IDs. Load clears only owned transient naming/voyage state and places boat at the loaded exterior endpoint.
- Compatible optional save fields, no version bump, dependencies or assets. No tests added/run.

Verification so far:
- Sequential workspace check, Clippy all-targets -D warnings, fmt --check pass. Capture build2m08s and default build1m58s pass. Release passes (9m10s); it reports an unrelated unused tree_palette warning in the other agent's disc_town work.
- Synthetic Temp/bevy-cottage-naming-20261107, BRP25742 via supported BRP_EXTRAS_PORT. Foreign preview15702/15703 untouched.
- Inspected native cottage screenshot and original naming window screenshot. Cottage native defaults match numeric IDs6164/6168 and decoration13/13. Backspace pickup removes only cottage cell99 and adds shirt0x241a; mainland snapshot unchanged. F5 inside/F9 preserves both rooms and returns to cottage exterior.
- Kapp'n initial No retains absent name, empty F4 refuses, palmbay accepted through actual UI, full crossing saves it, F9 preserves name and numeric possessions/rooms and island boat endpoint, return/repeat crossing uses3076 without naming again. Evidence naming-*.json/png and preview logs.

Final bounded checks pass:
- Actual cottage exit/reentry retains exact records. Placed the picked aloha shirt through native pockets:3 ->4 room actors, exact pocket removal, mainland untouched. F9/reentry retains it; final exit succeeds after allowing the full facing turn (the first short QA turn timed out, no source change needed).
- Synthetic upgraded home size3 with basement and separate upstairs/basement records: cottage has no house stairs. Distinct nonzero music-box/switch records remain independent across entry/exit and F9.
- Fresh native restart, cottage entry and F5 preserve exact cottage/mainland/floor records and palmbay. Missing-disc restart/F5 also preserves them exactly. Inspected native-restarted and no-disc screenshots.
- Owned previews closed, real save SHA256 remains0495927D27B005E9FD0A4AED424EAAD883F56BBD8A9C4F5910167C1F9BBB1BD9. Evidence stays in Temp, including numeric-*.json and naming-cottage-ui-*.json. BRP25742; two early preview exits had no panic log and checks were rerun after relaunch.

Limits: existing owner-cell storage/diary/move/rotate transaction engine is reused and source-reviewed; this task did not exhaustively replay every furniture interaction, banned-name/gender variant, or turnip rollover. No tests requested/run. Philip visual/audio acceptance and M4 remain open.
