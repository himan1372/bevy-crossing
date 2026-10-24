# Catalog preview review fixes

Philip authorized this bounded follow-up on 2026-10-24: fix the reviewed
uncommitted catalog slice, then ask before further catalog/raffle work.

- Remove unsupported page-pixel recolouring; original clg_mwin_mode copies
  TEXEL0 colour and TEXEL1 coverage without a green tint.
- Fit all furniture renderer paths using rendered hierarchy bounds, including
  skinned mesh bounds; avoid changing shared furniture spawning for the catalog.
- Match mSM_change_view's width-256 branch: 20-degree perspective, near 100,
  far 800. Preserve page scroll and selected-item rotation.
- Validate sequentially without tests; record evidence and limitations here.

Parent nook-tickets-catalog remains claimed by claude; do not steal its claim.
Purchase/refusal/delivery/raffle gameplay paths and non-furniture previews are
outside this bounded fix.

## Implementation and evidence

- Original source: m_catalog_ovl.c mCL_set_page_dl/mCL_item_move/mCL_item_draw;
  m_submenu_ovl.c mSM_change_view/mSM_tex_move; clg_win1.c clg_mwin_mode.
- Removed green_catalog_leaves and the decoded-image mutation. Kept the
  existing page-cloth material handles/scroll and separate preview camera.
- Catalog-local fitting measures all descendant mesh Aabbs in root space.
  Bevy 0.19.1 updates joint-aware bounds after transform propagation; it
  retains generated skinned bounds even for RENDER_WORLD-only meshes.
  Wait for every mesh's Aabb before revealing the preview. The incomplete
  shared LocalBounds addition is removed; furniture.rs matches HEAD.
- Camera20deg/near100/far800 matches the source width256 branch. A fitted
  sphere radius50 fits within the narrower frustum at distance330.
- Independent read-only code review found no actionable issues. Follow-up
  review confirmed the clock's conservative bounds are geometry-derived,
  not a propagation or mesh-extraction race. Exact texture-coverage-aware
  fitting is deferred; the alarm clock renders smaller/higher in the circle.
- Synthetic Temp preview PID19424, explicit original ISO, offset hour -4.
  Inspected screenshots of blue bed (index214), blue wardrobe (index2),
  retro TV (index78) and alarm clock (index154): all render/rotate. Page
  pattern moves, palette is unmodified. Escape removes catalog hierarchy.
  An initial interaction aimed at stationery; dismissed without purchase
  and approached Nook. No purchase/mail/raffle verification claimed.
- Evidence directory: %TEMP%/bevy-crossing-room-critters/
  catalog-review-b537b162a14a4522ab3f8e5c1dbf8de7. Helpers/screenshots/synthetic
  save remain there. Preview closed; stderr has no WARN/ERROR/panic entries.
- Real save SHA256 before/after runtime unchanged:
  C15154C01FE105A33229406AD7D16499D0A79F7961F5D28DFDB20B9A43562FBA.
- Check29.67s, all-target clippy -D warnings12.99s, fmt, capture56.81s,
  default27.17s pass. First default attempt hit the still-running preview's
  executable lock; closed that owned process with escalation, then retried
  successfully. Release3m03s passed. No tests added/run.
- Parent task stays open/claude-owned; both roadmaps and parent note updated
  without changing gameplay progress counts. .gitignore and real save excluded.

Bounded fixes complete; parent task and user acceptance remain open.
