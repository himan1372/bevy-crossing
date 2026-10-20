# player-footsteps — M4 player walking and running sounds

Requested by Philip 2026-10-14 after confirming the F8 villager voice comparison sounds good without hiccups.

- [x] Port original outdoor surface labels and Sou_WalkSe variants/volumes into logic/src/footsteps.rs.
- [x] Trigger sounds at actual walk/run animation frame crossings (1 and 9). Queue contacts only with actual horizontal motion; suppress stationary/collision-blocked, menu and warp sounds. Drop contacts on frames longer than 100 ms to avoid bursts after a stall. Character animation runs after a shared collision-resolved MovementSet.
- [x] Select town surfaces through the existing translated wpos2attribute resolver and use season for snow. Register each room's actual floor index in IndoorFloor; read SE_FLOOR_DATA as big-endian u16 values at runtime for houses, villager rooms and the shop.
- [x] Build/check: cargo check -p bevy-crossing, workspace clippy --all-targets -- -D warnings, workspace formatting, cargo test --workspace (609 passed, 20 ignored), and cargo build -p bevy-crossing passed.
- [x] Ignored real-disc sound probe passed for all 18 distinct outdoor/floor sound groups; samples remain in Windows temp/bevy-crossing-footsteps. The probe caught and corrected the initial byte-vs-u16 floor-table interpretation.
- [x] Philip listening check (2026-10-14): footstep sounds work fine; menus and pushing against walls are silent.

Implementation: app/src/character.rs emits foot-contact messages from the rendered player rig. app/src/footsteps.rs consumes them after Update and queues volume-scaled SE triggers; app/src/sound.rs shares play_effect with the disc probe. No sounds are synthesized or extracted into the repository. Without a disc, existing silent audio fallback is preserved.

Local qwen2.5-coder:7b drafted Sou_WalkSe's variant branch. Its output retained C global names and unrelated stubs; Codex corrected it to owned Rust state and checked the alternating/rare-sample behavior against the source.

Limits: shoreline evaluation uses a neutral wave phase until animated tides/waves exist; reverb remains absent in the mixer. The unimplemented optional extra bridge has no special sound override. Current player Shift-run uses retail RUN volume (0.8 outdoors / 0.72 indoors); the separate retail DASH sample offset is ported in logic but not activated by current locomotion. NPC footsteps are outside this task. Philip confirmed the footstep sounds work fine and menus/wall pushing are silent on 2026-10-14. Requested implementation and playtest are complete.
