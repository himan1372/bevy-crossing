# audio-action-coverage — M4 playable action sounds

Philip accepted scene music and requested continuation (2026-10-14).

- [x] Source-map current house/shop doors, rod/net/catch/putaway, pickup, quest feedback and existing shop/mail triggers.
- [x] Wire missing source-backed actions at the available gameplay timing; document unavailable retail animation/fanfare flows.
- [x] Add nearby NPC foot contacts using current movement, surface lookup and original variant rules.
- [x] Compile, lint, format and build. No tests unless requested.
- [ ] Philip: doors both directions, rod cast/bite/reel/escape/catch, net swing/catch, pickup/putaway, quest accept/reward and nearby villagers.

Keep accepted footsteps/voices/UI/music. Do not create a generic beep for actions that are silent in the original. Record action -> ID -> source/timing evidence here. Ambience is the following task.

## Action evidence and current timing

| Action | Sound / source | App timing and limits |
|---|---|---|
| House entry/exit | 6/7/8/9; ac_house_move/ac_my_house_move | Frames 2/8/33/40 entering and 10/14/35/50 leaving; 0.5 frame/tick, start 1. Cues run from successful traversal. New traversal cancels previous scheduled tail. Door visuals/wipes and house-size animation variants remain unimplemented. |
| Upgraded shop doors | 44B open / 44C close; ac_conveni/super/depart setup_action | Open at traversal, close after 30 ticks (opening span). Retail proximity hold behavior awaits actor animation. |
| Cranny door traversal | ac_shop_open_wait/pl_into_wait | Silent while its door is already open. Business-hours opening/closing actor state is not implemented; no house-latch sound substituted. |
| Rod cast / no water | 109 / 445; m_player_main_cast_rod / air_rod + m_player_sound | Successful cast / unsuccessful air stroke. |
| Bobber landing / retrieval | 10B / 10C; ac_uki_cast / set_proc_hit | Bobber spawns on water immediately, so landing accompanies spawn; retrieval accompanies despawn. No cast arc currently exists. |
| Reel back / fish lift | 10A / 11A; m_player_main_collect_rod/fly_rod; ac_gyo_test status 6 | Reel press / successful inventory catch. Existing ITEM_GET retained. Instant catch collapses retail animation spacing. |
| Nibble / bite / escape | ac_uki_touch/set_proc_bite; ef_turi_mizu -> hamon/hane; ac_gyo_test | No invented notification beep. Original level 24 runs during hooked fight (gyo_status 4), a state absent from current instant reel/catch. Follow-up depends on reel-fight gameplay. 11A is fish lift, not escape. |
| Net swing / catch | TOOL_FURI 5A / TOOL_GET 5D; m_player_main_swing_net + m_player_sound | Swing press / inventory capture. Corrected old 5C AMI_HIT, which belongs to obstacle impact; obstacle collision is not currently modeled. |
| Tool equip/change/putaway | GASAGOSO 69; m_player_main_takeout_item/putaway_rod/putaway_net | Actual equipment changes only; repeating the same selection is silent. |
| Pick up | ITEM_GET 40; m_player_main_pickup frame 10 | Existing successful pickup hook retained. Separate frame-20 storage rustle requires a pickup/putaway animation phase. |
| Quest choices | Existing choice open C / confirm D / cancel 31 policy | Existing modal offer/turn-in panels. No separate quest-completion fanfare invented. |
| Quest parcel/handoff/reward | GASAGOSO 69; m_player_main_give / recieve_putaway | Parcel accepted, requested item handed over, or owed item actually received. No receipt sound for a reward that remains owed. Bell-only owed payment has no invented register beep. |
| Shop / letters | Prior accepted sound mappings | Preserved; see ui-mailbox-sounds. |
| Villager foot contacts | npc_1_walk1/run1/walk_ki1/walk_do1 frames 1/9; ac_npc_anime/sound and Na_NpcWalkSe[Room] | Actual rig contacts plus real displacement, no warps/standing/menu steps. Current surface/floor table and variants reused. Radius 540 game units (13.5 world units), original distance curve; optional gain 0.68 outside / 0.612 inside. General stereo positioning/reverb remains stage 6. |

Workspace check, all-target clippy, formatting and build passed. No tests or
probes added/run. Listening: restart; enter/exit your house and a villager's
house; try upgraded shop doors (Cranny stays open/silent), select/put away
rod/net, cast on water or air, retrieve/catch, net swing/capture, quest choice
and handoffs. Stand near a walking villager and back away; standing, menus,
warps and distant villagers should produce no footstep sounds.
Player footsteps retain their accepted gain and collision/menu rules.
Standalone NPC contact timing was checked against all four source animation
metadata tables; no copied disc audio/assets were written to the repo.
