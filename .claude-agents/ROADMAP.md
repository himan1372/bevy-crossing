# bevy-crossing roadmap (revised 2026-10-14)

Claimable tasks live in `backlog/` (one file per task). Claim in `claims.jsonl`
before starting work; announce claims, completions, blockers,
and questions on `comms.txt` on the **main** branch (append-only).

## Open runtime bugs
- [x] **fix-shop-entry** — done (claude, 60e5688): door check measured to a point 0.7 out from the wall; playtested.
- [x] **fix-letter-notify** — done (claude, 969ca92): mail list child was Visibility::Visible over a hidden panel; playtested.

## Pipeline / docs
- [x] **document-glb-pipeline** — done (cosmo) — Blender -> .glb for Bevy 0.19: export settings, naming, scale, verification checklist → `.claude/skills/asset-pipeline/SKILL.md`.

## Code health
- [x] **logic-clippy-cleanup** — done (claude): `cargo clippy --workspace --all-targets -- -D warnings` passes (with and without `--features remote`); 475 tests pass.
- [x] **workspace-fmt** — done (claude): `cargo fmt --all --check` clean. 38 hand-laid decomp tables and the two generated modules (fish/insect tables) are `#[rustfmt::skip]`. The full rust-workflow gate now passes.

## Direction (Philip, 2026-10-12): OpenMW model

bevy-crossing rebuilds the **engine** in Bevy. All game **content**
(models, textures, sounds, maps) loads at runtime from the player's own
Animal Crossing GameCube disc image. This replaces the earlier
"make our own art" plan: art-direction v2 is retired, and
`tools/concept-art/` and `docs/art-direction.md` are leftovers.

**Hard rule:** no data copied from the disc goes in any repo, ever. That
means no extracted models, textures, audio, `.bin` dumps, or test files cut
from the disc. Tests use made-up data built inside the test. Tests that need
a real disc are `#[ignore]` and read the disc path from `BEVY_CROSSING_ISO`;
CI never has a disc. The game must still start without a disc (placeholder
art plus a clear message).

## Vertical slice plan (revised 2026-10-12)

**Philip's scope:** a playable area with animations, drawn from the player's
own disc, and the main game loop of Animal Crossing (GameCube):
**walk/run, talk, get quests, and an inventory.**

**Slice = done when:** pointed at the player's disc image, the game opens
into a town drawn with the original models and textures (no placeholder
capsules or boxes). An animated player walks and runs, talks to animated
villagers, accepts a quest, carries items in a 15-slot pocket inventory,
and completes the quest for a reward. Steady 60 fps in a release build on
Philip's PC. Without a disc, the same loop runs on placeholder art.

**Build on the code Philip already translated.** `logic/` is the Rust
translation from **rustimal-crossing** (`rust/src/`). Gameplay tasks
**must** call the named `logic::` module instead of writing new logic in
`app/`. After the direction change, `logic::graphics` is now the core of
rendering. It holds the translated emu64/GBI display-list interpreter, the
texture conversions and the `GxBackend` trait. The disc's models are
N64-style `Gfx` display lists, and Bevy implements the GX backend.

**Port existing code from rustimal-crossing instead of rewriting it:**
- `rust/src/lib.rs` (`Disc`, `FstFile`, `decode_yaz0`): reads ISO/GCM/CISO,
  checks the GCM header, parses the disc's file table (FST), finds the DOL
  size, and decodes Yaz0. `src/pc_disc.c` is the C version, for reference.
- `.claude/skills/asset-pipeline/SKILL.md` + `tools/gen_runtime_assets.py` +
  `src/pc_assets.c`: the asset table (~2,500 assets, each with ROM offset,
  size, source DOL/REL, and byte-swap type). The assets are packed inside
  `main.dol` and `foresta.rel.szs`, not stored as separate files on the disc.
- `logic/src/dvd.rs`, `aram.rs` and `gbi_runtime.rs` are C-ABI shims for the
  C port; bevy-crossing should use plain Rust APIs instead.

Task slugs below become `backlog/<slug>.md` files when they're ready to
claim. **(P)** = needs Philip (decision or playtest). **[blocked: x]** = do x first.
Each milestone ends with a Philip playtest.

