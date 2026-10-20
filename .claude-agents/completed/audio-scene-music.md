# audio-scene-music — M4 scene arbitration

Requested by Philip after accepting letter sounds (2026-10-14).

- [x] Trace original shop tier, quiet player-house, outdoor hourly/season music and menu/dialogue volume policy.
- [x] Add pure scene policy to logic; wire current house/shop/menu state to the disc music engine.
- [x] Preserve hourly changes and safe dual-group fades; restore town on exit.
- [x] Compile, lint, format and build; record a short listening checklist.
- [x] Philip: town -> house -> town -> shop -> town, conversations and menu open/close, hour rollover.

Weather/event/furniture-player systems not present in gameplay remain named follow-ups. Do not infer missing state. Trigger lifecycle validation remains independent and pending; letters/pockets accepted.

Implementation: logic/src/scene_music.rs is pure policy; app/src/sound.rs applies
it in PostUpdate while InGame, including pause. Current location wins over
hourly music. Player house has no background song without a furniture player.
Shop tiers use 44/37/38/39; closing themes 79/80/81/82 start five minutes before
22/23/22/22 hours, preceded by the source's five-second quiet fade. Scene stop
0x168 and closing stop 0x13DE's low 0x3DE duration retained. The port has instant
doors, so song switches fade both handles rather than using a retail scene wipe;
accepted hourly transition duration retained. Clearing all mute bits precedes
new arrangements. Starting sequences resets group gain, so duck level is
reapplied after START_SEQ to both handles, including outgoing tails.

Conversation/shop transaction/quest panels use 70% over 30 frames; inventory,
letters and the port's pause menu use 50% over 2 frames. Strongest request wins.
Real-time gain ramps continue through Virtual-time pause and restore on close.
Fine, rain, snow and sakura policies reuse the existing translated masks; app
retains clear weather until simulation provides weather state. Ground season
alone does not select snow/rain. No new weather/event/house-furniture simulation.
Shop door-hours/part-time work and scene wipe remain separate gameplay follow-ups.

Sources: original m_bgm.c (room selection, shop_close_check, talk/pause volume),
m_shop.c (closing hours), m_inventory_ovl.c (pause 1/STOP), game64.c_inc
(Sou_BgmTenkiConv, Sou_BgmStart, Sou_Na_BgmStop, Na_BGMVolume), and current
nas_seq group/subtrack command handlers. Workspace check/clippy/fmt passed.
No tests or disc probes were added/run. Workspace build passed. Listening
check: restart; town -> house (town song fades out) -> town (current hour),
then shop (tier theme) -> town; open/close pockets and letters, start/end a
conversation, pause/unpause and confirm music returns to full level. F6 hour
advance inside must stay quiet/shop; exit must select the new outdoor hour.

Philip listening update: reports things work perfectly and requests the next
stage. Scene music/volume restoration accepted; named unavailable systems
remain follow-ups, not inferred implemented.
