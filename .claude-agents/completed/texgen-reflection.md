# G_TEXTURE_GEN reflection-map rendering (and balloon placement)

Philip authorized ("le go") the next proposal after fish/insect placement.
The app ignores G_TEXTURE_GEN: reflection-mapped lists sample their stored ST
values, so balloons render as rim slivers (completed/room-critters-balloons.md).
Port emu64 texture_gen (UVs from the model-view normal projected on the
look-at axes, then gSPTexture scale, tile shift/position and size) into the
material/shader path, verify on balloons and other affected furniture, then
re-enable balloon placement (conversion, fNBC sway and switch kick already
ported). Agent: claude. Skills read this session: shared workflow, rust,
bevy-019, module-port, gc-texture-pipeline, brp-visual-qa, save-persistence,
gc-iso-reader. No tests; Temp-only previews; real save hashed, never loaded.

## Source (emu64.c, retail GC code in ACGC-PC-Port; ac-decomp lighting identical)

- texture_gen(tex): mf = size^-1 * T(-sl/16) * shift^-1 * S(32768/texture_scale)
  * T(0.5,0.5) S(0.5,0.5) * lookAt; GX_TG_NRM with the normal matrix, which is
  the model_view stack (model matrices only: the view is in the projection).
- dl_G_TEXTURE: texture_scale = 32*65536/scale (32 when 0), so the scale term
  is gSPTexture scale / 64 texels.
- Dolphin settile shifts are copied into settile_cmds before texture_gen;
  dl_G_SETTILE_DOLPHIN resets the size origin to 0; G_SETTILESIZE Dolphin form
  stores sl/tl, the N64 form x4.
- Furniture never calls HiliteReflect; the lookAt in effect is the last
  guLookAtHilite (camera eye toward an object, Look negated): X = camera right,
  Y = camera up, FTOFRAC8 truncated s8. Port uses the rendering camera's
  right/up, quantised the same way (per-object eye-to-object direction and
  stale lookAt ordering are approximated by the camera axes).
- Balloon head: D7 0x0FA0, IA8 32x32, Dolphin shift 1 -> u = 0.977 s'.
- Strings (a/b/c): alpha TEX0*PRIM (prim 0x64), render mode c8113078 has
  AA|CVG_X_ALPHA|ALPHA_CVG_SEL, so emu64 alpha_compare requires >= 144/255:
  cut in the source as by the port's 0.5 cutout. Not changed.

## Implementation

logic graphics: GbiState.texture_scale; TileState shift_s/t, sl/tl; G_TEXTURE,
G_SETTILE (w1 shifts), G_SETTILE_DOLPHIN (w0 shifts, origin reset), new
G_SETTILESIZE handler. app: disc_gfx TexGen in PartKey (texgen + textured);
CombinerUniform.texgen (A_s, A_t, B_s, B_t) + FLAG_TEXGEN via
texgen_coefficients; shader texgen_uv; unlit texgen meshes use vertex normals
(part_mesh and skinned_part_mesh). Balloon placement re-enabled in
room_item_to_furniture.

## Verification (Temp only; no tests)

Real save hash changed between tasks: 26CD8CBC... -> C15154C0... at 17:19, one
minute after target/release was rebuilt at 17:18 (Philip playing). Every
preview ran in Temp; new baseline C15154C0... unchanged through this task.
Disc scan: ~270 REL symbols contain a D9 with G_TEXTURE_GEN (some texture-data
false positives), including all cage glass/komado, fish evw tanks, blue/antique
clocks, mirrors, cabinets, statues, lighthouse.
Preview %TEMP%/bevy-crossing-room-critters: four balloons render as solid
shiny balloons and a bunny balloon over their weights (screenshots); 16/24
balloon joints move over 3 s. Place -> F9 -> exact pickup for 0x2244 and
0x224B (0x1FF1/0x300D, rot +1). E toggles the balloon switch bit (switches[0]
65536), F9 equal, exact pickup. Room with cage, tank, blue and antique clocks,
balloon: tank glass shows a reflection streak, clock glass reflects, no
corrupt texturing. Town after F9 renders normally. No shader/WARN/ERROR lines.
Gates: check, all-target clippy -D warnings, fmt, capture 1m15s, default
1m40s, release 4m24s. No-disc not rerun: placeholder rendering does not use
N64 materials and no numeric/save code changed.

## Limits / follow-ups

Mirrors, cabinets, statues and the lighthouse are on the texgen path but not
visually compared. Exact per-object hilite eye direction and stale lookAt
ordering are approximated by the camera axes. Retail comparison (Philip) open.
