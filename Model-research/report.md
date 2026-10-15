# Animal Crossing (GameCube, 2001) — 3D model formats, version selection, disc layout

Research for writing a model extraction/rendering tool. Researched 2026-10-15.

## Summary

**Q1 — Model formats (confidence: high).** Animal Crossing GameCube does **not** use BMD and does **not** use any container called "cMP" or "SRL" — neither name appears in the decompilation or port projects (see "Could not verify"). All 3D models are **Nintendo 64 Fast3D-derived display lists** (`Vtx` vertex arrays + `Gfx` command lists) executed through a custom microcode extension set, translated at runtime to GameCube GX by an N64→GC interpreter ("emu64"). Three model families exist: (1) static single-display-list props/furniture, (2) multi-part display-list creatures (fish, insects), (3) villagers built from per-species **cKF skeletons** with one display list per body part, skinned by per-bone matrices. Textures are CI4/CI8 referenced through Dolphin-extended tile opcodes; palettes are stored N64-endian and converted at load.

**Q2 — Which version is rendered (confidence: high for selection mechanism; medium for unused-content list).** There is **no LOD system** — numbered model variants (e.g. insect `1`/`2`) are wing-flap **animation frames**, not detail levels. Runtime selection is by a 246-entry actor profile table (`actor_dlftbls`) plus 700+ object banks; the whole game is RAM-resident at boot (the disc's FST `rom_start`/`rom_end` fields are unused — consistent with the known trivia that the disc can be removed after boot). Multiple never-loaded models ship on disc: the unused squirrel `xsq`, a debug Gyroid, three test actors (`mbg`, `airplane`, `mural`), 8 fossil bases, DUMMY-shared items, a crashing `hayakawa_famicom` item, unobtainable SMB/Zelda NES-game items, and 8 unobtainable clothing items.

**Q3 — Disc locations (confidence: high).** The USA Rev 0 (GAFE01) disc FST contains **11 entries: 1 root directory and 10 files — no subdirectories at all**. There are no `/actor`, `/object`, or `/demo` directories on the disc (those are *decomp source* directories). All model data lives inside **`main.dol`** (boot executable, located via the disc header, not the FST) and **`foresta.rel.szs`** (Yaz0-compressed REL). `forest_1st.arc` / `forest_2nd.arc` are RARC (JSystem JKRAramArchive) graphics archives DMA'd into ARAM — not model storage. `famicom.arc` is a RARC holding the NES ROMs. Real content is ~26.3 MB of a 1.46 GB image (98% zero padding).

---

## Findings

### Q1. Model formats

#### (a) "cMP" / "SRL" — not attested
- No source found uses "cMP" or "SRL" for Animal Crossing GameCube models. The decompilation (`ACreTeam/ac-decomp`, ex-`Prakxo/ac-decomp`, CC0) names model symbols `*_model` (type `Gfx`), `*_v` (type `Vtx`), `cKF_*` (skeleton/animation), `aNPC_*` (NPC animation). There are no per-model file headers or magic bytes — models are C arrays linked into the executables, identified by symbol name only.
- "SRL" as a format is the **Nintendo DS ROM extension** (`.srl`), per third-party tool docs (e.g. quatric/nintoolbox lists "NDS / SRL / DSI | .nds,.srd,.dsi | All DS / DSi games"). The mission's "SRL" is most likely a confusion with the DS format; it has no meaning for AC GameCube.
- Nookipedia image filenames like `Bronto_D_PG_Model.png` use "PG Model" as a render label (PG = the GameCube title's tentative name "Population Growing"), not a file format.

#### (b) Display lists, not BMD — custom N64 microcode
- The PC-port documentation (`liquidazir/animal-crossing-vr`, `pc/DOCUMENTATION.md`) states the rendering pipeline as: game code (**N64 display lists**) → **emu64** (N64→GC display-list interpreter) → GX → GL. The game is an N64 title (`Doubutsu no Mori`) ported to GameCube; its graphics were never converted to BMD.
- Custom microcode opcodes are defined in the decomp's `include/libforest/gbi_extensions.h`:
  | Opcode | Value | Purpose |
  |---|---|---|
  | `G_TRIN` | 0x09 | triangle (custom) |
  | `G_TRIN_INDEPEND` | 0x0A | triangle, independent verts |
  | `G_SETTEXEDGEALPHA` | 0xCE | texture edge alpha |
  | `G_SETCOMBINE_NOTEV` | 0xCF | combiner, non-TEV path |
  | `G_SETCOMBINE_TEV` | 0xD0 | combiner, TEV path |
  | `G_SETTILE_DOLPHIN` | 0xD2 | Dolphin-extended tile setup |
- Triangles are emitted with 5-bit vertex indices via `gsSPNTrianglesInit_5b` / `gsSPNTriangles_5b` (a bandwidth optimization over stock F3D).
- N64 segment addressing is retained: `SOFTSPRITE_MTX_SEG` = 0x07, `ANIME_1_TXT_SEG`…`ANIME_6_TXT_SEG` = 0x08–0x0D (per-instance texture segments).
- Asset-type inference used by the decomp's `tools/converters/assetdis.py`: `*_v` → `Vtx`, `*_model`/`*_gfx` → `Gfx`, `cKF_bs_r*` → `cKF_Skeleton_R_c`, `cKF_ba_r_npc*` → `aNPC_Animation_c`, `cKF_ba_r*` → `cKF_Animation_R_c`. Related converters: `gfxdis.py` (N64 DL disassembler incl. custom opcodes), `vtxdis.py`, `pal16dis*.py`, `texture_tool.py`, `arc_tool.py` (RARC).
- Endianness/format notes for a tool writer (from `OpenCrossing-Dreamcast/kb/platform-api-dvd-aram.md`): `Vtx` records are mixed-width (s16 position + u8 color) and need a dedicated byte-swap routine (`SWAP_VTX`); CI palettes are stored N64-endian and converted via `pc_assets_pal_n64_to_gc` at load; each asset carries a swap class (NONE/U16/U32/VTX).

#### (c) Villagers vs static furniture/buildings
- **Static furniture/props** (`src/furniture/*.c`, e.g. `ac_ike_*`): one static display list per item. Example `src/data/model/act_f03_koi.c` (koi fish): `u16 *_pal[]` (CI palette), `u8 *_tex[]`, `Vtx *_a_v[]`, `Gfx *_aT_model[]`, with body-segment parts `a`/`b`/`c` and `T` = textured.
- **Insects** (`src/data/model/act_m_*.c`, e.g. `act_m_abura.c`): same pattern in translucent (`XLU`) render mode; the `1`/`2` filename suffixes are **animation frames** (e.g. `act_m_abura1_2T_model` = wings frame 1 using verts 6–13, `act_m_abura2_2T_model` = wings frame 2 using verts 14–21 of the same vertex pool) — not LOD.
- **Villagers** (`src/data/npc/model/mdl/{species}_1.c`, 72 species: ant…xsq; textures in `model/tex/`, animations in `src/data/npc/npc_1_*.c`): one shared vertex pool per species (`cat_1_v[]`) plus **one display list per body part** (`head_cat_model`, `Rarm1/2_cat_model`, `Larm1/2_cat_model`, `chest_cat_model`, `tail1_cat_model`, …). Each part DL starts with `gsSPMatrix(anime_6_model + sizeof(Mtx)*N, …)` — per-part bone matrices from the **cKF skeleton** (`cKF_Skeleton_R_c* model_skeleton` in `aNPC_draw_data_c`, `include/ac_npc.h`). Textures come from per-instance segment symbols (`anime_1_txt`…`anime_4_txt`), so eyes (8 variants), mouths (6 variants), and clothing are data-driven, not baked into the species model. All parts use `G_IM_FMT_CI` (CI4) textures, opaque mode `G_RM_AA_ZB_OPA_SURF2`.
- In short: furniture = 1 static DL; creatures = multi-part DLs with frame variants; villagers = skeleton + ~10 part DLs + per-instance texture segments.

#### (d) Model version headers / revisions
- None found. There are no per-model magic bytes or version fields — identification is purely by decomp symbol naming. Regional content differences exist instead: e.g. the unused ghost chair (`sum_gst_chair01`) is functional in Doubutsu no Mori+ but replaced by a DUMMY model in AC; `ac_mural` was added between DnM and DnM+; SMB/Zelda NES items were removed in DnM e+.

### Q2. Which model version is rendered

#### (a) Duplicate/unused model files
- No true duplicate (same model stored twice) was found. Unreferenced-data analysis (`dcasset` README) shows only 8,664,560 of the 15,640,056 decompressed `foresta.rel.szs` bytes are reachable via the PC ports' ~2,500–16,400 asset-offset table — but that span includes code and unreferenced data, not confirmed duplicate models.
- The DUMMY model is **shared** across many items (zabuton, chest, rack, unused dresser, unused monkey, modern den chair, trophies) — one model, many item IDs.

#### (b) LOD — no evidence of any LOD system
- Searched the decomp actor/draw code; no distance-based model switching found. Numbered variants are animation frames (insect wings, §Q1c). What exists is frustum **culling** (`ACTOR_STATE_NO_DRAW_WHILE_CULLED`), not LOD.

#### (c) Runtime selection: profile tables + object banks
- `src/game/m_actor_dlftbls.c`: `ACTOR_DLFTBL actor_dlftbls[]` with **246 entries** (`actor_dlftbls_num = 246`), one `MAKE_ACTOR_NULL()` entry at profile index 185 (`mAc_PROFILE_185`).
- `include/m_actor_dlftbls.h`: `rom_start`/`rom_end` are explicitly **"unused in AC"** — actors are not streamed from disc; the whole game loads into RAM at boot.
- `include/m_actor.h`: `bank_id` enum with 700+ `ACTOR_OBJ_BANK_*` entries (KEEP, BGITEM, UMBRELLA, TOOLS, ROOM_SUNSHINE, …), an `mAc_PROFILE_*` enum mirroring the profile table order, and an actor-parts enum (FG/ITEM/PLAYER/NPC/BG/EFFECT/CONTROL).
- Villager instances resolve species → `{species}_1.c` part DLs + skeleton + per-instance textures (eyes/mouth/clothing indices in `aNPC_draw_data_c`).

#### (d) Models on disc that are never loaded in normal gameplay
Per Nookipedia ("Prerelease and unused content in Animal Crossing", index fetch 2026-10-15) cross-checked against decomp filenames:
- `xsq` — unused squirrel; model ships (`src/data/npc/model/mdl/xsq_1.c`), spawnable only via memory editing.
- Debug Gyroid — unused Gyroid Assistant in test maps; corrupted textures, no animations.
- Test actors present in the actor table but only reachable via debug/test maps or unnamed droppable items: `ac_mbg` ("Move BG" yellow cube), `ac_airplane` (paper airplane), `ac_mural` (shirt-texture grid; N64 draw routine never updated → corrupted textures on GC).
- 8 fossil bases (`ac_din_*_dummy.c`: tricera/T-rex/bronto/ptera/HUTABAD/mammoth/stego/stego D2) — unobtainable furniture.
- DUMMY-shared items (zabuton, chest, rack, unused dresser/monkey, trophies, …).
- `ac_hayakawa_famicom` ("Legend of Zelda" duplicate item) — crashes AC on interaction.
- Super Mario Bros / Legend of Zelda NES-game items — fully functional, unobtainable in AC (ROMs inside `famicom.arc`).
- 8 unobtainable clothing items (plum kimono, somber robe, sweatsuits, puffy vests, summer/bamboo robes) — exist as data, worn by villagers at events.

### Q3. Disc locations

#### (a) Disc filesystem layout (FST) — GAFE01 USA Rev 0
The FST has **11 entries: 1 root directory + 10 files, zero subdirectories** (per `gabeconway/opencrossing-dreamcast`, `tools/dcasset/README.md`, index fetch 2026-10-15):

| File | Disc offset | Size (bytes) | Magic / notes |
|---|---|---|---|
| `audiorom.img` | 0x000FE588 | 8,300,384 | `D3 20 D5 78` — jaudio sound bank (largest file) |
| `foresta.rel.szs` | 0x00F27B34 | 6,137,393 | `Yaz0` → decompresses to 15,640,056 B |
| `foresta.map` | 0x00A87D3C | 4,849,144 | text — linker map (debug leftover) |
| `forest_2nd.arc` | 0x015D2508 | 4,132,608 | `RARC` — ARAM graphics archive |
| `famicom.arc` | 0x008E8CFC | 1,699,904 | `RARC` — NES ROMs |
| `forest_1st.arc` | 0x01502168 | 852,896 | `RARC` — ARAM graphics archive |
| `static.map` | 0x019C4D68 | 552,879 | text — linker map (debug leftover) |
| `opening.bnr` | 0x019C3408 | 6,496 | `BNR1` — GameCube banner |
| `COPYDATE` | 0x008E8CE8 | 19 | build date string |
| `static.str` | 0x01A4BD18 | 56 | build path string |
| `main.dol` | via disc header (not in FST) | 918,720 | 7 text + 11 data sections |

Real content ≈ 26.3 MB of a 1.46 GB image — ~98% zero padding.

#### (b) Which directories hold model data
- **None — there are no directories on the disc.** `/actor`, `/object`, `/demo` style paths exist only as *decomp source* directories (`src/actor/`, `src/data/model/`, `src/data/npc/`, …), not on the disc. A tool must not look for disc paths.
- Model data is inside `main.dol` and the Yaz0-decompressed `foresta.rel.szs` (REL), as static C arrays. The PC ports load ~2,500–16,365 assets by **absolute DOL/REL ROM offset** with a per-asset byte-swap class (`pc/tools/gen_runtime_assets.py` generates the table).

#### (c) Archive formats
- `.szs` = **Yaz0** (`Yaz0` magic, big-endian) wrapping the REL.
- `.arc` = **RARC** (`RARC` magic), i.e. JSystem `JKRArchive` / `JKRAramArchive`. The decomp's `tools/arc_tool.py` packs/unpacks them (via pyjkernel; files use `JKRPreloadType.ARAM`). `forest_1st.arc` + `forest_2nd.arc` (6.96 MB combined) are DMA'd into ARAM at boot via `JKRAramArchive`/`ARStartDMA` — they hold streamed graphics data, not the model arrays. `famicom.arc` holds NES ROMs.
- Models are stored **raw** (as linked binary blobs in DOL/REL), not inside the ARCs.

#### (d) Naming conventions (decomp source → on-disc symbol names)
- Static actors/objects: `{name}_{part}_v` (verts), `{name}_{part}T_model` (textured DL; `X` suffix variant = translucent), `{name}_pal` (CI palette), `{name}_tex` (texture); binary blobs in `assets/{name}.inc`.
- Fish: `act_f01`…`act_f40_*` (Japanese names); insects: `act_m_*` (+ `2` frame variants).
- Villagers: `src/data/npc/model/mdl/{species}_1.c` (72 species), textures in `model/tex/`, animations `src/data/npc/npc_1_*.c` (`cKF_*` types).
- Furniture: `src/furniture/ac_*.c`; fossil dummies: `ac_din_*_dummy.c`.

---

## Could not verify / open questions
1. **"cMP" format** — no source found using this name for anything in AC GameCube. If the requester has a concrete origin for the term (a specific tool or forum post), that would be needed to chase it.
2. **"SRL"** — only attested meaning is the Nintendo DS ROM extension `.srl`; almost certainly a confusion, not an AC model format.
3. **LOD** — no LOD system found in the decomp sources examined; absence of evidence, but the codebase shows culling flags instead of LOD switching.
4. **Whale model** — a community video ("Detailing the Whale") references one, but its location in the data was not verified.
5. **`forest_1st/2nd.arc` internal file listing** — confirmed they are ARAM graphics archives, but their inner file names were not enumerated.
6. **Regional binary differences** — findings are from USA Rev 0 (GAFE01) disc data and the decomp (which targets the international releases); DnM/DnM+/e+ content deltas noted where known.
7. **TCRF** (`tcrf.net/Animal_Crossing`) was blocked (403 via index fetch; Cloudflare challenge in the live browser — not bypassed per research boundaries), so its cutting-room-floor notes could not be cross-checked.

## Sources
| # | Source | URL | Read | Flag |
|---|---|---|---|---|
| 1 | dcasset README (FST table, offsets, magics, REL analysis) | https://github.com/gabeconway/opencrossing-dreamcast/blob/HEAD/tools/dcasset/README.md | 2026-10-15 | index |
| 2 | OpenCrossing-Dreamcast KB: DVD/ARAM, asset table, swap classes | https://github.com/gabeconway/opencrossing-dreamcast/blob/HEAD/kb/platform-api-dvd-aram.md | 2026-10-15 | index |
| 3 | ac-decomp (CC0): gbi_extensions.h, converters, actor tables, model sources | https://github.com/ACreTeam/ac-decomp | 2026-10-15 | index |
| 4 | ac-decomp `tools/arc_tool.py` (RARC pack/unpack) | https://github.com/ACreTeam/ac-decomp/blob/master/tools/arc_tool.py | 2026-10-15 | index |
| 5 | ac-decomp `src/data/npc/model/mdl/cat_1.c` (villager part-DL structure) | https://github.com/ACreTeam/ac-decomp/blob/master/src/data/npc/model/mdl/cat_1.c | 2026-10-15 | index |
| 6 | ac-decomp `src/data/npc/model/mdl/` listing (72 species incl. `xsq_1.c`) | https://github.com/ACreTeam/ac-decomp/tree/master/src/data/npc/model/mdl | 2026-10-15 | index |
| 7 | animal-crossing-vr `pc/DOCUMENTATION.md` (emu64 N64→GC pipeline) | https://github.com/liquidazir/animal-crossing-vr/blob/HEAD/pc/DOCUMENTATION.md | 2026-10-15 | index |
| 8 | Nookipedia: Prerelease and unused content in Animal Crossing | https://nookipedia.com/wiki/Prerelease_and_unused_content_in_Animal_Crossing | 2026-10-15 | index |
| 9 | ac-decomp `src/data/model/act_f03_koi.c`, `act_m_abura.c` (creature DL layout, animation frames) | https://github.com/ACreTeam/ac-decomp/tree/master/src/data/model | 2026-10-15 | index |

Per-source detail notes: `notes/source-dcasset-readme.md`, `notes/source-dcasset-kb-aram.md`, `notes/source-decomp-villager-models.md`, `notes/source-nookipedia-prerelease.md`.
