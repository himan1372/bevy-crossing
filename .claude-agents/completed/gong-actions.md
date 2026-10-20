# Gong strike and ringside camera flashes

Philip authorizes gong after electric fan. Source ac_nog_gong.c/c_keyframe.c,
ef_flash.c and effect-light helpers. Gong index4EE/item33B8; E toggles native
switch silently, source stopped-only strike0x174 and two cKF play calls while
running. WALL_NPC065 adds singleton817E plus FLASHC240 ticks: every8 ticks
spawn5-tick disc flash at room-S back-wall random positions; alternating effect
light. Preserve source light registration timing/parameter order, not guessed
full-screen flashes. Existing room/sound/save input paths. New native/app gong
modules, clean shader/environment/light integration plus isolated room/lib hooks.
Claude owns insect task; do not edit/stage his pending hunks or sound/rig files.
No tests requested. Normal sequential gates; Temp synthetic preview/save only;
actual-save read-only baseline; all gameplay evidence distinct from acceptance.

## Source decisions and first preview

GAFE01 gong clip is10 frames; constructor samples1 then advances0.5 silently.
The move callback samples/advances twice while running; stopped-only change
restarts at1 and emits174, with no generic power click. Native kernel preserves
that order rather than using one generic stop_frame call. Ringside wall index65
adds singleton817E and FLASHC240 ticks, child flashes every8 ticks with5-tick
life, room-S x40..200/y65..85/z43, alternating upward diffuse light. The raw
IA8 sprite ef_takurami01_1 is row-major16x16, source mesh/mode from disc.
Native source light argument order length/half-length is retained: the light
resets before the nominal27 peak. Adapter uses independent cosmetic xorshift
random stream and a100-slot gong-effect cap; not the shared retail global pool.
Retail shadow-alpha response to the flash light remains a render-fidelity limit;
this task adds the source upward diffuse contribution, not a shadow system.

Initial check9.90s/clippy20.91s/capture57.52s passed, no tests. First Temp7780
preview correctly refused placement and retained pocket item: plain SETTILE
metadata isn't populated by the generic decoder (same issue as sunshine).
Fixed exact local IA8 tile dimensions/mirror wraps and repeat preview next;
global texture decoder unchanged. This failed placement is excluded from success
evidence. Diagnostic warnings retained only for genuine missing gong body/rig.
Original actual-save baseline C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.

## Corrected runtime evidence

Corrected capture17.00s passes. Temp16364: placement and both E switch-direction
strikes, non-idle bone poses, exact pause, end-frame stop, ordinary-wall absence
of flashes, F5/F9, exact pickup/rig retirement, move34->18/turn ownership and
full-pocket refusal pass. Ringside65 starts one source camera sequence; rapid
second E inside the running ten-frame clip does not start a second sequence
(max one live flash observed), and full numeric state equals two toggles.
Flash sprites actually visible in inspected screenshots; sequence retires,
transient state doesn't serialize, live FX pause exactly and F9 clears/no replay.
Mixed fan/TV/gong/hidden storage preserves all numeric state; verified room exit
clears FX, town render inspected. Fresh15592 restores exact data, hidden gong
stays hidden, no effect replay. No-disc18072 F5 preserves room/switch/hidden/
pocket/condition/wallet state; placeholder inspected. No source/render/shader
errors or nonfinite audio samples; audible acceptance not inferred. All owned
previews closed. Actual save SHA256 unchanged from baseline. Helpers/synthetic
saves/images stay in Temp, no tests. Final check2.72s/clippy-Dwarnings5.19s and
owned rustfmt pass; default/release next. Workspace fmt still only flags Claude's
existing sound.rs/insects.rs. Gates use shared WIP tree, not validation of his task.

## Final validation so far

Corrected final check2.72s/clippy-Dwarnings5.19s/owned rustfmt/capture17.00s and
default45.04s pass; release running with original settings. No source edits
since the corrected passing gates. Shared-tree validation includes Claude's WIP;
it is not validation/publication of his insect task. Whole-workspace fmt still
flags only his existing sound.rs/insects.rs formatting. No tests run/added.
Three room hunks (wall accessor, silent interaction, effect system) and one
native export prepared/staged independently of all insect hunks. HTML85=52/6/27,
unique IDs/anchors and nine audit links verified. Actual save unchanged; previews
7780(initial refusal)/16364(corrected)/15592(fresh)/18072(no-disc) all closed.
Source shadow-alpha/global effect pool/random-stream fidelity limits remain
explicit, alongside separate Philip appearance/listening acceptance.

## Complete

Release3m05s passes at original settings. Corrected final check2.72s/clippy5.19s/
owned rustfmt/capture17.00s/default45.04s/release gates pass. Workspace fmt still
only flags Claude's unfinished sound.rs/insects.rs; left untouched. No tests
added/run, no further source changes after corrected gates. All runtime helpers/
images/synthetic saves stay in Temp, actual save unchanged and all owned previews
closed. Both roadmaps/task/audit links updated and checked. Release own claim,
publish only own source/hooks/docs/logs, preserving Claude WIP and user files.
Next candidate slot-machine animation/sound, not started; source callback has
no payout/wallet feature. Shadow-alpha/global pool/random-stream parity and
Philip visual/audio acceptance remain separate from implementation completion.
