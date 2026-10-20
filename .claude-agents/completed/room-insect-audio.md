# Room-insect sounds and callback critters

Philip chose roadmap item 1 ("do 1") after texgen-reflection. Cicadas
(0x2F6..0x2F9), grasshoppers/crickets (0x2FF..0x304), firefly 0x30D, mole
cricket 0x313, mosquito 0x319 and frog 0x33B stay in pockets because their
profiles carry callbacks (room-insect sound via sAdo_RoomIncectPos, firefly
glow). Audit the callbacks and audio path, implement them through the existing
Nas engine/level sources, then allow placement. Agent: claude. Skills read
this session incl. gc-audio-pipeline. No tests; Temp-only previews; real save
hashed (baseline C15154C0...), never loaded.

## Source audit (ACGC-PC-Port jaudio_NES / furniture; ac-decomp logic)

- Callers: ac_sum_{minmin 64, tukutuku 63, higurashi 61, abura 60, syouryou 58,
  tonosama 59, koorogi 56, kirigirisu 55, suzumushi 57, matumushi 54}: ct
  standard_stop speed 0; mv (aFTR_CAN_PLAY_SE) re-inits the stop clip at 0.5
  when sAdo_RoomIncectPos returns nonzero. ac_nog_{kera 0x42, ka 0x43, kaeru
  0x41}: plain aFTR_rig (system repeat 0.5) plus the call. ac_sum_genji:
  repeat 0.5, segment-8 prim (255,180,0,prim_a_table[counter%68]) env
  (0,255,0). ac_sum_pet01 (0x7C, arg 62) is not a critter: out of scope.
- Na_RoomIncectPos: index = arg - 0x36; beat r27 = Nap_ReadGrpPort(FTR_INST, 2)
  in scene mode 2 (rooms) else (frame & 0x78) >> 3; 50 slots; new: countdown
  (rand&15)+OFFSET[index], bump loop with j=0 rescan, plays only when no
  sou_trg_se id matches SE_ROOM_INS_DATA (whose trailing 0 matches free slots);
  found: on a changed even beat, countdown 1 plays and redraws, then --.
  Sou_Insect_Lev_Cont each Na_GameFrame frees unreported slots.
- Rooms start seq 246 on FTR_INST (Na_SceneMode 2). Its group script loops
  delay1; 0xBE 01 (GRP_CMD_CALLBACK slot 1); beqz -> port2 = (port2+1)&15.
  AG.seq_callbacks are only set by AUDIOCMD_SET_CALLBACK, which no game code
  sends (system.c clears them), so the value stays 0xFF and port 2 stays 0:
  in retail the room beat never advances. Ended trigger slots reset id 0
  (Sou_TrgEndCheck), so registration is normally silent too. Net: room insects
  are quiet unless all six trigger slots hold other sounds when they register.
  The engine's 0xBE handling (no value change without a callback) matches.

## Implementation

logic::room_insect (exact kernel), logic::furniture::critter_callback
(replaces the placement exclusion), logic::furniture_rig::stop_frame,
Engine::trigger_ids, Sound::furniture_sequence/furniture_beat/trigger_ids,
player_room::insects (60 Hz ticks, seq 246 on entering any house, stopped
on leaving, slots cleared outside the player room, se_at positional chirps),
furniture::rigs Stopped/Firefly play modes and spawn_skinned_colored seeding
the firefly's segment-8 colours.

## Verification (Temp only; no tests)

Real save baseline C15154C0... unchanged; never loaded. Codex committed
fan/gong/slot-machine (0cd90f24..2f6222a3) in this checkout mid-task with
hunk-level staging; this task's edits stayed separate and were rebuilt on top.
Preview %TEMP%/bevy-crossing-room-critters, room with minmin, syouryou,
matumushi, genji, kera, ka, kaeru:
- Faithful run: seq 246 starts and plays; ports [-1,-1,0,0,0,-1,-1,-1], beat 0
  throughout (~980 ticks logged); 0 chirps.
- Temporary forced frame-counter beat (QA_TEMP_BEAT, removed): chirps 0x501,
  0x505, 0x515, 0x52D, 0x531, 0x535 at the cages; dense joint sampling shows
  stopped minmin/syouryou/matumushi clips replaying (15/14/5 pose changes),
  repeating rigs moving; audio nonfinite 0, 2/24 voices.
- Firefly: one glow material captured per spawn; cage-area brightness across
  four pulse phases unchanged (~0.1), so the small tail glow is not visually
  confirmed at this distance.
- Temporary logs and the seq-246 dump were removed before commit.
Gates: check, all-target clippy -D warnings, fmt, capture, default 25s,
release 2m47s.

## Limits

Placement/pickup of the newly allowed thirteen not re-driven through the
pockets menu (same conversion path as the verified fish/insects). No-disc not
rerun (insects return early without Sound; no save change). Audible retail
comparison and the firefly glow visual remain for Philip.
