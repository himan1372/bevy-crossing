# Mailbox flicker

Status: implemented; Philip playtest acceptance pending.

Philip accepts ground lights and requests mailbox flicker next.

- [x] Ruled out duplicate placeholder/field-item mailbox drawing: mail.rs registers interaction position without fallback mesh in retail mode, and field_items does not draw actor-prop IDs.
- [x] Original obj_s_post_flag_model.c and obj_w_post_model.c main/front lists use G_CULL_BACK. They contain opposing, coplanar inside/outside surfaces; front lid vertices 8..15 repeat positions with opposite winding/normals, and main shell repeats interior/exterior surface positions. Shared N64Material previously disabled culling, exposing both and causing depth competition.
- [x] Added FLAG_CULL_BACK to the material pipeline key. Mailbox loader marks its lit body/front materials for back-face culling, matching both original seasonal lists. Unlit flag arm/tip explicitly omit source culling and remain double-sided. All four mailboxes share the corrected models; ordinary models and accepted ground decals unchanged.
- [x] Workspace cargo check, clippy --all-targets -- -D warnings, fmt, capture/default build pass. No automated tests added/run.
- [x] Disposable runtime visual review of all four summer mailboxes and all four winter mailboxes at actual HUD Oct 15/Dec 14 18:13-18:14. Synthetic Temp-only unread letter raises/waves mailbox 0 flag; other three down. Body/lid intact across sampled viewpoints, flags retained. No renderer/shader errors observed. This is sampled image evidence, not a frame-by-frame video flicker analysis.
- [ ] Philip playtest acceptance.

Screenshots/logs stay in Temp/bevy-crossing-mailbox-review (summer-up, summer-pair-north/south, winter-pair-north/south). The attempted winter mailbox E interaction entered the nearby house instead; winter-read.png is an interior view, not evidence of reading mail or a flag-lowering transition. No save inputs sent. Preview no longer running at final rebuild. Real save SHA256 unchanged:
0CD03EE33FE4FCFAAFF893271D79A0C6C5885C34B5C9A93D1C375350AA4D49A0.
No user settings edits or disc bytes/screenshots committed; unrelated .gitignore and savegame.json excluded.

## Philip acceptance

Restart the rebuilt game and walk around the mailboxes. Body panels/lid should retain one surface instead of flashing between inside/outside textures. The mail indicator should remain visible and keep its normal waving animation with unread mail. Full render-look and M4 stay open.
