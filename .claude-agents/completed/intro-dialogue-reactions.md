# Intro dialogue expressions and gestures

Philip selected priority3 on2026-10-31, after accepting the smoke/camera build's
smoothness (53c42320). Agent codex; current main checkout workflow.

## Design

Use native message orders at the actual reveal position, original reaction
animation metadata, eye/mouth sequences and associated disc effects. Focus on
Rover, Porter, Nook and player reactions in the existing new-game intro. Reuse
the runtime disc renderer, NpcClip/CharacterRig and existing audio bridge.
Keep actor movement, current camera framing, hand-over ownership, numeric save
transactions and accepted smoothness. Never create material-count lighting
rebuilds again; environment's targeted initialization must remain intact.

Source: ac_npc_talk.c_inc (manpu/mood/demo timing), ac_npc_anime.c_inc (face
sequences/animation metadata), NPC reaction data, Rover/Nook intro controllers,
m_player_main_talk.c_inc and original player animation data. Inventory the
actual intro commands before selecting effects/variants; do not invent poses
or turn this into a general NPC-AI rewrite.

## Checklist

- [x] Audit intro order values, clip pairs, metadata, facial/effect requirements.
- [x] Reaction state/transition kernel and non-stealing message-order handling.
- [x] Integrate gestures/expressions without replacing scripted movement or transfers.
- [x] Native eye/mouth and relevant effect playback plus reset/scene/load cleanup.
- [x] Source review and sequential check/clippy/fmt/capture/default/release gates.
- [x] Synthetic Temp intro scenarios, inspected poses/faces/effects and bounded FPS.
- [x] Record acceptance/limits and roadmaps/comms; complete claim and publish explicit files.

No tests without Philip's request. Preserve actual save and unrelated.gitignore.

## Implementation and source evidence

Original source checkout4099d24. Rover's reachable messages2AC6..2AE6 and Nook's
intro use NPC0[0] values2,3,7,11,12,15,21,23,24..31 plus254/255 resets.
No NPC mood or player reaction commands occur in this slice; Porter07DD has none.
All27 reaction clips and effect display lists were confirmed on the disc.
Runtime assets remain on the disc; only source-derived mappings/code are stored.

intro_reactions.rs consumes only the NPC0 reaction value after reveal; it never
drains the hand-over/quest orders. Native clip start/end/mode/morph, eye/mouth
tables/types/stopped patterns and effect trigger frames are read at runtime.
Scripted movement, sitting and phone clips retain priority. Reaction state is
separate from NpcClip; idle restoration/scene/load clears it. Native identical
seqNo orders preserve their current frame, and hate_d2 holds instead of looping.

intro_reaction_effects.rs renders shock, laugh, sweat, question, thought, bulb
and exclamation effects (including bulb glow) at joint25's native spawn point.
It caches models, pools bounded visual slots and edits only changed effect
uniforms; continuous emitters feed the existing ambience mixer. Native effects
deduplicate continuous type/owner pairs and retain their fixed spawn anchor.
Scene/load/despawn clears ownership and sounds; pooled meshes remain hidden.

Astra's source audit/review caught same-clip frame preservation, native0.5 morph
countdown and the need to keep player blinks free of NPC facial metadata. Root
also cross-checked final display-list colors/LOD (inner Gfx state takes priority).
The isolated train pass exposed the final sit clip holding pose ownership;
intro_train now returns to seated idle after that clip completes, enabling the
last conversation's niko_d1/ha_d1 reactions. The final train replay confirmed both.

## Final verification - 2026-11-01

Final check/clippy/fmt and capture/default/release builds pass; release7m36s.
No tests added or run. Temp evidence: bevy-intro-reactions-20261101.
Full train flow completed, including name/destination windows, seated reactions,
standing/phone/return and final seated smile/exclamation before station arrival.
Full Nook flow completed: welcome/tour, home1 refusal, home2 acceptance, repeated
explanation, payment cancellation and Give/transfer/departure. Final synthetic
save has home2, loan17400, no starting bag, intro_done/first_job true.
Repeated21 orders preserve niko1's frame rather than restarting its pose.

Inspected shock/laugh/question/thought/sweat/bulb/exclamation models and facial
poses. All15 cached effect mesh entities were Hidden at completion. F9 retained
house/loan/pockets/room data. Missing-disc completed-save startup retained
bells/loan/pockets/conditions/house/furniture/floors/identity and created0 effects.
Audio diagnostic counters: nonfinite0, limiting0; audible acceptance remains open.

No Cargo builds or screenshots ran during the frame-cost samples. A matched smoke
window after its first2s had p95/peak13.82/18.24ms,0 samples>20ms (75 with smoke).
An active-reaction sample had median/p95/peak9.11/14.22/16.61ms. Explicit
NO_VSYNC walking had median/p95/peak10.75/13.99/15.81ms,0 samples>20ms (228).
Baseline and other launches varied considerably, including occasional startup
walking spikes in both builds; these bounded samples establish no exact FPS
parity or claimed speedup. The prior smoke material invalidation fix is untouched.

All16 mappings are source/disc verified; the complete representative flow was
played, rather than every possible rude-answer permutation. Philip's visual and
audio acceptance and broader M4 fidelity remain open. Prior build smoothness was
accepted before Philip selected3. All owned previews are closed.
Real-save baseline/final SHA256 for this task is
0495927D27B005E9FD0A4AED424EAAD883F56BBD8A9C4F5910167C1F9BBB1BD9
(Philip's save changed since the prior task; never read/copied for this task).