### M0 — Decisions and foundations
- [x] **slice-scope** — Philip set the scope (2026-10-10; graphics source changed 2026-10-12).
- [ ] **ip-rename** (P, deferred by Philip 2026-10-10: keep names for now) — Nintendo names in `app/src/shop.rs` and the villager list. Revisit before any release.
- [x] **app-states** — done (claude): `AppState` (Loading / Title / InGame) + `PauseState`; HouseState/ShopLocation are sub-states.
- [x] **input-actions** — done (claude): one action map, a single input-focus rule, consumable presses (`app/src/input.rs`).
- [x] **assets-scaffold** — done (claude): `app/assets/`, loading gate, placeholder fallback. (Now holds only our own UI and placeholder files; disc content never goes here.)
- [x] **ci-gate** — done (claude): `.github/workflows/ci.yml` runs the full gate on every push to `main`.
- [x] ~~**art-direction**~~ — replaced 2026-10-12 by the disc-asset direction; v2 sign-off no longer needed.
- [x] **disc-setup** — Philip OK'd the proposed default (2026-10-12). Was: Philip decides three things. Which disc formats to accept: ISO/GCM/CISO as in rustimal-crossing, with RVZ/GCZ maybe later. Which regions: NTSC-U `GAFE01` only to start? How the player points the game at the disc: a settings file, a `--disc` flag, a file picker, or a `rom/` folder next to the exe. Proposed default: `GAFE01`, ISO/GCM/CISO, look in `rom/` first, then use a path from the settings file.

### M1 — Main loop on the translated code (placeholder art) — DONE 2026-10-10 (Philip playtested; U-turn drift fixed in 6bb80f7)
- [x] **logic-expose** — done (claude): slice modules `pub` in `logic/src/lib.rs`, one test each.
- [x] **walk-run** — done (claude, playtested; walk ~3.9 u/s, run 6.8 u/s): `logic::player_move`, Shift = run.
- [x] **pockets-inventory** — done (claude, playtested): 15-slot pockets on `logic::inventory`.
- [x] **talk-system** — done (claude, playtested): `logic::talk_request` / `talk_topics` / `dialogue_topics`.
- [x] **quests-full** — done (claude, playtested): `logic::quest` / `quest_gen` / `request_selector`.
- [x] **villager-routines** — done (claude, playtested): `logic::npc_ai` / `behavior` / `movement`.
- [x] **game-clock** — done (claude): `app/src/clock.rs` on `logic::game_time`.

