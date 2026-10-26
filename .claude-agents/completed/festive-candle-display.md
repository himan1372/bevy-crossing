# Festive candle display

2026-10-26 codex. Philip authorizes fixing the missing seasonal furniture
model iam_yaz_candle (item3298), then isolated visual/runtime verification.

- [x] Trace native profile/model and reproduce the missing display.
- [x] Apply the smallest source-backed renderer fix; retain no-disc behavior.
- [x] Sequential Rust gates/builds and isolated candle/regression checks.
- [x] Update task/parent/roadmaps/comms, complete claim and commit/push.

No tests requested. Real save hash baseline:
C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
Preserve unrelated .gitignore and savegame.json. Qwen optional summaries or
explanations only, English each prompt; no call planned for this diagnosis.

## Root cause and implementation

Native src/furniture/ac_yaz_candle.c registers no model lists or skeleton in
iam_yaz_candle. Its custom fYC_dw draws int_yaz_candle_body_model and
int_yaz_candle_fire_model, binding segment9 to fire1/fire2 textures. Generic
fallback only looked for _model/_on/_off lists, so created an empty root and
logged missing-model warnings. Reproduced before editing in Temp Dec26 shop.

Explicit candle dispatch now mirrors fYC_dw using original profile scale,
shape, geometry, texture palettes, opaque texture-edge materials and segment9.
Both original flame images are decoded once; a four-tick loop swaps existing
handles every two 60Hz ticks. No light, switch, guessed art or per-frame asset
allocation. Shared can_render validates the same model/frame set so player
room placement now recognizes the candle. GameplaySet supplies pause gating.
No general decoder, saved schema, stock calendar or transaction changes.

## Verification

Final check34.91s, all-target warnings-denied clippy19.51s, fmt, capture1m07s
and default55.02s pass. Final release3m55s passes. No tests added/run.
Temp evidence: bevy-crossing-candle-20261026 only.

- Before: iam_yaz_candle3298 missing model, empty root. After: six mesh parts;
  seven red/black/green candles, original patterned base and seven flames
  visually inspected with neighboring festive flag and ordinary shop room.
- Eight fixed-camera flame crops form exactly two pixel groups; side-by-side
  captures visibly match source alternate flame art. Both original palettes
  decode with texture-edge transparency, no rectangular background.
- Cancel preserves all41 saved fields except position/playtime. Purchase1600
  changes80000->78400, ledger1600, exact candle stock sold and pocket3298 plus
  December ticket2C58. Other stock remains; F9 retains exact state and sold
  candle display disappears. QA first assumed wrong December ticket2C4C;
  corrected against ticket_get_item source, product behavior was correct.
- Four synthetic room candles3298..329B have correct quarter-turn transforms,
  render from front/back/edge and retain grid cells/switches on F9. Full40-field
  comparison unchanged except position/playtime. Home-door fixture initially
  had residual UI focus; Escape then source door position/facing enters normally.
- One BRP serialization warning came from querying Mesh3d on an unrelated
  entity; no missing candle warning, renderer ERROR or panic in fixed runtime.

Qwen not called: diagnosis/explanation was already clear from the short native
callback, so an optional summary would add work. Codex reviewed/validated.
Philip visual acceptance and parent retail/M4 remain open.

Final no-disc runtime retains exact four home candle IDs/rotations/grid cells, pockets/conditions, Bells, sold shop stock and ledger. Expected missing-disc warning only; no product error. Owned previews closed. Actual save hash unchanged. Full implementation and source review complete; explicit task files only staged, unrelated .gitignore/save preserved.
