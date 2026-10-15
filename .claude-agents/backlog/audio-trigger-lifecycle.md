# audio-trigger-lifecycle — next M4 audio implementation

Priority: next coding task after the 2026-10-14 roadmap audit. Philip can
recheck UI/mailbox mappings independently. Parent audio-from-disc remains
under its existing claim; claim this task separately before implementation.

Problem: app/src/sound.rs currently rotates ordinary sounds through slots
0–5 without checking completion, priority or singleton flags. Dedicated MONO
slot 14 now protects the register, but other long effects remain vulnerable.
Controlled disc probing reproduced ordinary register truncation from 1.163 s
to 0.437 s; the MONO version survives the same traffic.

Sources:
- logic/src/audio.rs: SeId, TrgSeManager, CommandRing, priority helpers.
- Original game64.c_inc: Sou_TrgStart, Sou_TrgEndCheck, Sou_TrgMake.
- Original audiocommon.h command definitions and Nap_Process1Command.
- app/src/sound.rs: play_effects/play_effect and probe_register_tail.
- logic/src/nas_seq.rs: process, sub_param and sequence/subtrack ports.

Plan:
- [x] Audit the translated manager against original behavior and current command opcodes; do not assume its older command ring can be sent unchanged.
- [x] Expose/reuse the needed safe logic APIs; keep engine rules in logic/.
- [x] Track pending, playing and completed slots (retail completion uses subtrack port 5); avoid freeing a just-submitted sound before its script starts.
- [x] Prefer free slots, apply original priority/replacement rules when full, honor singleton IDs, and retain dedicated MONO handling.
- [x] Preserve player footstep/UI volume and keep game/audio-thread access bounded.
- [ ] Check overlapping long effects, cursor bursts, footsteps, mono isolation and natural completion with fresh additive-mixer buffers; real-disc checks stay ignored and write clips only to temp.
- [ ] Run the appropriate Rust gate and give Philip one short overlap/listening check. Record remaining limitations in ROADMAP/comms.

Done when: ordinary effects no longer overwrite one another merely because
a round-robin index wrapped; priority replacements are intentional, singleton
events do not stack, and completed sounds release their slot without a stuck
or prematurely cut tail. A successful non-silence probe alone is insufficient.

Implementation (2026-10-14, Codex): Engine::trigger_effect submits through the
existing TrgSeManager; live app no longer rotates a slot counter. Corrected
trigger opcodes (volume 01, pitch 04, pan 03, reverb 05, port 06), setting all
parameters before ports. Each renderer update observes consumed request port 1
and busy port 5; a pending replacement cannot inherit the previous busy bit.
Unstarted idle requests get half a second before cleanup; sequence stop/restart
resets allocation. Priorities preserve oldest-slot/equal-or-higher rules; unknown
priority indices retain the safe existing fallback/free-slot policy. Header's
stale 120-entry description corrected to match the 128-entry table.

Workspace check, all-target clippy, formatting, build and patch whitespace checks passed. No tests/probes rerun in this
continuation; old raw probe helper is retained under cfg(test) for comparison.
Qwen local review used; suggestions were not actionable. Pending: real-disc
natural completion and overlapping effects checks, then Philip's listening
acceptance. Do not infer successful playback from compile checks alone.
