# Part-time job: core state and restrictions

Philip: "part time job" (2026-10-28), slice chosen: core state + restrictions (not the stages, not the intro demo). Source: `m_event.c` (SetFirstJob/UnSetFirstJob/CheckFirstJob/CheckArbeit/CheckRealArbeit,
done in `completed/shop-hratalk.md` as `logic/src/personal_events.rs`), and every `mEv_CheckFirstJob / Arbeit / RealArbeit` site. Intro demo that starts the job stays deferred, so a debug/save
switch starts it. The job ends via `unset_first_job` which feeds HRATALK.

## Implemented (2026-10-28, claude)
State: `logic/src/personal_events.rs` (`PersonalEvents`, saved in `PlayerIdentity.events`, from `completed/shop-hratalk.md`). The job's flag is `identity.events.first_job`; with no intro demo yet,
it is switched on by editing the save (`"identity": {"events": [true, false, false]}`). Nothing calls `unset_first_job` yet (the stages do), so the job does not end by itself.
Restrictions wired (each source site checked):
- `mEv_CheckFirstJob` in `mNPS_schedule_manager`: every villager is forced outside (`npc_ai::global_schedule_override`, the existing kernel; `npc.rs::schedule_now` takes the flag at its three callers).
- `mEv_CheckFirstJob` in `mSP_ShopOpen`: the shop is open whatever the hour, a renovation or a bargain closure (`shop/schedule.rs::open_now`; `shop_enter`, the Cranny door and the building lights use it).
- `mEv_CheckRealArbeit` in `m_bgm.c` (shop close check): the shop theme never switches to its closing arrangement (`logic/src/scene_music.rs::select_with`, `sound.rs::update_music`).
Not applicable in the app yet (no code to gate): rain (`ac_weather.c`, no weather simulation), the train (`m_train_control.c`), Feng Shui scoring (`m_huusui_room.c`), mark room (`m_mark_room.c`),
the mayor's lighthouse/Soncho line (`m_soncho.c`, `m_field_make.c`), the watch-my-step coin sound, notices/tags (`m_notice_ovl.c`, `m_tag_ovl.c`, planting "job" tag) and the errand-progress hooks
(`m_npc.c` letters, `m_quest.c`), plus Copper's hint (`copper.rs:278`) — all belong with the stages. The intro demo that starts the job (`ac_intro_demo.c`, `m_actor.c`) and the 11 errand stages are not built.
Gates: fmt, clippy -D warnings (workspace, all targets), capture build; `cargo test -p rustimal-logic scene_music` (the new test plus the module's two existing ones).
Verified (Temp, disc, synthetic tier-0 save, game clock 23:00): without the job the shop door stays shut (player stays outside); with `first_job` it opens and the player goes inside. No WARN/ERROR.
Not driven: villagers staying outside at night, and the shop theme at closing time (compiled; the music rule has its kernel test).
