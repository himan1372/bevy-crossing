# Held net retail visual fidelity

2026-10-16, codex. Planned only; no implementation or Philip acceptance.

Philip sees a net but doubts it looks like the retail version. Existing runtime
normal-net asset selection is source-backed; rendering fidelity is unresolved.

## Scope

Compare original tool selection and an unobstructed retail reference with the
port. Audit textures/palette/combiner/culling, segmentD skinning, attachment,
scale and held/swing/recovery poses before changing the demonstrated defect.

Relevant files: app/src/player_tools.rs, character.rs, skeleton.rs, disc_gfx.rs,
n64_material.rs; original m_player_lib.c, m_player_item_net.c_inc and player_tool.c.

## Acceptance and boundaries

Philip confirms normal-net shape, texture, scale and pose match the retail
reference through held idle, walking/turning and swing/recovery. Record code
checks separately from visual acceptance. Full capture/rod work remains separate.

Load original assets at runtime from Philip's own disc; never commit extracted
bytes. Keep the no-disc placeholder usable. Use source and existing translations,
not guessed layouts or gameplay rules. Claim before implementation. Run suitable
compile/lint/format/build checks for code changes; no tests unless Philip requests
them. Record implementation evidence separately from Philip's actual acceptance.
M4 remains open; title and rename remain deferred.