### M2 — Reading the disc (pure Rust, no Bevy; testable without a disc)
- [x] **disc-reader** — done (claude, crate `disc/`; passes on Philip's disc): — open ISO/GCM/CISO, check the GameCube magic number and game ID, parse the FST, and find and read files by path. Port of rustimal-crossing's `Disc`/`FstFile`. Tests build a tiny made-up GCM in memory. [blocked: disc-setup]
- [x] **yaz0** — done (claude, `disc/src/yaz0.rs`): — Yaz0 (`.szs`) decoder, port of `decode_yaz0`. Tests round-trip through a small encoder written for the test.
- [x] **dol-rel-image** — done (claude, `disc/src/rom.rs`; 72,924 REL data pointers resolve on Philip's disc): — extract `main.dol` and decompress `foresta.rel.szs`; map the DOL and REL sections so an asset's ROM offset resolves to bytes. [blocked: disc-reader, yaz0]
- [x] **asset-table** — done (claude, `disc/src/assets.rs` + `disc/data/*.sym` from ac-decomp symbol maps via `tools/gen_symbols.py`; 39,861 assets, offsets match rustimal-crossing on Philip's disc). Byte-swap helpers dropped: consumers read big-endian directly. Was: — port the asset table from rustimal-crossing (`pc_assets.c` / `gen_runtime_assets.py`) into Rust: name → ROM offset, size, source (DOL/REL), swap type. It holds metadata only, no disc bytes. Plus byte-swap helpers (u16 / u32 / `Vtx`). [blocked: dol-rel-image]
- [x] **disc-loader-app** — done (claude, `app/src/game_disc.rs`: `--disc`, `settings.json` `disc_path`, then `rom/`; ~0.5 s load on Philip's disc; notice + placeholder art when missing). Was: — Bevy side: find and open the disc at startup as decided in disc-setup. If it's missing or wrong, show a clear message and fall back to placeholder art. Expose the disc to the renderer as an `AssetSource` or resource. [blocked: asset-table]

### M3 — Rendering the original assets
- [x] **gx-backend-bevy** — done (claude: `app/src/disc_gfx.rs`, `disc/src/gx_texture.rs`, `GxBackend::set_state` hook in logic; gaps listed in completed/gx-backend-bevy.md). Was: — implement `logic::graphics::gx_backend` for Bevy: run display lists through `logic::graphics::interpreter`, turn the decoded vertices into Bevy `Mesh`es and the converted textures (`logic::graphics::texture`) into `Image`s, and cache them per asset. [blocked: disc-loader-app]
- [x] **model-viewer** — done (claude; Philip OK'd 2026-10-12): `--view <model> [--pal <palette>[:i]]`, orbit/zoom, [ ] palettes, V vertex colours. Was: — debug scene (F-key or `--view <asset>`) that loads one asset by name (a tree, then a building) and orbits the camera around it. This is the first proof for Philip that disc rendering works. [blocked: gx-backend-bevy]
- [x] **tev-materials** — done (claude: `app/src/n64_material.rs` + `n64_combiner.wgsl`; gaps in completed/tev-materials.md). Was: — turn the GameCube's colour-combiner settings (TEV, `logic::graphics::combine`) into Bevy materials. Start with texture × vertex colour, then add the modes the town actually uses, then alpha test/blend and fog. [blocked: gx-backend-bevy]
- [x] **town-from-disc** — done (claude; river mouth confirmed fixed by Philip 2026-10-13): retail generator, disc acre templates with seasonal textures, collision from the templates. Calendar palettes are complete (completed/town-from-disc.md); remaining migration is backlog/retire-town-gen.md. Was: — draw the town ground, cliffs, river and beach from the original acre/field models for the generated town (`logic::field_gen` / `town_gen`), replacing the per-cell boxes. Collision stays on `logic::collision`. [blocked: tev-materials]
- [x] **world-objects-from-disc** — done (claude): trees, rocks, houses, player houses, public buildings, mailboxes (ac_mailbox) and field items (bg_item tables: fruit, shells, fences, notice board, flowers) from the disc. Was: trees, rocks, flowers, houses, shop and mailbox from the disc. [blocked: tev-materials]
- [x] **skeletal-animation** — done (claude, `app/src/skeleton.rs`: cKF skeletons as Bevy skinned meshes, cKF_KeyCalc Hermite port). Was: research first: how the original stores character skeletons and animations (cKF-style keyframes in the decomp; `logic/` has no translation yet). Then decode them and drive Bevy transforms. May need more decomp code translated first. [blocked: gx-backend-bevy]
- [x] **player-from-disc** — done (claude, `app/src/character.rs`: cKF_bs_r_boy_1 with wait/walk/run, face and shirt from forest_1st.arc). Was: player model with idle / walk / run animations matched to the speed from `logic::player_move`. [blocked: skeletal-animation]
- [x] **villagers-from-disc** — done (claude: npc_draw_data_tbl models, shared npc animations, blinking/talking, greeting). Was: villager models and animations per species/villager. [blocked: skeletal-animation]
- [~] **render-look** — camera/follow, calendar palettes, colour-space/fog corrections and clock-driven lighting are implemented. Combined comparison against Philip's retail game still needs sign-off; town-from-disc dependency is complete. No curved horizon (that is Wild World, not GameCube).
- [x] **render-perf** — done (claude, 2026-10-13): measured with vsync off (`BEVY_CROSSING_NO_VSYNC=1`), 1,704 meshes: release ~300 fps (3.3 ms), dev ~260 fps (3.8 ms) after optimising dependencies in dev (was 57 fps). Shared mesh/material handles already batch automatically; no instancing needed yet. Was: batching/instancing, 60 fps in a release build. [blocked: town-from-disc, world-objects-from-disc]

### M4 — UI, audio, feel — OPEN

- [x] **ui-from-disc** — completed scope: disc font, dialogue/name plate and choice windows with a no-disc fallback. The retail pockets/mail follow-ups below extend that foundation; this does not mean every HUD/menu has been recreated from the disc.
- [ ] **net-render-fidelity** — Philip reports the held net does not look retail. Audit source asset selection, textures/materials, mesh skinning, scale and pose against retail; correct the demonstrated mismatch before net visual sign-off. See backlog/net-render-fidelity.md.
- [~] **retail-pockets-ui** — original inventory frame, runtime icons/font, animated player/hand, slide, 15-slot selection, Grab/swap and supported net/rod Hold. Corrections cover shared up-left cloth scrolling (Philip's latest direction), source perspective/menu lighting, close/release, grid alignment and held-item rim exposure. Added saved phi/woponese labels and source-scaled bells. Static layout previously looked much better; final retail comparison acceptance remains open. Full retail tags/actions, mail integration and remaining hand/morph/recessed-slot fidelity stay named follow-ups; introduction/rename remains deferred. See backlog/retail-pockets-ui.md.
- [~] **retail-mail-ui** — ten disc envelope slots, original stationery/font, paged reading and existing compose/send. Philip accepts UI/paper appearance. Follow-up fixes world camera query regression and adds compact Read/Grab/Throw Away/Quit menus, confirmed discard, slot-preserving move/swap and save compatibility. Corrected camera/menu/actions await Philip comparison; separate mail transfer/storage, broader retail tags/keyboard, attachments and paper effects remain open. See backlog/retail-mail-ui.md.
- [ ] **retail-shop-ui** — first Cranny flow implemented: display-linked purchases, source zone-follow/facing, original Nook dialogue/voice/choices, pockets Sell/Quit and offer confirmation. Disposable purchase/refusal/full-pocket/funds/per-slot sale/cancel and table collision reviewed; Philip acceptance pending. Category previews, paint, tickets, multi-sale/special item flows, services and other layouts remain open; see backlog/retail-shop-ui.md.
- [ ] **retail-pause-ui** — compare the original pause/save flow and implement its applicable presentation; retain accessible port Settings and Quit controls. See backlog/retail-pause-ui.md.
- [ ] **retail-hud-ui** — source-backed clock/date, bells and context prompts; replace the current development-style overlay where retail has an equivalent. See backlog/retail-hud-ui.md.

UI order: pockets → mail → shop → pause/save → HUD. Pockets has a first
implementation; the other UI tasks remain planned. Each requires source tracing, runtime disc loading,
a usable no-disc fallback and Philip's comparison acceptance. This order does
not move title-screen or rename out of their deferred status.

- [x] **dialogue-polish** — typing, pauses, name plate and normal villager Animalese. Philip's F8 comparison sounds good without hiccups. Remaining voice modes/edge cases are tracked below.
- [x] **pause-menu** — Save / Settings / Quit; Settings currently edits the disc path.
- [ ] **title-screen** — remains on the back burner per Philip; loading currently enters InGame directly. Keep this deferral visible when assessing M4.
- [~] **audio-from-disc** — working foundation and several accepted features; scene music/effect management/action/ambience and initial mixer work implemented; listening, coverage and fidelity follow-ups remain. See the audit and ordered plan below and `backlog/audio-from-disc.md`.

#### Audio audit (2026-10-14)

| Area | Evidence/current behavior | Status |
|---|---|---|
| Disc loading and playback | `sound_rom.rs`, `nas_bank.rs`, `nas_seq.rs`: runtime banks/samples/sequences, streamed to Bevy | Implemented; foundation tested |
| Hourly outdoor music | `sound::update_music` selects hour+1 in town, restores the current hour after interiors; clear-weather mask retained | Earlier playback accepted; new scene transitions await listening. Weather-driven masks await simulation |
| Ordinary villager voices | Speaker personality selects one of three retail sequences; typed phoneme playback | Philip confirms F8 voices sound good |
| Player footsteps | Actual foot contacts, collision/menu silence, outdoor surfaces and indoor floor table | Philip confirms sounds and silence against walls/in menus |
| UI/mailbox/shop sounds | Pockets and letter sounds accepted. Mailbox opens with inventory ZOOMUP and closes with inventory NA_SE_31. Earlier register probe preserves MONO tail | Philip accepts letters and asked to move on; actor lid timing and general overlap validation remain outstanding |
| Existing gameplay actions | Source matrix and hooks for doors, rod cast/retrieve, net capture, tools, pickups, quest choices/handoffs and nearby NPC steps | Implemented; listening pending. Full reel-fight, fanfare and animation timing await gameplay support |
| Music by location and interaction | Quiet player houses, tier/closing shop themes, current-hour town restoration, 70% talk/50% menus with pause-safe fades | Philip reports it works perfectly; accepted. Weather/events/furniture and retail scene wipes remain follow-ups |
| Environmental audio | River/shore source positions read from disc metadata; localized waterfall fallback and state-driven insect loops, four spatial level slots, distance/pan, menu attenuation and indoor/scene-exit cleanup | Philip confirms playback; cleanup/stress acceptance, waterfall actor positions, pond/frog coverage, broader insect events and weather/wind remain follow-ups |
| Effect lifetime and position | Live engine now reuses TrgSeManager: free slots, oldest/priority replacement, singleton suppression and MONO 14. Completion observed on audio updates using request port 1 and busy port 5, with half-second unstarted-request cleanup | Implemented; overlap listening pending. Available world effects now use shared listener pan/distance; full camera/actor smoothing remains |
| Voice fidelity beyond ordinary speech | Supported Latin charset, message size/mood, deferred syllables and disc-backed resident melody requests implemented | Current villager playback accepted; configured town tune, unavailable special-speaker interactions, click/silent configuration and full message timing remain |
| Mixer fidelity | Source-based CPU reverb, indoor step sends, world-effect pan/falloff, peak meters/protection implemented; linear resampling/shared voice pool remain | Philip reports successful playtest; fixed-point DSP, filters, Haas, surround and reserve lists remain |

Audit sources: current `app/src/sound.rs`, `sound_rom.rs`, `footsteps.rs`,
`msg_window.rs`, sound-call search across app systems, `logic/src/audio.rs`,
`nas_seq.rs`, `na_voice.rs`, completed UI task, and local original
`game64.c_inc`, `m_bgm.c`, `m_player_sound.c_inc`, `m_tag_ovl.c` and
`ac_mailbox_move.c_inc`. Existing translated helpers are starting points;
verify their command formats against the current sequence engine before integration.

Previous local validation: 609 workspace tests passed, 21 ignored; clippy/fmt
and build passed, plus three explicitly run real-disc probes. These are code
checks, not proof of retail sound identity. Reused probe buffers were corrected
to clear before the additive mixer update; earlier RMS values were inflated.
Audio batch after base voice commit `599590c` includes footsteps, UI fixes,
trigger lifecycle, scene music, action cues and ambience. Philip authorized
commits/pushes for future work; see Git history/upstream for publication state.

#### Remaining M4 audio plan — execute in this order

1. **ui-mailbox-sounds: close the reported defects** (P).
   Recheck pocket/menu exit, mailbox panel open/close and a single purchase.
   Trace each rejected sound to the exact original action; do not pick from
   enum names alone. Preserve accepted cursor/pocket-open sounds.
   Done when Philip accepts the corrections and effects finish without repeats.
   This listening check can happen while step 2 is implemented.

2. **audio-trigger-lifecycle — implemented locally; listening validation pending.**
   Replace blind slot rotation with free/busy completion, priority and singleton
   rules; retain MONO subtrack 14. Adapt existing `logic::audio` trigger helpers
   through safe Rust APIs after checking opcode compatibility.
   Implemented: corrected trigger command opcodes, atomic submission under the
   engine lock, audio-thread completion, pending-start grace and reset on effect
   sequence stop/restart. Pockets are accepted; mailbox close now follows the
   actual inventory-in-mailbox-mode call path. Physical lid animation/audio
   timing remains deferred rather than attaching lid sounds to menu actions.
   Done when rapid cursor inputs, footsteps and overlapping longer effects do
   not restart/truncate one another incorrectly; duration, completion and
   return-to-silence checks pass. See `backlog/audio-trigger-lifecycle.md`.

3. **audio-scene-music — accepted by Philip.**
   Port the relevant `m_bgm.c` room/field arbitration: select the original
   shop/house policy, restore town music on exit, preserve hour changes and
   fades, and apply the original menu/dialogue volume changes. Verify seasonal
   masks already translated in `logic::audio`; weather-driven variants need a
   real weather state, not a guessed condition.
   Implemented: `logic/src/scene_music.rs` and `sound::update_music` own scene
   arbitration. House stays quiet without a furniture player; shop uses tier
   IDs 44/37/38/39 and closing IDs 79/80/81/82 (including the pre-closing
   five-second quiet interval). Inventory/mail/pause use 50%, dialogue/shop
   transactions/quests 70%; restoration and pause fades use real time.
   Clear-weather masks remain live; translated snow/sakura/rain policy is
   available but awaits actual weather state. Scene fades adapt to this
   port's instant doors; retail's wipe and complete event arbitration remain
   follow-ups. No new tests or disc probes were run for this continuation.
   Done when town -> house -> town -> shop -> town, an hour rollover and
   opening/closing conversations/menus all produce the intended music/mix.

4. **audio-action-coverage — implemented available hooks; listening pending.**
   Build an action -> original call site/ID/timing checklist for actions already
   playable: doors, fishing cast/bite/hook/catch/putaway, net use/catches,
   pick-up, quest accept/complete/reward and existing shop/mail flows. Add
   missing hooks only where the original has a sound; do not invent a sound
   for every action. Add NPC foot contacts with nearby, surface-correct audio.
   Implemented: house 6/7/8/9 cues on entry/exit, sliding shop 44B/44C;
   Cranny traversal stays silent because its business-hours door is already
   open. Added rod cast/retrieve/landing/lift, tool change/putaway rustle,
   corrected net capture to TOOL_GET (5D), quest choice/handoff feedback,
   and nearby surface-aware NPC steps (normal/happy/angry/run contacts 1/9).
   Door cue spacing is sourced, but visuals still teleport; sliding-door hold
   timing is a stand-in until actor animation/proximity behavior exists.
   Nibble/escape have no added beep. Sustained fish-pull 24 belongs to the
   missing reel-fight state; complete catch fanfare/putaway animation timing
   remains gameplay-dependent. See audio-action-coverage.md source matrix.
   Done when each existing action is covered or explicitly silent per source.

5. **audio-world-ambience — playback confirmed by Philip; follow-ups remain.**
   Integrate the translated level-sound/emitter management for the current town:
   river/sea/waterfall and applicable nearby insect sources. Match source
   lifetimes, distance/pan and indoor muting; connect wind/rain only when the
   corresponding simulation exists. Stop emitters on room changes/despawn.
   Done when entering/leaving audible range and town/interiors leaves no
   global, duplicated or stuck loops.
   Implemented: six runtime data_bgd records per acre feed river 0B/shore 1C;
   marine classification and shore offset follow C. Waterfall 0C uses collision
   footprint centres until standalone actors exist. Four stable nearest voices
   use SE slots 8-11, masked level IDs, port-5 stop and a sequencer-update grace
   before replacement; LevSeManager and original spatial curves are reused.
   Calm cicadas/crickets and mosquito/available flight states report directly
   from live AI. Indoors/scene exit clears loops, menus/pause halve gain.
   Pond/frog ambience, additional insect event phases, weather/wind and complete
   spatial/reverb fidelity remain named follow-ups. No tests/probes were run.
   See audio-world-ambience.md for implementation and resume instructions.

6. **audio-mix-and-fidelity — initial mixer/spatial work playtested successfully by Philip.**
   Implement scene/room reverb and general spatial gain/pan; compare music,
   voice and effects levels and guard against clipping. Audit filtered
   resampling and voice allocation against the original using shared source
   and controlled listening clips. Fix audible mismatches. Record any remaining
   hardware-specific filter/Haas/surround gaps explicitly; do not claim a full
   jaudio_NES port while those remain.
   Implemented 2026-10-15: source NA_HALL_DELAY topology/preset in a float CPU
   wet bus, note/release-tail sends, subtrack reverb routing and normal indoor
   step send 5. Original ordinary room policy is retained; no blanket UI/voice
   echo or invented room-size presets. Added shared-listener world effects
   (bobber, net, pickup, mailbox flag), NPC step pan, linked stereo peak
   protection and opt-in cumulative four-bus/mix diagnostics.
   Accepted sequence/voice/footstep gains and scene ducks remain the baseline.
   Original filtered aResample and reserve-list allocation were audited;
   current linear interpolation/shared pool remain explicit gaps alongside
   fixed-point/filter/Haas/surround and special unplayable scene echo.
   See backlog/audio-mix-and-fidelity.md for sources and listening checklist.
   No tests/probes added or run; compile/lint/format/build gate recorded there.

7. **audio-voice-edge-cases — current villager paths playtested; town tune remains a follow-up.**
   Retain the accepted normal voices. Map supported non-ASCII characters through
   the game's charset, wire message size/mood where the dialogue carries them,
   and audit deferred syllables and melody requests against C. Give special
   speakers their original spec when their speaking interaction exists.
   Missing interactions (for example a new shopkeeper conversation) stay
   separate gameplay work rather than being assumed implemented by audio.
   Done when applicable cases have source-backed behavior and listening checks;
   unavailable gameplay-dependent modes remain named follow-ups.
   Implemented 2026-10-15: font charset round-trip for supported Latin text,
   retained message mood/character/line size controls, matching glyph scaling,
   deferred f=2 syllables, punctuation slot updates and resident melody requests.
   Melody uses runtime DOL metadata/sequence 248, actual draw-record voice_type,
   original SE slot 6 and safely owned SFX-state macros. Page/close cancels old
   pending syllables. Game-frame and melody handoff timing adapt to this port.
   No new special-NPC conversation or click/silent settings are implied.
   See backlog/audio-voice-edge-cases.md for sources, gaps and listening steps.
   Compile/lint/format/build status is recorded there; no tests/probes added/run.
   Philip accepts ordinary voices, emotion/size, pause/page behavior and the
   requested F8 accented-text examples. Both music-note examples produce sound;
   he reports a beep/other note sounds with no configured town tune. Melody
   request playback is accepted; a composed town tune/retail identity remains
   a follow-up rather than inferred complete.

8. **audio-acceptance — automated pass complete; combined listening pending** (P).
   Run one combined checklist across music/scene changes, overlapping effects,
   menus, voices, movement, ambience, save/load, pause/resume and restart.
   Verify missing/wrong disc keeps the placeholder loop usable; run the Rust
   gate and compare full sound identity/timing/tails with Philip's own retail
   game. Document accepted gaps and resolve audible defects before closing
   `audio-from-disc`. Only then schedule the M5 slice-wide playtest.
   2026-10-15: fixed stream/resource cleanup on game-state exit and pending
   voice/door/footstep/effect cleanup on successful F9 load, preserving music.
   Workspace check/clippy/fmt/build passed; 624 tests passed (22 ignored), plus
   four explicit disc audio probes. Combined render peak 0.5172, no limiting or
   nonfinite samples; MONO register retains 1.163 s under traffic. Synthetic
   overload, delay decay, scene/menu gain, emitter stability and no-disc queue
   checks pass. Actual device listening, load/restart and fallback UI remain
   Philip's final route in backlog/audio-acceptance.md. No user save/settings
   overwritten; configured town tune and fidelity/gameplay gaps remain open.

#### Villager-home transparency fix (2026-10-15)

Philip's three new screenshots exposed missing opaque window backing. Scope is
**all five regular villager home styles, both summer and winter variants**;
roof colors are not the concern. **villager-house-glass** marks opaque panels in
the shared window builder so zero combiner alpha cannot discard them. Body/frame
cutouts and blended glow retain their existing paths. All ten variants were
visually inspected in daytime and at night in a disposable Temp town; original
save untouched. See backlog/villager-house-glass.md for source/evidence and
Philip's provisional acceptance: "looks fine for now." This does not close full render-look or M4.

**house-board-collision** follows Philip's report that player homes, the bulletin
board and map boards lack collision. All four player homes now use posed-model
footprints and the real door position; both units of each board are blocked,
including save/load field edits. Revised home bounds exclude the yard fence/step,
follow the front wall and include body clearance. Philip accepts the revised
180-degree door approach ("looks good"). Evidence and limits are tracked in
backlog/house-board-collision.md. **ground-light-flicker** adds targeted decal depth
bias to coplanar window-light patches; Philip reports "that looked good".
Evidence/scope: backlog/ground-light-flicker.md. **mailbox-flicker** restores
source back-face culling for summer/winter body and lid surfaces; thin waving
flags stay double-sided. All four mailboxes reviewed in both seasons in a
Temp-only town. Philip acceptance pending; see backlog/mailbox-flicker.md.
**player-door-interaction** narrows player-home entry to a short ray aimed at
the finite door panel. Left mailbox opens from street and left edge in preview;
direct door entry works. Philip acceptance pending. Gyroid conversation handler
is not implemented (render/animation exist). **wishing-fountain** restores the
original layout actor, summer/winter meshes, foliage, translucent water layers
and first-tile scrolling with basin collision. Seasonal preview evidence and
remaining two-tile water/shadow/rating-conversation limits are recorded in
backlog/wishing-fountain.md. Philip visual acceptance pending.

#### Special-building comparison fixes (2026-10-15)

Philip prioritized his plat-test-pics side-by-sides over the proposed net action.
**special-building-look** is implemented: corrected all five special-building
palette indices, missing winter post-office t3 part, Cranny's sampled open/closed
door pose, and business-dependent Able Sisters/Cranny window lights.
Source/disc checks and in-game day/04:40 screenshots support the change;
Philip's retail comparison acceptance is still pending. See
backlog/special-building-look.md for evidence, source references and limits.
No global camera/season/lighting sign-off or M4 completion is implied.

#### Model action follow-up (2026-10-15)

Model-research/report.md and index.html corroborate the existing runtime
display-list/cKF pipeline; no new fundamental rendering defect was established.
Next bounded model task: **model-actions-net-swing**, a held normal net with one
complete empty swing, procedural tool-joint adjustment and stop/recovery.
See backlog/model-actions-net-swing.md for source references, relevant files,
acceptance and research-access limits. First slice implemented: runtime held
normal net, source hand/held-arm mask, independent procedural bend, shared
swing/recovery clock, movement/input gating and load/tool/scene cancellation.
Disposable preview supports empty swing/recovery and pause/unequip/F9 behavior;
Philip's visual/action acceptance remains pending. Existing catches retain
their distance/facing selection, now delayed after frame6; retail capture,
obstacle/pull, ready/creep and golden-tool work remain explicit follow-ups.
Main M3 loading work remains complete; render-look sign-off stays open.
Rod/capture families follow acceptance of this attachment/action-clock slice.
Philip's 2026-10-16 feedback: a net is visible, but he doubts its retail look.
Track the unresolved comparison/correction as **net-render-fidelity**; the
disc asset name alone does not establish correct rendering or acceptance.

Philip's latest audio report: “there is absolutely zero audio overlap.”
Preserve that wording; it does not establish that every F9/save/load/restart
or missing-disc fallback check passed. Combined acceptance remains open.

#### Other work before slice sign-off

- **render-look** remains open: camera, palette/colour, fog and time-of-day
  lighting have implementations, but the combined retail comparison still
  needs sign-off. No curved horizon.
- **retire-town-gen** remains in the backlog: migrate remaining disc-mode
  consumers to retail data while preserving the no-disc placeholder generator.
- The completed disc UI work covers conversation/choices. Remaining retail
  pockets/mail/shop/pause/HUD visuals are explicit open M4 tasks above,
  starting with **retail-pockets-ui**.
- **title-screen** stays deferred under the existing decision; audio completion
  alone does not complete every M4 item. Keep the deferral in the release scope.
- Critter follow-ups (weather, tree interactions, sting/Wisp reactions and
  glow lights) are recorded in the completed critter task; do not silently count
  these as finished or expand the current audio task to implement those systems.

### Later (after the slice)
Tree shaking, furniture placement, further lighting/weather work beyond the
implemented clock-driven lights, daily rollover, mail and shop polish,
new-game flow, more disc formats/regions.

### M5 — Slice hardening
- [ ] **playtest-script** (P) — a step-by-step checklist for the full "done when" list; Philip runs it with his disc.
- [ ] **bug-bash** — fix everything the playtest turns up.
- [ ] **windows-release** — a release build zip (no disc content in it) and plain-English steps: install, then point the game at your own disc image.

### Suggested order
Philip reports the mix/spatial playtest went well (2026-10-15), following
ambience playback confirmation. Philip accepts the current voice edge cases (step 7), including accented
speech and music-note request playback; no configured town tune exists yet.
Combined audio acceptance (step 8) now has a passing automated gate and four
disc audio probes, with lifecycle fixes rebuilt. Next: Philip's short combined
listening route (backlog/audio-acceptance.md), including F9, pause/restart,
room changes, overlap and missing-disc fallback UI. Review remaining scope
before closing audio-from-disc.
Hardware DSP/resampling/reserve-list gaps remain
explicit, not silently completed. Resolve render-look and record deferred scope
before M5: playtest-script → bug-bash → windows-release.
M0–M2 are complete except the explicitly deferred rename; do not restart them.

## Done (pre-board, with Philip)
- Bevy 0.14 → 0.19 upgrade (branch bevy-0.19-upgrade, merged to main).
- B0001 query-conflict fix (Without<> disambiguators in mail.rs / quest.rs).
- Phases 0–3: plane, town render, villagers + dialogue, save/load.
- Workbook bevy-upgrade-workbook.xlsx through finding #21.

2026-10-14 continuation: mailbox close and trigger scheduler compile with workspace
check, all-target clippy, formatting and build.
Tests and disc probes were not rerun in this continuation. Qwen reviewed the small
completion method locally; its suggestions were speculative (no actionable defect).
Audio thread uses exclusive mutable access under the existing engine mutex.

## Migration: rustimal-crossing → bevy-crossing

- [ ] **migrate-rustimal-modules** (backlog) — Review 194 Rustimal module paths:
  104 source-only, 90 overlapping (88 differ, 2 exact); preserve 27 Bevy-only
  files. All nine graphics modules already exist here. Muse furniture pilot
  copy verified against pinned source; Codex corrected lint/fmt and native
  save-angle parity. Storage/actor-loading follow-up now corrects owner-cell/layer
  save-back/restore, furniture-bank IDs, actor/bank/budget gates and separate
  layer occupancy; 18 focused furniture tests pass. First player-room gameplay
  slice now connects placement/pickup, three-item storage and numeric layout
  save/F9 restore to runtime disc rendering and collision. Retail movement,
  animations and per-type behavior remain separate/open. Philip accepts the
  current slice so far: collision, remembered furniture and saving work. Next
  bounded movement adapter now adds G + directions for push/pull/turn, original
  clips, atomic storage/stack/switch preservation and save/load focus. Philip
  reports this adapter works (2026-10-17). Exact long-piece retail pivots stay
  open; see backlog/furniture-movement-animation.md. Next bounded slice:
  backlog/furniture-birth-pickup-animation.md now implements original leaf
  display lists, birth pop and pickup clip/shrink with atomic save/load reset.
  Philip accepts this bounded presentation slice (2026-10-17). Exact dust and
  remaining furniture fidelity stay open. Storage opening/closing now connects
  19 audited original wardrobe/closet rigs, original player clips and safe
  storage UI completion/cancel/load handling. Check/lint/format/builds and bounded
  disposable previews pass; Philip acceptance is pending. See
  backlog/furniture-storage-animation.md. Special constructors remain open.
  Philip lacked a wardrobe to check it; furniture-playtest-access.md adds F4 in
  his house to put an original blue cabinet in an empty pocket, then normal
  I -> Place in room / E at its front. Storage animation acceptance still pending.
  Player-home landing now permits walking on the original brown entrance strip;
  exit requires standing there and facing the doorway. See player-home-entrance.md.
  The next bounded drawer slice adds 16 audited common rigs, original h1 player
  clips/sounds and source wide-drawer front-cell obstruction checks. F3 supplies
  a blue bureau for playtest. Runtime dynamic skin bounds fix its hidden model;
  classic drawers use their source rig alias. Check/lint/format, capture/default
  builds, final release build and bounded disposable interactions pass; pending
  Philip acceptance is recorded in furniture-drawer-animation.md. No new tests
  or module batch; exact source fidelity, M4 and migration remain open.
  Bounded diary follow-up now renders original diary00..15 goods on floor/surfaces
  and restores the source starter diary only for default rooms. Backspace recovers
  visible diaries atomically; storage contents stay hidden. Evidence/limits and
  pending acceptance: backlog/room-loose-diary.md. Other loose families remain open.
  Authorized follow-up diary-placement.md connects normal-condition pocket
  diaries to front surfaces/floor with atomic blocked refusal and save/load.
  Bounded disposable previews pass; Philip acceptance remains pending.
  Stationery-room follow-up extends the same path to all encoded paper bundles,
  preserving design/quantity, hidden storage and atomic refusal. Source review
  and bounded disposable checks pass; acceptance/evidence in stationery-room.md.
  See backlog/migrate-rustimal-modules.md and docs/rustimal-migration-inventory.md.
  Philip's directive: the rustimal modules were built to end up here.

