# bevy-crossing roadmap (revised 2026-10-14)

2026-10-30 codex: station arrival recheck and Philip's walking/jump feedback.
The scripted walk now supplies locomotion state, runs before character animation
and uses half its previous speed (30 movement ticks/s). Terrain grounding yields
to train/get-off height control; clip progress settles the landing height and
the stopped player clip uses the existing exit blend. Porter's completed talk
result is consumed, releasing input. Isolated runtime capture confirms settling,
changing walk poses, returned keyboard movement and F5 with intro/job state.
See intro-arrival-recheck task; Nook's welcome/house choice and Philip's acceptance
remain open.

## Remaining shop services (Philip requested roadmap, 2026-10-25)

Remaining priorities after native shop tiers and renovation lifecycle. The visible checklist is
[Remaining shop work](../docs/roadmap.html#shop-services-next); detailed scope is
[retail-shop-ui](backlog/retail-shop-ui.md). Keep that parent/M4 open.

1. Turnip price questions and selling: native price/date/quantity/spoilage rules.
2. Repeat-service conversations and original password validation/item delivery.
3. Scheduled bargain sales with native event stock/layouts and the missing
   festive-candle model. November grab bags/opening and Halloween candy are
   connected; December furniture substitutions and raffle exist.
4. Twins' full escort/echo, renovation-board presentation and neighboring-field
   rewrites; original pockets quest-warning presentation, remaining approach/transfer poses,
   cameras/movement blending and Philip's
   browsing/purchase/selling/audio acceptance.

Already implemented: Nookway paint service and ordinary room/merchandise layout;
all ordinary shop tier rooms/exteriors/clocks/uniforms, department floors/stairs
and paint access, native tier hours and persisted two-day renovation lifecycle;
ordinary purchases and normal single/marked/duplicate-item sales, category previews and
clothing effects, tickets, catalog orders/delivery, raffle, house expansion roof
selection/basement/statue progression. Do not schedule these again as missing.
Pause/save screens and special quest/mail tags remain separate UI follow-ups.
Item counts, parent ownership and M4 acceptance remain unchanged.

2026-10-26 codex: shop-november-grab-bags connects the day after November's
fourth Thursday, current-year price, native stock6/9/13/18 and ordered reserves
(all18 downstairs at Nookington's), exact purchase/sold-slot/no-ticket behavior.
Pockets Open requires three existing free slots, stages three rare ABC goods
with native seasonal clothing/duplicate exclusion and possible one pinwheel,
then commits contents/bag removal together. Native six-frame bag icon and
70tick shrink/staggered growth are transient; wrapped bags unwrap separately.
Capacity refusal uses original words in the existing message window; full mWR
panel and unimplemented spotlight rare_item exclusion remain fidelity limits.
See completed/shop-november-grab-bags.md for source/QA/build evidence and Qwen
assessment. No roadmap count or parent/M4/Philip acceptance changes.

2026-10-25 codex: shop-seasonal-stock connects October16..30 candy2806 at
100Bells, quantities2/3/4/5 by tier, with flowers1/1/2/3 moved to FE05 and
candy in FE08. Explicit saved stock/sold slots reuse existing purchase,
refusal and no-ticket rules; legacy same-day lineups keep their arrangement
until daily restock. December source substitutions already match native rules.
November grab bags require stock plus three-item opening; bargain events need
saved scheduling, stock and sale rooms. Those services, Philip acceptance and
M4 remain open; see completed/shop-seasonal-stock.md for verification evidence.
December26 stock checks also exposed the existing missing iam_yaz_candle3298
model warning; numeric substitutions pass, candle rendering remains unfinished.

2026-10-25 codex: [shop-higher-tiers](completed/shop-higher-tiers.md) connects
all four native tier rooms/exteriors/uniforms/clocks, department floors/stairs,
downstairs paint and upper Timmy service/preview camera. Replaces immediate
upgrade with persisted +2-date renewal, closed intervening day, native hours,
leaflet and retained visitor gate. Exact source portals, owned room replacement,
native lower-tier route tables and raffle offsets; synthetic entry/exit, saved
state, renovation and department transactions checked. No tests; see task for
final gates and explicit twin/board/field-rewrite/travel limits. Philip/M4 open.

2026-10-25 codex: [shop-selling-warnings](completed/shop-selling-warnings.md)
adds native57 quest refusal, wrapped slot sales and56/216 free disposal with
58/59 follow-ups. Any quest in a marked offer refuses the whole batch; mixed
junk/paid offers remove exact slots and pay only priced goods. Disposal/cancel,
wrapped accept/cancel and quest single/batch checked in synthetic Temp, preserving
unrelated fields. Original pockets warning-window presentation stays in fidelity
work; turnips remain separate. See task for build/QA evidence and limits.

2026-10-25 codex: [shop-multi-selling](completed/shop-multi-selling.md) adds
native M marks/Sell All and duplicate15 all-or-one followed by27, or marked214.
Paper quantities aggregate sheets and retain exact selected-stack remainders;
slot/condition snapshots and staged wallet/overflow bags prevent partial sales.
Temp batch/cancel/duplicate/paper/overflow/capacity/protected/mark-clear checks,
no-disc fallback, check/clippy/fmt and capture/default/release builds pass.
No tests added/run. Real save hash unchanged; warning/disposal/turnip flows and
Philip's playtest remain open. See task for numeric evidence and source limits.

2026-10-25 codex: shop-paint-service connects native202/203 paint offers to
exact-slot purchase, funds/roof queue/ledger/sold sign, bypassing pockets and
tickets. Pending palette applies on startup/load; same-size own roof refresh
uses a bounded twelve-palette native-model cache without changing neighbours.
Cranny FG22 has no paint, so ordinary SUPER/Nookway FG24/rom_shop3 layout was
added as a dependency, including native room/default69/shelves/collision/offset
door and preview coordinates. Other tiers and raffle retain previous layouts;
Nookway clock/uniform/exterior and Nookington's paint floor remain follow-ups.
Normal/full purchases, cancel/short refusals, duplicate sold-slot repeat,
all12 roof palettes/cache reuse, reload/restart, Nookway preview/door and Cranny
regression checked. Unrelated empty field_items repopulates20 on fresh restart;
paint state otherwise preserved. See completed/shop-paint-service.md for exact
evidence and limits. Check/lint/fmt/capture/default/release pass, no tests.

2026-10-25 codex: shop-change-effects adds ef_kigae/ef_kigae_light spirals
and NA_SE_WEAR0x11C at MENU_CHANGE1 frame16 for try-on and return. Source feel
joint25, radius22/every2ticks/60tick emitter,30tick scale/fall children, original
sparkle geometry/legacy IA8 decode and billboard. Gong keeps alpha200; clothing
uses255. Once-per-gesture latch, pause/scene/load/AppState cleanup; explicit
LoadSet ordering from review. Repeated captures and complete39-field saved
comparisons pass; source sound ID/diagnostics checked, audible acceptance still
requires listening. See completed/shop-change-effects.md for gate/runtime
evidence and limits. No item-count/parent-ownership/M4 acceptance changes.
Next remaining shop services, pause/save screens, special quest/mail tags.

2026-10-25 codex: shop-category-previews adds original preview/skip/cancel
questions for ordinary Cranny clothing, wallpaper and carpet. Temporary shirt/
hat and room texture swaps restore before/after the final decision; MENU_CHANGE1
frame36, bare mannequin, native room-camera endpoints and28tick Hermite/return.
Exact stock/funds/full-pocket/ticket transaction retained. Input held throughout
gestures; scene/reload and shared mannequin cleanup reviewed. Inspected images,
39-field cancel/refusal comparisons, exact purchase/ticket totals, camera and
gesture-input assertions pass; sequential compile/lint/fmt/capture/default/
release gates pass, no tests. Evidence in completed/shop-category-previews.md.
Real save hash unchanged. Counts, parent ownership and M4 acceptance unchanged.
Next pause/save presentation, special quest/mail tags, remaining shop effects/
services; clothing puff/wear sound and Philip comparison still open.

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
- [ ] **net-render-fidelity** — Deferred by Philip (2026-10-18). Held-net appearance mismatch remains recorded; revisit asset selection, textures/materials, skinning, scale and pose when requested. See backlog/net-render-fidelity.md.
- [~] **retail-pockets-ui** — original inventory frame, runtime icons/font, animated player/hand, slide, 15-slot selection, Grab/swap and supported net/rod Hold. Corrections cover shared up-left cloth scrolling (Philip's latest direction), source perspective/menu lighting, close/release, grid alignment and held-item rim exposure. Added saved phi/woponese labels and source-scaled bells. Static layout previously looked much better; final retail comparison acceptance remains open. Full retail tags/actions, mail integration and remaining hand/morph/recessed-slot fidelity stay named follow-ups; introduction/rename remains deferred. See backlog/retail-pockets-ui.md.
- [~] **retail-mail-ui** — ten disc envelope slots, original stationery/font, paged reading and existing compose/send. Philip accepts UI/paper appearance. Follow-up fixes world camera query regression and adds compact Read/Grab/Throw Away/Quit menus, confirmed discard, slot-preserving move/swap and save compatibility. Corrected camera/menu/actions await Philip comparison; separate mail transfer/storage, broader retail tags/keyboard, attachments and paper effects remain open. See backlog/retail-mail-ui.md.
- [ ] **retail-shop-ui** — first Cranny flow implemented: display-linked purchases, source zone-follow/facing, original Nook dialogue/voice/choices, pockets Sell/Quit and offer confirmation. Disposable purchase/refusal/full-pocket/funds/per-slot sale/cancel and table collision reviewed; Philip acceptance pending. Category previews, paint, tickets, multi-sale/special item flows, services and other layouts remain open; see backlog/retail-shop-ui.md.
- [ ] **retail-pause-ui** — compare the original pause/save flow and implement its applicable presentation; retain accessible port Settings and Quit controls. See backlog/retail-pause-ui.md.
- [ ] **retail-hud-ui** — source-backed clock/date, bells and context prompts; replace the current development-style overlay where retail has an equivalent. See backlog/retail-hud-ui.md.

UI order: pockets → mail → shop → pause/save → HUD. Pockets/mail/shop have
initial implementations; pause/save and remaining HUD presentation stay open.
Each requires source tracing, runtime disc loading, a usable no-disc fallback
and Philip's comparison acceptance. This order does not move title-screen or
rename out of their deferred status.

Shared original tag-art follow-up (2026-10-24, codex): sen_itemw/sen_win frames,
shadows and cursor, source colors/font/body sizing, tall-popup name clearance
and wrapped Present labels. Tool/gift/paper/mail-discard/shop Sell/Quit capture
and cancel-state comparisons pass; no-disc fallback inspected. See
completed/retail-tag-art.md. The later completed/retail-tag-layout-motion.md
adds native ordinary tag anchors/edge rules/item arrows and20tick wait/.125/.185
scale animation with retained roots. Cancel returns name+action and reinitializes
name timing; busy guards prevent invisible actions and duplicate reopening.
Pockets/mail edges, nested confirmation, cursor retention, rapid gift presses,
reload/empty slots, shop Sell/Quit and no-disc fallback checked with numeric
comparisons. Special multiline quest/mail tag families, remaining shop/pause/HUD
and Philip acceptance stay open; parent retail tasks are not closed by this.

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
   See completed/audio-mix-and-fidelity.md for sources and listening checklist.
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
   See completed/audio-voice-edge-cases.md for sources, gaps and listening steps.
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
save untouched. See completed/villager-house-glass.md for source/evidence and
Philip's provisional acceptance: "looks fine for now." This does not close full render-look or M4.

**house-board-collision** follows Philip's report that player homes, the bulletin
board and map boards lack collision. All four player homes now use posed-model
footprints and the real door position; both units of each board are blocked,
including save/load field edits. Revised home bounds exclude the yard fence/step,
follow the front wall and include body clearance. Philip accepts the revised
180-degree door approach ("looks good"). Evidence and limits are tracked in
completed/house-board-collision.md. **ground-light-flicker** adds targeted decal depth
bias to coplanar window-light patches; Philip reports "that looked good".
Evidence/scope: completed/ground-light-flicker.md. **mailbox-flicker** restores
source back-face culling for summer/winter body and lid surfaces; thin waving
flags stay double-sided. All four mailboxes reviewed in both seasons in a
Temp-only town. Philip acceptance pending; see completed/mailbox-flicker.md.
**player-door-interaction** narrows player-home entry to a short ray aimed at
the finite door panel. Left mailbox opens from street and left edge in preview;
direct door entry works. Philip acceptance pending. Gyroid conversation handler
is not implemented (render/animation exist). **wishing-fountain** restores the
original layout actor, summer/winter meshes, foliage, translucent water layers
and first-tile scrolling with basin collision. Seasonal preview evidence and
remaining two-tile water/shadow/rating-conversation limits are recorded in
completed/wishing-fountain.md. Philip visual acceptance pending.

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
See completed/model-actions-net-swing.md for source references, relevant files,
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
  open; see completed/furniture-movement-animation.md. Next bounded slice:
  completed/furniture-birth-pickup-animation.md now implements original leaf
  display lists, birth pop and pickup clip/shrink with atomic save/load reset.
  Philip accepts this bounded presentation slice (2026-10-17). Exact dust and
  remaining furniture fidelity stay open. Storage opening/closing now connects
  19 audited original wardrobe/closet rigs, original player clips and safe
  storage UI completion/cancel/load handling. Check/lint/format/builds and bounded
  disposable previews pass; Philip acceptance is pending. See
  completed/furniture-storage-animation.md. Special constructors remain open.
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
  pending acceptance: completed/room-loose-diary.md. Other loose families remain open.
  Authorized follow-up diary-placement.md connects normal-condition pocket
  diaries to front surfaces/floor with atomic blocked refusal and save/load.
  Bounded disposable previews pass; Philip acceptance remains pending.
  Stationery-room follow-up extends the same path to all encoded paper bundles,
  preserving design/quantity, hidden storage and atomic refusal. Source review
  and bounded disposable checks pass; acceptance/evidence in stationery-room.md.
  Fruit-shell-room.md extends that path to six fruit and eight shells/coral;
  original models and bounded placement/pickup/save/refusal checks pass. Philip
  acceptance remains pending; no broader loose-item or M4/migration completion.
  Furniture-carry-loose.md adds push/pull with supported loose surface goods,
  synchronized visuals and atomic destination validation. Mixed stacks and
  save/load/refusal previews pass; loaded turns remain safely refused and Philip
  acceptance remains pending.
  Furniture-turn-loose.md adds loaded TypeA/TypeC turns with source-relative item
  offsets and goods yaw flags. Numeric positions save; runtime loose-facing resets
  on reload. Long loose-loaded turns remain refused. Bounded previews pass;
  Philip acceptance and broader movement fidelity remain open.
  See backlog/migrate-rustimal-modules.md and docs/rustimal-migration-inventory.md.
  Philip's directive: the rustimal modules were built to end up here.
  Authorized furniture-turn-long follow-up now selects source TypeB endpoint
  pivots from contact side/half, checks complete/friction cells and rotates
  canonical primary/model/loaded tops about the chosen endpoint. All four
  orientations/both endpoints, mixed tops, atomic refusals and shifted storage
  save/reload have bounded disposable evidence. Exact player contact/root/slip
  and conservative player sweep remain open; Philip acceptance is pending.
  See completed/furniture-turn-long.md. M4/migration remain open.
  room-loose-animation now adds source idle-player throw/growth/bounce and
  distinct floor pickup1/surface clear_table1 clips for supported diary, paper,
  fruit and shells. Original goods approach the rendered hand; atomic numeric
  transfers, bundle quantity, hidden storage and pause/save/reload remain intact.
  Bounded disposable previews pass; Philip acceptance and broader fidelity remain
  open. See completed/room-loose-animation.md; M4/migration remain open.
  Mushroom/candy follow-up now completes loose room support for all eight
  source foods0x2800..2807, using original indoor models and existing transactions
  and presentation. Bounded model/place/pickup/carry/turn/refusal/storage/save
  previews pass; eating/seasonal acquisition and Philip acceptance remain open.
  See completed/mushroom-candy-room.md; M4/migration remain open.
  Money/turnip follow-up adds original loose bag2100..2103 and turnip2F00..2F03
  models, source money landing sound, exact denomination/bundle/spoiled ID
  transfers and shared carry/turn/presentation. Bounded room/pocket/wallet and
  storage/refusal/save checks pass. Turnip storage remains excluded; redemption,
  trading/aging/acquisition and Philip acceptance remain open. See
  completed/money-turnip-room.md; M4/migration remain open.
  Music/fan/pinwheel follow-up adds55 loose music-item IDs plus eight fans/eight
  pinwheels, using original indoor models and source surface-yaw flags. Mixed
  turn/carry/place/pickup, variant galleries, atomic refusal/hidden storage and
  save/reload have bounded evidence. Music registration/playback, toy use and
  Philip acceptance remain separate/open. See completed/music-fan-pinwheel-room.md;
  M4/migration remain open.
  Room music-player follow-up implements shared 55-song home registration,
  duplicate refusal, selection/stop/single-title take-out, exclusive device
  switches, source BGM_MD0+index sequences and save/F9 collection persistence.
  Current pockets/dialogue menus are reused; F2 grants K.K. Love Song inside
  the home for playtesting. Original music grid, device/player open-close clips,
  speaker effects/positioning and exact placement/toggle details remain open.
  See completed/room-music-player.md. Philip listening/visual acceptance is pending;
  this does not close M4, audio or migration.
  docs/roadmap.html now separates implemented work from acceptance and has
  search/status filters. Codex proposes music presentation, toy holding/use,
  remaining item conversions, then decorating/special furniture actions. Those
  follow-ups are planned, not started; broader M4 acceptance precedes M5.
  Philip subsequently accepts music playback ("music is good") and authorizes
  music presentation plus fan/pinwheel use. The new original11x5 collection UI
  uses disc frame/cloth masks/icons/font, the source hand view and choice menu;
  includes title/unknown slots, wrapped navigation, listen/single-title take-out,
  full-pocket refusal, exclusive focus and reload cleanup. Native MD distance,
  pan and original device gain classes now feed existing BGM groups.
  All16 audited constructors pass start/end0,0 to MiniDiskCommonMove: no invented
  lids or kagu clips. Hardware filters/hiss, menu transitions/cloth timing,
  multi-mark take-out and source placement selection remain fidelity follow-ups.
  Eight fans/eight pinwheels now Hold from pockets using original hand assets,
  player poses/part table, fan E repeat/sound and movement-driven rotor/coasting
  with source level sound. 3 puts away; ownership/reload/placement stay atomic.
  F1 in the home grants two toys only with two free pockets. Environmental wind
  awaits weather. See completed/music-presentation-toys.md; new presentation/toy
  acceptance remains pending, while playback acceptance is recorded separately.

  2026-10-18 Codex music-source-fidelity (Philip explicitly chose item3): M marking
  and source ordered/partial take-out, original pulsing mark assets, panel motion
  and cloth scrolling, placement-time registered selection/off state, front-only
  home interaction and Start/Stop. Runtime DOL taps now feed source subtrack
  filter commands and per-voice CPU convolution; old-device hiss shares existing
  ambience with pinwheels. See completed/music-source-fidelity.md for bounded checks
  and final gates. Exact hardware DSP/remaining window animation and weather wind
  remain open, along with Philip acceptance, M4 and migration. Roadmap.html now
  records implementation separately from acceptance and proposes remaining room
  item families/decorating next; net appearance stays deferred.
  Final music-source-fidelity gates pass: check/clippy/fmt/capture/default/release (2m57s), no tests; disposable source/transaction/visual/audio and no-disc checks recorded in task. Actual save unchanged. New feature acceptance remains pending; release claim and publish.
  2026-10-19 Codex clothing-room-conversion: standard shirts2400..24FE now
  Place in room as original mannequin furniture1EB..2E9 with disc shirt textures.
  Native conversion helpers reverse pickup after rotation, placement/model guard
  and cloned-pocket transactions reused; original clothing icon2 presentation,
  movement/save and hidden storage stay intact. Reserved24FF/custom designs
  excluded. Temp previews verify first/middle/last, rotations/live push-pull-turn,
  F5/F9, stored/conditioned shirts and full/blocked refusals; no-disc preserves
  numeric gallery. See completed/clothing-room-conversion.md for final gates.
  Philip playtest/M4/migration remain open. Roadmap proposes umbrellas/tools next,
  then wallpaper/carpet and remaining special behavior; net appearance deferred.
  Clothing final gates pass: check/all-target clippy/fmt/capture/default/release (2m38s), no tests; live source/numeric/no-disc checks and roadmap browser evidence in task. Actual save unchanged; Philip acceptance pending. Release claim and publish.
  2026-10-19 Codex umbrella-tool-room: 32 standard umbrellas2204..2223 now
  convert to original stands342..361; exact pickup ignores saved rotation.
  Room source no_convert_tools TRUE retains normal/golden tools and all seven
  worn-axe IDs as loose goods, using original models/surface yaw. Net/rod Hold
  retained; dispatch follows the displayed choices. No new held-tool actions,
  custom patterns, assets or save fields. Bounded Temp previews pass umbrella
  placement/four rotations/movement/reload/pickup, tool floor/surface transactions,
  mixed table turn/reload, atomic full/blocked refusals, hidden storage and
  excluded/conditioned menus. Fresh no-disc preserves complete numeric state.
  Source/evidence and final gates: completed/umbrella-tool-room.md. Philip playtest,
  M4 and migration remain open; wallpaper/carpet decorating is proposed next.
  Held-net appearance, title and rename remain deferred.
  Umbrella/tool final gates pass: check/all-target clippy/fmt/capture/default24.91s/release3m27s; no tests. Source, numeric, visual and no-disc evidence recorded in task; actual save unchanged. Philip acceptance pending.
  2026-10-19 Codex room-decorating: standard wallpaper/carpet pocket actions swap
  previous installed pattern into the selected slot, including full pockets.
  Original71-pattern banks/geometry and0x11B sound reused, shell waits for menu
  closure, IndoorFloor index follows carpet for existing footstep lookup.
  Compatible optional home_furniture.decoration retains version1; legacy defaults,
  explicit empty furniture and no-disc numeric preservation stay distinct.
  Supported loose rolls reuse existing transactions/presentation. Temp source,
  numeric, visual, first/middle/last, full/same/cancel/conditioned/reserved refusal,
  hidden storage/music, F5/F9/fresh-start and no-disc checks recorded in
  completed/room-decorating.md. Custom designs, expanded rooms and basements stay
  separate; Philip playtest/M4/migration open. Special furniture/diary writing is
  proposed next; held-net appearance/title/rename remain deferred.
  Decorating final gates pass: check/all-target clippy -D warnings/fmt/capture12.22s/default43.54s/release2m38s; no tests. Source, transaction, visual and fresh/no-disc evidence in task. Actual save unchanged; Philip acceptance pending.
  2026-10-19 Codex diary-writing: placed surface diaries now open the current
  player's shared twelve monthly992-byte entries, using original diary paper/
  font, native trim/wrap/scroll/confirmation kernels and PC month/text controls.
  Safe partial migration from pinned Rustimal diary.rs retains bytes/newline205,
  padding, six-line viewport and bounded edit limits; no ABI/test/card framework
  imported. Yes commits; Rewrite retains draft; Escape cancels a draft. Optional
  room diary data persist independently of physical diary pickup. Dedicated
  modal input prevents movement/menu/tool/pause/save/load leaks while editing.
  Floor/hidden/misaligned diaries do not open. Source/numeric/visual/F5-F9/fresh/
  no-disc evidence and final gates in completed/diary-writing.md. Full calendar,
  month sprites/virtual keyboard/transitions, contact pose/repeat and other-player
  locks remain separate; current PC UI is a bounded adapter. Philip playtest/M4/
  migration open; remaining special furniture actions proposed next. Net/title/
  rename remain deferred.
  Diary final gates pass: check/all-target clippy -D warnings/fmt/capture22.27s/default46.94s/release3m04s, no tests. Exact bytes, cancellation/rewrite, limits, access/input isolation, pickup retention, fresh process and no-disc evidence in task. Actual save unchanged; Philip acceptance pending.

2026-10-19 Codex television-actions: source audit bounds power actions to
sum_tv01, sum_tv02 and sum_fruittv01. Original US callbacks draw both model
parts and select screen textures via segment8, with54/10/26-tick cycles and
level sounds5/4/6, positional16/17 switch effects. Native furniture switch
save-back and source animation kernels now drive cached disc image/material
handles; existing four-source ambience owns loops and scene retirement.
Power persists through room movement/turning and serialization; pickup clears
its cell bit and new placement starts off. Secondary-layer sets remain separate
from supporting furniture switches. Other TV callbacks, lamp interactions and
room illumination are separate: audited lamp profiles/callbacks do not draw
according to switch_bit, so no invented lamp toggle is exposed. See
completed/television-actions.md for bounded evidence/gates and remaining limits.
Philip playtest and audible acceptance/M4/migration remain open; deferred
held-net appearance/title/rename remain untouched.
Television final gates pass: workspace check/all-target clippy -D warnings/fmt/capture47.74s/default43.29s/release3m08s, no tests. Three TV variants, exact state/F5-F9/fresh/no-disc, cached screen visuals, powered movement/turns, surface/pickup, independent sources and scene audio retirement checked in Temp. Actual save unchanged; Philip visual/audio acceptance pending.

2026-10-19 Codex snow-tv-lamp-draw: kon_snowtv joins the existing TV/native
switch path with original US60 powered ticks,07a..d screens, face/body models
and level0x2B. Complete model/screen preflight now matches the renderer for all
four audited TVs, including the snowman's previously unsupported symbol names.
Cached runtime images retain the existing animation/mixer/save architecture.
Four lamps (bla/whi/gre/hal) now append omitted unique draw-callback model parts
after registered profile lists, in original opaque stream/order. Lamp callbacks
are static; no switch-dependent PointLight/room illumination is claimed. Source
aMR_FurnitureLightSet reads global light data, not a per-lamp insertion path.
No save schema changes. Details and bounded evidence/gates in
completed/snow-tv-lamp-draw.md. Philip visual/audio acceptance, M4 and migration
remain open; net/title/rename deferrals unchanged.
Snow-TV/lamp final gates pass: workspace check11.33s/all-target clippy -D warnings11.51s/fmt/capture23.51s/default21.18s/release2m39s, all original settings after Philip freed RAM; no tests. Exact placement/power/movement/pickup/F5-F9/fresh/no-disc and older-TV regressions, lamp/TV screenshots and audio scene retirement in task. Actual save unchanged; Philip visual/audio acceptance remains pending.

2026-10-19 Codex furniture-clocks: ten original clock callbacks now drive their
hour/minute joints from the existing local game time, including second fractions
and signed16-bit afternoon wrap. Native furniture_clock kernels implement the
m_time hand math, cKF half-speed repeat and shared120-tick room clock pulse.
Cached disc-backed skinned rigs retain source parent transforms, palettes/streams
and separately drawn short/long hands for snow/meza. Waclock has only its source
hour adjustment. Antique ticking uses positional12D at beat15/45, animation reset0;
scene/pause/load and owned birth/retirement gates preserve existing audio ownership.
Room origin gains a read-only accessor for active-room sound selection. No save
schema or alarm controls added: the audited meza callback only draws its hands.
See completed/furniture-clocks.md for source/joint/time-change/runtime evidence and
final gates. Configurable alarms, room illumination, Philip visual/audio acceptance,
M4/migration and deferred net/title/rename remain separate.
Clock final gates pass: workspace check8.98s/all-target clippy -D warnings24.90s/fmt/capture1m01s/default1m02s/release3m25s, normal project settings, no tests. All ten F6 hour deltas about-30deg, authored motion/pause, per-variant numeric roundtrip, extra-hand close-ups, movement/hidden/surface checks, F5/F9/fresh/no-disc and sound retirement evidence in task. Actual save unchanged; Philip visual/audio acceptance remains pending.

2026-10-19 claude room-light-switch: player-house Z light switch and switch-driven
room lighting from ac-decomp m_kankyo/m_room_type/m_lights (identical in ACGC-PC-Port
apart from dt scaling). Session switch bits with game-start 18:00-05:00 default,
reset on load, not saved; Z toggle with SE0x16/0x17, 20-frame switch-on flicker
(Z ignored meanwhile) and retail add_calc fades; room-S point light (120,180,180)
r1000 turned into per-object directional light for furniture/loose/retiring pieces,
the player and the room shell (control-actor origin); sun/moon x(1-0.6 level);
switch-dependent room prim on the floor/wall lists (disc lists verified to inherit
prim). Native logic::room_light plus ported subset of pinned 122867f lights.rs
(add_point_relative only). EnvUniform/shader gain one directional point slot. No save
schema change. Window sunshine (ef_room_sunshine), villager-room schedules,
basements/cottage remain follow-ups. Evidence/gates in completed/room-light-switch.md;
Philip visual/audio acceptance, M4 and migration remain open.

2026-10-19 claude room-sunshine: player-room window sunshine from ef_room_sunshine
and m_kankyo mEnv_MakeWindowLightAlpha (ACGC-PC-Port, identical to ac-decomp apart
from dt scaling). SCENE_MY_ROOM_S actors ct2 (40,0,120) / ct3 (200,0,120) with
their ct offsets and disc floor height -> L (38,41,120), R (200,41,120);
light_floor/light_shine lists and row-major I4 light_window read from the disc at
runtime; time-of-day scale ramps; prim = period sun/moon window colour with alpha
(u8)calc_alpha x window alpha. Window alpha: pinned 122867f kankyo_env subset
(window_light_target, s16 quirk) plus source easing add_calc(1-sqrt(0.8), 0.005)
after each light tick and the 0-reset (0.5 steps) on entry/load, using the live
RoomLight.percent. Native logic::room_sunshine. No save schema change; no-disc
fallback unchanged. Weather dimming (x0.6) waits for weather simulation; other
room scenes' sunshine variants not covered. Evidence/gates in
completed/room-sunshine.md; Philip visual acceptance, M4 and migration remain open.

2026-10-19 claude room-critters-balloons (branch claude/focused-mayer-edf6d2,
not merged): fish 0x2300..0x2327 -> aquariums 0x31A+n and insects 0x2D00..0x2D27
-> cages 0x2F2+n per mRmTp, exact reverse pickup; native logic::furniture_rig
(repeat frame, fNBC balloon speed) and app furniture::rigs shared skinned builder
(clocks moved onto it). Callback critters stay in pockets (room-insect audio).
Balloons deferred: G_TEXTURE_GEN reflection mapping is not rendered; their
conversion/sway/switch logic is kept. Evidence in completed/room-critters-balloons.md.
Philip playtest, M4 and migration remain open.

2026-10-19 claude texgen-reflection: emu64 texture_gen ported (world normal on
s8 camera right/up look-at axes, gSPTexture scale/64, settile shift, size
origin, size); decoder now records G_TEXTURE scale, settile shifts and
G_SETTILESIZE origin. Balloons place again with original sway/switch kick.
All G_TEXTURE_GEN lists (tank/cage glass, clocks, mirrors, cabinets, statues)
use it; strings stay cut as in emu64 alpha_compare. Evidence in
completed/texgen-reflection.md; Philip visual acceptance pending.

2026-10-19 Codex turkey-tv: fifth TV iam_iku_turkey_TV (index4B9/item32E4)
uses the existing TV/E/native switch/ambience/save paths. Only TV adapter/native
kernel changed; Claude's room-insect/audio files untouched. Original opaque
model_b then model_a and CI4 16x16 f/g/h/i screens decoded from the disc.
Turkey phase advances modulo24 even while off, g/h/i/h every6 ticks while on;
other four TVs retain powered-only two-tick table timing. Level sound5E and
switch16/17 use existing generic routing. No new UI/schema. See
completed/turkey-tv.md for evidence and validation limits. Existing TV roadmap
item extended; tracked counts remain83 items50 implemented6 doing27 planned.
Philip visual/audio acceptance and broader M4/migration remain open.

2026-10-19 Codex electric-fan: iam_nog_fan01 (index22/item1088) now uses a
native motor/delay kernel, source nog_fan skeleton alias and full decoded asset
preflight. Three cached blade textures follow the six-frame US delay counter;
skinned joints oscillate at eased speed0..0.5. E uses native saved switches and
16/17 sounds; level1 motor routes through the existing four-source ambience
mixer, with room/birth/pickup lifecycle gates. No save schema/UI changes.
Source CALC_EASE(0.05) and US stopped signed-counter semantics retained; 60Hz
catch-up capped4 as existing adapters. See completed/electric-fan.md for source,
Temp behavioral evidence and gate limitations. Fan item added:84 tracked items,
51 implemented6 in progress27 planned; not overall completion. Philip visual/
audio acceptance/M4/migration and Claude's insect task remain open.

2026-10-19 Codex gong-actions: source iam_nog_gong (boxing bell,index4EE,
item33B8) now has stopped-only E strike174 and exact double-play cKF control
order, using the shared skinned builder. Ringside wall65 adds singleton817E
and source FLASHC240/FLASH5 timing, back-wall random camera sprites and upward
diffuse light. Cached disc sparkle mesh and scoped classic IA8 row conversion/
tile metadata leave the global texture decoder unchanged. New effect uniform
is zero outside this room effect. Flash light registration's source parameter
order retained, including early reset; adapter uses its own cosmetic random
stream/100-slot gong cap, not retail's global effect pool. Retail shadow-alpha
response remains a fidelity limit. Pause freezes effects; load/scene exit clear
them; no effect state serialized. Temp checks in completed/gong-actions.md.
New tracked bell item:85 total52 implemented6 doing27 planned; Philip visual/
audio acceptance, M4/migration and Claude's insect task remain open.

2026-10-19 Codex slot-machine: iam_sum_slot (indexC6/item1318) now uses a
native stopped-only one-shot controller and exact 121-entry reel table, read
from ac_sum_slot.c. Constructor speed0 and sample-before-advance stop crossing
are retained; reel phase persists between spins. Full decoded disc skeleton/
animation/two-texture preflight, cached skinned rig and existing mixer level2.
Generic E switches silently; busy changes are consumed, no queue/payout/wallet
mutation. QuickLoad and room exit reset transient state even when refresh keeps
the same render entity. Pause freezes it; numeric placement/switch ownership,
movement and pickup reuse existing room logic. No schema change. Evidence and
validation in completed/slot-machine.md. New item:86 tracked53 implemented6 doing
27 planned; not game completion. Philip visual/audio acceptance/M4/migration
and Claude's insect task remain open.

2026-10-19 claude room-insect-audio: all forty insects and the frog place. Native
Na_RoomIncectPos/Sou_Insect_Lev_Cont kernel with the engine's trigger slots;
rooms run furniture seq 246, whose port-2 beat never advances in retail because
group callback 1 is never registered, so room insects stay mostly quiet as in
the original. Stopped cicada/cricket clips replay per chirp; frog/mosquito/
mole cricket repeat; firefly pulses its segment-8 prim alpha. Evidence and the
callback finding in completed/room-insect-audio.md; Philip acceptance pending.

2026-10-19 claude cuckoo-clock: ac_sum_log_hatoclk ported (native call state
machine with exact cKF stop timing and frame-2 sound 0x133, hands at joints
7/8 by yaw and 90deg minus angle). 12 calls observed at 00:00; Philip
confirmed the 21:00 call. See completed/cuckoo-clock.md.

2026-10-19 claude sprinkler: ac_nog_sprinkler ported (native fade/toggle
kernel, generic silent E, 0x16/0x17, level 0x5B, destructor write-back on
leaving). Mid-fade press quirk reproduced and verified; water flow needs
TEXEL1. See completed/sprinkler.md.

2026-10-19 claude lawnmower: ac_nog_lawnmower ported (grass level on the lawn
carpets with back-contact push/wait, prim alpha; 0x177 per back push).
With the cuckoo clock and sprinkler, all seven callbacks from the
furniture-callback audit are implemented. See completed/lawnmower.md.

2026-10-19 claude retail-hud-ui: retail has no field HUD (source trace), so
the dev text is a DevOverlay hidden by default with F10 to toggle (Philip's
choice). Shop wall clock (ac_house_clock) split to shop-wall-clock: blocked on
legacy N64 tiles and segment-7 palettes. texel1-tile-scroll queued next per
Philip. See completed/retail-hud-ui.md.

2026-10-19 claude texel1-tile-scroll: tile 1 bound for two-cycle parts
(TEXEL1, and cycle-2 TEXEL0 per emu64 combine_auto), Dolphin tile-size
scroll on tile 1; sprinkler water flows, lawnmower grass scrolls. See
completed/texel1-tile-scroll.md.

2026-10-19 claude shop-wall-clock: ac_house_clock in the shop; legacy N64
texture path (plain settile from w1, tmem map, setup_texture_tile sizes,
N64 loadtlut RGBA5551, row-major decode); zero-size tiles untextured. See
completed/shop-wall-clock.md.

2026-10-20 claude: Philip accepted special-building-look ("good now") and the audio trigger lifecycle ("fine"); both completed. New reports: idle clock/date display missing, July bunting in the shop out of season, black lines above the town name in pockets.

2026-10-20 claude retail-idle-clock: m_banti idle clock ported (logic banti + app idle_clock); shop raffle (fukubiki) room only on the last day of the month; font glyph UV inset fixes marks above pockets text. See completed/retail-idle-clock.md.

2026-10-20 claude: Philip accepted room music player and music-source fidelity ("i was listening to all the play tests").

2026-10-20 claude post-office-repay: post office interior, Pelly/Phyllis, loan repay (m_repay_ovl) and saved loan. See completed/post-office-repay.md. Next: house-upgrades.

2026-10-20 claude house-upgrades stage 1: saved house record, Nook offers/orders/billing, construction at load, exterior per size and roof colour. Interiors next. See backlog/house-upgrades.md.

2026-10-20 claude house-upgrades stage 2: player room size follows the house (6/8/10/10 units). Next: basement and upstairs.

2026-10-20 claude house-upgrades stage 3: basement and upstairs floors with stairs and per-floor saves. Next: statue (stage 4).

2026-10-20 claude house-upgrades complete: statue (stage 4) added. See completed/house-upgrades.md.

2026-10-20 claude police-box: interior, Booker, lost and found. See completed/police-box.md.

2026-10-20 claude museum: lobby, wings, Blathers, donating, fossil and painting displays. Fish/insect displays next. See completed/museum.md.

2026-10-20 claude police-officers: Copper outside (schedule, talk, menu), Booker follows, lost-and-found daily growth. See completed/police-officers.md.

2026-10-20 claude post-office-mail: real letters, counter sending, delivery, villager replies, museum appraisal. See completed/post-office-mail.md.
2026-10-21 claude nook-tickets-catalog stage 1: tickets with purchases, ticket/order start-up mail, raffle (fukubiki), collection bits. Catalog screen/ordering open. See backlog/nook-tickets-catalog.md.
2026-10-23 grok nook-tickets-catalog raffle room: three prizes on the fuku furniture spots, Nook at the drum, drum model and ball at tick 46. Catalog screen still open. See backlog/nook-tickets-catalog.md.
2026-10-23 grok nook-tickets-catalog screen: clg_* catalog, paid orders, price 0 refused, Escape cancels. Short on bells and five full slots not played. See backlog/nook-tickets-catalog.md.
2026-10-23 grok nook-tickets-catalog look: disc page textures on the catalog window, and the selected furniture drawn from the side in the circle. Wallpaper, carpet, stationery and music previews stay empty. See backlog/nook-tickets-catalog.md.
2026-10-24 codex catalog-preview-review: reviewed/fixed grok's pending scrolling/camera slice. Removed unsupported page-pixel recolour (clg_mwin_mode copies TEXEL0 RGB/TEXEL1 coverage); preview uses width256 camera20deg/near100/far800. Hierarchy Aabb/GlobalTransform fitting covers specialised/skinned furniture without modifying shared spawn_furniture. Bed, wardrobe, TV and clock screenshots inspected; conservative clock geometry bounds yield a smaller/higher preview, exact visible-coverage fitting deferred. Parent task and remaining gameplay paths stay open; see catalog-preview-review task evidence.
2026-10-24 codex catalog-raffle-verification: purchase ticket/overflow, insufficient Bells/full-five-order refusals, paid order and restart delivery verified with saved state comparisons and inspected original letters. Raffle consumed five tickets on loss; entity moves but ball visual check failed (double scale,4000x too small). Two renderer slab errors during mail viewing recorded, later fixed. See completed/catalog-raffle-verification.md, completed/raffle-ball-scale.md and completed/mail-render-slab-errors.md. No gameplay code/count changes; parent/acceptance/non-furniture previews remain open.
2026-10-24 codex raffle-ball-scale: ball parent unit scale fixes duplicate4000x shrink; drum/ball coordinate conversion now matches shell centreY1, seating drum atworld0.5. Philip's hat-clipping report addressed by adding exact posed drum footprint to raffle-day shop obstacles, retaining player radius0.4; verified front stopz-2998.38110 vs-2998.59985, held forward stable. Inspected visible ball roll/table contact/clearance; exact five-ticket loss and scene cleanup/reentry pass. See completed/raffle-ball-scale.md. Renderer allocator errors also reproduced on prior winning-prize display refresh; later fixed in mail-render-slab-errors. Existing item counts unchanged; parent/acceptance/previews/clock-fit remain open.
2026-10-24 codex mail-render-slab-errors: reproduced blank letter line and markup-only raffle reveal submitting zero-vertex indexed text meshes; Bevy0.19.1 skips their allocation but still uploads their keys. Shared builder returns None for no glyphs, all six callers skip asset/entity creation. Same letter and actual third-prize win replayed with no allocator errors; spacing inspected, complete affected letter state comparison and exact five-ticket/wrapped-prize/sold-slot assertions pass. No tests or dependency changes. See completed/mail-render-slab-errors.md and comms.txt for build/runtime evidence. Counts unchanged; parent remains claude-owned/open.
2026-10-24 codex catalog-flat-previews: original flat mCL display lists for wallpaper/carpet/stationery and shared animated music note now fill catalog preview, separate from existing furniture camera. Correct floor paletteD/textures8-B, wall paletteA, source placement/scales, canonical paper design modulo64 and I4 ruling alpha. Multiple patterns/empty tab/music motion inspected; navigation/cancel numeric comparison passes. See completed/catalog-flat-previews.md and comms.txt for final gates and lifecycle evidence. Item counts unchanged. Next exact clock preview fitting, remaining retail UI, Philip playtest/acceptance; parent claude-owned/open.

2026-10-26 codex: shop-bargain-sales connects saved one-hour special shop
sales, native preparation/packing closures, source FG layouts for all tiers,
separate offer/ordinary stock, uncollected-first rare goods, original leaflet,
intro and wrapped balloon. Active-stock getters preserve exact purchase,
preview, ledger and normal raffle-ticket behavior. Renovation lookahead now
also honors the sale date on load. The six-event calendar reserves other
visitor turns without spawning actors; stable town/resident seed and saved
rank0 are explicit adapters pending personal IDs/field assessment. Exact gift
handover/twin choreography and festive-candle rendering remain follow-ups.
Parent ownership, item counts, M4 and Philip acceptance remain open. See
completed/shop-bargain-sales.md for validation and limits.

2026-10-26 codex: festive-candle-display supplies the missing ac_yaz_candle
custom draw path: native body/fire models, segment9 and two original flame
textures alternating every two native ticks. Assets are decoded once, reused
through material handles and gated by GameplaySet. Shared placement availability
now recognizes the same asset set. Dec26 shop display/purchase/cancel/reload,
flame captures and all four player-room rotations inspected. See completed task
for final gates and limits. No stock/calendar/transaction/schema changes; item
counts, parent/M4 and Philip acceptance remain open.

2026-10-26 codex: shop-turnip-service connects native m_kabu_manager daily
prices and three weekly trends with saved Sunday/date state; source Other
things price inquiry for Nook/Timmy and10/50/100-bundle selling. Exact duplicate,
marked mixed, quest/present, Sunday refusal, zero disposal and atomic overflow
rules retain existing sale flow. Turnip-specific Town growth hook spoils exposed
pockets/field/all house layers at Sunday06/backward dates and clears old spoiled
outdoor goods on forward growth; missing legacy age clock adopts first load.
PC RNG supplies native probability draws, not original global RNG state. Joan,
other players/island/gyroid storage and password service remain separate;
counts/parent/M4 and Philip acceptance stay open. See completed task evidence.

2026-10-26 claude: shop-password-service connects the counter's Say code flow
for user and popularity-contest passwords: typed two-line entry on the disc font,
native codec/validation (logic/password.rs, app/shop_nook/password.rs), the
source gift limit, full-pocket and wrong/foreign/declined branches, retry with
remembered text, and the present delivered with its present condition. Send
direction, Famicom, magazine and mini card-e codes plus the retail character grid
remain (backlog/shop-password-send.md). Counts/parent/M4 and Philip acceptance
stay open. See completed/shop-password-service.md.

2026-10-26 claude: shop-password-send connects the counter's Send a code
direction (receiver names window, same-town refusal, confirmation, item picker
in a new pockets mode, trade-in with the made code shown in Nook's own message).
Redeeming and sending user codes now both work; Famicom, magazine and mini
card-e codes, the retail letter grid and hand-over animations remain. See
backlog/shop-password-send.md. Counts/parent/M4 and Philip acceptance stay open.

2026-10-26 claude: shop-password-codes redeems Famicom, magazine (random win/loss
by the source hit-rate table) and mini card-e passwords with the original
messages; every password kind except the retail letter grid and hand-over
animation now works at the counter. See backlog/shop-password-codes.md.
Counts/parent/M4 and Philip acceptance stay open.

2026-10-26 claude: shop-password-handover plays the original present hand-over for
counter passwords: the disc's wrapped present grows out of Nook's left hand and is put
away into the player's, driven by the ported logic/hand_over_item kernel. Nook's arm
gesture and the exact mid-message start remain. See backlog/shop-password-handover.md.

2026-10-26 claude: shop-password-arm plays Nook's own transfer and hold arm clips
during the counter present hand-over (shop master animation table), so the gift now has
his gesture as well as the disc's wrapped present. Frame sync, the spring chase and the
mid-message start remain. See backlog/shop-password-arm.md.

2026-10-26 claude: shop-password-grid replaces the typed password entry with the retail
editor grid drawn from the disc (purple controller, buttons, 10x4 keys) over the original
secret-code and receiver windows, driven by the new disc-free logic/editor_keyboard kernel
(key tables, cursor repeat, button repeat, name fields). Keyboard and typing both work;
the exchange (accent) key, voices and scrolling window texture remain. The diary and
letters can reuse the kernel and editor module. See backlog/shop-password-grid.md.

2026-10-26 claude: joan-turnip-peddler brings Joan out in town on Sunday mornings
(06:00-11:59) with her disc model, voice and full script; she sells 10/50/100 turnip
bundles at Sunday's saved price with the original refusals and re-ask, and remembers
who she has spoken to. Her walk-away, the money/turnip hand-over demos and the exact
placement rules remain. See completed/joan-turnip-peddler.md. Counts/parent/M4 and
Philip acceptance stay open.

2026-10-26 claude: joan-handover-demos. The hand-over is now generic (giver/taker/item): after Joan's sale the player hands her the money bag and she hands back the turnip bundle with the original arm clips, models and curves; she walks to a new nearby spot after each talk. Nook's present uses the same code and the player now plays its give/get arm clips. Exact placement rules remain. See completed/joan-handover-demos.md.

2026-10-26 claude: cuckoo-catalog-fit. The cabin clock's catalog preview now fills the circle: a visible-box override (furniture::FitBounds) replaces its mesh-bounds fit, which three never-drawn spire triangles had inflated threefold. Other pieces unchanged. See completed/cuckoo-catalog-fit.md.

2026-10-27 claude: seasonal-balloon-handover. The sale-day gift balloon is handed over with the generic hand-over (present model and arm clips). Remaining seasonal items are blocked or optional: identity/assessment inputs, spotlight exclusion, escort/twin, grab-bag warning panel. See completed/seasonal-balloon-handover.md.

2026-10-27 claude: field-rank-player-id. The town's beauty rating is now computed from the live field items (kernel logic/field_assessment.rs, app/field_rank.rs) with a saved trash flag, and the player gets a saved personal ID; both feed the special-event scheduler, and the rating drives insect hatching. Villager move-in, fish, balloons and the shrine remain unwired. See completed/field-rank-player-id.md.

2026-10-27 claude: shop-spotlight-rare. Nookway and Nookington's stock a daily spotlight rare furniture (rolled first from the rare list, shown at its own FE09 spot, saved, excluded from other draws incl. bargains and grab bags). The rare-furniture notice letter is not wired. See completed/shop-spotlight-rare.md.

2026-10-27 claude: diary-retail-grid. The monthly diary's writing mode now uses the retail letter grid (disc controller and keys) with the source's in-text cursor (insert, erase, left/right, line up/down, space and newline at the end), new disc-free kernel functions in logic/diary.rs, and a rolled-up paper showing five lines. Start (F4) leads to Yes/Rewrite. Letters still need their own editor host. See backlog/diary-retail-grid.md.

2026-10-27 claude: letters-retail-grid. Letter writing (new on stationery and Rewrite) uses the retail grid with the shared in-text cursor kernel (logic/diary.rs, now with Limits), a rolled-up paper and 192-byte/6-line limits; Page Up/Down picks the addressee. See backlog/letters-retail-grid.md.

2026-10-27 claude: shop-rare-notice. Nookway/Nookington's rare spotlight furniture now sends the original rare-furniture handbill on the first play of a day (kernel logic/leaflet.rs rare_notice_gate, saved play time shop_progress.played). See backlog/shop-rare-notice.md.

2026-10-27 claude: villager-move-in-rank. The daily move-in check uses the town's field rank (40-100% by rank, npc_grow_prob) computed at start-up from the retail field items plus the save's changes. See backlog/villager-move-in-rank.md.

2026-10-27 claude: fish-spawn-field-rank. River/sea/pond fish spawns use the original selection with the field-rank rate (0.5, 0.75, 0.875, then 1) and habitat rerolls (logic::ecology::fish_select, ecology.rs place_fish). See backlog/fish-spawn-field-rank.md.

2026-10-27 claude: shop-rare-sold-sign. Buying the spotlight rare furniture was driven end to end: the SOLD OUT sign replaces it, survives a restart and the next day's restock brings a new piece and notice. No code change. See backlog/shop-rare-sold-sign.md.

2026-10-27 claude: grid-leftovers. The retail letter grid gets its X accent exchange (Ctrl+X in the diary and letters; arrow plate, ornament table kernel), a voice for typed keys and the R pressed face. Scrolling window texture, cursor pulse and pad rumble remain. See backlog/grid-leftovers.md.

2026-10-27 claude: grid-leftovers (scrolling window texture). The secret-code and receiver windows' cloth background now scrolls at the original menu rate. Cursor pulse and pad rumble remain. See backlog/grid-window-scroll.md.
