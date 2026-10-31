# Native NPC movement speed

Philip accepted the Nook house flow on 2026-10-31, but reported Nook and other
NPCs moving too fast. Match original movement and animation rates.

Design: use original aNPC_spd_data walk1/run3, Actor_position_move's 0.5 factor
per 60-Hz tick, and 40 game units per world cell. Shared outdoor caps are
0.75/2.25 world units/s. Intro Nook and his player follower use that run cap.
NPC run animation uses the native locked 0.5 frames/60-Hz tick; keep villagers'
existing 0.5 rate and lower their run-selection threshold with the new caps.
Retain existing steering, schedules, timers, special shop speeds and player input.

Authority: ACGC-PC-Port src/actor/npc/ac_npc_data.c_inc (aNPC_spd_data),
ac_npc_move.c_inc (aNPC_setupAction/position_move), ac_npc_anime.c_inc
(aNPC_set_anime_speed), game/m_actor.c (Actor_position_move). Pinned Rustimal
movement.rs has no native speed table; keep that kernel unchanged.

Plan: claim; change shared caps/intro/run playback/threshold; sequential
check/clippy/fmt; isolated capture measurements and visuals; default/release;
record evidence, complete claim, explicit-file commit/push. No tests requested.
Preserve unrelated .gitignore/real save; all QA in Temp. Philip acceptance open.

Implemented in app/src/npc.rs, intro_house.rs and character.rs. Shared constants
prevent the guide and residents drifting apart; run selection uses the midpoint
between caps (1.5), so the slower run still plays a run clip. Player-controlled
movement, existing NPC steering/schedules/timers, and special shop translation
rates retain their previous behavior. Shopkeeper run playback also uses the
native default0.5 rate. This is a speed correction, not a full NPC-AI migration.

Verification: check/clippy(-D warnings)/fmt/capture/default passed. Release
passed (5m19s). No tests added/run. Temp/bevy-npc-retail-speed-20261031 contains only
synthetic scenario saves, helper scripts, measurements and inspected Nook run
captures. Nook's tour completed with median speed2.263 against target2.25.
All six villagers sampled for45s: one-second windows walk median0.74997 (646
samples), run median2.24929 (190 samples), maximum2.29262 including query timing
jitter. Five villagers were observed running. No frame-rate edits/teleports used
in those movement samples. Nook run-pose captures inspected; Philip's motion
acceptance remains open. Real save hash C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA unchanged.

Complete on base fe1a3b2e: all gates/builds passed, no tests. Fresh interactive
station preview reopened in the same Temp folder (owned-pid.txt/log-name.txt).
Philip's speed acceptance remains open; only task files committed/pushed.
